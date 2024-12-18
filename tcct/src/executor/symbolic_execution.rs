use std::cmp::max;
use std::rc::Rc;

use colored::Colorize;
use log::{trace, warn};
use num_bigint_dig::BigInt;
use num_traits::cast::ToPrimitive;
use num_traits::FromPrimitive;
use num_traits::One;
use num_traits::Zero;
use rustc_hash::FxHashMap;

use program_structure::ast::{
    AssignOp, Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode, Meta, SignalType,
    VariableType,
};

use crate::executor::debug_ast::{
    DebugAccess, DebugAssignOp, DebugExpression, DebugExpressionInfixOpcode,
    DebugExpressionPrefixOpcode, DebugStatement, DebugVariableType,
};
use crate::executor::symbolic_value::{
    access_multidimensional_array, enumerate_array, evaluate_binary_op,
    generate_lessthan_constraint, is_true, negate_condition, register_array_elements, OwnerName,
    SymbolicAccess, SymbolicComponent, SymbolicLibrary, SymbolicName, SymbolicTemplate,
    SymbolicValue, SymbolicValueRef,
};
use crate::executor::utils::italic;

/// Represents the state of symbolic execution, holding symbolic values,
/// trace constraints, side constraints, and depth information.
#[derive(Clone)]
pub struct SymbolicState {
    pub owner_name: Rc<Vec<OwnerName>>,
    pub template_id: usize,
    pub is_within_initialization_block: bool,
    depth: usize,
    pub values: FxHashMap<SymbolicName, SymbolicValueRef>,
    pub trace_constraints: Vec<SymbolicValueRef>,
    pub side_constraints: Vec<SymbolicValueRef>,
}

impl SymbolicState {
    /// Creates a new `SymbolicState` with default values.
    ///
    /// # Returns
    ///
    /// A new instance of `SymbolicState` with empty fields.
    pub fn new() -> Self {
        SymbolicState {
            owner_name: Rc::new(Vec::new()),
            template_id: usize::MAX,
            is_within_initialization_block: false,
            depth: 0_usize,
            values: FxHashMap::default(),
            trace_constraints: Vec::new(),
            side_constraints: Vec::new(),
        }
    }

    /// Adds an owner to the current symbolic state.
    ///
    /// This method appends a new owner name to the existing list of owners.
    ///
    /// # Arguments
    ///
    /// * `oname` - The `OwnerName` to be added.
    pub fn add_owner(&mut self, oname: &OwnerName) {
        let mut updated_owner_list = (*self.owner_name.clone()).clone();
        updated_owner_list.push(oname.clone());
        self.owner_name = Rc::new(updated_owner_list);
    }

