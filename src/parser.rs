use std::cell::Cell;

use crate::expr::{self, Expr};
use crate::scanner::{Token, TokenType};
use crate::stmt::{self, Stmt};
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
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = vec![];
        while !self.is_at_end() {
            stmts.push(self.declaration());
        }
        stmts
    }
    fn expression(&mut self) -> Box<expr::Expr> {
        self.assignment()
    }
    // equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Box<expr::Expr> {
        let mut expr = self.comparison();

        while self.tmatch([TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let operator = self.previous().clone();
            let right = self.comparison();
            expr = expr::Expr::Binary(expr::Binary {
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
            let operator = self.previous().clone();
            let right = self.term();
            expr = Expr::Binary(expr::Binary {
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
            let operator = self.previous().clone();
            let right = self.factor();
            expr = Expr::Binary(expr::Binary {
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
            let operator = self.previous().clone();
            let right = self.unary();
            expr = Expr::Binary(expr::Binary {
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
            let operator = self.previous().clone();
            let right = self.unary();
            return Expr::Unary(expr::Unary { operator, right }).into();
        }
        self.primary()
    }
    //primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;
    fn primary(&mut self) -> Box<Expr> {
        if self.tmatch([TokenType::FALSE]) {
            return Expr::Literal(expr::Literal {
                value: Some("false".into()),
            })
            .into();
        }
        if self.tmatch([TokenType::TRUE]) {
            return Expr::Literal(expr::Literal {
                value: Some("true".into()),
            })
            .into();
        }
        if self.tmatch([TokenType::NIL]) {
            return Expr::Literal(expr::Literal {
                value: Some("null".into()),
            })
            .into();
        }

        if self.tmatch([TokenType::NUMBER, TokenType::STRING]) {
            return Expr::Literal(expr::Literal {
                value: self.previous().clone().literal,
            })
            .into();
        }
        if self.tmatch([TokenType::IDENTIFIER]) {
            return Expr::Variable(expr::Variable {
                name: self.previous().clone(),
            })
            .into();
        }

        if self.tmatch([TokenType::LEFT_PAREN]) {
            let expr = self.expression();
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.");
            return Expr::Grouping(expr::Grouping { expression: expr }).into();
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
            if self.previous().ttype == TokenType::SEMICOLON {
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
        self.previous()
    }
    fn is_at_end(&self) -> bool {
        self.peek()
            .map(|token| token.ttype == TokenType::EOF)
            .unwrap_or(false)
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }
    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn statement(&mut self) -> Stmt {
        if self.tmatch([TokenType::PRINT]) {
            return self.print_statement();
        }
        if self.tmatch([TokenType::LEFT_BRACE]) {
            return Stmt::Block(stmt::Block {
                statements: self.block(),
            });
        }
        self.expression_statement()
    }

    fn print_statement(&mut self) -> Stmt {
        let value = self.expression();
        self.consume(TokenType::SEMICOLON, "expect ';' after value.");
        Stmt::Print(stmt::Print { expression: value })
    }

    fn expression_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.consume(TokenType::SEMICOLON, "expect ';' after expression.");
        Stmt::Expression(stmt::Expression { expression: expr })
    }

    fn declaration(&mut self) -> Stmt {
        if self.tmatch([TokenType::VAR]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn var_declaration(&mut self) -> Stmt {
        let name = self
            .consume(TokenType::IDENTIFIER, "Expect variable name.")
            .clone();

        let mut initializer = None;
        if self.tmatch([TokenType::EQUAL]) {
            initializer = Some(self.expression());
        }
        self.consume(TokenType::SEMICOLON, "Expect ; after variable declaration.");
        Stmt::Var(stmt::Var { name, initializer })
    }

    fn assignment(&mut self) -> Box<Expr> {
        let expr = self.equality();
        if self.tmatch([TokenType::EQUAL]) {
            let equals = self.previous().clone();
            let value = self.assignment();

            if let Expr::Variable(var) = *expr {
                let name = var.name;
                return Expr::Assign(expr::Assign { name, value }).into();
            }
            self.error(
                self.current,
                &format!("Invalid assignment target {}", equals),
            );
        }
        expr
    }

    fn block(&mut self) -> Vec<Stmt> {
        let mut statements = vec![];
        while !self.check(TokenType::RIGHT_BRACE) && !self.is_at_end() {
            statements.push(self.declaration());
        }
        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after block.");
        statements
    }
}

impl LError for Parser {
    fn had_error(&self) -> &std::cell::Cell<bool> {
        &self.had_error
    }
}
