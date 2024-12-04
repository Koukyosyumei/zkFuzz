use num_bigint_dig::BigInt;
use std::collections::HashMap;
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
pub struct DebugAssignOp(pub AssignOp);
#[derive(Clone, PartialEq)]
pub struct DebugExpressionInfixOpcode(pub ExpressionInfixOpcode);
#[derive(Clone, PartialEq)]
pub struct DebugExpressionPrefixOpcode(pub ExpressionPrefixOpcode);

#[derive(Clone)]
pub enum DebugAccess {
    ComponentAccess(usize),
    ArrayAccess(DebugExpression),
}

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
        name: usize,
        access: Vec<DebugAccess>,
    },
    Number(Meta, BigInt),
    Call {
        meta: Meta,
        id: usize,
        args: Vec<DebugExpression>,
    },
    BusCall {
        meta: Meta,
        id: usize,
        args: Vec<DebugExpression>,
    },
    AnonymousComp {
        meta: Meta,
        id: usize,
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
        name: usize,
        dimensions: Vec<DebugExpression>,
        is_constant: bool,
    },
    Substitution {
        meta: Meta,
        var: usize,
        access: Vec<DebugAccess>,
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

impl DebugAccess {
    pub fn from(access: Access, name2id: &mut HashMap<String, usize>) -> Self {
        match access {
            Access::ComponentAccess(name) => {
                let i = if let Some(i) = name2id.get(&name) {
                    *i
                } else {
                    name2id.insert(name, name2id.len());
                    name2id.len() - 1
                };
                DebugAccess::ComponentAccess(i)
            }
            Access::ArrayAccess(expr) => {
                DebugAccess::ArrayAccess(DebugExpression::from(expr, name2id))
            }
        }
    }
}

impl DebugExpression {
    pub fn from(expr: Expression, name2id: &mut HashMap<String, usize>) -> Self {
        match expr {
            Expression::InfixOp {
                meta,
                lhe,
                infix_op,
                rhe,
            } => DebugExpression::InfixOp {
                meta,
                lhe: Box::new(DebugExpression::from(*lhe, name2id)),
                infix_op: DebugExpressionInfixOpcode(infix_op),
                rhe: Box::new(DebugExpression::from(*rhe, name2id)),
            },
            Expression::PrefixOp {
                meta,
                prefix_op,
                rhe,
            } => DebugExpression::PrefixOp {
                meta,
                prefix_op: DebugExpressionPrefixOpcode(prefix_op),
                rhe: Box::new(DebugExpression::from(*rhe, name2id)),
            },
            Expression::InlineSwitchOp {
                meta,
                cond,
                if_true,
                if_false,
            } => DebugExpression::InlineSwitchOp {
                meta,
                cond: Box::new(DebugExpression::from(*cond, name2id)),
                if_true: Box::new(DebugExpression::from(*if_true, name2id)),
                if_false: Box::new(DebugExpression::from(*if_false, name2id)),
            },
            Expression::ParallelOp { meta, rhe } => DebugExpression::ParallelOp {
                meta,
                rhe: Box::new(DebugExpression::from(*rhe, name2id)),
            },
            Expression::Variable { meta, name, access } => {
                let i = if let Some(i) = name2id.get(&name) {
                    *i
                } else {
                    name2id.insert(name, name2id.len());
                    name2id.len() - 1
                };
                DebugExpression::Variable {
                    meta: meta,
                    name: i,
                    access: access
                        .into_iter()
                        .map(|a| DebugAccess::from(a, name2id))
                        .collect(),
                }
            }
            Expression::Number(meta, value) => DebugExpression::Number(meta, value),
            Expression::Call { meta, id, args } => {
                let i = if let Some(i) = name2id.get(&id) {
                    *i
                } else {
                    name2id.insert(id, name2id.len());
                    name2id.len() - 1
                };
                DebugExpression::Call {
                    meta: meta,
                    id: i,
                    args: args
                        .into_iter()
                        .map(|arg| DebugExpression::from(arg, name2id))
                        .collect(),
                }
            }
            Expression::BusCall { meta, id, args } => {
                let i = if let Some(i) = name2id.get(&id) {
                    *i
                } else {
                    name2id.insert(id, name2id.len());
                    name2id.len() - 1
                };
                DebugExpression::BusCall {
                    meta: meta,
                    id: i,
                    args: args
                        .into_iter()
                        .map(|arg| DebugExpression::from(arg, name2id))
                        .collect(),
                }
            }
            Expression::AnonymousComp {
                meta,
                id,
                is_parallel,
                params,
                signals,
                names,
            } => {
                let i = if let Some(i) = name2id.get(&id) {
                    *i
                } else {
                    name2id.insert(id, name2id.len());
                    name2id.len() - 1
                };
                DebugExpression::AnonymousComp {
                    meta,
                    id: i,
                    is_parallel,
                    params: params
                        .into_iter()
                        .map(|p| DebugExpression::from(p, name2id))
                        .collect(),
                    signals: signals
                        .into_iter()
                        .map(|s| DebugExpression::from(s, name2id))
                        .collect(),
                    names,
                }
            }
            Expression::ArrayInLine { meta, values } => DebugExpression::ArrayInLine {
                meta,
                values: values
                    .into_iter()
                    .map(|v| DebugExpression::from(v, name2id))
                    .collect(),
            },
            Expression::Tuple { meta, values } => DebugExpression::Tuple {
                meta,
                values: values
                    .into_iter()
                    .map(|v| DebugExpression::from(v, name2id))
                    .collect(),
            },
            Expression::UniformArray {
                meta,
                value,
                dimension,
            } => DebugExpression::UniformArray {
                meta,
                value: Box::new(DebugExpression::from(*value, name2id)),
                dimension: Box::new(DebugExpression::from(*dimension, name2id)),
            },
        }
    }
}

impl DebugStatement {
    pub fn from(stmt: Statement, name2id: &mut HashMap<String, usize>) -> Self {
        match stmt {
            Statement::IfThenElse {
                meta,
                cond,
                if_case,
                else_case,
            } => DebugStatement::IfThenElse {
                meta,
                cond: DebugExpression::from(cond, name2id),
                if_case: Box::new(DebugStatement::from(*if_case, name2id)),
                else_case: else_case
                    .map(|else_case| Box::new(DebugStatement::from(*else_case, name2id))),
            },
            Statement::While { meta, cond, stmt } => DebugStatement::While {
                meta,
                cond: DebugExpression::from(cond, name2id),
                stmt: Box::new(DebugStatement::from(*stmt, name2id)),
            },
            Statement::Return { meta, value } => DebugStatement::Return {
                meta,
                value: DebugExpression::from(value, name2id),
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
                    .map(|stmt| DebugStatement::from(stmt, name2id))
                    .collect(),
            },
            Statement::Declaration {
                meta,
                xtype,
                name,
                dimensions,
                is_constant,
            } => {
                let i = if let Some(i) = name2id.get(&name) {
                    *i
                } else {
                    name2id.insert(name, name2id.len());
                    name2id.len() - 1
                };
                DebugStatement::Declaration {
                    meta: meta,
                    xtype: xtype,
                    name: i,
                    dimensions: dimensions
                        .into_iter()
                        .map(|dim| DebugExpression::from(dim, name2id))
                        .collect(),
                    is_constant: is_constant,
                }
            }
            Statement::Substitution {
                meta,
                var,
                access,
                op,
                rhe,
            } => {
                let i = if let Some(i) = name2id.get(&var) {
                    *i
                } else {
                    name2id.insert(var, name2id.len());
                    name2id.len() - 1
                };
                DebugStatement::Substitution {
                    meta,
                    var: i,
                    access: access
                        .into_iter()
                        .map(|a| DebugAccess::from(a, name2id))
                        .collect(),
                    op: DebugAssignOp(op),
                    rhe: DebugExpression::from(rhe, name2id),
                }
            }
            Statement::MultSubstitution { meta, lhe, op, rhe } => {
                DebugStatement::MultSubstitution {
                    meta,
                    lhe: DebugExpression::from(lhe, name2id),
                    op: DebugAssignOp(op),
                    rhe: DebugExpression::from(rhe, name2id),
                }
            }
            Statement::UnderscoreSubstitution { meta, op, rhe } => {
                DebugStatement::UnderscoreSubstitution {
                    meta,
                    op: DebugAssignOp(op),
                    rhe: DebugExpression::from(rhe, name2id),
                }
            }
            Statement::ConstraintEquality { meta, lhe, rhe } => {
                DebugStatement::ConstraintEquality {
                    meta,
                    lhe: DebugExpression::from(lhe, name2id),
                    rhe: DebugExpression::from(rhe, name2id),
                }
            }
            Statement::LogCall { meta, args } => DebugStatement::LogCall { meta, args },
            Statement::Block { meta, stmts } => DebugStatement::Block {
                meta,
                stmts: stmts
                    .into_iter()
                    .map(|stmt| DebugStatement::from(stmt, name2id))
                    .collect(),
            },
            Statement::Assert { meta, arg } => DebugStatement::Assert {
                meta,
                arg: DebugExpression::from(arg, name2id),
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
        match &self {
            DebugAccess::ComponentAccess(name) => {
                writeln!(f, "{}ComponentAccess", indentation)?;
                writeln!(f, "{}  name: {}", indentation, name)
            }
            DebugAccess::ArrayAccess(expr) => {
                writeln!(f, "{}ArrayAccess:", indentation)?;
                expr.pretty_fmt(f, indent + 2)
            }
        }
    }
}

impl DebugAccess {
    fn compact_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            DebugAccess::ComponentAccess(name) => {
                write!(f, ".{}", name)
            }
            DebugAccess::ArrayAccess(expr) => {
                write!(
                    f,
                    "[{}]",
                    format!("{:?}", expr).replace("\n", "").replace("  ", " ")
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
                    arg0.pretty_fmt(f, indent + 2)?;
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
                    arg0.pretty_fmt(f, indent + 2)?;
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
