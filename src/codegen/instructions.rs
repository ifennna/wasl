use std::fmt::{Display, Formatter, Error};

type ReferenceNumber = usize;

/// Only operations on i64 numbers are supported at the moment
pub enum Types {
    I64Param(ReferenceNumber),
    I64Result
}

pub enum Opcodes {
    GetLocal, // Get a local variable from the stack
    Add, // Add two i64 constants
    Const(i64), // Push a constant on the stack
}

impl Display for Types {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Types::I64Param(name) => write!(f, "(param $p{:?} i64)", name),
            Types::I64Result=> write!(f, "(result i64)"),
        }
    }
}

impl Display for Opcodes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Opcodes::GetLocal => write!(f, "(get_local)"),
            Opcodes::Add => write!(f, "(i64.add"),
            Opcodes::Const(constant) => write!(f, "(i64.const {:?})", constant),
        }
    }
}