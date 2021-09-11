use crate::interpreter::object::function::{Clock, LoxFunction};
use crate::scanner::Token;
use crate::{ar, downcast, null_obj};
use crate::{expr, obj, scanner::TokenType, stmt};
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::{Arc, RwLock};

mod environment;
use environment::Environment;
mod object;
pub use object::{class::LoxClass, Object, ObjectInner};

use trycatch::{throw, Exception};

#[derive(Clone, Debug)]
pub struct Interpreter {
    environment: Arc<RwLock<Environment>>,
    globals: Arc<RwLock<Environment>>,
    pub locals: Arc<RwLock<HashMap<expr::Expr, usize>>>,
}

impl stmt::Visit<()> for Interpreter {
    fn visit_expression_stmt(&mut self, stmt: &stmt::Expression) {
        self.evaluate(&stmt.expression);
    }

    fn visit_print_stmt(&mut self, stmt: &stmt::Print) {
        let value = self.evaluate(&stmt.expression);
        println!("{}", stringify(value));
    }

    fn visit_var_stmt(&mut self, stmt: &stmt::Var) {
        let value = if let Some(ref initializer) = stmt.initializer {
            self.evaluate(initializer)
        } else {
            null_obj!()
        };
        self.environment
            .try_write()
            .unwrap()
            .define(stmt.name.lexeme.clone(), Some(value));
    }

    fn visit_block_stmt(&mut self, stmt: &stmt::Block) {
        self.execute_block(
            &stmt.statements,
            Environment::new(Some(self.environment.clone())),
        );
    }

    fn visit_if_stmt(&mut self, stmt: &stmt::If) {
        if is_truthy(&self.evaluate(&stmt.condition)) {
            self.execute(&stmt.then_branch)
        } else if let Some(ref else_stmt) = stmt.else_branch {
            self.execute(else_stmt)
        }
    }

    fn visit_while_stmt(&mut self, stmt: &stmt::While) {
        while is_truthy(&self.evaluate(&stmt.condition)) {
            self.execute(&stmt.body)
        }
    }

    fn visit_function_stmt(&mut self, stmt: &stmt::Function) {
        let function = LoxFunction::new(stmt.clone(), self.environment.clone(), false);
        self.environment.try_write().unwrap().define(
            stmt.name.lexeme.clone(),
            Some(obj!(function; @rr ObjectInner::Function)),
        );
    }

    fn visit_return_stmt(&mut self, stmt: &stmt::Return) {
        let value = if let Some(ref value) = stmt.value {
            self.evaluate(value)
        } else {
            null_obj!()
        };
        throw(ReturnException(value));
    }

    fn visit_class_stmt(&mut self, stmt: &stmt::Class) {
        let superclass = if let Some(ref superclass) = stmt.superclass {
            let superclass = self.evaluate(&superclass.clone().into());
            if !superclass.is_class() {
                throw(RuntimeError::new(
                    stmt.clone().superclass.unwrap().name,
                    "Superclass must be a class.",
                ))
            }
            Some(superclass)
        } else {
            None
        };

        self.environment
            .try_write()
            .unwrap()
            .define(stmt.name.lexeme.clone(), None);

        if let Some(ref superclass) = superclass {
            let mut environment = Environment::new(Some(self.environment.clone()));
            environment.define("super".into(), Some(superclass.clone()));
            self.environment = Arc::new(RwLock::new(environment));
        }

        let mut methods = HashMap::new();
        for method in &stmt.methods {
            let function = LoxFunction::new(
                method.clone(),
                self.environment.clone(),
                method.name.lexeme == "init",
            );
            methods.insert(method.name.lexeme.clone(), function);
        }

        let class = ar!(ObjectInner::Class(LoxClass::new(
            stmt.name.lexeme.clone(),
            superclass.map(|class| downcast!(class => ObjectInner::Class)),
            methods,
        )));
        if stmt.superclass.is_some() {
            self.environment = self
                .environment
                .clone()
                .try_read()
                .unwrap()
                .enclosing
                .clone()
                .unwrap();
        }

        self.environment
            .try_write()
            .unwrap()
            .assign(stmt.name.clone(), class);
    }
}

