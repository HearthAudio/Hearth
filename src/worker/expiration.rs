use std::sync::Arc;

use crate::worker::constants::EXPIRATION_CHECK_TIME;
use crate::worker::queue_processor::{
    Infrastructure, JobID, ProcessorIPCData, ProcessorIncomingAction,
};
use log::{error, info};
use tokio::sync::broadcast::Sender;
use tokio::{task, time};

pub fn init_expiration_timer(s: Arc<Sender<ProcessorIPCData>>) {
    let _expiration_timer = task::spawn(async move {
        let mut interval = time::interval(EXPIRATION_CHECK_TIME);

        loop {
            interval.tick().await;
            let x = s.send(ProcessorIPCData {
                action_type: ProcessorIncomingAction::Infrastructure(Infrastructure::CheckTime),
                songbird: None,
                dwc: None,
                error_report: None,
                job_id: JobID::Global(),
            });
            match x {
                Ok(_) => {
                    info!("Sent expiration check for jobs")
                }
                Err(e) => {
                    error!("Failed to send expiration check with error: {}", e)
                }
            }
        }
    });
}
