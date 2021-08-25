use crate::ast;
use crate::expr::Expr;
use crate::scanner::Token;

ast!(
-Stmt-

Block => visit_block_stmt => statements Vec<Stmt>,

Expression => visit_expression_stmt => expression Expr,

Function => visit_function_stmt => name Token params Vec<Token> body Vec<Stmt>,

If => visit_if_stmt => condition Expr then_branch Box<Stmt> else_branch Option<Box<Stmt>>,

Print => visit_print_stmt => expression Expr,

Var => visit_var_stmt => name Token initializer Option<Expr>,

Return => visit_return_stmt => keyword Token value Option<Expr>,

While => visit_while_stmt => condition Expr body Box<Stmt>,
);
