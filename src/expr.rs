use crate::scanner::Token;

#[macro_export]
macro_rules! ast {
    (-$name: ident- $($ast_expr: ident => $($field: ident $type: ty)+,)+) => {
        $(
        #[derive(Debug)]
        pub struct $ast_expr {
            $(pub $field: $type,)+
        })+
        #[derive(Debug)]
        pub enum $name {
        $(
             $ast_expr($ast_expr),
         )+
        }
    }
}
ast!(
-Expr-
Binary => left Box<Expr> operator Token  right Box<Expr>,
Assign => name Token value Box<Expr>,
Grouping => expression Box<Expr>,
Literal => value Option<String>,
Unary => operator Token right Box<Expr>,
Variable => name Token,
);

impl Expr {
    pub fn accept<R>(&self, visitor: &mut dyn Visit<R>) -> R {
        match self {
            Expr::Binary(binary) => visitor.visit_binary_expr(binary),
            Expr::Grouping(group) => visitor.visit_grouping_expr(group),
            Expr::Literal(lit) => visitor.visit_literal_expr(lit),
            Expr::Unary(un) => visitor.visit_unary_expr(un),
            Expr::Variable(var) => visitor.visit_variable_expr(var),
            Expr::Assign(expr) => visitor.visit_assign_expr(expr),
        }
    }
}

pub trait Visit<R> {
    fn visit_binary_expr(&mut self, expr: &Binary) -> R;
    fn visit_grouping_expr(&mut self, expr: &Grouping) -> R;
    fn visit_literal_expr(&mut self, expr: &Literal) -> R;
    fn visit_unary_expr(&mut self, expr: &Unary) -> R;
    fn visit_variable_expr(&mut self, expr: &Variable) -> R;
    fn visit_assign_expr(&mut self, expr: &Assign) -> R;
}

pub struct AstPrinter {}
impl Visit<String> for AstPrinter {
    fn visit_binary_expr(&mut self, expr: &Binary) -> String {
        self.parenthesize(&expr.operator.lexeme, [&expr.left, &expr.right])
    }
    fn visit_grouping_expr(&mut self, expr: &Grouping) -> String {
        self.parenthesize("group", [&expr.expression])
    }

    fn visit_literal_expr(&mut self, expr: &Literal) -> String {
        if let Some(ref value) = expr.value {
            value.clone()
        } else {
            "nil".to_string()
        }
    }

    fn visit_unary_expr(&mut self, expr: &Unary) -> String {
        self.parenthesize(&expr.operator.lexeme, [&expr.right])
    }

    fn visit_variable_expr(&mut self, expr: &Variable) -> String {
        expr.name.to_string()
    }

    fn visit_assign_expr(&mut self, expr: &Assign) -> String {
        format!(
            "name: {} value: {}",
            expr.name.to_string(),
            expr.value.accept(self)
        )
    }
}

#[allow(dead_code)]
impl AstPrinter {
    pub fn print(&mut self, expr: Expr) -> String {
        expr.accept(self)
    }

    fn parenthesize<'a>(
        &mut self,
        name: &str,
        exprs: impl IntoIterator<Item = &'a Box<Expr>>,
    ) -> String {
        let mut builder = String::new();

        builder.push('(');
        builder.push_str(name);
        for expr in exprs {
            builder.push(' ');
            builder.push_str(&expr.accept::<String>(self));
        }
        builder.push(')');
        builder
    }
}

#[test]
pub fn ast() {
    use crate::scanner::TokenType;

    let expr = Expr::Binary(Binary {
        left: Box::new(Expr::Unary(Unary {
            operator: Token::new(TokenType::MINUS, "-".into(), 1),
            right: Box::new(Expr::Literal(Literal {
                value: Some("123".into()),
            })),
        })),
        operator: Token::new(TokenType::STAR, "*".into(), 1),
        right: Box::new(Expr::Grouping(Grouping {
            expression: Box::new(Expr::Literal(Literal {
                value: Some("45".into()),
            })),
        })),
    });

    let code = AstPrinter {}.print(expr);
    println!("{}", code);
}
