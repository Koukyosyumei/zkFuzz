use core::panic;
use std::cmp::max;
use std::rc::Rc;

use colored::Colorize;
use log::trace;
use num_bigint_dig::BigInt;
use num_traits::cast::ToPrimitive;
use num_traits::FromPrimitive;
use rustc_hash::{FxHashMap, FxHashSet};

use program_structure::ast::{
    AssignOp, Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode, Meta, SignalType,
    VariableType,
};

use crate::executor::coverage::CoverageTracker;
use crate::executor::debug_ast::{
    DebugAccess, DebuggableAssignOp, DebuggableExpression, DebuggableExpressionInfixOpcode,
    DebuggableStatement, DebuggableVariableType,
};
use crate::executor::symbolic_setting::SymbolicExecutorSetting;
use crate::executor::symbolic_state::SymbolicState;
use crate::executor::symbolic_value::{
    access_multidimensional_array, decompose_uniform_array, enumerate_array, evaluate_binary_op,
    extract_variables_from_symbolic_value, generate_lessthan_constraint,
    get_coefficient_of_polynomials, get_degree_polynomial,
    initialize_symbolic_nested_array_with_value, is_concrete_array, register_array_elements,
    update_nested_array, OwnerName, SymbolicAccess, SymbolicComponent, SymbolicLibrary,
    SymbolicName, SymbolicTemplate, SymbolicValue, SymbolicValueRef,
};
use crate::executor::utils::generate_cartesian_product_indices;

use super::symbolic_value::ExecutionResult;

pub struct SymbolicStore {
    pub components_store: FxHashMap<SymbolicName, SymbolicComponent>,
    pub variable_types: FxHashMap<usize, DebuggableVariableType>,
    pub max_depth: usize,
}

impl SymbolicStore {
    pub fn clear(&mut self) {
        self.components_store.clear();
        self.max_depth = 0;
    }
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
/// * `symbolic_library` – A mutable reference to the symbolic library containing execution templates.
/// * `setting` – A reference to execution settings that configure runtime behavior.
/// * `symbolic_store` – A structured store maintaining components, variable types, and execution states.
/// * `cur_state` – The current symbolic execution state being processed.
/// * `violated_condition` – An optional tuple indicating a violated condition (index and symbolic value).
/// * `id2dimensions` – A mapping from identifiers to their respective dimensional constraints.
/// * `mindim` – The minimum dimensionality constraint applied during execution.
/// * `execution_failed` – A flag indicating whether execution has encountered a failure.
/// * `coverage_tracker` – An internal tracker for execution path coverage (enabled when configured).
/// * `enable_coverage_tracking` – A flag controlling whether execution path coverage is tracked.
/// * `is_concrete_mode` – A flag indicating whether the engine is running in concrete execution mode.
pub struct SymbolicExecutor<'a> {
    pub symbolic_library: &'a mut SymbolicLibrary,
    pub setting: &'a SymbolicExecutorSetting,
    pub symbolic_store: SymbolicStore,
    pub cur_state: SymbolicState,
    pub violated_condition: Option<(usize, SymbolicValue)>,
    pub id2dimensions: FxHashMap<usize, Vec<usize>>,
    pub mindim: usize,
    pub execution_failed: bool,
    coverage_tracker: CoverageTracker,
    enable_coverage_tracking: bool,
    is_concrete_mode: bool,
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
                max_depth: 0,
            },
            cur_state: SymbolicState::new(),
            violated_condition: None,
            id2dimensions: FxHashMap::default(),
            mindim: std::usize::MAX,
            execution_failed: false,
            coverage_tracker: CoverageTracker::new(),
            setting: setting,
            enable_coverage_tracking: false,
            is_concrete_mode: false,
        }
    }

    pub fn turn_on_coverage_tracking(&mut self) {
        self.enable_coverage_tracking = true;
    }

    pub fn turn_off_coverage_tracking(&mut self) {
        self.enable_coverage_tracking = false;
    }

    pub fn record_path(&mut self) {
        self.coverage_tracker.record_path();
    }

    pub fn coverage_count(&self) -> usize {
        self.coverage_tracker.coverage_count()
    }

    pub fn clear_coverage_tracker(&mut self) {
        self.coverage_tracker.clear();
    }

    /// Clears the current state and resets the symbolic executor.
    ///
    /// This method resets the current state, clears the symbolic store,
    /// and resets the function counter in the symbolic library.
    pub fn clear(&mut self) {
        self.cur_state = SymbolicState::new();
        self.symbolic_store.clear();
        self.symbolic_library.clear_function_counter();
        self.coverage_tracker.clear_current_path();
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
            let evaled_a = self.evaluate_expression(
                &DebuggableExpression::from(a.clone(), &mut name2id, &mut id2name),
                usize::MAX,
            );
            let mut memo = FxHashSet::default();
            let simplified_a =
                self.simplify_variables(&evaled_a, usize::MAX, true, false, &mut memo);
            let sym_name = SymbolicName::new(name2id[n], self.cur_state.owner_name.clone(), None);
            let cond = SymbolicValue::AssignTemplParam(
                Rc::new(SymbolicValue::Variable(sym_name.clone())),
                Rc::new(simplified_a.clone()),
            );
            self.cur_state.set_sym_val(sym_name, simplified_a);
            if self.setting.keep_track_constraints {
                self.cur_state.push_symbolic_trace(&cond);
                self.cur_state.push_side_constraint(&cond);
            }
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
    pub fn execute(&mut self, statements: &Vec<DebuggableStatement>, cur_bid: usize) {
        if cur_bid < statements.len() {
            self.symbolic_store.max_depth =
                max(self.symbolic_store.max_depth, self.cur_state.get_depth());

            if self.setting.only_initialization_blocks {
                match &statements[cur_bid] {
                    DebuggableStatement::InitializationBlock { .. }
                    | DebuggableStatement::Block { .. } => {}
                    _ => {
                        if !self.cur_state.is_within_initialization_block {
                            self.execute(statements, cur_bid + 1);
                            return;
                        }
                    }
                }
            }

            match &statements[cur_bid] {
                DebuggableStatement::InitializationBlock { .. } => {
                    self.handle_initialization_block(statements, cur_bid);
                }
                DebuggableStatement::Block { .. } => {
                    self.handle_block(statements, cur_bid);
                }
                DebuggableStatement::IfThenElse { .. } => {
                    self.handle_if_then_else(statements, cur_bid);
                }
                DebuggableStatement::While { .. } => {
                    self.handle_while(statements, cur_bid);
                }
                DebuggableStatement::Return { .. } => {
                    self.handle_return(statements, cur_bid);
                }
                DebuggableStatement::Declaration { meta, .. } => {
                    self.handle_declaration(statements, cur_bid, meta.elem_id);
                }
                DebuggableStatement::Substitution { .. } => {
                    self.handle_substitution(statements, cur_bid);
                }
                DebuggableStatement::MultSubstitution { .. } => {
                    self.handle_multi_substitution(statements, cur_bid);
                }
                DebuggableStatement::ConstraintEquality { .. } => {
                    self.handle_constraint_equality(statements, cur_bid);
                }
                DebuggableStatement::Assert { .. } => {
                    self.handle_assert(statements, cur_bid);
                }
                DebuggableStatement::UnderscoreSubstitution {
                    meta,
                    op: _,
                    rhe: _,
                    ..
                } => {
                    self.trace_if_enabled(&meta);
                    self.execute(statements, cur_bid + 1);
                }
                DebuggableStatement::LogCall { meta, .. } => {
                    self.trace_if_enabled(&meta);
                    self.execute(statements, cur_bid + 1);
                }
                DebuggableStatement::Ret => {
                    self.handle_ret();
                }
            }
        }
    }

    /// Executes a symbolic expression concretely with given variable assignments.
    ///
    /// # Arguments
    ///
    /// * `name` - The template name to be executed.
    /// * `assignments` - A map of variable assignments for concrete execution.
    pub fn concrete_execute(
        &mut self,
        name: &String,
        assignment: &FxHashMap<SymbolicName, BigInt>,
    ) {
        self.is_concrete_mode = true;

        self.cur_state.template_id = self.symbolic_library.name2id[name];
        for (sym_name, sym_value) in assignment.into_iter() {
            self.cur_state.set_sym_val(
                sym_name.clone(),
                SymbolicValue::ConstantInt(sym_value.clone()),
            );
        }

        self.execute(
            &self.symbolic_library.template_library[&self.cur_state.template_id]
                .body
                .clone(),
            0,
        );
    }
}

// Evaluation and simplification methods
impl<'a> SymbolicExecutor<'a> {
    /// Evaluates a symbolic access from a debug access representation.
    ///
    /// This function processes a given `DebugAccess` (such as accessing a component or an array)
    /// and converts it into a `SymbolicAccess` by resolving symbolic expressions and simplifying variables.
    ///
    /// # Parameters
    /// - `access`: A reference to the `DebugAccess` that specifies the access pattern to evaluate.
    /// - `elem_id`: The element ID used for variable evaluation in the context of the access.
    ///
    /// # Returns
    /// A `SymbolicAccess` representing the evaluated and potentially simplified access pattern.
    ///
    /// # Behavior
    /// - For `ComponentAccess`, the symbolic name is directly cloned into the result.
    /// - For `ArrayAccess`, the expression is evaluated and simplified before being returned as a symbolic access.
    fn evaluate_access(&mut self, access: &DebugAccess, elem_id: usize) -> SymbolicAccess {
        match &access {
            DebugAccess::ComponentAccess(sym_name) => {
                SymbolicAccess::ComponentAccess(sym_name.clone())
            }
            DebugAccess::ArrayAccess(expr) => {
                let tmp_e = self.evaluate_expression(&expr, elem_id);
                let mut memo = FxHashSet::default();
                SymbolicAccess::ArrayAccess(
                    self.simplify_variables(&tmp_e, elem_id, false, false, &mut memo),
                )
            }
        }
    }

