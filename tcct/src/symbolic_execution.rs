use colored::Colorize;
use log::{error, trace, warn};
use num_bigint_dig::BigInt;
use num_traits::cast::ToPrimitive;
use num_traits::Signed;
use num_traits::Zero;
use rustc_hash::FxHashMap;
use std::cmp::max;
use std::rc::Rc;

use program_structure::ast::{
    AssignOp, Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode, Meta, SignalType,
    VariableType,
};

use crate::debug_ast::{
    DebugAccess, DebugAssignOp, DebugExpression, DebugExpressionInfixOpcode,
    DebugExpressionPrefixOpcode, DebugStatement, DebugVariableType,
};
use crate::symbolic_value::{
    OwnerName, SymbolicAccess, SymbolicComponent, SymbolicLibrary, SymbolicName, SymbolicValue,
};
use crate::utils::{extended_euclidean, italic};

/// Represents the state of symbolic execution, holding symbolic values,
/// trace constraints, side constraints, and depth information.
#[derive(Clone)]
pub struct SymbolicState {
    pub owner_name: Rc<Vec<OwnerName>>,
    pub template_id: usize,
    depth: usize,
    pub values: FxHashMap<SymbolicName, Rc<SymbolicValue>>,
    pub trace_constraints: Vec<Rc<SymbolicValue>>,
    pub side_constraints: Vec<Rc<SymbolicValue>>,
}

impl SymbolicState {
    /// Creates a new `SymbolicState` with default values.
    pub fn new() -> Self {
        SymbolicState {
            owner_name: Rc::new(Vec::new()),
            template_id: usize::MAX,
            depth: 0_usize,
            values: FxHashMap::default(),
            trace_constraints: Vec::new(),
            side_constraints: Vec::new(),
        }
    }

    /// Sets the owner name of the current symbolic state.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the owner to set.
    ///
    pub fn add_owner(&mut self, oname: &OwnerName) {
        let mut on = (*self.owner_name.clone()).clone();
        on.push(oname.clone());
        self.owner_name = Rc::new(on);
    }

    pub fn get_owner(&self, lookup: &FxHashMap<usize, String>) -> String {
        self.owner_name
            .iter()
            .map(|e: &OwnerName| lookup[&e.name].clone())
            .collect::<Vec<_>>()
            .join(".")
    }

    pub fn set_template_id(&mut self, name: usize) {
        self.template_id = name;
    }

