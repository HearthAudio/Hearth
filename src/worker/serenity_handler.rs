use std::time::Duration;
use log::{error, info, warn};
use serenity::client::Context;
use serenity::{
    async_trait,
    client::{EventHandler},
    model::{gateway::Ready},
    prelude::GatewayIntents,
};
use songbird::{SerenityInit};
use tokio::time::sleep;
use crate::config::Config;
use crate::deco::over_servers_warning;
use crate::worker::queue_processor::{Infrastructure, ProcessorIncomingAction, ProcessorIPC, ProcessorIPCData};


struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

pub async fn initialize_songbird(config: &Config,ipc: &mut ProcessorIPC) {

    let intents = GatewayIntents::non_privileged();


    let mut client = serenity::Client::builder(&config.config.discord_bot_token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .unwrap()
        ;
    // let client = register(client_b);

    let client_data = client.data.clone();
    let server_count = client.cache.guild_count();
    if server_count > 5000 {
        over_servers_warning();
    }
    tokio::spawn(async move {
        let _ = client.start().await.map_err(|why| println!("Client ended: {:?}", why));
    });

    info!("Songbird INIT");

    while let Ok(msg) = ipc.receiver.recv().await {
        match msg.action_type {
            ProcessorIncomingAction::Infrastructure(Infrastructure::SongbirdInstanceRequest) => {
                sleep(Duration::from_millis(250)).await;
                let manager = songbird::get(client_data.read().await).await;
                match manager {
                    Some(manager) => {
                        let result = ipc.sender.send(ProcessorIPCData {
                            action_type: ProcessorIncomingAction::Infrastructure(Infrastructure::SongbirdIncoming),
                            songbird: Some(manager),
                            dwc: None,
                            job_id: msg.job_id.clone(),
                            error_report: None,
                        });
                        match result {
                            Ok(_) => {},
                            Err(_e) => error!("Failed to send songbird instance to job: {}",&msg.job_id.to_string())
                        }
                    },
                    None => error!("Failed to get songbird instance for job: {}",&msg.job_id.to_string())
                }

            }
            _ => {}
        }
    }
    tokio::signal::ctrl_c().await.unwrap();
    warn!("Received Ctrl-C, shutting down.");
}