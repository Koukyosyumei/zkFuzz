use num_bigint_dig::BigInt;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem;

use program_structure::abstract_syntax_tree::ast::{
    Access, AssignOp, Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode, SignalType,
    Statement, VariableType,
};
use program_structure::ast::LogArgument;
use program_structure::ast::Meta;
use program_structure::constants::UsefulConstants;
use program_structure::error_definition::Report;
use program_structure::program_archive::ProgramArchive;

use super::input_user::Input;
use crate::VERSION;

#[derive(Clone)]
pub struct DebugSignalType(pub SignalType);
#[derive(Clone)]
pub struct DebugVariableType(pub VariableType);
#[derive(Clone)]
pub struct DebugAccess(pub Access);
#[derive(Clone)]
pub struct DebugAssignOp(pub AssignOp);
#[derive(Clone, PartialEq)]
pub struct DebugExpressionInfixOpcode(pub ExpressionInfixOpcode);
#[derive(Clone, PartialEq)]
pub struct DebugExpressionPrefixOpcode(pub ExpressionPrefixOpcode);

#[derive(Clone)]
pub enum DebugExpression {
    InfixOp {
        meta: Meta,
        lhe: Box<DebugExpression>,
        infix_op: DebugExpressionInfixOpcode,
        rhe: Box<DebugExpression>,
    },
    PrefixOp {
        meta: Meta,
        prefix_op: DebugExpressionPrefixOpcode,
        rhe: Box<DebugExpression>,
    },
    InlineSwitchOp {
        meta: Meta,
        cond: Box<DebugExpression>,
        if_true: Box<DebugExpression>,
        if_false: Box<DebugExpression>,
    },
    ParallelOp {
        meta: Meta,
        rhe: Box<DebugExpression>,
    },
    Variable {
        meta: Meta,
        name: String,
        access: Vec<Access>,
    },
    Number(Meta, BigInt),
    Call {
        meta: Meta,
        id: String,
        args: Vec<DebugExpression>,
    },
    BusCall {
        meta: Meta,
        id: String,
        args: Vec<DebugExpression>,
    },
    AnonymousComp {
        meta: Meta,
        id: String,
        is_parallel: bool,
        params: Vec<DebugExpression>,
        signals: Vec<DebugExpression>,
        names: Option<Vec<(AssignOp, String)>>,
    },
    ArrayInLine {
        meta: Meta,
        values: Vec<DebugExpression>,
    },
    Tuple {
        meta: Meta,
        values: Vec<DebugExpression>,
    },
    UniformArray {
        meta: Meta,
        value: Box<DebugExpression>,
        dimension: Box<DebugExpression>,
    },
}

#[derive(Clone)]
pub enum DebugStatement {
    IfThenElse {
        meta: Meta,
        cond: DebugExpression,
        if_case: Box<DebugStatement>,
        else_case: Option<Box<DebugStatement>>,
    },
    While {
        meta: Meta,
        cond: DebugExpression,
        stmt: Box<DebugStatement>,
    },
    Return {
        meta: Meta,
        value: DebugExpression,
    },
    InitializationBlock {
        meta: Meta,
        xtype: VariableType,
        initializations: Vec<DebugStatement>,
    },
    Declaration {
        meta: Meta,
        xtype: VariableType,
        name: String,
        dimensions: Vec<DebugExpression>,
        is_constant: bool,
    },
    Substitution {
        meta: Meta,
        var: String,
        access: Vec<Access>,
        op: DebugAssignOp,
        rhe: DebugExpression,
    },
    MultSubstitution {
        meta: Meta,
        lhe: DebugExpression,
        op: DebugAssignOp,
        rhe: DebugExpression,
    },
    UnderscoreSubstitution {
        meta: Meta,
        op: DebugAssignOp,
        rhe: DebugExpression,
    },
    ConstraintEquality {
        meta: Meta,
        lhe: DebugExpression,
        rhe: DebugExpression,
    },
    LogCall {
        meta: Meta,
        args: Vec<LogArgument>,
    },
    Block {
        meta: Meta,
        stmts: Vec<DebugStatement>,
    },
    Assert {
        meta: Meta,
        arg: DebugExpression,
    },
    Ret,
}

