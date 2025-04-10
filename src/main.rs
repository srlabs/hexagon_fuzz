//! A fuzzer using qemu in systemmode for binary-only coverage of kernels

#[macro_use]
extern crate lazy_static;

use libafl_qemu::{emu::Emulator, Regs};
use std::env;

mod breakpoints;
use breakpoints::handle_breakpoint;

pub static mut MAX_INPUT_SIZE: usize = 50;

pub fn main() {
    env_logger::init();

    // Initialize QEMU
    let args: Vec<String> = env::args().collect();
    let env: Vec<(String, String)> = env::vars().collect();
    let emu = Emulator::new(&args, &env).unwrap();

    let devices = emu.list_devices();
    println!("Devices = {devices:?}");

    let mut _snap = None;

    // boot
    unsafe {
        breakpoints::set_breakpoints(&emu);
        println!("Breakpoints set");

        let _ = emu.run();
        loop {
            let breakpoint_name = handle_breakpoint(&emu).unwrap();
            println!("handled breakpoint {breakpoint_name}");
            if breakpoint_name == "app_init_done" {
                _snap = Some(emu.create_fast_snapshot(true));
                break;
            }
            let _ = emu.run();
        }
    }

    let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
    println!("app init done {current_pc:#x}");

    println!("lets go for adventures");
    unsafe {
        let _ = emu.run();
    }
    loop {
        let breakpoint_name = handle_breakpoint(&emu).unwrap();
        println!("handled breakpoint {breakpoint_name}");

        unsafe {
            let _ = emu.run();
        }
    }

    /*
    emu.current_cpu()
        .unwrap()
        .write_reg(Regs::Pc, 0xfe000000u32)
        .unwrap();

    let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
    println!("jumping to a random address {current_pc:#x}");

    emu.restore_fast_snapshot(snap.unwrap());
    let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
    println!("restored snapshot {current_pc:#x}");
    unsafe{
        emu.run();
    }
     */
}
