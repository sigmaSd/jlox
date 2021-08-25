#[macro_export]
macro_rules! ast {
    (-$name: ident- $($ast_expr: ident => $visit: ident => $($field: ident $type: ty)+,)+) => {
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
