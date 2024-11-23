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
        self.pretty_fmt(f, 0)
    }
}

impl fmt::Display for DebugAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.compact_fmt(f)
    }
}

impl DebugAccess {
    fn pretty_fmt(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let indentation = "  ".repeat(indent);
        match &self.0 {
            Access::ComponentAccess(name) => {
                writeln!(f, "{}ComponentAccess", indentation)?;
                writeln!(f, "{}  name: {}", indentation, name)
            }
            Access::ArrayAccess(expr) => {
                writeln!(f, "{}ArrayAccess:", indentation)?;
                DebugExpression(expr.clone()).pretty_fmt(f, indent + 2)
            }
        }
    }
}

impl DebugAccess {
    fn compact_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Access::ComponentAccess(name) => {
                write!(f, ".{}", name)
            }
            Access::ArrayAccess(expr) => {
                write!(
                    f,
                    "[{}]",
                    format!("{:?}", DebugExpression(expr.clone()))
                        .replace("\n", "")
                        .replace("  ", " ")
                )
            }
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
            ExpressionInfixOpcode::ShiftL => f.debug_struct("ShL").finish(),
            ExpressionInfixOpcode::ShiftR => f.debug_struct("ShR").finish(),
            ExpressionInfixOpcode::LesserEq => f.debug_struct("LEq").finish(),
            ExpressionInfixOpcode::GreaterEq => f.debug_struct("GEq").finish(),
            ExpressionInfixOpcode::Lesser => f.debug_struct("Lt").finish(),
            ExpressionInfixOpcode::Greater => f.debug_struct("Gt").finish(),
            ExpressionInfixOpcode::Eq => f.debug_struct("Eq").finish(),
            ExpressionInfixOpcode::NotEq => f.debug_struct("NEq").finish(),
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
        self.pretty_fmt(f, 0)
    }
}

impl fmt::Debug for ExtendedStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pretty_fmt(f, 0)
    }
}

const RESET: &str = "\x1b[0m";
const BLUE: &str = "\x1b[34m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";

