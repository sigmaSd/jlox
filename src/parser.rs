use std::fmt;
use std::sync::atomic::{self, AtomicUsize};
use std::sync::Arc;

use trycatch::{catch, throw, CatchError};

use crate::expr::{self, Expr};
use crate::interpreter::{Object, ObjectInner};
use crate::scanner::{Token, TokenType};
use crate::stmt::{self, Stmt};
use crate::{downcast_exception, null_obj, obj};

#[derive(Clone, Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: Arc<AtomicUsize>,
    pub had_error: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: Default::default(),
            had_error: false,
        }
    }
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = vec![];
        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                stmts.push(stmt);
            } else {
                self.had_error = true;
            }
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
        self.call()
    }
    //primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;
    fn primary(&mut self) -> Box<Expr> {
        if self.tmatch([TokenType::FALSE]) {
            return Expr::Literal(expr::Literal {
                value: obj!(false; ObjectInner::Bool),
            })
            .into();
        }
        if self.tmatch([TokenType::TRUE]) {
            return Expr::Literal(expr::Literal {
                value: obj!(true; ObjectInner::Bool),
            })
            .into();
        }
        if self.tmatch([TokenType::NIL]) {
            return Expr::Literal(expr::Literal { value: null_obj!() }).into();
        }
        if self.tmatch([TokenType::NUMBER, TokenType::STRING]) {
            return Expr::Literal(expr::Literal {
                value: self.previous().clone().literal,
            })
            .into();
        }
        if self.tmatch(TokenType::SUPER) {
            let keyword = self.previous().clone();
            self.consume(TokenType::DOT, "Expect '.' after 'super'.");
            let method = self
                .consume(TokenType::IDENTIFIER, "Expect superclass method name.")
                .clone();
            return Expr::Super(expr::Super { keyword, method }).into();
        }
        if self.tmatch([TokenType::THIS]) {
            return Expr::This(expr::This {
                keyword: self.previous().clone(),
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
        self.throw_error(self.peek().unwrap(), "Expect expression.");
    }
    fn consume(&mut self, ttype: TokenType, message: impl fmt::Display) -> &Token {
        if self.check(ttype) {
            self.advance()
        } else {
            self.throw_error(self.peek().unwrap(), message)
        }
    }
    fn report_error(&mut self, token: &Token, message: impl fmt::Display) {
        self.had_error = true;
        let token = token;
        let message = message;
        if token.ttype == TokenType::EOF {
            eprintln!("[line {}] Error at end: {}", token.line, message);
        } else {
            eprintln!(
                "[line {}] Error at '{}': {}",
                token.line, token.lexeme, message
            );
        }
    }
    fn throw_error(&self, token: &Token, message: impl fmt::Display) -> ! {
        let token = token;
        let message = message;
        if token.ttype == TokenType::EOF {
            throw(format!("[line {}] Error at end: {}", token.line, message));
        } else {
            throw(format!(
                "[line {}] Error at '{}': {}",
                token.line, token.lexeme, message
            ));
        }
    }
    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().ttype == TokenType::SEMICOLON {
                return;
            }
            match self.peek().unwrap().ttype {
                TokenType::CLASS
                | TokenType::FUN
                | TokenType::VAR
                | TokenType::FOR
                | TokenType::IF
                | TokenType::WHILE
                | TokenType::PRINT
                | TokenType::RETURN => return,
                _ => (),
            }
            self.advance();
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
        self.peek().unwrap().ttype == ttype
    }
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current.store(
                self.current.load(atomic::Ordering::Relaxed) + 1,
                atomic::Ordering::Relaxed,
            );
        }
        self.previous()
    }
    fn is_at_end(&self) -> bool {
        self.peek()
            .map(|token| token.ttype == TokenType::EOF)
            .unwrap_or(false)
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens
            .get(self.current.load(atomic::Ordering::Relaxed))
    }
    fn previous(&self) -> &Token {
        let t = self.current.load(atomic::Ordering::Relaxed) - 1;
        self.tokens.get(t).unwrap()
    }

    fn statement(&mut self) -> Stmt {
        if self.tmatch([TokenType::FOR]) {
            return self.for_statement();
        }
        if self.tmatch([TokenType::IF]) {
            return self.if_statement();
        }
        if self.tmatch([TokenType::PRINT]) {
            return self.print_statement();
        }
        if self.tmatch([TokenType::RETURN]) {
            return self.return_statement();
        }
        if self.tmatch([TokenType::WHILE]) {
            return self.while_statement();
        }
        if self.tmatch([TokenType::LEFT_BRACE]) {
            return Stmt::Block(stmt::Block {
                statements: self.block(),
            });
        }
        self.expression_statement()
    }

    fn print_statement(&mut self) -> Stmt {
        let value = *self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.");
        Stmt::Print(stmt::Print { expression: value })
    }

    fn expression_statement(&mut self) -> Stmt {
        let expr = *self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after expression.");
        Stmt::Expression(stmt::Expression { expression: expr })
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let mut parser = self.clone();

        match catch(move || {
            if parser.tmatch([TokenType::CLASS]) {
                let val = parser.class_declaration();
                return (parser, val);
            }
            if parser.tmatch([TokenType::FUN]) {
                let val = Stmt::Function(parser.function("function"));
                return (parser, val);
            }
            if parser.tmatch([TokenType::VAR]) {
                let val = parser.var_declaration();
                return (parser, val);
            }
            let val = parser.statement();
            (parser, val)
        }) {
            Ok((parser, stmt)) => {
                *self = parser;
                Some(stmt)
            }
            Err(CatchError::Exception(e)) => {
                downcast_exception!(65, e => &'static str String);
                self.synchronize();
                None
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn var_declaration(&mut self) -> Stmt {
        let name = self
            .consume(TokenType::IDENTIFIER, "Expect variable name.")
            .clone();

        let mut initializer = None;
        if self.tmatch([TokenType::EQUAL]) {
            initializer = Some(*self.expression());
        }
        self.consume(TokenType::SEMICOLON, "Expect ; after variable declaration.");
        Stmt::Var(stmt::Var { name, initializer })
    }

    fn assignment(&mut self) -> Box<Expr> {
        let expr = self.or();
        if self.tmatch([TokenType::EQUAL]) {
            let equals = self.previous().clone();
            let value = self.assignment();

            match *expr {
                Expr::Variable(var) => {
                    let name = var.name;
                    return Expr::Assign(expr::Assign { name, value }).into();
                }
                Expr::Get(get) => {
                    return Expr::Set(expr::Set {
                        object: get.object,
                        name: get.name,
                        value,
                    })
                    .into();
                }
                _ => {
                    self.report_error(&equals, "Invalid assignment target.");
                }
            }
        }
        expr
    }

    fn block(&mut self) -> Vec<Stmt> {
        let mut statements = vec![];
        while !self.check(TokenType::RIGHT_BRACE) && !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            } else {
                self.had_error = true;
            }
        }
        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after block.");
        statements
    }

    fn if_statement(&mut self) -> Stmt {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'if'.");
        let condition = *self.expression();
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after if condition.");

        let then_branch = self.statement().into();
        let mut else_branch = None;
        if self.tmatch([TokenType::ELSE]) {
            else_branch = Some(self.statement().into());
        }
        Stmt::If(stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn or(&mut self) -> Box<Expr> {
        let mut expr = self.and();
        while self.tmatch([TokenType::OR]) {
            let operator = self.previous().clone();
            let right = self.and();
            expr = Expr::Logical(expr::Logical {
                left: expr,
                operator,
                right,
            })
            .into();
        }
        expr
    }

    fn and(&mut self) -> Box<Expr> {
        let mut expr = self.equality();
        while self.tmatch([TokenType::AND]) {
            let operator = self.previous().clone();
            let right = self.equality();
            expr = Expr::Logical(expr::Logical {
                left: expr,
                operator,
                right,
            })
            .into();
        }
        expr
    }

    fn while_statement(&mut self) -> Stmt {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'while'");
        let condition = *self.expression();
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after condition");
        let body = self.statement().into();

        Stmt::While(stmt::While { condition, body })
    }

    fn for_statement(&mut self) -> Stmt {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'for'");
        let initializer = if self.tmatch(TokenType::SEMICOLON) {
            None
        } else if self.tmatch(TokenType::VAR) {
            Some(self.var_declaration())
        } else {
            Some(self.expression_statement())
        };

        //FIXME
        //self.consume(TokenType::SEMICOLON, "Expect ';' after loop condition.");
        let condition = if !self.check(TokenType::SEMICOLON) {
            Some(self.expression())
        } else {
            None
        };
        self.consume(TokenType::SEMICOLON, "Expect ';' after loop condition.");

        let increment = if !self.check(TokenType::RIGHT_PAREN) {
            Some(self.expression())
        } else {
            None
        };
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after clauses.");

        let mut body = self.statement();

        if let Some(increment) = increment {
            body = Stmt::Block(stmt::Block {
                statements: vec![
                    body,
                    Stmt::Expression(stmt::Expression {
                        expression: *increment,
                    }),
                ],
            });
        }
        let condition = if let Some(condition) = condition {
            *condition
        } else {
            Expr::Literal(expr::Literal {
                value: obj!(true; ObjectInner::Bool),
            })
        };

        body = Stmt::While(stmt::While {
            condition,
            body: body.into(),
        });

        if let Some(initializer) = initializer {
            body = Stmt::Block(stmt::Block {
                statements: vec![initializer, body],
            });
        }

        body
    }

    fn call(&mut self) -> Box<Expr> {
        let mut expr = self.primary();

        loop {
            if self.tmatch(TokenType::LEFT_PAREN) {
                expr = self.finish_call(expr).into();
            } else if self.tmatch(TokenType::DOT) {
                let name = self
                    .consume(TokenType::IDENTIFIER, "Expect property name after '.'.")
                    .clone();
                expr = Expr::Get(expr::Get { object: expr, name }).into();
            } else {
                break;
            }
        }
        expr
    }

    fn finish_call(&mut self, callee: Box<Expr>) -> Expr {
        let mut arguemnts = vec![];
        if !self.check(TokenType::RIGHT_PAREN) {
            arguemnts.push(*self.expression());
            while self.tmatch(TokenType::COMMA) {
                if arguemnts.len() >= 255 {
                    self.throw_error(self.peek().unwrap(), "Can't have more than 255 arguments.");
                }
                arguemnts.push(*self.expression());
            }
        }
        let paren = self
            .consume(TokenType::RIGHT_PAREN, "Expect ')' after arguments.")
            .clone();

        Expr::Call(expr::Call {
            callee,
            paren,
            arguemnts,
        })
    }

    fn function(&mut self, kind: &str) -> stmt::Function {
        let name = self
            .consume(TokenType::IDENTIFIER, format!("Expect {} name.", kind))
            .clone();
        self.consume(
            TokenType::LEFT_PAREN,
            format!("Expect '(' after {} name.", kind),
        );
        let mut params = vec![];
        if !self.check(TokenType::RIGHT_PAREN) {
            params.push(
                self.consume(TokenType::IDENTIFIER, "Expect parameter name.")
                    .clone(),
            );
            while self.tmatch(TokenType::COMMA) {
                if params.len() >= 255 {
                    self.throw_error(self.peek().unwrap(), "Can't have more than 255 parameters.");
                }
                params.push(
                    self.consume(TokenType::IDENTIFIER, "Expect parameter name.")
                        .clone(),
                );
            }
        }
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after parameters.");
        self.consume(
            TokenType::LEFT_BRACE,
            format!("Expect '{{' before {} body.", kind),
        );

        let body = self.block();
        stmt::Function { name, params, body }
    }

    fn return_statement(&mut self) -> Stmt {
        let keyword = self.previous().clone();
        let mut value = None;
        if !self.check(TokenType::SEMICOLON) {
            value = Some(*self.expression());
        }
        self.consume(TokenType::SEMICOLON, "Expect ';' after return value.");
        Stmt::Return(stmt::Return { keyword, value })
    }

    fn class_declaration(&mut self) -> Stmt {
        let name = self
            .consume(TokenType::IDENTIFIER, "Expect class name")
            .clone();

        let mut superclass = None;
        if self.tmatch(TokenType::LESS) {
            self.consume(TokenType::IDENTIFIER, "Expect superclass name.");
            superclass = Some(expr::Variable {
                name: self.previous().clone(),
            });
        }

        self.consume(TokenType::LEFT_BRACE, "Expect '{' before class body.");

        let mut methods = vec![];
        while !self.check(TokenType::RIGHT_BRACE) && !self.is_at_end() {
            methods.push(self.function("method"));
        }
        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after class body.");
        Stmt::Class(stmt::Class {
            name,
            superclass,
            methods,
        })
    }
}
