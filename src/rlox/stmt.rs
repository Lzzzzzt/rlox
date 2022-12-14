use super::{expr::Expression, token::Token, types::FuncType};
use paste::paste;
macro_rules! stmt {
    ($($name: ident { $($attr: ident: $attr_type: ty), * }),* $(,)?) => {
        paste! {
            $(
                #[derive(Debug, Clone)]
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

            #[derive(Debug, Clone)]
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

            #[allow(unused)]
            pub trait Visitor<T, E> {
                $(
                    fn [<visit_ $name: snake>](&mut self, [<$name: snake>]: &$name) -> Result<T, E> {
                        todo!()
                    }
                )*
            }
        }
    };
}

stmt! {
    ExpressionStatement { expression: Expression, end: Token },
    PrintStatement { expression: Expression, keyword: Token },
    VarStatement { name: Token, initializer: Option<Expression> },
    MultiVarStatement { vars: Vec<Statement> },
    BlockStatement { statements: Vec<Statement> },
    BranchStatement { condition: Expression, then_branch: Box<Statement>, else_branch: Option<Box<Statement>> },
    WhileStatement { condition: Expression, body: Box<Statement>, increment: Option<Box<Statement>> },
    ContinueStatement { token: Token },
    BreakStatement { token: Token },
    FunctionStatement { name: Token, params: Vec<Token>, body: Vec<Statement>, function_type: FuncType },
    ReturnStatement { key_word: Token, value: Option<Expression> },
    ClassStatement { name: Token, methods: Vec<Statement>, static_methods: Vec<Statement> }
}
