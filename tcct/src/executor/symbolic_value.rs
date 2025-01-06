use std::cell::RefCell;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use colored::Colorize;
use num_bigint_dig::BigInt;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use num_traits::{One, Signed, Zero};
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

use program_structure::ast::{
    ExpressionInfixOpcode, ExpressionPrefixOpcode, SignalType, Statement, VariableType,
};

use crate::executor::debug_ast::{
    DebugExpression, DebugExpressionInfixOpcode, DebugExpressionPrefixOpcode, DebugStatement,
};
use crate::executor::utils::{extended_euclidean, generate_cartesian_product_indices, modpow};

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
    pub id: usize,
    pub access: Option<Vec<SymbolicAccess>>,
    pub counter: usize,
}

#[derive(Clone, Debug)]
pub struct SymbolicName {
    pub id: usize,
    pub owner: Rc<Vec<OwnerName>>,
    pub access: Option<Vec<SymbolicAccess>>,
    precomputed_hash: RefCell<Option<u64>>,
}

impl SymbolicName {
    pub fn new(id: usize, owner: Rc<Vec<OwnerName>>, access: Option<Vec<SymbolicAccess>>) -> Self {
        SymbolicName {
            id,
            owner,
            access,
            precomputed_hash: RefCell::new(None),
        }
    }

    pub fn get_the_final_component_id(&self) -> Option<usize> {
        let mut fti = None;
        for o in self.owner.iter() {
            if o.access.is_some() {
                for a in o.access.clone().unwrap() {
                    if let SymbolicAccess::ComponentAccess(i) = a {
                        fti = Some(i);
                    }
                }
            }
        }
        fti
    }

    pub fn get_dim(&self) -> usize {
        if let Some(ref local_access) = self.access {
            local_access.len()
        } else {
            0
        }
    }

