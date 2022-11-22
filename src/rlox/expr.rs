use std::fmt::Display;

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
                    fn [<visit_ $name: snake>](&mut self, [<$name: snake>]: &$name) -> Result<T, E>;
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
    LiteralExpression { value: Literal, token: Token },
    LogicalExpression { left: Box<Expression>, op: Token, right: Box<Expression> },
    SetExpression { object: Box<Expression>, name: Token, value: Box<Expression> },
    SuperExpression { keyword: Token, method: Token },
    SelfExpression { keyword: Token },
    TernaryExpression { cmp: Box<Expression>, true_value: Box<Expression>, false_value: Box<Expression> },
    UnaryExpression { op: Token, right: Box<Expression> },
    VariableExpression { name: Token },
    LambdaExpression { params: Vec<Token>, body: Vec<Statement> },
    OperateAndAssignExpression { name: Token, op: Token, value: Box<Expression> }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::AssignExpression(a) => write!(f, "{} = {}", a.name.lexeme, a.value),
            Expression::BinaryExpression(b) => write!(f, "{} {} {}", b.left, b.op, b.right),
            Expression::CallExpression(c) => write!(
                f,
                "{}({})",
                c.callee,
                c.arguments
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Expression::GetExpression(g) => write!(f, "{}.{}", g.object, g.name),
            Expression::GroupingExpression(g) => write!(f, "({})", g.expression),
            Expression::LiteralExpression(l) => write!(f, "{}", l.value),
            Expression::LogicalExpression(l) => write!(f, "{} {} {}", l.left, l.op, l.right),
            Expression::SetExpression(s) => write!(f, "{}.{} = {}", s.object, s.name, s.value),
            Expression::SuperExpression(_) => todo!(),
            Expression::SelfExpression(t) => write!(f, "{}", t.keyword.lexeme),
            Expression::TernaryExpression(t) => {
                write!(f, "{} ? {} : {}", t.cmp, t.true_value, t.false_value)
            }
            Expression::UnaryExpression(u) => write!(f, "{}{}", u.op, u.right),
            Expression::VariableExpression(v) => write!(f, "{}", v.name),
            Expression::LambdaExpression(l) => write!(
                f,
                "func({}) {{ ... }}",
                l.params
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Expression::OperateAndAssignExpression(s) => {
                write!(f, "{} {} {}", s.name.lexeme, s.op.lexeme, s.value)
            }
        }
    }
}
