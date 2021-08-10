use std::cell::Cell;

use crate::expr::{Binary, Expr, Grouping, Literal, Unary};
use crate::scanner::{Token, TokenType};
use crate::LError;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    had_error: Cell<bool>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            had_error: Cell::new(false),
        }
    }
    pub fn parse(&mut self) -> Box<Expr> {
        self.expression()
    }
    fn expression(&mut self) -> Box<Expr> {
        self.equality()
    }
    // equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Box<Expr> {
        let mut expr = self.comparison();

        while self.tmatch([TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let operator = self.previous().unwrap().clone();
            let right = self.comparison();
            expr = Expr::Binary(Binary {
                left: expr,
                operator,
                right,
            })
            .into();
        }
        expr
    }
    //comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Box<Expr> {
        let mut expr = self.term();
        while self.tmatch([
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
        ]) {
            let operator = self.previous().unwrap().clone();
            let right = self.term();
            expr = Expr::Binary(Binary {
                left: expr,
                operator,
                right,
            })
            .into();
        }
        expr
    }
    fn term(&mut self) -> Box<Expr> {
        let mut expr = self.factor();
        while self.tmatch([TokenType::MINUS, TokenType::PLUS]) {
            let operator = self.previous().unwrap().clone();
            let right = self.factor();
            expr = Expr::Binary(Binary {
                left: expr,
                operator,
                right,
            })
            .into();
        }
        expr
    }
    fn factor(&mut self) -> Box<Expr> {
        let mut expr = self.unary();
        while self.tmatch([TokenType::SLASH, TokenType::STAR]) {
            let operator = self.previous().unwrap().clone();
            let right = self.unary();
            expr = Expr::Binary(Binary {
                left: expr,
                operator,
                right,
            })
            .into();
        }
        expr
    }
    //unary          → ( "!" | "-" ) unary | primary ;
    fn unary(&mut self) -> Box<Expr> {
        if self.tmatch([TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous().unwrap().clone();
            let right = self.unary();
            return Expr::Unary(Unary { operator, right }).into();
        }
        self.primary()
    }
    //primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;
    fn primary(&mut self) -> Box<Expr> {
        if self.tmatch([TokenType::FALSE]) {
            return Expr::Literal(Literal {
                value: Some("false".into()),
            })
            .into();
        }
        if self.tmatch([TokenType::TRUE]) {
            return Expr::Literal(Literal {
                value: Some("true".into()),
            })
            .into();
        }
        if self.tmatch([TokenType::NIL]) {
            return Expr::Literal(Literal {
                value: Some("null".into()),
            })
            .into();
        }

        if self.tmatch([TokenType::NUMBER, TokenType::STRING]) {
            return Expr::Literal(Literal {
                value: self.previous().unwrap().clone().literal,
            })
            .into();
        }

        if self.tmatch([TokenType::LEFT_PAREN]) {
            let expr = self.expression();
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.");
            return Expr::Grouping(Grouping { expression: expr }).into();
        }
        self.parse_error(self.peek().unwrap(), "Expected expression.");
    }
    fn consume(&mut self, ttype: TokenType, message: &'static str) -> &Token {
        if self.check(ttype) {
            self.advance()
        } else {
            self.parse_error(self.peek().unwrap(), message)
        }
    }
    fn parse_error(&self, token: &Token, message: &'static str) -> ! {
        if token.ttype == TokenType::EOF {
            self.report(token.line, Some(" at end".into()), message);
        } else {
            self.report(token.line, Some(format!(" at '{}'", token.lexeme)), message);
        }
        panic!("parser errored")
    }
    fn _synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().unwrap().ttype == TokenType::SEMICOLON {
                return;
            }
        }
        match self.peek().unwrap().ttype {
            TokenType::CLASS
            | TokenType::FUN
            | TokenType::VAR
            | TokenType::FOR
            | TokenType::IF
            | TokenType::WHILE
            | TokenType::PRINT
            | TokenType::RETURN => (),
            _ => {
                self.advance();
            }
        }
    }

    fn tmatch(&mut self, types: impl IntoIterator<Item = TokenType>) -> bool {
        for ttype in types {
            if self.check(ttype) {
                self.advance();
                return true;
            }
        }
        false
    }
    fn check(&self, ttype: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek()
            .map(|token| token.ttype == ttype)
            .unwrap_or(false)
    }
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous().unwrap()
    }
    fn is_at_end(&self) -> bool {
        self.peek()
            .map(|token| token.ttype == TokenType::EOF)
            .unwrap_or(false)
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }
    fn previous(&self) -> Option<&Token> {
        self.tokens.get(self.current - 1)
    }
}

impl LError for Parser {
    fn had_error(&self) -> &std::cell::Cell<bool> {
        &self.had_error
    }
}
