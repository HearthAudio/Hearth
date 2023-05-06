



use log::info;
use songbird::{SerenityInit};
use serenity::client::Context;
use serenity::{
    async_trait,
    client::{Client, EventHandler},
    model::{gateway::Ready},
    prelude::GatewayIntents,
};




use crate::config::Config;
use crate::worker::queue_processor::{Infrastructure, ProcessorIncomingAction, ProcessorIPC, ProcessorIPCData};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

pub async fn initialize_songbird(config: &Config,ipc: &mut ProcessorIPC) {

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&config.config.discord_bot_token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .expect("Err creating client");

    let client_data = client.data.clone();
    tokio::spawn(async move {
        let _ = client.start().await.map_err(|why| println!("Client ended: {:?}", why));
    });
    info!("Songbird INIT");



    while let Ok(msg) = ipc.receiver.recv().await {
        match msg.action_type {
            ProcessorIncomingAction::Infrastructure(Infrastructure::SongbirdInstanceRequest) => {
                //TODO: Do we need to clone data over here?
                let manager = songbird::get(client_data.read().await)
                    .expect("Songbird Voice client placed in at initialisation.").clone();
                //TODO: Match here
                ipc.sender.send(ProcessorIPCData {
                    action_type: ProcessorIncomingAction::Infrastructure(Infrastructure::SongbirdIncoming),
                    songbird: Some(manager),
                    dwc: None,
                }).expect("Failed to send Songbird result");
            },
            _ => {}
        }
    }
    tokio::signal::ctrl_c().await.unwrap();
    println!("Received Ctrl-C, shutting down.");
}