
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
use crate::worker::queue_processor::{ProcessorIPC};
use songbird::Config as SongbirdConfig;

use std::sync::Arc;
use songbird::driver::{CryptoMode, MixMode};

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
        .expect("Failed to register Songbird")
        ;

    let client_data = client.data.clone();
    let server_count = client.cache.guild_count();

    tokio::spawn(async move {
        let _ = client.start_autosharded().await.map_err(|why| println!("Client ended: {:?}", why));
    });

    info!("Songbird INIT");
    let mut manager = songbird::get(client_data.read().await).await;
    let mut config = SongbirdConfig::default();
    config.use_softclip = false; // Disable soft clip as Hearth only allows one audio source to play at a time. So this should result in a marginal performance improvement
    manager.as_mut().unwrap().set_config(config);
    return manager;
}