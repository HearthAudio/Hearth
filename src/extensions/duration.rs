use std::time::Duration;

// It would be really nice if this was an extension but const traits are still experimental

// pub trait DurationExt {
//     fn from_hours(hours: u64) -> Duration;
// }
//
// impl DurationExt for Duration {
//     fn from_hours(hours: u64) -> Duration {
//         Duration::from_secs(hours * 60 * 60)
//     }
// }

pub const fn hours_to_duration(hours: u64) -> Duration {
    Duration::from_secs(hours * 60 * 60)
}
