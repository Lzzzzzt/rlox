use crate::lib::token::{Literal, Token};

// macro_rules! ast {
//     ($($name: ident { $($attr: ident: $attr_type: ty), * }), * $(,)?) => {
//         pub enum Expression {
//             $($name { $($attr: $attr_type), * }), *
//         }
//
//         impl Expression {
//             $(
//                 pub fn $name($($attr: $attr_type), *) -> Expression {
//                     Expression::$name { $($attr), * }
//                 }
//
//             )*
//         }
//     };
// }
//
// ast! {
//     assign { name: Token, value: Box<Expression> },
//     binary { left: Box<Expression>, op: Token, right: Box<Expression> },
//     call { callee: Box<Expression>, paren: Token, argument: Vec<Expression> },
//     get { object: Box<Expression>, name: Token },
//     grouping { expression: Box<Expression> },
//     literal { value: literal },
//     logical { left: Box<Expression>, op: Token, right: Box<Expression> },
//     set { object: Box<Expression>, name: Token, value: Box<Expression> },
//     Super { keyword: Token, method: Token },
//     this { keyword: Token },
//     unary { op: Token, right: Token },
//     variable { name: Token },
// }

#[derive(Debug)]
pub enum Expression {
    Assign { name: Token, value: Box<Expression> },
    Binary { left: Box<Expression>, op: Token, right: Box<Expression> },
    Call { callee: Box<Expression>, paren: Token, argument: Vec<Expression> },
    Get { object: Box<Expression>, name: Token },
    Grouping { expression: Box<Expression> },
    Literal { value: Literal },
    Logical { left: Box<Expression>, op: Token, right: Box<Expression> },
    Set { object: Box<Expression>, name: Token, value: Box<Expression> },
    Super { keyword: Token, method: Token },
    Ternary { cmp: Box<Expression>, true_value: Box<Expression>, false_value: Box<Expression> },
    This { keyword: Token },
    Unary { op: Token, right: Box<Expression> },
    Variable { name: Token },
}

#[allow(unused)]
impl Expression {
    pub fn assign(name: Token, value: Box<Expression>) -> Expression {
        Expression::Assign { name, value }
    }
    pub fn binary(left: Box<Expression>, op: Token, right: Box<Expression>) -> Expression {
        Expression::Binary { left, op, right }
    }
    pub fn call(callee: Box<Expression>, paren: Token, argument: Vec<Expression>) -> Expression {
        Expression::Call { callee, paren, argument }
    }
    pub fn get(object: Box<Expression>, name: Token) -> Expression {
        Expression::Get { object, name }
    }
    pub fn grouping(expression: Box<Expression>) -> Expression {
        Expression::Grouping { expression }
    }
    pub fn literal(value: Literal) -> Expression {
        Expression::Literal { value }
    }
    pub fn logical(left: Box<Expression>, op: Token, right: Box<Expression>) -> Expression {
        Expression::Logical { left, op, right }
    }
    pub fn set(object: Box<Expression>, name: Token, value: Box<Expression>) -> Expression {
        Expression::Set { object, name, value }
    }
    pub fn create_super(keyword: Token, method: Token) -> Expression {
        Expression::Super { keyword, method }
    }
    pub fn ternary(cmp: Box<Expression>, true_value: Box<Expression>, false_value: Box<Expression>) -> Expression {
        Expression::Ternary { cmp, true_value, false_value }
    }
    pub fn this(keyword: Token) -> Expression {
        Expression::This { keyword }
    }

    pub fn unary(op: Token, right: Box<Expression>) -> Expression {
        Expression::Unary { op, right }
    }
    pub fn variable(name: Token) -> Expression {
        Expression::Variable { name }
    }
}