impl From<Expression> for DebugExpression {
    fn from(expr: Expression) -> Self {
        match expr {
            Expression::InfixOp {
                meta,
                lhe,
                infix_op,
                rhe,
            } => DebugExpression::InfixOp {
                meta,
                lhe: Box::new((*lhe).into()),
                infix_op: DebugExpressionInfixOpcode(infix_op),
                rhe: Box::new((*rhe).into()),
            },
            Expression::PrefixOp {
                meta,
                prefix_op,
                rhe,
            } => DebugExpression::PrefixOp {
                meta,
                prefix_op: DebugExpressionPrefixOpcode(prefix_op),
                rhe: Box::new((*rhe).into()),
            },
            Expression::InlineSwitchOp {
                meta,
                cond,
                if_true,
                if_false,
            } => DebugExpression::InlineSwitchOp {
                meta,
                cond: Box::new((*cond).into()),
                if_true: Box::new((*if_true).into()),
                if_false: Box::new((*if_false).into()),
            },
            Expression::ParallelOp { meta, rhe } => DebugExpression::ParallelOp {
                meta,
                rhe: Box::new((*rhe).into()),
            },
            Expression::Variable { meta, name, access } => {
                DebugExpression::Variable { meta, name, access }
            }
            Expression::Number(meta, value) => DebugExpression::Number(meta, value),
            Expression::Call { meta, id, args } => DebugExpression::Call {
                meta,
                id,
                args: args.into_iter().map(|arg| arg.into()).collect(),
            },
            Expression::BusCall { meta, id, args } => DebugExpression::BusCall {
                meta,
                id,
                args: args.into_iter().map(|arg| arg.into()).collect(),
            },
            Expression::AnonymousComp {
                meta,
                id,
                is_parallel,
                params,
                signals,
                names,
            } => DebugExpression::AnonymousComp {
                meta,
                id,
                is_parallel,
                params: params.into_iter().map(|p| p.into()).collect(),
                signals: signals.into_iter().map(|s| s.into()).collect(),
                names,
            },
            Expression::ArrayInLine { meta, values } => DebugExpression::ArrayInLine {
                meta,
                values: values.into_iter().map(|v| v.into()).collect(),
            },
            Expression::Tuple { meta, values } => DebugExpression::Tuple {
                meta,
                values: values.into_iter().map(|v| v.into()).collect(),
            },
            Expression::UniformArray {
                meta,
                value,
                dimension,
            } => DebugExpression::UniformArray {
                meta,
                value: Box::new((*value).into()),
                dimension: Box::new((*dimension).into()),
            },
        }
    }
}

impl From<Statement> for DebugStatement {
    fn from(stmt: Statement) -> Self {
        match stmt {
            Statement::IfThenElse {
                meta,
                cond,
                if_case,
                else_case,
            } => DebugStatement::IfThenElse {
                meta,
                cond: cond.into(),
                if_case: Box::new((*if_case).into()),
                else_case: else_case.map(|else_case| Box::new((*else_case).into())),
            },
            Statement::While { meta, cond, stmt } => DebugStatement::While {
                meta,
                cond: cond.into(),
                stmt: Box::new((*stmt).into()),
            },
            Statement::Return { meta, value } => DebugStatement::Return {
                meta,
                value: value.into(),
            },
            Statement::InitializationBlock {
                meta,
                xtype,
                initializations,
            } => DebugStatement::InitializationBlock {
                meta,
                xtype,
                initializations: initializations
                    .into_iter()
                    .map(|stmt| stmt.into())
                    .collect(),
            },
            Statement::Declaration {
                meta,
                xtype,
                name,
                dimensions,
                is_constant,
            } => DebugStatement::Declaration {
                meta,
                xtype,
                name,
                dimensions: dimensions.into_iter().map(|dim| dim.into()).collect(),
                is_constant,
            },
            Statement::Substitution {
                meta,
                var,
                access,
                op,
                rhe,
            } => DebugStatement::Substitution {
                meta,
                var,
                access,
                op: DebugAssignOp(op),
                rhe: rhe.into(),
            },
            Statement::MultSubstitution { meta, lhe, op, rhe } => {
                DebugStatement::MultSubstitution {
                    meta,
                    lhe: lhe.into(),
                    op: DebugAssignOp(op),
                    rhe: rhe.into(),
                }
            }
            Statement::UnderscoreSubstitution { meta, op, rhe } => {
                DebugStatement::UnderscoreSubstitution {
                    meta,
                    op: DebugAssignOp(op),
                    rhe: rhe.into(),
                }
            }
            Statement::ConstraintEquality { meta, lhe, rhe } => {
                DebugStatement::ConstraintEquality {
                    meta,
                    lhe: lhe.into(),
                    rhe: rhe.into(),
                }
            }
            Statement::LogCall { meta, args } => DebugStatement::LogCall { meta, args },
            Statement::Block { meta, stmts } => DebugStatement::Block {
                meta,
                stmts: stmts.into_iter().map(|stmt| stmt.into()).collect(),
            },
            Statement::Assert { meta, arg } => DebugStatement::Assert {
                meta,
                arg: arg.into(),
            },
        }
    }
}

