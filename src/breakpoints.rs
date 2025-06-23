use crate::config::Config;
use libafl_qemu::ArchExtras;
use libafl_qemu::CallingConvention;
use libafl_qemu::Emulator;
use libafl_qemu::Regs;
use log::{debug, error, info};
use serde::Deserialize;
use std::process;

#[derive(Debug, Clone, Deserialize)]
pub enum HandlerFunction {
    HandlePrintln,
    HandleNextPc,
    HandleJumpOver,
    HandleSecondClade,
    HandleFatalError,
    HandleZeroingYetAnother,
    HandlerEmpty, // Add other handlers here
}

// Implement a method to call the actual handler function
impl HandlerFunction {
    pub fn call(&self, emu: &Emulator) {
        debug!("Executing handler function: {:?}", self);
        match self {
            HandlerFunction::HandlePrintln => handle_println(emu),
            HandlerFunction::HandleNextPc => handle_next_pc(emu),
            HandlerFunction::HandleJumpOver => handle_jump_over(emu),
            HandlerFunction::HandleSecondClade => handle_second_clade(emu),
            HandlerFunction::HandleFatalError => handle_fatal_error(emu),
            HandlerFunction::HandleZeroingYetAnother => handle_zeroing_yet_another(emu),
            HandlerFunction::HandlerEmpty => handler_empty(emu),
            // Add cases for other handlers
        }
    }
}

pub fn set_breakpoints(emu: &Emulator, config: Config) {
    for bp in config.breakpoints.iter() {
        debug!(
            "Setting breakpoint: '{}' at address 0x{:x} (handler: {:?})",
            bp.name, bp.address, bp.handler
        );
        emu.set_breakpoint(bp.address);
    }
    debug!("All breakpoints set successfully");
}

pub fn backtrace(emu: &Emulator) {
    error!("--------------------------");
    error!("BACKTRACE");
    let mut frame_pointer: u32 = emu.current_cpu().unwrap().read_reg(Regs::R30).unwrap();
    let mut return_address: u32 = emu.current_cpu().unwrap().read_reg(Regs::R31).unwrap();
    debug!(
        "Starting backtrace: FP=0x{:x}, RA=0x{:x}",
        frame_pointer, return_address
    );

    if frame_pointer != 0 {
        let mut frame_count = 0;
        while return_address != 0 {
            error!("{return_address:#x}");
            frame_count += 1;
            debug!(
                "Frame {}: FP=0x{:x}, RA=0x{:x}",
                frame_count, frame_pointer, return_address
            );

            let mut buf = [0, 0, 0, 0];
            unsafe {
                emu.read_mem(frame_pointer, &mut buf);
            }
            return_address = u32::from_le_bytes(buf);
            unsafe {
                emu.read_mem(frame_pointer - 4, &mut buf);
            }
            frame_pointer = u32::from_be_bytes(buf);
        }
        debug!("Backtrace completed with {} frames", frame_count);
    } else {
        error!("{return_address:#x}");
        debug!("Single frame backtrace (FP was 0)");
    };
    error!("--------------------------");
}

pub fn handle_breakpoint(emu: &Emulator, config: Config) -> Result<String, String> {
    debug!("Handling breakpoint - checking {} CPUs", emu.num_cpus());
    let pcs = (0..emu.num_cpus())
        .map(|i| emu.cpu_from_index(i))
        .map(|cpu| -> Result<u32, String> { cpu.read_reg(Regs::Pc) });
    for (cpu_id, pc) in pcs.clone().enumerate() {
        let pc = pc.unwrap();
        debug!("CPU {}: pc: {pc:#x}", cpu_id);
    }
    let mut broken_pcs: String = String::new();
    for pc in pcs {
        for bp in config.breakpoints.iter() {
            if pc.clone().unwrap() == bp.address {
                info!("Breakpoint reached: {} at 0x{:x}", bp.name, bp.address);
                debug!("Calling handler: {:?}", bp.handler);
                bp.handler.call(emu);
                return Ok(bp.name.clone());
            }
        }
        let pc = pc.unwrap();
        broken_pcs.push_str(&format!("{pc:#x} "));
    }
    debug!(
        "No matching breakpoint found for PCs: {}",
        broken_pcs.trim()
    );
    Ok("unexpected break at: ".to_string() + &broken_pcs)
}

fn read_cstring_from_ptr(emu: &Emulator, ptr: u32) -> String {
    let mut string: Vec<u8> = vec![0; 100];
    unsafe {
        emu.read_mem(ptr, &mut string);
    }
    let string = std::str::from_utf8(&string)
        .unwrap_or("Invalid utf-8 string")
        .split('\0')
        .next()
        .unwrap_or("Invalid or unterminated C-string");
    string.to_owned()
}

