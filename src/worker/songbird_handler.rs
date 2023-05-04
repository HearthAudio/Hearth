use std::env;
use std::sync::Arc;
use std::time::Duration;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use log::info;
use songbird::{SerenityInit, Songbird};
use serenity::client::Context;
use serenity::{
    async_trait,
    client::{Client, EventHandler},
    model::{channel::Message, gateway::Ready},
    prelude::GatewayIntents,
    Result as SerenityResult,
};
use serenity::client::bridge::gateway::ShardMessenger;
use serenity::gateway::InterMessage;
use songbird::id::{ChannelId, GuildId};
use tokio::time::sleep;
use crate::config::Config;
use crate::worker::queue_processor::{LeaveAction, ProcessorIncomingAction, ProcessorIPC, ProcessorIPCData};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}


pub struct SongbirdRequestData {
    pub guild_id: u64,
    pub channel_id: u64
}


pub async fn initialize_songbird(config: &Config,ipc: &mut ProcessorIPC) {

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&config.config.discord_bot_token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .expect("Err creating client");

    //TODO: This is a bit of a hack create a forked version of songbird that just takes in data
    // This works because only data is used because Songbird actually only needs data but still takes the full context
    let (tx,rx) : (UnboundedSender<InterMessage>,UnboundedReceiver<InterMessage>) = futures::channel::mpsc::unbounded();
    let http = client.cache_and_http.http.clone();
    let cache =  client.cache_and_http.cache.clone();
    let data = client.data.clone();
    tokio::spawn(async move {
        let _ = client.start().await.map_err(|why| println!("Client ended: {:?}", why));
    });
    info!("Songbird INIT");

    let b_context = Context {
        data: data,
        shard: ShardMessenger::new(tx),
        shard_id: 0,
        http: http,
        cache:cache,
    };

    while let Ok(msg) = ipc.receiver.recv().await {
        match msg.action {
            ProcessorIncomingAction::SongbirdInstanceRequest => {
                //TODO: Do we need to clone data over here?
                let manager = songbird::get(&b_context).await
                    .expect("Songbird Voice client placed in at initialisation.").clone();
                //TODO: Match here
                ipc.sender.send(ProcessorIPCData {
                    action: ProcessorIncomingAction::SongbirdIncoming,
                    songbird: Some(manager),
                    leave_action: None,
                    job_id: msg.job_id
                }).expect("Failed to send Songbird result");
            },
            _ => {}
        }
    }
    tokio::signal::ctrl_c().await;
    println!("Received Ctrl-C, shutting down.");
}