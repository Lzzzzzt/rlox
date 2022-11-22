use std::{collections::HashMap, rc::Rc};

use super::{
    error::LoxError,
    expr::{Expression, Visitor as ExprVisitor},
    stmt::{FunctionStatement, Statement, Visitor as StmtVisitor},
    types::{ClassType, FuncType},
};

#[allow(unused)]
pub struct Resolver {
    function_type: FuncType,
    class_type: ClassType,
    is_in_while: bool,
    var_use_table: HashMap<Rc<String>, bool>,
}

#[allow(unused)]
impl Resolver {
    pub fn new() -> Self {
        Self {
            function_type: FuncType::Main,
            class_type: ClassType::None,
            is_in_while: false,
            var_use_table: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, statements: &[Statement]) -> Result<(), LoxError> {
        self.resolve_statements(statements)?;
        if std::env::var("RLOX_RUN_MODE").unwrap().eq("F") {
            self.var_use_table
                .iter()
                .filter(|(_, &used)| !used)
                .for_each(|(name, _)| {
                    println!("\x1b[1;33m[WARN]:\x1b[0m Unused variable `{}`", name)
                });
        }
        Ok(())
    }

    fn variable_define(&mut self, name: Rc<String>) {
        self.var_use_table.insert(name, false);
    }

    fn variable_used(&mut self, name: Rc<String>) {
        if self.var_use_table.contains_key(&name) {
            self.var_use_table.insert(name, true);
        }
    }

    fn resolve_expression(&mut self, expression: &Expression) -> Result<(), LoxError> {
        expression.accept(self)
    }

    fn resolve_statement(&mut self, statement: &Statement) -> Result<(), LoxError> {
        statement.accept(self)
    }

    fn resolve_function(
        &mut self,
        statement: &FunctionStatement,
        function_type: FuncType,
    ) -> Result<(), LoxError> {
        let pre = self.function_type;
        self.function_type = function_type;
        self.resolve_statements(&statement.body)?;
        self.function_type = pre;
        Ok(())
    }

    fn resolve_statements(&mut self, statements: &[Statement]) -> Result<(), LoxError> {
        for statement in statements {
            statement.accept(self)?;
        }

        Ok(())
    }
}

#[allow(unused)]
impl ExprVisitor<(), LoxError> for Resolver {
    fn visit_assign_expression(
        &mut self,
        assign_expression: &super::expr::AssignExpression,
    ) -> Result<(), LoxError> {
        self.resolve_expression(&assign_expression.value)
    }

    fn visit_binary_expression(
        &mut self,
        binary_expression: &super::expr::BinaryExpression,
    ) -> Result<(), LoxError> {
        self.resolve_expression(&binary_expression.left)?;
        self.resolve_expression(&binary_expression.right)
    }

    fn visit_call_expression(
        &mut self,
        call_expression: &super::expr::CallExpression,
    ) -> Result<(), LoxError> {
        self.resolve_expression(&call_expression.callee)?;
        for param in &call_expression.arguments {
            self.resolve_expression(param)?;
        }
        Ok(())
    }

    fn visit_get_expression(
        &mut self,
        get_expression: &super::expr::GetExpression,
    ) -> Result<(), LoxError> {
        self.resolve_expression(&get_expression.object)
    }

    fn visit_grouping_expression(
        &mut self,
        grouping_expression: &super::expr::GroupingExpression,
    ) -> Result<(), LoxError> {
        self.resolve_expression(&grouping_expression.expression)
    }

    fn visit_literal_expression(
        &mut self,
        literal_expression: &super::expr::LiteralExpression,
    ) -> Result<(), LoxError> {
        Ok(())
    }

    fn visit_logical_expression(
        &mut self,
        logical_expression: &super::expr::LogicalExpression,
    ) -> Result<(), LoxError> {
        self.resolve_expression(&logical_expression.left)?;
        self.resolve_expression(&logical_expression.right)
    }

    fn visit_set_expression(
        &mut self,
        set_expression: &super::expr::SetExpression,
    ) -> Result<(), LoxError> {
        self.resolve_expression(&set_expression.object);
        self.resolve_expression(&set_expression.value)
    }

    fn visit_super_expression(
        &mut self,
        super_expression: &super::expr::SuperExpression,
    ) -> Result<(), LoxError> {
        todo!()
    }

    fn visit_self_expression(
        &mut self,
        self_expression: &super::expr::SelfExpression,
    ) -> Result<(), LoxError> {
        if ClassType::None == self.class_type
            && !(FuncType::Initializer == self.function_type
                || FuncType::Method == self.function_type)
        {
            return Err(LoxError::ParseError {
                position: self_expression.keyword.position,
                lexeme: self_expression.keyword.lexeme.clone(),
                token_type: self_expression.keyword.token_type,
                msg: String::from(
                    "Keyword `self` can only be used in method(static methods are not included).",
                ),
            });
        }

        Ok(())
    }

    fn visit_ternary_expression(
        &mut self,
        ternary_expression: &super::expr::TernaryExpression,
    ) -> Result<(), LoxError> {
        self.resolve_expression(&ternary_expression.cmp)?;
        self.resolve_expression(&ternary_expression.true_value)?;
        self.resolve_expression(&ternary_expression.false_value)
    }

    fn visit_unary_expression(
        &mut self,
        unary_expression: &super::expr::UnaryExpression,
    ) -> Result<(), LoxError> {
        self.resolve_expression(&unary_expression.right)
    }

