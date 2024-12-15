use std::hash::Hash;
use std::rc::Rc;

use colored::Colorize;
use log::warn;
use num_bigint_dig::BigInt;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use rustc_hash::{FxHashMap, FxHashSet};

use program_structure::ast::{ExpressionInfixOpcode, SignalType, Statement, VariableType};

use crate::executor::debug_ast::{
    DebugExpression, DebugExpressionInfixOpcode, DebugExpressionPrefixOpcode, DebugStatement,
};

/// Represents the access type within a symbolic expression, such as component or array access.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct OwnerName {
    pub name: usize,
    pub access: Option<Vec<SymbolicAccess>>,
    pub counter: usize,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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
                .map(|e: &OwnerName| {
                    let access_str = if e.access.is_none() {
                        ""
                    } else {
                        &e.access
                            .clone()
                            .unwrap()
                            .iter()
                            .map(|s: &SymbolicAccess| s.lookup_fmt(lookup))
                            .collect::<Vec<_>>()
                            .join("")
                    };
                    lookup[&e.name].clone() + &access_str
                })
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
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum SymbolicValue {
    ConstantInt(BigInt),
    ConstantBool(bool),
    Variable(SymbolicName),
    Assign(SymbolicValueRef, SymbolicValueRef),
    AssignEq(SymbolicValueRef, SymbolicValueRef),
    BinaryOp(
        SymbolicValueRef,
        DebugExpressionInfixOpcode,
        SymbolicValueRef,
    ),
    Conditional(SymbolicValueRef, SymbolicValueRef, SymbolicValueRef),
    UnaryOp(DebugExpressionPrefixOpcode, SymbolicValueRef),
    Array(Vec<SymbolicValueRef>),
    Tuple(Vec<SymbolicValueRef>),
    UniformArray(SymbolicValueRef, SymbolicValueRef),
    Call(usize, Vec<SymbolicValueRef>),
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
            SymbolicValue::AssignEq(lhs, rhs) => {
                format!(
                    "({} {} {})",
                    "AssignEq".green(),
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

pub type SymbolicValueRef = Rc<SymbolicValue>;

/// Represents a symbolic template used in the symbolic execution process.
#[derive(Default, Clone)]
pub struct SymbolicTemplate {
    pub template_parameter_names: Vec<usize>,
    pub inputs: FxHashSet<usize>,
    pub input_dimensions: FxHashMap<usize, Vec<DebugExpression>>,
    pub output_dimensions: FxHashMap<usize, Vec<DebugExpression>>,
    pub outputs: FxHashSet<usize>,
    pub var2type: FxHashMap<usize, VariableType>,
    pub body: Vec<DebugStatement>,
    pub require_bound_check: bool,
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
    pub args: Vec<SymbolicValueRef>,
    pub inputs: FxHashMap<SymbolicName, Option<SymbolicValue>>,
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
    pub fn register_template(
        &mut self,
        name: String,
        body: &Statement,
        template_parameter_names: &Vec<String>,
    ) {
        let mut inputs = FxHashSet::default();
        let mut input_dimensions = FxHashMap::default();
        let mut output_dimensions = FxHashMap::default();
        let mut outputs = FxHashSet::default();
        let mut var2type: FxHashMap<usize, VariableType> = FxHashMap::default();

        let require_bound_check = &name == "LessThan"
            || &name == "LessEqThan"
            || &name == "GreaterThan"
            || &name == "GreaterEqThan";

        let i = if let Some(i) = self.name2id.get(&name) {
            *i
        } else {
            self.name2id.insert(name.clone(), self.name2id.len());
            self.id2name.insert(self.name2id[&name], name.clone());
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
                            if let DebugStatement::Declaration {
                                name,
                                xtype,
                                dimensions,
                                ..
                            } = &init
                            {
                                var2type.insert(name.clone(), xtype.clone());
                                if let VariableType::Signal(typ, _taglist) = &xtype {
                                    match typ {
                                        SignalType::Input => {
                                            inputs.insert(*name);
                                            input_dimensions.insert(*name, dimensions.clone());
                                        }
                                        SignalType::Output => {
                                            outputs.insert(*name);
                                            output_dimensions.insert(*name, dimensions.clone());
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
                    .map(|p: &String| {
                        if let Some(i) = self.name2id.get(p) {
                            *i
                        } else {
                            self.name2id.insert(p.clone(), self.name2id.len());
                            self.id2name.insert(self.name2id[p], name.clone());
                            self.name2id.len() - 1
                        }
                    })
                    .collect::<Vec<_>>(),
                inputs: inputs,
                input_dimensions: input_dimensions,
                output_dimensions: output_dimensions,
                outputs: outputs,
                var2type: var2type,
                body: vec![dbody.clone(), DebugStatement::Ret],
                require_bound_check: require_bound_check,
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

pub fn access_multidimensional_array(
    values: &Vec<SymbolicValueRef>,
    dims: &[SymbolicAccess],
) -> SymbolicValue {
    let mut current_values = values.clone();
    for dim in dims {
        if let SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(a)) = dim {
            let index = a.to_usize().unwrap();
            if index < current_values.len() {
                match &*current_values[index] {
                    SymbolicValue::Array(inner_values) => {
                        current_values = inner_values.clone();
                    }
                    value => return value.clone(),
                };
            } else {
                panic!("Out of range");
            }
        } else {
            panic!("dims should be a list of SymbolicAccess::ArrayAccess");
        }
    }
    panic!("Incomplete dimensions");
}

/// Registers all elements of a multi-dimensional array in a component's map.
///
/// This function generates all possible index combinations for a multi-dimensional
/// array and registers each combination in the `elements_of_component` map.
/// For scalar values (i.e., when `dims` is empty), it registers a single entry without array access.
///
/// # Arguments
///
/// * `name` - The unique identifier for the array.
/// * `dims` - A vector containing the dimensions of the array.
/// * `elements_of_component` - A mutable reference to the map storing the component's elements.
///
/// # Examples
///
/// ```
/// use rustc_hash::FxHashMap;
/// use tcct::executor::symbolic_value::{register_array_elements,SymbolicName,SymbolicValue};
///
/// let mut elements: FxHashMap<SymbolicName, Option<SymbolicValue>> = FxHashMap::default();
/// register_array_elements(0, &vec![2, 3], None, &mut elements);
/// assert_eq!(elements.len(), 6); // 2 * 3 elements registered
/// ```
pub fn register_array_elements<T>(
    name: usize,
    dims: &Vec<usize>,
    owner: Option<Rc<Vec<OwnerName>>>,
    elements_of_component: &mut FxHashMap<SymbolicName, Option<T>>,
) {
    let mut positions = vec![vec![]];
    for size in dims {
        let mut new_positions = vec![];
        for combination in &positions {
            for i in 0..*size {
                let mut new_combination = combination.clone();
                new_combination.push(i);
                new_positions.push(new_combination);
            }
        }
        positions = new_positions;
    }

    if positions.is_empty() {
        elements_of_component.insert(
            SymbolicName {
                name: name.clone(),
                owner: if owner.is_none() {
                    Rc::new(Vec::new())
                } else {
                    owner.clone().unwrap()
                },
                access: None,
            },
            None,
        );
    } else {
        for p in positions {
            if p.is_empty() {
                elements_of_component.insert(
                    SymbolicName {
                        name: name.clone(),
                        owner: if owner.is_none() {
                            Rc::new(Vec::new())
                        } else {
                            owner.clone().unwrap()
                        },
                        access: None,
                    },
                    None,
                );
            } else {
                elements_of_component.insert(
                    SymbolicName {
                        name: name.clone(),
                        owner: if owner.is_none() {
                            Rc::new(Vec::new())
                        } else {
                            owner.clone().unwrap()
                        },
                        access: Some(
                            p.iter()
                                .map(|arg0: &usize| {
                                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(
                                        BigInt::from_usize(*arg0).unwrap(),
                                    ))
                                })
                                .collect::<Vec<_>>(),
                        ),
                    },
                    None,
                );
            }
        }
    }
}
