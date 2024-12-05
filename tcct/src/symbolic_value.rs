use colored::Colorize;
use num_bigint_dig::BigInt;
use rustc_hash::FxHashMap;
use std::collections::HashSet;
use std::rc::Rc;

use program_structure::ast::{
    ExpressionInfixOpcode,
    VariableType,
};

use crate::debug_ast::{
    DebugExpressionInfixOpcode,
    DebugExpressionPrefixOpcode, DebugStatement,
};

/// Represents the access type within a symbolic expression, such as component or array access.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum SymbolicAccess {
    ComponentAccess(usize),
    ArrayAccess(SymbolicValue),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct OwnerName {
    pub name: usize,
    pub counter: usize,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SymbolicName {
    pub name: usize,
    pub owner: Rc<Vec<OwnerName>>,
    pub access: Vec<SymbolicAccess>,
}

/// Represents a symbolic value used in symbolic execution, which can be a constant, variable, or an operation.
/// It supports various operations like binary, unary, conditional, arrays, tuples, uniform arrays, and function calls.
#[derive(Clone, Hash, Eq, PartialEq)]
pub enum SymbolicValue {
    ConstantInt(BigInt),
    ConstantBool(bool),
    Variable(SymbolicName),
    BinaryOp(
        Rc<SymbolicValue>,
        DebugExpressionInfixOpcode,
        Rc<SymbolicValue>,
    ),
    Conditional(Rc<SymbolicValue>, Rc<SymbolicValue>, Rc<SymbolicValue>),
    UnaryOp(DebugExpressionPrefixOpcode, Rc<SymbolicValue>),
    Array(Vec<Rc<SymbolicValue>>),
    Tuple(Vec<Rc<SymbolicValue>>),
    UniformArray(Rc<SymbolicValue>, Rc<SymbolicValue>),
    Call(usize, Vec<Rc<SymbolicValue>>),
}

/// Represents a symbolic template used in the symbolic execution process.
#[derive(Default, Clone)]
pub struct SymbolicTemplate {
    pub template_parameter_names: Vec<usize>,
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
    pub unrolled_outputs: HashSet<SymbolicName>,
    pub var2type: FxHashMap<usize, VariableType>,
    pub body: Vec<DebugStatement>,
}

/// Represents a symbolic function used in the symbolic execution process.
#[derive(Default, Clone)]
pub struct SymbolicFunction {
    pub function_argument_names: Vec<usize>,
    pub body: Vec<DebugStatement>,
}

/// Represents a symbolic component used in the symbolic execution process.
#[derive(Default, Clone)]
pub struct SymbolicComponent {
    pub template_name: usize,
    pub args: Vec<Rc<SymbolicValue>>,
    pub inputs: FxHashMap<usize, Option<SymbolicValue>>,
    pub is_done: bool,
}

impl SymbolicAccess {
    /// Provides a compact format for displaying symbolic access in expressions.
    pub fn lookup_fmt(&self, lookup: &FxHashMap<usize, String>) -> String {
        match &self {
            SymbolicAccess::ComponentAccess(name) => {
                format!(".{}", lookup[name])
            }
            SymbolicAccess::ArrayAccess(val) => {
                format!(
                    "[{}]",
                    val.lookup_fmt(lookup).replace("\n", "").replace("  ", " ")
                )
            }
        }
    }
}

impl SymbolicName {
    pub fn lookup_fmt(&self, lookup: &FxHashMap<usize, String>) -> String {
        format!(
            "{}.{}{}",
            self.owner
                .iter()
                .map(|e: &OwnerName| lookup[&e.name].clone())
                .collect::<Vec<_>>()
                .join("."),
            lookup[&self.name].clone(),
            self.access
                .iter()
                .map(|s: &SymbolicAccess| s.lookup_fmt(lookup))
                .collect::<Vec<_>>()
                .join("")
        )
    }
}

/// Implements the `Debug` trait for `SymbolicValue` to provide custom formatting for debugging purposes.
impl SymbolicValue {
    pub fn lookup_fmt(&self, lookup: &FxHashMap<usize, String>) -> String {
        match self {
            SymbolicValue::ConstantInt(value) => format!("{}", value),
            SymbolicValue::ConstantBool(flag) => {
                format!("{} {}", if *flag { "✅" } else { "❌" }, flag)
            }
            SymbolicValue::Variable(sname) => sname.lookup_fmt(lookup),
            SymbolicValue::BinaryOp(lhs, op, rhs) => match &op.0 {
                ExpressionInfixOpcode::Eq
                | ExpressionInfixOpcode::NotEq
                | ExpressionInfixOpcode::LesserEq
                | ExpressionInfixOpcode::GreaterEq
                | ExpressionInfixOpcode::Lesser
                | ExpressionInfixOpcode::Greater => {
                    format!(
                        "({} {} {})",
                        format!("{:?}", op).green(),
                        lhs.lookup_fmt(lookup),
                        rhs.lookup_fmt(lookup)
                    )
                }
                ExpressionInfixOpcode::ShiftL
                | ExpressionInfixOpcode::ShiftR
                | ExpressionInfixOpcode::BitAnd
                | ExpressionInfixOpcode::BitOr
                | ExpressionInfixOpcode::BitXor
                | ExpressionInfixOpcode::Div
                | ExpressionInfixOpcode::IntDiv => {
                    format!(
                        "({} {} {})",
                        format!("{:?}", op).red(),
                        lhs.lookup_fmt(lookup),
                        rhs.lookup_fmt(lookup)
                    )
                }
                _ => format!(
                    "({} {} {})",
                    format!("{:?}", op).yellow(),
                    lhs.lookup_fmt(lookup),
                    rhs.lookup_fmt(lookup)
                ),
            },
            SymbolicValue::Conditional(cond, if_branch, else_branch) => {
                format!(
                    "({} {} {})",
                    cond.lookup_fmt(lookup),
                    if_branch.lookup_fmt(lookup),
                    else_branch.lookup_fmt(lookup)
                )
            }
            SymbolicValue::UnaryOp(op, expr) => match &op.0 {
                _ => format!(
                    "({} {})",
                    format!("{:?}", op).magenta(),
                    expr.lookup_fmt(lookup)
                ),
            },
            SymbolicValue::Call(name, args) => {
                format!(
                    "📞{}({})",
                    lookup[&name],
                    args.into_iter()
                        .map(|a| a.lookup_fmt(lookup))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            SymbolicValue::Array(elems) => {
                format!(
                    "🧬 {}",
                    elems
                        .into_iter()
                        .map(|a| a.lookup_fmt(lookup))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            SymbolicValue::UniformArray(elem, counts) => {
                format!(
                    "🧬 ({}, {})",
                    elem.lookup_fmt(lookup),
                    counts.lookup_fmt(lookup)
                )
            }
            _ => format!("❓Unknown symbolic value"),
        }
    }
}
