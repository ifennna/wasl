#![feature(box_syntax, box_patterns)]
#![feature(try_trait)]

mod frontend;
mod codegen;

use frontend::parser::Parser;
use codegen::emitter::Emitter;
use std::fs::File;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let text = "(defn main (print \"Hello world\"))".to_string();
    let parser = Parser::new(&text);

    let parse_result = parser.parse();
    let tree  = parse_result.unwrap();
    let mut emitter = Emitter::new();
    let content = emitter.emit(tree);

    let mut out = File::create("main.wat")?;
    out.write_all(content.as_bytes())?;
    Ok(())
}
