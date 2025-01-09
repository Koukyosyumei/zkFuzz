use num_bigint_dig::BigInt;
use rustc_hash::FxHashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

use program_structure::abstract_syntax_tree::ast::{
    Access, AssignOp, Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode, SignalType,
    Statement, VariableType,
};
use program_structure::ast::LogArgument;
use program_structure::ast::Meta;

const RESET: &str = "\x1b[0m";
const BLUE: &str = "\x1b[34m"; //94
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";

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
        lhe: Box<DebugExpression>,
        infix_op: DebugExpressionInfixOpcode,
        rhe: Box<DebugExpression>,
    },
    PrefixOp {
        prefix_op: DebugExpressionPrefixOpcode,
        rhe: Box<DebugExpression>,
    },
    InlineSwitchOp {
        cond: Box<DebugExpression>,
        if_true: Box<DebugExpression>,
        if_false: Box<DebugExpression>,
    },
    ParallelOp {
        rhe: Box<DebugExpression>,
    },
    Variable {
        id: usize,
        access: Vec<DebugAccess>,
    },
    Number(BigInt),
    Call {
        id: usize,
        args: Vec<DebugExpression>,
    },
    BusCall {
        id: usize,
        args: Vec<DebugExpression>,
    },
    AnonymousComp {
        id: usize,
        is_parallel: bool,
        params: Vec<DebugExpression>,
        signals: Vec<DebugExpression>,
        names: Option<Vec<(AssignOp, String)>>,
    },
    ArrayInLine {
        values: Vec<DebugExpression>,
    },
    Tuple {
        values: Vec<DebugExpression>,
    },
    UniformArray {
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
        id: usize,
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
                DebugAccess::ArrayAccess(DebugExpression::from(expr, name2id, id2name))
            }
        }
    }
}

impl DebugExpression {
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
            } => DebugExpression::InfixOp {
                lhe: Box::new(DebugExpression::from(*lhe, name2id, id2name)),
                infix_op: DebugExpressionInfixOpcode(infix_op),
                rhe: Box::new(DebugExpression::from(*rhe, name2id, id2name)),
            },
            Expression::PrefixOp {
                meta: _,
                prefix_op,
                rhe,
            } => DebugExpression::PrefixOp {
                prefix_op: DebugExpressionPrefixOpcode(prefix_op),
                rhe: Box::new(DebugExpression::from(*rhe, name2id, id2name)),
            },
            Expression::InlineSwitchOp {
                meta: _,
                cond,
                if_true,
                if_false,
            } => DebugExpression::InlineSwitchOp {
                cond: Box::new(DebugExpression::from(*cond, name2id, id2name)),
                if_true: Box::new(DebugExpression::from(*if_true, name2id, id2name)),
                if_false: Box::new(DebugExpression::from(*if_false, name2id, id2name)),
            },
            Expression::ParallelOp { meta: _, rhe } => DebugExpression::ParallelOp {
                rhe: Box::new(DebugExpression::from(*rhe, name2id, id2name)),
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
                DebugExpression::Variable {
                    id: i,
                    access: access
                        .into_iter()
                        .map(|a| DebugAccess::from(a, name2id, id2name))
                        .collect(),
                }
            }
            Expression::Number(_, value) => DebugExpression::Number(value),
            Expression::Call { meta: _, id, args } => {
                let i = if let Some(i) = name2id.get(&id) {
                    *i
                } else {
                    name2id.insert(id.clone(), name2id.len());
                    id2name.insert(name2id[&id], id);
                    name2id.len() - 1
                };
                DebugExpression::Call {
                    id: i,
                    args: args
                        .into_iter()
                        .map(|arg| DebugExpression::from(arg, name2id, id2name))
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
                DebugExpression::BusCall {
                    id: i,
                    args: args
                        .into_iter()
                        .map(|arg| DebugExpression::from(arg, name2id, id2name))
                        .collect(),
                }
            }
            Expression::AnonymousComp {
                meta: _,
                id,
                is_parallel,
                params,
                signals,
                names,
            } => {
                let i = if let Some(i) = name2id.get(&id) {
                    *i
                } else {
                    name2id.insert(id.clone(), name2id.len());
                    id2name.insert(name2id[&id], id);
                    name2id.len() - 1
                };
                DebugExpression::AnonymousComp {
                    id: i,
                    is_parallel,
                    params: params
                        .into_iter()
                        .map(|p| DebugExpression::from(p, name2id, id2name))
                        .collect(),
                    signals: signals
                        .into_iter()
                        .map(|s| DebugExpression::from(s, name2id, id2name))
                        .collect(),
                    names,
                }
            }
            Expression::ArrayInLine { meta: _, values } => DebugExpression::ArrayInLine {
                values: values
                    .into_iter()
                    .map(|v| DebugExpression::from(v, name2id, id2name))
                    .collect(),
            },
            Expression::Tuple { meta: _, values } => DebugExpression::Tuple {
                values: values
                    .into_iter()
                    .map(|v| DebugExpression::from(v, name2id, id2name))
                    .collect(),
            },
            Expression::UniformArray {
                meta: _,
                value,
                dimension,
            } => DebugExpression::UniformArray {
                value: Box::new(DebugExpression::from(*value, name2id, id2name)),
                dimension: Box::new(DebugExpression::from(*dimension, name2id, id2name)),
            },
        }
    }
}

