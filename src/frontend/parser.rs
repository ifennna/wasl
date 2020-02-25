use super::scanner::{scan_into_peekable, Lexeme, Token};
use std::vec::IntoIter;
use crate::frontend::ast::{Node, ListDetails, ConstantLiteral, ConstantType, KeywordDetails, MapItem};

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
            Err(ScanError) => return Err(CompilationError::ScanError),
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
            Lexeme::NumberLiteral(number) => Ok(Node::Constant(ConstantLiteral {
                token: item,
                token_type: ConstantType::IntegerLiteral(number),
            })),
            Lexeme::Plus => Ok(Node::Keyword(KeywordDetails { token: item })),
            _ => Ok(Node::Null)
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::scanner::{Lexeme, Position, Token};
    use crate::frontend::ast::{Node, ListDetails, KeywordDetails, ConstantLiteral, ConstantType, MapItem};
    use crate::frontend::parser::Parser;

    #[test]
    fn parse_list() {
        let text = "(+ 1 2)".to_string();
        let mut parser = Parser::new(&text);

        let tree = Node::List(ListDetails {
            head: Box::from(Node::Keyword(KeywordDetails {
                token: Token {
                    lexeme: Lexeme::Plus,
                    position: Position { line: 1, column: 3 },
                },
            })),
            rest: vec![
                Node::Constant(ConstantLiteral {
                    token: Token {
                        lexeme: Lexeme::NumberLiteral(1 as f64),
                        position: Position { line: 1, column: 5 },
                    },
                    token_type: ConstantType::IntegerLiteral(1 as f64),
                }),
                Node::Constant(ConstantLiteral {
                    token: Token {
                        lexeme: Lexeme::NumberLiteral(2 as f64),
                        position: Position { line: 1, column: 7 },
                    },
                    token_type: ConstantType::IntegerLiteral(2 as f64),
                }),
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
                token: Token {
                    lexeme: Lexeme::Plus,
                    position: Position { line: 1, column: 3 },
                },
            })),
            rest: vec![
                Node::Constant(ConstantLiteral {
                    token: Token {
                        lexeme: Lexeme::NumberLiteral(1 as f64),
                        position: Position { line: 1, column: 5 },
                    },
                    token_type: ConstantType::IntegerLiteral(1 as f64),
                }),
                Node::List(ListDetails {
                    head: Box::from(Node::Keyword(KeywordDetails {
                        token: Token {
                            lexeme: Lexeme::Plus,
                            position: Position { line: 1, column: 8 },
                        },
                    })),
                    rest: vec![
                        Node::Constant(ConstantLiteral {
                            token: Token {
                                lexeme: Lexeme::NumberLiteral(2 as f64),
                                position: Position { line: 1, column: 10 },
                            },
                            token_type: ConstantType::IntegerLiteral(2 as f64),
                        }),
                        Node::Constant(ConstantLiteral {
                            token: Token {
                                lexeme: Lexeme::NumberLiteral(3 as f64),
                                position: Position { line: 1, column: 12 },
                            },
                            token_type: ConstantType::IntegerLiteral(3 as f64),
                        }),
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
            MapItem{ key: "guten".to_string(), value: Node::Constant(ConstantLiteral {
                token: Token {
                    lexeme: Lexeme::NumberLiteral(1 as f64),
                    position: Position { line: 1, column: 10 },
                },
                token_type: ConstantType::IntegerLiteral(1 as f64),
            }) },
            MapItem{ key: "tag".to_string(), value: Node::Constant(ConstantLiteral {
                token: Token {
                    lexeme: Lexeme::NumberLiteral(2 as f64),
                    position: Position { line: 1, column: 17 },
                },
                token_type: ConstantType::IntegerLiteral(2 as f64),
            }) },
        ]);

        assert_eq!(parser.parse(), Ok(tree))
    }

    #[test]
    fn parse_vector() {
        let text = "[1 2]".to_string();
        let mut parser = Parser::new(&text);

        let tree = Node::Vector(vec![
            Node::Constant(ConstantLiteral {
                token: Token {
                    lexeme: Lexeme::NumberLiteral(1 as f64),
                    position: Position { line: 1, column: 3 },
                },
                token_type: ConstantType::IntegerLiteral(1 as f64),
            }),
            Node::Constant(ConstantLiteral {
                token: Token {
                    lexeme: Lexeme::NumberLiteral(2 as f64),
                    position: Position { line: 1, column: 5 },
                },
                token_type: ConstantType::IntegerLiteral(2 as f64),
            }),
        ]);

        assert_eq!(parser.parse(), Ok(tree))
    }
}
