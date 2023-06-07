use crate::extensions::duration::hours_to_duration;
use std::time::Duration;

pub const DEFAULT_JOB_EXPIRATION_TIME: u64 = 21600; // Every 6 hours - seconds
pub const DEFAULT_JOB_EXPIRATION_TIME_NOT_PLAYING: u64 = 3600; // Every 1 hour - seconds
pub const EXPIRATION_CHECK_TIME: Duration = hours_to_duration(1); // Every hour
