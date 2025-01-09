use std::cmp::max;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use colored::Colorize;
use log::trace;
use num_bigint_dig::BigInt;
use num_traits::cast::ToPrimitive;
use num_traits::FromPrimitive;
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

use program_structure::ast::{
    AssignOp, Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode, Meta, SignalType,
    VariableType,
};

use crate::executor::debug_ast::{
    DebugAccess, DebuggableAssignOp, DebuggableExpression, DebuggableExpressionInfixOpcode,
    DebuggableStatement, DebuggableVariableType,
};
use crate::executor::symbolic_setting::SymbolicExecutorSetting;
use crate::executor::symbolic_state::SymbolicState;
use crate::executor::symbolic_value::{
    access_multidimensional_array, create_nested_array, decompose_uniform_array, enumerate_array,
    evaluate_binary_op, generate_lessthan_constraint, is_concrete_array, register_array_elements,
    update_nested_array, OwnerName, SymbolicAccess, SymbolicComponent, SymbolicLibrary,
    SymbolicName, SymbolicTemplate, SymbolicValue, SymbolicValueRef,
};
use crate::executor::utils::generate_cartesian_product_indices;

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

#[derive(Clone)]
pub struct CoverageTracker {
    paths: FxHashSet<u64>,
    visit_counter: FxHashMap<usize, usize>,
    current_path: Vec<(usize, usize, bool)>,
}

impl CoverageTracker {
    pub fn new() -> Self {
        CoverageTracker {
            paths: FxHashSet::default(),
            visit_counter: FxHashMap::default(),
            current_path: Vec::new(),
        }
    }

    pub fn record_branch(&mut self, meta_elem_id: usize, branch_cond: bool) {
        *self.visit_counter.entry(meta_elem_id).or_insert(0) += 1;
        self.current_path
            .push((meta_elem_id, self.visit_counter[&meta_elem_id], branch_cond));
    }

    pub fn record_path(&mut self) {
        let path_hash = self.hash_current_path();
        self.paths.insert(path_hash);
    }

    fn hash_current_path(&self) -> u64 {
        let mut hasher = FxHasher::default();
        self.current_path.hash(&mut hasher);
        hasher.finish()
    }

    pub fn clear(&mut self) {
        self.clear_current_path();
        self.paths.clear();
    }

    pub fn clear_current_path(&mut self) {
        self.visit_counter.clear();
        self.current_path.clear();
    }

