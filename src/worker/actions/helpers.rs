use std::sync::{Arc};
use songbird::{Call, Songbird};
use songbird::id::GuildId;
use anyhow::{Context, Result};

pub async fn get_manager_call(guild_id: &str, manager: &mut Option<Arc<Songbird>>) -> Result<Arc<tokio::sync::Mutex<Call>>> {
   let h = manager.as_mut()
       .context("Manager not initialized")?.get(GuildId(guild_id.parse()
       .context("Failed to parse Guild ID to u64")?))
       .context("Failed to retrieve manager Call")?;
    return Ok(h);
}

#[macro_export]
macro_rules! error_report {
    ($x: expr,$rid: expr,$job_id: expr,$guild_id: expr, $config: expr) => {
        {
            use $crate::errors::report_error;
            match $x {
                Ok(t) => {
                    Some(t)
                },
                Err(e) => {
                    report_error(ErrorReport {
                        error: e.to_string(),
                        request_id: $rid,
                        job_id: $job_id,
                        guild_id: $guild_id
                    },$config);
                    None
                }
            }
        }
    }
}