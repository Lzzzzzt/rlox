use super::{stmt::Statement, token::Token, types::Literal};
use paste::paste;
macro_rules! expr {
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
            pub enum Expression {
                $($name($name)), *
            }

            #[allow(dead_code)]
            impl Expression {
                $(
                    pub fn [<create_ $name: snake>]($($attr: $attr_type), *) -> Expression {
                        Expression::$name ($name {$($attr), *})
                    }
                ) *

                pub fn accept<T, E>(&self, visitor: &mut impl Visitor<T, E>) -> Result<T, E> {
                    match self {
                        $(
                            Self::$name(expr) => visitor.[<visit_ $name: snake>](expr),
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

expr! {
    AssignExpression { name: Token, value: Box<Expression> },
    BinaryExpression { left: Box<Expression>, op: Token, right: Box<Expression> },
    CallExpression { callee: Box<Expression>, paren: Token, arguments: Vec<Expression> },
    GetExpression { object: Box<Expression>, name: Token },
    GroupingExpression { expression: Box<Expression> },
    LiteralExpression { value: Literal },
    LogicalExpression { left: Box<Expression>, op: Token, right: Box<Expression> },
    SetExpression { object: Box<Expression>, name: Token, value: Box<Expression> },
    SuperExpression { keyword: Token, method: Token },
    ThisExpression { keyword: Token },
    TernaryExpression { cmp: Box<Expression>, true_value: Box<Expression>, false_value: Box<Expression> },
    UnaryExpression { op: Token, right: Box<Expression> },
    VariableExpression { name: Token },
    LambdaExpression { params: Vec<Token>, body: Vec<Statement> },
}
