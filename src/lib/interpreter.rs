use super::{
    error::LoxError,
    expr::{Expression, Visitor},
    token::{Literal, Token},
    token_type::TokenType,
};

pub struct Interpreter;

impl Interpreter {
    pub fn interpreter(&self, expr: Expression) -> Result<Literal, LoxError> {
        Ok(self.evaluate(&expr)?)
    }

    fn evaluate(&self, expr: &Expression) -> Result<Literal, LoxError> {
        expr.accept(self)
    }

    fn get_num(&self, obj: &Literal, op: &Token) -> Result<f64, LoxError> {
        if let Literal::Number(num) = obj {
            return Ok(*num);
        }

        Err(LoxError::create_runtime_error(
            op,
            "Operand must be a number.".into(),
        ))
    }

    fn get_string(&self, obj: &Literal, op: &Token) -> Result<String, LoxError> {
        if let Literal::String(string) = obj {
            return Ok(string.clone());
        }

        Err(LoxError::create_runtime_error(
            op,
            "Operand must be a string.".into(),
        ))
    }

    fn get_bool(&self, obj: &Literal) -> Result<bool, LoxError> {
        if let Literal::Bool(b) = obj {
            return Ok(*b);
        }

        Err(LoxError::RuntimeError { line: 0, lexeme: obj.to_string(), msg: "Expected type is `bool`".into() })
    }

    fn is_true(&self, obj: Literal) -> bool {
        match obj {
            Literal::String(_) | Literal::Number(_) => true,
            Literal::Bool(b) => b,
            Literal::Nil => false,
        }
    }
}

#[allow(unused)]
impl Visitor<Literal, LoxError> for Interpreter {
    fn visit_binary_expression(
        &self,
        binary_expression: &super::expr::BinaryExpression,
    ) -> Result<Literal, LoxError> {
        let left = self.evaluate(&binary_expression.left)?;
        let right = self.evaluate(&binary_expression.right)?;
        let op = &binary_expression.op;

        match op.token_type {
            TokenType::Minus => {
                let left = self.get_num(&left, op)?;
                let right = self.get_num(&right, op)?;
                Ok(Literal::Number(left - right))
            }
            TokenType::Slash => {
                let left = self.get_num(&left, op)?;
                let right = self.get_num(&right, op)?;
                Ok(Literal::Number(left / right))
            }
            TokenType::Star => {
                let left = self.get_num(&left, op)?;
                let right = self.get_num(&right, op)?;
                Ok(Literal::Number(left * right))
            }
            TokenType::Plus => match left {
                Literal::String(left) => {
                    let right = self.get_string(&right, op)?;

                    let str = String::from_iter([left, right]);
                    Ok(Literal::String(str))
                }
                Literal::Number(left) => {
                    let right = self.get_num(&right, op)?;
                    Ok(Literal::Number(left + right))
                }
                _ => Err(LoxError::create_runtime_error(
                    &binary_expression.op,
                    "Operands must be two numbers or two strings.".into(),
                )),
            },
            TokenType::Greater => {
                let left = self.get_num(&left, op)?;
                let right = self.get_num(&right, op)?;
                Ok(Literal::Bool(left > right))
            }
            TokenType::GreaterEqual => {
                let left = self.get_num(&left, op)?;
                let right = self.get_num(&right, op)?;
                Ok(Literal::Bool(left >= right))
            }
            TokenType::Less => {
                let left = self.get_num(&left, op)?;
                let right = self.get_num(&right, op)?;
                Ok(Literal::Bool(left < right))
            }
            TokenType::LessEqual => {
                let left = self.get_num(&left, op)?;
                let right = self.get_num(&right, op)?;
                Ok(Literal::Bool(left <= right))
            }
            TokenType::EqualEqual => Ok(Literal::Bool(left == right)),
            TokenType::BangEqual => Ok(Literal::Bool(left != right)),
            TokenType::Comma => Ok(binary_expression.right.accept(self)?),
            _ => Err(LoxError::create_runtime_error(
                &binary_expression.op,
                "Unexpected operator".into(),
            )),
        }
    }

    fn visit_grouping_expression(
        &self,
        grouping_expression: &super::expr::GroupingExpression,
    ) -> Result<Literal, LoxError> {
        Ok(self.evaluate(&grouping_expression.expression)?)
    }

    fn visit_literal_expression(
        &self,
        literal_expression: &super::expr::LiteralExpression,
    ) -> Result<Literal, LoxError> {
        Ok(literal_expression.value.clone())
    }

    fn visit_unary_expression(
        &self,
        unary_expression: &super::expr::UnaryExpression,
    ) -> Result<Literal, LoxError> {
        let right = self.evaluate(&unary_expression.right)?;
        let op = &unary_expression.op;

        match op.token_type {
            TokenType::Minus => Ok(Literal::Number(-self.get_num(&right, op)?)),
            TokenType::Bang => Ok(Literal::Bool(!self.is_true(right))),
            _ => Err(LoxError::create_runtime_error(
                &unary_expression.op,
                "Operand must be two number.".into(),
            )),
        }
    }

    fn visit_assign_expression(
        &self,
        assign_expression: &super::expr::AssignExpression,
    ) -> Result<Literal, LoxError> {
        todo!()
    }

    fn visit_call_expression(
        &self,
        call_expression: &super::expr::CallExpression,
    ) -> Result<Literal, LoxError> {
        todo!()
    }

    fn visit_get_expression(
        &self,
        get_expression: &super::expr::GetExpression,
    ) -> Result<Literal, LoxError> {
        todo!()
    }

    fn visit_logical_expression(
        &self,
        logical_expression: &super::expr::LogicalExpression,
    ) -> Result<Literal, LoxError> {
        todo!()
    }

    fn visit_set_expression(
        &self,
        set_expression: &super::expr::SetExpression,
    ) -> Result<Literal, LoxError> {
        todo!()
    }

    fn visit_super_expression(
        &self,
        super_expression: &super::expr::SuperExpression,
    ) -> Result<Literal, LoxError> {
        todo!()
    }

    fn visit_this_expression(
        &self,
        this_expression: &super::expr::ThisExpression,
    ) -> Result<Literal, LoxError> {
        todo!()
    }

    fn visit_ternary_expression(
        &self,
        ternary_expression: &super::expr::TernaryExpression,
    ) -> Result<Literal, LoxError> {
        let cmp = self.get_bool(&self.evaluate(&ternary_expression.cmp)?)?;

        if cmp {
            Ok(self.evaluate(&ternary_expression.true_value)?)
        } else {
            Ok(self.evaluate(&ternary_expression.false_value)?)
        }
    }

    fn visit_variable_expression(
        &self,
        variable_expression: &super::expr::VariableExpression,
    ) -> Result<Literal, LoxError> {
        todo!()
    }
}