impl DebugExpression {
    fn pretty_fmt(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let indentation = "  ".repeat(indent);
        match &self.0 {
            Expression::Number(_, value) => {
                writeln!(f, "{}{}Number:{} {}", indentation, BLUE, RESET, value)
            }
            Expression::InfixOp {
                lhe, infix_op, rhe, ..
            } => {
                writeln!(f, "{}{}InfixOp:{}", indentation, GREEN, RESET)?;
                writeln!(
                    f,
                    "{}  {}Operator:{} {:?}",
                    indentation,
                    CYAN,
                    RESET,
                    DebugExpressionInfixOpcode(*infix_op)
                )?;
                writeln!(
                    f,
                    "{}  {}Left-Hand Expression:{}",
                    indentation, YELLOW, RESET
                )?;
                DebugExpression(*lhe.clone()).pretty_fmt(f, indent + 2)?;
                writeln!(
                    f,
                    "{}  {}Right-Hand Expression:{}",
                    indentation, YELLOW, RESET
                )?;
                DebugExpression(*rhe.clone()).pretty_fmt(f, indent + 2)
            }
            Expression::PrefixOp { prefix_op, rhe, .. } => {
                writeln!(f, "{}{}PrefixOp:{}", indentation, GREEN, RESET)?;
                writeln!(
                    f,
                    "{}  {}Operator:{} {:?}",
                    indentation,
                    CYAN,
                    RESET,
                    DebugExpressionPrefixOpcode(*prefix_op)
                )?;
                writeln!(
                    f,
                    "{}  {}Right-Hand Expression:{}",
                    indentation, YELLOW, RESET
                )?;
                DebugExpression(*rhe.clone()).pretty_fmt(f, indent + 2)
            }
            Expression::ParallelOp { rhe, .. } => {
                writeln!(f, "{}ParallelOp", indentation)?;
                writeln!(
                    f,
                    "{}  {}Right-Hand Expression:{}",
                    indentation, YELLOW, RESET
                )?;
                DebugExpression(*rhe.clone()).pretty_fmt(f, indent + 2)
            }
            Expression::Variable { name, access, .. } => {
                writeln!(f, "{}{}Variable:{}", indentation, BLUE, RESET)?;
                writeln!(f, "{}  Name: {}", indentation, name)?;
                writeln!(f, "{}  Access:", indentation)?;
                for arg0 in access {
                    DebugAccess(arg0.clone()).pretty_fmt(f, indent + 2)?;
                }
                Ok(())
            }
            Expression::InlineSwitchOp {
                cond: _,
                if_true,
                if_false,
                ..
            } => {
                writeln!(f, "{}InlineSwitchOp:", indentation)?;
                writeln!(f, "{}  if_true:", indentation)?;
                DebugExpression(*if_true.clone()).pretty_fmt(f, indent + 2)?;
                writeln!(f, "{}  if_false:", indentation)?;
                DebugExpression(*if_false.clone()).pretty_fmt(f, indent + 2)
            }
            Expression::Call { id, args, .. } => {
                writeln!(f, "{}Call", indentation)?;
                writeln!(f, "{}  id: {}", indentation, id)?;
                writeln!(f, "{}  args:", indentation)?;
                for arg0 in args {
                    DebugExpression(arg0.clone()).pretty_fmt(f, indent + 2)?;
                }
                Ok(())
            }
            Expression::ArrayInLine { values, .. } => {
                writeln!(f, "{}ArrayInLine", indentation)?;
                writeln!(f, "{}  values:", indentation)?;
                for v in values {
                    DebugExpression(v.clone()).pretty_fmt(f, indent + 2)?;
                }
                Ok(())
            }
            Expression::Tuple { values, .. } => {
                writeln!(f, "{}Tuple", indentation)?;
                writeln!(f, "{}  values:", indentation)?;
                for v in values {
                    DebugExpression(v.clone()).pretty_fmt(f, indent + 2)?;
                }
                Ok(())
            }
            Expression::UniformArray {
                value, dimension, ..
            } => {
                writeln!(f, "{}UniformArray", indentation)?;
                writeln!(f, "{}  value:", indentation)?;
                DebugExpression(*value.clone()).pretty_fmt(f, indent + 2)?;
                writeln!(f, "{}  dimension:", indentation)?;
                DebugExpression(*dimension.clone()).pretty_fmt(f, indent + 2)
            }
            Expression::BusCall { id, args, .. } => {
                writeln!(f, "{}BusCall", indentation)?;
                writeln!(f, "{}  id:", id)?;
                writeln!(f, "{}  args:", indentation)?;
                for a in args {
                    DebugExpression(a.clone()).pretty_fmt(f, indent + 2)?;
                }
                Ok(())
            }
            Expression::AnonymousComp {
                id,
                is_parallel,
                params,
                signals,
                names: _,
                ..
            } => {
                writeln!(f, "{}AnonymousComp", indentation)?;
                writeln!(f, "{}  id: {}", indentation, id)?;
                //writeln!(f, "{}  name: {}", indentation, names)?;
                writeln!(f, "{}  is_parallel: {}", indentation, is_parallel)?;
                writeln!(f, "{}  params:", indentation)?;
                for p in params {
                    DebugExpression(p.clone()).pretty_fmt(f, indent + 2)?;
                }
                writeln!(f, "{}  signals:", indentation)?;
                for s in signals {
                    DebugExpression(s.clone()).pretty_fmt(f, indent + 2)?;
                }
                Ok(())
            }
        }
    }
}

