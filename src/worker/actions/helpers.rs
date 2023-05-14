use std::sync::{Arc};

use snafu::{OptionExt, ResultExt, Whatever};
use songbird::{Call, Songbird};
use songbird::id::GuildId;

pub async fn get_manager_call(guild_id: &String, manager: &mut Option<Arc<Songbird>>) -> Result<Arc<tokio::sync::Mutex<Call>>,Whatever> {
   let h = manager.as_mut()
       .with_whatever_context(|| "Manager not initialized")?.get(GuildId(guild_id.parse()
       .with_whatever_context(|e| format!("Failed to parse Guild ID to u64 with error: {}", e))?))
       .with_whatever_context(|| "Failed to retrieve manager Call")?;
    return Ok(h);
}

#[macro_export]
macro_rules! error_report {
    ($x: expr,$rid: expr,$job_id: expr,$config: expr) => {
        {
            use crate::errors::report_error;
            match $x {
                Ok(t) => {
                    Some(t)
                },
                Err(e) => {
                    report_error(ErrorReport {
                        error: e.to_string(),
                        request_id: $rid,
                        job_id: $job_id
                    },$config);
                    None
                }
            }
        }
    }
}