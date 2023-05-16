use hearth_interconnect::errors::ErrorReport;
use hearth_interconnect::messages::Message;
use log::error;
use tokio::runtime::Handle;
use crate::config::Config;
use crate::utils::generic_connector::PRODUCER;
use crate::worker::connector::send_message;

pub fn report_error(error: ErrorReport, config: &Config) {
    error!("{}",error.error);

    let t_config = config.clone();

    tokio::task::spawn(async move {
        let mut px = PRODUCER.lock().await;
        let p = px.as_mut();

        send_message(&Message::ErrorReport(error),t_config.config.kafka_topic.as_str(),&mut p.unwrap()).await;
    });
}