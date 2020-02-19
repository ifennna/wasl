use crate::frontend::ir::{BinaryOp, Chunk, Offset, OpCode};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("=== {} ===", name);

    for (index, instruction) in chunk.code.iter().enumerate() {
        disassemble_instruction(&chunk, *instruction, index)
    }
}

pub fn disassemble_instruction(chunk: &Chunk, instruction: OpCode, index: usize) {
    print!("{} ", index);

    if index > 0 && chunk.get_line(index) == chunk.get_line(index - 1) {
        print!("  | ");
    } else {
        print!("{} ", chunk.get_line(index));
    }

    match instruction {
        OpCode::OpConstant(offset) => constant_instruction("OpConstant", chunk, offset),
        OpCode::OpNegate => simple_instruction("OpNegate"),
        OpCode::BinaryOperation(operator) => match operator {
            BinaryOp::Add => simple_instruction("OpAdd"),
            BinaryOp::Subtract => simple_instruction("OpSubtract"),
            BinaryOp::Multiply => simple_instruction("OpMultiply"),
            BinaryOp::Divide => simple_instruction("OpDivide"),
        },
    }
}

fn simple_instruction(name: &str) {
    print!("{}\n", name);
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: Offset) {
    let constant = chunk.get_constant(offset);

    print!("{} {:?}\n", name, constant);
}