impl expr::Visit<Object> for Interpreter {
    fn visit_binary_expr(&mut self, expr: &crate::expr::Binary) -> Object {
        let right = self.evaluate(&expr.right);
        let left = self.evaluate(&expr.left);

        match expr.operator.ttype {
            TokenType::MINUS => {
                check_number_operands(&expr.operator, [&left, &right]);
                return obj!(downcast!(left => ObjectInner::Number) - downcast!(right => ObjectInner::Number) ; ObjectInner::Number);
            }
            TokenType::PLUS => {
                if left.is_num() && right.is_num() {
                    return obj!(downcast!(left => ObjectInner::Number) + downcast!(right => ObjectInner::Number) ; ObjectInner::Number);
                }
                if left.is_str() && right.is_str() {
                    return obj!(downcast!(left => ObjectInner::String) + &downcast!(right => ObjectInner::String) ; ObjectInner::String);
                }
                throw(RuntimeError::new(
                    expr.operator.clone(),
                    "Operands must be two numbers or two strings.",
                ))
            }
            TokenType::SLASH => {
                check_number_operands(&expr.operator, [&left, &right]);
                return obj!(downcast!(left => ObjectInner::Number) / downcast!(right => ObjectInner::Number) ; ObjectInner::Number);
            }
            TokenType::STAR => {
                check_number_operands(&expr.operator, [&left, &right]);
                return obj!(downcast!(left => ObjectInner::Number) * downcast!(right => ObjectInner::Number) ; ObjectInner::Number);
            }
            TokenType::GREATER => {
                check_number_operands(&expr.operator, [&left, &right]);
                return obj!(downcast!(left => ObjectInner::Number) > downcast!(right => ObjectInner::Number) ; ObjectInner::Bool);
            }
            TokenType::GREATER_EQUAL => {
                check_number_operands(&expr.operator, [&left, &right]);
                return obj!(downcast!(left => ObjectInner::Number) >= downcast!(right => ObjectInner::Number) ; ObjectInner::Bool);
            }
            TokenType::LESS => {
                check_number_operands(&expr.operator, [&left, &right]);
                return obj!(downcast!(left => ObjectInner::Number) < downcast!(right => ObjectInner::Number) ; ObjectInner::Bool);
            }
            TokenType::LESS_EQUAL => {
                check_number_operands(&expr.operator, [&left, &right]);
                return obj!(downcast!(left => ObjectInner::Number) <= downcast!(right => ObjectInner::Number) ; ObjectInner::Bool);
            }
            TokenType::BANG_EQUAL => return obj!(!is_equal(left, right) ; ObjectInner::Bool),
            TokenType::EQUAL_EQUAL => return obj!(is_equal(left, right) ; ObjectInner::Bool),
            _ => unreachable!(),
        }
    }

    fn visit_grouping_expr(&mut self, expr: &crate::expr::Grouping) -> Object {
        self.evaluate(&expr.expression)
    }

    fn visit_literal_expr(&mut self, expr: &crate::expr::Literal) -> Object {
        expr.value.clone()
    }

    fn visit_unary_expr(&mut self, expr: &crate::expr::Unary) -> Object {
        let right = self.evaluate(&expr.right);
        match expr.operator.ttype {
            TokenType::MINUS => {
                check_number_operands(&expr.operator, [&right]);
                return obj!(-downcast!(right =>ObjectInner::Number); ObjectInner::Number);
            }
            TokenType::BANG => return obj!(!is_truthy(&right); ObjectInner::Bool),

            _ => unreachable!(),
        }
    }

    fn visit_variable_expr(&mut self, expr: &expr::Variable) -> Object {
        self.lookup_variable(&expr.name, &expr::Expr::Variable(expr.clone()))
    }

    fn visit_assign_expr(&mut self, expr: &expr::Assign) -> Object {
        let value = self.evaluate(&expr.value);
        let distance = self.locals.try_read();

        let distance = distance
            .as_ref()
            .unwrap()
            .get(&expr::Expr::Assign(expr.clone()));
        if let Some(distance) = distance {
            self.environment.try_write().unwrap().assign_at(
                distance,
                expr.name.clone(),
                value.clone(),
            );
        } else {
            self.globals
                .try_write()
                .unwrap()
                .assign(expr.name.clone(), value.clone());
        }
        value
    }

    fn visit_logical_expr(&mut self, expr: &expr::Logical) -> Object {
        let left = self.evaluate(&expr.left);

        if expr.operator.ttype == TokenType::OR {
            if is_truthy(&left) {
                return left;
            }
        } else if !is_truthy(&left) {
            return left;
        }

        self.evaluate(&expr.right)
    }

    fn visit_call_expr(&mut self, expr: &expr::Call) -> Object {
        let callee = self.evaluate(&expr.callee);

        let mut arguemnts = vec![];
        for arguemnt in &expr.arguemnts {
            arguemnts.push(self.evaluate(arguemnt));
        }

        if !callee.is_fun() {
            throw(RuntimeError::new(
                expr.paren.clone(),
                "Can only call functions and classes.",
            ))
        }

        let function = crate::downcast_to_lox_callable!(callee);
        if arguemnts.len() != function.try_read().unwrap().arity() {
            throw(RuntimeError::new(
                expr.paren.clone(),
                format!(
                    "Expected {} arguments but got {}.\n",
                    function.try_read().unwrap().arity(),
                    arguemnts.len(),
                ),
            ))
        }

        function.clone().try_read().unwrap().call(self, arguemnts)
    }

