use super::input_user::Input;
use crate::VERSION;
use program_structure::abstract_syntax_tree::ast::{
    Access, AssignOp, Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode, Statement,
};
use program_structure::constants::UsefulConstants;
use program_structure::error_definition::Report;
use program_structure::program_archive::ProgramArchive;
use std::fmt;

#[derive(Clone)]
pub struct DebugAccess(pub Access);
#[derive(Clone)]
pub struct DebugAssignOp(pub AssignOp);
#[derive(Clone)]
pub struct DebugExpressionInfixOpcode(pub ExpressionInfixOpcode);
#[derive(Clone)]
pub struct DebugExpressionPrefixOpcode(pub ExpressionPrefixOpcode);
#[derive(Clone)]
pub struct DebugExpression(pub Expression);
#[derive(Clone)]
pub enum ExtendedStatement {
    DebugStatement(Statement),
    Ret,
}

impl fmt::Debug for DebugAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Access::ComponentAccess(name) => f
                .debug_struct("ComponentAccess")
                .field("name", &name)
                .finish(),
            Access::ArrayAccess(expr) => f
                .debug_struct("ArrayAccess")
                .field("expr", &DebugExpression(expr.clone()))
                .finish(),
        }
    }
}

impl fmt::Debug for DebugAssignOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            AssignOp::AssignVar => f.debug_struct("AssignVar").finish(),
            AssignOp::AssignSignal => f.debug_struct("AssignSignal").finish(),
            AssignOp::AssignConstraintSignal => f.debug_struct("AssignConstraintSignal").finish(),
        }
    }
}

impl fmt::Debug for DebugExpressionInfixOpcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ExpressionInfixOpcode::Mul => f.debug_struct("Mul").finish(),
            ExpressionInfixOpcode::Div => f.debug_struct("Div").finish(),
            ExpressionInfixOpcode::Add => f.debug_struct("Add").finish(),
            ExpressionInfixOpcode::Sub => f.debug_struct("Sub").finish(),
            ExpressionInfixOpcode::Pow => f.debug_struct("Pow").finish(),
            ExpressionInfixOpcode::IntDiv => f.debug_struct("IntDiv").finish(),
            ExpressionInfixOpcode::Mod => f.debug_struct("Mod").finish(),
            ExpressionInfixOpcode::ShiftL => f.debug_struct("ShiftL").finish(),
            ExpressionInfixOpcode::ShiftR => f.debug_struct("ShiftR").finish(),
            ExpressionInfixOpcode::LesserEq => f.debug_struct("LesserEq").finish(),
            ExpressionInfixOpcode::GreaterEq => f.debug_struct("GreaterEq").finish(),
            ExpressionInfixOpcode::Lesser => f.debug_struct("Lesser").finish(),
            ExpressionInfixOpcode::Greater => f.debug_struct("Greater").finish(),
            ExpressionInfixOpcode::Eq => f.debug_struct("Eq").finish(),
            ExpressionInfixOpcode::NotEq => f.debug_struct("NotEq").finish(),
            ExpressionInfixOpcode::BoolOr => f.debug_struct("BoolOr").finish(),
            ExpressionInfixOpcode::BoolAnd => f.debug_struct("BoolAnd").finish(),
            ExpressionInfixOpcode::BitOr => f.debug_struct("BitOr").finish(),
            ExpressionInfixOpcode::BitAnd => f.debug_struct("BitAnd").finish(),
            ExpressionInfixOpcode::BitXor => f.debug_struct("BitXor").finish(),
        }
    }
}

impl fmt::Debug for DebugExpressionPrefixOpcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ExpressionPrefixOpcode::Sub => f.debug_struct("Minus").finish(),
            ExpressionPrefixOpcode::BoolNot => f.debug_struct("BoolNot").finish(),
            ExpressionPrefixOpcode::Complement => f.debug_struct("Complement").finish(),
        }
    }
}

