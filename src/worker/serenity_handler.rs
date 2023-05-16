
use log::{info};
use serenity::client::Context;
use serenity::{
    async_trait,
    client::{EventHandler},
    model::{gateway::Ready},
    prelude::GatewayIntents,
};
use songbird::{SerenityInit};
use crate::config::Config;
use crate::deco::over_servers_warning;
use crate::worker::queue_processor::{ProcessorIPC};


use std::sync::Arc;

use songbird::Songbird;



struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

pub async fn initialize_songbird(config: &Config,_ipc: &mut ProcessorIPC) -> Option<Arc<Songbird>> {

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
    let manager = songbird::get(client_data.read().await).await;
    return manager;
}