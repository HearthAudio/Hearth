use std::{env, fs};
use log::{info, warn};

fn is_program_in_path(program: &str) -> bool {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(":") {
            let p_str = format!("{}/{}", p, program);
            if fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }
    false
}

pub fn print_warnings() {
    println!("{}",is_program_in_path("youtube-dl"));
    if is_program_in_path("youtube-dl") == false {
        warn!("youtube-dl is not installed! This Lantern instance will not be able to play tracks from YouTube until it is installed!")
    }
}

pub fn print_intro() {
    println!(r"  _                _
 | |    __ _ _ __ | |_ ___ _ __ _ __
 | |   / _` | '_ \| __/ _ \ '__| '_ \
 | |__| (_| | | | | ||  __/ |  | | | |
 |_____\__,_|_| |_|\__\___|_|  |_| |_|")
}