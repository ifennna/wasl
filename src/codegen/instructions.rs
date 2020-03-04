use std::fmt::{Display, Formatter, Error};

type ReferenceNumber = usize;

/// Only operations on i32 numbers are supported at the moment
pub enum Types {
    I32param(ReferenceNumber),
    I32result
}

pub struct OpData {
    pub location: Opcodes,
    pub data: String
}

#[derive(Copy, Clone)]
pub enum Opcodes {
    GetLocal, // Get a local variable from the stack
    Add, // Add two i32 constants
    Subtract, // Subtract two i32 constants
    Load, // Load 4 bytes as an i32 from linear memory
    Store(i32, i32), // Store 4 bytes as an i32 into linear memory
    Const(i32), // Push a constant on the stack
    Drop
}

pub enum WASIImports {
    FDWrite
}

pub enum SysCalls {
    Write(Opcodes, Opcodes, Opcodes, Opcodes)
}

impl Display for Types {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Types::I32param(name) => write!(f, "(param $p{:?} i32)", name),
            Types::I32result => write!(f, "(result i32)"),
        }
    }
}

impl Display for OpData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "(data {} {:?})", self.location, self.data)
    }
}

impl Display for Opcodes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Opcodes::GetLocal => write!(f, "(get_local)"),
            Opcodes::Add => write!(f, "(i32.add"),
            Opcodes::Subtract => write!(f, "(i32.sub"),
            Opcodes::Load => write!(f, "(i32.load32_s)"),
            Opcodes::Store(address, value) =>
                write!(f, "(i32.store {} {})", Opcodes::Const(*address), Opcodes::Const(*value)),
            Opcodes::Const(constant) => write!(f, "(i32.const {:?})", constant),
            Opcodes::Drop => write!(f, "drop")
        }
    }
}

impl Display for SysCalls {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            SysCalls::Write(file_descriptor, iov_ptr, iov_len, num_written) =>
                write!(f, "(call $fd_write {} {} {} {})", file_descriptor, iov_ptr, iov_len, num_written)
        }
    }
}

impl Display for WASIImports {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            WASIImports::FDWrite =>
                write!(f, "(import \"wasi_unstable\" \"fd_write\" (func $fd_write (param i32 i32 i32 i32) {}))",
                       Types::I32result),
        }
    }
}