    fn visit_get_expr(&mut self, expr: &expr::Get) -> Object {
        let object = self.evaluate(&expr.object);
        if let ObjectInner::Instance(instance) = object.0 {
            return instance.get(&expr.name);
        }
        throw(RuntimeError::new(
            expr.name.clone(),
            "Only instances have properties.",
        ))
    }

    fn visit_set_expr(&mut self, expr: &expr::Set) -> Object {
        let object = self.evaluate(&expr.object);
        if let ObjectInner::Instance(mut instance) = object.0 {
            let value = self.evaluate(&expr.value);
            instance.set(expr.name.clone(), value.clone());
            return value;
        }
        throw(RuntimeError::new(
            expr.name.clone(),
            "Only instances have fields.",
        ))
    }

    fn visit_this_expr(&mut self, expr: &expr::This) -> Object {
        self.lookup_variable(&expr.keyword, &expr::Expr::This(expr.clone()))
    }

    fn visit_super_expr(&mut self, expr: &expr::Super) -> Object {
        let distance = self.locals.try_read();
        let distance = distance
            .as_ref()
            .unwrap()
            .get(&expr.clone().into())
            .unwrap();
        let superclass = self
            .environment
            .try_read()
            .unwrap()
            .get_at(distance, "super");
        let object = self
            .environment
            .try_read()
            .unwrap()
            .get_at(&(*distance - 1), "this");
        if let Some(method) =
            downcast!(superclass => ObjectInner::Class).find_method(&expr.method.lexeme)
        {
            ar!(ObjectInner::Function(Arc::new(RwLock::new(
                method.bind(downcast!( object => ObjectInner::Instance)),
            ))))
        } else {
            throw(RuntimeError::new(
                expr.method.clone(),
                format!("Undefined property '{}'.", expr.method.lexeme,),
            ))
        }
    }
}

fn check_number_operands<'a>(
    operator: &crate::scanner::Token,
    operators: impl IntoIterator<Item = &'a Object>,
) {
    let operators: Vec<_> = operators.into_iter().collect();
    if operators.iter().all(|obj| obj.is_num()) {
        return;
    }
    if operators.len() > 1 {
        throw(RuntimeError::new(
            operator.clone(),
            "Operands must be numbers.",
        ));
    } else {
        throw(RuntimeError::new(
            operator.clone(),
            "Operand must be a number.",
        ));
    }
}
fn is_equal(right: Object, left: Object) -> bool {
    right == left
}

fn is_truthy(right: &Object) -> bool {
    if right.is_null() {
        return false;
    }
    crate::try_downcast!(right => ObjectInner::Bool).unwrap_or(true)
}

impl Default for Interpreter {
    fn default() -> Self {
        let globals = Arc::new(RwLock::new(Environment::new(None)));
        let environment = globals.clone();

        globals.try_write().unwrap().define(
            "clock".into(),
            Some(obj!(Clock{}; @rr ObjectInner::Function)),
        );

        Self {
            globals,
            environment,
            locals: Default::default(),
        }
    }
}
impl Interpreter {
    fn evaluate(&mut self, expression: &crate::expr::Expr) -> Object {
        expression.accept(self)
    }
    pub fn interpret(&mut self, statements: Vec<crate::stmt::Stmt>) {
        for stmt in statements {
            self.execute(&stmt);
        }
    }

    fn execute(&mut self, stmt: &crate::stmt::Stmt) {
        stmt.accept(self);
    }

    pub(crate) fn resolve(&mut self, expr: &expr::Expr, depth: usize) {
        self.locals.try_write().unwrap().insert(expr.clone(), depth);
    }

    /// Execute a block using a new empty environment with our original environment as enclosing
    fn execute_block(&mut self, statements: &[stmt::Stmt], environment: Environment) {
        let previous = self.environment.clone();
        self.environment = Arc::new(RwLock::new(environment));
        for statement in statements {
            self.execute(statement);
        }
        self.environment = previous;
    }

    fn lookup_variable(&mut self, name: &crate::scanner::Token, expr: &expr::Expr) -> Object {
        if let Some(distance) = self.locals.try_read().unwrap().get(expr) {
            self.environment
                .try_read()
                .unwrap()
                .get_at(distance, &name.lexeme)
        } else {
            self.globals.try_read().unwrap().get(name)
        }
    }
}
fn stringify(obj: Object) -> String {
    if obj.is_num() {
        let text = downcast!(obj => ObjectInner::Number).to_string();
        text.trim_end_matches(".0").to_string()
    } else {
        obj.to_string()
    }
}

#[derive(Debug, Exception)]
pub struct RuntimeError {
    token: Token,
    message: String,
}

impl RuntimeError {
    fn new(token: Token, message: impl ToString) -> Self {
        Self {
            token,
            message: message.to_string(),
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n[line {}]", self.message, self.token.line)
    }
}

#[derive(Debug, Exception)]
pub struct ReturnException(Object);
