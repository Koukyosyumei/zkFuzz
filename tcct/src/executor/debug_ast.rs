use num_bigint_dig::BigInt;
use rustc_hash::FxHashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

use program_structure::abstract_syntax_tree::ast::{
    Access, AssignOp, Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode, SignalType,
    Statement, VariableType,
};
use program_structure::ast::Meta;

const RESET: &str = "\x1b[0m";
const BLUE: &str = "\x1b[34m"; //94
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";

#[derive(Clone)]
pub struct DebuggableSignalType(pub SignalType);
#[derive(Clone)]
pub struct DebuggableVariableType(pub VariableType);
#[derive(Clone)]
pub struct DebuggableAssignOp(pub AssignOp);
#[derive(Clone, PartialEq)]
pub struct DebuggableExpressionInfixOpcode(pub ExpressionInfixOpcode);
#[derive(Clone, PartialEq)]
pub struct DebuggableExpressionPrefixOpcode(pub ExpressionPrefixOpcode);

#[derive(Clone)]
pub enum DebugAccess {
    ComponentAccess(usize),
    ArrayAccess(DebuggableExpression),
}

#[derive(Clone)]
pub enum DebuggableExpression {
    InfixOp {
        lhe: Box<DebuggableExpression>,
        infix_op: DebuggableExpressionInfixOpcode,
        rhe: Box<DebuggableExpression>,
    },
    PrefixOp {
        prefix_op: DebuggableExpressionPrefixOpcode,
        rhe: Box<DebuggableExpression>,
    },
    InlineSwitchOp {
        cond: Box<DebuggableExpression>,
        if_true: Box<DebuggableExpression>,
        if_false: Box<DebuggableExpression>,
    },
    ParallelOp {
        rhe: Box<DebuggableExpression>,
    },
    Variable {
        id: usize,
        access: Vec<DebugAccess>,
    },
    Number(BigInt),
    Call {
        id: usize,
        args: Vec<DebuggableExpression>,
    },
    BusCall {
        id: usize,
        args: Vec<DebuggableExpression>,
    },
    AnonymousComp {
        id: usize,
        is_parallel: bool,
        params: Vec<DebuggableExpression>,
        signals: Vec<DebuggableExpression>,
    },
    ArrayInLine {
        values: Vec<DebuggableExpression>,
    },
    Tuple {
        values: Vec<DebuggableExpression>,
    },
    UniformArray {
        value: Box<DebuggableExpression>,
        dimension: Box<DebuggableExpression>,
    },
}

#[derive(Clone)]
pub enum DebuggableStatement {
    IfThenElse {
        meta: Meta,
        cond: DebuggableExpression,
        if_case: Box<DebuggableStatement>,
        else_case: Option<Box<DebuggableStatement>>,
    },
    While {
        meta: Meta,
        cond: DebuggableExpression,
        stmt: Box<DebuggableStatement>,
    },
    Return {
        meta: Meta,
        value: DebuggableExpression,
    },
    InitializationBlock {
        meta: Meta,
        xtype: VariableType,
        initializations: Vec<DebuggableStatement>,
    },
    Declaration {
        meta: Meta,
        xtype: VariableType,
        id: usize,
        dimensions: Vec<DebuggableExpression>,
        is_constant: bool,
    },
    Substitution {
        meta: Meta,
        var: usize,
        access: Vec<DebugAccess>,
        op: DebuggableAssignOp,
        rhe: DebuggableExpression,
    },
    MultSubstitution {
        meta: Meta,
        lhe: DebuggableExpression,
        op: DebuggableAssignOp,
        rhe: DebuggableExpression,
    },
    UnderscoreSubstitution {
        meta: Meta,
        op: DebuggableAssignOp,
        rhe: DebuggableExpression,
    },
    ConstraintEquality {
        meta: Meta,
        lhe: DebuggableExpression,
        rhe: DebuggableExpression,
    },
    LogCall {
        meta: Meta,
    },
    Block {
        meta: Meta,
        stmts: Vec<DebuggableStatement>,
    },
    Assert {
        meta: Meta,
        arg: DebuggableExpression,
    },
    Ret,
}

