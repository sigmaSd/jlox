use crate::ast;
use crate::expr::Expr;
use crate::scanner::Token;

ast!(
-Stmt-
Block => statements Vec<Stmt>,
Expression => expression Box<Expr>,
Print => expression Box<Expr>,
Var => name Token initializer Option<Box<Expr>>,
);

impl Stmt {
    pub fn accept<R>(&self, visitor: &mut dyn Visit<R>) -> R {
        match self {
            Stmt::Expression(stmt) => visitor.visit_expression_stmt(stmt),
            Stmt::Print(stmt) => visitor.visit_print_stmt(stmt),
            Stmt::Var(stmt) => visitor.visit_var_stmt(stmt),
            Stmt::Block(stmt) => visitor.visit_block_stmt(stmt),
        }
    }
}
pub trait Visit<R> {
    fn visit_block_stmt(&mut self, stmt: &Block) -> R;
    fn visit_expression_stmt(&mut self, stmt: &Expression) -> R;
    fn visit_print_stmt(&mut self, stmt: &Print) -> R;
    fn visit_var_stmt(&mut self, stmt: &Var) -> R;
}
