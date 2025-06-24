use crate::{
    breakpoints::{handle_breakpoint, set_breakpoints},
    config::Config,
};
use libafl_qemu::{Emulator, FastSnapshot, Regs};
use log::{debug, info};
use std::env;

/// Runs the emulator without fuzzing, continuously handling breakpoints.
///
/// This function starts the emulator from the current program counter position
/// and enters an infinite loop where it runs the emulator and handles any
/// breakpoints that are encountered.
///
/// # Arguments
/// * `config` - Configuration settings for the emulator
/// * `emu` - The emulator instance to run
///
/// # Returns
/// This function never returns
pub(crate) fn run_no_fuzzer(config: Config) -> ! {
    // Initialize QEMU
    let emu = init_qemu(&config);
    set_breakpoints(&emu, config.clone());
    info!("Breakpoints set");

    
    unsafe {
        let _ = emu.run();
    }
    debug!("Emulator stopped, entering breakpoint handling loop");
    loop {
        let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
        debug!("Emulator stopped at PC: 0x{:x}", current_pc);
        let breakpoint_name = handle_breakpoint(&emu, config.clone()).unwrap();
        debug!("handled breakpoint: {breakpoint_name}");
        unsafe {
            let _ = emu.run();
        }
    }
}

/// Boots the firmware and creates a snapshot at an appropriate stopping point.
///
/// This function sets up breakpoints and runs the emulator until it reaches either
/// the fuzz target address (if fuzzing is enabled) or the "app_init_done" breakpoint.
/// When either condition is met, it creates and returns a fast snapshot of the
/// emulator state.
///
/// # Arguments
/// * `config` - Configuration settings containing breakpoint addresses and fuzz settings
/// * `emu` - The emulator instance to boot
///
/// # Returns
/// * `Some(FastSnapshot)` - A snapshot created at the fuzz target or app init completion
/// * `None` - This should not occur in normal operation as the function loops until a snapshot is created
///
/// # Behavior
/// 1. Sets up standard breakpoints from config
/// 2. If fuzzing is enabled, sets a breakpoint at the fuzz target address
/// 3. Runs the emulator and handles breakpoints in a loop
/// 4. Creates a snapshot when reaching the fuzz target or "app_init_done" breakpoint
pub(crate) fn boot_firmware(config: &Config, emu: &Emulator) -> Option<FastSnapshot> {
    debug!("Starting firmware boot process");

    debug!("Setting up breakpoints from configuration");
    set_breakpoints(&emu, config.clone());
    emu.set_breakpoint(config.fuzz_target_address);

    info!("Breakpoints set");
    debug!("Starting initial firmware boot run");
    unsafe {
        let _ = emu.run();
    }
    debug!("Emulator stopped during boot, entering breakpoint handling loop");

    loop {
        let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
        debug!("Boot: emulator stopped at PC: 0x{:x}", current_pc);

        let breakpoint_name = handle_breakpoint(emu, config.clone()).unwrap();
        debug!("Boot: handled breakpoint: {}", breakpoint_name);

        if current_pc == config.fuzz_target_address {
            info!("reached fuzz target during normal boot");
            emu.remove_breakpoint(config.fuzz_target_address);

            info!("Creating fast snapshot at fuzz target address");
            return Some(emu.create_fast_snapshot(true));
        }

        if breakpoint_name == "app_init_done" {
            emu.current_cpu()
                .unwrap()
                .write_reg(Regs::Pc, config.fuzz_target_address)
                .unwrap();
            info!("app init done, creating snapshot at fuzz target: {:#x}", config.fuzz_target_address);
            return Some(emu.create_fast_snapshot(true));
        }

        debug!("Boot: resuming emulator execution");
        unsafe {
            let _ = emu.run();
        }
    }
}

pub(crate) fn init_qemu(config: &Config) -> Emulator {
    debug!("Initializing QEMU emulator");
    debug!("QEMU args: {:?}", config.qemu_args);

    let env: Vec<(String, String)> = env::vars().collect();
    debug!("Environment variables count: {}", env.len());

    debug!("Creating QEMU emulator instance");
    let emu = Emulator::new(&config.qemu_args, &env).unwrap();

    let devices = emu.list_devices();
    debug!("Available QEMU devices: {:?}", devices);

    emu
}
