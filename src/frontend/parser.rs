use super::scanner::{scan_into_peekable, Lexeme, Token};
use crate::frontend::ast::Node::Constant;
use crate::frontend::ast::{
    ConstantLiteral, FunctionDetails, KeywordDetails, ListDetails, MainDetails, MapItem, Node,
};
use crate::frontend::scanner::{Position, ScanError};
use std::iter::Peekable;
use std::option::NoneError;
use std::vec::IntoIter;

type TokenStream = Peekable<IntoIter<Token>>;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    ScanError(ScanError),
    UnexpectedEndOfFile,
    UnexpectedToken(Position, Lexeme),
    InvalidFunctionName(Position, Lexeme),
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

    pub(crate) fn parse(&self) -> Result<Vec<Node>, ParseError> {
        let mut tokens = match scan_into_peekable(self.source.to_owned()) {
            Ok(tokens) => tokens,
            Err(err) => return Err(ParseError::ScanError(err)),
        };

        let mut nodes = vec![];
        while (tokens.peek()?).lexeme != Lexeme::EOF {
            nodes.push(self.parse_token_stream(&mut tokens)?)
        }
        Ok(nodes)
    }

    fn parse_token_stream(&self, tokens: &mut TokenStream) -> Result<Node, ParseError> {
        return match tokens.next()? {
            Token {
                lexeme: Lexeme::LeftParen,
                ..
            } => self.parse_list(tokens),
            Token {
                lexeme: Lexeme::LeftBrace,
                ..
            } => self.parse_map(tokens),
            Token {
                lexeme: Lexeme::LeftBracket,
                ..
            } => self.parse_vector(tokens),
            random => Err(ParseError::UnexpectedToken(random.position, random.lexeme)),
        };
    }

    fn parse_list(&self, token_stream: &mut TokenStream) -> Result<Node, ParseError> {
        match token_stream.peek() {
            Some(Token {
                lexeme: Lexeme::Defn,
                ..
            }) => self.parse_function_definition(token_stream),
            _ => self.parse_seq_list(token_stream),
        }
    }

    fn parse_function_definition(
        &self,
        token_stream: &mut TokenStream,
    ) -> Result<Node, ParseError> {
        // dump the defn token
        token_stream.next();
        let name_token = token_stream.next()?;
        let name = match &name_token {
            Token {
                lexeme: Lexeme::Main,
                ..
            } => self.build_fake_main_node(),
            Token {
                lexeme: Lexeme::Identifier(_),
                ..
            } => self.parse_item(name_token)?,
            _ => {
                return Err(ParseError::InvalidFunctionName(
                    name_token.position,
                    name_token.lexeme,
                ))
            }
        };

        let next_element = match token_stream.next()? {
            Token {
                lexeme: Lexeme::LeftBracket,
                ..
            } => self.parse_vector(token_stream)?,
            token => return Err(ParseError::UnexpectedToken(token.position, token.lexeme)),
        };

        let args = match next_element {
            Node::Vector(arguments) => arguments,
            _ => vec![],
        };

        let body = self.parse_function_body(token_stream)?;

        match name {
            Node::Main(..) => Ok(Node::Main(MainDetails { args, body })),
            _ => Ok(Node::Function(FunctionDetails {
                name: Box::new(name),
                args,
                body,
            })),
        }
    }

    fn parse_function_body(&self, token_stream: &mut TokenStream) -> Result<Vec<Node>, ParseError> {
        let mut body = Vec::<Node>::new();
        while let Some(token) = token_stream.peek() {
            if token.lexeme == Lexeme::LeftParen {
                // move to function body
                token_stream.next();
                body.push(self.parse_seq_list(token_stream)?);
            } else {
                break;
            }
        }
        // skip trailing right parenthesis
        token_stream.next();

        Ok(body)
    }

    fn parse_seq_list(&self, token_stream: &mut TokenStream) -> Result<Node, ParseError> {
        let mut list = Vec::<Node>::new();
        while let Some(token) = token_stream.next() {
            if token.lexeme == Lexeme::RightParen {
                break;
            } else if token.lexeme == Lexeme::LeftParen {
                list.push(self.parse_seq_list(token_stream)?)
            } else {
                list.push(self.parse_item(token)?);
            }
        }
        let top = list.remove(0);
        Ok(Node::List(ListDetails {
            head: Box::from(top),
            rest: list,
        }))
    }

    fn parse_vector(&self, token_stream: &mut TokenStream) -> Result<Node, ParseError> {
        let mut list = Vec::<Node>::new();

        while let Some(token) = token_stream.next() {
            if token.lexeme == Lexeme::RightBracket {
                break;
            } else {
                list.push(self.parse_item(token)?);
            }
        }

        Ok(Node::Vector(list))
    }

    fn parse_map(&self, token_stream: &mut TokenStream) -> Result<Node, ParseError> {
        let mut map_items = Vec::<MapItem>::new();
        while let Some(token) = token_stream.next() {
            match token.lexeme {
                Lexeme::MapKey(name) => {
                    let item = match token_stream.next() {
                        Some(value) => MapItem {
                            key: name,
                            value: self.parse_item(value)?,
                        },
                        None => return Err(ParseError::UnexpectedEndOfFile),
                    };
                    map_items.push(item);
                }
                Lexeme::RightBrace => break,
                _ => {
                    map_items.push(MapItem {
                        key: String::from(""),
                        value: self.parse_item(token)?,
                    });
                }
            }
        }

        Ok(Node::Map(map_items))
    }

    fn parse_item(&self, item: Token) -> Result<Node, ParseError> {
        return match item.lexeme {
            Lexeme::NumberLiteral(number) => {
                Ok(Node::Constant(ConstantLiteral::IntegerLiteral(number)))
            }
            Lexeme::StringLiteral(string) => {
                Ok(Node::Constant(ConstantLiteral::StringLiteral(string)))
            }
            Lexeme::Plus | Lexeme::Minus | Lexeme::And | Lexeme::Or | Lexeme::Print => {
                Ok(Node::Keyword(KeywordDetails { token: item.lexeme }))
            }
            Lexeme::Identifier(name) => Ok(Node::Variable(name)),
            Lexeme::Main => Ok(Node::Variable("main".to_owned())),
            _ => Ok(Node::Null),
        };
    }

    fn build_fake_main_node(&self) -> Node {
        Node::Main(MainDetails {
            args: Vec::new(),
            body: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::ast::{
        ConstantLiteral, FunctionDetails, KeywordDetails, ListDetails, MapItem, Node,
    };
    use crate::frontend::parser::Parser;
    use crate::frontend::scanner::Lexeme;

    #[test]
    fn parse_list() {
        let text = "(+ 1 2)".to_string();
        let parser = Parser::new(&text);

        let tree = Node::List(ListDetails {
            head: Box::from(Node::Keyword(KeywordDetails {
                token: Lexeme::Plus,
            })),
            rest: vec![
                Node::Constant(ConstantLiteral::IntegerLiteral(1 as i32)),
                Node::Constant(ConstantLiteral::IntegerLiteral(2 as i32)),
            ],
        });
        let nodes = parser.parse().unwrap();

        assert_eq!(nodes[0], tree)
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
                Node::Constant(ConstantLiteral::IntegerLiteral(1 as i32)),
                Node::List(ListDetails {
                    head: Box::from(Node::Keyword(KeywordDetails {
                        token: Lexeme::Plus,
                    })),
                    rest: vec![
                        Node::Constant(ConstantLiteral::IntegerLiteral(2 as i32)),
                        Node::Constant(ConstantLiteral::IntegerLiteral(3 as i32)),
                    ],
                }),
            ],
        });

        let nodes = parser.parse().unwrap();
        assert_eq!(nodes[0], tree)
    }

    #[test]
    fn parse_map() {
        let text = "{:guten 1 :tag 2}".to_string();
        let parser = Parser::new(&text);

        let tree = Node::Map(vec![
            MapItem {
                key: "guten".to_string(),
                value: Node::Constant(ConstantLiteral::IntegerLiteral(1 as i32)),
            },
            MapItem {
                key: "tag".to_string(),
                value: Node::Constant(ConstantLiteral::IntegerLiteral(2 as i32)),
            },
        ]);

        let nodes = parser.parse().unwrap();

        assert_eq!(nodes[0], tree)
    }

    #[test]
    fn parse_vector() {
        let text = "[1 2]".to_string();
        let parser = Parser::new(&text);

        let tree = Node::Vector(vec![
            Node::Constant(ConstantLiteral::IntegerLiteral(1 as i32)),
            Node::Constant(ConstantLiteral::IntegerLiteral(2 as i32)),
        ]);

        let nodes = parser.parse().unwrap();

        assert_eq!(nodes[0], tree)
    }

    #[test]
    fn parse_function_definition() {
        let text = "(defn add [x y] (+ x y))".to_string();
        let parser = Parser::new(&text);

        let tree = Node::Function(FunctionDetails {
            name: Box::new(Node::Variable("add".to_owned())),
            args: vec![
                Node::Variable("x".to_owned()),
                Node::Variable("y".to_owned()),
            ],
            body: vec![Node::List(ListDetails {
                head: Box::from(Node::Keyword(KeywordDetails {
                    token: Lexeme::Plus,
                })),
                rest: vec![
                    Node::Variable("x".to_owned()),
                    Node::Variable("y".to_owned()),
                ],
            })],
        });

        let nodes = parser.parse().unwrap();
        assert_eq!(nodes[0], tree)
    }
}
