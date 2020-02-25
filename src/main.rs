mod frontend;

use frontend::parser::Parser;

fn main() {
    let text = "(+ 1 2)".to_string();
    let mut parser = Parser::new(&text);
    parser.parse();
}
