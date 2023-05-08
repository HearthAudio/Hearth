use log::{error, info};
use songbird::{SerenityInit};
use serenity::client::Context;
use serenity::{
    async_trait,
    client::{Client, EventHandler},
    model::{gateway::Ready},
    prelude::GatewayIntents,
};
use crate::config::Config;
use crate::deco::over_1000_servers_warning;
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

    let mut client = Client::builder(&config.config.discord_bot_token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .expect("Err creating client");

    let client_data = client.data.clone();
    let server_count = client.cache_and_http.cache.guild_count();
    if server_count > 1000 {
        over_1000_servers_warning();
    }
    tokio::spawn(async move {
        let _ = client.start().await.map_err(|why| println!("Client ended: {:?}", why));
    });

    info!("Songbird INIT");

    while let Ok(msg) = ipc.receiver.recv().await {
        match msg.action_type {
            ProcessorIncomingAction::Infrastructure(Infrastructure::SongbirdInstanceRequest) => {
                let manager = songbird::get(client_data.read().await);
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
                            Err(e) => error!("Failed to send songbird instance to job: {}",&msg.job_id)
                        }
                    },
                    None => error!("Failed to get songbird instance for job: {}",&msg.job_id)
                }

            }
            _ => {}
        }
    }
    tokio::signal::ctrl_c().await.unwrap();
    println!("Received Ctrl-C, shutting down.");
}