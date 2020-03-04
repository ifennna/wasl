use std::fmt::{Display, Formatter, Error};

type ReferenceNumber = usize;

/// Only operations on i32 numbers are supported at the moment
pub enum Types {
    i32Param(ReferenceNumber),
    i32Result
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
    Store(u8), // Store 4 bytes as an i32 into linear memory
    Const(i32), // Push a constant on the stack
}

impl Display for Types {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Types::i32Param(name) => write!(f, "(param $p{:?} i32)", name),
            Types::i32Result=> write!(f, "(result i32)"),
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
            Opcodes::Store(constant) => write!(f, "(i32.store32 {:?})", constant),
            Opcodes::Const(constant) => write!(f, "(i32.const {:?})", constant),
        }
    }
}