use itertools::MultiPeek;
use std::str::Chars;
use std::vec::IntoIter;
use std::{error, fmt};
use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone)]
pub enum Lexeme {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Minus,
    Plus,
    Colon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    DoubleEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier(String),
    StringLiteral(String),
    NumberLiteral(i64),

    And,
    MapKey(String),
    False,
    For,
    Cond,
    Def,
    Defn,
    Nil,
    Or,
    Print,
    True,
    Main,

    Comment,
    Whitespace,
    EOF,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn reset() -> Position {
        Position { line: 1, column: 1 }
    }

    fn increment_column(&mut self) {
        self.column += 1;
    }

    fn next_line(&mut self) {
        self.line += 1;
        self.column = 1;
    }
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub lexeme: Lexeme,
    pub position: Position,
}

impl Token {
    pub fn new() -> Token {
        Token {
            lexeme: Lexeme::Whitespace,
            position: Position::reset(),
        }
    }
}

fn is_whitespace(c: char) -> bool {
    match c {
        ' ' | '\r' | '\t' | '\n' => true,
        _ => false,
    }
}

fn is_digit(c: char) -> bool {
    return c >= '0' && c <= '9';
}

fn is_alpha(c: char) -> bool {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
}

fn check_keyword(
    input_string: &String,
    index: usize,
    token_string: String,
    token: Lexeme,
) -> Lexeme {
    if input_string[index..] == token_string {
        return token;
    }

    Lexeme::Identifier(String::from(input_string))
}

#[derive(Debug)]
pub enum ScanError {
    UnknownCharacter(Position, String),
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ScanError::UnknownCharacter(ref pos, ref string) => {
                write!(f, "unknown character {:?} at {:?}", pos, string)
            }
        }
    }
}

impl error::Error for ScanError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

pub struct Scanner<'a> {
    source: MultiPeek<Chars<'a>>,
    current_string: String,
    current_position: Position,
}