impl DebugAccess {
    pub fn from(
        access: Access,
        name2id: &mut FxHashMap<String, usize>,
        id2name: &mut FxHashMap<usize, String>,
    ) -> Self {
        match access {
            Access::ComponentAccess(name) => {
                let i = if let Some(i) = name2id.get(&name) {
                    *i
                } else {
                    name2id.insert(name.clone(), name2id.len());
                    id2name.insert(name2id[&name], name);
                    name2id.len() - 1
                };
                DebugAccess::ComponentAccess(i)
            }
            Access::ArrayAccess(expr) => {
                DebugAccess::ArrayAccess(DebuggableExpression::from(expr, name2id, id2name))
            }
        }
    }
}

impl DebuggableExpression {
    pub fn from(
        expr: Expression,
        name2id: &mut FxHashMap<String, usize>,
        id2name: &mut FxHashMap<usize, String>,
    ) -> Self {
        match expr {
            Expression::InfixOp {
                meta: _,
                lhe,
                infix_op,
                rhe,
            } => DebuggableExpression::InfixOp {
                lhe: Box::new(DebuggableExpression::from(*lhe, name2id, id2name)),
                infix_op: DebuggableExpressionInfixOpcode(infix_op),
                rhe: Box::new(DebuggableExpression::from(*rhe, name2id, id2name)),
            },
            Expression::PrefixOp {
                meta: _,
                prefix_op,
                rhe,
            } => DebuggableExpression::PrefixOp {
                prefix_op: DebuggableExpressionPrefixOpcode(prefix_op),
                rhe: Box::new(DebuggableExpression::from(*rhe, name2id, id2name)),
            },
            Expression::InlineSwitchOp {
                meta: _,
                cond,
                if_true,
                if_false,
            } => DebuggableExpression::InlineSwitchOp {
                cond: Box::new(DebuggableExpression::from(*cond, name2id, id2name)),
                if_true: Box::new(DebuggableExpression::from(*if_true, name2id, id2name)),
                if_false: Box::new(DebuggableExpression::from(*if_false, name2id, id2name)),
            },
            Expression::ParallelOp { meta: _, rhe } => DebuggableExpression::ParallelOp {
                rhe: Box::new(DebuggableExpression::from(*rhe, name2id, id2name)),
            },
            Expression::Variable {
                meta: _,
                name,
                access,
            } => {
                let i = if let Some(i) = name2id.get(&name) {
                    *i
                } else {
                    name2id.insert(name.clone(), name2id.len());
                    id2name.insert(name2id[&name], name);
                    name2id.len() - 1
                };
                DebuggableExpression::Variable {
                    id: i,
                    access: access
                        .into_iter()
                        .map(|a| DebugAccess::from(a, name2id, id2name))
                        .collect(),
                }
            }
            Expression::Number(_, value) => DebuggableExpression::Number(value),
            Expression::Call { meta: _, id, args } => {
                let i = if let Some(i) = name2id.get(&id) {
                    *i
                } else {
                    name2id.insert(id.clone(), name2id.len());
                    id2name.insert(name2id[&id], id);
                    name2id.len() - 1
                };
                DebuggableExpression::Call {
                    id: i,
                    args: args
                        .into_iter()
                        .map(|arg| DebuggableExpression::from(arg, name2id, id2name))
                        .collect(),
                }
            }
            Expression::BusCall { meta: _, id, args } => {
                let i = if let Some(i) = name2id.get(&id) {
                    *i
                } else {
                    name2id.insert(id.clone(), name2id.len());
                    id2name.insert(name2id[&id], id);
                    name2id.len() - 1
                };
                DebuggableExpression::BusCall {
                    id: i,
                    args: args
                        .into_iter()
                        .map(|arg| DebuggableExpression::from(arg, name2id, id2name))
                        .collect(),
                }
            }
            Expression::AnonymousComp {
                meta: _,
                id,
                is_parallel,
                params,
                signals,
                names: _,
            } => {
                let i = if let Some(i) = name2id.get(&id) {
                    *i
                } else {
                    name2id.insert(id.clone(), name2id.len());
                    id2name.insert(name2id[&id], id);
                    name2id.len() - 1
                };
                DebuggableExpression::AnonymousComp {
                    id: i,
                    is_parallel,
                    params: params
                        .into_iter()
                        .map(|p| DebuggableExpression::from(p, name2id, id2name))
                        .collect(),
                    signals: signals
                        .into_iter()
                        .map(|s| DebuggableExpression::from(s, name2id, id2name))
                        .collect(),
                }
            }
            Expression::ArrayInLine { meta: _, values } => DebuggableExpression::ArrayInLine {
                values: values
                    .into_iter()
                    .map(|v| DebuggableExpression::from(v, name2id, id2name))
                    .collect(),
            },
            Expression::Tuple { meta: _, values } => DebuggableExpression::Tuple {
                values: values
                    .into_iter()
                    .map(|v| DebuggableExpression::from(v, name2id, id2name))
                    .collect(),
            },
            Expression::UniformArray {
                meta: _,
                value,
                dimension,
            } => DebuggableExpression::UniformArray {
                value: Box::new(DebuggableExpression::from(*value, name2id, id2name)),
                dimension: Box::new(DebuggableExpression::from(*dimension, name2id, id2name)),
            },
        }
    }
}