impl Hash for DebugExpressionInfixOpcode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(&self.0).hash(state);
    }
}

impl Eq for DebugExpressionInfixOpcode {}

impl Hash for DebugExpressionPrefixOpcode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(&self.0).hash(state);
    }
}

impl Eq for DebugExpressionPrefixOpcode {}

impl fmt::Debug for DebugSignalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            SignalType::Output => {
                write!(f, "Output")
            }
            SignalType::Input => {
                write!(f, "Input")
            }
            SignalType::Intermediate => {
                write!(f, "Intermediate")
            }
        }
    }
}

impl fmt::Debug for DebugVariableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            VariableType::Var => {
                write!(f, "Var")
            }
            VariableType::Signal(signaltype, taglist) => {
                write!(
                    f,
                    "Signal: {:?} {:?}",
                    &DebugSignalType(*signaltype),
                    &taglist
                )
            }
            VariableType::Component => {
                write!(f, "Component")
            }
            VariableType::AnonymousComponent => {
                write!(f, "AnonymousComponent")
            }
            VariableType::Bus(name, signaltype, taglist) => {
                write!(
                    f,
                    "Bus: {} {:?} {:?}",
                    name,
                    &DebugSignalType(*signaltype),
                    &taglist
                )
            }
        }
    }
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
                DebugExpression::from(expr.clone()).pretty_fmt(f, indent + 2)
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
                    format!("{:?}", DebugExpression::from(expr.clone()))
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

impl fmt::Debug for DebugStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pretty_fmt(f, 0)
    }
}

const RESET: &str = "\x1b[0m";
const BLUE: &str = "\x1b[34m"; //94
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";