    pub fn coverage_count(&self) -> usize {
        self.paths.len()
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
/// * `symbolic_library`: A mutable reference to the library storing templates for execution.
/// * `setting`: A reference to the execution settings.
/// * `symbolic_store`: A store for components, variable types, and execution states.
/// * `cur_state`: The current symbolic execution state.
pub struct SymbolicExecutor<'a> {
    pub symbolic_library: &'a mut SymbolicLibrary,
    pub setting: &'a SymbolicExecutorSetting,
    pub symbolic_store: SymbolicStore,
    pub cur_state: SymbolicState,
    pub violated_condition: Option<SymbolicValue>,
    coverage_tracker: CoverageTracker,
    enable_coverage_tracking: bool,
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
            coverage_tracker: CoverageTracker::new(),
            setting: setting,
            enable_coverage_tracking: false,
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
            let simplified_a = self.simplify_variables(&evaled_a, usize::MAX, true, false);
            let sym_name = SymbolicName::new(name2id[n], self.cur_state.owner_name.clone(), None);
            let cond = SymbolicValue::AssignEq(
                Rc::new(SymbolicValue::Variable(sym_name.clone())),
                Rc::new(simplified_a.clone()),
            );
            self.cur_state.set_sym_val(sym_name, simplified_a);
            if self.setting.keep_track_constraints {
                self.cur_state.push_trace_constraint(&cond);
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
                DebuggableStatement::Declaration { .. } => {
                    self.handle_declaration(statements, cur_bid);
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
                }
                DebuggableStatement::LogCall { meta, .. } => {
                    self.trace_if_enabled(&meta);
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
    /// Evaluates a symbolic access expression, converting it into a `SymbolicAccess` value.
    ///
    /// # Arguments
    ///
    /// * `access` - The `Access` to evaluate.
    /// * `elem_id` - Unique element id
    ///
    /// # Returns
    ///
    /// A `SymbolicAccess` representing the evaluated access.
    fn evaluate_access(&mut self, access: &DebugAccess, elem_id: usize) -> SymbolicAccess {
        match &access {
            DebugAccess::ComponentAccess(sym_name) => {
                SymbolicAccess::ComponentAccess(sym_name.clone())
            }
            DebugAccess::ArrayAccess(expr) => {
                let tmp_e = self.evaluate_expression(&expr, elem_id);
                SymbolicAccess::ArrayAccess(self.simplify_variables(&tmp_e, elem_id, false, false))
            }
        }
    }

    pub fn evaluate_dimension(
        &mut self,
        dims: &Vec<DebuggableExpression>,
        elem_id: usize,
    ) -> Vec<usize> {
        dims.iter()
            .map(|arg0: &DebuggableExpression| {
                let evaled_arg0 = self.evaluate_expression(arg0, elem_id);
                let simplified_arg0 = self.simplify_variables(&evaled_arg0, elem_id, false, false);
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
    /// * `sym_val` - The symbolic expression to simplify.
    /// * `elem_id` - Unique element id
    /// * `only_constatant_simplification`
    /// * `only_variable_simplification`
    ///
    /// # Returns
    ///
    /// A new `SymbolicValue` representing the simplified expression.
    fn simplify_variables(
        &mut self,
        sym_val: &SymbolicValue,
        elem_id: usize,
        only_constatant_simplification: bool,
        only_variable_simplification: bool,
    ) -> SymbolicValue {
        match &sym_val {
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
                            return self.cur_state.get_sym_val_or_make_symvar(&sym_name);
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
                    self.cur_state.get_sym_val_or_make_symvar(&sym_name)
                }
            }
            SymbolicValue::BinaryOp(lv, infix_op, rv) => {
                let lhs = self.simplify_variables(
                    lv,
                    elem_id,
                    only_constatant_simplification,
                    only_variable_simplification,
                );
                let rhs = self.simplify_variables(
                    rv,
                    elem_id,
                    only_constatant_simplification,
                    only_variable_simplification,
                );
                evaluate_binary_op(&lhs, &rhs, &self.setting.prime, infix_op)
            }
            SymbolicValue::Conditional(cond, then_val, else_val) => {
                let simplified_cond = self.simplify_variables(
                    cond,
                    elem_id,
                    only_constatant_simplification,
                    only_variable_simplification,
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
                        )
                    }
                    _ => SymbolicValue::Conditional(
                        Rc::new(self.simplify_variables(
                            cond,
                            elem_id,
                            only_constatant_simplification,
                            only_variable_simplification,
                        )),
                        Rc::new(self.simplify_variables(
                            then_val,
                            elem_id,
                            only_constatant_simplification,
                            only_variable_simplification,
                        )),
                        Rc::new(self.simplify_variables(
                            else_val,
                            elem_id,
                            only_constatant_simplification,
                            only_variable_simplification,
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
                        ))
                    })
                    .collect(),
            ),
            SymbolicValue::Tuple(elements) => SymbolicValue::Tuple(
                elements
                    .iter()
                    .map(|e| {
                        Rc::new(self.simplify_variables(
                            e,
                            elem_id,
                            only_constatant_simplification,
                            only_variable_simplification,
                        ))
                    })
                    .collect(),
            ),
            SymbolicValue::UniformArray(element, count) => SymbolicValue::UniformArray(
                Rc::new(self.simplify_variables(
                    element,
                    elem_id,
                    only_constatant_simplification,
                    only_variable_simplification,
                )),
                Rc::new(self.simplify_variables(
                    count,
                    elem_id,
                    only_constatant_simplification,
                    only_variable_simplification,
                )),
            ),
            SymbolicValue::Call(func_id, args) => SymbolicValue::Call(
                *func_id,
                args.iter()
                    .map(|arg| {
                        Rc::new(self.simplify_variables(
                            arg,
                            elem_id,
                            only_constatant_simplification,
                            only_variable_simplification,
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
                    for acc in access {
                        let evaled_access = self.evaluate_access(&acc.clone(), elem_id);
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
                let simplified_args = evaluated_args
                    .iter()
                    .map(|arg| Rc::new(self.simplify_variables(&arg, elem_id, false, false)))
                    .collect();
                if self.symbolic_library.template_library.contains_key(id) {
                    SymbolicValue::Call(*id, simplified_args)
                } else if self.symbolic_library.function_library.contains_key(id) {
                    let symbolic_library = &mut self.symbolic_library;
                    let mut subse_setting = self.setting.clone();
                    subse_setting.only_initialization_blocks = false;
                    let mut subse = SymbolicExecutor::new(symbolic_library, &subse_setting);

                    let mut updated_owner_list = (*self.cur_state.owner_name.clone()).clone();
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

                    if !subse.setting.off_trace {
                        trace!("{}", format!("{}", "===========================").cyan());
                    }

                    if !subse.cur_state.contains_symbolic_loop {
                        // NOTE: a function does not produce any constraint
                        self.cur_state
                            .trace_constraints
                            .append(&mut subse.cur_state.trace_constraints);

                        let return_sym_name =
                            SymbolicName::new(usize::MAX, subse.cur_state.owner_name.clone(), None);
                        let return_value =
                            (*subse.cur_state.symbol_binding_map[&return_sym_name].clone()).clone();
                        match return_value {
                            SymbolicValue::ConstantBool(_) | SymbolicValue::ConstantInt(_) => {
                                return_value
                            }
                            _ => {
                                if is_concrete_array(&return_value) {
                                    return_value
                                } else {
                                    SymbolicValue::Call(id.clone(), simplified_args)
                                }
                            }
                        }
                    } else {
                        SymbolicValue::Call(id.clone(), simplified_args)
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
    fn handle_initialization_block(
        &mut self,
        statements: &Vec<DebuggableStatement>,
        cur_bid: usize,
    ) {
        if let DebuggableStatement::InitializationBlock {
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
            self.execute(statements, cur_bid + 1);
        }
    }

    fn handle_block(&mut self, statements: &Vec<DebuggableStatement>, cur_bid: usize) {
        if let DebuggableStatement::Block { meta, stmts, .. } = &statements[cur_bid] {
            self.trace_if_enabled(&meta);
            self.execute(&stmts, 0);
            self.execute(statements, cur_bid + 1);
        }
    }

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
            let simplified_condition =
                self.simplify_variables(&evaled_cond, meta.elem_id, true, false);

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
            let simplified_rhe = self.simplify_variables(&evaled_rhe, meta.elem_id, true, false);
            let (left_base_name, left_var_name) =
                self.construct_symbolic_name(*var, access, meta.elem_id);

            let mut is_array_assignment = false;
            let mut is_bulk_assignment = false;
            let mut left_var_names = Vec::new();
            let mut right_values = Vec::new();
            let mut symbolic_positions = Vec::new();

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
                    let id_of_direct_owner = self.get_id_of_direct_owner(&left_base_name);
                    let full_dim_of_left_var =
                        self.get_full_dimension_of_var(&left_var_name, id_of_direct_owner);
                    is_bulk_assignment = full_dim_of_left_var > dim_of_left_var;
                    if full_dim_of_left_var > dim_of_left_var {
                        self.handle_bulk_assignment(
                            &left_var_name,
                            dim_of_left_var,
                            full_dim_of_left_var,
                            id_of_direct_owner,
                            &simplified_rhe,
                            &mut left_var_names,
                            &mut right_values,
                            &mut symbolic_positions,
                            meta.elem_id,
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
                    meta.elem_id,
                );
            } else {
                if is_bulk_assignment {
                    for (lvn, rv) in left_var_names.iter().zip(right_values.iter()) {
                        self.handle_non_call_substitution(op, &lvn, &rv);
                    }
                } else if !is_array_assignment {
                    let semi_simplified_rhe =
                        self.simplify_variables(&evaled_rhe, meta.elem_id, true, true);
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
            let simplified_lhe_val = self.simplify_variables(&lhe_val, meta.elem_id, true, false);
            let simplified_rhe_val = self.simplify_variables(&rhe_val, meta.elem_id, true, false);

            if self.setting.keep_track_constraints {
                match op {
                    DebuggableAssignOp(AssignOp::AssignConstraintSignal) => {
                        let cont = SymbolicValue::AssignEq(
                            Rc::new(simplified_lhe_val),
                            Rc::new(simplified_rhe_val),
                        );
                        self.cur_state.push_trace_constraint(&cont);
                        self.cur_state.push_side_constraint(&cont);
                    }
                    DebuggableAssignOp(AssignOp::AssignSignal) => {
                        let cont = SymbolicValue::Assign(
                            Rc::new(simplified_lhe_val),
                            Rc::new(simplified_rhe_val),
                            self.symbolic_library.template_library[&self.cur_state.template_id]
                                .is_safe,
                        );
                        self.cur_state.push_trace_constraint(&cont);
                    }
                    _ => {}
                }
            }

            self.execute(statements, cur_bid + 1);
        }
    }

    fn handle_while(&mut self, statements: &Vec<DebuggableStatement>, cur_bid: usize) {
        if let DebuggableStatement::While {
            meta, cond, stmt, ..
        } = &statements[cur_bid]
        {
            self.trace_if_enabled(&meta);
            // Symbolic execution of loops is complex. This is a simplified approach.
            let tmp_cond = self.evaluate_expression(cond, meta.elem_id);
            let evaled_condition = self.simplify_variables(&tmp_cond, meta.elem_id, true, false);

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
            let return_value = self.simplify_variables(&tmp_val, meta.elem_id, true, false);

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

    fn handle_declaration(&mut self, statements: &Vec<DebuggableStatement>, cur_bid: usize) {
        if let DebuggableStatement::Declaration { id, xtype, .. } = &statements[cur_bid] {
            let var_name = SymbolicName::new(*id, self.cur_state.owner_name.clone(), None);
            self.symbolic_store
                .variable_types
                .insert(*id, DebuggableVariableType(xtype.clone()));
            let value = SymbolicValue::Variable(var_name.clone());
            self.cur_state.set_sym_val(var_name, value);
            self.execute(statements, cur_bid + 1);
        }
    }

    fn handle_constraint_equality(
        &mut self,
        statements: &Vec<DebuggableStatement>,
        cur_bid: usize,
    ) {
        if let DebuggableStatement::ConstraintEquality { meta, lhe, rhe } = &statements[cur_bid] {
            self.trace_if_enabled(&meta);

            let lhe_val = self.evaluate_expression(lhe, meta.elem_id);
            let rhe_val = self.evaluate_expression(rhe, meta.elem_id);
            let simplified_lhe_val = self.simplify_variables(&lhe_val, meta.elem_id, false, true);
            let simplified_rhe_val = self.simplify_variables(&rhe_val, meta.elem_id, false, true);

            let cond = SymbolicValue::BinaryOp(
                Rc::new(simplified_lhe_val),
                DebuggableExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                Rc::new(simplified_rhe_val),
            );

            if self.setting.keep_track_constraints {
                self.cur_state.push_trace_constraint(&cond);
                self.cur_state.push_side_constraint(&cond);
            } else {
                let simplified_cond = self.simplify_variables(&cond, meta.elem_id, false, false);
                if let SymbolicValue::ConstantBool(false) = simplified_cond {
                    self.cur_state.is_failed = true;
                    self.violated_condition = Some(simplified_cond.clone());
                }
            }

            self.execute(statements, cur_bid + 1);
        }
    }

    fn handle_assert(&mut self, statements: &Vec<DebuggableStatement>, cur_bid: usize) {
        if let DebuggableStatement::Assert { meta, arg, .. } = &statements[cur_bid] {
            self.trace_if_enabled(&meta);
            let expr = self.evaluate_expression(&arg, meta.elem_id);
            let condition = self.simplify_variables(&expr, meta.elem_id, true, true);
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
    /// * `elem_id` - Unique element id
    ///
    /// # Side Effects
    ///
    /// Updates the current symbolic state with individual array element assignments.
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
            base_array = match (*self.cur_state.symbol_binding_map[left_var_name]).clone() {
                SymbolicValue::Array(elems) => SymbolicValue::Array(elems),
                SymbolicValue::UniformArray(_, _) => {
                    let (elem, counts) = decompose_uniform_array(
                        self.cur_state.symbol_binding_map[left_var_name].clone(),
                    );
                    let mut concrete_counts = Vec::new();
                    let mut is_success = true;
                    for c in counts.iter() {
                        let s = self.simplify_variables(&c, elem_id, false, false);
                        if let SymbolicValue::ConstantInt(v) = s {
                            concrete_counts.push(v.to_usize().unwrap())
                        } else {
                            is_success = false;
                            break;
                        }
                    }
                    if is_success {
                        SymbolicValue::Array(create_nested_array(&concrete_counts, elem))
                    } else {
                        SymbolicValue::Array(Vec::new())
                    }
                }
                _ => SymbolicValue::Array(Vec::new()),
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

            self.handle_non_call_substitution(op, &new_left_var_name, elem);
            self.cur_state.set_sym_val(new_left_var_name, elem.clone());

            if let SymbolicValue::Array(ref arr) = base_array {
                if !arr.is_empty() {
                    base_array =
                        (*update_nested_array(&pos, Rc::new(base_array), Rc::new(elem.clone())))
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
    /// * `elem_id` - Unique element id
    ///
    /// # Side Effects
    ///
    /// May initialize a new component in the symbolic store or update
    fn handle_call_substitution(
        &mut self,
        op: &DebuggableAssignOp,
        callee_name: &usize,
        args: &Vec<Rc<SymbolicValue>>,
        var_name: &SymbolicName,
        right_call: &SymbolicValue,
        elem_id: usize,
    ) {
        let is_mutable = match op {
            DebuggableAssignOp(AssignOp::AssignSignal) => true,
            _ => false,
        };
        if self
            .symbolic_library
            .template_library
            .contains_key(callee_name)
        {
            self.initialize_template_component(callee_name, args, var_name, elem_id);
        } else {
            let cont = SymbolicValue::AssignCall(
                Rc::new(SymbolicValue::Variable(var_name.clone())),
                Rc::new(right_call.clone()),
                is_mutable,
            );
            self.cur_state.push_trace_constraint(&cont);
        }
    }

    fn initialize_template_component(
        &mut self,
        callee_name: &usize,
        args: &Vec<Rc<SymbolicValue>>,
        var_name: &SymbolicName,
        elem_id: usize,
    ) {
        let mut subse_setting = self.setting.clone();
        subse_setting.only_initialization_blocks = true;
        subse_setting.off_trace = true;
        let mut se_for_initialization =
            SymbolicExecutor::new(&mut self.symbolic_library, &subse_setting);
        se_for_initialization.cur_state.owner_name = self.cur_state.owner_name.clone();
        se_for_initialization
            .cur_state
            .set_template_id(*callee_name);

        let template = se_for_initialization.symbolic_library.template_library[callee_name].clone();
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

        se_for_initialization.execute(&template.body.clone(), 0);

        let mut symbol_optional_binding_map = FxHashMap::default();

        se_for_initialization.initialize_template_inputs(
            &template,
            &mut symbol_optional_binding_map,
            elem_id,
        );

        self.restore_escaped_variables(&escaped_vars);

        let component = SymbolicComponent {
            template_name: *callee_name,
            args: args.clone(),
            symbol_optional_binding_map: symbol_optional_binding_map,
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
        elem_id: usize,
    ) {
        for id in &template.input_ids {
            let dims = self.evaluate_dimension(&template.id2dimensions[id], elem_id);
            register_array_elements(*id, &dims, None, inputs_of_component);
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
                    .symbol_optional_binding_map
                    .insert(inp_name, Some(sym_val.clone()));
            }
        }

        if self.is_ready(base_name) {
            self.execute_ready_component(var, base_name, &pre_dims);
        }
    }

    fn handle_bulk_assignment(
        &mut self,
        left_var_name: &SymbolicName,
        dim_of_left_var: usize,
        full_dim_of_left_var: usize,
        id_of_direct_owner: usize,
        rhe: &SymbolicValue,
        left_var_names: &mut Vec<SymbolicName>,
        right_values: &mut Vec<SymbolicValue>,
        symbolic_positions: &mut Vec<Vec<SymbolicAccess>>,
        elem_id: usize,
    ) {
        if let SymbolicValue::Variable(ref right_var_name) = rhe {
            let omitted_dims = self.recover_omitted_dims(
                &left_var_name,
                dim_of_left_var,
                full_dim_of_left_var,
                id_of_direct_owner,
                elem_id,
            );
            let positions = generate_cartesian_product_indices(&omitted_dims);
            for p in positions {
                let mut left_var_name_p = left_var_name.clone();
                let mut right_var_name_p = right_var_name.clone();
                let mut symbolic_p = p
                    .iter()
                    .map(|arg0: &usize| {
                        SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(
                            BigInt::from_usize(*arg0).unwrap(),
                        ))
                    })
                    .collect::<Vec<_>>();
                symbolic_positions.push(symbolic_p.clone());
                if let Some(local_access) = left_var_name_p.access.as_mut() {
                    local_access.append(&mut symbolic_p);
                } else {
                    left_var_name_p.access = Some(symbolic_p.clone());
                }
                left_var_name_p.update_hash();
                if let Some(local_access) = right_var_name_p.access.as_mut() {
                    local_access.append(&mut symbolic_p);
                } else {
                    right_var_name_p.access = Some(symbolic_p);
                }
                right_var_name_p.update_hash();
                left_var_names.push(left_var_name_p.clone());
                right_values.push(SymbolicValue::Variable(right_var_name_p));
            }
        } else {
            left_var_names.push(left_var_name.clone());
            right_values.push(rhe.clone());
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
    ///
    /// # Side Effects
    ///
    /// Updates the current symbolic state and may add constraints.
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
                    self.cur_state.push_trace_constraint(&cont);
                    self.cur_state.push_side_constraint(&cont);
                }
                DebuggableAssignOp(AssignOp::AssignSignal) => {
                    let cont = SymbolicValue::Assign(
                        Rc::new(SymbolicValue::Variable(var_name.clone())),
                        Rc::new(value.clone()),
                        self.symbolic_library.template_library[&self.cur_state.template_id].is_safe,
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
                            .symbol_optional_binding_map
                            .insert(new_inp_name, Some(elem.clone()));
                    }

                    // TODO: is this line necessary?
                    component
                        .symbol_optional_binding_map
                        .insert(inp_name, Some(value.clone()));
                }
                _ => {
                    component
                        .symbol_optional_binding_map
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
            && self.symbolic_store.components_store[name]
                .symbol_optional_binding_map
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
                id: var,
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
                let tp_name = SymbolicName::new(
                    templ.template_parameter_names[i],
                    subse.cur_state.owner_name.clone(),
                    None,
                );
                let tp_val = self.symbolic_store.components_store[base_name].args[i].clone();
                subse
                    .cur_state
                    .set_rc_sym_val(tp_name.clone(), tp_val.clone());

                /*
                let tp_cond =
                    SymbolicValue::AssignEq(Rc::new(SymbolicValue::Variable(tp_name)), tp_val);
                self.cur_state.push_trace_constraint(&tp_cond);
                */
            }

            // Set inputs of the component
            for (k, v) in self.symbolic_store.components_store[base_name]
                .symbol_optional_binding_map
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
                        [&self.symbolic_store.components_store[base_name].template_name]
                );
            }

            let is_lessthan = templ.is_lessthan;
            subse.execute(&templ.body.clone(), 0);

            self.cur_state
                .trace_constraints
                .append(&mut subse.cur_state.trace_constraints);
            self.cur_state
                .side_constraints
                .append(&mut subse.cur_state.side_constraints);
            if self.setting.propagate_assignments {
                for (k, v) in subse.cur_state.symbol_binding_map.iter() {
                    self.cur_state.set_rc_sym_val(k.clone(), v.clone());
                }
            }

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
            let evaled_access = self.evaluate_access(&acc.clone(), elem_id);
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
            let mut owner_name = (*self.cur_state.owner_name.clone()).clone();
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

    fn get_id_of_direct_owner(&self, base_name: &SymbolicName) -> usize {
        if let Some(c) = self.symbolic_store.components_store.get(base_name) {
            c.template_name
        } else {
            self.cur_state.template_id
        }
    }

    fn get_full_dimension_of_var(
        &self,
        var_name: &SymbolicName,
        id_of_direct_owner: usize,
    ) -> usize {
        if let Some(template) = self
            .symbolic_library
            .template_library
            .get(&id_of_direct_owner)
        {
            return template
                .id2dimensions
                .get(&var_name.id)
                .map_or(0, |dimensions| dimensions.len());
        } else if let Some(function) = self
            .symbolic_library
            .function_library
            .get(&id_of_direct_owner)
        {
            return function
                .id2dimensions
                .get(&var_name.id)
                .map_or(0, |dimensions| dimensions.len());
        } else if id_of_direct_owner == std::usize::MAX {
            return 0;
        }

        panic!(
            "Cannot find the owner template/function: {}",
            self.symbolic_library.id2name[&id_of_direct_owner]
        );
    }

    fn recover_omitted_dims(
        &mut self,
        var_name: &SymbolicName,
        cur_dim: usize,
        full_dim: usize,
        id_of_direct_owner: usize,
        elem_id: usize,
    ) -> Vec<usize> {
        let mut omitted_dims = Vec::new();
        for i in cur_dim..full_dim {
            let dim_clone = if self
                .symbolic_library
                .template_library
                .contains_key(&id_of_direct_owner)
            {
                self.symbolic_library.template_library[&id_of_direct_owner].id2dimensions
                    [&var_name.id][i]
                    .clone()
            } else {
                self.symbolic_library.function_library[&id_of_direct_owner].id2dimensions
                    [&var_name.id][i]
                    .clone()
            };
            let evaled_dim = self.evaluate_expression(&dim_clone, elem_id);
            let simplified_dim = self.simplify_variables(&evaled_dim, elem_id, false, false);
            if let SymbolicValue::ConstantInt(ref num) = simplified_dim {
                omitted_dims.push(num.to_usize().unwrap());
            }
        }
        return omitted_dims;
    }
}
