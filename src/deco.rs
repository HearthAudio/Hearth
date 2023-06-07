use colored::Colorize;
use log::warn;
use rand::Rng;
use std::{env, fs};

fn is_program_in_path(program: &str) -> bool {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(':') {
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
        0 => println!("{}", s.red()),
        1 => print!("{}", s.yellow()),
        _ => {
            // This really should not happen but just in case
            println!("{}", s.red());
        }
    }
    println!();
}
