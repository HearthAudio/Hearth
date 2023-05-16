
use anyhow::{Context, Result};
use sysinfo::{System, SystemExt};

// Get platform info and make sure it is supported
pub fn check_platform_supported() -> Result<bool> {
    let mut sys = System::new_all();
    sys.refresh_all();
    if sys.name().context("Failed to get OS Name")? == "Darwin" {
        return Ok(true);
    }
    return Ok(false);
}