impl DebugStatement {
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
            } => DebugStatement::IfThenElse {
                meta,
                cond: DebugExpression::from(cond, name2id, id2name),
                if_case: Box::new(DebugStatement::from(*if_case, name2id, id2name)),
                else_case: else_case
                    .map(|else_case| Box::new(DebugStatement::from(*else_case, name2id, id2name))),
            },
            Statement::While { meta, cond, stmt } => DebugStatement::While {
                meta,
                cond: DebugExpression::from(cond, name2id, id2name),
                stmt: Box::new(DebugStatement::from(*stmt, name2id, id2name)),
            },
            Statement::Return { meta, value } => DebugStatement::Return {
                meta,
                value: DebugExpression::from(value, name2id, id2name),
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
                    .map(|stmt| DebugStatement::from(stmt, name2id, id2name))
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
                DebugStatement::Declaration {
                    meta: meta,
                    xtype: xtype,
                    id: i,
                    dimensions: dimensions
                        .into_iter()
                        .map(|dim| DebugExpression::from(dim, name2id, id2name))
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
                DebugStatement::Substitution {
                    meta,
                    var: i,
                    access: access
                        .into_iter()
                        .map(|a| DebugAccess::from(a, name2id, id2name))
                        .collect(),
                    op: DebugAssignOp(op),
                    rhe: DebugExpression::from(rhe, name2id, id2name),
                }
            }
            Statement::MultSubstitution { meta, lhe, op, rhe } => {
                DebugStatement::MultSubstitution {
                    meta,
                    lhe: DebugExpression::from(lhe, name2id, id2name),
                    op: DebugAssignOp(op),
                    rhe: DebugExpression::from(rhe, name2id, id2name),
                }
            }
            Statement::UnderscoreSubstitution { meta, op, rhe } => {
                DebugStatement::UnderscoreSubstitution {
                    meta,
                    op: DebugAssignOp(op),
                    rhe: DebugExpression::from(rhe, name2id, id2name),
                }
            }
            Statement::ConstraintEquality { meta, lhe, rhe } => {
                DebugStatement::ConstraintEquality {
                    meta,
                    lhe: DebugExpression::from(lhe, name2id, id2name),
                    rhe: DebugExpression::from(rhe, name2id, id2name),
                }
            }
            Statement::LogCall { meta, args } => DebugStatement::LogCall { meta, args },
            Statement::Block { meta, stmts } => DebugStatement::Block {
                meta,
                stmts: stmts
                    .into_iter()
                    .map(|stmt| DebugStatement::from(stmt, name2id, id2name))
                    .collect(),
            },
            Statement::Assert { meta, arg } => DebugStatement::Assert {
                meta,
                arg: DebugExpression::from(arg, name2id, id2name),
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

impl DebugExpression {
    pub fn lookup_fmt(&self, lookup: &FxHashMap<usize, String>, indent: usize) -> String {
        let mut s = "".to_string();
        let indentation = "  ".repeat(indent);
        match &self {
            DebugExpression::Number(value) => {
                format!("{}{}Number:{} {}\n", indentation, BLUE, RESET, value)
            }
            DebugExpression::InfixOp {
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
            DebugExpression::PrefixOp { prefix_op, rhe, .. } => {
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
            DebugExpression::ParallelOp { rhe, .. } => {
                s += &format!("{}ParallelOp\n", indentation);
                s += &format!(
                    "{}  {}Right-Hand Expression:{}\n",
                    indentation, YELLOW, RESET
                );
                s += &(*rhe.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebugExpression::Variable { id, access, .. } => {
                s += &format!("{}{}Variable:{}\n", indentation, BLUE, RESET);
                s += &format!("{}  Name: {}\n", indentation, lookup[id]);
                s += &format!("{}  Access:\n", indentation);
                for arg0 in access {
                    s += &arg0.lookup_fmt(lookup, indent + 2);
                }
                s
            }
            DebugExpression::InlineSwitchOp {
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
            DebugExpression::Call { id, args, .. } => {
                s += &format!("{}Call\n", indentation);
                s += &format!("{}  id: {}\n", indentation, lookup[id]);
                s += &format!("{}  args:\n", indentation);
                for arg0 in args {
                    s += &(arg0.clone()).lookup_fmt(lookup, indent + 2);
                }
                s
            }
            DebugExpression::ArrayInLine { values, .. } => {
                s += &format!("{}ArrayInLine\n", indentation);
                s += &format!("{}  values:\n", indentation);
                for v in values {
                    s += &(v.clone()).lookup_fmt(lookup, indent + 2);
                }
                s
            }
            DebugExpression::Tuple { values, .. } => {
                s += &format!("{}Tuple\n", indentation);
                s += &format!("{}  values:\n", indentation);
                for v in values {
                    s += &(v.clone()).lookup_fmt(lookup, indent + 2);
                }
                s
            }
            DebugExpression::UniformArray {
                value, dimension, ..
            } => {
                s += &format!("{}UniformArray\n", indentation);
                s += &format!("{}  value:\n", indentation);
                s += &(*value.clone()).lookup_fmt(lookup, indent + 2);
                s += &format!("{}  dimension:\n", indentation);
                s += &(*dimension.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebugExpression::BusCall { id, args, .. } => {
                s += &format!("{}BusCall\n", indentation);
                s += &format!("{}  id:\n", id);
                s += &format!("{}  args:\n", indentation);
                for a in args {
                    s += &(a.clone()).lookup_fmt(lookup, indent + 2);
                }
                s
            }
            DebugExpression::AnonymousComp {
                id,
                is_parallel,
                params,
                signals,
                names: _,
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

impl DebugStatement {
    pub fn apply_iterative<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut DebugStatement),
    {
        let mut stack = vec![self]; // Stack to store DebugStatements for traversal

        while let Some(current) = stack.pop() {
            func(current); // Apply the function to the current statement

            // Push child nodes onto the stack for further processing
            match current {
                DebugStatement::IfThenElse {
                    if_case, else_case, ..
                } => {
                    stack.push(if_case);
                    if let Some(else_case) = else_case {
                        stack.push(else_case);
                    }
                }
                DebugStatement::While { stmt, .. } => {
                    stack.push(stmt);
                }
                DebugStatement::InitializationBlock {
                    initializations, ..
                } => {
                    for init in initializations.iter_mut() {
                        stack.push(init);
                    }
                }
                DebugStatement::Block { stmts, .. } => {
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
            DebugStatement::IfThenElse {
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
            DebugStatement::While { cond, stmt, meta } => {
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
            DebugStatement::Return { value, meta, .. } => {
                s += &format!(
                    "{}{}Return{} (elem_id={}):\n",
                    indentation, GREEN, RESET, meta.elem_id
                );
                s += &format!("{}  {}Value:{}:\n", indentation, MAGENTA, RESET);
                s += &(value.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebugStatement::Substitution {
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
            DebugStatement::Block { stmts, meta, .. } => {
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
            DebugStatement::Assert { arg, meta, .. } => {
                s += &format!(
                    "{}{}Assert{} (elem_id={}):\n",
                    indentation, GREEN, RESET, meta.elem_id
                );
                s += &format!("{}  {}Argument:{}:\n", indentation, YELLOW, RESET);
                s += &(arg.clone()).lookup_fmt(lookup, indent + 2);
                s
            }
            DebugStatement::InitializationBlock {
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
                    &DebugVariableType(xtype.clone())
                );
                s += &format!("{}  {}Initializations:{}\n", indentation, YELLOW, RESET);
                for i in initializations {
                    s += &(i.clone()).lookup_fmt(lookup, indent + 2);
                }
                s
            }
            DebugStatement::Declaration {
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
                    &DebugVariableType(xtype.clone())
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
            DebugStatement::MultSubstitution {
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
            DebugStatement::UnderscoreSubstitution { op, rhe, meta, .. } => {
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
            DebugStatement::ConstraintEquality { lhe, rhe, meta, .. } => {
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
            DebugStatement::LogCall { args: _, .. } => {
                format!("{}{}LogCall{}\n", indentation, GREEN, RESET)
            }
            DebugStatement::Ret => format!("{}{}Ret{}\n", indentation, BLUE, RESET),
        }
    }
}
