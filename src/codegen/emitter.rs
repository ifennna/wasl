use crate::frontend::ast::{Node, ListDetails, ConstantLiteral};
use crate::frontend::scanner::{Lexeme};
use crate::codegen::instructions::{Opcodes, Types};

pub struct Emitter {
    tree: Node
}

impl Emitter {
    pub(crate) fn new(head: Node) -> Self {
        Emitter{
            tree: head
        }
    }

    pub fn emit(&self) -> String {
        let body = self.build_body();
        Emitter::get_body_with_header(body)
    }

    fn get_body_with_header(mut body: Vec<String>) -> String {
        body.insert(0, "(module ".to_owned());
        body.push(")".to_owned());

        body.join("\n ")
    }

    fn build_body(&self) -> Vec<String> {
        let mut body = Vec::<String>::new();
        match &self.tree {
            Node::List(list) => body.append(self.emit_function(list).as_mut()),
            _ => {}
        };
        body.append(self.emit_export().as_mut());
        body
    }

    fn emit_function(&self, list: &ListDetails) -> Vec<String> {
        if let box Node::Keyword(details) = &list.head {
            match &details.token {
                &Lexeme::Plus => self.emit_add_function(&list.rest),
                _ => vec![]
            }
        } else {vec![]}
    }

    fn emit_export(&self) -> Vec<String> {
        vec!["(export \"_start\" (func 0))".to_owned()]
    }

    fn emit_add_function(&self, args: &Vec<Node>) -> Vec<String> {
        let mut body = Vec::new();
        let mut types = Vec::new();
        for argument in args {
            body.append(self.evaluate_node(argument).as_mut())
        }
        types.push(Types::I64Result.to_string());
        body.push(Opcodes::Add.to_string());
        let mut function = vec!["(func ".to_owned()];
        function.append(types.as_mut());
        function.append(body.as_mut());
        function.push(")".to_owned());
        function
    }

    fn evaluate_node(&self, node: &Node) -> Vec<String> {
        match node {
            Node::Constant(constant) => self.emit_constant(constant),
            _ => vec![]
        }
    }

    fn emit_constant(&self, constant: &ConstantLiteral) -> Vec<String> {
        match constant {
            ConstantLiteral::IntegerLiteral(integer) => self.emit_integer_constant(*integer),
            ConstantLiteral::StringLiteral(string) => vec![]
        }
    }

    fn emit_integer_constant(&self, constant: i64) -> Vec<String> {
        vec![Opcodes::Const(constant).to_string()]
    }
}