    /// Evaluates the dimensions of a symbolic expression list.
    ///
    /// This function resolves and simplifies a vector of symbolic expressions representing dimensions,
    /// converting them into concrete `usize` values if possible.
    ///
    /// # Parameters
    /// - `dims`: A reference to a vector of `DebuggableExpression` objects representing the dimensions.
    /// - `elem_id`: The element ID used for variable evaluation in the context of the dimensions.
    ///
    /// # Returns
    /// A vector of `usize` values representing the evaluated dimensions. If a dimension cannot be determined,
    /// it defaults to `0`.
    ///
    /// # Behavior
    /// - Each dimension expression is evaluated and simplified.
    /// - If the simplified result is a constant integer, it is converted to `usize`.
    /// - If the result cannot be determined (e.g., due to unresolved symbolic values), the dimension is set to `0`.
    pub fn evaluate_dimension(
        &mut self,
        dims: &Vec<DebuggableExpression>,
        elem_id: usize,
    ) -> Vec<usize> {
        dims.iter()
            .map(|arg0: &DebuggableExpression| {
                let evaled_arg0 = self.evaluate_expression(arg0, elem_id);
                let mut memo = FxHashSet::default();
                let simplified_arg0 =
                    self.simplify_variables(&evaled_arg0, elem_id, false, false, &mut memo);
                if let SymbolicValue::ConstantInt(bint) = &simplified_arg0 {
                    bint.to_usize().unwrap()
                } else {
                    /*
                    panic!(
                        "Undetermined dimension: {}",
                        simplified_arg0.lookup_fmt(&self.symbolic_library.id2name)
                    )
                    */
                    0
                }
            })
            .collect::<Vec<_>>()
    }

    /// Simplifies a given symbolic value (`sym_val`) based on the specified settings for constant
    /// and variable simplifications, returning a potentially simplified version of the input.
    ///
    /// # Parameters
    /// - `sym_val`: A reference to the symbolic value to be simplified. This value can represent
    ///   variables, constants, expressions, or other symbolic constructs.
    /// - `elem_id`: A unique identifier for the symbolic element being processed. This is used for
    ///   tracking purposes, such as recording coverage during branch simplifications.
    /// - `only_constatant_simplification`: If true, only variables ans signals to which constatns are assigned
    ///   are simplified, leaving non-determined variables and non-constant expressions untouched.
    /// - `only_variable_simplification`: If true, only variable substitutions are performed, leaving
    ///   non-variable expressions, including signals, untouched.
    ///
    /// # Returns
    /// A `SymbolicValue` representing the simplified result. Depending on the input and settings:
    /// - Constants may be replaced with their simplified forms.
    /// - Variables ans signals may be substituted based on the current state or template configuration.
    /// - Composite structures (e.g., arrays, tuples, and operations) are recursively simplified.
    ///
    /// # Behavior
    /// - If `only_constatant_simplification` is true, the function focuses on resolving constant
    ///   expressions while ignoring variable substitutions.
    /// - If `only_variable_simplification` is true, the function resolves symbolic variables
    ///   without modifying constants.
    /// - If neither flag is set, both constants and variables are simplified.
    /// - For composite structures like arrays, tuples, and conditional branches, the function
    ///   recursively simplifies each element or branch.
    /// - Conditional branches are further simplified based on the evaluation of their condition.
    ///   For example, a branch with a true or false condition can reduce to a single branch.
    /// - Unary and binary operations are evaluated when possible, leveraging any simplified
    ///   components.
    /// - Array and tuple elements are recursively simplified.
    /// - Uniform arrays are returned as-is, and its dimensions and intial value are simplified.
    ///
    /// # Notes
    /// - This function depends on the state and configuration of the context, including
    ///   `symbolic_library`, `cur_state`, and `setting`.
    /// - The implementation respects the `enable_coverage_tracking` setting to track branch
    ///   execution during simplification.
    ///
    /// # Performance
    /// - Recursive simplification can have significant computational overhead for deeply nested
    ///   structures or large arrays. Ensure input sizes are manageable in performance-critical
    ///   contexts.
    pub fn simplify_variables(
        &mut self,
        sym_val: &SymbolicValue,
        elem_id: usize,
        only_constatant_simplification: bool,
        only_variable_simplification: bool,
        memo: &mut FxHashSet<SymbolicValue>,
    ) -> SymbolicValue {
        match &sym_val {
            SymbolicValue::NOP => SymbolicValue::NOP,
            SymbolicValue::Variable(sym_name) => {
                if only_variable_simplification {
                    if let Some(template) = self
                        .symbolic_library
                        .template_library
                        .get(&self.cur_state.template_id)
                    {
                        if let Some(VariableType::Signal(_, _)) = template.id2type.get(&sym_name.id)
                        {
                            return sym_val.clone();
                        } else {
                            let new_sym_val = self.cur_state.get_sym_val_or_make_symvar(&sym_name);
                            memo.insert(sym_val.clone());
                            memo.insert(new_sym_val.clone());
                            if new_sym_val != *sym_val {
                                return self.simplify_variables(
                                    &new_sym_val,
                                    elem_id,
                                    only_constatant_simplification,
                                    only_variable_simplification,
                                    memo,
                                );
                            } else {
                                return new_sym_val;
                            }
                        }
                    }
                    sym_val.clone()
                } else if only_constatant_simplification {
                    if let Some(template) = self
                        .symbolic_library
                        .template_library
                        .get(&self.cur_state.template_id)
                    {
                        if let Some(typ) = template.id2type.get(&sym_name.id) {
                            if let VariableType::Signal(SignalType::Output, _) = typ {
                                if self.setting.substitute_output {
                                    return self.cur_state.get_sym_val_or_make_symvar(&sym_name);
                                } else {
                                    return sym_val.clone();
                                }
                            } else if let VariableType::Var = typ {
                                return self.cur_state.get_sym_val_or_make_symvar(&sym_name);
                            }
                        }
                    }
                    match self.cur_state.get_sym_val(&sym_name).map(|v| &**v) {
                        Some(SymbolicValue::ConstantInt(v)) => {
                            SymbolicValue::ConstantInt(v.clone())
                        }
                        Some(SymbolicValue::ConstantBool(b)) => SymbolicValue::ConstantBool(*b),
                        _ => sym_val.clone(),
                    }
                } else {
                    if self.is_concrete_mode && !memo.contains(&sym_val) {
                        let new_sym_val = self.cur_state.get_sym_val_or_make_symvar(&sym_name);
                        memo.insert(sym_val.clone());
                        memo.insert(new_sym_val.clone());
                        if new_sym_val != *sym_val {
                            self.simplify_variables(
                                &new_sym_val,
                                elem_id,
                                only_constatant_simplification,
                                only_variable_simplification,
                                memo,
                            )
                        } else {
                            new_sym_val
                        }
                    } else {
                        self.cur_state.get_sym_val_or_make_symvar(&sym_name)
                    }
                }
            }
            SymbolicValue::BinaryOp(lv, infix_op, rv)
            | SymbolicValue::AuxBinaryOp(lv, infix_op, rv) => {
                let lhs = self.simplify_variables(
                    lv,
                    elem_id,
                    only_constatant_simplification,
                    only_variable_simplification,
                    memo,
                );
                let rhs = self.simplify_variables(
                    rv,
                    elem_id,
                    only_constatant_simplification,
                    only_variable_simplification,
                    memo,
                );
                evaluate_binary_op(&lhs, &rhs, &self.setting.prime, infix_op)
            }
            SymbolicValue::Conditional(cond, then_val, else_val) => {
                let simplified_cond = self.simplify_variables(
                    cond,
                    elem_id,
                    only_constatant_simplification,
                    only_variable_simplification,
                    memo,
                );
                match simplified_cond {
                    SymbolicValue::ConstantBool(true) => {
                        if self.enable_coverage_tracking {
                            self.coverage_tracker.record_branch(elem_id, true);
                        }
                        self.simplify_variables(
                            then_val,
                            elem_id,
                            only_constatant_simplification,
                            only_variable_simplification,
                            memo,
                        )
                    }
                    SymbolicValue::ConstantBool(false) => {
                        if self.enable_coverage_tracking {
                            self.coverage_tracker.record_branch(elem_id, false);
                        }
                        self.simplify_variables(
                            else_val,
                            elem_id,
                            only_constatant_simplification,
                            only_variable_simplification,
                            memo,
                        )
                    }
                    _ => SymbolicValue::Conditional(
                        Rc::new(self.simplify_variables(
                            cond,
                            elem_id,
                            only_constatant_simplification,
                            only_variable_simplification,
                            memo,
                        )),
                        Rc::new(self.simplify_variables(
                            then_val,
                            elem_id,
                            only_constatant_simplification,
                            only_variable_simplification,
                            memo,
                        )),
                        Rc::new(self.simplify_variables(
                            else_val,
                            elem_id,
                            only_constatant_simplification,
                            only_variable_simplification,
                            memo,
                        )),
                    ),
                }
            }
            SymbolicValue::UnaryOp(prefix_op, value) => {
                let simplified_sym_val = self.simplify_variables(
                    value,
                    elem_id,
                    only_constatant_simplification,
                    only_variable_simplification,
                    memo,
                );
                match &simplified_sym_val {
                    SymbolicValue::ConstantInt(rv) => match prefix_op.0 {
                        ExpressionPrefixOpcode::Sub => SymbolicValue::ConstantInt(-1 * rv),
                        _ => SymbolicValue::UnaryOp(prefix_op.clone(), Rc::new(simplified_sym_val)),
                    },
                    SymbolicValue::ConstantBool(rv) => match prefix_op.0 {
                        ExpressionPrefixOpcode::BoolNot => SymbolicValue::ConstantBool(!rv),
                        _ => SymbolicValue::UnaryOp(prefix_op.clone(), Rc::new(simplified_sym_val)),
                    },
                    _ => SymbolicValue::UnaryOp(prefix_op.clone(), Rc::new(simplified_sym_val)),
                }
            }
            SymbolicValue::Array(elements) => SymbolicValue::Array(
                elements
                    .iter()
                    .map(|e| {
                        Rc::new(self.simplify_variables(
                            e,
                            elem_id,
                            only_constatant_simplification,
                            only_variable_simplification,
                            memo,
                        ))
                    })
                    .collect(),
            ),
            SymbolicValue::UniformArray(element, count) => {
                let uarray = SymbolicValue::UniformArray(
                    Rc::new(self.simplify_variables(
                        element,
                        elem_id,
                        only_constatant_simplification,
                        only_variable_simplification,
                        memo,
                    )),
                    Rc::new(self.simplify_variables(
                        count,
                        elem_id,
                        only_constatant_simplification,
                        only_variable_simplification,
                        memo,
                    )),
                );
                // self.convert_uniform_array_to_array(Rc::new(uarray), elem_id)
                uarray
            }
            SymbolicValue::Call(func_id, args) => SymbolicValue::Call(
                *func_id,
                args.iter()
                    .map(|arg| {
                        Rc::new(self.simplify_variables(
                            arg,
                            elem_id,
                            only_constatant_simplification,
                            only_variable_simplification,
                            memo,
                        ))
                    })
                    .collect(),
            ),
            _ => sym_val.clone(),
        }
    }