    /// Retrieves the full owner name as a string.
    ///
    /// This method joins all owner names in the current state using a dot separator.
    ///
    /// # Arguments
    ///
    /// * `name_lookup_map` - A hash map containing mappings from usize to String for name lookups.
    ///
    /// # Returns
    ///
    /// A string representing the full owner name.
    pub fn get_owner(&self, name_lookup_map: &FxHashMap<usize, String>) -> String {
        self.owner_name
            .iter()
            .map(|e: &OwnerName| {
                let access_str = if e.access.is_none() {
                    ""
                } else {
                    &e.access
                        .clone()
                        .unwrap()
                        .iter()
                        .map(|s: &SymbolicAccess| s.lookup_fmt(name_lookup_map))
                        .collect::<Vec<_>>()
                        .join("")
                };
                name_lookup_map[&e.name].clone() + &access_str
            })
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Sets the template ID for the current symbolic state.
    ///
    /// # Arguments
    ///
    /// * `name` - The usize value representing the template ID.
    pub fn set_template_id(&mut self, name: usize) {
        self.template_id = name;
    }

    /// Sets the current depth of the symbolic state.
    ///
    /// # Arguments
    ///
    /// * `d` - The depth level to set.
    pub fn set_depth(&mut self, depth_level: usize) {
        self.depth = depth_level;
    }

    /// Retrieves the current depth of the symbolic state.
    ///
    /// # Returns
    ///
    /// The depth as an unsigned integer.
    pub fn get_depth(&self) -> usize {
        self.depth
    }

    /// Sets a symbolic value for a given variable name in the state.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable.
    /// * `value` - The symbolic value to associate with the variable.
    pub fn set_symval(&mut self, name: SymbolicName, value: SymbolicValue) {
        self.values.insert(name, Rc::new(value));
    }

    /// Sets a reference-counted symbolic value for a given variable name in the state.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable.
    /// * `value` - The reference-counted symbolic value to associate with the variable.
    pub fn set_rc_symval(&mut self, name: SymbolicName, value: SymbolicValueRef) {
        self.values.insert(name, value);
    }

    /// Retrieves a symbolic value associated with a given variable name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable to retrieve.
    ///
    /// # Returns
    ///
    /// An optional reference to the symbolic value if it exists.
    pub fn get_symval(&self, name: &SymbolicName) -> Option<&SymbolicValueRef> {
        self.values.get(name)
    }

    /// Adds a trace constraint to the current state.
    ///
    /// # Arguments
    ///
    /// * `constraint` - The symbolic value representing the constraint.
    pub fn push_trace_constraint(&mut self, constraint: &SymbolicValue) {
        self.trace_constraints.push(Rc::new(constraint.clone()));
    }

    /// Adds a side constraint to the current state.
    ///
    /// # Arguments
    ///
    /// * `constraint` - The symbolic value representing the constraint.
    pub fn push_side_constraint(&mut self, constraint: &SymbolicValue) {
        self.side_constraints.push(Rc::new(constraint.clone()));
    }

    /// Formats the symbolic state for lookup and display.
    ///
    /// This method creates a string representation of the symbolic state,
    /// including owner, depth, values, trace constraints, and side constraints.
    ///
    /// # Arguments
    ///
    /// * `name_lookup_map` - A hash map containing mappings from usize to String for name lookups.
    ///
    /// # Returns
    ///
    /// A formatted string representation of the symbolic state.
    pub fn lookup_fmt(&self, name_lookup_map: &FxHashMap<usize, String>) -> String {
        let mut s = "".to_string();
        s += &format!("🛠️ {}", format!("{}", "SymbolicState [\n").cyan());
        s += &format!(
            "  {} {}\n",
            format!("👤 {}", "owner:").cyan(),
            italic(&format!("{:?}", &self.get_owner(name_lookup_map))).magenta()
        );
        s += &format!("  📏 {} {}\n", format!("{}", "depth:").cyan(), self.depth);
        s += &format!("  📋 {}\n", format!("{}", "values:").cyan());
        for (k, v) in self.values.iter() {
            s += &format!(
                "      {}: {}\n",
                k.lookup_fmt(name_lookup_map),
                format!("{}", v.lookup_fmt(name_lookup_map))
                    .replace("\n", "")
                    .replace("  ", " ")
            );
        }
        s += &format!(
            "  {} {}\n",
            format!("{}", "🪶 trace_constraints:").cyan(),
            format!(
                "{}",
                &self
                    .trace_constraints
                    .iter()
                    .map(|c| c.lookup_fmt(name_lookup_map))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .replace("\n", "")
            .replace("  ", " ")
            .replace("  ", " ")
        );
        s += &format!(
            "  {} {}\n",
            format!("{}", "⛓️ side_constraints:").cyan(),
            format!(
                "{}",
                &self
                    .side_constraints
                    .iter()
                    .map(|c| c.lookup_fmt(name_lookup_map))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .replace("\n", "")
            .replace("  ", " ")
            .replace("  ", " ")
        );
        s += &format!("{}\n", format!("{}", "]").cyan());
        s
    }
}

pub struct SymbolicStore {
    pub components_store: FxHashMap<SymbolicName, SymbolicComponent>,
    pub variable_types: FxHashMap<usize, DebugVariableType>,
    pub block_end_states: Vec<SymbolicState>,
    pub final_states: Vec<SymbolicState>,
    pub max_depth: usize,
}

impl SymbolicStore {
    pub fn clear(&mut self) {
        self.components_store.clear();
        self.block_end_states.clear();
        self.final_states.clear();
        self.max_depth = 0;
    }
}

#[derive(Clone)]
pub struct SymbolicExecutorSetting {
    pub prime: BigInt,
    pub propagate_substitution: bool,
    pub only_initialization_blocks: bool,
    pub skip_initialization_blocks: bool,
    pub off_trace: bool,
    pub keep_track_constraints: bool,
    pub substitute_output: bool,
}

/// A symbolic execution engine for analyzing and executing statements symbolically.
///
/// The `SymbolicExecutor` maintains multiple execution states, handles branching logic,
/// and updates constraints during execution flow. It is designed to work with a
/// `SymbolicLibrary` and a `SymbolicExecutorSetting`.
///
/// # Type Parameters
///
/// * `'a`: Lifetime for borrowing constraint statistics references.
///
/// # Fields
///
/// * `symbolic_library`: A mutable reference to the library storing templates for execution.
/// * `setting`: A reference to the execution settings.
/// * `symbolic_store`: A store for components, variable types, and execution states.
/// * `cur_state`: The current symbolic execution state.
pub struct SymbolicExecutor<'a> {
    pub symbolic_library: &'a mut SymbolicLibrary,
    pub setting: &'a SymbolicExecutorSetting,
    pub symbolic_store: SymbolicStore,
    pub cur_state: SymbolicState,
}

impl<'a> SymbolicExecutor<'a> {
    /// Creates a new instance of `SymbolicExecutor`.
    ///
    /// This method initializes all necessary states and statistics trackers.
    ///
    /// # Arguments
    ///
    /// * `symbolic_library` - A mutable reference to the `SymbolicLibrary`.
    /// * `setting` - A reference to the `SymbolicExecutorSetting`.
    ///
    /// # Returns
    ///
    /// A new instance of `SymbolicExecutor`.
    pub fn new(
        symbolic_library: &'a mut SymbolicLibrary,
        setting: &'a SymbolicExecutorSetting,
    ) -> Self {
        SymbolicExecutor {
            symbolic_library: symbolic_library,
            symbolic_store: SymbolicStore {
                components_store: FxHashMap::default(),
                variable_types: FxHashMap::default(),
                block_end_states: Vec::new(),
                final_states: Vec::new(),
                max_depth: 0,
            },
            cur_state: SymbolicState::new(),
            setting: setting,
        }
    }

    /// Clears the current state and resets the symbolic executor.
    ///
    /// This method resets the current state, clears the symbolic store,
    /// and resets the function counter in the symbolic library.
    pub fn clear(&mut self) {
        self.cur_state = SymbolicState::new();
        self.symbolic_store.clear();
        self.symbolic_library.clear_function_counter();
    }

    /// Feeds arguments into current state variables.
    ///
    /// This method evaluates the provided expressions and assigns their results
    /// to the corresponding variables in the current state.
    ///
    /// # Arguments
    ///
    /// * `names` - Vector containing names corresponding with expressions being fed as arguments.
    /// * `args` - Vector containing expressions whose evaluated results will be assigned as argument values.
    pub fn feed_arguments(&mut self, names: &Vec<String>, args: &Vec<Expression>) {
        let mut name2id = self.symbolic_library.name2id.clone();
        let mut id2name = self.symbolic_library.id2name.clone();
        for (n, a) in names.iter().zip(args.iter()) {
            let evaled_a = self.evaluate_expression(&DebugExpression::from(
                a.clone(),
                &mut name2id,
                &mut id2name,
            ));
            let simplified_a =
                self.simplify_variables(&evaled_a, !self.setting.propagate_substitution);
            self.cur_state.set_symval(
                SymbolicName {
                    name: name2id[n],
                    owner: self.cur_state.owner_name.clone(),
                    access: None,
                },
                simplified_a,
            );
        }
    }

    /// Executes a sequence of statements symbolically.
    ///
    /// This method starts execution from a specified block index, updating internal states
    /// and handling control structures like if-else and loops appropriately.
    ///
    /// # Arguments
    ///
    /// * `statements` - A vector of extended statements representing program logic to execute symbolically.
    /// * `cur_bid` - Current block index to start execution from.
    pub fn execute(&mut self, statements: &Vec<DebugStatement>, cur_bid: usize) {
        if cur_bid < statements.len() {
            self.symbolic_store.max_depth =
                max(self.symbolic_store.max_depth, self.cur_state.get_depth());

            if self.setting.only_initialization_blocks {
                match &statements[cur_bid] {
                    DebugStatement::InitializationBlock { .. } | DebugStatement::Block { .. } => {}
                    _ => {
                        if !self.cur_state.is_within_initialization_block {
                            self.execute(statements, cur_bid + 1);
                            return;
                        }
                    }
                }
            }

            match &statements[cur_bid] {
                DebugStatement::InitializationBlock { .. } => {
                    self.handle_initialization_block(statements, cur_bid);
                }
                DebugStatement::Block { meta, stmts, .. } => {
                    self.handle_block(statements, cur_bid);
                }
                DebugStatement::IfThenElse { .. } => {
                    self.handle_if_then_else(statements, cur_bid);
                }
                DebugStatement::While { .. } => {
                    self.handle_while(statements, cur_bid);
                }
                DebugStatement::Return { .. } => {
                    self.handle_return(statements, cur_bid);
                }
                DebugStatement::Declaration { .. } => {
                    self.handle_declaration(statements, cur_bid);
                }
                DebugStatement::Substitution { .. } => {
                    self.handle_substitution(statements, cur_bid);
                }
                DebugStatement::MultSubstitution { .. } => {
                    self.handle_multi_substitution(statements, cur_bid);
                }
                DebugStatement::ConstraintEquality { .. } => {
                    self.handle_constraint_equality(statements, cur_bid);
                }
                DebugStatement::Assert { .. } => {
                    self.handle_assert(statements, cur_bid);
                }
                DebugStatement::UnderscoreSubstitution {
                    meta,
                    op: _,
                    rhe: _,
                    ..
                } => {
                    self.trace_if_enabled(&meta);
                }
                DebugStatement::LogCall { meta, .. } => {
                    self.trace_if_enabled(&meta);
                }
                DebugStatement::Ret => {
                    self.handle_ret();
                }
            }
        } else {
            self.symbolic_store
                .block_end_states
                .push(self.cur_state.clone());
        }
    }

    /// Executes a symbolic expression concretely with given variable assignments.
    ///
    /// # Arguments
    ///
    /// * `expression` - The symbolic expression to execute.
    /// * `assignments` - A map of variable assignments for concrete execution.
    ///
    /// # Returns
    ///
    /// The result of the concrete execution as a `SymbolicValue`.
    pub fn concrete_execute(&mut self, id: &String, assignment: &FxHashMap<SymbolicName, BigInt>) {
        self.cur_state.template_id = self.symbolic_library.name2id[id];
        for (vname, value) in assignment.into_iter() {
            self.cur_state
                .set_symval(vname.clone(), SymbolicValue::ConstantInt(value.clone()));
        }

        self.execute(
            &self.symbolic_library.template_library[&self.cur_state.template_id]
                .body
                .clone(),
            0,
        );
    }
}

// State Expansion
impl<'a> SymbolicExecutor<'a> {
    /// Expands all stack states by executing each statement block recursively.
    ///
    /// This method updates depth and manages branching paths in execution flow.
    ///
    /// # Arguments
    ///
    /// * `statements` - A vector of extended statements to execute symbolically.
    /// * `cur_bid` - Current block index being executed.
    /// * `depth` - Current depth level in execution flow for tracking purposes.
    fn expand_all_stack_states(
        &mut self,
        statements: &Vec<DebugStatement>,
        cur_bid: usize,
        depth: usize,
    ) {
        let drained_states: Vec<_> = self.symbolic_store.block_end_states.drain(..).collect();
        for state in drained_states {
            self.cur_state = state;
            self.cur_state.set_depth(depth);
            self.execute(statements, cur_bid);
        }
    }
}

// Evaluation and simplification methods
impl<'a> SymbolicExecutor<'a> {
    /// Evaluates a symbolic access expression, converting it into a `SymbolicAccess` value.
    ///
    /// # Arguments
    ///
    /// * `access` - The `Access` to evaluate.
    ///
    /// # Returns
    ///
    /// A `SymbolicAccess` representing the evaluated access.
    fn evaluate_access(&mut self, access: &DebugAccess) -> SymbolicAccess {
        match &access {
            DebugAccess::ComponentAccess(name) => SymbolicAccess::ComponentAccess(name.clone()),
            DebugAccess::ArrayAccess(expr) => {
                let tmp_e = self.evaluate_expression(&expr);
                SymbolicAccess::ArrayAccess(self.simplify_variables(&tmp_e, false))
            }
        }
    }

    pub fn evaluate_dimension(&mut self, dims: &Vec<DebugExpression>) -> Vec<usize> {
        dims.iter()
            .map(|arg0: &DebugExpression| {
                let evaled_arg0 = self.evaluate_expression(arg0);
                let simplified_arg0 = self.simplify_variables(&evaled_arg0, false);
                if let SymbolicValue::ConstantInt(bint) = &simplified_arg0 {
                    bint.to_usize().unwrap()
                } else {
                    panic!(
                        "Undetermined dimension: {}",
                        simplified_arg0.lookup_fmt(&self.symbolic_library.id2name)
                    )
                }
            })
            .collect::<Vec<_>>()
    }

    /// Folds variables in a symbolic expression, potentially simplifying it.
    ///
    /// # Arguments
    ///
    /// * `expression` - The symbolic expression to fold.
    /// * `propagate` - A boolean flag indicating whether to propagate substitutions.
    ///
    /// # Returns
    ///
    /// A new `SymbolicValue` representing the simplified expression.
    fn simplify_variables(
        &self,
        symval: &SymbolicValue,
        only_constatant_folding: bool,
    ) -> SymbolicValue {
        match &symval {
            SymbolicValue::Variable(sname) => {
                if only_constatant_folding {
                    if let Some(template) = self
                        .symbolic_library
                        .template_library
                        .get(&self.cur_state.template_id)
                    {
                        if let Some(typ) = template.var2type.get(&sname.name) {
                            if let VariableType::Signal(SignalType::Output, _) = typ {
                                if self.setting.substitute_output {
                                    return (*self
                                        .cur_state
                                        .get_symval(&sname)
                                        .cloned()
                                        .unwrap_or_else(|| {
                                            Rc::new(SymbolicValue::Variable(sname.clone()))
                                        })
                                        .clone())
                                    .clone();
                                } else {
                                    return symval.clone();
                                }
                            } else if let VariableType::Var = typ {
                                return (*self
                                    .cur_state
                                    .get_symval(&sname)
                                    .cloned()
                                    .unwrap_or_else(|| {
                                        Rc::new(SymbolicValue::Variable(sname.clone()))
                                    })
                                    .clone())
                                .clone();
                            }
                        }
                    }
                    if let Some(SymbolicValue::ConstantInt(v)) =
                        self.cur_state.get_symval(&sname).map(|v| &**v)
                    {
                        return SymbolicValue::ConstantInt(v.clone());
                    }
                    symval.clone()
                } else {
                    (*self
                        .cur_state
                        .get_symval(&sname)
                        .cloned()
                        .unwrap_or_else(|| Rc::new(SymbolicValue::Variable(sname.clone())))
                        .clone())
                    .clone()
                }
            }
            SymbolicValue::BinaryOp(lv, infix_op, rv) => {
                let lhs = self.simplify_variables(lv, only_constatant_folding);
                let rhs = self.simplify_variables(rv, only_constatant_folding);
                evaluate_binary_op(&lhs, &rhs, &self.setting.prime, infix_op)
            }
            SymbolicValue::Conditional(cond, then_val, else_val) => {
                let simplified_cond = self.simplify_variables(cond, only_constatant_folding);
                match simplified_cond {
                    SymbolicValue::ConstantBool(true) => {
                        self.simplify_variables(then_val, only_constatant_folding)
                    }
                    SymbolicValue::ConstantBool(false) => {
                        self.simplify_variables(else_val, only_constatant_folding)
                    }
                    _ => SymbolicValue::Conditional(
                        Rc::new(self.simplify_variables(cond, only_constatant_folding)),
                        Rc::new(self.simplify_variables(then_val, only_constatant_folding)),
                        Rc::new(self.simplify_variables(else_val, only_constatant_folding)),
                    ),
                }
            }
            SymbolicValue::UnaryOp(prefix_op, value) => {
                let simplified_symval = self.simplify_variables(value, only_constatant_folding);
                match &simplified_symval {
                    SymbolicValue::ConstantInt(rv) => match prefix_op.0 {
                        ExpressionPrefixOpcode::Sub => SymbolicValue::ConstantInt(-1 * rv),
                        _ => SymbolicValue::UnaryOp(prefix_op.clone(), Rc::new(simplified_symval)),
                    },
                    SymbolicValue::ConstantBool(rv) => match prefix_op.0 {
                        ExpressionPrefixOpcode::BoolNot => SymbolicValue::ConstantBool(!rv),
                        _ => SymbolicValue::UnaryOp(prefix_op.clone(), Rc::new(simplified_symval)),
                    },
                    _ => SymbolicValue::UnaryOp(prefix_op.clone(), Rc::new(simplified_symval)),
                }
            }
            SymbolicValue::Array(elements) => SymbolicValue::Array(
                elements
                    .iter()
                    .map(|e| Rc::new(self.simplify_variables(e, only_constatant_folding)))
                    .collect(),
            ),
            SymbolicValue::Tuple(elements) => SymbolicValue::Tuple(
                elements
                    .iter()
                    .map(|e| Rc::new(self.simplify_variables(e, only_constatant_folding)))
                    .collect(),
            ),
            SymbolicValue::UniformArray(element, count) => SymbolicValue::UniformArray(
                Rc::new(self.simplify_variables(element, only_constatant_folding)),
                Rc::new(self.simplify_variables(count, only_constatant_folding)),
            ),
            SymbolicValue::Call(func_name, args) => SymbolicValue::Call(
                func_name.clone(),
                args.iter()
                    .map(|arg| Rc::new(self.simplify_variables(arg, only_constatant_folding)))
                    .collect(),
            ),
            _ => symval.clone(),
        }
    }

    /// Evaluates a symbolic expression, converting it into a `SymbolicValue`.
    ///
    /// This function handles various types of expressions, including constants, variables,
    /// and complex operations. It recursively evaluates sub-expressions as needed.
    ///
    /// # Arguments
    ///
    /// * `expr` - The `DebugExpression` to evaluate.
    ///
    /// # Returns
    ///
    /// A `SymbolicValue` representing the evaluated expression.
    fn evaluate_expression(&mut self, expr: &DebugExpression) -> SymbolicValue {
        match &expr {
            DebugExpression::Number(value) => SymbolicValue::ConstantInt(value.clone()),
            DebugExpression::Variable { name, access } => {
                let resolved_name = if access.is_empty() {
                    SymbolicName {
                        name: *name,
                        owner: self.cur_state.owner_name.clone(),
                        access: None,
                    }
                } else {
                    let tmp_name = SymbolicName {
                        name: *name,
                        owner: self.cur_state.owner_name.clone(),
                        access: None,
                    };
                    let sv = self.cur_state.get_symval(&tmp_name).cloned();

                    let mut component_name = None;
                    let mut dims = Vec::new();
                    for acc in access {
                        let evaled_access = self.evaluate_access(&acc.clone());
                        match evaled_access {
                            SymbolicAccess::ComponentAccess(tmp_name) => {
                                component_name = Some(tmp_name.clone());
                            }
                            SymbolicAccess::ArrayAccess(_) => {
                                dims.push(evaled_access);
                            }
                        }
                    }

                    if sv.is_some() && component_name.is_none() {
                        match (*sv.unwrap().clone()).clone() {
                            SymbolicValue::Array(values) => {
                                return access_multidimensional_array(&values, &dims);
                            }
                            _ => {}
                        }
                    }

                    self.construct_symbolic_name(*name, access).1
                };
                SymbolicValue::Variable(resolved_name)
            }
            DebugExpression::InfixOp { lhe, infix_op, rhe } => {
                let lhs = self.evaluate_expression(lhe);
                let rhs = self.evaluate_expression(rhe);
                SymbolicValue::BinaryOp(Rc::new(lhs), infix_op.clone(), Rc::new(rhs))
            }
            DebugExpression::PrefixOp { prefix_op, rhe } => {
                let expr = self.evaluate_expression(rhe);
                SymbolicValue::UnaryOp(prefix_op.clone(), Rc::new(expr))
            }
            DebugExpression::InlineSwitchOp {
                cond,
                if_true,
                if_false,
            } => {
                let condition = self.evaluate_expression(cond);
                let true_branch = self.evaluate_expression(if_true);
                let false_branch = self.evaluate_expression(if_false);
                SymbolicValue::Conditional(
                    Rc::new(condition),
                    Rc::new(true_branch),
                    Rc::new(false_branch),
                )
            }
            DebugExpression::ParallelOp { rhe, .. } => self.evaluate_expression(rhe),
            DebugExpression::ArrayInLine { values } => {
                let elements = values
                    .iter()
                    .map(|v| Rc::new(self.evaluate_expression(v)))
                    .collect();
                SymbolicValue::Array(elements)
            }
            DebugExpression::Tuple { values } => {
                let elements = values
                    .iter()
                    .map(|v| Rc::new(self.evaluate_expression(v)))
                    .collect();
                SymbolicValue::Array(elements)
            }
            DebugExpression::UniformArray {
                value, dimension, ..
            } => {
                let evaluated_value = self.evaluate_expression(value);
                let evaluated_dimension = self.evaluate_expression(dimension);
                SymbolicValue::UniformArray(Rc::new(evaluated_value), Rc::new(evaluated_dimension))
            }
            DebugExpression::Call { id, args, .. } => {
                let evaluated_args: Vec<_> = args
                    .iter()
                    .map(|arg| self.evaluate_expression(arg))
                    .collect();
                let simplified_args = evaluated_args
                    .iter()
                    .map(|arg| Rc::new(self.simplify_variables(&arg, false)))
                    .collect();
                if self.symbolic_library.template_library.contains_key(id) {
                    SymbolicValue::Call(id.clone(), simplified_args)
                } else if self.symbolic_library.function_library.contains_key(id) {
                    let symbolic_library = &mut self.symbolic_library;
                    let mut subse_setting = self.setting.clone();
                    subse_setting.only_initialization_blocks = false;
                    let mut subse = SymbolicExecutor::new(symbolic_library, &subse_setting);

                    let mut updated_owner_list = (*self.cur_state.owner_name.clone()).clone();
                    updated_owner_list.push(OwnerName {
                        name: *id,
                        counter: subse.symbolic_library.function_counter[id],
                        access: None,
                    });
                    subse.cur_state.owner_name = Rc::new(updated_owner_list);
                    subse
                        .symbolic_library
                        .function_counter
                        .insert(*id, subse.symbolic_library.function_counter[id] + 1);

                    let func = &subse.symbolic_library.function_library[id];
                    for i in 0..(func.function_argument_names.len()) {
                        let sname = SymbolicName {
                            name: func.function_argument_names[i],
                            owner: subse.cur_state.owner_name.clone(),
                            access: None,
                        };
                        subse
                            .cur_state
                            .set_rc_symval(sname, simplified_args[i].clone());
                    }

                    if !subse.setting.off_trace {
                        trace!("{}", format!("{}", "===========================").cyan());
                        trace!("📞 Call {}", subse.symbolic_library.id2name[id]);
                    }

                    subse.execute(&func.body.clone(), 0);

                    if !subse.setting.off_trace {
                        trace!("{}", format!("{}", "===========================").cyan());
                    }

                    if subse.symbolic_store.final_states.len() > 1 {
                        warn!("TODO: This tool currently cannot handle multiple branches within the callee.");
                    }

                    if !subse.symbolic_store.final_states.is_empty() {
                        // NOTE: a function does not produce any constraint
                        self.cur_state
                            .trace_constraints
                            .append(&mut subse.symbolic_store.final_states[0].trace_constraints);

                        let return_name = SymbolicName {
                            name: usize::MAX,
                            owner: subse.symbolic_store.final_states[0].owner_name.clone(),
                            access: None,
                        };
                        let return_value =
                            (*subse.symbolic_store.final_states[0].values[&return_name].clone())
                                .clone();
                        match return_value {
                            SymbolicValue::ConstantBool(_) | SymbolicValue::ConstantInt(_) => {
                                return_value
                            }
                            _ => SymbolicValue::Call(id.clone(), simplified_args),
                        }
                    } else {
                        panic!(
                            "{} did not return any final state",
                            subse.symbolic_library.id2name[id]
                        );
                    }
                } else {
                    panic!("Unknown Callee: {}", self.symbolic_library.id2name[id]);
                }
            }
            /*
            DebugExpression::BusCall { id, args, .. } => {
                let evaluated_args = args.iter()
                    .map(|arg| self.evaluate_expression(&DebugExpression(arg.clone())))
                    .collect();
                SymbolicValue::FunctionCall(format!("Bus_{}", id), evaluated_args)
            }
            DebugExpression::AnonymousComp { id, params, signals, .. } => {
                let evaluated_params = params.iter()
                    .map(|param| self.evaluate_expression(&DebugExpression(param.clone())))
                    .collect();
                let evaluated_signals = signals.iter()
                    .map(|signal| self.evaluate_expression(&DebugExpression(signal.clone())))
                    .collect();
                SymbolicValue::FunctionCall(format!("AnonymousComp_{}", id),
                    [evaluated_params, evaluated_signals].concat())
            }*/
            // Handle other expression types
            _ => {
                panic!("Unhandled expression type: {:?}", expr);
                //SymbolicValue::Variable(format!("Unhandled({:?})", expr), "".to_string())
            }
        }
    }
}

impl<'a> SymbolicExecutor<'a> {
    fn handle_initialization_block(&mut self, statements: &Vec<DebugStatement>, cur_bid: usize) {
        if let DebugStatement::InitializationBlock {
            initializations,
            xtype,
            ..
        } = &statements[cur_bid]
        {
            let is_input = matches!(xtype, VariableType::Signal(SignalType::Input, _));

            self.cur_state.is_within_initialization_block = true;

            // We do not need to initialize the inputs during concrete execution
            if !(self.setting.skip_initialization_blocks && is_input) {
                for init in initializations {
                    self.execute(&vec![init.clone()], 0);
                }
            }

            self.cur_state.is_within_initialization_block = false;
            self.symbolic_store.block_end_states = vec![self.cur_state.clone()];
            self.expand_all_stack_states(statements, cur_bid + 1, self.cur_state.get_depth());
        }
    }

    fn handle_block(&mut self, statements: &Vec<DebugStatement>, cur_bid: usize) {
        if let DebugStatement::Block { meta, stmts, .. } = &statements[cur_bid] {
            self.trace_if_enabled(&meta);
            self.execute(&stmts, 0);
            self.expand_all_stack_states(statements, cur_bid + 1, self.cur_state.get_depth());
        }
    }

    fn handle_if_then_else(&mut self, statements: &Vec<DebugStatement>, cur_bid: usize) {
        if let DebugStatement::IfThenElse {
            meta,
            cond,
            if_case,
            else_case,
            ..
        } = &statements[cur_bid]
        {
            self.trace_if_enabled(meta);

            let evaled_cond = self.evaluate_expression(cond);
            let original_condition = self.simplify_variables(&evaled_cond, true);
            let simplified_condition =
                self.simplify_variables(&evaled_cond, !self.setting.propagate_substitution);

            // Save the current state
            let saved_state = self.cur_state.clone();
            let current_depth = self.cur_state.get_depth();
            let saved_block_end_states = self.symbolic_store.block_end_states.clone();

            // Handle the 'then' branch
            self.process_branch(
                &simplified_condition,
                &original_condition,
                Some(if_case),
                statements,
                cur_bid,
                meta.elem_id,
                current_depth,
                true,
            );

            let mut then_branch_states = self.symbolic_store.block_end_states.clone();
            self.cur_state = saved_state;
            self.symbolic_store.block_end_states = saved_block_end_states;

            // Handle the 'else' branch
            let negated_condition = negate_condition(&simplified_condition);
            let original_negated_condition = negate_condition(&original_condition);

            self.process_branch(
                &negated_condition,
                &original_negated_condition,
                else_case.as_ref().map(|boxed| boxed.as_ref()),
                statements,
                cur_bid,
                meta.elem_id,
                current_depth,
                false,
            );

            // Merge the states from both branches
            self.symbolic_store
                .block_end_states
                .append(&mut then_branch_states);
        }
    }

    fn handle_substitution(&mut self, statements: &Vec<DebugStatement>, cur_bid: usize) {
        if let DebugStatement::Substitution {
            meta,
            var,
            access,
            op,
            rhe,
        } = &statements[cur_bid]
        {
            self.trace_if_enabled(meta);

            let evaled_rhe = self.evaluate_expression(rhe);
            let original_simplified_rhe = self.simplify_variables(&evaled_rhe, true);
            let simplified_rhe =
                self.simplify_variables(&evaled_rhe, !self.setting.propagate_substitution);
            let (left_base_name, left_var_name) = self.construct_symbolic_name(*var, access);

            match &simplified_rhe {
                SymbolicValue::Array(_) => {
                    self.handle_array_substitution(&left_var_name, &simplified_rhe);
                }
                _ => {
                    self.cur_state
                        .set_symval(left_var_name.clone(), simplified_rhe.clone());
                }
            }

            if let SymbolicValue::Call(callee_name, args) = &simplified_rhe {
                self.handle_call_substitution(callee_name, args, &left_var_name, &left_base_name);
            } else {
                self.handle_non_call_substitution(
                    op,
                    &left_var_name,
                    &simplified_rhe,
                    &original_simplified_rhe,
                );
            }

            if !access.is_empty() {
                self.handle_component_access(*var, access, &left_base_name, &simplified_rhe);
            }

            self.execute(statements, cur_bid + 1);
        }
    }

    fn handle_multi_substitution(&mut self, statements: &Vec<DebugStatement>, cur_bid: usize) {
        if let DebugStatement::MultSubstitution {
            meta, lhe, op, rhe, ..
        } = &statements[cur_bid]
        {
            self.trace_if_enabled(&meta);

            let lhe_val = self.evaluate_expression(lhe);
            let rhe_val = self.evaluate_expression(rhe);
            let simple_lhs = self.simplify_variables(&lhe_val, true);
            let lhs = self.simplify_variables(&lhe_val, !self.setting.propagate_substitution);
            let simple_rhs = self.simplify_variables(&rhe_val, true);
            let rhs = self.simplify_variables(&rhe_val, !self.setting.propagate_substitution);

            if self.setting.keep_track_constraints {
                match op {
                    DebugAssignOp(AssignOp::AssignConstraintSignal) => {
                        let cont = SymbolicValue::AssignEq(Rc::new(lhs), Rc::new(rhs));
                        self.cur_state.push_trace_constraint(&cont);

                        let simple_cont = SymbolicValue::BinaryOp(
                            Rc::new(simple_lhs),
                            DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                            Rc::new(simple_rhs),
                        );
                        self.cur_state.push_side_constraint(&simple_cont);
                    }
                    DebugAssignOp(AssignOp::AssignSignal) => {
                        let cont = SymbolicValue::Assign(Rc::new(lhs), Rc::new(rhs));
                        self.cur_state.push_trace_constraint(&cont);
                    }
                    _ => {}
                }
            }

            self.execute(statements, cur_bid + 1);
        }
    }

    fn handle_while(&mut self, statements: &Vec<DebugStatement>, cur_bid: usize) {
        if let DebugStatement::While {
            meta, cond, stmt, ..
        } = &statements[cur_bid]
        {
            self.trace_if_enabled(&meta);
            // Symbolic execution of loops is complex. This is a simplified approach.
            let tmp_cond = self.evaluate_expression(cond);
            let evaled_condition =
                self.simplify_variables(&tmp_cond, !self.setting.propagate_substitution);

            if let SymbolicValue::ConstantBool(flag) = evaled_condition {
                if flag {
                    let mut stack_states = self.symbolic_store.block_end_states.clone();
                    self.symbolic_store.block_end_states.clear();
                    self.execute(&vec![*stmt.clone()], 0);

                    self.expand_all_stack_states(statements, cur_bid, self.cur_state.get_depth());

                    self.symbolic_store
                        .block_end_states
                        .append(&mut stack_states);
                } else {
                    self.symbolic_store
                        .block_end_states
                        .push(self.cur_state.clone());

                    self.expand_all_stack_states(
                        statements,
                        cur_bid + 1,
                        self.cur_state.get_depth(),
                    );
                }
            } else {
                panic!(
                    "This tool currently cannot handle the symbolic condition of While Loop: {}",
                    evaled_condition.lookup_fmt(&self.symbolic_library.id2name)
                );
            }
        }
    }

    fn handle_return(&mut self, statements: &Vec<DebugStatement>, cur_bid: usize) {
        if let DebugStatement::Return { meta, value, .. } = &statements[cur_bid] {
            self.trace_if_enabled(&meta);
            let tmp_val = self.evaluate_expression(value);
            let return_value =
                self.simplify_variables(&tmp_val, !self.setting.propagate_substitution);

            // Handle return value (e.g., store in a special "return" variable)
            if !self.symbolic_library.id2name.contains_key(&usize::MAX) {
                self.symbolic_library
                    .name2id
                    .insert("__return__".to_string(), usize::MAX);
                self.symbolic_library
                    .id2name
                    .insert(usize::MAX, "__return__".to_string());
            }

            self.cur_state.set_symval(
                SymbolicName {
                    name: usize::MAX,
                    owner: self.cur_state.owner_name.clone(),
                    access: None,
                },
                return_value,
            );
            self.execute(statements, cur_bid + 1);
        }
    }

    fn handle_declaration(&mut self, statements: &Vec<DebugStatement>, cur_bid: usize) {
        if let DebugStatement::Declaration { name, xtype, .. } = &statements[cur_bid] {
            let var_name = SymbolicName {
                name: *name,
                owner: self.cur_state.owner_name.clone(),
                access: None,
            };
            self.symbolic_store
                .variable_types
                .insert(*name, DebugVariableType(xtype.clone()));
            let value = SymbolicValue::Variable(var_name.clone());
            self.cur_state.set_symval(var_name, value);
            self.execute(statements, cur_bid + 1);
        }
    }

    fn handle_constraint_equality(&mut self, statements: &Vec<DebugStatement>, cur_bid: usize) {
        if let DebugStatement::ConstraintEquality { meta, lhe, rhe } = &statements[cur_bid] {
            self.trace_if_enabled(&meta);

            let lhe_val = self.evaluate_expression(lhe);
            let rhe_val = self.evaluate_expression(rhe);
            let original_lhs = self.simplify_variables(&lhe_val, true);
            let lhs = self.simplify_variables(&lhe_val, !self.setting.propagate_substitution);
            let original_rhs = self.simplify_variables(&rhe_val, true);
            let rhs = self.simplify_variables(&rhe_val, !self.setting.propagate_substitution);

            let original_cond = SymbolicValue::BinaryOp(
                Rc::new(original_lhs),
                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                Rc::new(original_rhs),
            );
            let cond = SymbolicValue::BinaryOp(
                Rc::new(lhs),
                DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                Rc::new(rhs),
            );

            if self.setting.keep_track_constraints {
                self.cur_state.push_trace_constraint(&cond);
                self.cur_state.push_side_constraint(&original_cond);
            }
            self.execute(statements, cur_bid + 1);
        }
    }

    fn handle_assert(&mut self, statements: &Vec<DebugStatement>, cur_bid: usize) {
        if let DebugStatement::Assert { meta, arg, .. } = &statements[cur_bid] {
            self.trace_if_enabled(&meta);
            let expr = self.evaluate_expression(&arg);
            let condition = self.simplify_variables(&expr, !self.setting.propagate_substitution);
            if self.setting.keep_track_constraints {
                self.cur_state.push_trace_constraint(&condition);
            }
            self.execute(statements, cur_bid + 1);
        }
    }

    fn handle_ret(&mut self) {
        if !self.setting.off_trace {
            trace!(
                "{} {}",
                format!("{}", "🔙 Ret:").red(),
                self.cur_state.lookup_fmt(&self.symbolic_library.id2name)
            );
        }
        self.symbolic_store
            .final_states
            .push(self.cur_state.clone());
    }
}

// Utility methods for substitution
impl<'a> SymbolicExecutor<'a> {
    /// Handles array substitution in symbolic execution.
    ///
    /// This method processes the assignment of array values, updating the symbolic state
    /// for each element of the array individually.
    ///
    /// # Arguments
    ///
    /// * `left_var_name` - The symbolic name of the array variable being assigned.
    /// * `elements` - A vector of reference-counted symbolic values representing the array elements.
    ///
    /// # Side Effects
    ///
    /// Updates the current symbolic state with individual array element assignments.
    fn handle_array_substitution(&mut self, left_var_name: &SymbolicName, arr: &SymbolicValue) {
        let enumerated_elements = enumerate_array(arr);
        for (pos, elem) in enumerated_elements {
            let mut new_left_var_name = left_var_name.clone();
            let mut access = new_left_var_name.access.unwrap_or_default();
            for p in pos {
                access.push(SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(
                    BigInt::from_usize(p).unwrap(),
                )));
            }
            new_left_var_name.access = Some(access);
            self.cur_state.set_symval(new_left_var_name, elem.clone());
        }
    }

    /// Handles call substitution in symbolic execution.
    ///
    /// This method processes the assignment of a template call result,
    /// potentially initializing a new component in the symbolic store.
    ///
    /// # Arguments
    ///
    /// * `callee_name` - The name of the called function or template.
    /// * `args` - The arguments passed to the call.
    /// * `var_name` - The symbolic name where the call result is being assigned.
    /// * `base_name` - The base symbolic name for component initialization.
    ///
    /// # Side Effects
    ///
    /// May initialize a new component in the symbolic store or update
    fn handle_call_substitution(
        &mut self,
        callee_name: &usize,
        args: &Vec<Rc<SymbolicValue>>,
        var_name: &SymbolicName,
        base_name: &SymbolicName,
    ) {
        if self
            .symbolic_library
            .template_library
            .contains_key(callee_name)
        {
            self.initialize_template_component(callee_name, args, var_name, base_name);
        }
    }

    fn initialize_template_component(
        &mut self,
        callee_name: &usize,
        args: &Vec<Rc<SymbolicValue>>,
        var_name: &SymbolicName,
        base_name: &SymbolicName,
    ) {
        let mut subse_setting = self.setting.clone();
        subse_setting.only_initialization_blocks = true;
        subse_setting.off_trace = true;
        let mut se_for_initialization =
            SymbolicExecutor::new(&mut self.symbolic_library, &subse_setting);
        se_for_initialization.cur_state.owner_name = self.cur_state.owner_name.clone();

        let template = se_for_initialization.symbolic_library.template_library[callee_name].clone();
        let mut escaped_vars = Vec::new();

        // Set template parameters
        for i in 0..template.template_parameter_names.len() {
            let tp_name = SymbolicName {
                name: template.template_parameter_names[i],
                owner: self.cur_state.owner_name.clone(),
                access: None,
            };
            if let Some(val) = self.cur_state.get_symval(&tp_name) {
                escaped_vars.push((tp_name.clone(), val.clone()));
            }
            self.cur_state
                .set_rc_symval(tp_name.clone(), args[i].clone());
            se_for_initialization
                .cur_state
                .set_rc_symval(tp_name, args[i].clone());
        }

        se_for_initialization.execute(&template.body.clone(), 0);

        let mut inputs_of_component = FxHashMap::default();

        se_for_initialization.initialize_template_inputs(&template, &mut inputs_of_component);

        self.restore_escaped_variables(&escaped_vars);

        let component = SymbolicComponent {
            template_name: *callee_name,
            args: args.clone(),
            inputs: inputs_of_component,
            is_done: false,
        };
        self.symbolic_store
            .components_store
            .insert(var_name.clone(), component);
    }

    fn initialize_template_inputs(
        &mut self,
        template: &SymbolicTemplate,
        inputs_of_component: &mut FxHashMap<SymbolicName, Option<SymbolicValue>>,
    ) {
        for inp_name in &template.inputs {
            let dims = self.evaluate_dimension(&template.input_dimensions[inp_name]);
            register_array_elements(*inp_name, &dims, None, inputs_of_component);
        }
    }

    fn restore_escaped_variables(&mut self, escaped_vars: &Vec<(SymbolicName, SymbolicValueRef)>) {
        for (n, v) in escaped_vars {
            self.cur_state.set_rc_symval(n.clone(), v.clone());
        }
    }

    /// Handles non-call substitution in symbolic execution.
    ///
    /// This method processes standard variable assignments, updating the symbolic state
    /// and potentially adding constraints based on the assignment type.
    ///
    /// # Arguments
    ///
    /// * `op` - The assignment operator.
    /// * `var_name` - The symbolic name of the variable being assigned.
    /// * `value` - The symbolic value being assigned.
    /// * `original_value` - The original (non-simplified) value being assigned.
    ///
    /// # Side Effects
    ///
    /// Updates the current symbolic state and may add constraints.
    fn handle_non_call_substitution(
        &mut self,
        op: &DebugAssignOp,
        var_name: &SymbolicName,
        value: &SymbolicValue,
        original_value: &SymbolicValue,
    ) {
        if self.setting.keep_track_constraints {
            match op {
                DebugAssignOp(AssignOp::AssignConstraintSignal) => {
                    let cont = SymbolicValue::AssignEq(
                        Rc::new(SymbolicValue::Variable(var_name.clone())),
                        Rc::new(value.clone()),
                    );
                    self.cur_state.push_trace_constraint(&cont);
                    let original_cont = SymbolicValue::BinaryOp(
                        Rc::new(SymbolicValue::Variable(var_name.clone())),
                        DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                        Rc::new(original_value.clone()),
                    );
                    self.cur_state.push_side_constraint(&original_cont);
                }
                DebugAssignOp(AssignOp::AssignSignal) => {
                    let cont = SymbolicValue::Assign(
                        Rc::new(SymbolicValue::Variable(var_name.clone())),
                        Rc::new(value.clone()),
                    );
                    self.cur_state.push_trace_constraint(&cont);
                }
                _ => {}
            }
        }
    }

    /// Handles component access during substitution.
    ///
    /// This method processes assignments involving component access, updating
    /// the component store and potentially executing ready components.
    ///
    /// # Arguments
    ///
    /// * `var` - The variable identifier.
    /// * `access` - A vector of accesses (e.g., array indices, component accesses).
    /// * `base_name` - The base symbolic name for the accessed component.
    /// * `value` - The symbolic value being assigned.
    ///
    /// # Side Effects
    ///
    /// Updates the component store and may trigger execution of ready components.
    fn handle_component_access(
        &mut self,
        var: usize,
        access: &Vec<DebugAccess>,
        base_name: &SymbolicName,
        value: &SymbolicValue,
    ) {
        let (component_name, pre_dims, post_dims) = self.parse_component_access(access);
        let inp_name = SymbolicName {
            name: component_name,
            owner: Rc::new(Vec::new()),
            access: if post_dims.is_empty() {
                None
            } else {
                Some(post_dims)
            },
        };

        if let Some(component) = self.symbolic_store.components_store.get_mut(base_name) {
            component.inputs.insert(inp_name, Some(value.clone()));
        }

        if self.is_ready(base_name) {
            self.execute_ready_component(var, base_name, &pre_dims);
        }
    }

    fn parse_component_access(
        &mut self,
        access: &Vec<DebugAccess>,
    ) -> (usize, Vec<SymbolicAccess>, Vec<SymbolicAccess>) {
        let mut component_name = 0;
        let mut pre_dims = Vec::new();
        let mut post_dims = Vec::new();
        let mut found_component = false;

        for acc in access {
            let evaled_access = self.evaluate_access(acc);
            match evaled_access {
                SymbolicAccess::ComponentAccess(tmp_name) => {
                    found_component = true;
                    component_name = tmp_name;
                }
                SymbolicAccess::ArrayAccess(_) => {
                    if found_component {
                        post_dims.push(evaled_access);
                    } else {
                        pre_dims.push(evaled_access);
                    }
                }
            }
        }

        (component_name, pre_dims, post_dims)
    }

    /// Checks if a component is ready based on its inputs being fully specified.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the component to check readiness for.
    ///
    /// # Returns
    ///
    /// A boolean indicating readiness status.
    fn is_ready(&self, name: &SymbolicName) -> bool {
        self.symbolic_store.components_store.contains_key(name)
            && self.symbolic_store.components_store[name]
                .inputs
                .iter()
                .all(|(_, v)| v.is_some())
    }

    fn execute_ready_component(
        &mut self,
        var: usize,
        base_name: &SymbolicName,
        pre_dims: &Vec<SymbolicAccess>,
    ) {
        if !self.symbolic_store.components_store[base_name].is_done {
            let mut subse = SymbolicExecutor::new(&mut self.symbolic_library, self.setting);
            let mut updated_owner_list = (*self.cur_state.owner_name.clone()).clone();
            updated_owner_list.push(OwnerName {
                name: var,
                counter: 0,
                access: if pre_dims.is_empty() {
                    None
                } else {
                    Some(pre_dims.clone())
                },
            });
            subse.cur_state.owner_name = Rc::new(updated_owner_list);

            let templ = &subse.symbolic_library.template_library
                [&self.symbolic_store.components_store[base_name].template_name];
            subse
                .cur_state
                .set_template_id(self.symbolic_store.components_store[base_name].template_name);

            // Set template-parameters of the component
            for i in 0..templ.template_parameter_names.len() {
                let tp_name = SymbolicName {
                    name: templ.template_parameter_names[i],
                    owner: subse.cur_state.owner_name.clone(),
                    access: None,
                };
                subse.cur_state.set_rc_symval(
                    tp_name,
                    self.symbolic_store.components_store[base_name].args[i].clone(),
                );
            }

            // Set inputs of the component
            for (k, v) in self.symbolic_store.components_store[base_name]
                .inputs
                .iter()
            {
                let n = SymbolicName {
                    name: k.name,
                    owner: subse.cur_state.owner_name.clone(),
                    access: k.access.clone(),
                };
                subse.cur_state.set_symval(n, v.clone().unwrap());
            }

            if !self.setting.off_trace {
                trace!("{}", "===========================".cyan());
                trace!(
                    "📞 Call {}",
                    subse.symbolic_library.id2name
                        [&self.symbolic_store.components_store[base_name].template_name]
                );
            }

            let is_lessthan = templ.is_lessthan;
            subse.execute(&templ.body.clone(), 0);

            // Handle the execution results
            if subse.symbolic_store.final_states.len() > 1 {
                warn!(
                    "TODO: This tool currently cannot handle multiple branches within the callee."
                );
            }
            self.cur_state
                .trace_constraints
                .append(&mut subse.symbolic_store.final_states[0].trace_constraints);
            self.cur_state
                .side_constraints
                .append(&mut subse.symbolic_store.final_states[0].side_constraints);

            if is_lessthan {
                let cond = generate_lessthan_constraint(
                    &subse.symbolic_library.name2id,
                    subse.cur_state.owner_name.clone(),
                );
                self.cur_state.push_trace_constraint(&cond);
            }

            if !self.setting.off_trace {
                trace!("{}", "===========================".cyan());
            }
        }
    }
}

// Utility methods for If-Then-Else
impl<'a> SymbolicExecutor<'a> {
    fn process_branch(
        &mut self,
        condition: &SymbolicValue,
        original_condition: &SymbolicValue,
        branch_case: Option<&DebugStatement>,
        statements: &Vec<DebugStatement>,
        cur_bid: usize,
        meta_elem_id: usize,
        current_depth: usize,
        is_then_branch: bool,
    ) {
        if let SymbolicValue::ConstantBool(false) = condition {
            if !self.setting.off_trace {
                let branch_name = if is_then_branch { "Then" } else { "Else" };
                trace!(
                    "{}",
                    format!(
                        "(elem_id={}) 🚧 Unreachable `{}` Branch",
                        meta_elem_id, branch_name
                    )
                    .yellow()
                );
            }
            return;
        }

        let mut branch_state = self.cur_state.clone();

        if self.setting.keep_track_constraints && !is_true(condition) {
            branch_state.push_trace_constraint(condition);
            branch_state.push_side_constraint(original_condition);
        }

        branch_state.set_depth(current_depth + 1);
        self.cur_state = branch_state;

        if let Some(stmt) = branch_case {
            self.execute(&vec![stmt.clone()], 0);
        } else {
            self.symbolic_store.block_end_states = vec![self.cur_state.clone()];
        }

        self.expand_all_stack_states(statements, cur_bid + 1, current_depth);
    }
}

// Other utility methods
impl<'a> SymbolicExecutor<'a> {
    /// Traces the current state if tracing is enabled.
    ///
    /// This method logs the current state information if tracing is not disabled.
    ///
    /// # Arguments
    ///
    /// * `meta` - The metadata associated with the current execution point.
    fn trace_if_enabled(&self, meta: &Meta) {
        if !self.setting.off_trace {
            trace!(
                "(elem_id={}) {}",
                meta.elem_id,
                self.cur_state.lookup_fmt(&self.symbolic_library.id2name)
            );
        }
    }

