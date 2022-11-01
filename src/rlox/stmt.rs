use super::{expr::Expression, token::Token};
use paste::paste;
macro_rules! stmt {
    ($($name: ident { $($attr: ident: $attr_type: ty), * }),* $(,)?) => {
        paste! {
            $(
                #[derive(Debug)]
                #[allow(dead_code)]
                pub struct $name {
                    $(pub $attr: $attr_type), *
                }

                #[allow(dead_code)]
                impl $name {
                    pub fn new($($attr: $attr_type), *) -> Self {
                        Self {
                            $($attr), *
                        }
                    }
                }
            ) *

            #[derive(Debug)]
            #[allow(dead_code)]
            #[allow(clippy::enum_variant_names)]
            pub enum Statement {
                $($name($name)), *
            }

            #[allow(dead_code)]
            impl Statement {
                $(
                    pub fn [<create_ $name: snake>]($($attr: $attr_type), *) -> Self {
                        Self::$name ($name {$($attr), *})
                    }
                ) *

                pub fn accept<T, E>(&self, visitor: &mut impl Visitor<T, E>) -> Result<T, E> {
                    match self {
                        $(
                            Self::$name(stmt) => visitor.[<visit_ $name: snake>](stmt),
                        ) *
                    }
                }
            }

            pub trait Visitor<T, E> {
                $(
                    fn [<visit_ $name: snake>](&mut self, [<$name: snake>]: &$name) -> Result<T, E>;
                )*
            }
        }
    };
}

stmt! {
    ExpressionStatement { expression: Expression },
    PrintStatement { expression: Expression },
    VarStatement { name: Token, initializer: Option<Expression> },
    MultiVarStatement { vars: Vec<VarStatement> },
    BlockStatement { statements: Vec<Statement> },
}