    /// Evaluates a symbolic expression, converting it into a `SymbolicValue`.
    ///
    /// This function handles various types of expressions, including constants, variables,
    /// and complex operations. It recursively evaluates sub-expressions as needed.
    ///
    /// # Arguments
    ///
    /// * `expr` - The `DebuggableExpression` to evaluate.
    /// * `elem_id` - Unique element id
    ///
    /// # Returns
    ///
    /// A `SymbolicValue` representing the evaluated expression.
    fn evaluate_expression(
        &mut self,
        expr: &DebuggableExpression,
        elem_id: usize,
    ) -> SymbolicValue {
        match &expr {
            DebuggableExpression::Number(value) => SymbolicValue::ConstantInt(value.clone()),
            DebuggableExpression::Variable { id, access } => {
                let resolved_sym_name = if access.is_empty() {
                    SymbolicName::new(*id, self.cur_state.owner_name.clone(), None)
                } else {
                    let tmp_name = SymbolicName::new(*id, self.cur_state.owner_name.clone(), None);
                    let sv = self.cur_state.get_sym_val(&tmp_name).cloned();

                    let mut component_name = None;
                    let mut dims = Vec::new();
                    for acc in access.iter() {
                        let evaled_access = self.evaluate_access(acc, elem_id);
                        match evaled_access {
                            SymbolicAccess::ComponentAccess(tmp_name) => {
                                component_name = Some(tmp_name);
                            }
                            SymbolicAccess::ArrayAccess(_) => {
                                dims.push(evaled_access);
                            }
                        }
                    }

                    if let Some(template) = self
                        .symbolic_library
                        .template_library
                        .get(&self.cur_state.template_id)
                    {
                        if let None = template.id2type.get(id) {
                            if sv.is_some() && component_name.is_none() {
                                match &*sv.unwrap() {
                                    SymbolicValue::Array(values) => {
                                        let ama = access_multidimensional_array(&values, &dims);
                                        if let ExecutionResult::Success(v) = ama {
                                            return v;
                                        }
                                        if let ExecutionResult::Failure = ama {
                                            self.execution_failed = true;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    self.construct_symbolic_name(*id, access, elem_id).1
                };
                SymbolicValue::Variable(resolved_sym_name)
            }
            DebuggableExpression::InfixOp { lhe, infix_op, rhe } => {
                let lhs = self.evaluate_expression(lhe, elem_id);
                let rhs = self.evaluate_expression(rhe, elem_id);
                SymbolicValue::BinaryOp(Rc::new(lhs), infix_op.clone(), Rc::new(rhs))
            }
            DebuggableExpression::PrefixOp { prefix_op, rhe } => {
                let expr = self.evaluate_expression(rhe, elem_id);
                SymbolicValue::UnaryOp(prefix_op.clone(), Rc::new(expr))
            }
            DebuggableExpression::InlineSwitchOp {
                cond,
                if_true,
                if_false,
            } => {
                let condition = self.evaluate_expression(cond, elem_id);
                let true_branch = self.evaluate_expression(if_true, elem_id);
                let false_branch = self.evaluate_expression(if_false, elem_id);
                SymbolicValue::Conditional(
                    Rc::new(condition),
                    Rc::new(true_branch),
                    Rc::new(false_branch),
                )
            }
            DebuggableExpression::ParallelOp { rhe, .. } => self.evaluate_expression(rhe, elem_id),
            DebuggableExpression::ArrayInLine { values } => {
                let elements = values
                    .iter()
                    .map(|v| Rc::new(self.evaluate_expression(v, elem_id)))
                    .collect();
                SymbolicValue::Array(elements)
            }
            DebuggableExpression::Tuple { values } => {
                let elements = values
                    .iter()
                    .map(|v| Rc::new(self.evaluate_expression(v, elem_id)))
                    .collect();
                SymbolicValue::Array(elements)
            }
            DebuggableExpression::UniformArray {
                value, dimension, ..
            } => {
                let evaluated_value = self.evaluate_expression(value, elem_id);
                let evaluated_dimension = self.evaluate_expression(dimension, elem_id);
                SymbolicValue::UniformArray(Rc::new(evaluated_value), Rc::new(evaluated_dimension))
            }
            DebuggableExpression::Call { id, args, .. } => {
                let evaluated_args: Vec<_> = args
                    .iter()
                    .map(|arg| self.evaluate_expression(arg, elem_id))
                    .collect();
                let mut memo = FxHashSet::default();
                let simplified_args = evaluated_args
                    .iter()
                    .map(|arg| {
                        Rc::new(self.simplify_variables(&arg, elem_id, false, false, &mut memo))
                    })
                    .collect();
                if self.symbolic_library.template_library.contains_key(id) {
                    SymbolicValue::Call(*id, simplified_args)
                } else if self.symbolic_library.function_library.contains_key(id) {
                    let symbolic_library = &mut self.symbolic_library;
                    let mut subse_setting = self.setting.clone();
                    subse_setting.only_initialization_blocks = false;
                    let mut subse = SymbolicExecutor::new(symbolic_library, &subse_setting);

                    let mut updated_owner_list = (*self.cur_state.owner_name).clone();
                    updated_owner_list.push(OwnerName {
                        id: *id,
                        counter: subse.symbolic_library.function_counter[id],
                        access: None,
                    });
                    subse.cur_state.owner_name = Rc::new(updated_owner_list);
                    subse
                        .symbolic_library
                        .function_counter
                        .insert(*id, subse.symbolic_library.function_counter[id] + 1);
                    subse.cur_state.set_template_id(*id);

                    let func = &subse.symbolic_library.function_library[id];
                    for i in 0..(func.function_argument_names.len()) {
                        let sym_name = SymbolicName::new(
                            func.function_argument_names[i],
                            subse.cur_state.owner_name.clone(),
                            None,
                        );
                        subse
                            .cur_state
                            .set_rc_sym_val(sym_name.clone(), simplified_args[i].clone());
                    }

                    if !subse.setting.off_trace {
                        trace!("{}", format!("{}", "===========================").cyan());
                        trace!("📞 Call {}", subse.symbolic_library.id2name[id]);
                    }

                    subse.execute(&func.body.clone(), 0);
                    self.mindim = std::cmp::min(subse.mindim, self.mindim);

                    if !subse.setting.off_trace {
                        trace!("{}", format!("{}", "===========================").cyan());
                    }

                    if !subse.cur_state.contains_symbolic_loop {
                        // NOTE: a function does not produce any constraint
                        self.cur_state
                            .symbolic_trace
                            .append(&mut subse.cur_state.symbolic_trace);
                        self.execution_failed = subse.execution_failed;

                        let return_sym_name =
                            SymbolicName::new(usize::MAX, subse.cur_state.owner_name.clone(), None);
                        let return_value =
                            (*subse.cur_state.symbol_binding_map[&return_sym_name]).clone();
                        match return_value {
                            SymbolicValue::ConstantBool(_) | SymbolicValue::ConstantInt(_) => {
                                return_value
                            }
                            _ => {
                                if is_concrete_array(&return_value) {
                                    return_value
                                } else {
                                    SymbolicValue::Call(*id, simplified_args)
                                }
                            }
                        }
                    } else {
                        SymbolicValue::Call(*id, simplified_args)
                    }
                } else {
                    panic!("Unknown Callee: {}", self.symbolic_library.id2name[id]);
                }
            }
            _ => {
                // We currently do not support BusCall and AnonymousComp.
                panic!(
                    "Unhandled expression type: {}",
                    expr.lookup_fmt(&self.symbolic_library.id2name, 0)
                );
            }
        }
    }
}

impl<'a> SymbolicExecutor<'a> {
    /// Handles the execution of an initialization block within a set of statements.
    ///
    /// This function identifies and processes an `InitializationBlock` from a list of statements.
    /// All initializations within the block are executed sequentially before continuing with the rest
    /// of the statements.
    ///
    /// # Parameters
    /// - `statements`: A vector of `DebuggableStatement` containing the program statements to execute.
    /// - `cur_bid`: The current statement index (block ID) to evaluate.
    ///
    /// # Behavior
    /// - Sets the state flag to indicate that execution is within an initialization block.
    /// - Executes each initialization statement in the block.
    /// - Resets the state flag and proceeds to the next statement after the block.
    fn handle_initialization_block(
        &mut self,
        statements: &Vec<DebuggableStatement>,
        cur_bid: usize,
    ) {
        if let DebuggableStatement::InitializationBlock {
            initializations, ..
        } = &statements[cur_bid]
        {
            self.cur_state.is_within_initialization_block = true;

            for init in initializations {
                self.execute(&vec![init.clone()], 0);
            }

            self.cur_state.is_within_initialization_block = false;
            self.execute(statements, cur_bid + 1);
        }
    }

    /// Handles the execution of a generic code block within a set of statements.
    ///
    /// This function identifies a `Block` from a list of statements, traces its metadata if enabled,
    /// and recursively executes the statements within the block.
    ///
    /// # Parameters
    /// - `statements`: A vector of `DebuggableStatement` containing the program statements to execute.
    /// - `cur_bid`: The current statement index (block ID) to evaluate.
    ///
    /// # Behavior
    /// - Executes all statements within the block, starting at index 0.
    /// - Continues execution with the next statement after the block.
    fn handle_block(&mut self, statements: &Vec<DebuggableStatement>, cur_bid: usize) {
        if let DebuggableStatement::Block { meta, stmts, .. } = &statements[cur_bid] {
            self.trace_if_enabled(&meta);
            self.execute(&stmts, 0);
            self.execute(statements, cur_bid + 1);
        }
    }

    /// Handles the execution of an `if-then-else` statement within a set of statements.
    ///
    /// This function evaluates the condition of an `IfThenElse` statement and determines
    /// which branch (if-case or else-case) to execute. It also tracks branch coverage if enabled.
    ///
    /// # Parameters
    /// - `statements`: A vector of `DebuggableStatement` containing the program statements to execute.
    /// - `cur_bid`: The current statement index (block ID) to evaluate.
    ///
    /// # Behavior
    /// - Evaluates the condition and simplifies it.
    /// - If the condition resolves to `true`, the if-case is executed.
    /// - If the condition resolves to `false` and an else-case exists, the else-case is executed.
    /// - If the condition cannot be simplified to a constant boolean, symbolic loops are flagged in the state.
    /// - Branch coverage is recorded if enabled.
    /// - Continues execution with the next statement after the `if-then-else`.
    fn handle_if_then_else(&mut self, statements: &Vec<DebuggableStatement>, cur_bid: usize) {
        if let DebuggableStatement::IfThenElse {
            meta,
            cond,
            if_case,
            else_case,
            ..
        } = &statements[cur_bid]
        {
            self.trace_if_enabled(meta);

            let evaled_cond = self.evaluate_expression(cond, meta.elem_id);
            let mut memo = FxHashSet::default();
            let simplified_condition =
                self.simplify_variables(&evaled_cond, meta.elem_id, true, false, &mut memo);

            match simplified_condition {
                SymbolicValue::ConstantBool(true) => {
                    if self.enable_coverage_tracking {
                        self.coverage_tracker.record_branch(meta.elem_id, true);
                    }
                    self.execute(&vec![*if_case.clone()], 0);
                }
                SymbolicValue::ConstantBool(false) => {
                    if let Some(stmt) = else_case {
                        if self.enable_coverage_tracking {
                            self.coverage_tracker.record_branch(meta.elem_id, false);
                        }
                        self.execute(&vec![*stmt.clone()], 0);
                    }
                }
                _ => {
                    self.cur_state.contains_symbolic_loop = true;
                }
            }
            self.execute(statements, cur_bid + 1);
        }
    }

    /// Handles the substitution of a value to a variable or data structure within a set of statements.
    ///
    /// This function processes a `Substitution` statement, performing symbolic evaluation and updates
    /// to maintain the current program state. It supports assignments to single variables, arrays,
    /// bulk assignments, and function call results.
    ///
    /// # Parameters
    /// - `statements`: A vector of `DebuggableStatement` representing the program statements to execute.
    /// - `cur_bid`: The current statement index (block ID) to evaluate.
    ///
    /// # Behavior
    /// - Evaluates the right-hand expression (RHE) symbolically and simplifies the result.
    /// - Handles specific cases based on the RHE:
    ///   - Updates uniform arrays if applicable.
    ///   - Processes array or bulk assignments by propagating values to multiple variables or array elements.
    /// - Sets the symbolic value of the assigned variables in the current state.
    /// - Handles assignments resulting from function calls.
    /// - If the left-hand side involves component access, it updates the relevant component variables.
    /// - Executes the next statement after processing the substitution.
    fn handle_substitution(&mut self, statements: &Vec<DebuggableStatement>, cur_bid: usize) {
        if let DebuggableStatement::Substitution {
            meta,
            var,
            access,
            op,
            rhe,
        } = &statements[cur_bid]
        {
            self.trace_if_enabled(meta);

            let evaled_rhe = self.evaluate_expression(rhe, meta.elem_id);
            let mut memo = FxHashSet::default();
            let mut simplified_rhe =
                self.simplify_variables(&evaled_rhe, meta.elem_id, true, false, &mut memo);
            let (left_base_name, left_var_name) =
                self.construct_symbolic_name(*var, access, meta.elem_id);
            let mut is_array_assignment = false;
            let mut is_bulk_assignment = false;
            let mut left_var_names = Vec::new();
            let mut right_values = Vec::new();
            let mut symbolic_positions = Vec::new();

            match (&evaled_rhe, &simplified_rhe) {
                (SymbolicValue::Variable(right_var_name), SymbolicValue::UniformArray(..)) => {
                    simplified_rhe =
                        self.update_uniform_array(right_var_name, &simplified_rhe, meta.elem_id);
                }
                _ => {}
            }

            match &simplified_rhe {
                SymbolicValue::Array(_) => {
                    is_array_assignment = true;
                    self.handle_array_substitution(
                        op,
                        &left_var_name,
                        &simplified_rhe,
                        meta.elem_id,
                    );
                }
                _ => {
                    let dim_of_left_var = left_var_name.get_dim();
                    let full_dim_of_left_var =
                        self.get_full_dimension_of_var(&left_var_name, &left_base_name);
                    is_bulk_assignment = full_dim_of_left_var > dim_of_left_var;
                    if full_dim_of_left_var > dim_of_left_var {
                        let component_name = if access.is_empty() {
                            None
                        } else {
                            Some(left_base_name.clone())
                        };
                        self.handle_bulk_assignment(
                            &component_name,
                            &left_var_name,
                            dim_of_left_var,
                            full_dim_of_left_var,
                            &simplified_rhe,
                            &mut left_var_names,
                            &mut right_values,
                            &mut symbolic_positions,
                        )
                    } else {
                        left_var_names.push(left_var_name.clone());
                        right_values.push(simplified_rhe.clone());
                    }
                    for (lvn, rv) in left_var_names.iter().zip(right_values.iter()) {
                        self.cur_state.set_sym_val(lvn.clone(), rv.clone());
                    }
                }
            }

            if let SymbolicValue::Call(callee_name, args) = &simplified_rhe {
                self.handle_call_substitution(
                    op,
                    callee_name,
                    args,
                    &left_var_name,
                    &simplified_rhe,
                );
            } else {
                if is_bulk_assignment {
                    for (lvn, rv) in left_var_names.iter().zip(right_values.iter()) {
                        self.handle_non_call_substitution(op, &lvn, &rv);
                    }
                } else if !is_array_assignment {
                    let mut memo = FxHashSet::default();
                    let semi_simplified_rhe =
                        self.simplify_variables(&evaled_rhe, meta.elem_id, true, true, &mut memo);
                    self.handle_non_call_substitution(op, &left_var_name, &semi_simplified_rhe);
                }
            }

            if !access.is_empty() {
                if is_bulk_assignment {
                    self.handle_component_bulk_access(
                        *var,
                        access,
                        &left_base_name,
                        &right_values,
                        &mut symbolic_positions,
                        meta.elem_id,
                    );
                } else {
                    self.handle_component_access(
                        *var,
                        access,
                        &left_base_name,
                        &simplified_rhe,
                        meta.elem_id,
                    );
                }
            }
            self.execute(statements, cur_bid + 1);
        }
    }

    fn handle_multi_substitution(&mut self, statements: &Vec<DebuggableStatement>, cur_bid: usize) {
        if let DebuggableStatement::MultSubstitution {
            meta, lhe, op, rhe, ..
        } = &statements[cur_bid]
        {
            self.trace_if_enabled(&meta);

            let lhe_val = self.evaluate_expression(lhe, meta.elem_id);
            let rhe_val = self.evaluate_expression(rhe, meta.elem_id);
            let mut memo_left = FxHashSet::default();
            let simplified_lhe_val =
                self.simplify_variables(&lhe_val, meta.elem_id, true, false, &mut memo_left);
            let mut memo_right = FxHashSet::default();
            let simplified_rhe_val =
                self.simplify_variables(&rhe_val, meta.elem_id, true, false, &mut memo_right);

            if self.setting.keep_track_constraints {
                match op {
                    DebuggableAssignOp(AssignOp::AssignConstraintSignal) => {
                        let cont = SymbolicValue::AssignEq(
                            Rc::new(simplified_lhe_val),
                            Rc::new(simplified_rhe_val),
                        );
                        self.cur_state.push_symbolic_trace(&cont);
                        self.cur_state.push_side_constraint(&cont);
                    }
                    DebuggableAssignOp(AssignOp::AssignSignal) => {
                        let cont = SymbolicValue::Assign(
                            Rc::new(simplified_lhe_val),
                            Rc::new(simplified_rhe_val),
                            self.symbolic_library.template_library[&self.cur_state.template_id]
                                .is_safe,
                            None,
                        );
                        self.cur_state.push_symbolic_trace(&cont);
                    }
                    _ => {}
                }
            }

            self.execute(statements, cur_bid + 1);
        }
    }

    /// Handles the execution of a `While` loop statement during symbolic evaluation.
    ///
    /// This function evaluates the condition of a `While` loop and determines whether to execute the
    /// loop body, exit the loop, or handle symbolic loops. It performs symbolic execution for each
    /// iteration and ensures the current program state is updated appropriately.
    ///
    /// # Parameters
    /// - `statements`: A vector of `DebuggableStatement` representing the program's statements.
    /// - `cur_bid`: The current statement index (block ID) being evaluated.
    ///
    /// # Behavior
    /// - Symbolically evaluates the loop condition (`cond`) and simplifies it.
    /// - If the condition evaluates to a constant boolean:
    ///   - `true`: Executes the loop body (`stmt`) and re-evaluates the `While` statement.
    ///   - `false`: Skips the loop body and proceeds to the next statement.
    /// - If the condition cannot be fully resolved (symbolic loop), marks the current state as containing
    ///   a symbolic loop and skips the loop execution.
    fn handle_while(&mut self, statements: &Vec<DebuggableStatement>, cur_bid: usize) {
        if let DebuggableStatement::While {
            meta, cond, stmt, ..
        } = &statements[cur_bid]
        {
            self.trace_if_enabled(&meta);
            // Symbolic execution of loops is complex. This is a simplified approach.
            let tmp_cond = self.evaluate_expression(cond, meta.elem_id);
            let mut memo = FxHashSet::default();
            let evaled_condition =
                self.simplify_variables(&tmp_cond, meta.elem_id, true, false, &mut memo);

            if let SymbolicValue::ConstantBool(flag) = evaled_condition {
                if flag {
                    self.execute(&vec![*stmt.clone()], 0);
                    self.execute(statements, cur_bid);
                } else {
                    self.execute(statements, cur_bid + 1);
                }
            } else {
                self.cur_state.contains_symbolic_loop = true;
                // symbolic loop can occur only within functions that always do not produce any constraints.
                self.execute(statements, cur_bid + 1);
            }
        }
    }

    fn handle_return(&mut self, statements: &Vec<DebuggableStatement>, cur_bid: usize) {
        if let DebuggableStatement::Return { meta, value, .. } = &statements[cur_bid] {
            self.trace_if_enabled(&meta);
            let tmp_val = self.evaluate_expression(value, meta.elem_id);
            let mut memo = FxHashSet::default();
            let return_value =
                self.simplify_variables(&tmp_val, meta.elem_id, true, false, &mut memo);

            // Handle return value (e.g., store in a special "return" variable)
            if !self.symbolic_library.id2name.contains_key(&usize::MAX) {
                self.symbolic_library
                    .name2id
                    .insert("__return__".to_string(), usize::MAX);
                self.symbolic_library
                    .id2name
                    .insert(usize::MAX, "__return__".to_string());
            }

            self.cur_state.set_sym_val(
                SymbolicName::new(usize::MAX, self.cur_state.owner_name.clone(), None),
                return_value,
            );
            self.execute(statements, cur_bid + 1);
        }
    }

    /// Handles the declaration of a variable during symbolic execution.
    ///
    /// This function processes variable declarations by registering the variable's type,
    /// dimensions, and initial symbolic value in the current program state. If the variable
    /// is an input signal and input overwriting is disabled, its initial value is not modified.
    ///
    /// # Parameters
    /// - `statements`: A vector of `DebuggableStatement` representing the program's statements.
    /// - `cur_bid`: The current statement index (block ID) being evaluated.
    /// - `elem_id`: The unique identifier for the current symbolic evaluation element.
    ///
    /// # Behavior
    /// - Extracts the variable ID and type from the declaration statement.
    /// - Constructs a `SymbolicName` for the variable using its ID and the current state's owner name.
    /// - Stores the variable's type in the symbolic store's type registry.
    /// - If the variable is not an input signal or input overwriting is not disabled:
    ///   - Assigns an initial symbolic value to the variable.
    /// - Evaluates the variable's dimensions using the current template or function library context
    ///   and stores them in the `id2dimensions` map.
    /// - Proceeds to execute the next statement in the program.
    ///
    /// # Panics
    /// - If the dimension expressions for the variable cannot be found in the template or function library.
    fn handle_declaration(
        &mut self,
        statements: &Vec<DebuggableStatement>,
        cur_bid: usize,
        elem_id: usize,
    ) {
        if let DebuggableStatement::Declaration { id, xtype, .. } = &statements[cur_bid] {
            let var_name = SymbolicName::new(*id, self.cur_state.owner_name.clone(), None);
            self.symbolic_store
                .variable_types
                .insert(*id, DebuggableVariableType(xtype.clone()));

            let is_input = matches!(xtype, VariableType::Signal(SignalType::Input, _));
            if !(self.setting.is_input_overwrite_disabled && is_input) {
                let value = SymbolicValue::Variable(var_name.clone());
                self.cur_state.set_sym_val(var_name, value);
            }

            let dims = if let Some(templ) = self
                .symbolic_library
                .template_library
                .get(&self.cur_state.template_id)
            {
                self.evaluate_dimension(&templ.id2dimension_expressions[id].clone(), elem_id)
            } else if let Some(func) = self
                .symbolic_library
                .function_library
                .get(&self.cur_state.template_id)
            {
                if let Some(dim_expr) = func.id2dimension_expressions.get(id) {
                    self.evaluate_dimension(&dim_expr.clone(), elem_id)
                } else {
                    panic!(
                        "Dim-expression of {} within {} cannt be found.",
                        self.symbolic_library.id2name[id],
                        self.symbolic_library.id2name[&self.cur_state.template_id]
                    );
                }
            } else {
                vec![]
                /*
                panic!(
                    "{} does not exist in the library",
                    self.symbolic_library.id2name[&self.cur_state.template_id]
                );*/
            };
            if let Some(md) = dims.iter().min() {
                self.mindim = std::cmp::min(self.mindim, *md);
            }
            self.id2dimensions.insert(*id, dims);

            self.execute(statements, cur_bid + 1);
        }
    }

    /// Handles the processing of an equality constraint during symbolic execution.
    ///
    /// This function evaluates and simplifies the left-hand expression (LHE) and right-hand expression (RHE)
    /// of an equality constraint. It determines if the constraint holds or is violated, tracks the constraint
    /// for debugging or analysis purposes, and updates the program's symbolic state accordingly.
    ///
    /// # Parameters
    /// - `statements`: A vector of `DebuggableStatement` representing the program's statements.
    /// - `cur_bid`: The current statement index (block ID) being evaluated.
    ///
    /// # Behavior
    /// - Extracts the metadata, LHE, and RHE from the equality constraint statement.
    /// - Evaluates and simplifies the LHE and RHE to their symbolic representations.
    /// - Constructs a symbolic equality condition using the simplified LHE and RHE.
    /// - Depending on the configuration:
    ///   - If `keep_track_constraints` is enabled:
    ///     - Records the constraint in the symbolic trace if assertions are not disabled.
    ///     - Adds the constraint to the list of side constraints.
    ///   - Otherwise:
    ///     - Simplifies the equality condition to check its truth value.
    ///     - Marks the program state as failed if the condition is found to be false.
    ///     - Stores the violated condition and its associated metadata for debugging.
    /// - Proceeds to execute the next statement in the program.
    ///
    /// # Configuration Options
    /// - `keep_track_constraints`: If true, constraints are tracked for debugging or analysis.
    /// - `constraint_assert_disabled`: If true, prevents constraints from being asserted into the symbolic trace.
    ///
    /// # State Updates
    /// - If the condition evaluates to `false` and constraints are not tracked, the program state is marked
    ///   as failed, and the violated condition is stored for later reporting.
    fn handle_constraint_equality(
        &mut self,
        statements: &Vec<DebuggableStatement>,
        cur_bid: usize,
    ) {
        if let DebuggableStatement::ConstraintEquality { meta, lhe, rhe } = &statements[cur_bid] {
            self.trace_if_enabled(&meta);

            let lhe_val = self.evaluate_expression(lhe, meta.elem_id);
            let rhe_val = self.evaluate_expression(rhe, meta.elem_id);
            let mut memo_left = FxHashSet::default();
            let simplified_lhe_val =
                self.simplify_variables(&lhe_val, meta.elem_id, false, true, &mut memo_left);
            let mut memo_right = FxHashSet::default();
            let simplified_rhe_val =
                self.simplify_variables(&rhe_val, meta.elem_id, false, true, &mut memo_right);

            let cond = SymbolicValue::BinaryOp(
                Rc::new(simplified_lhe_val),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                Rc::new(simplified_rhe_val),
            );

            if self.setting.keep_track_constraints {
                if !self.setting.constraint_assert_dissabled {
                    self.cur_state.push_symbolic_trace(&cond);
                }
                self.cur_state.push_side_constraint(&cond);
            } else {
                if !self.cur_state.is_failed {
                    let mut memo = FxHashSet::default();
                    let simplified_cond =
                        self.simplify_variables(&cond, meta.elem_id, false, false, &mut memo);
                    if let SymbolicValue::ConstantBool(false) = simplified_cond {
                        self.cur_state.is_failed = true;
                        let original_cond = SymbolicValue::BinaryOp(
                            Rc::new(lhe_val),
                            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                            Rc::new(rhe_val),
                        );
                        self.violated_condition = Some((meta.elem_id, original_cond));
                    }
                }
            }

            self.execute(statements, cur_bid + 1);
        }
    }

    fn handle_assert(&mut self, statements: &Vec<DebuggableStatement>, cur_bid: usize) {
        if let DebuggableStatement::Assert { meta, arg, .. } = &statements[cur_bid] {
            self.trace_if_enabled(&meta);
            let expr = self.evaluate_expression(&arg, meta.elem_id);
            let mut memo = FxHashSet::default();
            let condition = self.simplify_variables(&expr, meta.elem_id, true, true, &mut memo);
            if self.setting.keep_track_constraints {
                self.cur_state.push_symbolic_trace(&condition);
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
    }
}

// Utility methods for substitution
impl<'a> SymbolicExecutor<'a> {
    fn convert_uniform_array_to_array(
        &mut self,
        uniform_array: Rc<SymbolicValue>,
        elem_id: usize,
    ) -> SymbolicValue {
        let (elem, counts) = decompose_uniform_array(uniform_array);
        let mut concrete_counts = Vec::new();
        let mut is_success = true;
        for c in counts.iter() {
            let mut memo = FxHashSet::default();
            let s = self.simplify_variables(&c, elem_id, false, false, &mut memo);
            if let SymbolicValue::ConstantInt(v) = s {
                concrete_counts.push(v.to_usize().unwrap())
            } else {
                is_success = false;
                break;
            }
        }
        if is_success {
            SymbolicValue::Array(initialize_symbolic_nested_array_with_value(
                &concrete_counts,
                elem,
            ))
        } else {
            SymbolicValue::Array(Vec::new())
        }
    }

    /// Handles array substitution operations, updating symbolic bindings and resolving array elements.
    ///
    /// This function processes assignments or substitutions involving arrays, including:
    /// - Enumerating array elements to perform substitutions.
    /// - Handling bulk assignments when array dimensions are not fully specified.
    /// - Updating symbolic state with the results of the substitutions.
    ///
    /// # Parameters
    /// - `op`: The assignment operation to perform (e.g., AssignSignal or AssignConstraintSignal).
    /// - `left_var_name`: The symbolic name of the target variable where the substitution occurs.
    /// - `arr`: The array being assigned or substituted.
    /// - `elem_id`: The identifier of the current element, used for dimensional evaluations.
    ///
    /// # Behavior
    /// - Initializes a base array from the current symbolic binding if the target variable is already defined.
    ///   - Converts uniform arrays to regular arrays if needed.
    /// - Enumerates all elements in the input array (`arr`) and performs substitutions for each element:
    ///   - Constructs new symbolic variable names for individual array elements based on their positions.
    ///   - Determines the dimensionality of the target variable and handles bulk assignments when necessary.
    ///   - Updates the symbolic binding map with new assignments for each element.
    /// - Maintains consistency of nested arrays by updating them recursively, ensuring that the symbolic representation matches the logical structure of the array.
    ///
    /// # Symbolic State Updates
    /// - Updates the symbolic state with the modified array or individual elements.
    /// - Handles non-call substitutions for each array element, applying the specified assignment operation.
    /// - Updates the symbolic representation of the target variable in the current state.
    fn handle_array_substitution(
        &mut self,
        op: &DebuggableAssignOp,
        left_var_name: &SymbolicName,
        arr: &SymbolicValue,
        elem_id: usize,
    ) {
        let mut base_array = SymbolicValue::Array(Vec::new());
        if self
            .cur_state
            .symbol_binding_map
            .contains_key(left_var_name)
        {
            base_array = match &*self.cur_state.symbol_binding_map[left_var_name] {
                SymbolicValue::Array(elems) => SymbolicValue::Array(elems.to_vec()),
                SymbolicValue::UniformArray(_, _) => self.convert_uniform_array_to_array(
                    self.cur_state.symbol_binding_map[left_var_name].clone(),
                    elem_id,
                ),
                _ => arr.clone(), //SymbolicValue::Array(Vec::new()),
            };
        }

        let enumerated_elements = enumerate_array(arr);
        for (pos, elem) in enumerated_elements {
            let mut new_left_var_name = left_var_name.clone();
            let mut access = new_left_var_name.access.unwrap_or_default();
            for p in &pos {
                access.push(SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(
                    BigInt::from_usize(*p).unwrap(),
                )));
            }
            new_left_var_name.access = Some(access);
            new_left_var_name.update_hash();

            let mut owner_new_left_var_name = new_left_var_name.clone();
            owner_new_left_var_name.access = None;
            owner_new_left_var_name.update_hash();
            let dim_of_left_var = new_left_var_name.get_dim();
            let full_dim_of_left_var =
                self.get_full_dimension_of_var(&new_left_var_name, &owner_new_left_var_name);

            let mut left_var_names = Vec::new();
            let mut right_values = Vec::new();
            let mut symbolic_positions = Vec::new();
            if full_dim_of_left_var > dim_of_left_var {
                self.handle_bulk_assignment(
                    &None,
                    &new_left_var_name,
                    dim_of_left_var,
                    full_dim_of_left_var,
                    &elem,
                    &mut left_var_names,
                    &mut right_values,
                    &mut symbolic_positions,
                )
            } else {
                left_var_names.push(new_left_var_name);
                right_values.push(elem.clone());
            }
            for (lvn, rv) in left_var_names.iter().zip(right_values.iter()) {
                self.cur_state.set_sym_val(lvn.clone(), rv.clone());
                self.handle_non_call_substitution(op, &lvn, &rv);
            }

            if let SymbolicValue::Array(ref arr) = base_array {
                if !arr.is_empty() {
                    base_array =
                        (*update_nested_array(&pos, &Rc::new(base_array), &Rc::new(elem.clone())))
                            .clone();
                }
            }
        }

        if let SymbolicValue::Array(ref arr) = base_array {
            if !arr.is_empty() {
                self.cur_state
                    .set_sym_val(left_var_name.clone(), base_array);
            }
        }
    }

    /// Handles the substitution of a function or component call in symbolic execution.
    ///
    /// This function processes symbolic substitutions involving calls to functions or components.
    /// It determines if the `callee_id` corresponds to a known template in the symbolic library
    /// and performs appropriate initialization or state updates.
    ///
    /// # Parameters
    /// - `op`: The assignment operation indicating mutability, such as assigning to a signal.
    /// - `callee_id`: The identifier of the called function or component template.
    /// - `args`: The arguments passed to the callee, represented as symbolic values.
    /// - `component_or_return_name`: The symbolic name representing the callee's component or the result of the call.
    /// - `right_call`: The symbolic representation of the call being processed.
    ///
    /// # Behavior
    /// - **Template Initialization**:
    ///   - If `callee_id` matches a known template in the symbolic library, the corresponding template component is initialized using the provided arguments (`args`).
    ///   - Checks if the component is ready for execution after initialization.
    ///   - If ready, executes the component using its identifier and dimensions derived from its symbolic access path.
    /// - **Symbolic Trace Update**:
    ///   - If `callee_id` does not correspond to a known template, creates a `SymbolicValue::AssignCall` object to represent the call.
    ///   - Pushes the symbolic representation of the call to the current state's symbolic trace for further analysis.
    ///
    /// # Notes
    /// - Assignments to signals (as indicated by `AssignOp::AssignSignal`) are treated as mutable operations.
    /// - The function integrates symbolic call handling into the execution trace, ensuring that symbolic dependencies are tracked.
    /// - Calls to known templates trigger initialization and execution in a structured manner.
    fn handle_call_substitution(
        &mut self,
        op: &DebuggableAssignOp,
        callee_id: &usize,
        args: &Vec<Rc<SymbolicValue>>,
        component_or_return_name: &SymbolicName,
        right_call: &SymbolicValue,
    ) {
        let is_mutable = match op {
            DebuggableAssignOp(AssignOp::AssignSignal) => true,
            _ => false,
        };
        if self
            .symbolic_library
            .template_library
            .contains_key(callee_id)
        {
            self.initialize_template_component(callee_id, args, component_or_return_name);
            if self.is_ready(component_or_return_name) {
                let pre_dims = if let Some(acc) = &component_or_return_name.access {
                    acc
                } else {
                    &Vec::new()
                };
                self.execute_ready_component(
                    component_or_return_name.id,
                    component_or_return_name,
                    pre_dims,
                );
            }
        } else {
            let cont = SymbolicValue::AssignCall(
                Rc::new(SymbolicValue::Variable(component_or_return_name.clone())),
                Rc::new(right_call.clone()),
                is_mutable,
            );
            self.cur_state.push_symbolic_trace(&cont);
        }
    }

    /// Initializes a symbolic template component by executing its initialization blocks and setting up its state.
    ///
    /// This function is responsible for creating an instance of a template component, executing its initialization logic,
    /// and storing the resulting symbolic state, including variable bindings and dimensions.
    ///
    /// # Parameters
    /// - `callee_template_id`: The identifier of the template to initialize.
    /// - `args`: A vector of symbolic values representing the arguments passed to the template.
    /// - `component_name`: The symbolic name of the component being initialized.
    ///
    /// # Behavior
    /// - Creates a separate symbolic executor (`se_for_initialization`) with restricted settings for initialization:
    ///   - Only initialization blocks are executed.
    ///   - Tracing is turned off.
    /// - Sets the owner and template ID for the initialization executor.
    /// - Extracts the template definition from the symbolic library.
    /// - Handles template parameter initialization:
    ///   - Maps each template parameter name to the corresponding argument value.
    ///   - Temporarily saves any existing variable bindings that overlap with template parameters and restores them after initialization.
    /// - Executes the initialization blocks defined in the template.
    /// - Pre-determines dimensions and input bindings based on the template's structure and stores these details in local maps.
    /// - Restores any saved variables that were temporarily overridden during initialization.
    /// - Creates a new `SymbolicComponent` to represent the initialized component and stores it in the symbolic component store.
    ///
    /// # Symbolic State Updates
    /// - Adds the initialized component to the `components_store` with its name as the key.
    /// - Updates the variable bindings in the current symbolic state to reflect the component's initialization.
    ///
    /// # Notes
    /// - The initialization logic is separated from the main execution logic to ensure modularity and proper handling
    ///   of escaped variables and dimensions.
    fn initialize_template_component(
        &mut self,
        callee_template_id: &usize,
        args: &Vec<Rc<SymbolicValue>>,
        component_name: &SymbolicName,
    ) {
        let mut subse_setting = self.setting.clone();
        subse_setting.only_initialization_blocks = true;
        subse_setting.off_trace = true;
        let mut se_for_initialization =
            SymbolicExecutor::new(&mut self.symbolic_library, &subse_setting);
        se_for_initialization.cur_state.owner_name = self.cur_state.owner_name.clone();
        se_for_initialization
            .cur_state
            .set_template_id(*callee_template_id);

        let template =
            se_for_initialization.symbolic_library.template_library[callee_template_id].clone();
        let mut escaped_vars = Vec::new();

        // Set template parameters
        for i in 0..template.template_parameter_names.len() {
            let tp_name = SymbolicName::new(
                template.template_parameter_names[i],
                self.cur_state.owner_name.clone(),
                None,
            );
            if let Some(val) = self.cur_state.get_sym_val(&tp_name) {
                // Save variables with the same name separately
                escaped_vars.push((tp_name.clone(), val.clone()));
            }

            self.cur_state
                .set_rc_sym_val(tp_name.clone(), args[i].clone());
            se_for_initialization
                .cur_state
                .set_rc_sym_val(tp_name, args[i].clone());
        }

        se_for_initialization.execute(&template.body, 0);

        let mut inputs_binding_map = FxHashMap::default();
        let mut id2dimensions = FxHashMap::default();

        se_for_initialization.pre_determine_dimensions(
            &template,
            &mut inputs_binding_map,
            &mut id2dimensions,
        );

        self.restore_escaped_variables(&escaped_vars);

        let component = SymbolicComponent {
            template_id: *callee_template_id,
            args: args.clone(),
            inputs_binding_map: inputs_binding_map,
            id2dimensions: id2dimensions,
            is_done: false,
        };
        self.symbolic_store
            .components_store
            .insert(component_name.clone(), component);
    }

    fn pre_determine_dimensions(
        &mut self,
        template: &SymbolicTemplate,
        inputs_of_component: &mut FxHashMap<SymbolicName, Option<SymbolicValue>>,
        dimensions_of_inputs: &mut FxHashMap<usize, Vec<usize>>,
    ) {
        for (id, dims) in &self.id2dimensions {
            if template.input_ids.contains(id) {
                register_array_elements(*id, &dims, None, inputs_of_component);
            }
            dimensions_of_inputs.insert(*id, dims.to_vec());
        }
    }

    fn restore_escaped_variables(&mut self, escaped_vars: &Vec<(SymbolicName, SymbolicValueRef)>) {
        for (n, v) in escaped_vars {
            self.cur_state.set_rc_sym_val(n.clone(), v.clone());
        }
    }

    fn handle_component_bulk_access(
        &mut self,
        var: usize,
        access: &Vec<DebugAccess>,
        base_name: &SymbolicName,
        symbolic_values: &Vec<SymbolicValue>,
        symbolic_positions: &mut Vec<Vec<SymbolicAccess>>,
        elem_id: usize,
    ) {
        let (component_name, pre_dims, post_dims) = self.parse_component_access(access, elem_id);

        if let Some(component) = self.symbolic_store.components_store.get_mut(base_name) {
            for (sym_pos, sym_val) in symbolic_positions.iter().zip(symbolic_values.iter()) {
                let mut inp_name = SymbolicName::new(
                    component_name,
                    Rc::new(Vec::new()),
                    if post_dims.is_empty() {
                        None
                    } else {
                        Some(post_dims.clone())
                    },
                );
                if let Some(local_access) = inp_name.access.as_mut() {
                    local_access.append(&mut sym_pos.clone());
                } else {
                    inp_name.access = Some(sym_pos.clone());
                }
                inp_name.update_hash();
                component
                    .inputs_binding_map
                    .insert(inp_name, Some(sym_val.clone()));
            }
        }

        if self.is_ready(base_name) {
            self.execute_ready_component(var, base_name, &pre_dims);
        }
    }

    /// Handles bulk assignment of symbolic variables with optional dimensional adjustments.
    ///
    /// This function processes assignments where symbolic variables may represent arrays or have
    /// omitted dimensions. It resolves these dimensions, generates the necessary symbolic access paths,
    /// and updates the provided lists of left-hand variable names, right-hand values, and symbolic positions.
    ///
    /// # Parameters
    /// - `component_name`: An optional symbolic name of the component being assigned, used to recover omitted dimensions.
    /// - `left_var_name`: The symbolic name of the left-hand variable in the assignment.
    /// - `dim_of_left_var`: The current dimensionality of the left-hand variable.
    /// - `full_dim_of_left_var`: The full dimensionality of the left-hand variable, including omitted dimensions.
    /// - `rhe`: The right-hand expression, which can be a symbolic value or variable.
    /// - `left_var_names`: A mutable list to store the resolved symbolic names of left-hand variables after dimension adjustments.
    /// - `right_values`: A mutable list to store the resolved right-hand values corresponding to the left-hand variables.
    /// - `symbolic_positions`: A mutable list to store the symbolic access positions generated during the assignment.
    ///
    /// # Behavior
    /// - If the right-hand expression (`rhe`) is a variable:
    ///   - Recovers omitted dimensions for the left-hand variable based on `component_name` and the dimensionality parameters.
    ///   - Generates a Cartesian product of the omitted dimensions to produce all possible access paths.
    ///   - Updates the symbolic access paths and hashes for both left and right variables, appending them to the respective lists.
    /// - If `rhe` is not a variable:
    ///   - Directly appends the `left_var_name` and `rhe` to the respective lists without further processing.
    ///
    /// # Notes
    /// - This function ensures that omitted dimensions in symbolic assignments are accounted for, enabling
    ///   precise symbolic execution for multi-dimensional variables.
    /// - It updates symbolic hashes and access paths to maintain consistency in symbolic state tracking.
    fn handle_bulk_assignment(
        &mut self,
        component_name: &Option<SymbolicName>,
        left_var_name: &SymbolicName,
        dim_of_left_var: usize,
        full_dim_of_left_var: usize,
        rhe: &SymbolicValue,
        left_var_names: &mut Vec<SymbolicName>,
        right_values: &mut Vec<SymbolicValue>,
        symbolic_positions: &mut Vec<Vec<SymbolicAccess>>,
    ) {
        if let SymbolicValue::Variable(ref right_var_name) = rhe {
            let left_omitted_dims = if let Some(cn) = component_name {
                self.recover_omitted_dims(
                    Some(&cn),
                    &left_var_name,
                    dim_of_left_var,
                    full_dim_of_left_var,
                )
            } else {
                self.recover_omitted_dims(
                    None,
                    &left_var_name,
                    dim_of_left_var,
                    full_dim_of_left_var,
                )
            };

            let positions = generate_cartesian_product_indices(&left_omitted_dims);
            for p in positions {
                let mut left_var_name_p = left_var_name.clone();
                let mut right_var_name_p = right_var_name.clone();
                let symbolic_p = p
                    .iter()
                    .map(|arg0: &usize| {
                        SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(
                            BigInt::from_usize(*arg0).unwrap(),
                        ))
                    })
                    .collect::<Vec<_>>();
                symbolic_positions.push(symbolic_p.clone());
                if let Some(local_access) = left_var_name_p.access.as_mut() {
                    local_access.append(&mut symbolic_p.clone());
                } else {
                    left_var_name_p.access = Some(symbolic_p.clone());
                }
                left_var_name_p.update_hash();
                if let Some(local_access) = right_var_name_p.access.as_mut() {
                    local_access.append(&mut symbolic_p.clone());
                } else {
                    right_var_name_p.access = Some(symbolic_p);
                }
                right_var_name_p.update_hash();
                left_var_names.push(left_var_name_p);
                right_values.push(SymbolicValue::Variable(right_var_name_p));
            }
        } else {
            left_var_names.push(left_var_name.clone());
            right_values.push(rhe.clone());
        }
    }

    /// Handles non-call substitutions of symbolic variables and optionally tracks constraints.
    ///
    /// This function processes assignments that do not involve function calls, updating the current
    /// symbolic state with the provided variable name and value. If constraint tracking is enabled,
    /// it records the substitution as a symbolic trace and optionally as a side constraint.
    ///
    /// # Parameters
    /// - `op`: The assignment operation, wrapped in a `DebuggableAssignOp`, which determines the type of substitution.
    /// - `var_name`: The symbolic name of the variable being assigned.
    /// - `value`: The symbolic value being assigned to the variable.
    ///
    /// # Behavior
    /// - If `keep_track_constraints` is enabled in the settings:
    ///   - For `AssignConstraintSignal` operations:
    ///     - Creates an equality constraint (`AssignEq`) between the variable and the value.
    ///     - Adds the constraint to the symbolic trace and side constraints.
    ///   - For `AssignSignal` operations:
    ///     - Creates a direct assignment (`Assign`) between the variable and the value.
    ///     - Adds the assignment to the symbolic trace.
    /// - For other assignment types, no action is taken.
    fn handle_non_call_substitution(
        &mut self,
        op: &DebuggableAssignOp,
        var_name: &SymbolicName,
        value: &SymbolicValue,
    ) {
        if self.setting.keep_track_constraints {
            match op {
                DebuggableAssignOp(AssignOp::AssignConstraintSignal) => {
                    let cont = SymbolicValue::AssignEq(
                        Rc::new(SymbolicValue::Variable(var_name.clone())),
                        Rc::new(value.clone()),
                    );
                    self.cur_state.push_symbolic_trace(&cont);
                    self.cur_state.push_side_constraint(&cont);
                }
                DebuggableAssignOp(AssignOp::AssignSignal) => {
                    // handling zero-division pattern
                    let zero_div_info = if !self.is_concrete_mode {
                        self.is_concrete_mode = true;
                        let mut memo = FxHashSet::default();
                        let simplified_value = self.simplify_variables(
                            value,
                            std::usize::MAX,
                            false,
                            false,
                            &mut memo,
                        );
                        if let SymbolicValue::BinaryOp(
                            left,
                            DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Div),
                            right,
                        ) = simplified_value
                        {
                            let mut left_vars = FxHashSet::default();
                            extract_variables_from_symbolic_value(&left, &mut left_vars);
                            let mut left_quad_poly = Vec::new();
                            for v in left_vars {
                                if get_degree_polynomial(&left, &v) <= 2 {
                                    let coefs = get_coefficient_of_polynomials(
                                        &left,
                                        &v,
                                        &self.setting.prime,
                                    );
                                    left_quad_poly.push((v, coefs));
                                }
                            }

                            let mut right_vars = FxHashSet::default();
                            extract_variables_from_symbolic_value(&right, &mut right_vars);
                            let mut right_quad_poly = Vec::new();
                            for v in right_vars {
                                if get_degree_polynomial(&right, &v) <= 2 {
                                    let coefs = get_coefficient_of_polynomials(
                                        &right,
                                        &v,
                                        &self.setting.prime,
                                    );
                                    right_quad_poly.push((v, coefs));
                                }
                            }
                            self.is_concrete_mode = false;

                            if left_quad_poly.is_empty() && right_quad_poly.is_empty() {
                                None
                            } else {
                                Some((left_quad_poly, right_quad_poly))
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let cont = SymbolicValue::Assign(
                        Rc::new(SymbolicValue::Variable(var_name.clone())),
                        Rc::new(value.clone()),
                        self.symbolic_library.template_library[&self.cur_state.template_id].is_safe,
                        zero_div_info,
                    );
                    self.cur_state.push_symbolic_trace(&cont);
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
    /// * `elem_id` - Unique element id
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
        elem_id: usize,
    ) {
        let (component_name, pre_dims, post_dims) = self.parse_component_access(access, elem_id);

        if let Some(component) = self.symbolic_store.components_store.get_mut(base_name) {
            let inp_name = SymbolicName::new(
                component_name,
                Rc::new(Vec::new()),
                if post_dims.is_empty() {
                    None
                } else {
                    Some(post_dims)
                },
            );

            match value {
                SymbolicValue::Array(..) => {
                    let enumerated_elements = enumerate_array(value);
                    for (pos, elem) in enumerated_elements {
                        let mut new_inp_name = inp_name.clone();
                        let mut access = new_inp_name.access.unwrap_or_default();
                        for p in &pos {
                            access.push(SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(
                                BigInt::from_usize(*p).unwrap(),
                            )));
                        }
                        new_inp_name.access = Some(access);
                        new_inp_name.update_hash();
                        component
                            .inputs_binding_map
                            .insert(new_inp_name, Some(elem.clone()));
                    }

                    // TODO: is this line necessary?
                    component
                        .inputs_binding_map
                        .insert(inp_name, Some(value.clone()));
                }
                _ => {
                    component
                        .inputs_binding_map
                        .insert(inp_name, Some(value.clone()));
                }
            }
        }

        if self.is_ready(base_name) {
            self.execute_ready_component(var, base_name, &pre_dims);
        }
    }

    fn parse_component_access(
        &mut self,
        access: &Vec<DebugAccess>,
        elem_id: usize,
    ) -> (usize, Vec<SymbolicAccess>, Vec<SymbolicAccess>) {
        let mut component_name = 0;
        let mut pre_dims = Vec::new();
        let mut post_dims = Vec::new();
        let mut found_component = false;

        for acc in access {
            let evaled_access = self.evaluate_access(acc, elem_id);
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
            && (self.symbolic_store.components_store[name]
                .inputs_binding_map
                .iter()
                .all(|(_, v)| v.is_some())
                || self.symbolic_store.components_store[name]
                    .inputs_binding_map
                    .is_empty())
    }

    /// Executes a ready-to-run component in the symbolic execution context.
    ///
    /// This function initializes and executes a specified component if it has not already been executed.
    /// The execution propagates the component's effects, such as symbolic traces, constraints, and assignments,
    /// back to the current symbolic execution state.
    ///
    /// # Parameters
    /// - `component_id`: The unique identifier of the component to be executed.
    /// - `component_name`: The symbolic name of the component.
    /// - `pre_dims`: A vector of symbolic accesses representing pre-computed dimensions or indices for the component.
    ///
    /// # Behavior
    /// - Initializes a new symbolic executor for the component, inheriting and updating the owner list for context.
    /// - Configures the component with its template parameters and input bindings.
    /// - Executes the body of the component as defined in the template library.
    /// - If `propagate_assignments` is enabled in the settings:
    ///   - Merges the symbol binding map of the component back into the parent executor.
    /// - Propagates symbolic traces and side constraints generated during the component's execution.
    /// - If the component's template specifies `is_lessthan`, generates and appends a "less-than" constraint.
    /// - Optionally logs detailed execution traces if tracing is enabled in the settings.
    ///
    /// # Notes
    /// - The function ensures that the component is executed only once by checking its `is_done` flag in the `components_store`.
    /// - The `SymbolicExecutor` is re-initialized for the component with an updated owner name that incorporates the component's ID, counter, and access dimensions.
    /// - Handles template parameters and inputs before execution to ensure consistency with the symbolic model.
    fn execute_ready_component(
        &mut self,
        component_id: usize,
        component_name: &SymbolicName,
        pre_dims: &Vec<SymbolicAccess>,
    ) {
        if !self.symbolic_store.components_store[component_name].is_done {
            let mut subse = SymbolicExecutor::new(&mut self.symbolic_library, self.setting);
            let mut updated_owner_list = (*self.cur_state.owner_name).clone();
            updated_owner_list.push(OwnerName {
                id: component_id,
                counter: 0,
                access: if pre_dims.is_empty() {
                    None
                } else {
                    Some(pre_dims.clone())
                },
            });
            subse.cur_state.owner_name = Rc::new(updated_owner_list);

            let templ = &subse.symbolic_library.template_library
                [&self.symbolic_store.components_store[component_name].template_id];
            subse
                .cur_state
                .set_template_id(self.symbolic_store.components_store[component_name].template_id);

            // Set template-parameters of the component
            for i in 0..templ.template_parameter_names.len() {
                let tp_name = SymbolicName::new(
                    templ.template_parameter_names[i],
                    subse.cur_state.owner_name.clone(),
                    None,
                );
                let tp_val = self.symbolic_store.components_store[component_name].args[i].clone();
                subse
                    .cur_state
                    .set_rc_sym_val(tp_name.clone(), tp_val.clone());
            }

            // Set inputs of the component
            for (k, v) in self.symbolic_store.components_store[component_name]
                .inputs_binding_map
                .iter()
            {
                let n =
                    SymbolicName::new(k.id, subse.cur_state.owner_name.clone(), k.access.clone());
                subse.cur_state.set_sym_val(n, v.clone().unwrap());
            }

            if !self.setting.off_trace {
                trace!("{}", "===========================".cyan());
                trace!(
                    "📞 Call {}",
                    subse.symbolic_library.id2name
                        [&self.symbolic_store.components_store[component_name].template_id]
                );
            }

            let is_lessthan = templ.is_lessthan;
            subse.execute(&templ.body.clone(), 0);

            self.cur_state
                .symbolic_trace
                .append(&mut subse.cur_state.symbolic_trace);
            self.cur_state
                .side_constraints
                .append(&mut subse.cur_state.side_constraints);
            self.execution_failed = subse.execution_failed;
            if self.setting.propagate_assignments {
                for (k, v) in subse.cur_state.symbol_binding_map.iter() {
                    self.cur_state.set_rc_sym_val(k.clone(), v.clone());
                }
            }

            if is_lessthan {
                let cond = generate_lessthan_constraint(
                    &subse.symbolic_library.name2id,
                    subse.cur_state.owner_name,
                );
                self.cur_state.push_symbolic_trace(&cond);
            }

            if !self.setting.off_trace {
                trace!("{}", "===========================".cyan());
            }
        }
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
    /// * `elem_id` - Unique element id
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
        elem_id: usize,
    ) -> (SymbolicName, SymbolicName) {
        // Style of component access: owner[access].component[access]
        // Example: bits[0].dblIn[0];
        let mut pre_dims = Vec::new();
        let mut component_name = None;
        let mut post_dims = Vec::new();
        let mut found_component = false;
        for acc in access {
            let evaled_access = self.evaluate_access(&acc, elem_id);
            match evaled_access {
                SymbolicAccess::ComponentAccess(tmp_name) => {
                    found_component = true;
                    component_name = Some(tmp_name);
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
                SymbolicName::new(
                    base_id,
                    self.cur_state.owner_name.clone(),
                    if pre_dims.is_empty() {
                        None
                    } else {
                        Some(pre_dims.clone())
                    },
                ),
                SymbolicName::new(
                    base_id,
                    self.cur_state.owner_name.clone(),
                    if pre_dims.is_empty() {
                        None
                    } else {
                        Some(pre_dims)
                    },
                ),
            )
        } else {
            let mut owner_name = (*self.cur_state.owner_name).clone();
            owner_name.push(OwnerName {
                id: base_id,
                counter: 0,
                access: if pre_dims.is_empty() {
                    None
                } else {
                    Some(pre_dims.clone())
                },
            });
            (
                SymbolicName::new(
                    base_id,
                    self.cur_state.owner_name.clone(),
                    if pre_dims.is_empty() {
                        None
                    } else {
                        Some(pre_dims)
                    },
                ),
                SymbolicName::new(
                    component_name.unwrap(),
                    Rc::new(owner_name),
                    if post_dims.is_empty() {
                        None
                    } else {
                        Some(post_dims)
                    },
                ),
            )
        }
    }

    fn get_full_dimension_of_var(
        &self,
        var_name: &SymbolicName,
        sym_name_of_direct_owner: &SymbolicName,
    ) -> usize {
        if let Some(cs) = self
            .symbolic_store
            .components_store
            .get(sym_name_of_direct_owner)
        {
            if let Some(dims) = cs.id2dimensions.get(&var_name.id) {
                dims.len()
            } else {
                panic!(
                    "The dimensions of {} within {} has not been registered.",
                    self.symbolic_library.id2name[&var_name.id],
                    sym_name_of_direct_owner.lookup_fmt(&self.symbolic_library.id2name)
                );
            }
        } else if let Some(dim) = self.id2dimensions.get(&sym_name_of_direct_owner.id) {
            dim.len()
        } else {
            0
        }
    }

    /// Recovers the omitted dimensions of a symbolic variable based on its current and full dimensions.
    ///
    /// This function identifies and returns the dimensions of a variable that were omitted, considering its current
    /// dimension and the total dimensions available.
    ///
    /// # Parameters
    /// - `component_name`: An optional symbolic name of the component to which the variable belongs.
    ///   If `None`, the variable is treated as global.
    /// - `var_name`: The symbolic name of the variable whose dimensions are to be recovered.
    /// - `cur_dim`: The current dimension of the variable.
    /// - `full_dim`: The total number of dimensions the variable is expected to have.
    ///
    /// # Returns
    /// - A vector of `usize` values representing the omitted dimensions of the variable, starting from
    ///   the current dimension (`cur_dim`) to the full dimension (`full_dim`).
    ///
    /// # Behavior
    /// - If `component_name` is provided and corresponds to an existing component in the `components_store`,
    ///   the dimensions are retrieved from the component-specific store.
    /// - If `component_name` is not provided or does not exist in the `components_store`, the dimensions
    ///   are retrieved from the global `id2dimensions` mapping.
    /// - Iterates from `cur_dim` to `full_dim` and appends the corresponding dimensions to the result.
    fn recover_omitted_dims(
        &mut self,
        component_name: Option<&SymbolicName>,
        var_name: &SymbolicName,
        cur_dim: usize,
        full_dim: usize,
    ) -> Vec<usize> {
        let dimensions = if component_name.is_none() {
            &self.id2dimensions[&var_name.id]
        } else if self
            .symbolic_store
            .components_store
            .contains_key(component_name.unwrap())
        {
            &self.symbolic_store.components_store[component_name.unwrap()].id2dimensions
                [&var_name.id]
        } else {
            &self.id2dimensions[&var_name.id]
        };
        let mut omitted_dims = Vec::new();
        for i in cur_dim..full_dim {
            omitted_dims.push(dimensions[i]);
        }
        omitted_dims
    }

    fn update_uniform_array(
        &mut self,
        var_name: &SymbolicName,
        uarray: &SymbolicValue,
        elem_id: usize,
    ) -> SymbolicValue {
        let (_, dims) = decompose_uniform_array(Rc::new(uarray.clone()));
        let mut concrete_dims = Vec::new();
        for c in dims.iter() {
            let mut memo = FxHashSet::default();
            let s = self.simplify_variables(&c, elem_id, false, false, &mut memo);
            if let SymbolicValue::ConstantInt(v) = s {
                concrete_dims.push(v.to_usize().unwrap())
            } else {
                panic!(
                    "Cannot determine the dimensions of {}",
                    uarray.lookup_fmt(&self.symbolic_library.id2name)
                );
            }
        }

        let positions = generate_cartesian_product_indices(&concrete_dims);

        let mut sym_array = self.convert_uniform_array_to_array(Rc::new(uarray.clone()), elem_id);

        let is_signal = if let Some(template) = self
            .symbolic_library
            .template_library
            .get(&self.cur_state.template_id)
        {
            if let Some(VariableType::Signal(_, _)) = template.id2type.get(&var_name.id) {
                true
            } else {
                false
            }
        } else {
            false
        };

        let mut symbolic_positions = Vec::new();
        for p in positions {
            let mut var_name_p = var_name.clone();
            let symbolic_p = p
                .iter()
                .map(|arg0: &usize| {
                    SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(
                        BigInt::from_usize(*arg0).unwrap(),
                    ))
                })
                .collect::<Vec<_>>();
            symbolic_positions.push(symbolic_p.clone());
            if let Some(local_access) = var_name_p.access.as_mut() {
                local_access.append(&mut symbolic_p.clone());
            } else {
                var_name_p.access = Some(symbolic_p.clone());
            }
            var_name_p.update_hash();

            let sval = if !is_signal && self.cur_state.symbol_binding_map.contains_key(&var_name_p)
            {
                self.cur_state.symbol_binding_map[&var_name_p].clone()
            } else {
                Rc::new(SymbolicValue::Variable(var_name_p))
            };

            sym_array = (*update_nested_array(&p, &Rc::new(sym_array), &sval)).clone();
        }

        sym_array
    }
}
