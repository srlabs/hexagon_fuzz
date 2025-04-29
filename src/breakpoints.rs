use crate::config::Config;
use crate::unwinder::{HexagonUnwinder, SymbolMap};
use libafl_qemu::ArchExtras;
use libafl_qemu::CallingConvention;
use libafl_qemu::Emulator;
use libafl_qemu::Regs;
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
    // Add other handlers here
}

// Implement a method to call the actual handler function
impl HandlerFunction {
    pub fn call(&self, emu: &Emulator) {
        match self {
            HandlerFunction::HandlePrintln => handle_println(emu),
            HandlerFunction::HandleNextPc => handle_next_pc(emu),
            HandlerFunction::HandleJumpOver => handle_jump_over(emu),
            HandlerFunction::HandleSecondClade => handle_second_clade(emu),
            HandlerFunction::HandleFatalError => handle_fatal_error(emu),
            HandlerFunction::HandleZeroingYetAnother => handle_zeroing_yet_another(emu),
            // Add cases for other handlers
        }
    }
}

/*
lazy_static! {
    pub static ref BREAKPOINTS: Vec<FirmwareFunction> = vec![
        FirmwareFunction {
            name: "qurt_println".to_string(),
            address: 0xfe10f2b0,
            handler: handle_println,
        },
        FirmwareFunction {
            name: "another_println".to_string(),
            address: 0xc03c96cc,
            handler: handle_println,
        },
        FirmwareFunction {
            name: "other_println".to_string(),
            address: 0xc08460e4,
            handler: handle_println,
        },
        FirmwareFunction {
            name: "diag_println".to_string(),
            address: 0xbfe8a1f4,
            handler: handle_println,
        },
        FirmwareFunction {
            name: "kernel_started".to_string(),
            address: 0xfe10c028,
            handler: handle_next_pc,
        },
        FirmwareFunction {
            name: "kernel_init".to_string(),
            address: 0xfe10c0a8,
            handler: handle_next_pc,
        },
        FirmwareFunction {
            name: "first_clade".to_string(),
            address: 0xfe10a3ec,
            handler: handle_jump_over,
        },
        FirmwareFunction {
            name: "second_clade".to_string(),
            address: 0xfe10a744,
            handler: handle_second_clade,
        },
        FirmwareFunction {
            name: "zeroeing".to_string(),
            address: 0xc083b9f0,
            handler: handle_jump_over,
        },
        FirmwareFunction {
            name: "app_init_done".to_string(),
            address: 0xfe008274,//0xc082e8ac,
            handler: handle_jump_over,
        },
        FirmwareFunction {
            name: "zeroing_yetanother".to_string(),
            address: 0xfe1012b4,//0xc082e8ac,
            handler: handle_zeroing_yetanother,
        },
        FirmwareFunction {
            name: "fatal_error".to_string(),
            address: 0xfe10ad10,//0xc082e8ac,
            handler: handle_fatal_error,
        },
        FirmwareFunction {
            name: "read_loop_hardware".to_string(),
            address: 0xfe115db0,
            handler: handle_jump_over,
        },
        FirmwareFunction {
            name: "calling_fatal_error".to_string(),
            address: 0xfe1021b4,
            handler: handle_fatal_error,
        },
        FirmwareFunction {
            name: "exception".to_string(),
            address: 0xfe000008,
            handler: handle_interrupt,
        },
        /*
        FirmwareFunction {
            name: "hardware_init_interrupt_related".to_string(),
            address: 0xfe0086d4,
            handler: skip,
        },
        FirmwareFunction {
            name: "current_debug".to_string(),
            address: 0xfe0010d4,
            handler: backtrace,
        },
        FirmwareFunction {
            name: "skipping_hardware_init".to_string(),
            address: 0xfe102138,
            handler: handle_skipping_hardware_init,
        },
        FirmwareFunction {
            name: "interrupt_table_1".to_string(),
            address: 0xfe1031b4,
            handler: handle_interrupt,
        },
        FirmwareFunction {
            name: "interrupt_table_2".to_string(),
            address: 0xfe000100,
            handler: handle_interrupt,
        },
        FirmwareFunction {
            name: "interrupt_table_3".to_string(),
            address: 0xfe000150,
            handler: handle_interrupt,
        },
        FirmwareFunction {
            name: "interrupt_table_4".to_string(),
            address: 0xfe0001f4,
            handler: handle_interrupt,
        },
        FirmwareFunction {
            name: "interrupt_tlb_miss".to_string(),
            address: 0xfe0001bc,
            handler: handle_interrupt,
        },
        FirmwareFunction {
            name: "interrupt_table_5".to_string(),
            address: 0xfe0001f0,
            handler: handle_interrupt,
        },
        FirmwareFunction {
            name: "interrupt_table_6".to_string(),
            address: 0xfe000388,
            handler: handle_interrupt,
        },
        FirmwareFunction {
            name: "interrupt_table_7".to_string(),
            address: 0xfe000240,
            handler: handle_interrupt,
        },
        FirmwareFunction {
            name: "interrupt_table_8".to_string(),
            address: 0xfe00017c,
            handler: handle_interrupt,
        },
        FirmwareFunction {
            name: "interrupt_table_9".to_string(),
            address: 0xfe0001cc,
            handler: handle_interrupt,
        },
        FirmwareFunction {
            name: "interrupt_table_10".to_string(),
            address: 0xfe00019c,
            handler: handle_interrupt,
        },
        */
    ];
}
*/
pub fn set_breakpoints(emu: &Emulator, config: Config) {
    for bp in config.breakpoints.iter() {
        emu.set_breakpoint(bp.address);
    }
}

