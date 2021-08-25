use crate::{expr, obj, scanner::TokenType, stmt, LError};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

mod environment;
use environment::Environment;
mod object;
use object::Object;

pub struct Interpreter {
    had_error: Cell<bool>,
    environment: Rc<RefCell<Environment>>,
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
        let mut value = None;
        if let Some(ref initializer) = stmt.initializer {
            value = Some(self.evaluate(initializer));
        }
        self.environment
            .borrow_mut()
            .define(stmt.name.lexeme.clone(), value);
    }

    fn visit_block_stmt(&mut self, stmt: &stmt::Block) {
        self.execute_block(
            &stmt.statements,
            Environment::new(Some(self.environment.clone())),
        );
    }
}
impl expr::Visit<Object> for Interpreter {
    fn visit_binary_expr(&mut self, expr: &crate::expr::Binary) -> Object {
        let right = self.evaluate(&expr.right);
        let left = self.evaluate(&expr.left);

        match expr.operator.ttype {
            TokenType::MINUS => {
                check_number_operands(&expr.operator, [&left, &right]);
                return obj!(left.downcast::<f64>() - right.downcast::<f64>());
            }
            TokenType::PLUS => {
                if left.is::<f64>() && right.is::<f64>() {
                    return obj!(left.downcast::<f64>() + right.downcast::<f64>());
                }
                if left.is::<String>() && right.is::<String>() {
                    return obj!(left.downcast::<String>() + &right.downcast::<String>());
                }
                panic!("+ only supports numbers and strings")
            }
            TokenType::SLASH => {
                check_number_operands(&expr.operator, [&left, &right]);
                return obj!(left.downcast::<f64>() / right.downcast::<f64>());
            }
            TokenType::STAR => {
                check_number_operands(&expr.operator, [&left, &right]);
                return obj!(left.downcast::<f64>() * right.downcast::<f64>());
            }
            TokenType::GREATER => {
                check_number_operands(&expr.operator, [&right]);
                return obj!(left.downcast::<f64>() > right.downcast::<f64>());
            }
            TokenType::GREATER_EQUAL => {
                check_number_operands(&expr.operator, [&left, &right]);
                return obj!(left.downcast::<f64>() >= right.downcast::<f64>());
            }
            TokenType::LESS => {
                check_number_operands(&expr.operator, [&left, &right]);
                return obj!(left.downcast::<f64>() < right.downcast::<f64>());
            }
            TokenType::LESS_EQUAL => {
                return obj!(left.downcast::<f64>() <= right.downcast::<f64>())
            }
            TokenType::BANG_EQUAL => return obj!(!is_equal(left, right)),
            TokenType::EQUAL_EQUAL => return obj!(is_equal(left, right)),
            _ => unreachable!(),
        }
    }

    fn visit_grouping_expr(&mut self, expr: &crate::expr::Grouping) -> Object {
        self.evaluate(&expr.expression)
    }

    fn visit_literal_expr(&mut self, expr: &crate::expr::Literal) -> Object {
        obj!(expr.value.as_ref().unwrap())
    }

    fn visit_unary_expr(&mut self, expr: &crate::expr::Unary) -> Object {
        let right = self.evaluate(&expr.right);
        match expr.operator.ttype {
            TokenType::MINUS => {
                check_number_operands(&expr.operator, [&right]);
                return obj!(-right.downcast::<f64>());
            }
            TokenType::BANG => return obj!(!is_truthy(right)),

            _ => unreachable!(),
        }
    }

    fn visit_variable_expr(&mut self, expr: &expr::Variable) -> Object {
        self.environment.borrow().get(&expr.name)
    }

    fn visit_assign_expr(&mut self, expr: &expr::Assign) -> Object {
        let value = self.evaluate(&expr.value);
        self.environment
            .borrow_mut()
            .assign(expr.name.clone(), value.clone());
        value
    }
}

fn check_number_operands<'a>(
    operator: &crate::scanner::Token,
    operators: impl IntoIterator<Item = &'a Object>,
) {
    if operators.into_iter().all(Object::is::<f64>) {
        return;
    }
    panic!("{} Operand must be a number.", operator);
}
fn is_equal(right: Object, left: Object) -> bool {
    right.0 == left.0
}

fn is_truthy(right: Object) -> bool {
    //TODO if right == null
    right.try_downcast::<bool>().unwrap_or(true)
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            had_error: Cell::new(false),
            environment: Rc::new(RefCell::new(Environment::new(None))),
        }
    }

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

    /// Execute a block using a new empty environment with our original environment as enclosing
    fn execute_block(&mut self, statements: &[stmt::Stmt], environment: Environment) {
        let previous = self.environment.clone();
        self.environment = Rc::new(RefCell::new(environment));
        for statement in statements {
            self.execute(statement);
        }
        self.environment = previous;
    }
}
fn stringify(obj: Object) -> String {
    if obj.is::<f64>() {
        let text = obj.downcast::<f64>().to_string();
        text.trim_end_matches(".0").to_string()
    } else {
        obj.downcast::<String>()
    }
}

impl LError for Interpreter {
    fn had_error(&self) -> &std::cell::Cell<bool> {
        &self.had_error
    }
}
