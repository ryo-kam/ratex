macro_rules! ast_derive {
    ($name: ident, $($type: ident ($($prop: ident : $class: ty),*)),+) => {
        paste::paste! {
            #[derive(Clone, Debug, PartialEq, Eq, Hash)]
            pub enum $name {
                Empty,
                $(
                    $type(Rc<$type>)
                ),+
            }

            $(
                #[derive(Clone, Debug, PartialEq, Eq, Hash)]
                pub struct $type {
                    $(
                        pub $prop: $class
                    ),*
                }

                impl $type {
                    pub fn new(
                        $(
                            $prop: $class
                        ),*
                    ) -> Rc<$name> {
                        return Rc::new(
                            $name::$type(
                                Rc::new(
                                    $type {$($prop: $prop),*}
                                )
                            )
                        )
                    }
                }
            )+

            pub trait [<$name Visitor>]<R> {
                $(
                        fn [<visit_ $type:snake>] (&mut self, target: Rc<$type>) -> Result<R, RatexError>;

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
                            $name::$type(x) => visitor.[<visit_ $type:snake>](Rc::clone(x))
                        ),+
                    }
                }
            }

            $(
                impl<R> [<$name Accept>]<R> for $type {
                    fn accept<V: [<$name Visitor>]<R>>(&self, visitor: &mut V) -> Result<R, RatexError> {
                        visitor.[<visit_ $type:snake>](Rc::new(self.clone()))
                    }
                }
            )+
        }
    }
}

pub(crate) use ast_derive;
