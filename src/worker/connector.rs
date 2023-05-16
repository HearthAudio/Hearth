use std::sync::Arc;
use std::thread;
// use std::thread::sleep;
use std::time::Duration;

use hearth_interconnect::messages::{ExternalQueueJobResponse, Message, PingPongResponse};




use kafka::producer::Producer;

use log::{error, info};


use snafu::{Whatever};
use songbird::Songbird;


use tokio::runtime::Builder;
use tokio::time::sleep;


use crate::config::Config;
use crate::utils::generic_connector::{initialize_kafka_config, initialize_producer, PRODUCER, send_message_generic};
// Internal connector
use crate::utils::initialize_consume_generic;
use crate::worker::actions::channel_manager::join_channel;
use crate::worker::errors::report_error;
use crate::worker::queue_processor::{JobID, process_job, ProcessorIncomingAction, ProcessorIPC, ProcessorIPCData};


pub async fn initialize_api(config: &Config, ipc: &mut ProcessorIPC,mut songbird: Option<Arc<Songbird>>) {
    let broker = config.config.kafka_uri.to_owned();
    let x = config.clone();
    // thread::spawn(move || {
    //     let rt = Builder::new_current_thread()
    //         .enable_all()
    //         .build()
    //         .unwrap();
    //     // rt.block_on(process_job(job, &proc_config, sender,report_error,songbird));
    //     println!("SW");
    //     sleep(Duration::from_secs(1));
    //     println!("EXEC J");
    //     rt.block_on(join_channel("1103424891329445989".to_string(),"1103424892541607939".to_string(),"123".to_string(),"987".to_string(),&mut songbird, report_error, x)).unwrap();
    // });

    // tokio::spawn(async move {
    //     println!("X");
    //     sleep(Duration::from_millis(500)).await;
    //     println!("XS");
    //     join_channel("1103424891329445989".to_string(),"1103424892541607939".to_string(),"123".to_string(),"987".to_string(),&mut songbird, report_error, x).await.unwrap();
    // }).await.unwrap();

    initialize_worker_consume(vec![broker],config,ipc,songbird).await;
}

fn parse_message_callback(message: Message, _producer: &PRODUCER, config: &Config, ipc: &mut ProcessorIPC,mut songbird: Option<Arc<Songbird>>) -> Result<(),Whatever> {

    match message {
        Message::DirectWorkerCommunication(dwc) => {
            if &dwc.worker_id == config.config.worker_id.as_ref().unwrap() {
                let job_id = dwc.job_id.clone();
                let result = ipc.sender.send(ProcessorIPCData {
                    action_type: ProcessorIncomingAction::Actions(dwc.action_type.clone()),
                    songbird: None,
                    job_id: JobID::Specific(job_id.clone()),
                    dwc: Some(dwc),
                    error_report: None
                });
                match result {
                    Ok(_) => {},
                    Err(_e) => error!("Failed to send DWC to job: {}",&job_id),
                }
            }
        },
        Message::InternalPingPongRequest => {
            let mut px = PRODUCER.lock().unwrap();
            let p = px.as_mut();
            send_message(&Message::InternalPongResponse(PingPongResponse {
                worker_id: config.config.worker_id.clone().unwrap()
            }),config.config.kafka_topic.as_str(), &mut *p.unwrap());
        }
        Message::InternalWorkerQueueJob(job) => {
            if &job.worker_id == config.config.worker_id.as_ref().unwrap() {
                let proc_config = config.clone();
                let job_id = job.job_id.clone();
                // This is a bit of a hack try and replace with tokio. Issue: Tokio task not executing when spawned inside another tokio task
                // rt.block_on(process_job(parsed_message, &proc_config, ipc.sender));
                // let sender = ipc.sender;
                let sender = ipc.sender.clone();
                info!("Starting new worker");

                tokio::spawn(async move {
                    println!("X");
                    sleep(Duration::from_millis(500)).await;
                    println!("XS");
                    join_channel("1103424891329445989".to_string(),"1103424892541607939".to_string(),"123".to_string(),"987".to_string(),&mut songbird, report_error, proc_config).await.unwrap();
                });

                // println!("IXX");
                // let x = config.clone();
                // tokio::spawn(async move {
                //     // sleep(Duration::from_millis(500)).await;
                //     println!("JOINT");
                //     join_channel("1103424891329445989".to_string(),"1103424892541607939".to_string(),"123".to_string(),"987".to_string(),&mut songbird, report_error, x).await.unwrap();
                // });

                // thread::spawn(move || {
                //     let rt = Builder::new_multi_thread()
                //         .worker_threads(4)
                //         .enable_all()
                //         .build()
                //         .unwrap();
                //     // rt.block_on(process_job(job, &proc_config, sender,report_error,songbird));
                //     println!("SW");
                //     sleep(Duration::from_secs(1));
                //     println!("EXEC J");
                //     rt.block_on(join_channel("1103424891329445989".to_string(),"1103424892541607939".to_string(),"123".to_string(),"987".to_string(),&mut songbird, report_error, proc_config)).unwrap();
                // });

                let mut px = PRODUCER.lock().unwrap();
                let p = px.as_mut();

                send_message(&Message::ExternalQueueJobResponse(ExternalQueueJobResponse {
                    job_id,
                    worker_id: config.config.worker_id.as_ref().unwrap().clone(),
                }), config.config.kafka_topic.as_str(), &mut *p.unwrap());
            }
        }
        _ => {}
    }
    Ok(())
}


pub async fn initialize_worker_consume(brokers: Vec<String>, config: &Config, ipc: &mut ProcessorIPC,mut songbird: Option<Arc<Songbird>>) {
    let producer : Producer = initialize_producer(initialize_kafka_config(&brokers));
    *PRODUCER.lock().unwrap() = Some(producer);

    initialize_consume_generic(brokers, config, parse_message_callback, ipc,&PRODUCER,initialized_callback,songbird);
}

fn initialized_callback(_: &Config) {}

pub fn send_message(message: &Message, topic: &str, producer: &mut Producer) {
    send_message_generic(message,topic,producer);
}