use std::{env, fs};
use log::{error, warn};
use colored::Colorize;
use rand::Rng;
use anyhow::Result;

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
    if !is_program_in_path("yt-dlp") {
        warn!("yt-dlp is not installed! This Lantern instance will not be able to play tracks from YouTube until it is installed!")
    }
}

pub fn over_servers_warning() {
    warn!("Guild count is over 5000. Hearth is only free up to 5000 discord servers. You must contact Hearth Industries within 14 Days of passing 5000 servers. For more details see the license in the github repo: LICENSE.md. If you have already acquired a paid license no further action is needed.")
}

// fn check_use_true_colors() -> bool {
//     let env_val = env::var("COLORTERM");
//
//     match env_val {
//         Ok(v) => {
//             matches!(v.as_str(), "truecolor")
//         }
//         Err(e) => {
//             error!("Failed to fetch terminal color scheme with error: {}",e);
//             false
//         }
//     }
// }

pub fn print_intro() {
    let s = r"
        ██╗░░██╗███████╗░█████╗░██████╗░████████╗██╗░░██╗
        ██║░░██║██╔════╝██╔══██╗██╔══██╗╚══██╔══╝██║░░██║
        ███████║█████╗░░███████║██████╔╝░░░██║░░░███████║
        ██╔══██║██╔══╝░░██╔══██║██╔══██╗░░░██║░░░██╔══██║
        ██║░░██║███████╗██║░░██║██║░░██║░░░██║░░░██║░░██║
        ╚═╝░░╚═╝╚══════╝╚═╝░░╚═╝╚═╝░░╚═╝░░░╚═╝░░░╚═╝░░╚═╝";
    let mut rng = rand::thread_rng();
    let dice = rng.gen_range(0..2);

    // let use_true_colors = check_use_true_colors();

    match dice {
        0 => println!("{}",s.red()),
        1 => print!("{}",s.yellow()),
        _ => {
            // This really should not happen but just in case
            println!("{}",s.red());
        }
    }
    println!();
}