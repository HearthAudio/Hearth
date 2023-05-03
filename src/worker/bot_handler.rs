use std::collections::HashMap;
use std::env;
use std::time::Duration;
use flume::{Receiver, Sender};
use futures::{SinkExt, StreamExt};
use futures::stream::SplitSink;
use log::{debug, error, info};
use reqwest::Url;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Number, Value};
use songbird::ConnectionInfo;
use songbird::id::{ChannelId, GuildId, UserId};
use songbird::model::OpCode::Heartbeat;
use tokio::net::TcpStream;
use tokio::time;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::{connect, Message};
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use crate::config::Config;

#[derive(Serialize)]
struct HeartBeatMessage {
    op: u8 // Hearbeat is opcode 1
}

#[derive(Serialize)]
struct IdentifyProperties {
    os: String,
    browser: String,
    device: String
}

#[derive(Serialize)]
struct IdentifyData {
    token: String,
    properties: IdentifyProperties,
    intents: u64
}

#[derive(Serialize)]
struct IdentifyMessage {
    op: u8, // Identifiy is opcode 2
    d: IdentifyData
}

#[derive(Serialize)]
struct ResumeData {
    token: String,
    session_id: String,
    seq: u64
}

#[derive(Serialize)]
struct ResumeMessage {
    op: u8, // Resume is op code 6
    d: ResumeData
}

pub struct VoiceConnectData {
    pub session_id: String,
    pub endpoint: String,
}

pub enum WebsocketInterconnectType {
    VoiceConnectDataResult,
    VoiceDataRequest
}

pub struct WebsocketInterconnect {
    pub voice_connect_data: VoiceConnectData,
    pub com_type: WebsocketInterconnectType
}

#[derive(Debug)]
struct IdentifyDataResult {
    session_id: String,
    resume_url: String,
    last_sequence: u64
}

enum ConnectionState {
    PreHeartBeatConfig,
    PreIdentify,
    PreResume,
    Connected
}

async fn reeval_connection_state(mut connection_state: &ConnectionState, mut writer: & mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, config: &Config,token: Option<String>,session_id: Option<String>,sequence: Option<u64>) {
    match connection_state {
        ConnectionState::PreIdentify => {
            let message = IdentifyMessage {
                op: 2,
                d: IdentifyData {
                    token: config.config.discord_bot_token.parse().unwrap(),
                    properties: IdentifyProperties {
                        os: env::consts::OS.to_string(),
                        browser: "lantern".to_string(), //TODO: Probably better to pull these from exe name or something
                        device: "lantern".to_string(),
                    },
                    intents: 8, // TODO: Replace with more limited perms and allow config
                },
            };
            let json_rep = serde_json::to_string(&message).unwrap();
            let x = Message::from(json_rep);
            writer.send(x).await.expect("Failed to send IDENT");
            connection_state = &ConnectionState::PreResume;
        },
        ConnectionState::PreResume => {
            let message = ResumeMessage {
                op: 6,
                d: ResumeData {
                    token: token.expect("No Token"),
                    session_id: session_id.expect("No Session ID"),
                    seq: sequence.expect("No Sequence"),
                }
            };
            let json_rep = serde_json::to_string(&message).unwrap();
            let x = Message::from(json_rep);
            writer.send(x).await.expect("Failed to send Resume");
        }
        _ => {}
    }
}

pub async fn initialize_websocket(config: &Config,tx: Sender<WebsocketInterconnect>,rx: Receiver<WebsocketInterconnect>) {
    let identify_result = identify(config).await.unwrap();
    resume_gateway(identify_result,config,tx,rx).await;
}

