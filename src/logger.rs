use std::fmt::format;
use std::time::SystemTime;

use fern::colors::{Color, ColoredLevelConfig};
use log::{info, Level};
use sentry::ClientInitGuard;
use crate::config::Config;


pub fn setup_logger(config: &Config) -> Result<(), fern::InitError> {

    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        // we actually don't need to specify the color for debug and info, they are white by default
        .info(Color::White)
        .debug(Color::White)
        // depending on the terminals color scheme, this is the same as the background color
        .trace(Color::BrightBlack);

    // configure colors for the name of the level.
    // since almost all of them are the same as the color for the whole line, we
    // just clone `colors_line` and overwrite our changes
    let colors_level = colors_line.info(Color::Green);


    let mut sentry : ClientInitGuard;
    if config.config.sentry_url.is_some() {
        println!("ES");
        sentry = sentry::init((config.config.sentry_url.clone().unwrap(), sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        }));
    }

    let config_f = config.clone();

    fern::Dispatch::new()
        .format(move |out, message, record| {
            // Handles Sentry logging
            if config_f.config.sentry_url.is_some() {
                if let Level::Error = record.level() {
                    sentry::capture_message(&format!("[{} {}] {}",record.level(),record.target(),message), sentry::Level::Error);
                    info!("Logged Message to Sentry");
                }
            }
            // Formats
            out.finish(format_args!(
                "{color_line}[{date} {level} {target} {color_line}] {message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                date = humantime::format_rfc3339_seconds(SystemTime::now()),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ));
        })
        .level(log::LevelFilter::Info)
        .level_for("serenity", log::LevelFilter::Warn)
        .level_for("tracing", log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    info!("------------------------------------------ NEW INSTANCE STARTED ------------------------------------------");
    Ok(())
}