// HANDLERS
fn handler_empty(emu: &Emulator) {
    let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
    debug!(
        "Empty handler called at address: 0x{:x} - no operation performed",
        current_pc
    );
}

fn handle_println(emu: &Emulator) {
    debug!("Handle printf: introspecting printf call");
    let format_ptr = emu.current_cpu().unwrap().read_reg(Regs::R0).unwrap();
    let format_string = read_cstring_from_ptr(emu, format_ptr);

    let a: u32 = emu
        .read_function_argument(CallingConvention::Cdecl, 0)
        .unwrap();
    debug!("function argument 0: {a:#x}");
    // Prepare to parse the arguments
    let mut arg_index = 1; // The first argument is the format string
    let mut args = Vec::new();
    let mut chars = format_string.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(&next_char) = chars.peek() {
                match next_char {
                    'd' => {
                        // Read an integer argument
                        let arg_value: u32 = emu
                            .read_function_argument(CallingConvention::Cdecl, arg_index)
                            .unwrap();
                        args.push(format!("{}", { arg_value }));
                        arg_index += 1;
                        chars.next();
                    }
                    's' => {
                        // Read a string argument
                        let arg_ptr = emu
                            .read_function_argument(CallingConvention::Cdecl, arg_index)
                            .unwrap();
                        let arg_value = read_cstring_from_ptr(emu, arg_ptr);
                        args.push(arg_value);
                        arg_index += 1;
                        chars.next();
                    }
                    'x' => {
                        // Read a hexadecimal integer argument
                        let arg_value: u32 = emu
                            .read_function_argument(CallingConvention::Cdecl, arg_index)
                            .unwrap();
                        args.push(format!("{arg_value:x}"));
                        arg_index += 1;
                        chars.next();
                    }
                    '%' => {
                        // Handle escaped percent (%%)
                        args.push("%".to_string());
                        chars.next();
                    }
                    _ => {}
                }
            }
        }
    }
    // Construct the formatted output
    let mut formatted_output = format_string.clone();
    for arg in &args {
        formatted_output = formatted_output.replacen("%d", arg, 1);
        formatted_output = formatted_output.replacen("%s", arg, 1);
        formatted_output = formatted_output.replacen("%x", arg, 1);
    }
    info!("INTROSPECTED println | {formatted_output}");
    let return_address: u32 = emu.current_cpu().unwrap().read_return_address().unwrap();
    emu.current_cpu()
        .unwrap()
        .write_reg(Regs::Pc, return_address)
        .unwrap();
}

fn handle_jump_over(emu: &Emulator) {
    let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
    let return_address: u32 = emu.current_cpu().unwrap().read_return_address().unwrap();
    debug!(
        "Jump over handler: current PC 0x{:x} -> jumping to 0x{:x}",
        current_pc, return_address
    );
    emu.current_cpu()
        .unwrap()
        .write_reg(Regs::Pc, return_address)
        .unwrap();
    debug!("jumping over to: {return_address:#x}");
}

fn handle_second_clade(emu: &Emulator) {
    let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
    let old_r3: u32 = emu.current_cpu().unwrap().read_reg(Regs::R3).unwrap();
    debug!(
        "Second clade handler at 0x{:x}: setting R3 from 0x{:x} to 28 (0x1c)",
        current_pc, old_r3
    );
    let _ = emu.write_reg(Regs::R3, 28u32);
}

fn handle_next_pc(emu: &Emulator) {
    let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
    let next_pc: u32 = current_pc + 4u32;
    debug!(
        "Next PC handler: advancing from 0x{:x} to 0x{:x}",
        current_pc, next_pc
    );
    emu.current_cpu()
        .unwrap()
        .write_reg(Regs::Pc, next_pc)
        .unwrap();
}

fn handle_fatal_error(emu: &Emulator) {
    let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
    error!("FATAL ERROR at 0x{:x}!", current_pc);
    backtrace(emu);
    error!("Exiting with 1337...");
    process::exit(1337);
}

fn handle_zeroing_yet_another(emu: &Emulator) {
    let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
    debug!(
        "Zeroing handler at 0x{:x}: jumping to 0xfe1012c0",
        current_pc
    );
    //let mut data = [0;1024];
    //let r4: u32 = emu.read_reg(Regs::R4).unwrap();
    //debug!("{r4:?}");
    //emu.read_mem(0xfe0125f0, &mut data);
    emu.current_cpu()
        .unwrap()
        .write_reg(Regs::Pc, 0xfe1012c0u32)
        .unwrap();
}
/*
fn handle_skipping_hardware_init(emu: &Emulator) {
    unsafe {
        emu.current_cpu()
            .unwrap()
            .write_reg(Regs::R0, 2u32)
            .unwrap();
    }
}
*/