async fn resume_gateway(identify: IdentifyDataResult,config: &Config,tx: Sender<WebsocketInterconnect>,rx: Receiver<WebsocketInterconnect>) {
    // Connect to Gateway using URL
    let x = connect_async(
        Url::parse(&identify.resume_url).unwrap()
    ).await;
    match x {
        Err(e) => {
            //TODO: Retry
            println!("{}",e);
        }
        Ok((mut socket,_)) => {
            println!("Connected");
            let (heart_tx, heart_rx) : (Sender<String>,Receiver<String>) = flume::unbounded();
            let mut connection_state = ConnectionState::PreHeartBeatConfig;

            let mut interval = tokio::time::interval(Duration::from_millis(1000));
            let (mut writer, mut rec) = socket.split();

            // Echo incoming WebSocket messages and send a message periodically every second.
            loop {
                tokio::select! {
                    msg = rec.next() => {
                        match msg {
                            Some(msg) => {
                                let msg = msg.unwrap();
                                let mut lookup: Result<HashMap<String, Value>,serde_json::Error> = serde_json::from_str(&msg.to_string());
                                match lookup {
                                    Ok(lookup_v) => {
                                        match lookup_v.get("op").expect("No OP Code").as_u64().unwrap() {
                                            10 => {
                                                let interval_period =  lookup_v.get("d").unwrap().get("heartbeat_interval").unwrap().as_u64().unwrap();
                                                interval = tokio::time::interval(Duration::from_millis(interval_period));
                                                connection_state = ConnectionState::PreResume;
                                                reeval_connection_state(&connection_state, &mut writer, config, Some(config.config.discord_bot_token.to_string()), Some(identify.session_id.to_string()), Some(identify.last_sequence)).await;
                                            },
                                            1 => {
                                                    let message = HeartBeatMessage {
                                                        op: 1
                                                    };
                                                    let json_rep = serde_json::to_string(&message).unwrap();
                                                    writer.send(Message::from(json_rep)).await.expect("Failed to send HeartBeat INTERRUPT");
                                             }
                                            _ => {}
                                        }

                                    },
                                    Err(e) => {
                                        info!("Empty! {}", e);
                                    }
                                }
                            }
                            None => break,
                        }
                    }
                    _ = interval.tick() => {
                        let message = HeartBeatMessage {
                                    op: 1
                        };
                        let json_rep = serde_json::to_string(&message).unwrap();
                        writer.send(Message::Text(json_rep)).await.expect("Failed to send HeartBeat INTERVAL");
                    },
                    flume_msg = rx.recv_async() => {
                        //TODO: Replace unwrap with match
                        let parsed_msg : WebsocketInterconnect = flume_msg.unwrap();
                        match parsed_msg.com_type {
                            WebsocketInterconnectType::VoiceDataRequest => {
                                //TODO: Voice Data request
                            },
                            WebsocketInterconnectType::VoiceConnectDataResult => {}
                        }
                    }
                }
            };
        }
    };
}

async fn identify(config: &Config) -> Option<IdentifyDataResult> {
    // let connection_info = ConnectionInfo {
    //     channel_id: Some(ChannelId(voice_channel_id)),
    //     guild_id: GuildId(guild_id),
    //     user_id: UserId(config.config.discord_bot_id),
    //     token: "".to_string(),
    //     session_id: "".to_string(),
    //     endpoint: "".to_string(),
    // };
    //TODO: Build into Struct instead of Hashmap
    //TODO: Proper error handling to auto retry a few times before failing
    // Get Gateway URL
    let resp = reqwest::get("https://discord.com/api/gateway")
        .await
        .unwrap()
        .json::<HashMap<String, String>>()
        .await
        .unwrap();
    let url = resp.get("url").expect("No URL");
    // Connect to Gateway using URL
    let x = connect_async(
        Url::parse(&format!("{}?v=10&encoding=json", url)).unwrap()
    ).await;
    match x {
        Err(e) => {
            //TODO: Retry
            println!("{}",e);
            return None;
        }
        Ok((mut socket,_)) => {
            println!("Connected");
            let mut connection_state = ConnectionState::PreHeartBeatConfig;

            let mut interval = tokio::time::interval(Duration::from_millis(1000));
            let (mut writer, mut rec) = socket.split();

            // Echo incoming WebSocket messages and send a message periodically every second.
            return loop {
                tokio::select! {
                    msg = rec.next() => {
                        match msg {
                            Some(msg) => {
                                let msg = msg.unwrap();
                                let mut lookup: Result<HashMap<String, Value>,serde_json::Error> = serde_json::from_str(&msg.to_string());
                                match lookup {
                                    Ok(lookup_v) => {
                                        match lookup_v.get("op").expect("No OP Code").as_u64().unwrap() {
                                            10 => {
                                                let interval_period =  lookup_v.get("d").unwrap().get("heartbeat_interval").unwrap().as_u64().unwrap();
                                                interval = tokio::time::interval(Duration::from_millis(interval_period));
                                                connection_state = ConnectionState::PreIdentify;
                                                // println!("CONFIG");
                                                reeval_connection_state(&connection_state,&mut writer,config,None,None,None).await;
                                            },
                                            1 => {
                                                    let message = HeartBeatMessage {
                                                        op: 1
                                                    };
                                                    let json_rep = serde_json::to_string(&message).unwrap();
                                                    writer.send(Message::from(json_rep)).await.expect("Failed to send HeartBeat INTERRUPT");
                                             }
                                            0 => {
                                                println!("OPCODE1 RECV");
                                                let resume_url = lookup_v.get("d").unwrap().get("resume_gateway_url").unwrap().to_string();
                                                let session_id = lookup_v.get("d").unwrap().get("session_id").unwrap().to_string();
                                                let last_sequence = lookup_v.get("s").unwrap();
                                                writer.close().await;
                                                let result = IdentifyDataResult {
                                                    session_id: session_id,
                                                    resume_url: resume_url,
                                                    last_sequence: last_sequence.as_u64().unwrap()
                                                };
                                                break Some(result);
                                            }
                                            _ => {}
                                        }

                                    },
                                    Err(e) => {
                                        info!("Empty! {}", e);
                                    }
                                }
                            }
                            None => {},
                        }
                    }
                }
            };
        }
    };
}