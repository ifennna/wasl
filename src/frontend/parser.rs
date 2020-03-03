use super::scanner::{scan_into_peekable, Lexeme, Token};
use std::vec::IntoIter;
use crate::frontend::ast::{Node, ListDetails, ConstantLiteral, KeywordDetails, MapItem, FunctionDetails, MainDetails};
use crate::frontend::scanner::Position;
use std::option::NoneError;
use std::iter::Peekable;

type TokenStream = Peekable<IntoIter<Token>>;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    ScanError,
    UnexpectedEndOfFile,
    UnexpectedToken(Position, Lexeme),
    InvalidFunctionName(Position, Lexeme)
}

impl From<NoneError> for ParseError {
    fn from(_: NoneError) -> Self {
        ParseError::UnexpectedEndOfFile
    }
}

pub(crate) struct Parser {
    source: String,
}

impl Parser {
    pub(crate) fn new(text: &str) -> Self {
        Parser {
            source: String::from(text),
        }
    }

    pub(crate) fn parse(&self) -> Result<Node, ParseError> {
        let mut tokens = match scan_into_peekable(self.source.to_owned()) {
            Ok(tokens) => tokens,
            Err(_) => return Err(ParseError::ScanError),
        };

        return match tokens.next() {
            Some(Token {
                lexeme: Lexeme::LeftParen,
                ..
            }) => self.parse_list(&mut tokens),
            Some(Token {
                lexeme: Lexeme::LeftBrace,
                ..
            }) => self.parse_map(&mut tokens),
            Some(Token {
                 lexeme: Lexeme::LeftBracket,
                 ..
            }) => self.parse_vector(&mut tokens),
            _ => Err(ParseError::ScanError),
        };
    }

    fn parse_list(&self, token_stream: &mut TokenStream) -> Result<Node, ParseError> {
        match token_stream.peek() {
            Some(Token{lexeme: Lexeme::Defn, ..}) => self.parse_function_definition(token_stream),
            _ => self.parse_seq_list(token_stream)
        }
    }

    fn parse_function_definition(&self, token_stream: &mut TokenStream) -> Result<Node, ParseError> {
        // dump the defn token
        token_stream.next();
        let name_token = token_stream.next()?;
        let name = match &name_token {
            Token{lexeme: Lexeme::Main, ..} => self.build_fake_main_node(),
            Token{lexeme: Lexeme::Identifier(_), ..} => self.parse_item(name_token)?,
            _ => return Err(ParseError::InvalidFunctionName(name_token.position, name_token.lexeme))
        };
        let mut args = Vec::new();
        let body;

        let next_element = match token_stream.next()? {
            Token{lexeme: Lexeme::LeftBracket, ..} => self.parse_vector(token_stream)?,
            Token{lexeme: Lexeme::LeftParen, ..} => self.parse_list(token_stream)?,
            token => return Err(ParseError::UnexpectedToken(token.position, token.lexeme))
        };

        match next_element {
            Node::Vector(arguments) => {
                println!("{:?}", arguments);
                args = arguments;
                token_stream.next();
                body = self.parse_list(token_stream)?;
            }
            _ => body = next_element
        };

        match name {
            Node::Main(..) => Ok(Node::Main(MainDetails{ args, body: Box::new(body) })),
            _ => Ok(Node::Function(FunctionDetails{name: Box::new(name), args, body: Box::new(body) }))
        }
    }

    fn parse_seq_list(&self, token_stream: &mut TokenStream) -> Result<Node, ParseError> {
        let mut list = Vec::<Node>::new();
        while let Some(token) = token_stream.next() {
            if token.lexeme == Lexeme::RightParen { break }
            else if token.lexeme == Lexeme::LeftParen {
                list.push(self.parse_list(token_stream)?)
            } else {
                list.push(self.parse_item(token)?);
            }
        };
        let top = list.remove(0);
        Ok(Node::List(ListDetails {
            head: Box::from(top),
            rest: list,
        }))
    }

    fn parse_vector(&self, token_stream: &mut TokenStream) -> Result<Node, ParseError> {
        let mut list = Vec::<Node>::new();

        while let Some(token) = token_stream.next() {
            if token.lexeme == Lexeme::RightBracket { break }
            else {
                list.push(self.parse_item(token)?);
            }
        };

        Ok(Node::Vector(list))
    }

