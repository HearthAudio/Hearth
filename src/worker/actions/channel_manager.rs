use std::sync::Arc;
use serenity::model::id::GuildId;
use songbird::Songbird;
use crate::utils::generic_connector::DirectWorkerCommunication;

pub async fn leave_channel(dwc: &DirectWorkerCommunication, mut manager: &mut Option<Arc<Songbird>>) {
    manager.as_mut().unwrap().remove(GuildId(dwc.guild_id.as_ref().unwrap().parse().unwrap())).await.unwrap();
}