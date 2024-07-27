macro_rules! ast_derive {
    ($name: ident, $($type: ident ($($prop: ident : $class: ty),*)),+) => {
        paste::paste! {
            #[derive(Clone, Debug, PartialEq, Eq, Hash)]
            pub enum $name {
                Empty,
                $(
                    $type($type)
                ),+
            }

            $(
                #[derive(Clone, Debug, PartialEq, Eq, Hash)]
                pub struct $type {
                    $(
                        pub $prop: $class
                    ),*
                }
            )+

            pub trait [<$name Visitor>]<R> {
                $(
                        fn [<visit_ $type:snake>] (&mut self, target: &$type) -> Result<R, RatexError>;

                )+
            }

            pub trait [<$name Accept>]<R> {
                fn accept<V: [<$name Visitor>]<R>>(&self, visitor: &mut V) -> Result<R, RatexError>;
            }

            impl<R> [<$name Accept>]<R> for $name {
                fn accept<V: [<$name Visitor>]<R>>(&self, visitor: &mut V) -> Result<R, RatexError> {
                    match self {
                        $name::Empty => {
                            panic!("Cannot visit empty");
                        }

                        $(
                            $name::$type(x) => visitor.[<visit_ $type:snake>](x)
                        ),+
                    }
                }
            }

            $(
                impl<R> [<$name Accept>]<R> for $type {
                    fn accept<V: [<$name Visitor>]<R>>(&self, visitor: &mut V) -> Result<R, RatexError> {
                        visitor.[<visit_ $type:snake>](self)
                    }
                }
            )+
        }
    }
}

pub(crate) use ast_derive;