    /// Constructs symbolic names for a given base ID and access pattern.
    ///
    /// This function parses a sequence of accesses to create symbolic names
    /// representing the accessed component or variable. It handles both simple
    /// variable access and complex component access patterns.
    ///
    /// # Arguments
    ///
    /// * `base_id` - The base identifier for the variable or component.
    /// * `access` - A vector of `DebugAccess` representing the access pattern.
    ///
    /// # Returns
    ///
    /// A tuple of two `SymbolicName`s:
    /// * The first represents the base variable or component.
    /// * The second represents the fully resolved name, including component access if present.
    ///
    /// # Examples
    ///
    /// Suppose the current owner_name is `[m]`.
    /// For a simple variable access like `x[0]`:
    /// * Returns `(SymbolicName{name: x, owner: [m], access: [0]}, SymbolicName{name: x, owner: [m], access: [0]})`
    ///
    /// For a component access like `x[0].y[1]`:
    /// * Returns `(SymbolicName{name: x, owner: [m], access: [0]}, SymbolicName{name: y, owner: [m, x[0]], access: [1]})`
    ///
    /// # Notes
    ///
    /// * The function distinguishes between array accesses before and after a component access.
    /// * If no component access is found, both returned `SymbolicName`s will be based on the `base_id`.
    /// * The owner of the returned `SymbolicName`s is set based on the current state's owner name.
    fn construct_symbolic_name(
        &mut self,
        base_id: usize,
        access: &Vec<DebugAccess>,
    ) -> (SymbolicName, SymbolicName) {
        // Style of component access: owner[access].component[access]
        // Example: bits[0].dblIn[0];
        let mut pre_dims = Vec::new();
        let mut component_name = None;
        let mut post_dims = Vec::new();
        let mut found_component = false;
        for acc in access {
            let evaled_access = self.evaluate_access(&acc.clone());
            match evaled_access {
                SymbolicAccess::ComponentAccess(tmp_name) => {
                    found_component = true;
                    component_name = Some(tmp_name.clone());
                }
                SymbolicAccess::ArrayAccess(_) => {
                    if found_component {
                        post_dims.push(evaled_access);
                    } else {
                        pre_dims.push(evaled_access);
                    }
                }
            }
        }

        if component_name.is_none() {
            (
                SymbolicName {
                    name: base_id,
                    owner: self.cur_state.owner_name.clone(),
                    access: if pre_dims.is_empty() {
                        None
                    } else {
                        Some(pre_dims.clone())
                    },
                },
                SymbolicName {
                    name: base_id,
                    owner: self.cur_state.owner_name.clone(),
                    access: if pre_dims.is_empty() {
                        None
                    } else {
                        Some(pre_dims)
                    },
                },
            )
        } else {
            let mut owner_name = (*self.cur_state.owner_name.clone()).clone();
            owner_name.push(OwnerName {
                name: base_id,
                counter: 0,
                access: if pre_dims.is_empty() {
                    None
                } else {
                    Some(pre_dims.clone())
                },
            });
            (
                SymbolicName {
                    name: base_id,
                    owner: self.cur_state.owner_name.clone(),
                    access: if pre_dims.is_empty() {
                        None
                    } else {
                        Some(pre_dims)
                    },
                },
                SymbolicName {
                    name: component_name.unwrap(),
                    owner: Rc::new(owner_name),
                    access: if post_dims.is_empty() {
                        None
                    } else {
                        Some(post_dims)
                    },
                },
            )
        }
    }
}
