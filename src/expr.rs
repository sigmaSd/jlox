use crate::scanner::Token;

macro_rules! ast {
    ($($ast_expr: ident => $($field: ident $type: ty)+,)+) => {
                $(
        pub struct $ast_expr {
            $($field: $type,)+
        })+
            pub enum Expr {
                $(
     $ast_expr($ast_expr),
     )+
            }
    }
}
ast!(
Binary => left Box<Expr> operator Token  right Box<Expr>,
Grouping => expression Box<Expr>,
Literal => value Option<String>,
Unary => operator Token right Box<Expr>,
);

impl Expr {
    fn accept<R>(&self, visitor: &dyn Visit<R>) -> R {
        match self {
            Expr::Binary(binary) => visitor.visit_binary_expr(binary),
            Expr::Grouping(group) => visitor.visit_grouping_expr(group),
            Expr::Literal(lit) => visitor.visit_literal_expr(lit),
            Expr::Unary(un) => visitor.visit_unary_expr(un),
        }
    }
}

trait Visit<R> {
    fn visit_binary_expr(&self, expr: &Binary) -> R;
    fn visit_grouping_expr(&self, expr: &Grouping) -> R;
    fn visit_literal_expr(&self, expr: &Literal) -> R;
    fn visit_unary_expr(&self, expr: &Unary) -> R;
}

struct AstPrinter {}
impl Visit<String> for AstPrinter {
    fn visit_binary_expr(&self, expr: &Binary) -> String {
        self.parenthesize(&expr.operator.lexeme, [&expr.left, &expr.right])
    }
    fn visit_grouping_expr(&self, expr: &Grouping) -> String {
        self.parenthesize("group", [&expr.expression])
    }

    fn visit_literal_expr(&self, expr: &Literal) -> String {
        if let Some(ref value) = expr.value {
            value.clone()
        } else {
            "nil".to_string()
        }
    }

    fn visit_unary_expr(&self, expr: &Unary) -> String {
        self.parenthesize(&expr.operator.lexeme, [&expr.right])
    }
}

impl AstPrinter {
    fn print(&self, expr: Expr) -> String {
        expr.accept(self)
    }

    fn parenthesize<'a>(
        &self,
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
