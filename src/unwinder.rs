use libafl_qemu::{emu::Emulator, Regs};
use std::error::Error;
use std::fmt;

/// Custom error type for unwinding operations
#[derive(Debug)]
pub struct UnwindError {
    message: String,
}

impl UnwindError {
    fn new(message: &str) -> Self {
        UnwindError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for UnwindError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unwinding error: {}", self.message)
    }
}

impl Error for UnwindError {}

/// Represents a stack frame in the backtrace
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub pc: u32, // Program Counter
    pub sp: u32, // Stack Pointer
    pub fp: u32, // Frame Pointer
    pub function_name: Option<String>,
}

impl fmt::Display for StackFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = &self.function_name {
            write!(f, "{} (0x{:08x})", name, self.pc)
        } else {
            write!(f, "0x{:08x}", self.pc)
        }
    }
}

/// The main unwinder for Hexagon CPU
pub struct HexagonUnwinder<'a> {
    emu: &'a Emulator,
    max_frames: usize,
    symbol_map: Option<SymbolMap>,
}

/// Simple symbol map to resolve addresses to function names
#[derive(Clone)]
pub struct SymbolMap {
    symbols: Vec<(u32, u32, String)>, // (start_addr, end_addr, name)
}

impl SymbolMap {
    pub fn new() -> Self {
        SymbolMap {
            symbols: Vec::new(),
        }
    }

    pub fn add_symbol(&mut self, start_addr: u32, end_addr: u32, name: String) {
        self.symbols.push((start_addr, end_addr, name));
    }

    pub fn lookup(&self, addr: u32) -> Option<&str> {
        for (start, end, name) in &self.symbols {
            if addr >= *start && addr < *end {
                return Some(name);
            }
        }
        None
    }
}

impl<'a> HexagonUnwinder<'a> {
    pub fn new(emu: &'a Emulator) -> Self {
        HexagonUnwinder {
            emu,
            max_frames: 32, // Default max frames to prevent infinite loops
            symbol_map: None,
        }
    }

    pub fn with_symbol_map(mut self, symbol_map: SymbolMap) -> Self {
        self.symbol_map = Some(symbol_map);
        self
    }

    pub fn with_max_frames(mut self, max_frames: usize) -> Self {
        self.max_frames = max_frames;
        self
    }

    // Safe wrapper to read memory as u32
    unsafe fn read_u32(&self, addr: u32) -> Option<u32> {
        let mut buf = [0u8; 4];

        // Attempt to read memory
        self.emu.read_mem(addr, &mut buf);

        // Check if the read was successful by verifying if buf was modified
        // This is a simplified check - in a real implementation you might want
        // to enhance error detection
        if buf != [0u8; 4] || addr == 0 {
            Some(u32::from_le_bytes(buf))
        } else {
            None
        }
    }

    /// Unwind the stack and return a vector of stack frames
    pub fn unwind(&self) -> Result<Vec<StackFrame>, Box<dyn Error>> {
        let mut frames = Vec::new();
        let cpu = self
            .emu
            .current_cpu()
            .ok_or(UnwindError::new("No current CPU"))?;

        // Get initial register values
        let pc: u32 = cpu.read_reg(Regs::Pc)?;
        let sp: u32 = cpu.read_reg(Regs::R29)?; // r29 is SP in Hexagon
        let fp: u32 = cpu.read_reg(Regs::R30)?; // r30 is FP in Hexagon
        let lr: u32 = cpu.read_reg(Regs::R31)?; // r31 is LR (Link Register) in Hexagon

        // Add the current frame
        frames.push(StackFrame {
            pc,
            sp,
            fp,
            function_name: self.lookup_symbol(pc),
        });

        // If we have a valid link register, add it as our caller
        if lr != 0 && lr != pc {
            frames.push(StackFrame {
                pc: lr,
                sp, // We don't know the exact SP for the caller yet
                fp, // We don't know the exact FP for the caller yet
                function_name: self.lookup_symbol(lr),
            });
        }

        // Use frame pointer chain to unwind
        let mut current_fp = fp;
        for _ in 0..self.max_frames {
            if current_fp == 0 || !self.is_valid_address(current_fp) {
                break;
            }

            // In Hexagon ABI, the previous FP is stored at FP+0,
            // and the return address is stored at FP+4

            // Safely read the return address and previous frame pointer
            let return_addr = unsafe {
                match self.read_u32(current_fp + 4) {
                    Some(addr) => addr,
                    None => break, // Error reading memory
                }
            };

            let prev_fp = unsafe {
                match self.read_u32(current_fp) {
                    Some(addr) => addr,
                    None => break, // Error reading memory
                }
            };

            // If we get a valid return address and it's different from the
            // previous frame's PC, add a new frame
            if return_addr != 0
                && return_addr != frames.last().unwrap().pc
                && self.is_valid_address(return_addr)
            {
                frames.push(StackFrame {
                    pc: return_addr,
                    sp: current_fp + 8, // Estimated SP based on frame layout
                    fp: prev_fp,
                    function_name: self.lookup_symbol(return_addr),
                });
            }

            // Move to the previous frame
            if prev_fp <= current_fp {
                // Prevent infinite loops if the frame pointer chain is corrupted
                break;
            }
            current_fp = prev_fp;
        }

        Ok(frames)
    }

    /// Generate a pretty backtrace string
    pub fn generate_backtrace(&self) -> Result<String, Box<dyn Error>> {
        let frames = self.unwind()?;
        let mut output = String::from("----- Backtrace -----\n");
        for (i, frame) in frames.iter().enumerate() {
            output.push_str(&format!(
                "#{}: {} (FP: 0x{:08x}, SP: 0x{:08x})\n",
                i, frame, frame.fp, frame.sp
            ));
        }
        output.push_str("----- End of Backtrace -----\n");
        Ok(output)
    }

    /// Look up a symbol by address
    fn lookup_symbol(&self, addr: u32) -> Option<String> {
        if let Some(ref symbol_map) = self.symbol_map {
            if let Some(name) = symbol_map.lookup(addr) {
                return Some(name.to_string());
            }
        }
        None
    }

    /// Check if an address is valid
    fn is_valid_address(&self, addr: u32) -> bool {
        // Basic validation: non-zero and within a reasonable range
        addr != 0 && addr < 0xFFFFFFFF && (addr & 0x3) == 0 // 4-byte aligned
    }

    /// Attempt to determine if we're currently in an exception handler
    pub fn is_in_exception(&self) -> bool {
        if let Some(cpu) = self.emu.current_cpu() {
            // Check for common exception indicators in Hexagon
            // This is hardware-specific and may need adjustment
            if let Ok(usr) = cpu.read_reg::<Regs, u32>(Regs::Usr) {
                // Check bits in the USR register that might indicate exception state
                return (usr & 0x10) != 0; // Example - check specific bit
            }
        }
        false
    }

    /// Get the exception cause if we're in an exception
    pub fn get_exception_cause(&self) -> Result<u32, Box<dyn Error>> {
        if let Some(cpu) = self.emu.current_cpu() {
            // In Hexagon, the cause register might contain the exception cause
            // This is hardware-specific and may need adjustment
            let cause = cpu.read_reg::<Regs, u32>(Regs::Usr)?; // Using USR as an example
            return Ok(cause & 0x7); // Extract the cause bits (example)
        }
        Err(Box::new(UnwindError::new(
            "Could not determine exception cause",
        )))
    }
}