impl DebuggableStatement {
    pub fn from(
        stmt: Statement,
        name2id: &mut FxHashMap<String, usize>,
        id2name: &mut FxHashMap<usize, String>,
    ) -> Self {
        match stmt {
            Statement::IfThenElse {
                meta,
                cond,
                if_case,
                else_case,
            } => DebuggableStatement::IfThenElse {
                meta,
                cond: DebuggableExpression::from(cond, name2id, id2name),
                if_case: Box::new(DebuggableStatement::from(*if_case, name2id, id2name)),
                else_case: else_case.map(|else_case| {
                    Box::new(DebuggableStatement::from(*else_case, name2id, id2name))
                }),
            },
            Statement::While { meta, cond, stmt } => DebuggableStatement::While {
                meta,
                cond: DebuggableExpression::from(cond, name2id, id2name),
                stmt: Box::new(DebuggableStatement::from(*stmt, name2id, id2name)),
            },
            Statement::Return { meta, value } => DebuggableStatement::Return {
                meta,
                value: DebuggableExpression::from(value, name2id, id2name),
            },
            Statement::InitializationBlock {
                meta,
                xtype,
                initializations,
            } => DebuggableStatement::InitializationBlock {
                meta,
                xtype,
                initializations: initializations
                    .into_iter()
                    .map(|stmt| DebuggableStatement::from(stmt, name2id, id2name))
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
                    name2id.insert(name.clone(), name2id.len());
                    id2name.insert(name2id[&name], name);
                    name2id.len() - 1
                };
                DebuggableStatement::Declaration {
                    meta: meta,
                    xtype: xtype,
                    id: i,
                    dimensions: dimensions
                        .into_iter()
                        .map(|dim| DebuggableExpression::from(dim, name2id, id2name))
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
                    name2id.insert(var.clone(), name2id.len());
                    id2name.insert(name2id[&var], var);
                    name2id.len() - 1
                };
                DebuggableStatement::Substitution {
                    meta,
                    var: i,
                    access: access
                        .into_iter()
                        .map(|a| DebugAccess::from(a, name2id, id2name))
                        .collect(),
                    op: DebuggableAssignOp(op),
                    rhe: DebuggableExpression::from(rhe, name2id, id2name),
                }
            }
            Statement::MultSubstitution { meta, lhe, op, rhe } => {
                DebuggableStatement::MultSubstitution {
                    meta,
                    lhe: DebuggableExpression::from(lhe, name2id, id2name),
                    op: DebuggableAssignOp(op),
                    rhe: DebuggableExpression::from(rhe, name2id, id2name),
                }
            }
            Statement::UnderscoreSubstitution { meta, op, rhe } => {
                DebuggableStatement::UnderscoreSubstitution {
                    meta,
                    op: DebuggableAssignOp(op),
                    rhe: DebuggableExpression::from(rhe, name2id, id2name),
                }
            }
            Statement::ConstraintEquality { meta, lhe, rhe } => {
                DebuggableStatement::ConstraintEquality {
                    meta,
                    lhe: DebuggableExpression::from(lhe, name2id, id2name),
                    rhe: DebuggableExpression::from(rhe, name2id, id2name),
                }
            }
            Statement::LogCall { meta, args: _ } => DebuggableStatement::LogCall { meta },
            Statement::Block { meta, stmts } => DebuggableStatement::Block {
                meta,
                stmts: stmts
                    .into_iter()
                    .map(|stmt| DebuggableStatement::from(stmt, name2id, id2name))
                    .collect(),
            },
            Statement::Assert { meta, arg } => DebuggableStatement::Assert {
                meta,
                arg: DebuggableExpression::from(arg, name2id, id2name),
            },
        }
    }
}

