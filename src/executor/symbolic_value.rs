use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use colored::Colorize;
use num_bigint_dig::BigInt;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use num_traits::{One, Signed, Zero};
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

use program_structure::ast::{ExpressionInfixOpcode, SignalType, Statement, VariableType};

use crate::executor::debug_ast::{
    DebuggableExpression, DebuggableExpressionInfixOpcode, DebuggableExpressionPrefixOpcode,
    DebuggableStatement,
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

#[derive(Clone)]
pub enum ExecutionResult<T> {
    Success(T),
    Unkonwn,
    Failure,
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

    fn get_or_update_hash(&self) -> u64 {
        // Check if the hash has been computed already.
        if let Some(hash) = *self.precomputed_hash.borrow() {
            hash
        } else {
            // Compute the hash and store it.
            self.update_hash();
            (*self.precomputed_hash.borrow()).unwrap()
        }
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

impl PartialOrd for SymbolicName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SymbolicName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_or_update_hash().cmp(&other.get_or_update_hash())
    }
}

pub type QuadraticPoly = (SymbolicName, [SymbolicValueRef; 3]);

/// Represents a symbolic value used in symbolic execution.
///
/// This enum can represent constants, variables, or operations such as binary, unary,
/// conditional, arrays, tuples, uniform arrays, and function calls.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum SymbolicValue {
    NOP,
    ConstantInt(BigInt),
    ConstantBool(bool),
    Variable(SymbolicName),
    Assign(
        SymbolicValueRef,
        SymbolicValueRef,
        bool,
        Option<(Vec<QuadraticPoly>, Vec<QuadraticPoly>)>,
    ),
    AssignEq(SymbolicValueRef, SymbolicValueRef),
    AssignTemplParam(SymbolicValueRef, SymbolicValueRef),
    AssignCall(SymbolicValueRef, SymbolicValueRef, bool),
    BinaryOp(
        SymbolicValueRef,
        DebuggableExpressionInfixOpcode,
        SymbolicValueRef,
    ),
    AuxBinaryOp(
        SymbolicValueRef,
        DebuggableExpressionInfixOpcode,
        SymbolicValueRef,
    ),
    Conditional(SymbolicValueRef, SymbolicValueRef, SymbolicValueRef),
    UnaryOp(DebuggableExpressionPrefixOpcode, SymbolicValueRef),
    Array(Vec<SymbolicValueRef>),
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
            SymbolicValue::NOP => "NOP".to_string(),
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
            SymbolicValue::Variable(sym_name) => sym_name.lookup_fmt(lookup),
            SymbolicValue::Assign(lhs, rhs, is_safe, ..) => {
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
            SymbolicValue::AssignTemplParam(lhs, rhs) => {
                format!(
                    "({} {} {})",
                    "AssignTemplParam".green(),
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
            SymbolicValue::BinaryOp(lhs, op, rhs) | SymbolicValue::AuxBinaryOp(lhs, op, rhs) => {
                match &op.0 {
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
                }
            }
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
                    "[🔗 {}]",
                    elems
                        .into_iter()
                        .map(|a| a.lookup_fmt(lookup))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            SymbolicValue::UniformArray(elem, counts) => {
                format!(
                    "(🔗 {}, {})",
                    elem.lookup_fmt(lookup),
                    counts.lookup_fmt(lookup)
                )
            }
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
    pub id2dimension_expressions: FxHashMap<usize, Vec<DebuggableExpression>>,
    pub body: Vec<DebuggableStatement>,
    pub is_lessthan: bool,
    pub is_safe: bool,
}

/// Represents a symbolic function used in the symbolic execution process.
#[derive(Default, Clone)]
pub struct SymbolicFunction {
    pub function_argument_names: Vec<usize>,
    pub id2dimension_expressions: FxHashMap<usize, Vec<DebuggableExpression>>,
    pub body: Vec<DebuggableStatement>,
}

/// Represents a symbolic component used in the symbolic execution process.
#[derive(Default, Clone)]
pub struct SymbolicComponent {
    pub template_id: usize,
    pub args: Vec<SymbolicValueRef>,
    pub inputs_binding_map: FxHashMap<SymbolicName, Option<SymbolicValue>>,
    pub id2dimensions: FxHashMap<usize, Vec<usize>>,
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
    dbody: &DebuggableStatement,
    input_ids: &mut FxHashSet<usize>,
    output_ids: &mut FxHashSet<usize>,
    id2type: &mut FxHashMap<usize, VariableType>,
    id2dimensions: &mut FxHashMap<usize, Vec<DebuggableExpression>>,
) {
    if let DebuggableStatement::Declaration {
        id,
        xtype,
        dimensions,
        ..
    } = dbody
    {
        id2type.insert(*id, xtype.clone());
        id2dimensions.insert(*id, dimensions.clone());
        if let VariableType::Signal(typ, _taglist) = &xtype {
            match typ {
                SignalType::Input => {
                    input_ids.insert(*id);
                }
                SignalType::Output => {
                    output_ids.insert(*id);
                }
                SignalType::Intermediate => {}
            }
        }
    }
}

fn gather_variables_for_function(
    dbody: &DebuggableStatement,
    id2dimensions: &mut FxHashMap<usize, Vec<DebuggableExpression>>,
) {
    if let DebuggableStatement::Declaration { id, dimensions, .. } = dbody {
        id2dimensions.insert(*id, dimensions.clone());
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
    /// * `whitelist` -
    /// * `is_lessthan_dissabled` -
    pub fn register_template(
        &mut self,
        name: String,
        body: &Statement,
        template_parameter_names: &Vec<String>,
        whitelist: &FxHashSet<String>,
        is_lessthan_dissabled: bool,
    ) {
        let mut input_ids = FxHashSet::default();
        let mut output_ids = FxHashSet::default();
        let mut id2type = FxHashMap::default();
        let mut id2dimension_expressions = FxHashMap::default();

        let is_lessthan = !is_lessthan_dissabled && &name == "LessThan";
        let is_safe = whitelist.contains(&name);

        let i = if let Some(i) = self.name2id.get(&name) {
            *i
        } else {
            self.name2id.insert((*name).to_string(), self.name2id.len());
            self.id2name.insert(self.name2id[&name], name.clone());
            self.name2id.len() - 1
        };

        let mut dbody =
            DebuggableStatement::from(body.clone(), &mut self.name2id, &mut self.id2name);
        dbody.apply_iterative(|stmt| {
            gather_variables_for_template(
                stmt,
                &mut input_ids,
                &mut output_ids,
                &mut id2type,
                &mut id2dimension_expressions,
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
                id2dimension_expressions: id2dimension_expressions,
                body: vec![dbody.clone(), DebuggableStatement::Ret],
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
        let mut id2dimension_expressions = FxHashMap::default();
        let i = if let Some(i) = self.name2id.get(&name) {
            *i
        } else {
            self.name2id.insert(name.clone(), self.name2id.len());
            self.id2name.insert(self.name2id[&name], name);
            self.name2id.len() - 1
        };

        let mut dbody = DebuggableStatement::from(body, &mut self.name2id, &mut self.id2name);
        dbody.apply_iterative(|stmt| {
            gather_variables_for_function(stmt, &mut id2dimension_expressions);
        });

        self.function_library.insert(
            i,
            Box::new(SymbolicFunction {
                function_argument_names: function_argument_names
                    .iter()
                    .map(|p: &String| self.name2id[p])
                    .collect::<Vec<_>>(),
                id2dimension_expressions: id2dimension_expressions,
                body: vec![dbody, DebuggableStatement::Ret],
            }),
        );
        self.function_counter.insert(i, 0_usize);
    }
}

pub fn access_multidimensional_array(
    values: &Vec<SymbolicValueRef>,
    dims: &[SymbolicAccess],
) -> ExecutionResult<SymbolicValue> {
    let mut current_values = values;
    for dim in dims {
        if let SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(a)) = dim {
            if let Some(index) = a.to_usize() {
                if index < current_values.len() {
                    match &*current_values[index] {
                        SymbolicValue::Array(inner_values) => {
                            current_values = &inner_values;
                        }
                        value => return ExecutionResult::Success(value.clone()),
                    };
                } else {
                    return ExecutionResult::Failure;
                }
            } else {
                return ExecutionResult::Failure;
            }
        } else {
            //panic!("dims should be a list of SymbolicAccess::ArrayAccess");
            return ExecutionResult::Unkonwn;
        }
    }
    //panic!("Incomplete dimensions");
    ExecutionResult::Success(SymbolicValue::Array(current_values.to_vec()))
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
/// use zkfuzz::executor::symbolic_value::{register_array_elements,SymbolicName,SymbolicValue};
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

pub fn val_for_relational_operators(z: &BigInt, p: &BigInt) -> BigInt {
    if &(p / BigInt::from(2) + BigInt::one()) <= z && z < p {
        z - p
    } else {
        z.clone()
    }
}

/// Evaluates a binary operation on two symbolic values, taking into account modular arithmetic
/// with a specified prime and the type of operation.
///
/// # Parameters
/// - `lhs`: A reference to the left-hand side operand, represented as a `SymbolicValue`.
/// - `rhs`: A reference to the right-hand side operand, represented as a `SymbolicValue`.
/// - `prime`: A reference to a `BigInt` representing the prime modulus used for modular arithmetic.
/// - `op`: A reference to a `DebuggableExpressionInfixOpcode` representing the binary operation to evaluate.
///
/// # Returns
/// A `SymbolicValue` representing the result of the binary operation. The result may be:
/// - A constant integer or boolean for fully evaluated operations.
/// - A new `BinaryOp` if the operands are not constant values.
///
/// # Behavior
/// 1. Normalizes `lhs` and `rhs` based on the operation:
///    - Arithmetic and bitwise operators convert operands to integers.
///    - Logical operators retain their boolean values.
/// 2. Performs the specified operation if both operands are constants:
///    - Arithmetic (e.g., addition, subtraction, multiplication, division, modulo).
///    - Bitwise operations (e.g., AND, OR, XOR, shifts).
///    - Logical comparisons (e.g., equality, greater than, less than).
/// 3. Returns a `BinaryOp` if the operands are non-constant values, preserving the symbolic structure.
///
/// # Special Cases
/// - Modular arithmetic operations respect the `prime` modulus.
/// - Division and modulo operations handle zero as a special case, returning zero when the divisor is zero.
/// - Negative values are normalized using the modulus where applicable.
///
/// # Panics
/// - The function may panic if an unsupported operation is encountered in a match statement with `todo!`.
///
/// # Example
/// ```rust
/// use num_bigint_dig::BigInt;
///
/// use program_structure::ast::ExpressionInfixOpcode;
///
/// use zkfuzz::executor::debug_ast::{DebuggableExpressionInfixOpcode};
/// use zkfuzz::executor::symbolic_value::{SymbolicValue, evaluate_binary_op};
///
/// let lhs = SymbolicValue::ConstantInt(BigInt::from(10));
/// let rhs = SymbolicValue::ConstantInt(BigInt::from(3));
/// let prime = BigInt::from(7);
/// let op = DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add);
///
/// let result = evaluate_binary_op(&lhs, &rhs, &prime, &op);
/// assert_eq!(result, SymbolicValue::ConstantInt(BigInt::from(6))); // (10 + 3) % 7 = 6
/// ```
pub fn evaluate_binary_op(
    lhs: &SymbolicValue,
    rhs: &SymbolicValue,
    prime: &BigInt,
    op: &DebuggableExpressionInfixOpcode,
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
        | ExpressionInfixOpcode::NotEq => {
            (normalize_to_int(lhs, prime), normalize_to_int(rhs, prime))
        }
        // Keep booleans as they are for logical operators
        ExpressionInfixOpcode::BoolAnd | ExpressionInfixOpcode::BoolOr => {
            (normalize_to_bool(lhs, prime), normalize_to_bool(rhs, prime))
        } //_ => (lhs.clone(), rhs.clone()), // Default case
    };

    match (&normalized_lhs, &normalized_rhs) {
        (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => match &op.0 {
            ExpressionInfixOpcode::Add => SymbolicValue::ConstantInt((lv + rv) % prime),
            ExpressionInfixOpcode::Sub => {
                let mut tmp = (lv - rv) % prime;
                if tmp.is_negative() {
                    tmp += prime;
                }
                SymbolicValue::ConstantInt(tmp)
            }
            ExpressionInfixOpcode::Mul => SymbolicValue::ConstantInt((lv * rv) % prime),
            ExpressionInfixOpcode::Pow => SymbolicValue::ConstantInt(modpow(lv, rv, prime)),
            ExpressionInfixOpcode::Div => {
                if lv.is_zero() || rv.is_zero() {
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
            ExpressionInfixOpcode::IntDiv => {
                SymbolicValue::ConstantInt(if lv.is_zero() || rv.is_zero() {
                    BigInt::zero()
                } else {
                    lv / rv
                })
            }
            ExpressionInfixOpcode::Mod => {
                SymbolicValue::ConstantInt(if lv.is_zero() || rv.is_zero() {
                    BigInt::zero()
                } else {
                    lv % rv
                })
            }
            ExpressionInfixOpcode::BitOr => SymbolicValue::ConstantInt(lv | rv),
            ExpressionInfixOpcode::BitAnd => SymbolicValue::ConstantInt(lv & rv),
            ExpressionInfixOpcode::BitXor => SymbolicValue::ConstantInt(lv ^ rv),
            ExpressionInfixOpcode::ShiftL => {
                SymbolicValue::ConstantInt(lv << rv.to_usize().unwrap())
            }
            ExpressionInfixOpcode::ShiftR => {
                SymbolicValue::ConstantInt(lv >> rv.to_usize().unwrap())
            }
            ExpressionInfixOpcode::Lesser => SymbolicValue::ConstantBool(
                val_for_relational_operators(&(lv % prime), prime)
                    < val_for_relational_operators(&(rv % prime), prime),
            ),
            ExpressionInfixOpcode::Greater => SymbolicValue::ConstantBool(
                val_for_relational_operators(&(lv % prime), prime)
                    > val_for_relational_operators(&(rv % prime), prime),
            ),
            ExpressionInfixOpcode::LesserEq => SymbolicValue::ConstantBool(
                val_for_relational_operators(&(lv % prime), prime)
                    <= val_for_relational_operators(&(rv % prime), prime),
            ),
            ExpressionInfixOpcode::GreaterEq => SymbolicValue::ConstantBool(
                val_for_relational_operators(&(lv % prime), prime)
                    >= val_for_relational_operators(&(rv % prime), prime),
            ),
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

pub fn evaluate_binary_op_integer_mode(
    lhs: &SymbolicValue,
    rhs: &SymbolicValue,
    prime: &BigInt,
    op: &DebuggableExpressionInfixOpcode,
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
        | ExpressionInfixOpcode::NotEq => {
            (normalize_to_int(lhs, prime), normalize_to_int(rhs, prime))
        }
        // Keep booleans as they are for logical operators
        ExpressionInfixOpcode::BoolAnd | ExpressionInfixOpcode::BoolOr => {
            (normalize_to_bool(lhs, prime), normalize_to_bool(rhs, prime))
        } //_ => (lhs.clone(), rhs.clone()), // Default case
    };

    match (&normalized_lhs, &normalized_rhs) {
        (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => match &op.0 {
            ExpressionInfixOpcode::Add => SymbolicValue::ConstantInt((lv + rv) % prime),
            ExpressionInfixOpcode::Sub => {
                let mut tmp = (lv - rv) % prime;
                if tmp.is_negative() {
                    tmp += prime;
                }
                SymbolicValue::ConstantInt(tmp)
            }
            ExpressionInfixOpcode::Mul => SymbolicValue::ConstantInt((lv * rv) % prime),
            ExpressionInfixOpcode::Pow => SymbolicValue::ConstantInt(modpow(lv, rv, prime)),
            ExpressionInfixOpcode::Div => {
                if lv.is_zero() || rv.is_zero() {
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
            ExpressionInfixOpcode::IntDiv => {
                SymbolicValue::ConstantInt(if lv.is_zero() || rv.is_zero() {
                    BigInt::zero()
                } else {
                    lv / rv
                })
            }
            ExpressionInfixOpcode::Mod => {
                SymbolicValue::ConstantInt(if lv.is_zero() || rv.is_zero() {
                    BigInt::zero()
                } else {
                    lv % rv
                })
            }
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

pub fn normalize_to_int(val: &SymbolicValue, prime: &BigInt) -> SymbolicValue {
    match val {
        SymbolicValue::ConstantBool(b) => {
            SymbolicValue::ConstantInt(if *b { BigInt::one() } else { BigInt::zero() })
        }
        SymbolicValue::ConstantInt(num) => {
            if num.is_negative() {
                SymbolicValue::ConstantInt(num + prime)
            } else {
                val.clone()
            }
        }
        _ => val.clone(),
    }
}

pub fn normalize_to_bool(val: &SymbolicValue, prime: &BigInt) -> SymbolicValue {
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
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
            lessthan_out.clone(),
        )),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::BoolAnd),
        Rc::new(SymbolicValue::AuxBinaryOp(
            in_0.clone(),
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Lesser),
            in_1.clone(),
        )),
    );
    let cond_0 = SymbolicValue::BinaryOp(
        Rc::new(SymbolicValue::BinaryOp(
            Rc::new(SymbolicValue::ConstantInt(BigInt::zero())),
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
            lessthan_out.clone(),
        )),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::BoolAnd),
        Rc::new(SymbolicValue::AuxBinaryOp(
            in_0,
            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::GreaterEq),
            in_1,
        )),
    );
    SymbolicValue::BinaryOp(
        Rc::new(cond_1),
        DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::BoolOr),
        Rc::new(cond_0),
    )
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

pub fn initialize_symbolic_nested_array_with_value(
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
            Rc::new(SymbolicValue::Array(
                initialize_symbolic_nested_array_with_value(&dims[1..], initial_value.clone())
            ));
            dims[0]
        ]
    }
}

pub fn update_nested_array(
    dims: &[usize],
    array: &SymbolicValueRef,
    value: &SymbolicValueRef,
) -> SymbolicValueRef {
    if let SymbolicValue::Array(arr) = (*array).as_ref() {
        let mut new_arr = arr.clone();
        if dims.len() == 1 {
            new_arr[dims[0]] = value.clone();
        } else {
            new_arr[dims[0]] = update_nested_array(&dims[1..], &arr[dims[0]], value);
        }
        Rc::new(SymbolicValue::Array(new_arr))
    } else {
        array.clone()
    }
}

/// Extracts all unique variable names referenced in a set of constraints.
///
/// # Parameters
/// - `constraints`: A slice of symbolic values representing the constraints.
///
/// # Returns
/// A vector of unique `SymbolicName`s referenced in the constraints.
pub fn extract_variables(constraints: &[SymbolicValueRef]) -> Vec<SymbolicName> {
    let mut variables = FxHashSet::default();
    for constraint in constraints {
        extract_variables_from_symbolic_value(constraint, &mut variables);
    }
    variables.into_iter().collect()
}

/// Recursively extracts variable names from a symbolic value.
///
/// # Parameters
/// - `value`: The `SymbolicValue` to analyze.
/// - `variables`: A mutable reference to a vector where extracted variable names will be stored.
pub fn extract_variables_from_symbolic_value(
    value: &SymbolicValue,
    variables: &mut FxHashSet<SymbolicName>,
) {
    match value {
        SymbolicValue::Variable(sym_name) => {
            variables.insert(sym_name.clone());
        }
        SymbolicValue::Assign(lhs, rhs, _, _)
        | SymbolicValue::AssignEq(lhs, rhs)
        | SymbolicValue::AssignTemplParam(lhs, rhs)
        | SymbolicValue::AssignCall(lhs, rhs, _) => {
            extract_variables_from_symbolic_value(&lhs, variables);
            extract_variables_from_symbolic_value(&rhs, variables);
        }
        SymbolicValue::BinaryOp(lhs, _, rhs) => {
            extract_variables_from_symbolic_value(&lhs, variables);
            extract_variables_from_symbolic_value(&rhs, variables);
        }
        SymbolicValue::UnaryOp(_, expr) => extract_variables_from_symbolic_value(&expr, variables),
        SymbolicValue::Array(elements) => {
            for elem in elements {
                extract_variables_from_symbolic_value(&elem, variables);
            }
        }
        SymbolicValue::UniformArray(value, size) => {
            extract_variables_from_symbolic_value(&value, variables);
            extract_variables_from_symbolic_value(&size, variables);
        }
        SymbolicValue::Call(_, args) => {
            for arg in args {
                extract_variables_from_symbolic_value(&arg, variables);
            }
        }
        SymbolicValue::Conditional(cond, then_val, else_val) => {
            extract_variables_from_symbolic_value(&cond, variables);
            extract_variables_from_symbolic_value(&then_val, variables);
            extract_variables_from_symbolic_value(&else_val, variables);
        }
        _ => {}
    }
}

pub fn get_coefficient_of_polynomials(
    expr: &SymbolicValue,
    target_name: &SymbolicName,
    prime: &BigInt,
) -> [SymbolicValueRef; 3] {
    match &expr {
        SymbolicValue::ConstantInt(_) => {
            let zero = Rc::new(SymbolicValue::ConstantInt(BigInt::zero()));
            [Rc::new(expr.clone()), zero.clone(), zero]
        }
        SymbolicValue::Variable(name) => {
            let zero = Rc::new(SymbolicValue::ConstantInt(BigInt::zero()));
            if name == target_name {
                let one = Rc::new(SymbolicValue::ConstantInt(BigInt::one()));
                [zero.clone(), one, zero]
            } else {
                [Rc::new(expr.clone()), zero.clone(), zero]
            }
        }
        SymbolicValue::BinaryOp(lhs, op, rhs) => match &op.0 {
            ExpressionInfixOpcode::Add => {
                let left = get_coefficient_of_polynomials(lhs, target_name, prime);
                let right = get_coefficient_of_polynomials(rhs, target_name, prime);
                [
                    Rc::new(evaluate_binary_op(
                        &left[0],
                        &right[0],
                        prime,
                        &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                    )),
                    Rc::new(evaluate_binary_op(
                        &left[1],
                        &right[1],
                        prime,
                        &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                    )),
                    Rc::new(evaluate_binary_op(
                        &left[2],
                        &right[2],
                        prime,
                        &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                    )),
                ]
            }
            ExpressionInfixOpcode::Sub => {
                let left = get_coefficient_of_polynomials(lhs, target_name, prime);
                let right = get_coefficient_of_polynomials(rhs, target_name, prime);
                [
                    Rc::new(evaluate_binary_op(
                        &left[0],
                        &right[0],
                        prime,
                        &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Sub),
                    )),
                    Rc::new(evaluate_binary_op(
                        &left[1],
                        &right[1],
                        prime,
                        &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Sub),
                    )),
                    Rc::new(evaluate_binary_op(
                        &left[2],
                        &right[2],
                        prime,
                        &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Sub),
                    )),
                ]
            }
            ExpressionInfixOpcode::Mul => {
                // (ax^2 + bx + c) * (dx^2 + ex + f) = adx^4 + aex^3 + afx^2 + bdx^3 + bex^2 + bfx + cdx^2 + cex + cf
                //                                   = adx^4 + (ae + bd)x^3 + (af + be + cd)x^2 + (bf + ce)x * cf

                let left = get_coefficient_of_polynomials(lhs, target_name, &prime);
                let right = get_coefficient_of_polynomials(rhs, target_name, &prime);
                let c0 = evaluate_binary_op(
                    &left[0],
                    &right[0],
                    &prime,
                    &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                );
                let c1 = evaluate_binary_op(
                    &left[0],
                    &right[1],
                    &prime,
                    &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                );
                let c2 = evaluate_binary_op(
                    &left[1],
                    &right[0],
                    &prime,
                    &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                );
                let c3 = evaluate_binary_op(
                    &left[2],
                    &right[0],
                    &prime,
                    &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                );
                let c4 = evaluate_binary_op(
                    &left[0],
                    &right[2],
                    &prime,
                    &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                );
                let c5 = evaluate_binary_op(
                    &left[1],
                    &right[1],
                    &prime,
                    &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Mul),
                );

                [
                    Rc::new(c0),
                    Rc::new(evaluate_binary_op(
                        &c1,
                        &c2,
                        prime,
                        &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                    )),
                    Rc::new(evaluate_binary_op(
                        &evaluate_binary_op(
                            &c3,
                            &c4,
                            prime,
                            &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                        ),
                        &c5,
                        prime,
                        &DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Add),
                    )),
                ]
            }
            _ => {
                let zero = Rc::new(SymbolicValue::ConstantInt(BigInt::zero()));
                [Rc::new(expr.clone()), zero.clone(), zero]
            }
        },
        _ => {
            let zero = Rc::new(SymbolicValue::ConstantInt(BigInt::zero()));
            [Rc::new(expr.clone()), zero.clone(), zero]
        }
    }
}

pub fn get_degree_polynomial(expr: &SymbolicValue, target_name: &SymbolicName) -> usize {
    match &expr {
        SymbolicValue::ConstantInt(_) => 0,
        SymbolicValue::Variable(name) => {
            if name == target_name {
                1
            } else {
                0
            }
        }
        SymbolicValue::BinaryOp(lhs, op, rhs) => match &op.0 {
            ExpressionInfixOpcode::Add | ExpressionInfixOpcode::Sub => std::cmp::max(
                get_degree_polynomial(lhs, target_name),
                get_degree_polynomial(rhs, target_name),
            ),
            ExpressionInfixOpcode::Mul => {
                get_degree_polynomial(lhs, target_name) + get_degree_polynomial(rhs, target_name)
            }
            _ => {
                if (get_degree_polynomial(lhs, target_name) != 0)
                    || (get_degree_polynomial(rhs, target_name) != 0)
                {
                    std::usize::MAX
                } else {
                    0
                }
            }
        },
        _ => {
            let mut tmp = FxHashSet::default();
            extract_variables_from_symbolic_value(expr, &mut tmp);
            if tmp.contains(target_name) {
                std::usize::MAX
            } else {
                0
            }
        }
    }
}
