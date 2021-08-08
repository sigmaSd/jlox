use std::{cell::Cell, collections::HashMap, fmt::Display};

use crate::LError;
pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    had_error: Cell<bool>,
    keywords: HashMap<&'static str, TokenType>,
}
impl Scanner {
    pub fn new(code: String) -> Self {
        Self {
            source: code,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            had_error: Cell::new(false),
            keywords: Self::keywords(),
        }
    }
    fn keywords() -> HashMap<&'static str, TokenType> {
        use TokenType::*;
        vec![
            ("and", AND),
            ("class", CLASS),
            ("else", ELSE),
            ("false", FALSE),
            ("for", FOR),
            ("fun", FUN),
            ("if", IF),
            ("nil", NIL),
            ("or", OR),
            ("print", PRINT),
            ("return", RETURN),
            ("super", SUPER),
            ("this", THIS),
            ("true", TRUE),
            ("var", VAR),
            ("while", WHILE),
        ]
        .into_iter()
        .collect()
    }
    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.clone()
    }
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        use TokenType::*;
        match c {
            '(' => self.add_token(LEFT_PAREN),
            ')' => self.add_token(RIGHT_PAREN),
            '{' => self.add_token(LEFT_BRACE),
            '}' => self.add_token(RIGHT_BRACE),
            ',' => self.add_token(COMMA),
            '.' => self.add_token(DOT),
            '-' => self.add_token(MINUS),
            '+' => self.add_token(PLUS),
            ';' => self.add_token(SEMICOLON),
            '*' => self.add_token(STAR),

            // 2char
            '!' if self.next_char_is('=') => self.add_token(BANG_EQUAL),
            '!' => self.add_token(BANG),
            '=' if self.next_char_is('=') => self.add_token(EQUAL_EQUAL),
            '=' => self.add_token(EQUAL),
            '<' if self.next_char_is('=') => self.add_token(LESS_EQUAL),
            '<' => self.add_token(LESS),
            '>' if self.next_char_is('=') => self.add_token(GREATER_EQUAL),
            '>' => self.add_token(GREATER),
            '/' if self.next_char_is('/') => {
                // comment
                while self.peek() != Some('\n') && !self.is_at_end() {
                    let _ = self.advance();
                }
            }
            '/' => self.add_token(SLASH),

            // whitespace
            ' ' | '\r' | '\t' => (),

            // new line
            '\n' => self.line += 1,

            // strings
            '"' => self.string(),

            // num
            c => {
                if c.is_ldigit() {
                    self.number();
                } else if c.is_lalpha() {
                    self.identifier();
                } else {
                    self.report(self.line, None, "Unexpected character.");
                }
            }
        }
    }
    fn identifier(&mut self) {
        while self.peek().is_lalpha_numeric() {
            self.advance();
        }
        let text = &self.source[self.start..self.current];
        let ttype = if let Some(ttype) = self.keywords.get(text) {
            ttype.clone()
        } else {
            TokenType::IDENTIFIER
        };
        self.add_token(ttype);
    }
    fn number(&mut self) {
        while self.peek().is_ldigit() {
            self.advance();
        }
        if self.peek() == Some('.') && self.peek_next().is_ldigit() {
            assert_eq!(self.advance(), '.');
        }
        while self.peek().is_ldigit() {
            self.advance();
        }
        self.add_token(TokenType::NUMBER(
            self.source[self.start..self.current].parse().unwrap(),
        ));
    }
    fn string(&mut self) {
        while self.peek() != Some('"') && !self.is_at_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            self.report(self.line, None, "Unterminated string.");
            return;
        }

        assert_eq!(self.advance(), '"');

        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token(TokenType::STRING(value));
    }
    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }
        Some(self.source.as_bytes()[self.current] as char)
    }
    fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.source.len() {
            return None;
        }
        Some(self.source.as_bytes()[self.current + 1] as char)
    }
    fn next_char_is(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.as_bytes()[self.current] as char != expected {
            return false;
        }

        self.current += 1;
        true
    }
    fn advance(&mut self) -> char {
        let cchar = self.source.as_bytes()[self.current] as char;
        self.current += 1;
        cchar
    }
    fn add_token(&mut self, ttype: TokenType) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens.push(Token::new(ttype, text, self.line));
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    ttype: TokenType,
    pub lexeme: String,
    line: usize,
}
impl Token {
    pub fn new(ttype: TokenType, lexeme: String, line: usize) -> Self {
        Self {
            ttype,
            lexeme,
            line,
        }
    }
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}", self.ttype, self.lexeme)
    }
}

#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // One or two character tokens.
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,

    // Literals.
    IDENTIFIER,
    STRING(String),
    NUMBER(f64),

    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
}
impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl LError for Scanner {
    fn had_error(&self) -> &Cell<bool> {
        &self.had_error
    }
}

trait Helper {
    fn is_lalpha(&self) -> bool;
    fn is_ldigit(&self) -> bool;
    fn is_lalpha_numeric(&self) -> bool;
}
impl Helper for char {
    fn is_lalpha(&self) -> bool {
        self.is_alphabetic() || self == &'_'
    }
    fn is_ldigit(&self) -> bool {
        self.is_digit(10)
    }
    fn is_lalpha_numeric(&self) -> bool {
        self.is_ldigit() || self.is_lalpha()
    }
}
impl Helper for Option<char> {
    fn is_lalpha(&self) -> bool {
        self.map(|this| this.is_lalpha()).unwrap_or(false)
    }
    fn is_ldigit(&self) -> bool {
        self.map(|this| this.is_ldigit()).unwrap_or(false)
    }
    fn is_lalpha_numeric(&self) -> bool {
        self.map(|this| this.is_lalpha_numeric()).unwrap_or(false)
    }
}
