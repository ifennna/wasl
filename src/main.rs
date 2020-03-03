#![feature(box_syntax, box_patterns)]
#![feature(try_trait)]

mod frontend;
mod codegen;

use frontend::parser::Parser;
use codegen::emitter::Emitter;
use std::fs::File;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let text = "(defn main (print (+ 40 3)))".to_string();
    let parser = Parser::new(&text);

    let parse_result = parser.parse();
    let tree  = parse_result.unwrap();
    let emitter = Emitter::new(tree);
    let content = emitter.emit();

    let mut out = File::create("main.wat")?;
    out.write_all(content.as_bytes())?;
    Ok(())
}
