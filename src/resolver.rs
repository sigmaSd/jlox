use std::collections::HashMap;

use crate::interpreter::Interpreter;
use crate::{expr, stmt};

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

#[derive(Clone, Copy)]
enum FunctionType {
    None,
    Function,
    Method,
    Initializer,
}

#[derive(Clone, Copy)]
enum ClassType {
    None,
    Class,
}

impl stmt::Visit<()> for Resolver<'_> {
    fn visit_block_stmt(&mut self, stmt: &stmt::Block) {
        self.begin_scope();
        self.resolve_stmts(&stmt.statements);
        self.end_scope();
    }

    fn visit_expression_stmt(&mut self, stmt: &stmt::Expression) {
        self.resolve_expr(&stmt.expression);
    }

    fn visit_function_stmt(&mut self, stmt: &stmt::Function) {
        self.declare(&stmt.name);
        self.define(&stmt.name);
        self.resolve_function(stmt, FunctionType::Function);
    }

    fn visit_if_stmt(&mut self, stmt: &stmt::If) {
        self.resolve_expr(&stmt.condition);
        self.resolve_stmt(&stmt.then_branch);
        if let Some(ref else_branch) = stmt.else_branch {
            self.resolve_stmt(else_branch);
        }
    }

    fn visit_print_stmt(&mut self, stmt: &stmt::Print) {
        self.resolve_expr(&stmt.expression);
    }

    fn visit_var_stmt(&mut self, stmt: &stmt::Var) {
        self.declare(&stmt.name);
        if let Some(ref initializer) = stmt.initializer {
            self.resolve_expr(initializer);
        }
        self.define(&stmt.name);
    }

    fn visit_return_stmt(&mut self, stmt: &stmt::Return) {
        if matches!(self.current_function, FunctionType::None) {
            panic!("{} can't return from top-level code.", stmt.keyword)
        }
        if let Some(ref value) = stmt.value {
            if matches!(self.current_function, FunctionType::Initializer) {
                panic!("Can't return a value from an initializer.");
            }
            self.resolve_expr(value);
        }
    }

    fn visit_while_stmt(&mut self, stmt: &stmt::While) {
        self.resolve_expr(&stmt.condition);
        self.resolve_stmt(&stmt.body);
    }

    fn visit_class_stmt(&mut self, stmt: &stmt::Class) {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;

        self.define(&stmt.name);

        self.begin_scope();
        self.scopes
            .last_mut()
            .unwrap()
            .insert("this".to_string(), true);

        for method in &stmt.methods {
            let declaration = if method.name.lexeme == "init" {
                FunctionType::Initializer
            } else {
                FunctionType::Method
            };
            self.resolve_function(method, declaration);
        }
        self.end_scope();
        self.current_class = enclosing_class;
    }
}

impl expr::Visit<()> for Resolver<'_> {
    fn visit_binary_expr(&mut self, expr: &expr::Binary) {
        self.resolve_expr(&expr.left);
        self.resolve_expr(&expr.right);
    }

    fn visit_call_expr(&mut self, expr: &expr::Call) {
        self.resolve_expr(&expr.callee);
        for argument in &expr.arguemnts {
            self.resolve_expr(argument);
        }
    }

    fn visit_assign_expr(&mut self, expr: &expr::Assign) {
        self.resolve_expr(&expr.value);
        self.resolve_local(&expr::Expr::Assign(expr.clone()), &expr.name);
    }

    fn visit_grouping_expr(&mut self, expr: &expr::Grouping) {
        self.resolve_expr(&expr.expression);
    }

    fn visit_literal_expr(&mut self, _expr: &expr::Literal) {
        //noop
    }

    fn visit_logical_expr(&mut self, expr: &expr::Logical) {
        self.resolve_expr(&expr.left);
        self.resolve_expr(&expr.right);
    }

    fn visit_unary_expr(&mut self, expr: &expr::Unary) {
        self.resolve_expr(&expr.right);
    }

    fn visit_variable_expr(&mut self, expr: &expr::Variable) {
        if !self.scopes.is_empty()
            && self
                .scopes
                .last()
                .unwrap()
                .get(&expr.name.lexeme)
                .map(|initialized| initialized == &false)
                .unwrap_or(false)
        {
            panic!("Can't read local variable in its own initializer.")
        }
        self.resolve_local(&expr::Expr::Variable(expr.clone()), &expr.name);
    }

    fn visit_get_expr(&mut self, expr: &expr::Get) {
        self.resolve_expr(&expr.object);
    }

    fn visit_set_expr(&mut self, expr: &expr::Set) {
        self.resolve_expr(&expr.value);
        self.resolve_expr(&expr.object);
    }

    fn visit_this_expr(&mut self, expr: &expr::This) {
        if matches!(self.current_class, ClassType::None) {
            panic!("Can't use 'this' outside of a class.")
        }
        self.resolve_local(&expr::Expr::This(expr.clone()), &expr.keyword);
    }
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve_stmts(&mut self, statements: &[stmt::Stmt]) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    fn resolve_stmt(&mut self, statement: &stmt::Stmt) {
        statement.accept(self)
    }

    fn resolve_expr(&mut self, expr: &crate::expr::Expr) {
        expr.accept(self)
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        let _ = self.scopes.pop();
    }

    fn declare(&mut self, name: &crate::scanner::Token) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(&name.lexeme) {
            panic!("A variable with this name is already in this scope.")
        }

        scope.insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &crate::scanner::Token) {
        if self.scopes.is_empty() {
            return;
        }
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.lexeme.clone(), true);
    }

    fn resolve_local(&mut self, expr: &expr::Expr, name: &crate::scanner::Token) {
        for (depth, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, depth);

                return;
            }
        }
    }

    fn resolve_function(&mut self, function: &stmt::Function, ftype: FunctionType) {
        let enclosing_function = self.current_function;
        self.current_function = ftype;

        self.begin_scope();
        for param in &function.params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmts(&function.body);
        self.end_scope();
        self.current_function = enclosing_function;
    }
}