    fn visit_variable_expression(
        &mut self,
        variable_expression: &super::expr::VariableExpression,
    ) -> Result<(), LoxError> {
        self.variable_used(variable_expression.name.lexeme.clone());
        Ok(())
    }

    fn visit_lambda_expression(
        &mut self,
        lambda_expression: &super::expr::LambdaExpression,
    ) -> Result<(), LoxError> {
        self.resolve_statements(&lambda_expression.body)
    }

    fn visit_operate_and_assign_expression(
        &mut self,
        operate_and_assign_expression: &super::expr::OperateAndAssignExpression,
    ) -> Result<(), LoxError> {
        self.resolve_expression(&operate_and_assign_expression.value)
    }
}

#[allow(unused)]
impl StmtVisitor<(), LoxError> for Resolver {
    fn visit_expression_statement(
        &mut self,
        expression_statement: &super::stmt::ExpressionStatement,
    ) -> Result<(), LoxError> {
        self.resolve_expression(&expression_statement.expression)
    }

    fn visit_print_statement(
        &mut self,
        print_statement: &super::stmt::PrintStatement,
    ) -> Result<(), LoxError> {
        self.resolve_expression(&print_statement.expression)
    }

    fn visit_var_statement(
        &mut self,
        var_statement: &super::stmt::VarStatement,
    ) -> Result<(), LoxError> {
        self.variable_define(var_statement.name.lexeme.clone());
        if let Some(init) = &var_statement.initializer {
            self.resolve_expression(init)
        } else {
            Ok(())
        }
    }

    fn visit_multi_var_statement(
        &mut self,
        multi_var_statement: &super::stmt::MultiVarStatement,
    ) -> Result<(), LoxError> {
        self.resolve_statements(&multi_var_statement.vars)
    }

    fn visit_block_statement(
        &mut self,
        block_statement: &super::stmt::BlockStatement,
    ) -> Result<(), LoxError> {
        self.resolve_statements(&block_statement.statements)
    }

    fn visit_branch_statement(
        &mut self,
        branch_statement: &super::stmt::BranchStatement,
    ) -> Result<(), LoxError> {
        self.resolve_expression(&branch_statement.condition)?;
        self.resolve_statement(&branch_statement.then_branch)?;
        if let Some(else_branch) = &branch_statement.else_branch {
            self.resolve_statement(else_branch)
        } else {
            Ok(())
        }
    }

    fn visit_while_statement(
        &mut self,
        while_statement: &super::stmt::WhileStatement,
    ) -> Result<(), LoxError> {
        self.resolve_expression(&while_statement.condition)?;
        let pre = self.is_in_while;
        self.is_in_while = true;
        self.resolve_statement(&while_statement.body)?;
        self.is_in_while = pre;
        if let Some(incr) = &while_statement.increment {
            self.resolve_statement(incr)
        } else {
            Ok(())
        }
    }

    fn visit_continue_statement(
        &mut self,
        continue_statement: &super::stmt::ContinueStatement,
    ) -> Result<(), LoxError> {
        if !self.is_in_while {
            Err(LoxError::ParseError {
                position: continue_statement.token.position,
                lexeme: continue_statement.token.lexeme.clone(),
                token_type: continue_statement.token.token_type,
                msg: String::from("`continue` can only be used in `while` or `for` statements"),
            })
        } else {
            Ok(())
        }
    }

    fn visit_break_statement(
        &mut self,
        break_statement: &super::stmt::BreakStatement,
    ) -> Result<(), LoxError> {
        if !self.is_in_while {
            Err(LoxError::ParseError {
                position: break_statement.token.position,
                lexeme: break_statement.token.lexeme.clone(),
                token_type: break_statement.token.token_type,
                msg: String::from("`break` can only be used in `while` or `for` statements"),
            })
        } else {
            Ok(())
        }
    }

    fn visit_function_statement(
        &mut self,
        function_statement: &super::stmt::FunctionStatement,
    ) -> Result<(), LoxError> {
        self.resolve_function(function_statement, function_statement.function_type)
    }

    fn visit_return_statement(
        &mut self,
        return_statement: &super::stmt::ReturnStatement,
    ) -> Result<(), LoxError> {
        if let FuncType::Main = self.function_type {
            Err(LoxError::ParseError {
                position: return_statement.key_word.position,
                lexeme: return_statement.key_word.lexeme.clone(),
                token_type: return_statement.key_word.token_type,
                msg: String::from("`return` can only be used in a function."),
            })
        } else if let Some(value) = &return_statement.value {
            if let FuncType::Initializer = self.function_type {
                return Err(LoxError::ParseError {
                    position: return_statement.key_word.position,
                    lexeme: return_statement.key_word.lexeme.clone(),
                    token_type: return_statement.key_word.token_type,
                    msg: String::from("function `__init__` can not return value."),
                });
            }
            self.resolve_expression(value)
        } else {
            Ok(())
        }
    }

    fn visit_class_statement(
        &mut self,
        class_statement: &super::stmt::ClassStatement,
    ) -> Result<(), LoxError> {
        let pre = self.class_type;
        self.class_type = ClassType::Class;
        for method in &class_statement.methods {
            if let Statement::FunctionStatement(m) = method {
                let mut func_type = FuncType::Method;
                if m.name.lexeme.as_ref().eq("__init__") {
                    func_type = FuncType::Initializer;
                }
                self.resolve_function(m, func_type)?;
            }
        }
        self.class_type = pre;
        for s_method in &class_statement.static_methods {
            if let Statement::FunctionStatement(sm) = s_method {
                self.resolve_function(sm, sm.function_type)?;
            }
        }
        Ok(())
    }
}