impl DebugExpression {
    fn pretty_fmt(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let indentation = "  ".repeat(indent);
        match &self {
            DebugExpression::Number(_, value) => {
                writeln!(f, "{}{}Number:{} {}", indentation, BLUE, RESET, value)
            }
            DebugExpression::InfixOp {
                lhe, infix_op, rhe, ..
            } => {
                writeln!(f, "{}{}InfixOp:{}", indentation, GREEN, RESET)?;
                writeln!(
                    f,
                    "{}  {}Operator:{} {:?}",
                    indentation, CYAN, RESET, infix_op
                )?;
                writeln!(
                    f,
                    "{}  {}Left-Hand Expression:{}",
                    indentation, YELLOW, RESET
                )?;
                (*lhe.clone()).pretty_fmt(f, indent + 2)?;
                writeln!(
                    f,
                    "{}  {}Right-Hand Expression:{}",
                    indentation, YELLOW, RESET
                )?;
                (*rhe.clone()).pretty_fmt(f, indent + 2)
            }
            DebugExpression::PrefixOp { prefix_op, rhe, .. } => {
                writeln!(f, "{}{}PrefixOp:{}", indentation, GREEN, RESET)?;
                writeln!(
                    f,
                    "{}  {}Operator:{} {:?}",
                    indentation, CYAN, RESET, prefix_op
                )?;
                writeln!(
                    f,
                    "{}  {}Right-Hand Expression:{}",
                    indentation, YELLOW, RESET
                )?;
                (*rhe.clone()).pretty_fmt(f, indent + 2)
            }
            DebugExpression::ParallelOp { rhe, .. } => {
                writeln!(f, "{}ParallelOp", indentation)?;
                writeln!(
                    f,
                    "{}  {}Right-Hand Expression:{}",
                    indentation, YELLOW, RESET
                )?;
                (*rhe.clone()).pretty_fmt(f, indent + 2)
            }
            DebugExpression::Variable { name, access, .. } => {
                writeln!(f, "{}{}Variable:{}", indentation, BLUE, RESET)?;
                writeln!(f, "{}  Name: {}", indentation, name)?;
                writeln!(f, "{}  Access:", indentation)?;
                for arg0 in access {
                    DebugAccess(arg0.clone()).pretty_fmt(f, indent + 2)?;
                }
                Ok(())
            }
            DebugExpression::InlineSwitchOp {
                cond: _,
                if_true,
                if_false,
                ..
            } => {
                writeln!(f, "{}InlineSwitchOp:", indentation)?;
                writeln!(f, "{}  if_true:", indentation)?;
                (*if_true.clone()).pretty_fmt(f, indent + 2)?;
                writeln!(f, "{}  if_false:", indentation)?;
                (*if_false.clone()).pretty_fmt(f, indent + 2)
            }
            DebugExpression::Call { id, args, .. } => {
                writeln!(f, "{}Call", indentation)?;
                writeln!(f, "{}  id: {}", indentation, id)?;
                writeln!(f, "{}  args:", indentation)?;
                for arg0 in args {
                    (arg0.clone()).pretty_fmt(f, indent + 2)?;
                }
                Ok(())
            }
            DebugExpression::ArrayInLine { values, .. } => {
                writeln!(f, "{}ArrayInLine", indentation)?;
                writeln!(f, "{}  values:", indentation)?;
                for v in values {
                    (v.clone()).pretty_fmt(f, indent + 2)?;
                }
                Ok(())
            }
            DebugExpression::Tuple { values, .. } => {
                writeln!(f, "{}Tuple", indentation)?;
                writeln!(f, "{}  values:", indentation)?;
                for v in values {
                    (v.clone()).pretty_fmt(f, indent + 2)?;
                }
                Ok(())
            }
            DebugExpression::UniformArray {
                value, dimension, ..
            } => {
                writeln!(f, "{}UniformArray", indentation)?;
                writeln!(f, "{}  value:", indentation)?;
                (*value.clone()).pretty_fmt(f, indent + 2)?;
                writeln!(f, "{}  dimension:", indentation)?;
                (*dimension.clone()).pretty_fmt(f, indent + 2)
            }
            DebugExpression::BusCall { id, args, .. } => {
                writeln!(f, "{}BusCall", indentation)?;
                writeln!(f, "{}  id:", id)?;
                writeln!(f, "{}  args:", indentation)?;
                for a in args {
                    (a.clone()).pretty_fmt(f, indent + 2)?;
                }
                Ok(())
            }
            DebugExpression::AnonymousComp {
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
                    (p.clone()).pretty_fmt(f, indent + 2)?;
                }
                writeln!(f, "{}  signals:", indentation)?;
                for s in signals {
                    (s.clone()).pretty_fmt(f, indent + 2)?;
                }
                Ok(())
            }
        }
    }
}