    pub fn lookup_fmt(&self, lookup: &FxHashMap<usize, String>) -> String {
        format!(
            "{}.{}{}",
            self.owner
                .iter()
                .map(|e: &OwnerName| {
                    let access_str: String = if let Some(accesses) = &e.access {
                        accesses
                            .iter()
                            .map(|s: &SymbolicAccess| s.lookup_fmt(lookup))
                            .collect::<Vec<_>>()
                            .join("")
                    } else {
                        "".to_string()
                    };
                    lookup[&e.id].clone() + &access_str
                })
                .collect::<Vec<_>>()
                .join("."),
            lookup[&self.id].clone(),
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

    fn compute_hash(&self) -> u64 {
        let mut hasher = FxHasher::default(); // Use FxHasher for consistency with FxHashMap
        self.id.hash(&mut hasher);
        self.owner.hash(&mut hasher);
        self.access.hash(&mut hasher);
        hasher.finish()
    }

    pub fn update_hash(&self) {
        *self.precomputed_hash.borrow_mut() = Some(self.compute_hash());
    }
}

impl PartialEq for SymbolicName {
    fn eq(&self, other: &Self) -> bool {
        // Check if precomputed hashes are available for both instances
        let self_hash = *self.precomputed_hash.borrow();
        let other_hash = *other.precomputed_hash.borrow();

        // If both hashes are available and not `None`, compare the hashes
        if let (Some(self_hash), Some(other_hash)) = (self_hash, other_hash) {
            return self_hash == other_hash;
        }

        self.id == other.id && *self.owner == *other.owner && self.access == other.access
    }
}

impl Eq for SymbolicName {}

impl Hash for SymbolicName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Use cached hash if available
        let cached_hash = *self.precomputed_hash.borrow();
        if let Some(hash) = cached_hash {
            hash.hash(state);
        } else {
            // Compute and cache the hash value
            let hash = self.compute_hash();
            *self.precomputed_hash.borrow_mut() = Some(hash);
            hash.hash(state);
        }
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
    Assign(SymbolicValueRef, SymbolicValueRef, bool),
    AssignEq(SymbolicValueRef, SymbolicValueRef),
    AssignCall(SymbolicValueRef, SymbolicValueRef, bool),
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
                format!(
                    "{}",
                    if *flag {
                        "true".on_bright_green().white()
                    } else {
                        "false".on_bright_red().white()
                    }
                )
            }
            SymbolicValue::Variable(sname) => sname.lookup_fmt(lookup),
            SymbolicValue::Assign(lhs, rhs, is_safe) => {
                format!(
                    "({} {} {})",
                    if *is_safe {
                        "Assign💖".green()
                    } else {
                        "Assign".green()
                    },
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
            SymbolicValue::AssignCall(lhs, rhs, _is_mutable) => {
                format!(
                    "({} {} {})",
                    "AssignCall".green(),
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
                    "<🤔 {} ? {} : {}>",
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
                    "[🧬 {}]",
                    elems
                        .into_iter()
                        .map(|a| a.lookup_fmt(lookup))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            SymbolicValue::UniformArray(elem, counts) => {
                format!(
                    "(🧬 {}, {})",
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
    pub input_ids: FxHashSet<usize>,
    pub output_ids: FxHashSet<usize>,
    pub id2type: FxHashMap<usize, VariableType>,
    pub id2dimensions: FxHashMap<usize, Vec<DebugExpression>>,
    pub body: Vec<DebugStatement>,
    pub is_lessthan: bool,
    pub is_safe: bool,
}

/// Represents a symbolic function used in the symbolic execution process.
#[derive(Default, Clone)]
pub struct SymbolicFunction {
    pub function_argument_names: Vec<usize>,
    pub id2dimensions: FxHashMap<usize, Vec<DebugExpression>>,
    pub body: Vec<DebugStatement>,
}

/// Represents a symbolic component used in the symbolic execution process.
#[derive(Default, Clone)]
pub struct SymbolicComponent {
    pub template_name: usize,
    pub args: Vec<SymbolicValueRef>,
    pub symbol_optional_binding_map: FxHashMap<SymbolicName, Option<SymbolicValue>>,
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

fn gather_variables_for_template(
    dbody: &DebugStatement,
    input_ids: &mut FxHashSet<usize>,
    output_ids: &mut FxHashSet<usize>,
    id2type: &mut FxHashMap<usize, VariableType>,
    id2dimensions: &mut FxHashMap<usize, Vec<DebugExpression>>,
) {
    if let DebugStatement::Declaration {
        name,
        xtype,
        dimensions,
        ..
    } = dbody
    {
        id2type.insert(name.clone(), xtype.clone());
        id2dimensions.insert(name.clone(), dimensions.clone());
        if let VariableType::Signal(typ, _taglist) = &xtype {
            match typ {
                SignalType::Input => {
                    input_ids.insert(*name);
                }
                SignalType::Output => {
                    output_ids.insert(*name);
                }
                SignalType::Intermediate => {}
            }
        }
    }
}

fn gather_variables_for_function(
    dbody: &DebugStatement,
    id2dimensions: &mut FxHashMap<usize, Vec<DebugExpression>>,
) {
    if let DebugStatement::Declaration {
        name, dimensions, ..
    } = dbody
    {
        id2dimensions.insert(name.clone(), dimensions.clone());
    }
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
        let mut input_ids = FxHashSet::default();
        let mut output_ids = FxHashSet::default();
        let mut id2type = FxHashMap::default();
        let mut id2dimensions = FxHashMap::default();

        let is_lessthan = &name == "LessThan";
        let is_safe = &name == "Num2Bits";

        let i = if let Some(i) = self.name2id.get(&name) {
            *i
        } else {
            self.name2id.insert(name.clone(), self.name2id.len());
            self.id2name.insert(self.name2id[&name], name.clone());
            self.name2id.len() - 1
        };

        let mut dbody = DebugStatement::from(body.clone(), &mut self.name2id, &mut self.id2name);
        dbody.apply_iterative(|stmt| {
            gather_variables_for_template(
                stmt,
                &mut input_ids,
                &mut output_ids,
                &mut id2type,
                &mut id2dimensions,
            );
        });

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
                input_ids: input_ids,
                output_ids: output_ids,
                id2type: id2type,
                id2dimensions: id2dimensions,
                body: vec![dbody.clone(), DebugStatement::Ret],
                is_lessthan: is_lessthan,
                is_safe: is_safe,
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
        let mut id2dimensions = FxHashMap::default();
        let i = if let Some(i) = self.name2id.get(&name) {
            *i
        } else {
            self.name2id.insert(name.clone(), self.name2id.len());
            self.id2name.insert(self.name2id[&name], name);
            self.name2id.len() - 1
        };

        let mut dbody = DebugStatement::from(body, &mut self.name2id, &mut self.id2name);
        dbody.apply_iterative(|stmt| {
            gather_variables_for_function(stmt, &mut id2dimensions);
        });

        self.function_library.insert(
            i,
            Box::new(SymbolicFunction {
                function_argument_names: function_argument_names
                    .iter()
                    .map(|p: &String| self.name2id[p])
                    .collect::<Vec<_>>(),
                id2dimensions: id2dimensions,
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
            //panic!("dims should be a list of SymbolicAccess::ArrayAccess");
            return SymbolicValue::Array(current_values);
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
    let positions = generate_cartesian_product_indices(dims);

    if positions.is_empty() {
        elements_of_component.insert(
            SymbolicName::new(
                name.clone(),
                if owner.is_none() {
                    Rc::new(Vec::new())
                } else {
                    owner.clone().unwrap()
                },
                None,
            ),
            None,
        );
    } else {
        for p in positions {
            if p.is_empty() {
                elements_of_component.insert(
                    SymbolicName::new(
                        name.clone(),
                        if owner.is_none() {
                            Rc::new(Vec::new())
                        } else {
                            owner.clone().unwrap()
                        },
                        None,
                    ),
                    None,
                );
            } else {
                elements_of_component.insert(
                    SymbolicName::new(
                        name.clone(),
                        if owner.is_none() {
                            Rc::new(Vec::new())
                        } else {
                            owner.clone().unwrap()
                        },
                        Some(
                            p.iter()
                                .map(|arg0: &usize| {
                                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(
                                        BigInt::from_usize(*arg0).unwrap(),
                                    ))
                                })
                                .collect::<Vec<_>>(),
                        ),
                    ),
                    None,
                );
            }
        }
    }
}

/// Recursively enumerates elements of a nested `SymbolicValue::Array`, returning a vector of tuples
/// containing the index path and a reference to each non-array element.
///
/// # Arguments
///
/// * `value` - A reference to the `SymbolicValue` to enumerate.
///
/// # Returns
///
/// A `Vec<(Vec<usize>, &SymbolicValue)>` where each tuple contains:
/// - A `Vec<usize>` representing the index path to the element.
/// - A reference to the non-array `SymbolicValue`.
pub fn enumerate_array(value: &SymbolicValue) -> Vec<(Vec<usize>, &SymbolicValue)> {
    let mut result = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back((Vec::new(), value));

    while let Some((index, val)) = queue.pop_front() {
        match val {
            SymbolicValue::Array(arr) => {
                for (i, item) in arr.iter().enumerate() {
                    let mut new_index = index.clone();
                    new_index.push(i);
                    queue.push_back((new_index, item));
                }
            }
            _ => {
                result.push((index, val));
            }
        }
    }

    result
}

pub fn is_true(val: &SymbolicValue) -> bool {
    if let SymbolicValue::ConstantBool(true) = val {
        true
    } else {
        false
    }
}

pub fn evaluate_binary_op(
    lhs: &SymbolicValue,
    rhs: &SymbolicValue,
    prime: &BigInt,
    op: &DebugExpressionInfixOpcode,
) -> SymbolicValue {
    let (normalized_lhs, normalized_rhs) = match &op.0 {
        // Convert booleans to integers for arithmetic or bitwise operators
        ExpressionInfixOpcode::Add
        | ExpressionInfixOpcode::Sub
        | ExpressionInfixOpcode::Mul
        | ExpressionInfixOpcode::Pow
        | ExpressionInfixOpcode::Div
        | ExpressionInfixOpcode::IntDiv
        | ExpressionInfixOpcode::Mod
        | ExpressionInfixOpcode::BitOr
        | ExpressionInfixOpcode::BitAnd
        | ExpressionInfixOpcode::BitXor
        | ExpressionInfixOpcode::ShiftL
        | ExpressionInfixOpcode::ShiftR
        | ExpressionInfixOpcode::Lesser
        | ExpressionInfixOpcode::Greater
        | ExpressionInfixOpcode::LesserEq
        | ExpressionInfixOpcode::GreaterEq
        | ExpressionInfixOpcode::Eq
        | ExpressionInfixOpcode::NotEq => (normalize_to_int(lhs), normalize_to_int(rhs)),
        // Keep booleans as they are for logical operators
        ExpressionInfixOpcode::BoolAnd | ExpressionInfixOpcode::BoolOr => {
            (normalize_to_bool(lhs, prime), normalize_to_bool(rhs, prime))
        }
        _ => (lhs.clone(), rhs.clone()), // Default case
    };

    match (&normalized_lhs, &normalized_rhs) {
        (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => match &op.0 {
            ExpressionInfixOpcode::Add => SymbolicValue::ConstantInt((lv + rv) % prime),
            ExpressionInfixOpcode::Sub => SymbolicValue::ConstantInt((lv - rv) % prime),
            ExpressionInfixOpcode::Mul => SymbolicValue::ConstantInt((lv * rv) % prime),
            ExpressionInfixOpcode::Pow => SymbolicValue::ConstantInt(modpow(lv, rv, prime)),
            ExpressionInfixOpcode::Div => {
                if rv.is_zero() {
                    SymbolicValue::ConstantInt(BigInt::zero())
                } else {
                    let mut r = prime.clone();
                    let mut new_r = rv.clone();
                    if r.is_negative() {
                        r += prime;
                    }
                    if new_r.is_negative() {
                        new_r += prime;
                    }

                    let (_, _, mut rv_inv) = extended_euclidean(r, new_r);
                    rv_inv %= prime;
                    if rv_inv.is_negative() {
                        rv_inv += prime;
                    }

                    SymbolicValue::ConstantInt((lv * rv_inv) % prime)
                }
            }
            ExpressionInfixOpcode::IntDiv => SymbolicValue::ConstantInt(lv / rv),
            ExpressionInfixOpcode::Mod => SymbolicValue::ConstantInt(lv % rv),
            ExpressionInfixOpcode::BitOr => SymbolicValue::ConstantInt(lv | rv),
            ExpressionInfixOpcode::BitAnd => SymbolicValue::ConstantInt(lv & rv),
            ExpressionInfixOpcode::BitXor => SymbolicValue::ConstantInt(lv ^ rv),
            ExpressionInfixOpcode::ShiftL => {
                SymbolicValue::ConstantInt(lv << rv.to_usize().unwrap())
            }
            ExpressionInfixOpcode::ShiftR => {
                SymbolicValue::ConstantInt(lv >> rv.to_usize().unwrap())
            }
            ExpressionInfixOpcode::Lesser => SymbolicValue::ConstantBool(lv % prime < rv % prime),
            ExpressionInfixOpcode::Greater => SymbolicValue::ConstantBool(lv % prime > rv % prime),
            ExpressionInfixOpcode::LesserEq => {
                SymbolicValue::ConstantBool(lv % prime <= rv % prime)
            }
            ExpressionInfixOpcode::GreaterEq => {
                SymbolicValue::ConstantBool(lv % prime >= rv % prime)
            }
            ExpressionInfixOpcode::Eq => SymbolicValue::ConstantBool(lv % prime == rv % prime),
            ExpressionInfixOpcode::NotEq => SymbolicValue::ConstantBool(lv % prime != rv % prime),
            _ => todo!("{:?} is currently not supported", op),
        },
        (SymbolicValue::ConstantBool(lv), SymbolicValue::ConstantBool(rv)) => match &op.0 {
            ExpressionInfixOpcode::BoolAnd => SymbolicValue::ConstantBool(*lv && *rv),
            ExpressionInfixOpcode::BoolOr => SymbolicValue::ConstantBool(*lv || *rv),
            _ => todo!("{:?} is currently not supported", op),
        },
        _ => SymbolicValue::BinaryOp(Rc::new(normalized_lhs), op.clone(), Rc::new(normalized_rhs)),
    }
}

fn normalize_to_int(val: &SymbolicValue) -> SymbolicValue {
    match val {
        SymbolicValue::ConstantBool(b) => {
            SymbolicValue::ConstantInt(if *b { BigInt::one() } else { BigInt::zero() })
        }
        _ => val.clone(),
    }
}

fn normalize_to_bool(val: &SymbolicValue, prime: &BigInt) -> SymbolicValue {
    match val {
        SymbolicValue::ConstantInt(v) => SymbolicValue::ConstantBool(!(v % prime).is_zero()),
        _ => val.clone(),
    }
}

pub fn generate_lessthan_constraint(
    name2id: &FxHashMap<String, usize>,
    owner_name: Rc<Vec<OwnerName>>,
) -> SymbolicValue {
    let in_0 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        name2id["in"],
        owner_name.clone(),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::zero()),
        )]),
    )));
    let in_1 = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        name2id["in"],
        owner_name.clone(),
        Some(vec![SymbolicAccess::ArrayAccess(
            SymbolicValue::ConstantInt(BigInt::one()),
        )]),
    )));
    let lessthan_out = Rc::new(SymbolicValue::Variable(SymbolicName::new(
        name2id["out"],
        owner_name,
        None,
    )));
    let cond_1 = SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::ConstantInt(BigInt::one())),
            DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
            lessthan_out.clone(),
        )),
        DebugExpressionInfixOpcode(ExpressionInfixOpcode::BoolAnd),
        Rc::new(SymbolicValue::BinaryOp(
            in_0.clone(),
            DebugExpressionInfixOpcode(ExpressionInfixOpcode::Lesser),
            in_1.clone(),
        )),
    );
    let cond_0 = SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
            DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
            lessthan_out.clone(),
        )),
        DebugExpressionInfixOpcode(ExpressionInfixOpcode::BoolAnd),
        Rc::new(SymbolicValue::BinaryOp(
            in_0,
            DebugExpressionInfixOpcode(ExpressionInfixOpcode::GreaterEq),
            in_1,
        )),
    );
    SymbolicValue::BinaryOp(
        Rc::new(cond_1),
        DebugExpressionInfixOpcode(ExpressionInfixOpcode::BoolOr),
        Rc::new(cond_0),
    )
}

