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
use crate::IPCWebsocketConnector;

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

#[derive(Serialize,Debug)]
struct ResumeData {
    token: String,
    session_id: String,
    seq: u64
}

#[derive(Serialize,Debug)]
struct ResumeMessage {
    op: u8, // Resume is op code 6
    d: ResumeData
}

#[derive(Serialize,Debug)]
pub struct VoiceDataRequestData {
    pub guild_id: String,
    pub channel_id: String,
    pub self_mute: bool,
    pub self_deaf: bool
}

#[derive(Serialize,Debug)]
struct VoiceDataRequest {
    op: u8,
    d: VoiceDataRequestData
}
#[derive(Debug)]
pub struct VoiceConnectData {
    pub session_id: String,
    pub endpoint: String,
}

#[derive(Debug)]
pub enum WebsocketInterconnectType {
    VoiceConnectDataResult,
    VoiceDataRequest
}

#[derive(Debug)]
pub struct WebsocketInterconnect {
    pub voice_connect_data: Option<VoiceConnectData>,
    pub voice_connect_request: Option<VoiceDataRequestData>,
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
            println!("Sending Resume Message: {:?}",message);
            let json_rep = serde_json::to_string(&message).unwrap();
            let x = Message::from(json_rep);
            writer.send(x).await.expect("Failed to send Resume");
        }
        _ => {}
    }
}

pub async fn initialize_websocket(config: &Config,ipc: IPCWebsocketConnector) {
    identify(config,ipc).await;
}

async fn handle_voice_req_result(ipc: &IPCWebsocketConnector,endpoint : &Option<String>,session_id : &Option<String>) {
        ipc.voice_data_response_tx.send(WebsocketInterconnect {
            com_type: WebsocketInterconnectType::VoiceConnectDataResult,
            voice_connect_data: Some(VoiceConnectData {
                endpoint: endpoint.clone().unwrap().to_string(),
                session_id: session_id.clone().unwrap().to_string()
            }),
            voice_connect_request: None
        }).unwrap();
}
//TODO: Dead Code We need to add support for disconnecting https://discord.com/developers/docs/topics/gateway#disconnecting
//TODO: A bunch of crappy duplicated code
//TODO: This is pretty much dead for now
async fn resume_gateway(identify: IdentifyDataResult,config: &Config,ipc: IPCWebsocketConnector) {
    // Connect to Gateway using URL
    let url = identify.resume_url.replace("\"",""); // For some reason resume_url comes with qoute marks around it
    let x = connect_async(
        Url::parse(&url).unwrap()
    ).await;
    match x {
        Err(e) => {
            //TODO: Retry
            println!("{}",e);
        }
        Ok((mut socket,_)) => {
            println!("Connected");
            let mut connection_state = ConnectionState::PreHeartBeatConfig;
            let mut voice_state_update = false;
            let mut voice_server_update = false;
            let mut endpoint : Option<String> = None;
            let mut session_id : Option<String> = None;

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
                                println!("PLKVAL {:?}",msg);
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
                                             },
                                            0 => {
                                                println!("ZERO MSG:");
                                                println!("{:?}",lookup_v);
                                                match lookup_v.get("t").expect("NO T").as_str().unwrap() {
                                                    "VOICE_SERVER_UPDATE" => {
                                                        voice_server_update = true;
                                                        endpoint = Some(lookup_v.get("d").unwrap().get("endpoint").unwrap().to_string());
                                                        if voice_state_update && voice_server_update {
                                                             handle_voice_req_result(&ipc,&endpoint,&session_id);
                                                        }
                                                    },
                                                    "VOICE_STATE_UPDATE" => {
                                                        voice_state_update = true;
                                                        session_id = Some(lookup_v.get("d").unwrap().get("session_id").unwrap().to_string());
                                                        if voice_state_update && voice_server_update {
                                                             handle_voice_req_result(&ipc,&endpoint,&session_id);
                                                        }
                                                    },
                                                    _ => {}
                                                }
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
                        println!("INT");
                        //TODO: Fix hearbeats
                        // writer.send(Message::Text(json_rep)).await.expect("Failed to send HeartBeat INTERVAL");
                    },
                    flume_msg = ipc.voice_data_request_rx.recv_async() => {
                        println!("WH Hook Recieved");
                        //TODO: Replace unwrap with match
                        let parsed_msg : WebsocketInterconnect = flume_msg.unwrap();
                        println!("RECV WH {:?}",parsed_msg);
                        match parsed_msg.com_type {
                            WebsocketInterconnectType::VoiceDataRequest => {
                                let connect_request = parsed_msg.voice_connect_request.unwrap();
                                let voice_data_request = VoiceDataRequest {
                                    op: 4,
                                    d: connect_request
                                };
                                let json_rep = serde_json::to_string(&voice_data_request).unwrap();
                                writer.send(Message::from(json_rep)).await.expect("Failed to send Voice Data Request");
                                println!("SENT REQ");
                            },
                            WebsocketInterconnectType::VoiceConnectDataResult => {}
                        }
                    }
                }
            };
        }
    };
}

