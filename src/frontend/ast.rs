use crate::frontend::scanner::Lexeme;

#[derive(Debug, PartialEq)]
pub enum ConstantLiteral {
    IntegerLiteral(i64),
    StringLiteral(String),
}

#[derive(Debug, PartialEq)]
pub struct KeywordDetails {
    pub token: Lexeme,
}

#[derive(Debug, PartialEq)]
pub struct VariableDetails {
    pub token: Lexeme,
}

#[derive(Debug, PartialEq)]
pub struct ListDetails {
    pub head: Box<Node>,
    pub rest: Vec<Node>,
}

#[derive(Debug, PartialEq)]
pub struct MapItem {
    pub key: String,
    pub value: Node
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Null,
    Constant(ConstantLiteral),
    Keyword(KeywordDetails),
    Variable(VariableDetails),
    Map(Vec<MapItem>),
    Vector(Vec<Node>),
    List(ListDetails),
}
