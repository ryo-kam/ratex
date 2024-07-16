use paste;

macro_rules! ast_derive {
    ($($type: ident ($($prop: ident : $class: ty),+)),+) => {
        paste::paste! {
            pub enum Expr {
                Empty,
                $(
                    $type(Box<$type>)
                ),+
            }

            pub enum LiteralValue {
                String(String),
                Number(f64),
                Nil,
            }

            $(
                pub struct $type {
                    $(
                        pub $prop: $class
                    ),+
                }
            )+

            pub trait AstVisitor<R> {
                $(
                        fn [<visit_ $type:snake>] (&mut self, target: &$type) -> R;

                )+
            }

            pub trait Accept<R> {
                fn accept<V: AstVisitor<R>>(&self, visitor: &mut V) -> R;
            }

            impl<R> Accept<R> for Expr {
                fn accept<V: AstVisitor<R>>(&self, visitor: &mut V) -> R {
                    match self {
                        Expr::Empty => {
                            panic!("Cannot visit empty");
                        }

                        $(
                            Expr::$type(x) => visitor.[<visit_ $type:snake>](x)
                        ),+
                    }
                }
            }

            $(
                impl<R> Accept<R> for $type {
                    fn accept<V: AstVisitor<R>>(&self, visitor: &mut V) -> R {
                        visitor.[<visit_ $type:snake>](self)
                    }
                }
            )+
        }
    }
}

pub(crate) use ast_derive;
