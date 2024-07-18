use paste;

macro_rules! ast_derive {
    ($name: ident, $($type: ident ($($prop: ident : $class: ty),+)),+) => {
        paste::paste! {
            #[derive(Clone)]
            pub enum $name {
                Empty,
                $(
                    $type(Box<$type>)
                ),+
            }

            $(
                #[derive(Clone)]
                pub struct $type {
                    $(
                        pub $prop: $class
                    ),+
                }
            )+

            pub trait [<$name Visitor>]<R> {
                $(
                        fn [<visit_ $type:snake>] (&mut self, target: &$type) -> R;

                )+
            }

            pub trait [<$name Accept>]<R> {
                fn accept<V: [<$name Visitor>]<R>>(&self, visitor: &mut V) -> R;
            }

            impl<R> [<$name Accept>]<R> for $name {
                fn accept<V: [<$name Visitor>]<R>>(&self, visitor: &mut V) -> R {
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
                    fn accept<V: [<$name Visitor>]<R>>(&self, visitor: &mut V) -> R {
                        visitor.[<visit_ $type:snake>](self)
                    }
                }
            )+
        }
    }
}

pub(crate) use ast_derive;
