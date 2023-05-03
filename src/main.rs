use std::future;
use std::future::Future;
use std::pin::Pin;
use flume::{Receiver, Sender};

use futures::{FutureExt, join};
use futures::future::{join_all, lazy};
use log::info;
use tokio::runtime::{Builder, Handle};
use tokio::task::{JoinHandle, JoinSet};

use crate::config::*;
use crate::deco::print_intro;
use crate::logger::setup_logger;
use crate::scheduler::*;
use crate::worker::*;
use crate::worker::webhook_handler::{initialize_websocket, VoiceConnectData, VoiceDataRequestData, WebsocketInterconnect, WebsocketInterconnectType};

mod config;

mod scheduler;

mod worker;

mod logger;

mod utils;

mod deco;

#[derive(Clone)]
pub struct IPCWebsocketConnector {
    pub voice_data_response_tx: Sender<WebsocketInterconnect>,
    pub voice_data_response_rx : Receiver<WebsocketInterconnect>,
    pub voice_data_request_tx: Sender<WebsocketInterconnect>,
    pub voice_data_request_rx: Receiver<WebsocketInterconnect>
}

// This is a bit of a hack to get around annoying type issues
async fn initialize_scheduler_internal(config: Config) {
    initialize_scheduler(config).await;
}

async fn initialize_worker_internal(config: Config,ipc: IPCWebsocketConnector) {
    initialize_worker(config,ipc).await;
}

//TODO Add fern as logging library instead of env_logger
#[tokio::main]
async fn main() {
    //TODO: Make these bounded
    let (voice_data_response_tx,voice_data_response_rx) : (Sender<WebsocketInterconnect>,Receiver<WebsocketInterconnect>) = flume::unbounded();
    let (voice_data_request_tx,voice_data_request_rx) : (Sender<WebsocketInterconnect>,Receiver<WebsocketInterconnect>) = flume::unbounded();
    let websocket_connector = IPCWebsocketConnector {
        voice_data_response_tx,
        voice_data_response_rx,
        voice_data_request_tx,
        voice_data_request_rx,
    };
    let worker_websocket_connector = websocket_connector.clone();
    print_intro();
    // Load config
    let worker_config = init_config();
    let scheduler_config = worker_config.clone();
    let websocket_config = worker_config.clone();
    // Setup logger
    setup_logger();
    // Depending on roles initialize worker and or scheduler on separate threads
    let mut futures = vec![];
    if worker_config.roles.worker {
        let worker = tokio::spawn(async move {
            return initialize_worker_internal(worker_config,worker_websocket_connector).await;
        });
        futures.push(worker);
    }
    if scheduler_config.roles.scheduler {
        let scheduler = tokio::spawn(async move {
            return initialize_scheduler_internal(scheduler_config).await;
        });
        futures.push(scheduler);
    }
    let websocket = tokio::task::spawn(async move {
        // initialize_bot_instance(&worker_config).await;
        initialize_websocket(&websocket_config,websocket_connector).await;
    });
    futures.push(websocket);
    //
    // let test_1 = tokio::task::spawn(async move {
    //     // initialize_bot_instance(&worker_config).await;
    //     tx.send_async(WebsocketInterconnect {
    //         com_type: WebsocketInterconnectType::VoiceDataRequest,
    //         voice_connect_data: None,
    //         voice_connect_request: Some(VoiceDataRequestData {
    //             guild_id: "43893854398".to,
    //             channel_id: "92949943".to_string(),
    //             self_mute: false,
    //             self_deaf: false,
    //         })
    //     }).await.unwrap();
    // });
    // futures.push(test_1);
    //
    //
    // let test_2 = tokio::task::spawn(async move {
    //     while let Ok(msg) = rx.recv_async().await {
    //         println!("Received: {:?}", msg);
    //     }
    // });
    // futures.push(test_2);

    futures::future::join_all(futures).await;
}