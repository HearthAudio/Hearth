use hearth_interconnect::errors::ErrorReport;
use hearth_interconnect::messages::Message;
use log::error;
use tokio::runtime::Handle;
use crate::config::Config;
use crate::utils::generic_connector::PRODUCER;
use crate::worker::connector::send_message;

pub fn report_error(error: ErrorReport, config: &Config) {
    error!("{}",error.error);

    let mut px = PRODUCER.lock().unwrap();
    let p = px.as_mut();

    let rt = Handle::current();
    rt.block_on(send_message(&Message::ErrorReport(error),config.config.kafka_topic.as_str(),&mut p.unwrap()));
}