async fn identify(config: &Config,ipc: IPCWebsocketConnector) {
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
    println!("{}",url);
    // Connect to Gateway using URL
    let x = connect_async(
        Url::parse(&format!("{}?v=10&encoding=json", url)).unwrap()
    ).await;
    match x {
        Err(e) => {
            //TODO: Retry
            println!("{}",e);
        }
        Ok((mut socket,_)) => {
            println!("Connected");
            let mut connection_state = ConnectionState::PreHeartBeatConfig;

            let mut interval = tokio::time::interval(Duration::from_millis(1000));
            let mut voice_state_update = false;
            let mut voice_server_update = false;
            let mut endpoint : Option<String> = None;
            let mut session_id : Option<String> = None;
            let (mut writer, mut rec) = socket.split();

            // Echo incoming WebSocket messages and send a message periodically every second.
            loop {
                tokio::select! {
                    msg = rec.next() => {
                        match msg {
                            Some(msg) => {
                                let msg = msg.unwrap();
                                let mut lookup: Result<HashMap<String, Value>,serde_json::Error> = serde_json::from_str(&msg.to_string());
                                println!("{:?}",msg);
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
                                                match lookup_v.get("t").expect("NO T").as_str().unwrap() {
                                                    "VOICE_SERVER_UPDATE" => {
                                                        voice_server_update = true;
                                                        println!("VSEU");
                                                        endpoint = Some(lookup_v.get("d").unwrap().get("endpoint").unwrap().to_string());
                                                        if voice_state_update && voice_server_update {
                                                             handle_voice_req_result(&ipc,&endpoint,&session_id).await;
                                                        }
                                                    },
                                                    "VOICE_STATE_UPDATE" => {
                                                        voice_state_update = true;
                                                        println!("VSTU");
                                                        session_id = Some(lookup_v.get("d").unwrap().get("session_id").unwrap().to_string());
                                                        if voice_state_update && voice_server_update {
                                                             handle_voice_req_result(&ipc,&endpoint,&session_id).await;
                                                        }
                                                    },
                                                    "READY" => {
                                                        println!("READY!");
                                                        let resume_url = lookup_v.get("d").unwrap().get("resume_gateway_url").unwrap().to_string();
                                                        let session_id = lookup_v.get("d").unwrap().get("session_id").unwrap().to_string();
                                                        let last_sequence = lookup_v.get("s").unwrap();
                                                    }
                                                    _ => {}
                                                }
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
                    },
                    flume_msg = ipc.voice_data_request_rx.recv_async() => {
                        println!("WH Hook Recieved");
                        //TODO: Replace unwrap with match
                        let parsed_msg : WebsocketInterconnect = flume_msg.unwrap();
                        println!("RECV WH {:?}",parsed_msg);
                        match parsed_msg.com_type {
                            WebsocketInterconnectType::VoiceDataRequest => {
                                let connect_request = parsed_msg.voice_connect_request.unwrap();
                                let voice_data_request = VoiceDataRequest {
                                    op: 4,
                                    d: connect_request
                                };
                                let json_rep = serde_json::to_string(&voice_data_request).unwrap();
                                writer.send(Message::from(json_rep)).await.expect("Failed to send Voice Data Request");
                                println!("SENT REQ {:?}",voice_data_request);
                            },
                            WebsocketInterconnectType::VoiceConnectDataResult => {}
                        }
                    },
                    _ = interval.tick() => {
                        let message = HeartBeatMessage {
                                    op: 1
                        };
                        let json_rep = serde_json::to_string(&message).unwrap();
                        println!("INT");
                        //TODO: Fix hearbeats
                        // writer.send(Message::Text(json_rep)).await.expect("Failed to send HeartBeat INTERVAL");
                    },
                }
            };
        }
    };
}