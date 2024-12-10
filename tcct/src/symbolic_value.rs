use colored::Colorize;
use log::warn;
use num_bigint_dig::BigInt;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;

use program_structure::ast::{ExpressionInfixOpcode, SignalType, Statement, VariableType};

use crate::debug_ast::{DebugExpressionInfixOpcode, DebugExpressionPrefixOpcode, DebugStatement};

/// Represents the access type within a symbolic expression, such as component or array access.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum SymbolicAccess {
    ComponentAccess(usize),
    ArrayAccess(SymbolicValue),
}

impl SymbolicAccess {
    /// Provides a compact format for displaying symbolic access in expressions.
    ///
    /// # Arguments
    ///
    /// * `lookup` - A hash map containing mappings of usize keys to String values.
    ///
    /// # Returns
    ///
    /// A String representation of the symbolic access.
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

/// Represents a symbolic value used in symbolic execution.
///
/// This enum can represent constants, variables, or operations such as binary, unary,
/// conditional, arrays, tuples, uniform arrays, and function calls.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct OwnerName {
    pub name: usize,
    pub counter: usize,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SymbolicName {
    pub name: usize,
    pub owner: Rc<Vec<OwnerName>>,
    pub access: Option<Vec<SymbolicAccess>>,
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
            if let Some(access) = &self.access {
                access
                    .iter()
                    .map(|s: &SymbolicAccess| s.lookup_fmt(lookup))
                    .collect::<Vec<_>>()
                    .join("")
            } else {
                "".to_string()
            }
        )
    }
}

/// Represents a symbolic value used in symbolic execution.
///
/// This enum can represent constants, variables, or operations such as binary, unary,
/// conditional, arrays, tuples, uniform arrays, and function calls.
#[derive(Clone, Hash, Eq, PartialEq)]
pub enum SymbolicValue {
    ConstantInt(BigInt),
    ConstantBool(bool),
    Variable(SymbolicName),
    Assign(Rc<SymbolicValue>, Rc<SymbolicValue>),
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

impl SymbolicValue {
    /// Formats the symbolic value for lookup and debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `lookup` - A hash map containing mappings of usize keys to String values.
    ///
    /// # Returns
    ///
    /// A String representation of the symbolic value.
    pub fn lookup_fmt(&self, lookup: &FxHashMap<usize, String>) -> String {
        match self {
            SymbolicValue::ConstantInt(value) => format!("{}", value),
            SymbolicValue::ConstantBool(flag) => {
                format!("{} {}", if *flag { "✅" } else { "❌" }, flag)
            }
            SymbolicValue::Variable(sname) => sname.lookup_fmt(lookup),
            SymbolicValue::Assign(lhs, rhs) => {
                format!(
                    "({} {} {})",
                    "Assign".green(),
                    lhs.lookup_fmt(lookup),
                    rhs.lookup_fmt(lookup)
                )
            }
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

/// Represents a symbolic template used in the symbolic execution process.
#[derive(Default, Clone)]
pub struct SymbolicTemplate {
    pub template_parameter_names: Vec<usize>,
    pub inputs: FxHashSet<usize>,
    pub outputs: FxHashSet<usize>,
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

/// Manages symbolic libraries, templates, and functions for symbolic execution.
#[derive(Default, Clone)]
pub struct SymbolicLibrary {
    pub template_library: FxHashMap<usize, Box<SymbolicTemplate>>,
    pub function_library: FxHashMap<usize, Box<SymbolicFunction>>,
    pub name2id: FxHashMap<String, usize>,
    pub id2name: FxHashMap<usize, String>,
    pub function_counter: FxHashMap<usize, usize>,
}

impl SymbolicLibrary {
    /// Clears the function counter for all registered functions.
    pub fn clear_function_counter(&mut self) {
        for (k, _) in self.function_library.iter() {
            self.function_counter.insert(*k, 0_usize);
        }
    }

    /// Registers a library template by extracting input signals from the provided block statement body.
    ///
    /// # Arguments
    ///
    /// * `name` - Name under which the template will be registered within the library.
    /// * `body` - Block statement serving as the main logic body defining the behavior captured by the template.
    /// * `template_parameter_names` - List of names identifying parameters used within the template logic.
    pub fn register_library(
        &mut self,
        name: String,
        body: &Statement,
        template_parameter_names: &Vec<String>,
    ) {
        let mut inputs = FxHashSet::default();
        let mut outputs = FxHashSet::default();
        let mut var2type: FxHashMap<usize, VariableType> = FxHashMap::default();

        let i = if let Some(i) = self.name2id.get(&name) {
            *i
        } else {
            self.name2id.insert(name.clone(), self.name2id.len());
            self.id2name.insert(self.name2id[&name], name);
            self.name2id.len() - 1
        };

        let dbody = DebugStatement::from(body.clone(), &mut self.name2id, &mut self.id2name);
        match dbody {
            DebugStatement::Block { ref stmts, .. } => {
                for s in stmts {
                    if let DebugStatement::InitializationBlock {
                        initializations, ..
                    } = &s
                    {
                        for init in initializations {
                            if let DebugStatement::Declaration { name, xtype, .. } = &init {
                                var2type.insert(name.clone(), xtype.clone());
                                if let VariableType::Signal(typ, _taglist) = &xtype {
                                    match typ {
                                        SignalType::Input => {
                                            inputs.insert(*name);
                                        }
                                        SignalType::Output => {
                                            outputs.insert(*name);
                                        }
                                        SignalType::Intermediate => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                warn!("Cannot Find Block Statement");
            }
        }

        self.template_library.insert(
            i,
            Box::new(SymbolicTemplate {
                template_parameter_names: template_parameter_names
                    .iter()
                    .map(|p: &String| self.name2id[p])
                    .collect::<Vec<_>>(),
                inputs: inputs,
                outputs: outputs,
                var2type: var2type,
                body: vec![dbody.clone(), DebugStatement::Ret],
            }),
        );
    }

    /// Registers a function in the symbolic library.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the function to be registered.
    /// * `body` - The function body as a Statement.
    /// * `function_argument_names` - List of argument names for the function.
    pub fn register_function(
        &mut self,
        name: String,
        body: Statement,
        function_argument_names: &Vec<String>,
    ) {
        let i = if let Some(i) = self.name2id.get(&name) {
            *i
        } else {
            self.name2id.insert(name.clone(), self.name2id.len());
            self.id2name.insert(self.name2id[&name], name);
            self.name2id.len() - 1
        };

        let dbody = DebugStatement::from(body, &mut self.name2id, &mut self.id2name);
        self.function_library.insert(
            i,
            Box::new(SymbolicFunction {
                function_argument_names: function_argument_names
                    .iter()
                    .map(|p: &String| self.name2id[p])
                    .collect::<Vec<_>>(),
                body: vec![dbody, DebugStatement::Ret],
            }),
        );
        self.function_counter.insert(i, 0_usize);
    }
}
