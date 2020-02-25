use crate::frontend::scanner::Token;

#[derive(Debug, PartialEq)]
pub enum ConstantType {
    IntegerLiteral(f64),
    StringLiteral(String),
}

#[derive(Debug, PartialEq)]
pub struct ConstantLiteral {
    pub token: Token,
    pub token_type: ConstantType,
}

#[derive(Debug, PartialEq)]
pub struct KeywordDetails {
    pub token: Token,
}

#[derive(Debug, PartialEq)]
pub struct VariableDetails {
    pub token: Token,
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