    /// Sets the current depth of the symbolic state.
    ///
    /// # Arguments
    ///
    /// * `d` - The depth level to set.
    pub fn set_depth(&mut self, d: usize) {
        self.depth = d;
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

    pub fn set_rc_symval(&mut self, name: SymbolicName, value: Rc<SymbolicValue>) {
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
    pub fn get_symval(&self, name: &SymbolicName) -> Option<&Rc<SymbolicValue>> {
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

    pub fn lookup_fmt(&self, lookup: &FxHashMap<usize, String>) -> String {
        let mut s = "".to_string();
        s += &format!("🛠️ {}", format!("{}", "SymbolicState [\n").cyan());
        s += &format!(
            "  {} {}\n",
            format!("👤 {}", "owner:").cyan(),
            italic(&format!(
                "{:?}",
                &self
                    .owner_name
                    .iter()
                    .map(|c: &OwnerName| lookup[&c.name].clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .magenta()
        );
        s += &format!("  📏 {} {}\n", format!("{}", "depth:").cyan(), self.depth);
        s += &format!("  📋 {}\n", format!("{}", "values:").cyan());
        for (k, v) in self.values.iter() {
            s += &format!(
                "      {}: {}\n",
                k.lookup_fmt(lookup),
                format!("{}", v.lookup_fmt(lookup))
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
                    .map(|c| c.lookup_fmt(lookup))
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
                    .map(|c| c.lookup_fmt(lookup))
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

pub struct SymbolicExecutorSetting {
    pub prime: BigInt,
    pub propagate_substitution: bool,
    pub skip_initialization_blocks: bool,
    pub off_trace: bool,
    pub keep_track_unrolled_offset: bool,
}

/// Executes symbolic execution on a series of statements while maintaining multiple states.
/// It handles branching logic and updates constraints accordingly during execution flow.
///
/// This struct is parameterized over a lifetime `'a`, which is used for borrowing constraint statistics references.
///
/// # Fields
///
/// * `template_library` - A library storing templates for execution.
/// * `components_store` - A store for components used in execution.
/// * `variable_types` - A map storing types of variables.
/// * `prime` - A prime number used in computations.
/// * `cur_state`, `block_end_states`, `final_states` - Various states managed during execution.
/// * `max_depth` - Tracks maximum depth reached during execution.
pub struct SymbolicExecutor<'a> {
    pub symbolic_library: &'a mut SymbolicLibrary,
    pub setting: &'a SymbolicExecutorSetting,
    pub symbolic_store: SymbolicStore,
    pub cur_state: SymbolicState,
}

impl<'a> SymbolicExecutor<'a> {
    /// Creates a new instance of `SymbolicExecutor`, initializing all necessary states and statistics trackers.
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

    pub fn clear(&mut self) {
        self.cur_state = SymbolicState::new();
        self.symbolic_store.clear();
        self.symbolic_library.clear_function_counter();
    }

    // Checks if a component is ready based on its inputs being fully specified.
    //
    // # Arguments
    //
    // * 'name' - Name of the component to check readiness for.
    //
    // # Returns
    //
    // A boolean indicating readiness status.
    fn is_ready(&self, name: &SymbolicName) -> bool {
        self.symbolic_store.components_store.contains_key(name)
            && self.symbolic_store.components_store[name]
                .inputs
                .iter()
                .all(|(_, v)| v.is_some())
    }

    // Feeds arguments into current state variables based on provided names and expressions.
    //
    // # Arguments
    //
    // * 'names' : Vector containing names corresponding with expressions being fed as arguments.
    // * 'args' : Vector containing expressions whose evaluated results will be assigned as argument values.
    pub fn feed_arguments(&mut self, names: &Vec<String>, args: &Vec<Expression>) {
        let mut name2id = self.symbolic_library.name2id.clone();
        let mut id2name = self.symbolic_library.id2name.clone();
        for (n, a) in names.iter().zip(args.iter()) {
            let evaled_a = self.evaluate_expression(&DebugExpression::from(
                a.clone(),
                &mut name2id,
                &mut id2name,
            ));
            self.cur_state.set_symval(
                SymbolicName {
                    name: name2id[n],
                    owner: self.cur_state.owner_name.clone(),
                    access: None,
                },
                evaled_a,
            );
        }
        //self.name2id = cloned_name2id;
        //self.id2name = cloned_id2name;
    }

    /// Expands all stack states by executing each statement block recursively,
    /// updating depth and managing branching paths in execution flow.
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

    fn trace_if_enabled(&self, meta: &Meta) {
        if !self.setting.off_trace {
            trace!(
                "(elem_id={}) {}",
                meta.elem_id,
                self.cur_state.lookup_fmt(&self.symbolic_library.id2name)
            );
        }
    }

    /// Executes a sequence of statements symbolically from a specified starting block index,
    /// updating internal states and handling control structures like if-else and loops appropriately.
    ///
    /// # Arguments
    ///
    /// * `statements` - A vector of extended statements representing program logic to execute symbolically.
    /// * `cur_bid` - Current block index to start execution from.
    pub fn execute(&mut self, statements: &Vec<DebugStatement>, cur_bid: usize) {
        if cur_bid < statements.len() {
            self.symbolic_store.max_depth =
                max(self.symbolic_store.max_depth, self.cur_state.get_depth());

            /*
            let n = SymbolicName {
                name: self.name2id["i"],
                owner: vec![OwnerName {
                    name: self.name2id["main"],
                    counter: 0,
                }],
                access: Vec::new(),
            };
            if self.cur_state.values.contains_key(&n) {
                println!(
                    "main.i={}",
                    self.cur_state.values[&n].lookup_fmt(&self.id2name)
                );
            }*/

            match &statements[cur_bid] {
                DebugStatement::InitializationBlock {
                    initializations,
                    xtype,
                    ..
                } => {
                    let mut is_input = false;
                    if let VariableType::Signal(SignalType::Input, _taglist) = &xtype {
                        is_input = true;
                    }

                    if !(self.setting.skip_initialization_blocks && is_input) {
                        for init in initializations {
                            self.execute(&vec![init.clone()], 0);
                        }
                    }
                    self.symbolic_store.block_end_states = vec![self.cur_state.clone()];
                    self.expand_all_stack_states(
                        statements,
                        cur_bid + 1,
                        self.cur_state.get_depth(),
                    );
                }
                DebugStatement::Block { meta, stmts, .. } => {
                    self.trace_if_enabled(&meta);
                    self.execute(&stmts, 0);
                    self.expand_all_stack_states(
                        statements,
                        cur_bid + 1,
                        self.cur_state.get_depth(),
                    );
                }
                DebugStatement::IfThenElse {
                    meta,
                    cond,
                    if_case,
                    else_case,
                    ..
                } => {
                    self.trace_if_enabled(&meta);
                    let tmp_cond = self.evaluate_expression(cond);
                    let original_evaled_condition = self.fold_variables(&tmp_cond, true);
                    let evaled_condition =
                        self.fold_variables(&tmp_cond, !self.setting.propagate_substitution);

                    // Save the current state
                    let cur_depth = self.cur_state.get_depth();
                    let stack_states = self.symbolic_store.block_end_states.clone();

                    // Create a branch in the symbolic state
                    let mut if_state = self.cur_state.clone();

                    if let SymbolicValue::ConstantBool(false) = evaled_condition {
                        if !self.setting.off_trace {
                            trace!(
                                "{}",
                                format!("(elem_id={}) 🚧 Unreachable `Then` Branch", meta.elem_id)
                                    .yellow()
                            );
                        }
                    } else {
                        if_state.push_trace_constraint(&evaled_condition);
                        if_state.push_side_constraint(&original_evaled_condition);
                        if_state.set_depth(cur_depth + 1);
                        self.cur_state = if_state.clone();
                        self.execute(&vec![*if_case.clone()], 0);
                        self.expand_all_stack_states(statements, cur_bid + 1, cur_depth);
                    }

                    let mut stack_states_if_true = self.symbolic_store.block_end_states.clone();
                    self.symbolic_store.block_end_states = stack_states;
                    let neg_evaled_condition =
                        if let SymbolicValue::ConstantBool(v) = evaled_condition {
                            SymbolicValue::ConstantBool(!v)
                        } else {
                            SymbolicValue::UnaryOp(
                                DebugExpressionPrefixOpcode(ExpressionPrefixOpcode::BoolNot),
                                Rc::new(evaled_condition),
                            )
                        };
                    let original_neg_evaled_condition =
                        if let SymbolicValue::ConstantBool(v) = original_evaled_condition {
                            SymbolicValue::ConstantBool(!v)
                        } else {
                            SymbolicValue::UnaryOp(
                                DebugExpressionPrefixOpcode(ExpressionPrefixOpcode::BoolNot),
                                Rc::new(original_evaled_condition),
                            )
                        };
                    if let SymbolicValue::ConstantBool(false) = neg_evaled_condition {
                        if !self.setting.off_trace {
                            trace!(
                                "{}",
                                format!("(elem_id={}) 🚧 Unreachable `Else` Branch", meta.elem_id)
                                    .yellow()
                            );
                        }
                    } else {
                        let mut else_state = self.cur_state.clone();
                        else_state.push_trace_constraint(&neg_evaled_condition);
                        else_state.push_side_constraint(&original_neg_evaled_condition);
                        else_state.set_depth(cur_depth + 1);
                        self.cur_state = else_state;
                        if let Some(else_stmt) = else_case {
                            self.execute(&vec![*else_stmt.clone()], 0);
                        } else {
                            self.symbolic_store.block_end_states = vec![self.cur_state.clone()];
                        }
                        self.expand_all_stack_states(statements, cur_bid + 1, cur_depth);
                    }
                    self.symbolic_store
                        .block_end_states
                        .append(&mut stack_states_if_true);
                }
                DebugStatement::While {
                    meta, cond, stmt, ..
                } => {
                    self.trace_if_enabled(&meta);
                    // Symbolic execution of loops is complex. This is a simplified approach.
                    let tmp_cond = self.evaluate_expression(cond);
                    let evaled_condition =
                        self.fold_variables(&tmp_cond, !self.setting.propagate_substitution);

                    if let SymbolicValue::ConstantBool(flag) = evaled_condition {
                        if flag {
                            self.execute(&vec![*stmt.clone()], 0);
                            self.symbolic_store.block_end_states.pop();
                            self.execute(statements, cur_bid);
                        } else {
                            self.symbolic_store
                                .block_end_states
                                .push(self.cur_state.clone());
                        }
                    } else {
                        panic!("This tool currently cannot handle the symbolic condition of While Loop: {}", evaled_condition.lookup_fmt(&self.symbolic_library.id2name));
                    }

                    self.expand_all_stack_states(
                        statements,
                        cur_bid + 1,
                        self.cur_state.get_depth(),
                    );
                    // Note: This doesn't handle loop invariants or fixed-point computation
                }
                DebugStatement::Return { meta, value, .. } => {
                    self.trace_if_enabled(&meta);
                    let tmp_val = self.evaluate_expression(value);
                    let return_value =
                        self.fold_variables(&tmp_val, !self.setting.propagate_substitution);
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
                DebugStatement::Declaration { name, xtype, .. } => {
                    /*
                    let var_name = if dimensions.is_empty() {
                        format!("{}.{}", self.cur_state.get_owner(), name.clone())
                    } else {
                        //"todo".to_string()
                        format!("{}.{}<{}>", self.cur_state.get_owner(), name, &dimensions)
                    };*/
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
                DebugStatement::Substitution {
                    meta,
                    var,
                    access,
                    op,
                    rhe,
                } => {
                    self.trace_if_enabled(&meta);
                    let expr = self.evaluate_expression(rhe);
                    let original_value = self.fold_variables(&expr, true);
                    let value = self.fold_variables(&expr, !self.setting.propagate_substitution);

                    /*
                    let accessed_name = if access.is_empty() {
                        var.clone()
                    } else {
                        format!(
                            "{}{}",
                            var,
                            &access
                                .iter()
                                .map(|arg0: &DebugAccess| self.evaluate_access(&arg0.clone(),))
                                .map(|debug_access| debug_access.to_string())
                                .collect::<Vec<_>>()
                                .join("")
                        )
                    };
                    */
                    //let var_name =
                    //    format!("{}.{}", self.cur_state.get_owner(), accessed_name.clone());

                    let var_name = SymbolicName {
                        name: *var,
                        owner: self.cur_state.owner_name.clone(),
                        access: if access.is_empty() {
                            None
                        } else {
                            Some(
                                access
                                    .iter()
                                    .map(|arg0: &DebugAccess| self.evaluate_access(&arg0.clone()))
                                    .collect::<Vec<_>>(),
                            )
                        },
                    };

                    if self.setting.keep_track_unrolled_offset {
                        if self
                            .symbolic_library
                            .template_library
                            .contains_key(&self.cur_state.template_id)
                            && self.symbolic_library.template_library[&self.cur_state.template_id]
                                .var2type
                                .contains_key(&var.clone())
                        {
                            if let Some(&VariableType::Signal(SignalType::Output, _)) =
                                self.symbolic_library.template_library[&self.cur_state.template_id]
                                    .var2type
                                    .get(&var)
                            {
                                self.symbolic_library
                                    .template_library
                                    .get_mut(&self.cur_state.template_id)
                                    .unwrap()
                                    .unrolled_outputs
                                    .insert(var_name.clone());
                            }
                        }
                    }

                    self.cur_state.set_symval(var_name.clone(), value.clone());

                    if !access.is_empty() {
                        for acc in access {
                            if let DebugAccess::ComponentAccess(tmp_name) = acc {
                                if let Some(component) =
                                    self.symbolic_store.components_store.get_mut(&var_name)
                                {
                                    component
                                        .inputs
                                        .insert(tmp_name.clone(), Some(value.clone()));
                                }
                            }
                        }

                        if self.is_ready(&var_name) {
                            if !self.symbolic_store.components_store[&var_name].is_done {
                                let symbolic_library = &mut self.symbolic_library;
                                let mut subse =
                                    SymbolicExecutor::new(symbolic_library, self.setting);

                                let mut on = (*self.cur_state.owner_name.clone()).clone();
                                on.push(OwnerName {
                                    name: *var,
                                    counter: 0,
                                });
                                subse.cur_state.owner_name = Rc::new(on);
                                //subse.cur_state.add_owner(*var, 0);

                                let templ = &subse.symbolic_library.template_library[&self
                                    .symbolic_store
                                    .components_store[&var_name]
                                    .template_name];
                                subse.cur_state.set_template_id(
                                    self.symbolic_store.components_store[&var_name]
                                        .template_name
                                        .clone(),
                                );

                                for i in 0..(templ.template_parameter_names.len()) {
                                    let n = SymbolicName {
                                        name: templ.template_parameter_names[i],
                                        owner: subse.cur_state.owner_name.clone(),
                                        access: None,
                                    };
                                    subse.cur_state.set_rc_symval(
                                        n,
                                        self.symbolic_store.components_store[&var_name].args[i]
                                            .clone(),
                                    );
                                }

                                for (k, v) in self.symbolic_store.components_store[&var_name]
                                    .inputs
                                    .iter()
                                {
                                    let n = SymbolicName {
                                        name: *k,
                                        owner: subse.cur_state.owner_name.clone(),
                                        access: None,
                                    };
                                    subse.cur_state.set_symval(n, v.clone().unwrap());
                                }

                                if !self.setting.off_trace {
                                    trace!(
                                        "{}",
                                        format!("{}", "===========================").cyan()
                                    );
                                    trace!(
                                        "📞 Call {}",
                                        subse.symbolic_library.id2name[&self
                                            .symbolic_store
                                            .components_store[&var_name]
                                            .template_name]
                                    );
                                }

                                subse.execute(&templ.body.clone(), 0);

                                if subse.symbolic_store.final_states.len() > 1 {
                                    warn!("TODO: This tool currently cannot handle multiple branches within the callee.");
                                }
                                self.cur_state.trace_constraints.append(
                                    &mut subse.symbolic_store.final_states[0].trace_constraints,
                                );
                                self.cur_state.side_constraints.append(
                                    &mut subse.symbolic_store.final_states[0].side_constraints,
                                );
                                if !self.setting.off_trace {
                                    trace!(
                                        "{}",
                                        format!("{}", "===========================").cyan()
                                    );
                                }
                            }
                        }
                    }

                    match value {
                        SymbolicValue::Call(callee_name, args) => {
                            // Initializing the Template Component
                            if self
                                .symbolic_library
                                .template_library
                                .contains_key(&callee_name)
                            {
                                let mut comp_inputs: FxHashMap<usize, Option<SymbolicValue>> =
                                    FxHashMap::default();
                                for inp_name in &self.symbolic_library.template_library
                                    [&callee_name]
                                    .inputs
                                    .clone()
                                {
                                    comp_inputs.insert(inp_name.clone(), None);
                                }
                                let c = SymbolicComponent {
                                    template_name: callee_name.clone(),
                                    args: args.clone(),
                                    inputs: comp_inputs,
                                    is_done: false,
                                };
                                self.symbolic_store
                                    .components_store
                                    .insert(var_name.clone(), c);
                            }
                        }
                        _ => {
                            if self.symbolic_store.variable_types[var].0 != VariableType::Var {
                                let cont = SymbolicValue::BinaryOp(
                                    Rc::new(SymbolicValue::Variable(var_name.clone())),
                                    DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                    Rc::new(value),
                                );
                                self.cur_state.push_trace_constraint(&cont);

                                if let DebugAssignOp(AssignOp::AssignConstraintSignal) = op {
                                    let original_cont = SymbolicValue::BinaryOp(
                                        Rc::new(SymbolicValue::Variable(var_name)),
                                        DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                                        Rc::new(original_value),
                                    );
                                    self.cur_state.push_side_constraint(&original_cont);
                                }
                            }
                        }
                    }

                    self.execute(statements, cur_bid + 1);
                }
                DebugStatement::MultSubstitution {
                    meta, lhe, op, rhe, ..
                } => {
                    self.trace_if_enabled(&meta);

                    let lhe_val = self.evaluate_expression(lhe);
                    let rhe_val = self.evaluate_expression(rhe);
                    let simple_lhs = self.fold_variables(&lhe_val, true);
                    let lhs = self.fold_variables(&lhe_val, !self.setting.propagate_substitution);
                    let simple_rhs = self.fold_variables(&rhe_val, true);
                    let rhs = self.fold_variables(&rhe_val, !self.setting.propagate_substitution);

                    // Handle multiple substitution (simplified)
                    let cont = SymbolicValue::BinaryOp(
                        Rc::new(lhs),
                        DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                        Rc::new(rhs),
                    );
                    self.cur_state.push_trace_constraint(&cont);
                    if let DebugAssignOp(AssignOp::AssignConstraintSignal) = op {
                        // Handle multiple substitution (simplified)
                        let simple_cont = SymbolicValue::BinaryOp(
                            Rc::new(simple_lhs),
                            DebugExpressionInfixOpcode(ExpressionInfixOpcode::Eq),
                            Rc::new(simple_rhs),
                        );
                        self.cur_state.push_side_constraint(&simple_cont);
                    }
                    self.execute(statements, cur_bid + 1);
                }
                DebugStatement::ConstraintEquality { meta, lhe, rhe } => {
                    self.trace_if_enabled(&meta);

                    let lhe_val = self.evaluate_expression(lhe);
                    let rhe_val = self.evaluate_expression(rhe);
                    let original_lhs = self.fold_variables(&lhe_val, true);
                    let lhs = self.fold_variables(&lhe_val, !self.setting.propagate_substitution);
                    let original_rhs = self.fold_variables(&rhe_val, true);
                    let rhs = self.fold_variables(&rhe_val, !self.setting.propagate_substitution);

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

                    self.cur_state.push_trace_constraint(&cond);
                    self.cur_state.push_side_constraint(&original_cond);

                    self.execute(statements, cur_bid + 1);
                }
                DebugStatement::Assert { meta, arg, .. } => {
                    self.trace_if_enabled(&meta);
                    let expr = self.evaluate_expression(&arg);
                    let condition =
                        self.fold_variables(&expr, !self.setting.propagate_substitution);
                    self.cur_state.push_trace_constraint(&condition);
                    self.execute(statements, cur_bid + 1);
                }
                DebugStatement::UnderscoreSubstitution {
                    meta,
                    op: _,
                    rhe: _,
                    ..
                } => {
                    self.trace_if_enabled(&meta);
                    // Underscore substitution doesn't affect the symbolic state
                }
                DebugStatement::LogCall { meta, args: _, .. } => {
                    self.trace_if_enabled(&meta);
                    // Logging doesn't affect the symbolic state
                }
                DebugStatement::Ret => {
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
        } else {
            self.symbolic_store
                .block_end_states
                .push(self.cur_state.clone());
        }
    }

    pub fn concrete_execute(
        &mut self,
        id: &String,
        assignment: &FxHashMap<SymbolicName, BigInt>,
        off_trace: bool,
    ) {
        self.cur_state.template_id = self.symbolic_library.name2id[id];
        for (vname, value) in assignment.into_iter() {
            self.cur_state
                .set_symval(vname.clone(), SymbolicValue::ConstantInt(value.clone()));
        }

        //self.setting.skip_initialization_blocks = true;
        //self.setting.off_trace = off_trace;
        self.execute(
            &self.symbolic_library.template_library[&self.cur_state.template_id]
                .body
                .clone(),
            0,
        );
    }

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
                SymbolicAccess::ArrayAccess(self.fold_variables(&tmp_e, false))
            }
        }
    }

    fn fold_variables(
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
                                return symval.clone();
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
                    if let Some(boxed_value) = self.cur_state.get_symval(&sname) {
                        if let SymbolicValue::ConstantInt(v) = (*boxed_value.clone()).clone() {
                            return SymbolicValue::ConstantInt(v);
                        }
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
                let lhs = self.fold_variables(lv, only_constatant_folding);
                let rhs = self.fold_variables(rv, only_constatant_folding);
                match (&lhs, &rhs) {
                    (SymbolicValue::ConstantInt(lv), SymbolicValue::ConstantInt(rv)) => {
                        match &infix_op.0 {
                            ExpressionInfixOpcode::Add => {
                                SymbolicValue::ConstantInt((lv + rv) % &self.setting.prime)
                            }
                            ExpressionInfixOpcode::Sub => {
                                SymbolicValue::ConstantInt((lv - rv) % &self.setting.prime)
                            }
                            ExpressionInfixOpcode::Mul => {
                                SymbolicValue::ConstantInt((lv * rv) % &self.setting.prime)
                            }
                            ExpressionInfixOpcode::Div => {
                                if rv.is_zero() {
                                    SymbolicValue::ConstantInt(BigInt::zero())
                                } else {
                                    let mut r = self.setting.prime.clone();
                                    let mut new_r = rv.clone();
                                    if r.is_negative() {
                                        r += &self.setting.prime;
                                    }
                                    if new_r.is_negative() {
                                        new_r += &self.setting.prime;
                                    }

                                    let (_, _, mut rv_inv) = extended_euclidean(r, new_r);
                                    rv_inv %= self.setting.prime.clone();
                                    if rv_inv.is_negative() {
                                        rv_inv += &self.setting.prime;
                                    }

                                    SymbolicValue::ConstantInt((lv * rv_inv) % &self.setting.prime)
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
                            ExpressionInfixOpcode::Lesser => SymbolicValue::ConstantBool(
                                lv % &self.setting.prime < rv % &self.setting.prime,
                            ),
                            ExpressionInfixOpcode::Greater => SymbolicValue::ConstantBool(
                                lv % &self.setting.prime > rv % &self.setting.prime,
                            ),
                            ExpressionInfixOpcode::LesserEq => SymbolicValue::ConstantBool(
                                lv % &self.setting.prime <= rv % &self.setting.prime,
                            ),
                            ExpressionInfixOpcode::GreaterEq => SymbolicValue::ConstantBool(
                                lv % &self.setting.prime >= rv % &self.setting.prime,
                            ),
                            ExpressionInfixOpcode::Eq => SymbolicValue::ConstantBool(
                                lv % &self.setting.prime == rv % &self.setting.prime,
                            ),
                            ExpressionInfixOpcode::NotEq => SymbolicValue::ConstantBool(
                                lv % &self.setting.prime != rv % &self.setting.prime,
                            ),
                            _ => SymbolicValue::BinaryOp(
                                Rc::new(lhs),
                                infix_op.clone(),
                                Rc::new(rhs),
                            ),
                        }
                    }
                    (SymbolicValue::ConstantBool(lv), SymbolicValue::ConstantBool(rv)) => {
                        match &infix_op.0 {
                            ExpressionInfixOpcode::BoolAnd => {
                                SymbolicValue::ConstantBool(*lv && *rv)
                            }
                            ExpressionInfixOpcode::BoolOr => {
                                SymbolicValue::ConstantBool(*lv || *rv)
                            }
                            _ => SymbolicValue::BinaryOp(
                                Rc::new(lhs),
                                infix_op.clone(),
                                Rc::new(rhs),
                            ),
                        }
                    }
                    _ => SymbolicValue::BinaryOp(Rc::new(lhs), infix_op.clone(), Rc::new(rhs)),
                }
            }
            SymbolicValue::Conditional(cond, then_val, else_val) => SymbolicValue::Conditional(
                Rc::new(self.fold_variables(cond, only_constatant_folding)),
                Rc::new(self.fold_variables(then_val, only_constatant_folding)),
                Rc::new(self.fold_variables(else_val, only_constatant_folding)),
            ),
            SymbolicValue::UnaryOp(prefix_op, value) => {
                let folded_symval = self.fold_variables(value, only_constatant_folding);
                match &folded_symval {
                    SymbolicValue::ConstantInt(rv) => match prefix_op.0 {
                        ExpressionPrefixOpcode::Sub => SymbolicValue::ConstantInt(-1 * rv),
                        _ => SymbolicValue::UnaryOp(prefix_op.clone(), Rc::new(folded_symval)),
                    },
                    SymbolicValue::ConstantBool(rv) => match prefix_op.0 {
                        ExpressionPrefixOpcode::BoolNot => SymbolicValue::ConstantBool(!rv),
                        _ => SymbolicValue::UnaryOp(prefix_op.clone(), Rc::new(folded_symval)),
                    },
                    _ => SymbolicValue::UnaryOp(prefix_op.clone(), Rc::new(folded_symval)),
                }
            }
            SymbolicValue::Array(elements) => SymbolicValue::Array(
                elements
                    .iter()
                    .map(|e| Rc::new(self.fold_variables(e, only_constatant_folding)))
                    .collect(),
            ),
            SymbolicValue::Tuple(elements) => SymbolicValue::Tuple(
                elements
                    .iter()
                    .map(|e| Rc::new(self.fold_variables(e, only_constatant_folding)))
                    .collect(),
            ),
            SymbolicValue::UniformArray(element, count) => SymbolicValue::UniformArray(
                Rc::new(self.fold_variables(element, only_constatant_folding)),
                Rc::new(self.fold_variables(count, only_constatant_folding)),
            ),
            SymbolicValue::Call(func_name, args) => SymbolicValue::Call(
                func_name.clone(),
                args.iter()
                    .map(|arg| Rc::new(self.fold_variables(arg, only_constatant_folding)))
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
            DebugExpression::Number(_meta, value) => SymbolicValue::ConstantInt(value.clone()),
            DebugExpression::Variable {
                name,
                access,
                meta: _,
            } => {
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
                    let evaluated_access = access
                        .iter()
                        .map(|arg0: &DebugAccess| self.evaluate_access(&arg0))
                        .collect::<Vec<_>>();

                    if evaluated_access.len() == 1 && sv.is_some() {
                        if let SymbolicAccess::ArrayAccess(SymbolicValue::ConstantInt(a)) =
                            &evaluated_access[0]
                        {
                            match (*sv.unwrap().clone()).clone() {
                                SymbolicValue::Array(values) => {
                                    return (*values[a.to_usize().unwrap()].clone()).clone();
                                }
                                _ => {}
                            }
                        }
                    }

                    SymbolicName {
                        name: *name,
                        owner: self.cur_state.owner_name.clone(),
                        access: if evaluated_access.is_empty() {
                            None
                        } else {
                            Some(evaluated_access)
                        },
                    }
                };
                SymbolicValue::Variable(resolved_name)
            }
            DebugExpression::InfixOp {
                meta: _,
                lhe,
                infix_op,
                rhe,
            } => {
                let lhs = self.evaluate_expression(lhe);
                let rhs = self.evaluate_expression(rhe);
                SymbolicValue::BinaryOp(Rc::new(lhs), infix_op.clone(), Rc::new(rhs))
            }
            DebugExpression::PrefixOp {
                meta: _,
                prefix_op,
                rhe,
            } => {
                let expr = self.evaluate_expression(rhe);
                SymbolicValue::UnaryOp(prefix_op.clone(), Rc::new(expr))
            }
            DebugExpression::InlineSwitchOp {
                meta: _,
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
            DebugExpression::ArrayInLine { meta: _, values } => {
                let elements = values
                    .iter()
                    .map(|v| Rc::new(self.evaluate_expression(v)))
                    .collect();
                SymbolicValue::Array(elements)
            }
            DebugExpression::Tuple { meta: _, values } => {
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
                let tmp_args: Vec<_> = args
                    .iter()
                    .map(|arg| self.evaluate_expression(arg))
                    .collect();
                let evaluated_args = tmp_args
                    .iter()
                    .map(|arg| Rc::new(self.fold_variables(&arg, false)))
                    .collect();
                if self.symbolic_library.template_library.contains_key(id) {
                    SymbolicValue::Call(id.clone(), evaluated_args)
                } else if self.symbolic_library.function_library.contains_key(id) {
                    let symbolic_library = &mut self.symbolic_library;
                    let mut subse = SymbolicExecutor::new(symbolic_library, self.setting);

                    let mut on = (*self.cur_state.owner_name.clone()).clone();
                    on.push(OwnerName {
                        name: *id,
                        counter: subse.symbolic_library.function_counter[id],
                    });
                    subse.cur_state.owner_name = Rc::new(on);
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
                            .set_rc_symval(sname, evaluated_args[i].clone());
                    }

                    if !subse.setting.off_trace {
                        trace!("{}", format!("{}", "===========================").cyan());
                        trace!("📞 Call {}", subse.symbolic_library.id2name[id]);
                    }

                    subse.execute(&func.body.clone(), 0);

                    if subse.symbolic_store.final_states.len() > 1 {
                        warn!("TODO: This tool currently cannot handle multiple branches within the callee.");
                    }

                    self.cur_state
                        .trace_constraints
                        .append(&mut subse.symbolic_store.final_states[0].trace_constraints);
                    self.cur_state
                        .side_constraints
                        .append(&mut subse.symbolic_store.final_states[0].side_constraints);

                    if !subse.setting.off_trace {
                        trace!("{}", format!("{}", "===========================").cyan());
                    }

                    let sname = SymbolicName {
                        name: usize::MAX,
                        owner: subse.symbolic_store.final_states[0].owner_name.clone(),
                        access: None,
                    };

                    (*subse.symbolic_store.final_states[0].values[&sname].clone()).clone()
                } else {
                    error!("Unknown Callee: {}", id);
                    SymbolicValue::Call(id.clone(), evaluated_args)
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
