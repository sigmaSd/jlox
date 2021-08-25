use crate::scanner::Token;

crate::ast!(
-Expr-
Binary => visit_binary_expr =>  left Box<Expr> operator Token  right Box<Expr>,
Assign => visit_assign_expr => name Token value Box<Expr>,
Grouping => visit_grouping_expr => expression Box<Expr>,
Literal => visit_literal_expr => value Option<String>,
Unary => visit_unary_expr => operator Token right Box<Expr>,
Variable => visit_variable_expr => name Token,
);
