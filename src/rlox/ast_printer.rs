use super::{
    error::LoxError,
    expr::{Expression, Visitor},
};

pub struct AstPrinter;

#[allow(unused)]
impl AstPrinter {
    pub fn new() -> Self {
        Self
    }

    pub fn print(&mut self, expr: &Expression) -> String {
        expr.accept(self).unwrap()
    }

    fn parenthesize(&mut self, name: &str, exprs: Vec<&Expression>) -> Result<String, LoxError> {
        let mut string = String::new();

        string.push('(');
        string.push_str(name);

        for expr in exprs {
            string.push(' ');
            string.push_str(expr.accept(self)?.as_str())
        }

        string.push(')');

        Ok(string)
    }
}

#[allow(unused)]
impl Visitor<String, LoxError> for AstPrinter {
    fn visit_assign_expression(
        &mut self,
        assign_expression: &super::expr::AssignExpression,
    ) -> Result<String, LoxError> {
        todo!()
    }

    fn visit_binary_expression(
        &mut self,
        binary_expression: &super::expr::BinaryExpression,
    ) -> Result<String, LoxError> {
        return self.parenthesize(
            binary_expression.op.lexeme.as_str(),
            vec![&binary_expression.left, &binary_expression.right],
        );
    }

    fn visit_call_expression(
        &mut self,
        call_expression: &super::expr::CallExpression,
    ) -> Result<String, LoxError> {
        todo!()
    }

    fn visit_get_expression(
        &mut self,
        get_expression: &super::expr::GetExpression,
    ) -> Result<String, LoxError> {
        todo!()
    }

    fn visit_grouping_expression(
        &mut self,
        grouping_expression: &super::expr::GroupingExpression,
    ) -> Result<String, LoxError> {
        self.parenthesize("group", vec![&grouping_expression.expression])
    }

    fn visit_literal_expression(
        &mut self,
        literal_expression: &super::expr::LiteralExpression,
    ) -> Result<String, LoxError> {
        match &literal_expression.value {
            super::token::Literal::String(string) => Ok(string.into()),
            super::token::Literal::Number(number) => Ok(format!("{}", number)),
            super::token::Literal::Bool(b) => Ok(b.to_string()),
            super::token::Literal::Nil => Ok("nil".into()),
        }
    }

    fn visit_logical_expression(
        &mut self,
        logical_expression: &super::expr::LogicalExpression,
    ) -> Result<String, LoxError> {
        todo!()
    }

    fn visit_set_expression(
        &mut self,
        set_expression: &super::expr::SetExpression,
    ) -> Result<String, LoxError> {
        todo!()
    }

    fn visit_super_expression(
        &mut self,
        super_expression: &super::expr::SuperExpression,
    ) -> Result<String, LoxError> {
        todo!()
    }

    fn visit_this_expression(
        &mut self,
        this_expression: &super::expr::ThisExpression,
    ) -> Result<String, LoxError> {
        todo!()
    }

    fn visit_ternary_expression(
        &mut self,
        ternary_expression: &super::expr::TernaryExpression,
    ) -> Result<String, LoxError> {
        self.parenthesize(
            "ternary",
            vec![
                &ternary_expression.cmp,
                &ternary_expression.true_value,
                &ternary_expression.false_value,
            ],
        )
    }

    fn visit_unary_expression(
        &mut self,
        unary_expression: &super::expr::UnaryExpression,
    ) -> Result<String, LoxError> {
        self.parenthesize(
            unary_expression.op.lexeme.as_str(),
            vec![&unary_expression.right],
        )
    }

    fn visit_variable_expression(
        &mut self,
        variable_expression: &super::expr::VariableExpression,
    ) -> Result<String, LoxError> {
        todo!()
    }
}
