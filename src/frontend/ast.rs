use crate::frontend::scanner::Lexeme;

type VariableName = String;

#[derive(Debug, PartialEq)]
pub enum ConstantLiteral {
    IntegerLiteral(i32),
    StringLiteral(String),
}

#[derive(Debug, PartialEq)]
pub struct KeywordDetails {
    pub token: Lexeme,
}

#[derive(Debug, PartialEq)]
pub struct ListDetails {
    pub head: Box<Node>,
    pub rest: Vec<Node>,
}

#[derive(Debug, PartialEq)]
pub struct FunctionDetails {
    pub name: Box<Node>,
    pub args: Vec<Node>,
    pub body: Box<Node>,
}

#[derive(Debug, PartialEq)]
pub struct MainDetails {
    pub args: Vec<Node>,
    pub body: Box<Node>,
}

#[derive(Debug, PartialEq)]
pub struct VariableInformation {
    pub name: Box<Node>,
    pub value: Box<Node>,
}

#[derive(Debug, PartialEq)]
pub struct MapItem {
    pub key: String,
    pub value: Node
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Null,
    Main(MainDetails),
    Def(VariableInformation),
    Function(FunctionDetails),
    Constant(ConstantLiteral),
    Keyword(KeywordDetails),
    Variable(VariableName),
    Map(Vec<MapItem>),
    Vector(Vec<Node>),
    List(ListDetails),
}
