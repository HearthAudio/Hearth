use std::sync::Arc;
use log::{error, info};
use serenity::client::Context;
use serenity::{
    async_trait,
    client::{Client, EventHandler},
    model::{gateway::Ready},
    prelude::GatewayIntents,
};
use serenity::client::ClientBuilder;
use serenity::prelude::TypeMapKey;
use songbird::{Songbird};
use crate::config::Config;
use crate::deco::over_1000_servers_warning;
use crate::worker::queue_processor::{Infrastructure, ProcessorIncomingAction, ProcessorIPC, ProcessorIPCData};
use songbird::Config as SongbirdConfig;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

/// Zero-size type used to retrieve the registered [`Songbird`] instance
/// from serenity's inner [`TypeMap`].
///
/// [`Songbird`]: Songbird
/// [`TypeMap`]: serenity::prelude::TypeMap
pub struct SongbirdKey;

impl TypeMapKey for SongbirdKey {
    type Value = Arc<Songbird>;
}

/// Installs a new songbird instance into the serenity client.
///
/// This should be called after any uses of `ClientBuilder::type_map`.
pub fn register(client_builder: ClientBuilder) -> ClientBuilder {
    let voice = Songbird::serenity();
    register_with(client_builder, voice)
}

/// Installs a given songbird instance into the serenity client.
///
/// This should be called after any uses of `ClientBuilder::type_map`.
pub fn register_with(client_builder: ClientBuilder, voice: Arc<Songbird>) -> ClientBuilder {
    client_builder
        .voice_manager_arc(voice.clone())
        .type_map_insert::<SongbirdKey>(voice)
}

/// Retrieve the Songbird voice client from a serenity context's
/// shared key-value store.
pub async fn get(ctx: &Context) -> Option<Arc<Songbird>> {
    let data = ctx.data.read().await;

    data.get::<SongbirdKey>().cloned()
}

/// Helper trait to add installation/creation methods to serenity's
/// `ClientBuilder`.
///
/// These install the client to receive gateway voice events, and
/// store an easily accessible reference to Songbird's managers.
pub trait SerenityInit {
    /// Registers a new Songbird voice system with serenity, storing it for easy
    /// access via [`get`].
    ///
    /// [`get`]: get
    #[must_use]
    fn register_songbird(self) -> Self;
    /// Registers a given Songbird voice system with serenity, as above.
    #[must_use]
    fn register_songbird_with(self, voice: Arc<Songbird>) -> Self;
}

impl SerenityInit for ClientBuilder {
    fn register_songbird(self) -> Self {
        register(self)
    }

    fn register_songbird_with(self, voice: Arc<Songbird>) -> Self {
        register_with(self, voice)
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