use super::scanner::{scan_into_peekable, Lexeme, Token};
use std::vec::IntoIter;
use crate::frontend::ast::{Node, ListDetails, ConstantLiteral, KeywordDetails, MapItem};

#[derive(Debug, PartialEq)]
pub enum CompilationError {
    ScanError,
}

pub(crate) struct Parser {
    source: String,
    current_token: Token,
}

impl Parser {
    pub(crate) fn new(text: &str) -> Self {
        Parser {
            source: String::from(text),
            current_token: Token::new(),
        }
    }

    pub(crate) fn parse(&self) -> Result<Node, CompilationError> {
        let mut tokens = match scan_into_peekable(self.source.to_owned()) {
            Ok(tokens) => tokens,
            Err(_) => return Err(CompilationError::ScanError),
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
            _ => Err(CompilationError::ScanError),
        };
    }

    fn parse_list(&self, token_stream: &mut IntoIter<Token>) -> Result<Node, CompilationError> {
        let mut list = Vec::<Node>::new();

        while let Some(token) = token_stream.next() {
            if token.lexeme == Lexeme::RightParen { break }
            else if token.lexeme == Lexeme::LeftParen {
                list.push(self.parse_list(token_stream)?)
            }
            else {
                list.push(self.parse_item(token)?);
            }
        };

        let top =  list.remove(0);

        Ok(Node::List(ListDetails {
            head: Box::from(top),
            rest: list,
        }))
    }

    fn parse_vector(&self, token_stream: &mut IntoIter<Token>) -> Result<Node, CompilationError> {
        let mut list = Vec::<Node>::new();

        while let Some(token) = token_stream.next() {
            if token.lexeme == Lexeme::RightBracket { break }
            else {
                list.push(self.parse_item(token)?);
            }
        };

        Ok(Node::Vector(list))
    }

    fn parse_map(&self, token_stream: &mut IntoIter<Token>) -> Result<Node, CompilationError> {
        let mut map_items = Vec::<MapItem>::new();
        while let Some(token) =  token_stream.next() {
            match token.lexeme {
                Lexeme::MapKey(name) => {
                    let item = match token_stream.next() {
                        Some(value) => MapItem { key: name, value: self.parse_item(value)? },
                        None => return Err(CompilationError::ScanError)
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

    fn parse_item(&self, item: Token) -> Result<Node, CompilationError> {
        return match item.lexeme {
            Lexeme::NumberLiteral(number) =>
                Ok(Node::Constant(ConstantLiteral::IntegerLiteral(number))),
            Lexeme::Plus | Lexeme::Minus |
            Lexeme::And | Lexeme::Or => Ok(Node::Keyword(KeywordDetails { token: item.lexeme })),
            _ => Ok(Node::Null)
        };
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