impl DebugStatement {
    fn pretty_fmt(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let indentation = "  ".repeat(indent);
        match &self {
            DebugStatement::IfThenElse {
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
                (cond.clone()).pretty_fmt(f, indent + 2)?;
                writeln!(f, "{}  {}If Case:{}:", indentation, CYAN, RESET)?;
                (*if_case.clone()).pretty_fmt(f, indent + 2)?;
                if let Some(else_case) = else_case {
                    writeln!(f, "{}  {}Else Case:{}:", indentation, CYAN, RESET)?;
                    (*else_case.clone()).pretty_fmt(f, indent + 2)?;
                }
                Ok(())
            }
            DebugStatement::While { cond, stmt, meta } => {
                writeln!(
                    f,
                    "{}{}While{} (elem_id={}):",
                    indentation, GREEN, RESET, meta.elem_id
                )?;
                writeln!(f, "{}  {}Condition:{}:", indentation, CYAN, RESET)?;
                (cond.clone()).pretty_fmt(f, indent + 2)?;
                writeln!(f, "{}  {}Statement:{}:", indentation, CYAN, RESET)?;
                (*stmt.clone()).pretty_fmt(f, indent + 2)
            }
            DebugStatement::Return { value, meta, .. } => {
                writeln!(
                    f,
                    "{}{}Return{} (elem_id={}):",
                    indentation, GREEN, RESET, meta.elem_id
                )?;
                writeln!(f, "{}  {}Value:{}:", indentation, MAGENTA, RESET)?;
                (value.clone()).pretty_fmt(f, indent + 2)
            }
            DebugStatement::Substitution {
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
                writeln!(f, "{}  {}Operation:{} {:?}", indentation, CYAN, RESET, op)?;
                writeln!(
                    f,
                    "{}  {}Right-Hand Expression:{}:",
                    indentation, YELLOW, RESET
                )?;
                (rhe.clone()).pretty_fmt(f, indent + 2)
            }
            DebugStatement::Block { stmts, meta, .. } => {
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
                    (stmt.clone()).pretty_fmt(f, indent + 2)?;
                    writeln!(
                        f,
                        "{}    {}-------------------------------{}",
                        indentation, RED, RESET
                    )?;
                }
                Ok(())
            }
            DebugStatement::Assert { arg, meta, .. } => {
                writeln!(
                    f,
                    "{}{}Assert{} (elem_id={}):",
                    indentation, GREEN, RESET, meta.elem_id
                )?;
                writeln!(f, "{}  {}Argument:{}:", indentation, YELLOW, RESET)?;
                (arg.clone()).pretty_fmt(f, indent + 2)
            }
            DebugStatement::InitializationBlock {
                meta,
                xtype,
                initializations,
            } => {
                writeln!(
                    f,
                    "{}{}InitializationBlock{} (elem_id={}):",
                    indentation, GREEN, RESET, meta.elem_id
                )?;
                writeln!(
                    f,
                    "{}  {}Type:{} {:?}",
                    indentation,
                    CYAN,
                    RESET,
                    &DebugVariableType(xtype.clone())
                )?;
                writeln!(f, "{}  {}Initializations:{}", indentation, YELLOW, RESET)?;
                for i in initializations {
                    (i.clone()).pretty_fmt(f, indent + 2)?;
                }
                Ok(())
            }
            DebugStatement::Declaration {
                meta,
                xtype,
                name,
                dimensions,
                is_constant,
            } => {
                writeln!(
                    f,
                    "{}{}Declaration{} (elem_id={}):",
                    indentation, GREEN, RESET, meta.elem_id
                )?;
                writeln!(
                    f,
                    "{}  {}Type:{} {:?}",
                    indentation,
                    CYAN,
                    RESET,
                    &DebugVariableType(xtype.clone())
                )?;
                writeln!(f, "{}  {}Name:{} {}", indentation, MAGENTA, RESET, name)?;
                writeln!(f, "{}  {}Dimensions:{}:", indentation, YELLOW, RESET)?;
                for dim in dimensions {
                    (dim.clone()).pretty_fmt(f, indent + 2)?;
                }
                writeln!(
                    f,
                    "{}  {}Is Constant:{} {}",
                    indentation, CYAN, RESET, is_constant
                )
            }
            DebugStatement::MultSubstitution {
                lhe, op, rhe, meta, ..
            } => {
                writeln!(
                    f,
                    "{}{}MultSubstitution{} (elem_id={}):",
                    indentation, GREEN, RESET, meta.elem_id
                )?;
                writeln!(f, "{}  {}Op:{} {:?}", indentation, CYAN, RESET, op)?;
                writeln!(
                    f,
                    "{}  {}Left-Hand Expression:{}:",
                    indentation, YELLOW, RESET
                )?;
                (lhe.clone()).pretty_fmt(f, indent + 2)?;
                writeln!(
                    f,
                    "{}  {}Right-Hand Expression:{}:",
                    indentation, YELLOW, RESET
                )?;
                (rhe.clone()).pretty_fmt(f, indent + 2)
            }
            DebugStatement::UnderscoreSubstitution { op, rhe, meta, .. } => {
                writeln!(
                    f,
                    "{}{}UnderscoreSubstitution{} (elem_id={}):",
                    indentation, GREEN, RESET, meta.elem_id
                )?;
                writeln!(f, "{}  {}Op:{} {:?}", indentation, CYAN, RESET, op)?;
                writeln!(
                    f,
                    "{}  {}Right-Hand Expression:{}:",
                    indentation, YELLOW, RESET
                )?;
                (rhe.clone()).pretty_fmt(f, indent + 2)
            }
            DebugStatement::ConstraintEquality { lhe, rhe, meta, .. } => {
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
                (lhe.clone()).pretty_fmt(f, indent + 2)?;
                writeln!(
                    f,
                    "{}  {}Right-Hand Expression:{}:",
                    indentation, YELLOW, RESET
                )?;
                (rhe.clone()).pretty_fmt(f, indent + 2)
            }
            DebugStatement::LogCall { args: _, .. } => {
                writeln!(f, "{}{}LogCall{}", indentation, GREEN, RESET)
            }
            DebugStatement::Ret => writeln!(f, "{}{}Ret{}", indentation, BLUE, RESET),
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
