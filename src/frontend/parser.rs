use super::scanner::{scan_into_peekable, Lexeme, Token};
use std::vec::IntoIter;

#[derive(Debug, PartialEq)]
pub enum CompilationError {
    ScanError,
}

#[derive(Debug, PartialEq)]
pub enum ConstantType {
    IntegerLiteral(f64),
    StringLiteral(String),
}

#[derive(Debug, PartialEq)]
pub struct ConstantLiteral {
    token: Token,
    token_type: ConstantType,
}

#[derive(Debug, PartialEq)]
pub struct KeywordDetails {
    token: Token,
}

#[derive(Debug, PartialEq)]
pub struct VariableDetails {
    token: Token,
}

#[derive(Debug, PartialEq)]
pub struct ListDetails {
    head: Box<Node>,
    rest: Vec<Node>,
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Null,
    Constant(ConstantLiteral),
    Keyword(KeywordDetails),
    Variable(VariableDetails),
    List(ListDetails),
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

        return self.parse_expressions(list);
    }

    fn parse_expressions(&self, mut list: Vec<Node>) -> Result<Node, CompilationError> {
        let top =  list.remove(0);

        Ok(Node::List(ListDetails {
            head: Box::from(top),
            rest: list,
        }))
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
    use crate::frontend::parser::{
        ConstantLiteral, ConstantType, KeywordDetails, ListDetails, Node, Parser,
    };
    use crate::frontend::scanner::{Lexeme, Position, Token};

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
}