impl ExtendedStatement {
    fn pretty_fmt(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let indentation = "  ".repeat(indent);
        match &self {
            ExtendedStatement::DebugStatement(stmt) => match &stmt {
                Statement::IfThenElse {
                    cond,
                    if_case,
                    else_case,
                    meta,
                    ..
                } => {
                    writeln!(
                        f,
                        "{}{}IfThenElse{} (elem_id={}):",
                        indentation, GREEN, RESET, meta.elem_id
                    )?;
                    writeln!(f, "{}  {}Condition:{}:", indentation, CYAN, RESET)?;
                    DebugExpression(cond.clone()).pretty_fmt(f, indent + 2)?;
                    writeln!(f, "{}  {}If Case:{}:", indentation, CYAN, RESET)?;
                    ExtendedStatement::DebugStatement(*if_case.clone())
                        .pretty_fmt(f, indent + 2)?;
                    if let Some(else_case) = else_case {
                        writeln!(f, "{}  {}Else Case:{}:", indentation, CYAN, RESET)?;
                        ExtendedStatement::DebugStatement(*else_case.clone())
                            .pretty_fmt(f, indent + 2)?;
                    }
                    Ok(())
                }
                Statement::While {
                    cond, stmt, meta, ..
                } => {
                    writeln!(
                        f,
                        "{}{}While{} (elem_id={}):",
                        indentation, GREEN, RESET, meta.elem_id
                    )?;
                    writeln!(f, "{}  {}Condition:{}:", indentation, CYAN, RESET)?;
                    DebugExpression(cond.clone()).pretty_fmt(f, indent + 2)?;
                    writeln!(f, "{}  {}Statement:{}:", indentation, CYAN, RESET)?;
                    ExtendedStatement::DebugStatement(*stmt.clone()).pretty_fmt(f, indent + 2)
                }
                Statement::Return { value, meta, .. } => {
                    writeln!(
                        f,
                        "{}{}Return{} (elem_id={}):",
                        indentation, GREEN, RESET, meta.elem_id
                    )?;
                    writeln!(f, "{}  {}Value:{}:", indentation, MAGENTA, RESET)?;
                    DebugExpression(value.clone()).pretty_fmt(f, indent + 2)
                }
                Statement::Substitution {
                    var,
                    access,
                    op,
                    rhe,
                    meta,
                    ..
                } => {
                    writeln!(
                        f,
                        "{}{}Substitution{} (elem_id={}):",
                        indentation, GREEN, RESET, meta.elem_id
                    )?;
                    writeln!(f, "{}  {}Variable:{} {}", indentation, BLUE, RESET, var)?;
                    writeln!(f, "{}  {}Access:{}", indentation, MAGENTA, RESET)?;
                    for arg0 in access {
                        DebugAccess(arg0.clone()).pretty_fmt(f, indent + 2)?;
                    }
                    writeln!(
                        f,
                        "{}  {}Operation:{} {:?}",
                        indentation,
                        CYAN,
                        RESET,
                        DebugAssignOp(op.clone())
                    )?;
                    writeln!(
                        f,
                        "{}  {}Right-Hand Expression:{}:",
                        indentation, YELLOW, RESET
                    )?;
                    DebugExpression(rhe.clone()).pretty_fmt(f, indent + 2)
                }
                Statement::Block { stmts, meta, .. } => {
                    writeln!(
                        f,
                        "{}{}Block{} (elem_id={}):",
                        indentation, GREEN, RESET, meta.elem_id
                    )?;
                    writeln!(
                        f,
                        "{}    {}-------------------------------{}",
                        indentation, RED, RESET
                    )?;
                    for stmt in stmts {
                        ExtendedStatement::DebugStatement(stmt.clone())
                            .pretty_fmt(f, indent + 2)?;
                        writeln!(
                            f,
                            "{}    {}-------------------------------{}",
                            indentation, RED, RESET
                        )?;
                    }
                    Ok(())
                }
                Statement::Assert { arg, meta, .. } => {
                    writeln!(
                        f,
                        "{}{}Assert{} (elem_id={}):",
                        indentation, GREEN, RESET, meta.elem_id
                    )?;
                    writeln!(f, "{}  {}Argument:{}:", indentation, YELLOW, RESET)?;
                    DebugExpression(arg.clone()).pretty_fmt(f, indent + 2)
                }
                Statement::InitializationBlock {
                    meta,
                    xtype: _,
                    initializations,
                } => {
                    writeln!(
                        f,
                        "{}{}InitializationBlock{} (elem_id={}):",
                        indentation, GREEN, RESET, meta.elem_id
                    )?;
                    writeln!(f, "{}  {}Initializations:{}:", indentation, YELLOW, RESET)?;
                    for i in initializations {
                        ExtendedStatement::DebugStatement(i.clone()).pretty_fmt(f, indent + 2)?;
                    }
                    Ok(())
                }
                Statement::Declaration {
                    meta,
                    xtype: _,
                    name,
                    dimensions,
                    is_constant,
                } => {
                    writeln!(
                        f,
                        "{}{}Declaration{} (elem_id={}):",
                        indentation, GREEN, RESET, meta.elem_id
                    )?;
                    writeln!(f, "{}  {}Name:{} {}", indentation, MAGENTA, RESET, name)?;
                    writeln!(f, "{}  {}Dimensions:{}:", indentation, YELLOW, RESET)?;
                    for dim in dimensions {
                        DebugExpression(dim.clone()).pretty_fmt(f, indent + 2)?;
                    }
                    writeln!(
                        f,
                        "{}  {}Is Constant:{} {}",
                        indentation, CYAN, RESET, is_constant
                    )
                }
                Statement::MultSubstitution {
                    lhe, op, rhe, meta, ..
                } => {
                    writeln!(
                        f,
                        "{}{}MultSubstitution{} (elem_id={}):",
                        indentation, GREEN, RESET, meta.elem_id
                    )?;
                    writeln!(
                        f,
                        "{}  {}Op:{} {:?}",
                        indentation,
                        CYAN,
                        RESET,
                        DebugAssignOp(op.clone())
                    )?;
                    writeln!(
                        f,
                        "{}  {}Left-Hand Expression:{}:",
                        indentation, YELLOW, RESET
                    )?;
                    DebugExpression(lhe.clone()).pretty_fmt(f, indent + 2)?;
                    writeln!(
                        f,
                        "{}  {}Right-Hand Expression:{}:",
                        indentation, YELLOW, RESET
                    )?;
                    DebugExpression(rhe.clone()).pretty_fmt(f, indent + 2)
                }
                Statement::UnderscoreSubstitution { op, rhe, meta, .. } => {
                    writeln!(
                        f,
                        "{}{}UnderscoreSubstitution{} (elem_id={}):",
                        indentation, GREEN, RESET, meta.elem_id
                    )?;
                    writeln!(
                        f,
                        "{}  {}Op:{} {:?}",
                        indentation,
                        CYAN,
                        RESET,
                        DebugAssignOp(op.clone())
                    )?;
                    writeln!(
                        f,
                        "{}  {}Right-Hand Expression:{}:",
                        indentation, YELLOW, RESET
                    )?;
                    DebugExpression(rhe.clone()).pretty_fmt(f, indent + 2)
                }
                Statement::ConstraintEquality { lhe, rhe, meta, .. } => {
                    writeln!(
                        f,
                        "{}{}ConstraintEquality{} (elem_id={}):",
                        indentation, GREEN, RESET, meta.elem_id
                    )?;
                    writeln!(
                        f,
                        "{}  {}Left-Hand Expression:{}:",
                        indentation, YELLOW, RESET
                    )?;
                    DebugExpression(lhe.clone()).pretty_fmt(f, indent + 2)?;
                    writeln!(
                        f,
                        "{}  {}Right-Hand Expression:{}:",
                        indentation, YELLOW, RESET
                    )?;
                    DebugExpression(rhe.clone()).pretty_fmt(f, indent + 2)
                }
                Statement::LogCall { args: _, .. } => {
                    writeln!(f, "{}{}LogCall{}", indentation, GREEN, RESET)
                }
            },
            ExtendedStatement::Ret => writeln!(f, "{}{}Ret{}", indentation, BLUE, RESET),
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