impl Hash for DebuggableExpressionInfixOpcode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(&self.0).hash(state);
    }
}

impl Eq for DebuggableExpressionInfixOpcode {}

impl Hash for DebuggableExpressionPrefixOpcode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(&self.0).hash(state);
    }
}

impl Eq for DebuggableExpressionPrefixOpcode {}

impl fmt::Debug for DebuggableSignalType {
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

impl fmt::Debug for DebuggableVariableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            VariableType::Var => {
                write!(f, "Var")
            }
            VariableType::Signal(signaltype, taglist) => {
                write!(
                    f,
                    "Signal: {:?} {:?}",
                    &DebuggableSignalType(*signaltype),
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
                    &DebuggableSignalType(*signaltype),
                    &taglist
                )
            }
        }
    }
}

impl DebugAccess {
    pub fn lookup_fmt(&self, lookup: &FxHashMap<usize, String>, indent: usize) -> String {
        let mut s = "".to_string();
        let indentation = "  ".repeat(indent);
        match &self {
            DebugAccess::ComponentAccess(name) => {
                s += &format!("{}ComponentAccess\n", indentation);
                s += &format!("{}  name: {}\n", indentation, lookup[name]);
            }
            DebugAccess::ArrayAccess(expr) => {
                s += &format!("{}ArrayAccess:\n", indentation);
                s += &expr.lookup_fmt(lookup, indent + 2);
            }
        }
        s
    }
}

impl fmt::Debug for DebuggableAssignOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            AssignOp::AssignVar => f.debug_struct("AssignVar").finish(),
            AssignOp::AssignSignal => f.debug_struct("AssignSignal").finish(),
            AssignOp::AssignConstraintSignal => f.debug_struct("AssignConstraintSignal").finish(),
        }
    }
}

impl fmt::Debug for DebuggableExpressionInfixOpcode {
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

impl fmt::Debug for DebuggableExpressionPrefixOpcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ExpressionPrefixOpcode::Sub => f.debug_struct("Minus").finish(),
            ExpressionPrefixOpcode::BoolNot => f.debug_struct("BoolNot").finish(),
            ExpressionPrefixOpcode::Complement => f.debug_struct("Complement").finish(),
        }
    }
}