impl fmt::Debug for DebugExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Expression::InfixOp {
                meta: _,
                lhe,
                infix_op,
                rhe,
            } => f
                .debug_struct("InfixOp")
                .field("infix_op", &DebugExpressionInfixOpcode(*infix_op))
                .field("lhe", &DebugExpression(*lhe.clone()))
                .field("rhe", &DebugExpression(*rhe.clone()))
                .finish(),
            Expression::PrefixOp {
                meta: _,
                prefix_op,
                rhe,
            } => f
                .debug_struct("PrefixOp")
                .field("prefix_op", &DebugExpressionPrefixOpcode(*prefix_op))
                .field("rhe", &DebugExpression(*rhe.clone()))
                .finish(),
            Expression::InlineSwitchOp {
                meta: _,
                cond,
                if_true,
                if_false,
            } => f
                .debug_struct("InlineSwitchOp")
                .field("cond", &DebugExpression(*cond.clone()))
                .field("if_true", &DebugExpression(*if_true.clone()))
                .field("if_false", &DebugExpression(*if_false.clone()))
                .finish(),
            Expression::ParallelOp { meta: _, rhe } => f
                .debug_struct("ParallelOp")
                .field("rhe", &DebugExpression(*rhe.clone()))
                .finish(),
            Expression::Variable {
                meta: _,
                name,
                access,
            } => f
                .debug_struct("Variable")
                .field("name", &name)
                .field(
                    "access",
                    &access
                        .iter()
                        .map(|arg0: &Access| DebugAccess(arg0.clone()))
                        .collect::<Vec<_>>(),
                )
                .finish(),
            Expression::Number(_, value) => {
                //write!("Number {}", "{}", value)
                f.debug_struct("Number").field("value", &value).finish()
            }
            Expression::Call { meta: _, id, args } => f
                .debug_struct("Call")
                .field("id", &id)
                .field(
                    "args",
                    &args
                        .iter()
                        .map(|arg0: &Expression| DebugExpression(arg0.clone()))
                        .collect::<Vec<_>>(),
                )
                .finish(),
            Expression::BusCall { meta: _, id, args } => f
                .debug_struct("BusCall")
                .field("id", &id)
                .field(
                    "args",
                    &args
                        .iter()
                        .map(|arg0: &Expression| DebugExpression(arg0.clone()))
                        .collect::<Vec<_>>(),
                )
                .finish(),
            Expression::AnonymousComp {
                meta: _,
                id,
                is_parallel,
                params,
                signals,
                names: _,
            } => f
                .debug_struct("AnonymousComp")
                .field("id", &id)
                .field("is_parallel", &is_parallel)
                .field(
                    "params",
                    &params
                        .iter()
                        .map(|arg0: &Expression| DebugExpression(arg0.clone()))
                        .collect::<Vec<_>>(),
                )
                .field(
                    "signals",
                    &signals
                        .iter()
                        .map(|arg0: &Expression| DebugExpression(arg0.clone()))
                        .collect::<Vec<_>>(),
                )
                //.field("names", names)
                .finish(),
            Expression::ArrayInLine { meta: _, values } => f
                .debug_struct("ArrayInLine")
                .field(
                    "values",
                    &values
                        .iter()
                        .map(|arg0: &Expression| DebugExpression(arg0.clone()))
                        .collect::<Vec<_>>(),
                )
                .finish(),
            Expression::Tuple { meta: _, values } => f
                .debug_struct("Tuple")
                .field(
                    "values",
                    &values
                        .iter()
                        .map(|arg0: &Expression| DebugExpression(arg0.clone()))
                        .collect::<Vec<_>>(),
                )
                .finish(),
            Expression::UniformArray {
                meta: _,
                value,
                dimension,
            } => f
                .debug_struct("UniformArray")
                .field("value", &DebugExpression(*value.clone()))
                .field("dimension", &DebugExpression(*dimension.clone()))
                .finish(),
        }
    }
}

