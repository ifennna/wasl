#![feature(box_syntax, box_patterns)]
#![feature(try_trait)]

mod codegen;
mod frontend;

use codegen::emitter::Emitter;
use frontend::parser::{ParseError, Parser};
use std::env;
use std::fs::File;
use std::io::{BufReader, Read, Write};

#[derive(Debug)]
enum AppError {
    Parse(ParseError),
    Io(std::io::Error),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<ParseError> for AppError {
    fn from(err: ParseError) -> Self {
        AppError::Parse(err)
    }
}

fn main() -> Result<(), AppError> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(args[1].to_owned())?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    let parser = Parser::new(&contents);

    let tree = parser.parse()?;
    let mut emitter = Emitter::new();
    let content = emitter.emit(tree);

    let mut out = File::create("main.wat")?;
    out.write_all(content.as_bytes())?;
    Ok(())
}
