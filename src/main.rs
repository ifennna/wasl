#![feature(box_syntax, box_patterns)]
#![feature(try_trait)]

mod frontend;
mod codegen;

use frontend::parser::Parser;
use codegen::emitter::Emitter;
use std::fs::File;
use std::io::{Write, BufReader, Read};
use std::env;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(args[1].to_owned())?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    let parser = Parser::new(&contents);

    let parse_result = parser.parse();
    let tree  = parse_result.unwrap();
    let mut emitter = Emitter::new();
    let content = emitter.emit(tree);

    let mut out = File::create("main.wat")?;
    out.write_all(content.as_bytes())?;
    Ok(())
}
