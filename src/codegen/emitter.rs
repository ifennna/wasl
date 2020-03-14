use crate::codegen::instructions::{OpData, Opcodes, SysCalls, Types, WASIImports};
use crate::frontend::ast::{ConstantLiteral, ListDetails, MainDetails, Node};
use crate::frontend::scanner::Lexeme;
use crate::frontend::scanner::Lexeme::StringLiteral;

pub struct Emitter {
    imports: Vec<WASIImports>,
    data: Vec<OpData>,
}

impl Emitter {
    pub(crate) fn new() -> Self {
        Emitter {
            imports: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn emit(&mut self, head: Vec<Node>) -> String {
        let body = self.build_body(&head);
        self.get_body_with_header(body)
    }

    fn get_body_with_header(&mut self, mut body: Vec<String>) -> String {
        body.insert(0, "(module ".to_owned());
        body.insert(
            1,
            self.imports
                .iter()
                .map(|item| return item.to_string())
                .collect(),
        );
        body.insert(2, self.emit_memory_initializer());
        body.insert(
            3,
            self.data
                .iter()
                .map(|item| return item.to_string())
                .collect(),
        );
        body.append(self.emit_export().as_mut());
        body.push(")".to_owned());

        body.join("\n ")
    }

    fn build_body(&mut self, nodes: &Vec<Node>) -> Vec<String> {
        let mut body = Vec::<String>::new();
        for node in nodes {
            body.append(self.emit_instructions(node).as_mut())
        }
        body
    }

    fn emit_instructions(&mut self, tree: &Node) -> Vec<String> {
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

    fn emit_main_function(&mut self, details: &MainDetails) -> Vec<String> {
        let mut types = Vec::new();
        for (index, _) in details.args.iter().enumerate() {
            types.push(Types::I32param(index).to_string());
        }
        let mut body = self.emit_function_body(details.body.as_ref());
        let mut function = vec!["(func $main ".to_owned()];
        function.append(types.as_mut());
        function.append(body.as_mut());
        function.push(")".to_owned());
        function
    }

    fn emit_function_body(&mut self, body: &Vec<Node>) -> Vec<String> {
        let mut instructions = Vec::new();
        for expression in body {
            instructions.append(self.emit_instructions(expression).as_mut());
        }
        instructions
    }

    fn emit_function_call(&mut self, list: &ListDetails) -> Vec<String> {
        if let box Node::Keyword(details) = &list.head {
            match &details.token {
                &Lexeme::Plus => self.emit_add_function(&list.rest),
                &Lexeme::Minus => self.emit_subtract_function(&list.rest),
                &Lexeme::Print => self.emit_print_function(&list.rest),
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    fn emit_export(&self) -> Vec<String> {
        vec!["(export \"_start\" (func $main))".to_owned()]
    }

    // Perhaps these functions are collapsible
    fn emit_add_function(&mut self, args: &Vec<Node>) -> Vec<String> {
        let mut body = vec![Opcodes::Add.to_string()];
        for argument in args {
            body.append(self.emit_instructions(argument).as_mut())
        }
        body.push(")".to_owned());
        body
    }

    fn emit_subtract_function(&mut self, args: &Vec<Node>) -> Vec<String> {
        let mut body = vec![Opcodes::Subtract.to_string()];
        for argument in args {
            body.append(self.emit_instructions(argument).as_mut())
        }
        body.push(")".to_owned());
        body
    }

    fn emit_print_function(&mut self, args: &Vec<Node>) -> Vec<String> {
        self.imports.push(WASIImports::FDWrite);
        let mut body = vec![];
        for argument in args {
            // build io vector
            body.push(Opcodes::Store(0, 8).to_string());
            body.push(Opcodes::Store(4, 12).to_string());
            body.push(
                SysCalls::Write(
                    Opcodes::Const(1),
                    Opcodes::Const(0),
                    Opcodes::Const(1),
                    Opcodes::Const(20),
                )
                .to_string(),
            );
            body.append(self.emit_instructions(argument).as_mut());
            body.push(Opcodes::Drop.to_string());
        }
        body
    }

    fn emit_constant(&mut self, constant: &ConstantLiteral) -> Vec<String> {
        match constant {
            ConstantLiteral::IntegerLiteral(integer) => self.emit_integer_constant(*integer),
            ConstantLiteral::StringLiteral(string) => self.emit_string_bytes(string),
        }
    }

    fn emit_integer_constant(&self, constant: i32) -> Vec<String> {
        vec![Opcodes::Const(constant).to_string()]
    }

    fn emit_string_bytes(&mut self, constant: &String) -> Vec<String> {
        let location = Opcodes::Const(8);
        let data = format!("{}\n", constant);
        self.data.push(OpData {
            location,
            data: data.parse().unwrap(),
        });
        vec![]
    }

    fn emit_memory_initializer(&self) -> String {
        String::from("(memory 1) (export \"memory\" (memory 0))")
    }
}