impl DebuggableExpression {
    pub fn lookup_fmt(&self, lookup: &FxHashMap<usize, String>, indent: usize) -> String {
        let mut s = "".to_string();
        let indentation = "  ".repeat(indent);
        match &self {
            DebuggableExpression::Number(value) => {
                format!("{}{}Number:{} {}\n", indentation, BLUE, RESET, value)
            }
            DebuggableExpression::InfixOp {
                lhe, infix_op, rhe, ..
            } => {
                s += &format!("{}{}InfixOp:{}\n", indentation, GREEN, RESET);
                s += &format!(
                    "{}  {}Operator:{} {:?}\n",
                    indentation, CYAN, RESET, infix_op
                );
                s += &format!(
                    "{}  {}Left-Hand Expression:{}\n",
                    indentation, YELLOW, RESET
                );
                s += &(*lhe.clone()).lookup_fmt(lookup, indent + 2);
                s += &format!(
                    "{}  {}Right-Hand Expression:{}\n",
                    indentation, YELLOW, RESET
                );
                s += &(*rhe.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebuggableExpression::PrefixOp { prefix_op, rhe, .. } => {
                s += &format!("{}{}PrefixOp:{}\n", indentation, GREEN, RESET);
                s += &format!(
                    "{}  {}Operator:{} {:?}\n",
                    indentation, CYAN, RESET, prefix_op
                );
                s += &format!(
                    "{}  {}Right-Hand Expression:{}\n",
                    indentation, YELLOW, RESET
                );
                s += &(*rhe.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebuggableExpression::ParallelOp { rhe, .. } => {
                s += &format!("{}ParallelOp\n", indentation);
                s += &format!(
                    "{}  {}Right-Hand Expression:{}\n",
                    indentation, YELLOW, RESET
                );
                s += &(*rhe.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebuggableExpression::Variable { id, access, .. } => {
                s += &format!("{}{}Variable:{}\n", indentation, BLUE, RESET);
                s += &format!("{}  Name: {}\n", indentation, lookup[id]);
                s += &format!("{}  Access:\n", indentation);
                for arg0 in access {
                    s += &arg0.lookup_fmt(lookup, indent + 2);
                }
                s
            }
            DebuggableExpression::InlineSwitchOp {
                cond: _,
                if_true,
                if_false,
                ..
            } => {
                s += &format!("{}InlineSwitchOp:\n", indentation);
                s += &format!("{}  if_true:\n", indentation);
                s += &(*if_true.clone()).lookup_fmt(lookup, indent + 2);
                s += &format!("{}  if_false:\n", indentation);
                s += &(*if_false.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebuggableExpression::Call { id, args, .. } => {
                s += &format!("{}Call\n", indentation);
                s += &format!("{}  id: {}\n", indentation, lookup[id]);
                s += &format!("{}  args:\n", indentation);
                for arg0 in args {
                    s += &(arg0.clone()).lookup_fmt(lookup, indent + 2);
                }
                s
            }
            DebuggableExpression::ArrayInLine { values, .. } => {
                s += &format!("{}ArrayInLine\n", indentation);
                s += &format!("{}  values:\n", indentation);
                for v in values {
                    s += &(v.clone()).lookup_fmt(lookup, indent + 2);
                }
                s
            }
            DebuggableExpression::Tuple { values, .. } => {
                s += &format!("{}Tuple\n", indentation);
                s += &format!("{}  values:\n", indentation);
                for v in values {
                    s += &(v.clone()).lookup_fmt(lookup, indent + 2);
                }
                s
            }
            DebuggableExpression::UniformArray {
                value, dimension, ..
            } => {
                s += &format!("{}UniformArray\n", indentation);
                s += &format!("{}  value:\n", indentation);
                s += &(*value.clone()).lookup_fmt(lookup, indent + 2);
                s += &format!("{}  dimension:\n", indentation);
                s += &(*dimension.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebuggableExpression::BusCall { id, args, .. } => {
                s += &format!("{}BusCall\n", indentation);
                s += &format!("{}  id:\n", id);
                s += &format!("{}  args:\n", indentation);
                for a in args {
                    s += &(a.clone()).lookup_fmt(lookup, indent + 2);
                }
                s
            }
            DebuggableExpression::AnonymousComp {
                id,
                is_parallel,
                params,
                signals,
                ..
            } => {
                s += &format!("{}AnonymousComp\n", indentation);
                s += &format!("{}  id: {}\n", indentation, id);
                //format!("{}  name: {}", indentation, names);
                s += &format!("{}  is_parallel: {}\n", indentation, is_parallel);
                s += &format!("{}  params:\n", indentation);
                for p in params {
                    s += &(p.clone()).lookup_fmt(lookup, indent + 2);
                }
                s += &format!("{}  signals:\n", indentation);
                for sig in signals {
                    s += &(sig.clone()).lookup_fmt(lookup, indent + 2);
                }
                s
            }
        }
    }
}

impl DebuggableStatement {
    pub fn apply_iterative<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut DebuggableStatement),
    {
        let mut stack = vec![self]; // Stack to store DebuggableStatements for traversal

        while let Some(current) = stack.pop() {
            func(current); // Apply the function to the current statement

            // Push child nodes onto the stack for further processing
            match current {
                DebuggableStatement::IfThenElse {
                    if_case, else_case, ..
                } => {
                    stack.push(if_case);
                    if let Some(else_case) = else_case {
                        stack.push(else_case);
                    }
                }
                DebuggableStatement::While { stmt, .. } => {
                    stack.push(stmt);
                }
                DebuggableStatement::InitializationBlock {
                    initializations, ..
                } => {
                    for init in initializations.iter_mut() {
                        stack.push(init);
                    }
                }
                DebuggableStatement::Block { stmts, .. } => {
                    for stmt in stmts.iter_mut() {
                        stack.push(stmt);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn lookup_fmt(&self, lookup: &FxHashMap<usize, String>, indent: usize) -> String {
        let mut s = "".to_string();
        let indentation = "  ".repeat(indent);
        match &self {
            DebuggableStatement::IfThenElse {
                cond,
                if_case,
                else_case,
                meta,
                ..
            } => {
                s += &format!(
                    "{}{}IfThenElse{} (elem_id={}):\n",
                    indentation, GREEN, RESET, meta.elem_id
                );
                s += &format!("{}  {}Condition:{}:\n", indentation, CYAN, RESET);
                (cond.clone()).lookup_fmt(lookup, indent + 2);
                s += &format!("{}  {}If Case:{}:\n", indentation, CYAN, RESET);
                (*if_case.clone()).lookup_fmt(lookup, indent + 2);
                if let Some(else_case) = else_case {
                    s += &format!("{}  {}Else Case:{}:\n", indentation, CYAN, RESET);
                    s += &(*else_case.clone()).lookup_fmt(lookup, indent + 2);
                }
                s
            }
            DebuggableStatement::While { cond, stmt, meta } => {
                s += &format!(
                    "{}{}While{} (elem_id={}):\n",
                    indentation, GREEN, RESET, meta.elem_id
                );
                s += &format!("{}  {}Condition:{}:\n", indentation, CYAN, RESET);
                (cond.clone()).lookup_fmt(lookup, indent + 2);
                s += &format!("{}  {}Statement:{}:\n", indentation, CYAN, RESET);
                s += &(*stmt.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebuggableStatement::Return { value, meta, .. } => {
                s += &format!(
                    "{}{}Return{} (elem_id={}):\n",
                    indentation, GREEN, RESET, meta.elem_id
                );
                s += &format!("{}  {}Value:{}:\n", indentation, MAGENTA, RESET);
                s += &(value.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebuggableStatement::Substitution {
                var,
                access,
                op,
                rhe,
                meta,
                ..
            } => {
                s += &format!(
                    "{}{}Substitution{} (elem_id={}):\n",
                    indentation, GREEN, RESET, meta.elem_id
                );
                s += &format!(
                    "{}  {}Variable:{} {}\n",
                    indentation, BLUE, RESET, lookup[var]
                );
                s += &format!("{}  {}Access:{}\n", indentation, MAGENTA, RESET);
                for arg0 in access {
                    s += &arg0.lookup_fmt(lookup, indent + 2);
                }
                s += &format!("{}  {}Operation:{} {:?}\n", indentation, CYAN, RESET, op);
                s += &format!(
                    "{}  {}Right-Hand Expression:{}:\n",
                    indentation, YELLOW, RESET
                );
                s += &(rhe.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebuggableStatement::Block { stmts, meta, .. } => {
                s += &format!(
                    "{}{}Block{} (elem_id={}):\n",
                    indentation, GREEN, RESET, meta.elem_id
                );
                s += &format!(
                    "{}    {}-------------------------------{}\n",
                    indentation, RED, RESET
                );
                for stmt in stmts {
                    s += &(stmt.clone()).lookup_fmt(lookup, indent + 2);
                    s += &format!(
                        "{}    {}-------------------------------{}\n",
                        indentation, RED, RESET
                    );
                }
                s
            }
            DebuggableStatement::Assert { arg, meta, .. } => {
                s += &format!(
                    "{}{}Assert{} (elem_id={}):\n",
                    indentation, GREEN, RESET, meta.elem_id
                );
                s += &format!("{}  {}Argument:{}:\n", indentation, YELLOW, RESET);
                s += &(arg.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebuggableStatement::InitializationBlock {
                meta,
                xtype,
                initializations,
            } => {
                s += &format!(
                    "{}{}InitializationBlock{} (elem_id={}):\n",
                    indentation, GREEN, RESET, meta.elem_id
                );
                s += &format!(
                    "{}  {}Type:{} {:?}\n",
                    indentation,
                    CYAN,
                    RESET,
                    &DebuggableVariableType(xtype.clone())
                );
                s += &format!("{}  {}Initializations:{}\n", indentation, YELLOW, RESET);
                for i in initializations {
                    s += &(i.clone()).lookup_fmt(lookup, indent + 2);
                }
                s
            }
            DebuggableStatement::Declaration {
                meta,
                xtype,
                id,
                dimensions,
                is_constant,
            } => {
                s += &format!(
                    "{}{}Declaration{} (elem_id={}):\n",
                    indentation, GREEN, RESET, meta.elem_id
                );
                s += &format!(
                    "{}  {}Type:{} {:?}\n",
                    indentation,
                    CYAN,
                    RESET,
                    &DebuggableVariableType(xtype.clone())
                );
                s += &format!(
                    "{}  {}Name:{} {}\n",
                    indentation, MAGENTA, RESET, lookup[id]
                );
                s += &format!("{}  {}Dimensions:{}:\n", indentation, YELLOW, RESET);
                for dim in dimensions {
                    s += &(dim.clone()).lookup_fmt(lookup, indent + 2);
                }
                s += &format!(
                    "{}  {}Is Constant:{} {}\n",
                    indentation, CYAN, RESET, is_constant
                );
                s
            }
            DebuggableStatement::MultSubstitution {
                lhe, op, rhe, meta, ..
            } => {
                s += &format!(
                    "{}{}MultSubstitution{} (elem_id={}):\n",
                    indentation, GREEN, RESET, meta.elem_id
                );
                s += &format!("{}  {}Op:{} {:?}\n", indentation, CYAN, RESET, op);
                s += &format!(
                    "{}  {}Left-Hand Expression:{}:\n",
                    indentation, YELLOW, RESET
                );
                s += &(lhe.clone()).lookup_fmt(lookup, indent + 2);
                s += &format!(
                    "{}  {}Right-Hand Expression:{}:\n",
                    indentation, YELLOW, RESET
                );
                s += &(rhe.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebuggableStatement::UnderscoreSubstitution { op, rhe, meta, .. } => {
                s += &format!(
                    "{}{}UnderscoreSubstitution{} (elem_id={}):\n",
                    indentation, GREEN, RESET, meta.elem_id
                );
                s += &format!("{}  {}Op:{} {:?}\n", indentation, CYAN, RESET, op);
                s += &format!(
                    "{}  {}Right-Hand Expression:{}:\n",
                    indentation, YELLOW, RESET
                );
                s += &(rhe.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebuggableStatement::ConstraintEquality { lhe, rhe, meta, .. } => {
                s += &format!(
                    "{}{}ConstraintEquality{} (elem_id={}):\n",
                    indentation, GREEN, RESET, meta.elem_id
                );
                s += &format!(
                    "{}  {}Left-Hand Expression:{}:\n",
                    indentation, YELLOW, RESET
                );
                (lhe.clone()).lookup_fmt(lookup, indent + 2);
                s += &format!(
                    "{}  {}Right-Hand Expression:{}:\n",
                    indentation, YELLOW, RESET
                );
                s += &(rhe.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebuggableStatement::LogCall { .. } => {
                format!("{}{}LogCall{}\n", indentation, GREEN, RESET)
            }
            DebuggableStatement::Ret => format!("{}{}Ret{}\n", indentation, BLUE, RESET),
        }
    }
}
