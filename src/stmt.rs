use crate::ast;
use crate::expr::Expr;
use crate::scanner::Token;

ast!(
-Stmt-
Block => visit_block_stmt => statements Vec<Stmt>,
Expression => visit_expression_stmt => expression Box<Expr>,
Print => visit_print_stmt => expression Box<Expr>,
Var => visit_var_stmt => name Token initializer Option<Box<Expr>>,
);
