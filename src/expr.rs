use crate::{interpreter::Object, scanner::Token};

crate::ast!(
-Expr-

Binary => visit_binary_expr =>  left Box<Expr> operator Token  right Box<Expr>,

Call => visit_call_expr =>  callee Box<Expr> paren Token  arguemnts Vec<Expr>,

Assign => visit_assign_expr => name Token value Box<Expr>,

Grouping => visit_grouping_expr => expression Box<Expr>,

Literal => visit_literal_expr => value Object,

Logical => visit_logical_expr => left Box<Expr> operator Token right Box<Expr>,

Unary => visit_unary_expr => operator Token right Box<Expr>,

Variable => visit_variable_expr => name Token,
);
