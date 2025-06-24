mod breakpoints;
mod config;
mod fuzz;
mod utils;

use crate::{fuzz::run_fuzzer, utils::run_no_fuzzer};
use config::{parse_config, CONFIG_PATH};
use env_logger::Env;
use log::{debug, info};

pub static mut MAX_INPUT_SIZE: usize = 50;

pub fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting hexagon_fuzz application");
    debug!("Parsing configuration from: {}", CONFIG_PATH);
    let config = parse_config(CONFIG_PATH).unwrap();

    // Fuzz or just continue booting without fuzzing
    if config.fuzz {
        debug!("Fuzzing mode enabled - starting fuzzer");
        run_fuzzer(config);
    } else {
        debug!("Fuzzing mode disabled - continuing without fuzzing");
        run_no_fuzzer(config)
    }
}
