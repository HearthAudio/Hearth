use log::warn;
use snafu::{OptionExt, Whatever};
use sysinfo::{System, SystemExt};

// Get platform info and make sure it is supported
pub fn check_platform_supported() -> Result<(),Whatever> {
    let mut sys = System::new_all();
    sys.refresh_all();
    if sys.name().with_whatever_context(|| "Failed to get OS Name")? == "Darwin" {
        warn!("Hearth may or may not work when running on MacOS. But it will probably not work");
    }
    Ok(())
}