    fn parse_map(&self, token_stream: &mut TokenStream) -> Result<Node, ParseError> {
        let mut map_items = Vec::<MapItem>::new();
        while let Some(token) =  token_stream.next() {
            match token.lexeme {
                Lexeme::MapKey(name) => {
                    let item = match token_stream.next() {
                        Some(value) => MapItem { key: name, value: self.parse_item(value)? },
                        None => return Err(ParseError::ScanError)
                    };
                    map_items.push(item);
                },
                Lexeme::RightBrace => { break }
                _ => {
                    map_items.push(MapItem {
                        key: String::from(""),
                        value: self.parse_item(token) ?
                    });
                }
            }
        }

        Ok(Node::Map(map_items))
    }

    fn parse_item(&self, item: Token) -> Result<Node, ParseError> {
        return match item.lexeme {
            Lexeme::NumberLiteral(number) =>
                Ok(Node::Constant(ConstantLiteral::IntegerLiteral(number))),
            Lexeme::Plus | Lexeme::Minus |
            Lexeme::And | Lexeme::Or |
            Lexeme::Print => Ok(Node::Keyword(KeywordDetails { token: item.lexeme })),
            Lexeme::Identifier(name) => Ok(Node::Variable(name)),
            _ => Ok(Node::Null)
        };
    }

    fn build_fake_main_node(&self) -> Node {
        Node::Main(MainDetails{
            args: Vec::new(),
            body: Box::new(Node::Null)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::scanner::{Lexeme};
    use crate::frontend::ast::{Node, ListDetails, KeywordDetails, ConstantLiteral, MapItem};
    use crate::frontend::parser::Parser;

    #[test]
    fn parse_list() {
        let text = "(+ 1 2)".to_string();
        let mut parser = Parser::new(&text);

        let tree = Node::List(ListDetails {
            head: Box::from(Node::Keyword(KeywordDetails {
                token: Lexeme::Plus,
            })),
            rest: vec![
                Node::Constant(ConstantLiteral::IntegerLiteral(1 as i64)),
                Node::Constant(ConstantLiteral::IntegerLiteral(2 as i64)),
            ],
        });

        assert_eq!(parser.parse(), Ok(tree))
    }

    #[test]
    fn parse_nested_list() {
        let text = "(+ 1 (+ 2 3))".to_string();
        let mut parser = Parser::new(&text);

        let tree = Node::List(ListDetails {
            head: Box::from(Node::Keyword(KeywordDetails {
                token: Lexeme::Plus,
            })),
            rest: vec![
                Node::Constant(ConstantLiteral::IntegerLiteral(1 as i64)),
                Node::List(ListDetails {
                    head: Box::from(Node::Keyword(KeywordDetails {
                        token: Lexeme::Plus,
                    })),
                    rest: vec![
                        Node::Constant(ConstantLiteral::IntegerLiteral(2 as i64)),
                        Node::Constant(ConstantLiteral::IntegerLiteral(3 as i64)),
                    ],
                }),
            ],
        });

        assert_eq!(parser.parse(), Ok(tree))
    }

    #[test]
    fn parse_map() {
        let text = "{:guten 1 :tag 2}".to_string();
        let mut parser = Parser::new(&text);

        let tree = Node::Map(vec![
            MapItem{
                key: "guten".to_string(),
                value: Node::Constant(ConstantLiteral::IntegerLiteral(1 as i64)), },
            MapItem{
                key: "tag".to_string(),
                value: Node::Constant(ConstantLiteral::IntegerLiteral(2 as i64)), },
        ]);

        assert_eq!(parser.parse(), Ok(tree))
    }

    #[test]
    fn parse_vector() {
        let text = "[1 2]".to_string();
        let mut parser = Parser::new(&text);

        let tree = Node::Vector(vec![
        Node::Constant(ConstantLiteral::IntegerLiteral(1 as i64)),
        Node::Constant(ConstantLiteral::IntegerLiteral(2 as i64)),
        ]);

        assert_eq!(parser.parse(), Ok(tree))
    }
}
