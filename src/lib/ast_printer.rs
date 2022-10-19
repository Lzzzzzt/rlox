use super::expr::{Expression, Visitor};

pub struct AstPrinter;

impl AstPrinter {
    pub fn new() -> Self {
        Self
    }

    pub fn print(&self, expr: &Expression) -> String {
        expr.accept(self)
    }

    fn parenthesize(&self, name: &str, exprs: Vec<&Expression>) -> String {
        let mut string = String::new();

        string.push('(');
        string.push_str(name);

        for expr in exprs {
            string.push(' ');
            string.push_str(expr.accept(self).as_str())
        }

        string.push(')');

        string
    }
}

#[allow(unused)]
impl Visitor<String> for AstPrinter {
    fn visit_assign_expression(&self, assign_expression: &super::expr::AssignExpression) -> String {
        todo!()
    }

    fn visit_binary_expression(&self, binary_expression: &super::expr::BinaryExpression) -> String {
        return self.parenthesize(
            binary_expression.op.lexeme.as_str(),
            vec![&binary_expression.left, &binary_expression.right],
        );
    }

    fn visit_call_expression(&self, call_expression: &super::expr::CallExpression) -> String {
        todo!()
    }

    fn visit_get_expression(&self, get_expression: &super::expr::GetExpression) -> String {
        todo!()
    }

    fn visit_grouping_expression(
        &self,
        grouping_expression: &super::expr::GroupingExpression,
    ) -> String {
        self.parenthesize("group", vec![&grouping_expression.expression])
    }

    fn visit_literal_expression(
        &self,
        literal_expression: &super::expr::LiteralExpression,
    ) -> String {
        match &literal_expression.value {
            super::token::Literal::String(string) => string.into(),
            super::token::Literal::Number(number) => format!("{}", number),
            super::token::Literal::False => "false".into(),
            super::token::Literal::True => "true".into(),
            super::token::Literal::Nil => "nil".into(),
        }
    }

    fn visit_logical_expression(
        &self,
        logical_expression: &super::expr::LogicalExpression,
    ) -> String {
        todo!()
    }

    fn visit_set_expression(&self, set_expression: &super::expr::SetExpression) -> String {
        todo!()
    }

    fn visit_super_expression(&self, super_expression: &super::expr::SuperExpression) -> String {
        todo!()
    }

    fn visit_this_expression(&self, this_expression: &super::expr::ThisExpression) -> String {
        todo!()
    }

    fn visit_ternary_expression(
        &self,
        ternary_expression: &super::expr::TernaryExpression,
    ) -> String {
        self.parenthesize(
            "ternary",
            vec![
                &ternary_expression.cmp,
                &ternary_expression.true_value,
                &ternary_expression.false_value,
            ],
        )
    }

    fn visit_unary_expression(&self, unary_expression: &super::expr::UnaryExpression) -> String {
        self.parenthesize(
            unary_expression.op.lexeme.as_str(),
            vec![&unary_expression.right],
        )
    }

    fn visit_variable_expression(
        &self,
        variable_expression: &super::expr::VariableExpression,
    ) -> String {
        todo!()
    }
}
