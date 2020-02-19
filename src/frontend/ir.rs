pub type Offset = usize;

struct AddressCode {

}

#[derive(Copy, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide
}

#[derive(Copy, Clone)]
pub enum OpCode {
    OpConstant(Offset),
    OpNegate,
    BinaryOperation(BinaryOp)
}

#[derive(Debug, Copy, Clone)]
pub enum Constant {
    Number(f64)
}

pub struct Chunk {
    pub code: Vec<OpCode>,
    constants: Vec<Constant>,
    lines: Vec<Offset>
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::<OpCode>::new(),
            constants: Vec::<Constant>::new(),
            lines: Vec::new()
        }
    }

    pub fn write (&mut self, code: OpCode, line: Offset) {
        self.code.push(code);
        self.lines.push(line)
    }

    pub fn add_constant(&mut self, value: Constant) -> Offset {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn get_constant(&self, position: Offset) -> Constant {
        self.constants[position]
    }

    pub fn get_line(&self, position: Offset) -> Offset {
        self.lines[position]
    }
}