pub fn negate_condition(condition: &SymbolicValue) -> SymbolicValue {
    match condition {
        SymbolicValue::ConstantBool(v) => SymbolicValue::ConstantBool(!v),
        _ => SymbolicValue::UnaryOp(
            DebugExpressionPrefixOpcode(ExpressionPrefixOpcode::BoolNot),
            Rc::new(condition.clone()),
        ),
    }
}

fn check_array_concrete(array: &Vec<SymbolicValueRef>) -> bool {
    for value_ref in array {
        match &**value_ref {
            SymbolicValue::Array(inner_array) => {
                if !check_array_concrete(inner_array) {
                    return false;
                }
            }
            value => match value {
                SymbolicValue::ConstantBool(_) => {}
                SymbolicValue::ConstantInt(_) => {}
                _ => return false,
            },
        }
    }
    true
}

pub fn is_concrete_array(value: &SymbolicValue) -> bool {
    match value {
        SymbolicValue::Array(array) => check_array_concrete(array),
        _ => false,
    }
}

pub fn decompose_uniform_array(
    symval: SymbolicValueRef,
) -> (SymbolicValueRef, Vec<SymbolicValueRef>) {
    let mut current = symval.clone();
    let mut dims = Vec::new();
    while let SymbolicValue::UniformArray(elem, count) = (*current).clone() {
        dims.push(count); // Append count to the dimensions vector
        current = elem.clone();
    }
    (current, dims)
}

pub fn create_nested_array(
    dims: &[usize],
    initial_value: SymbolicValueRef,
) -> Vec<SymbolicValueRef> {
    if dims.is_empty() {
        panic!("empty dimensions");
    }

    if dims.len() == 1 {
        vec![initial_value; dims[0]]
    } else {
        vec![
            Rc::new(SymbolicValue::Array(create_nested_array(
                &dims[1..],
                initial_value.clone()
            )));
            dims[0]
        ]
    }
}

pub fn update_nested_array(
    dims: &[usize],
    array: SymbolicValueRef,
    value: SymbolicValueRef,
) -> SymbolicValueRef {
    if let SymbolicValue::Array(arr) = (*array).clone() {
        let mut new_arr = arr.clone();
        if dims.len() == 1 {
            new_arr[dims[0]] = value;
        } else {
            new_arr[dims[0]] = update_nested_array(&dims[1..], arr[dims[0]].clone(), value);
        }
        Rc::new(SymbolicValue::Array(new_arr))
    } else {
        array
    }
}
