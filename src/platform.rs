use anyhow::Result;
use std::env;

// Get platform info and make sure it is supported
pub fn check_platform_supported() -> Result<bool> {
    if env::consts::OS == "macos" {
        return Ok(true);
    }
    Ok(false)
}
