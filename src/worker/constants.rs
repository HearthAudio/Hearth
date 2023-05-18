use std::time::Duration;
use crate::extensions::duration::hours_to_duration;

pub const DEFAULT_JOB_EXPIRATION_TIME : u64 = 21600; // Every 6 hours - seconds
pub const EXPIRATION_CHECK_TIME : Duration = hours_to_duration(1); // Every hour