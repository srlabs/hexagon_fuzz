mod breakpoints;
mod config;
mod fuzz;
mod utils;

use crate::{
    fuzz::run_fuzzer,
    utils::{boot_firmware, init_qemu, run_no_fuzzer},
};
use config::{parse_config, CONFIG_PATH};
use env_logger::Env;
use log::{debug, error};
use std::process;

pub static mut MAX_INPUT_SIZE: usize = 50;

pub fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let config = parse_config(CONFIG_PATH).unwrap();
    debug!("{config:?}");

    // Initialize QEMU
    let emu = init_qemu(&config);

    // boot
    let snap = boot_firmware(&config, &emu).unwrap_or_else(|| {
        error!("Could not snapshot firmware!");
        process::exit(1);
    });

    // Fuzz or just continue booting without fuzzing
    if config.fuzz {
        run_fuzzer(config, emu, snap);
    } else {
        run_no_fuzzer(&config, &emu)
    }
}
