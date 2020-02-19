use std::iter::Peekable;
use std::vec::IntoIter;
use crate::frontend::ir::{Chunk, Constant, OpCode, BinaryOp, Offset};
use crate::frontend::scanner::{Token, Position, ScanError, Lexeme, scan_into_peekable};
use crate::frontend::debug::disassemble_chunk;

type Rule = fn(&mut Parser) -> Result<(), CompilationError>;
type Prefix = Option<Rule>;
type Infix = Option<Rule>;

#[derive(Debug)]
pub enum CompilationError {
    UnknownCharacter(Position, String),
    UnexpectedToken(Token),
}

impl From<ScanError> for Vec<CompilationError> {
    fn from(err: ScanError) -> Vec<CompilationError> {
        match err {
            ScanError::UnknownCharacter(pos, string) => {
                vec![CompilationError::UnknownCharacter(pos, string)]
            }
        }
    }
}

#[derive(PartialOrd, PartialEq)]
enum Precedence {
    None,
    Assignment,
    // =
    Or,
    // or
    And,
    // and
    Equality,
    // == !=
    Comparison,
    // < > <= >=
    Term,
    // + -
    Factor,
    // * /
    Unary,
    // ! -
    Call,
    // . () []
    Primary,
}

struct Parser {
    chunk: Chunk,
    tokens: Peekable<IntoIter<Token>>,
    errors: Vec<CompilationError>,
}

impl Parser {
    fn new(chunk: Chunk, scanner_tokens: IntoIter<Token>) -> Parser {
        Parser {
            chunk,
            tokens: scanner_tokens.peekable(),
            errors: Vec::new(),
        }
    }

    fn find_rule(token: &Lexeme) -> (Prefix, Infix, Precedence) {
        match token {
            Lexeme::LeftParen => (Some(Parser::parse_group), None, Precedence::None),
            Lexeme::RightParen => (None, None, Precedence::None),
            Lexeme::LeftBrace | Lexeme::RightBrace => (None, None, Precedence::None),
            Lexeme::Minus => (
                Some(Parser::parse_unary_expression),
                Some(Parser::parse_binary_expression),
                Precedence::None,
            ),
            Lexeme::Plus | Lexeme::Slash | Lexeme::Star => (
                None,
                Some(Parser::parse_binary_expression),
                Precedence::None,
            ),
            Lexeme::NumberLiteral(_) => (Some(Parser::parse_number), None, Precedence::None),
            _ => (None, None, Precedence::None),
        }
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn consume(&mut self, token_type: Lexeme) -> Result<Token, CompilationError> {
        match self.tokens.peek() {
            Some(Token {
                     lexeme: token_type, ..
                 }) => Ok(self.advance().unwrap()),
            _ => Err(CompilationError::UnexpectedToken(self.advance().unwrap())),
        }
    }

    fn emit_byte(&mut self, byte: OpCode, line: Offset) {
        self.chunk.write(byte, line);
    }

    fn emit_constant(&mut self, constant: Constant, line: Offset) {
        let offset = self.chunk.add_constant(constant);
        match constant {
            Constant::Number(_) => self.emit_byte(OpCode::OpConstant(offset), line),
        }
    }

    fn parse(mut self) -> Result<Chunk, Vec<CompilationError>> {
        while let Some(_) = self.tokens.peek() {
            if let Err(error) = self.parse_expression() {
                self.errors.push(error);
            }
        }
        if !self.errors.is_empty() {
            Err(self.errors)
        } else {
            Ok(self.chunk)
        }
    }

    fn parse_expression(&mut self) -> Result<(), CompilationError> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<(), CompilationError> {
        let mut token = self.advance().unwrap();
        let (prefix_function, _, _) = Self::find_rule(&token.lexeme);
        let function = prefix_function.ok_or_else(|| CompilationError::UnexpectedToken(token))?;
        function(self)?;

        loop {
            let next_token = self.tokens.peek().unwrap();
            let (_, infix_function, next_precedence) = Self::find_rule(&next_token.lexeme);

            if precedence <= next_precedence {
                token = self.advance().unwrap();
                let function =
                    infix_function.ok_or_else(|| CompilationError::UnexpectedToken(token))?;
                function(self)?;
            }
        }
    }

    fn parse_number(&mut self) -> Result<(), CompilationError> {
        let token = self.advance().unwrap();
        if let Lexeme::NumberLiteral(number) = token.lexeme {
            self.emit_constant(Constant::Number(number), token.position.line)
        }
        Ok(())
    }

    fn parse_group(&mut self) -> Result<(), CompilationError> {
        self.parse_expression()?;
        self.consume(Lexeme::RightParen)?;
        Ok(())
    }

    fn parse_binary_expression(&mut self) -> Result<(), CompilationError> {
        let operator = self.advance().unwrap();

        let (_, _, precedence) = Self::find_rule(&operator.lexeme);
        self.parse_precedence(precedence)?;

        match operator.lexeme {
            Lexeme::Minus => self.emit_byte(
                OpCode::BinaryOperation(BinaryOp::Add),
                operator.position.line,
            ),
            _ => {}
        }

        Ok(())
    }

    fn parse_unary_expression(&mut self) -> Result<(), CompilationError> {
        let token = self.advance().unwrap();

        self.parse_precedence(Precedence::Unary)?;

        match token.lexeme {
            Lexeme::Minus => self.emit_byte(OpCode::OpNegate, token.position.line),
            _ => {}
        }
        Ok(())
    }
}

pub fn compile(source: String) -> Result<Chunk, Vec<CompilationError>> {
    let mut chunk = Chunk::new();
    let tokens = scan_into_peekable(source)?;

    let parser = Parser::new(chunk, tokens);
    chunk = parser.parse()?;

    disassemble_chunk(&chunk, "code");

    Ok(chunk)
}
