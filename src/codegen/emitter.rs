use crate::frontend::ast::{Node, ListDetails, ConstantLiteral, MainDetails};
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
        let body = self.build_body(&self.tree);
        self.get_body_with_header(body)
    }

    fn get_body_with_header(&self, mut body: Vec<String>) -> String {
        body.insert(0, "(module ".to_owned());
        body.append(self.emit_export().as_mut());
        body.push(")".to_owned());

        body.join("\n ")
    }

    fn build_body(&self, tree: &Node) -> Vec<String> {
        let mut body = Vec::<String>::new();
        match tree {
            Node::List(list) => body.append(self.emit_function_call(list).as_mut()),
            Node::Null => {}
            Node::Main(details) => body.append(self.emit_main_function(details).as_mut()),
            Node::Def(_) => {}
            Node::Function(_) => {}
            Node::Constant(constant) => body.append(self.emit_constant(constant).as_mut()),
            Node::Keyword(_) => {}
            Node::Variable(_) => {}
            Node::Map(_) => {}
            Node::Vector(_) => {}
        };
        body
    }

    fn emit_main_function(&self, details: &MainDetails) -> Vec<String> {
        let mut types = Vec::new();
        for (index, _) in details.args.iter().enumerate() {
            types.push(Types::I64Param(index).to_string());
        }
        types.push(Types::I64Result.to_string());
        let mut body = self.build_body(details.body.as_ref());
        let mut function = vec!["(func $main ".to_owned()];
        function.append(types.as_mut());
        function.append(body.as_mut());
        function.push(")".to_owned());
        function
    }

    fn emit_function_call(&self, list: &ListDetails) -> Vec<String> {
        if let box Node::Keyword(details) = &list.head {
            match &details.token {
                &Lexeme::Plus => self.emit_add_function(&list.rest),
                &Lexeme::Print => self.emit_print_function(&list.rest),
                _ => vec![]
            }
        } else {vec![]}
    }

    fn emit_export(&self) -> Vec<String> {
        vec!["(export \"_start\" (func $main))".to_owned()]
    }

    fn emit_add_function(&self, args: &Vec<Node>) -> Vec<String> {
        let mut body = vec![Opcodes::Add.to_string()];
        for argument in args {
            body.append(self.build_body(argument).as_mut())
        }
        body.push(")".to_owned());
        body
    }

    fn emit_print_function(&self, args: &Vec<Node>) -> Vec<String> {
        let mut body = vec![];
        for argument in args {
            body.append(self.build_body(argument).as_mut())
        }
        body
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