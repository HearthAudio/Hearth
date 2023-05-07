use std::sync::{Arc, Mutex};
use serenity::model::id::GuildId;
use snafu::{OptionExt, ResultExt, Whatever};
use songbird::{Call, Songbird};

pub async fn get_manager_call(guild_id: &String, manager: &mut Option<Arc<Songbird>>) -> Result<Arc<tokio::sync::Mutex<Call>>,Whatever> {
   let h = manager.as_mut()
       .with_whatever_context(|| "Manager not initialized")?.get(GuildId(guild_id.parse()
       .with_whatever_context(|e| format!("Failed to parse Guild ID to u64 with error: {}", e))?))
       .with_whatever_context(|| "Failed to retrieve manager Call")?;
    return Ok(h);
}