pub fn backtrace(emu: &Emulator) {
    println!("--------------------------");
    println!("BACKTRACE");
    let mut frame_pointer: u32 = emu.current_cpu().unwrap().read_reg(Regs::R30).unwrap();
    let mut return_address: u32 = emu.current_cpu().unwrap().read_reg(Regs::R31).unwrap();
    if frame_pointer != 0 {
        while return_address != 0 {
            println!("{return_address:#x}");
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
    } else {
        println!("{return_address:#x}");
    };
    println!("--------------------------");
}

// Helper function to map register index to Regs enum
fn index_to_reg(index: u32) -> Option<Regs> {
    match index {
        0 => Some(Regs::R0),
        1 => Some(Regs::R1),
        2 => Some(Regs::R2),
        3 => Some(Regs::R3),
        4 => Some(Regs::R4),
        5 => Some(Regs::R5),
        6 => Some(Regs::R6),
        7 => Some(Regs::R7),
        8 => Some(Regs::R8),
        9 => Some(Regs::R9),
        10 => Some(Regs::R10),
        11 => Some(Regs::R11),
        12 => Some(Regs::R12),
        13 => Some(Regs::R13),
        14 => Some(Regs::R14),
        15 => Some(Regs::R15),
        16 => Some(Regs::R16),
        17 => Some(Regs::R17),
        18 => Some(Regs::R18),
        19 => Some(Regs::R19),
        20 => Some(Regs::R20),
        21 => Some(Regs::R21),
        22 => Some(Regs::R22),
        23 => Some(Regs::R23),
        24 => Some(Regs::R24),
        25 => Some(Regs::R25),
        26 => Some(Regs::R26),
        27 => Some(Regs::R27),
        28 => Some(Regs::R28),
        29 => Some(Regs::R29),
        30 => Some(Regs::R30),
        31 => Some(Regs::R31),
        _ => None,
    }
}

pub fn handle_breakpoint(emu: &Emulator, config: Config) -> Result<String, String> {
    let pcs = (0..emu.num_cpus())
        .map(|i| emu.cpu_from_index(i))
        .map(|cpu| -> Result<u32, String> { cpu.read_reg(Regs::Pc) });
    //for pc in pcs.clone() {
    //    let pc = pc.unwrap();
    //    println!("pc: {pc:#x}");
    //}
    let mut broken_pcs: String = String::new();
    for pc in pcs {
        for bp in config.breakpoints.iter() {
            if pc.clone().unwrap() == bp.address {
                bp.handler.call(emu);
                return Ok(bp.name.clone());
            }
        }
        let pc = pc.unwrap();
        broken_pcs.push_str(&format!("{pc:#x} "));
    }
    Ok("unexpected break at: ".to_string() + &broken_pcs)
}

fn read_cstring_from_ptr(emu: &Emulator, ptr: u32) -> String {
    let mut string: Vec<u8> = vec![0; 100];
    unsafe {
        emu.read_mem(ptr, &mut string);
    }
    let string = std::str::from_utf8(&string)
        .unwrap_or("bruno's shitty parsing")
        .split('\0')
        .next()
        .unwrap_or("bruno's shitty parsing2");
    string.to_owned()
}

// HANDLERS
fn handle_println(emu: &Emulator) {
    let format_string =
        read_cstring_from_ptr(emu, emu.current_cpu().unwrap().read_reg(Regs::R0).unwrap());
    let a: u32 = emu
        .read_function_argument(CallingConvention::Cdecl, 0)
        .unwrap();
    println!("{a:?}");
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
    println!("INTROSPECTED println | {formatted_output}");
    let return_address: u32 = emu.current_cpu().unwrap().read_return_address().unwrap();
    emu.current_cpu()
        .unwrap()
        .write_reg(Regs::Pc, return_address)
        .unwrap();
}

fn handle_jump_over(emu: &Emulator) {
    let return_address: u32 = emu.current_cpu().unwrap().read_return_address().unwrap();
    emu.current_cpu()
        .unwrap()
        .write_reg(Regs::Pc, return_address)
        .unwrap();
    println!("hit first clade (0xfe10a3ec), jumping over.");
}

fn handle_second_clade(emu: &Emulator) {
    let _ = emu.write_reg(Regs::R3, 28u32);
}

fn handle_next_pc(emu: &Emulator) {
    let current_pc: u32 = emu.current_cpu().unwrap().read_reg(Regs::Pc).unwrap();
    let next_pc: u32 = current_pc + 4u32;
    emu.current_cpu()
        .unwrap()
        .write_reg(Regs::Pc, next_pc)
        .unwrap();
}

fn handle_fatal_error(emu: &Emulator) {
    println!("FATAL ERROR!");
    backtrace(emu);
    println!("Exiting with 1337...");
    process::exit(1337);
}

fn handle_zeroing_yet_another(emu: &Emulator) {
    //let mut data = [0;1024];
    //let r4: u32 = emu.read_reg(Regs::R4).unwrap();
    //println!("{r4:?}");
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
