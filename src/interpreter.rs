use std::{cell::Cell, str::FromStr};

use crate::{expr::Visit, scanner::TokenType, LError};

macro_rules! obj {
    ($e: expr) => {
        (Object($e.to_string()))
    };
}

pub struct Interpreter {
    had_error: Cell<bool>,
}
impl Visit<Object> for Interpreter {
    fn visit_binary_expr(&self, expr: &crate::expr::Binary) -> Object {
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

    fn visit_grouping_expr(&self, expr: &crate::expr::Grouping) -> Object {
        self.evaluate(&expr.expression)
    }

    fn visit_literal_expr(&self, expr: &crate::expr::Literal) -> Object {
        obj!(expr.value.as_ref().unwrap())
    }

    fn visit_unary_expr(&self, expr: &crate::expr::Unary) -> Object {
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
        }
    }

    fn evaluate(&self, expression: &crate::expr::Expr) -> Object {
        expression.accept(self)
    }
    pub fn interpret(&mut self, expression: &crate::expr::Expr) {
        let value = self.evaluate(expression);
        println!("{}", stringify(value));
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

pub struct Object(String);
impl Object {
    fn downcast<T: FromStr>(&self) -> T {
        self.try_downcast().unwrap()
    }
    fn try_downcast<T: FromStr>(&self) -> Option<T> {
        self.0.parse().ok()
    }
    fn is<T: FromStr>(&self) -> bool {
        self.try_downcast::<T>().is_some()
    }
}

impl LError for Interpreter {
    fn had_error(&self) -> &std::cell::Cell<bool> {
        &self.had_error
    }
}