impl fmt::Debug for ExtendedStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            ExtendedStatement::DebugStatement(stmt) => {
                match &stmt {
                    Statement::IfThenElse {
                        meta: _,
                        cond,
                        if_case,
                        else_case,
                    } => {
                        if else_case.is_none() {
                            f.debug_struct("IfThenElse")
                                .field("condition", &DebugExpression(cond.clone()))
                                .field(
                                    "if_case",
                                    &ExtendedStatement::DebugStatement(*if_case.clone()),
                                )
                                .finish()
                        } else {
                            f.debug_struct("IfThenElse")
                                .field("condition", &DebugExpression(cond.clone()))
                                .field(
                                    "if_case",
                                    &ExtendedStatement::DebugStatement(*if_case.clone()),
                                )
                                .field(
                                    "else_case",
                                    &ExtendedStatement::DebugStatement(
                                        *else_case.clone().unwrap().clone(),
                                    ),
                                )
                                .finish()
                        }
                    }
                    Statement::While {
                        meta: _,
                        cond,
                        stmt,
                    } => f
                        .debug_struct("While")
                        .field("condition", &DebugExpression(cond.clone()))
                        .field(
                            "statement",
                            &ExtendedStatement::DebugStatement(*stmt.clone()),
                        )
                        .finish(),
                    Statement::Return { meta: _, value } => f
                        .debug_struct("Return")
                        .field("value", &DebugExpression(value.clone()))
                        .finish(),
                    Statement::InitializationBlock {
                        meta: _,
                        xtype: _,
                        initializations,
                    } => f
                        .debug_struct("InitializationBlock")
                        //.field("type", xtype)
                        .field(
                            "initializations",
                            &initializations
                                .iter()
                                .map(|arg0: &Statement| {
                                    ExtendedStatement::DebugStatement(arg0.clone())
                                })
                                .collect::<Vec<_>>(),
                        )
                        .finish(),
                    Statement::Declaration {
                        meta: _,
                        xtype: _,
                        name,
                        dimensions,
                        is_constant,
                    } => f
                        .debug_struct("Declaration")
                        //.field("type", xtype)
                        .field("name", &name)
                        .field(
                            "dimensions",
                            &dimensions
                                .iter()
                                .map(|arg0: &Expression| DebugExpression(arg0.clone()))
                                .collect::<Vec<_>>(),
                        )
                        .field("is_constant", &is_constant)
                        .finish(),
                    Statement::Substitution {
                        meta: _,
                        var,
                        access,
                        op,
                        rhe,
                    } => f
                        .debug_struct("Substitution")
                        .field("variable", &var)
                        .field(
                            "access",
                            &access
                                .iter()
                                .map(|arg0: &Access| DebugAccess(arg0.clone()))
                                .collect::<Vec<_>>(),
                        )
                        .field("operation", &DebugAssignOp(op.clone()))
                        .field("rhe", &DebugExpression(rhe.clone()))
                        .finish(),
                    Statement::MultSubstitution { lhe, op, rhe, .. } => f
                        .debug_struct("MultSubstitution")
                        .field("lhs_expression", &DebugExpression(lhe.clone()))
                        .field("operation", &DebugAssignOp(op.clone()))
                        .field("rhs_expression", &DebugExpression(rhe.clone()))
                        .finish(),
                    Statement::UnderscoreSubstitution { op, rhe, .. } => f
                        .debug_struct("UnderscoreSubstitution")
                        .field("operation", &DebugAssignOp(op.clone()))
                        .field("rhe", &DebugExpression(rhe.clone()))
                        .finish(),
                    Statement::ConstraintEquality { meta: _, lhe, rhe } => f
                        .debug_struct("ConstraintEquality")
                        .field("lhs_expression", &DebugExpression(lhe.clone()))
                        .field("rhs_expression", &DebugExpression(rhe.clone()))
                        .finish(),
                    Statement::LogCall { meta: _, args: _ } => {
                        f.debug_struct("LogCall").finish()
                        //f.debug_struct("LogCall").field("arguments", args).finish()
                    }
                    Statement::Block { meta: _, stmts } => f
                        .debug_struct("Block")
                        .field(
                            "statements",
                            &stmts
                                .iter()
                                .map(|arg0: &Statement| {
                                    ExtendedStatement::DebugStatement(arg0.clone())
                                })
                                .collect::<Vec<_>>(),
                        )
                        .finish(),
                    Statement::Assert { meta: _, arg } => f
                        .debug_struct("Assert")
                        .field("argument", &DebugExpression(arg.clone()))
                        .finish(),
                }
            }
            ExtendedStatement::Ret => f.debug_struct("Ret").finish(),
        }
    }
}

pub fn parse_project(input_info: &Input) -> Result<ProgramArchive, ()> {
    let initial_file = input_info.input_file().to_string();
    //We get the prime number from the input
    let prime = UsefulConstants::new(&input_info.prime()).get_p().clone();
    let result_program_archive = parser::run_parser(
        initial_file,
        VERSION,
        input_info.get_link_libraries().to_vec(),
        &prime,
    );
    match result_program_archive {
        Result::Err((file_library, report_collection)) => {
            Report::print_reports(&report_collection, &file_library);
            Result::Err(())
        }
        Result::Ok((program_archive, warnings)) => {
            Report::print_reports(&warnings, &program_archive.file_library);
            Result::Ok(program_archive)
        }
    }
}
