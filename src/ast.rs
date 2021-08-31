#[macro_export]
macro_rules! ast {
    (-$name: ident- $($ast_expr: ident => $visit: ident => $($field: ident $type: ty)+,)+) => {
        $(
        #[derive(Debug, Clone, PartialEq, Eq,Hash)]
        pub struct $ast_expr {
            $(pub $field: $type,)+
        })+
        #[derive(Debug, Clone, PartialEq, Eq,Hash)]
        pub enum $name {
        $(
             $ast_expr($ast_expr),
         )+
        }
        $(
        impl From<$ast_expr> for $name {
            fn from(expr: $ast_expr) -> Self {
                Self::$ast_expr(expr)
            }
        }
        )+

        impl $name {
        pub fn accept<R>(&self, visitor: &mut dyn Visit<R>) -> R {
            match self {
                $(
                $name::$ast_expr(inner) => visitor.$visit(inner),
                )+
            }
        }
        }

        pub trait Visit<R> {
            $(
                fn $visit(&mut self, expr: &$ast_expr) -> R;
             )+
        }
    }
}
