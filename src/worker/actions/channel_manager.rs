use std::sync::Arc;
use songbird::id::GuildId;
use songbird::Songbird;
use crate::utils::generic_connector::DirectWorkerCommunication;

pub async fn leave_channel(dwc: &DirectWorkerCommunication, manager: &mut Option<Arc<Songbird>>) {
    manager.as_mut().unwrap().remove(GuildId(dwc.guild_id.as_ref().unwrap().parse().unwrap())).await.unwrap();
}