impl<'a> Scanner<'a> {
    pub fn new(text: &'a String) -> Scanner<'a> {
        Scanner {
            source: itertools::multipeek(text.chars()),
            current_string: String::new(),
            current_position: Position::reset(),
        }
    }

    pub fn scan_token(&mut self) -> Result<Token, ScanError> {
        self.current_string.clear();
        match self.advance() {
            Some('(') => self.make_token(Lexeme::LeftParen),
            Some(')') => self.make_token(Lexeme::RightParen),
            Some('{') => self.make_token(Lexeme::LeftBrace),
            Some('}') => self.make_token(Lexeme::RightBrace),
            Some('[') => self.make_token(Lexeme::LeftBracket),
            Some(']') => self.make_token(Lexeme::RightBracket),
            Some(':') => {
                if self.peek_match(' ') {
                    self.make_token(Lexeme::Colon)
                } else {
                    self.make_map_key()
                }
            },
            Some(',') => self.make_token(Lexeme::Comma),
            Some('.') => self.make_token(Lexeme::Dot),
            Some('-') => self.make_token(Lexeme::Minus),
            Some('+') => self.make_token(Lexeme::Plus),
            Some('*') => self.make_token(Lexeme::Star),
            Some('!') => {
                if self.peek_match('=') {
                    self.make_token(Lexeme::BangEqual)
                } else {
                    self.make_token(Lexeme::Bang)
                }
            }
            Some('=') => {
                if self.peek_match('=') {
                    self.make_token(Lexeme::DoubleEqual)
                } else {
                    self.make_token(Lexeme::Equal)
                }
            }
            Some('>') => {
                if self.peek_match('=') {
                    self.make_token(Lexeme::GreaterEqual)
                } else {
                    self.make_token(Lexeme::Greater)
                }
            }
            Some('<') => {
                if self.peek_match('=') {
                    self.make_token(Lexeme::LessEqual)
                } else {
                    self.make_token(Lexeme::Less)
                }
            }
            Some('/') => {
                if self.peek_match('/') {
                    let token = self.make_token(Lexeme::Comment);
                    self.advance_until_newline();
                    token
                } else {
                    self.make_token(Lexeme::Slash)
                }
            }
            Some('"') => self.make_string(),
            Some(c) if is_whitespace(c) => self.make_token(Lexeme::Whitespace),
            Some(c) if is_digit(c) => self.make_digit(),
            Some(c) if is_alpha(c) => self.make_identifier(),
            None => self.make_token(Lexeme::EOF),
            _ => Err(ScanError::UnknownCharacter(
                self.current_position,
                String::from(&self.current_string),
            )),
        }
    }

    fn advance(&mut self) -> Option<char> {
        let character = self.source.next();
        if let Some(ch) = character {
            self.current_string.push(ch);
            if ch == '\n' {
                self.current_position.next_line();
            } else {
                self.current_position.increment_column();
            }
        }
        character
    }

    fn peek_match(&mut self, ch: char) -> bool {
        if self.source.peek() == Some(&ch) {
            self.source.next();
            return true;
        }
        false
    }

    fn advance_until_newline(&mut self) {
        loop {
            if let Some('\n') = self.advance() {
                break;
            }
        }
    }

    fn make_string(&mut self) -> Result<Token, ScanError> {
        // remove the starting '"'
        self.current_string.pop();
        loop {
            self.advance();
            if let Some('"') = self.source.peek() {
                break;
            }
        }
        // skip the trailing '"'
        self.source.next();
        self.make_token(Lexeme::StringLiteral(String::from(&self.current_string)))
    }

    fn make_digit(&mut self) -> Result<Token, ScanError> {
        let mut decimal_count = 1;
        loop {
            match self.source.peek() {
                // handle decimals if present
                Some('.') if decimal_count != 0 => match self.source.peek() {
                    // ensure digit after decimal is a valid number, if not we treat the
                    // decimal as a dot instead
                    Some(&ch) if is_digit(ch) => {
                        decimal_count -= 1;
                        self.advance();
                    }
                    _ => {}
                },
                Some(&c) if is_digit(c) => {
                    self.advance();
                }
                _ => break,
            }
        }

        self.make_token(Lexeme::NumberLiteral(self.current_string.parse().unwrap()))
    }

    fn make_identifier(&mut self) -> Result<Token, ScanError> {
        self.scan_word();
        let token_type = self.check_identifier_type();

        self.make_token(token_type)
    }

    fn make_map_key(&mut self) -> Result<Token, ScanError> {
        // remove the starting ':'
        self.current_string.pop();

        self.scan_word();
        self.make_token(Lexeme::MapKey(String::from(&self.current_string)))
    }

    fn scan_word(&mut self) {
        loop {
            match self.source.peek() {
                Some(&ch) if is_alpha(ch) || is_digit(ch) => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn check_identifier_type(&mut self) -> Lexeme {
        let mut current_chars = itertools::multipeek(self.current_string.chars());
        match current_chars.peek().unwrap() {
            'a' => check_keyword(&self.current_string, 1, "nd".into(), Lexeme::And),
            'f' if self.current_string.len() > 1 => match current_chars.peek().unwrap() {
                'a' => check_keyword(&self.current_string, 2, "lse".into(), Lexeme::False),
                'o' => check_keyword(&self.current_string, 2, "r".into(), Lexeme::For),
                _ => Lexeme::Identifier(String::from(&self.current_string)),
            },
            'c' => check_keyword(&self.current_string, 1, "ond".into(), Lexeme::Cond),
            'd' if self.current_string.len() > 1 => match current_chars.peek().unwrap() {
                'e' if self.current_string.len() > 2 => match current_chars.peek().unwrap() {
                    'f' => check_keyword(&self.current_string, 3, "n".into(), Lexeme::Defn),
                    _ => Lexeme::Identifier(String::from(&self.current_string)),
                },
                'e' => check_keyword(&self.current_string, 2, "f".into(), Lexeme::Def),
                _ => Lexeme::Identifier(String::from(&self.current_string)),
            },
            'm' => check_keyword(&self.current_string, 1, "ain".into(), Lexeme::Main),
            'n' => check_keyword(&self.current_string, 1, "il".into(), Lexeme::Nil),
            'o' => check_keyword(&self.current_string, 1, "r".into(), Lexeme::Or),
            'p' => check_keyword(&self.current_string, 1, "rint".into(), Lexeme::Print),
            't' => check_keyword(&self.current_string, 1, "rue".into(), Lexeme::True),
            _ => Lexeme::Identifier(String::from(&self.current_string)),
        }
    }

    fn make_token(&self, token_type: Lexeme) -> Result<Token, ScanError> {
        Ok(Token {
            lexeme: token_type,
            position: self.current_position,
        })
    }
}

pub fn scan_into_peekable(source: String) -> Result<Peekable<IntoIter<Token>>, ScanError> {
    let mut scanner = Scanner::new(&source);
    let mut tokens = Vec::new();
    loop {
        match scanner.scan_token()? {
            Token {
                lexeme: Lexeme::Whitespace,
                ..
            } => (),
            Token {
                lexeme: Lexeme::Comment,
                ..
            } => (),
            Token {
                lexeme: Lexeme::EOF,
                ..
            } => break,
            any => tokens.push(any),
        }
    }
    Ok(tokens.into_iter().peekable())
}

#[cfg(test)]
mod tests {
    use crate::frontend::scanner::Lexeme::NumberLiteral;
    use crate::frontend::scanner::Scanner;

    #[test]
    fn parse_numbers() {
        let text = "123".to_string();
        let mut scanner = Scanner::new(&text);

        assert_eq!(NumberLiteral(123 as i64), scanner.scan_token().unwrap().lexeme)
    }
}
