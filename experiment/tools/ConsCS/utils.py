import re
import copy
import time
import os
import yaml
from sympy import symbols, groebner, GF, Poly, prod
from z3 import *
import signal
from datetime import datetime

from misc import * 
from r1cs_utils import *
from data_structures import *


############################## Vars ##############################

sympy_symbols = {}
counter_example = dict()

target_file_path_g = None

DO_SIMPLIFICATION_PHASE = True
DO_BPG_PHASE = True
DO_ASSUMPTION_PHASE = True
BPG_DEPTH = 4

NO_CLAUSE_LEARNING = False

cur_processing_cons_id = 0
equivalent_vars = UnionFind() # one group is one group of equivalent vars
equivalent_cons_vars = UnionFind() # one group is one group of equivalent vars
edge_id_to_edge = dict() # int (edge ID) -> Edge
cons_id_to_edges = dict() # int (constraint ID) -> set of edge IDs
replacement_dict = dict() # tuple (one term) -> sdp
binary_vars = set() # a set of vars proven to be binary
var_to_cons_id = dict() # int (one var) -> dict(tuple -> set()) (latter dict has tuple as the tuple 
                        # the one var appears in, and the set() is set of all cons ID the var appears in) 
                        # Should be a sparse set.
term_to_constrainedness = dict() # tuple -> index, with 1: constrained, 2: NOT SURE, 3: UNDERconstrained
# candidate_hypothesis = dict() # constraint IDs -> list(hypothesis), which we could leverage for SMT solving and information unearthing
linear_subs_back_solving = dict() # key are single vars, value is a list of all expressions (all vars that were reduced to it) of substitution in the form of a dict. # idea is that following this, we can use all known values to solve for all eliminated variables
linear_subs_back_solving_constants = dict() # keeps track of all constants that were subed. map var to a value.

contribution_count = dict() # for counting the contribution of different design components.

DEBUG = 0

assumptions = Hypothesis_Assumptions()
bpg_graph = BPG()




############################## Working with global info ##############################


def set_bpg_depth(new_depth):
    global BPG_DEPTH
    BPG_DEPTH = new_depth


def set_flags(SIMPLIFICATION_flag, BPG_flag, ASSUMPTION_flag):
    global DO_SIMPLIFICATION_PHASE
    global DO_BPG_PHASE
    global DO_ASSUMPTION_PHASE
    
    DO_SIMPLIFICATION_PHASE = SIMPLIFICATION_flag
    DO_BPG_PHASE = BPG_flag
    DO_ASSUMPTION_PHASE = ASSUMPTION_flag



def disable_clause_learning():
    global NO_CLAUSE_LEARNING
    NO_CLAUSE_LEARNING = True



def get_cur_processing_cons_id():
    global cur_processing_cons_id
    return cur_processing_cons_id


def increment_cur_processing_cons_id():
    global cur_processing_cons_id
    cur_processing_cons_id += 1
    
   
def add_contribution_count(name_contribution, contribution_content):
    if name_contribution not in contribution_count:
        contribution_count[name_contribution] = set()
    contribution_count[name_contribution].add(contribution_content)


def print_contribution_count():
    for k in contribution_count:
        if k == 'bpg_time_records':
            print(k, contribution_count[k])
        else:
            print(k, len(contribution_count[k]))

############################## SMT ##############################
def has_raw_trivial_solution(r1cs_circuit):
    solver = Solver()
    all_terms_d = get_all_terms_d_of_r1cs(r1cs_circuit)
    l_z3_expr = list_of_terms_d_to_z3(all_terms_d)
    add_0_constraints_to_solver(solver, l_z3_expr)
    add_all_finite_field_range_restrictions(solver)
    add_binary_restrictions(solver)
    
    result, model_solution = check_results(solver)
    return result is not None


def main_solve(target_file_path):
    # Entry point for solving a circuit
    # returns result, time_taken
    
    global DO_SIMPLIFICATION_PHASE
    global DO_ASSUMPTION_PHASE
    global DO_BPG_PHASE
    global target_file_path_g
    global counter_example
    
    target_file_path_g = target_file_path
    
    print_all_circuits = 0
    clear_data_structures()

    start_time_one_iteration = datetime.now()

    ################## START ##################
    start_time = time.time() 
    timeout_reasoning_engine = 7200
    timeout_assumptions_smt = 5
    
    try:
        r1cs_circuit = parse_entire_circuit(target_file_path, print_all_circuits = print_all_circuits, timeout_time = timeout_reasoning_engine, starting_time = start_time)
    except TimeoutError as e:
        return "TIMEOUT", time.time() - start_time, None
    
    assert r1cs_circuit is not None
    
    
    constraint_id = get_cur_processing_cons_id()
    while constraint_id < len(r1cs_circuit.all_constraints):
        terminated, message = check_termination(r1cs_circuit)
        if terminated:
            break
        if time.time() - start_time > timeout_reasoning_engine:
            return "TIMEOUT", time.time() - start_time, None

        cur_sdp = r1cs_circuit.all_constraints[constraint_id]
        constraint_info = get_constraint_info(r1cs_circuit, constraint_id, cur_sdp)
        
        if DO_SIMPLIFICATION_PHASE:
            run_simplification(cur_sdp, constraint_info)
        if DO_BPG_PHASE:
            construct_BPG_edges(cur_sdp, constraint_info)

        increment_cur_processing_cons_id()
        constraint_id = get_cur_processing_cons_id()

    terminated, message = check_termination(r1cs_circuit)
    
    counterexample = None
    progress_made = True
    while progress_made:
        progress_made = False
        # calls to simplification/deduction are internally handled by address_backwards_dependency
        if time.time() - start_time > timeout_reasoning_engine:
            return "TIMEOUT", time.time() - start_time, None

        bpg_progress_made = False
        if DO_BPG_PHASE:
            try:
                affected_vars, replacements = traverse_BPG_for_info(start_time, timeout_reasoning_engine)
            except TimeoutError as e:
                return "TIMEOUT", time.time() - start_time, None

            if len(affected_vars) > 0:
                address_backwards_dependency(affected_vars, None, r1cs_circuit, len(r1cs_circuit.all_constraints) + 10)
                bpg_progress_made = True
            if len(replacements) > 0:
                for replacement in replacements:
                    address_backwards_dependency(None, replacement, r1cs_circuit, len(r1cs_circuit.all_constraints) + 10)
                bpg_progress_made = True
            terminated, message = check_termination(r1cs_circuit)
            if terminated:
                break
        
        if time.time() - start_time > timeout_reasoning_engine:
            return "TIMEOUT", time.time() - start_time, None
            
        smt_progress_made = False
        if DO_ASSUMPTION_PHASE:
            add_contribution_count("DO_ASSUMPTION_PHASE", 1)

            found_UNDER, affected_vars, counterexample = try_assumptions_smt(r1cs_circuit, timeout_assumptions_smt)
            if found_UNDER:
                terminated, message = check_termination(r1cs_circuit)
                assert terminated
                break
            elif len(affected_vars) > 0:
                address_backwards_dependency(affected_vars, None, r1cs_circuit, len(r1cs_circuit.all_constraints) + 10)
                smt_progress_made = True

            if time.time() - start_time > timeout_reasoning_engine:
                return "TIMEOUT", time.time() - start_time, None


            # a regular SMT call
            found_UNDER, affected_vars, counterexample = solve_smt_raw(r1cs_circuit, timeout_assumptions_smt)
            if found_UNDER:
                terminated, message = check_termination(r1cs_circuit)
                assert terminated
                break
            elif len(affected_vars) > 0:
                address_backwards_dependency(affected_vars, None, r1cs_circuit, len(r1cs_circuit.all_constraints) + 10)
                smt_progress_made = True
        

        progress_made = bpg_progress_made or smt_progress_made
                  
                
        terminated, message = check_termination(r1cs_circuit)
        if terminated:
            break
        
    ################## OVER ##################
    elapsed_time = (datetime.now() - start_time_one_iteration).total_seconds()
    terminated, message = check_termination(r1cs_circuit)
    # If UNDER, verify the counterexamples
    reconstruct_and_try_full_counter_examples(r1cs_circuit)
    
    if "solved_assumption_smt_count" in contribution_count:
        if len(contribution_count['solved_assumption_smt_count']) > 0:
            if message == "\n\nCONSTRAINED!":
                if not has_raw_trivial_solution(r1cs_circuit):
                    message = "NOT SURE"
    
    return message.strip(), elapsed_time, counter_example
    
    

    
    
def solve_smt_raw(r1cs_circuit, timeout_seconds = 5):
    # if assumptions did not work, then try 1 round of direct solving
    # Returns: found_UNDER, affected_vars, counterexample
    affected_vars = set()
    
    
    pre_encoded_solver = Solver()    
                    
    all_terms_d = get_all_terms_d_of_r1cs(r1cs_circuit)
    l_z3_expr = list_of_terms_d_to_z3(all_terms_d)
    add_0_constraints_to_solver(pre_encoded_solver, l_z3_expr)
    add_all_finite_field_range_restrictions(pre_encoded_solver)
    add_binary_restrictions(pre_encoded_solver)

    result, model_solution = check_results(pre_encoded_solver)
    if result == "SAT":
        # If one solution found, then try a second query by pushing, and then adding requirement of same input but diff output. with a timeout 
        target_var = None
        found_alternative_sol = handle_sat_result(pre_encoded_solver, model_solution, target_var, "NO_Assumptions_raw_SMT_UNDER", r1cs_circuit)
        if found_alternative_sol is not None:
            return True, None, found_alternative_sol

    terminated, message = check_termination(r1cs_circuit)
    if terminated:
        return True, None, None

    return False, affected_vars, None
    
    

def try_assumptions_smt(r1cs_circuit, timeout_seconds = 5):
    # performs the assumption-guided SMT solving
    # Returns: found_UNDER, affected_vars, counterexample
    
    global assumptions
    global counter_example
    global edge_id_to_edge
    global contribution_count
    
    
    
    start_time = time.time() 
    affected_vars = set()
    
    # enables pushing and popping of the solver states.
    pre_encoded_solver = Solver()
    all_terms_d = get_all_terms_d_of_r1cs(r1cs_circuit)
    l_z3_expr = list_of_terms_d_to_z3(all_terms_d)
    add_0_constraints_to_solver(pre_encoded_solver, l_z3_expr)
    
    
    from datetime import datetime
    start = datetime.now()

    for target_var, edges in assumptions.assumptions.items():
        if is_constrained((target_var,)) or is_underconstrained((target_var,)):
            # already solved, no need to proceed
            continue
        if target_var in affected_vars:
            continue
                
        for edge_id in edges:
            # try each assumption
            if time.time() - start_time > timeout_seconds:
                return False, affected_vars, None

            if edge_id not in edge_id_to_edge:
                continue
            
            if target_var in affected_vars:
                continue
            breaked = False
            # for each assumption of the target variable
            edge = edge_id_to_edge[edge_id]
            
            if "assumption_smt_count" not in contribution_count: 
                add_contribution_count("assumption_smt_count", 1)
            else:
                add_contribution_count("assumption_smt_count", max(contribution_count["assumption_smt_count"]) + 1)
            
            end_time = datetime.utcnow()
            elapsed_time = (datetime.now() - start).total_seconds()
            print(f"considering edge {edge_id}  {elapsed_time}")
            for grobner_method in [1,2]:
                if target_var in affected_vars:
                    continue
                end_time = datetime.utcnow()
                elapsed_time = (datetime.now() - start).total_seconds()
                print(f"trying grobner {grobner_method} {elapsed_time}")
                G, l_expr = get_grobner_basis(r1cs_circuit, with_hypothesis = edge, timeout=5, grobner_method = grobner_method)
                if G is not None:
                    end_time = datetime.utcnow()
                    elapsed_time = (datetime.now() - start).total_seconds()
                    print(f"found grobner {elapsed_time}")
                    # check for early conflicts
                    cur_solver = Solver()

                    conflict_found = clause_learning(G, cur_solver, [], time.time(), timeout_seconds)
                    if conflict_found:
                        end_time = datetime.utcnow()
                        elapsed_time = (datetime.now() - start).total_seconds()
                        print(f"conflict_found  {elapsed_time}")
                        # found a conflict, the target var is constrained
                        add_contribution_count("Assumptions_grobner_early_conflict_cons", target_var)
                        
                        if "solved_assumption_smt_count" not in contribution_count: 
                            add_contribution_count("solved_assumption_smt_count", 1)
                        else:
                            add_contribution_count("solved_assumption_smt_count", max(contribution_count["solved_assumption_smt_count"]) + 1)
                
                        set_constrained((target_var,))
                        affected_vars.add(target_var)
                        continue # since the current var has already been proved
                    else:
                        end_time = datetime.utcnow()
                        elapsed_time = (datetime.now() - start).total_seconds()
                        print(f"conflict NOT found  {elapsed_time}")
                        # if no early conflict found, then invoke SMT on grobner
                        z3l = [sympy_to_z3(one_cons) for one_cons in l_expr]
                        add_0_constraints_to_solver(cur_solver, z3l)
                        add_all_finite_field_range_restrictions(cur_solver)
                        add_binary_restrictions(cur_solver)
                        result, model_solution = check_results(cur_solver)
                        if result == "SAT":
                            end_time = datetime.utcnow()
                            elapsed_time = (datetime.now() - start).total_seconds()
                            print(f"BUT SAT FOUND  {elapsed_time}")
                            # If one solution found, then try a second query by pushing, and then adding requirement of same input but diff output. with a timeout 
                            found_alternative_sol = handle_sat_result(cur_solver, model_solution, target_var, "Assumptions_via_grobner_UNDER", r1cs_circuit)
                            if found_alternative_sol is not None:
                                end_time = datetime.utcnow()
                                elapsed_time = (datetime.now() - start).total_seconds()
                                print(f"COUNTER FOUND  {elapsed_time}")
                                if "unsolved_assumption_smt_count" not in contribution_count: 
                                    add_contribution_count("unsolved_assumption_smt_count", 1)
                                else:
                                    add_contribution_count("unsolved_assumption_smt_count", max(contribution_count["unsolved_assumption_smt_count"]) + 1)
                                return True, None, found_alternative_sol

                            found, found_alternative_sols = try_alternative_solution(cur_solver, model_solution, target_var, r1cs_circuit)
                        elif result == "UNSAT":
                            # constrained
                            add_contribution_count("Assumptions_via_grobner_cons", target_var)
                            if "solved_assumption_smt_count" not in contribution_count: 
                                add_contribution_count("solved_assumption_smt_count", 1)
                            else:
                                add_contribution_count("solved_assumption_smt_count", max(contribution_count["solved_assumption_smt_count"]) + 1)

                            set_constrained((target_var,))
                            affected_vars.add(target_var)
                            
                        terminated, message = check_termination(r1cs_circuit)
                        if terminated:
                            return True, None, None
                        # if timeout, just give up this one.

            if target_var in affected_vars:
                continue
                    
            # Grobner part done. move on to direct input of the constraint system            
            # entire r1cs is already encoded in pre_encoded_solver

            # add hypothesis
            push_state(pre_encoded_solver)
            
            
            conflict_found = clause_learning_sdp(r1cs_circuit, pre_encoded_solver, [], time.time(), timeout_seconds)
            if conflict_found:
                # found a conflict, the target var is constrained
                add_contribution_count("Assumptions_sdp_early_conflict_cons", target_var)
                if "solved_assumption_smt_count" not in contribution_count: 
                    add_contribution_count("solved_assumption_smt_count", 1)
                else:
                    add_contribution_count("solved_assumption_smt_count", max(contribution_count["solved_assumption_smt_count"]) + 1)
                            
                set_constrained((target_var,))
                affected_vars.add(target_var)
                break 
                      
            l_z3_expr = list_of_terms_d_to_z3([edge.condition])
            add_0_constraints_to_solver(pre_encoded_solver, l_z3_expr)
            add_all_finite_field_range_restrictions(pre_encoded_solver)
            add_binary_restrictions(pre_encoded_solver)

            result, model_solution = check_results(pre_encoded_solver)
            if result == "SAT":
                # If one solution found, then try a second query by pushing, and then adding requirement of same input but diff output. with a timeout 
                found_alternative_sol = handle_sat_result(pre_encoded_solver, model_solution, target_var, "Assumptions_sdp_SMT_UNDER", r1cs_circuit)
                if found_alternative_sol is not None:
                    if "unsolved_assumption_smt_count" not in contribution_count: 
                        add_contribution_count("unsolved_assumption_smt_count", 1)
                    else:
                        add_contribution_count("unsolved_assumption_smt_count", max(contribution_count["unsolved_assumption_smt_count"]) + 1)
                            
                    return True, None, found_alternative_sol
            elif result == "UNSAT":
                # constrained
                add_contribution_count("Assumptions_sdp_SMT_cons", target_var)
                
                if "solved_assumption_smt_count" not in contribution_count: 
                    add_contribution_count("solved_assumption_smt_count", 1)
                else:
                    add_contribution_count("solved_assumption_smt_count", max(contribution_count["solved_assumption_smt_count"]) + 1)
                            
                set_constrained((target_var,))
                affected_vars.add(target_var)
           
            pop_state(pre_encoded_solver)
            
            
            # check for termination after each assumption was checked
            terminated, message = check_termination(r1cs_circuit)
            if terminated:
                return True, None, None
    return False, affected_vars, None



def try_alternative_solution(solver, one_solution, target_var, r1cs_circuit):
    # tries to find a counter example from a known solution
    # Returns: found, found_alternative_sol
    
    # NOTE: if some output variable does not show up in the solution,
        # it means they were either canceled out (by indirect subs) or never showed up (we don't remove inputs/outputs intentionally)
            
    # NOTE: if this happens to input vars, this means some input vars do not matter,
        # then we just ignore them and input any input var we have. just assign trivial 0.
    global target_file_path_g
    
    input_var_strs = [f"x{v}" for v in r1cs_circuit.input_list if v != 0] # no need to consider constants
    output_var_strs = [f"x{v}" for v in r1cs_circuit.output_list]
    
    for one_var_str in input_var_strs:
        if one_var_str not in one_solution:
            one_solution[one_var_str] = 0
    
    # handled output vars that never show up
    alternative_solution = None
    for one_var_str in output_var_strs:
        if one_var_str not in one_solution:
            # one output variable does not show up, UNDER
            if alternative_solution is None:
                alternative_solution = one_solution.copy()
            assert one_var_str not in alternative_solution
            alternative_solution[one_var_str] = 0
            one_solution[one_var_str] = 1
    
    if alternative_solution is not None:
        return True, (one_solution, alternative_solution)

    input_part = {get_variable(one_var):one_solution[one_var] for one_var in input_var_strs}
    output_part = {get_variable(one_var):one_solution[one_var] for one_var in output_var_strs}
    
    push_state(solver)
    add_same_input_constraints(solver, input_part)
    add_different_output_constraints(solver, output_part)
    
    if target_var is not None and f"x{target_var}" in one_solution:
        add_different_target_var_constraints(solver, get_variable(f"x{target_var}"), one_solution[f"x{target_var}"])
        
        
        
    result, model_solution = check_results(solver)
    pop_state(solver)
    if result == "SAT":
        # returns found, found_alternative_sol
        return True, (one_solution, model_solution)
    elif result == "UNSAT":
        return False, None
    else:
        print("REDIRECT")
        print(main_solve(target_file_path_g))
        return False, None
        


            

    
    
def add_same_input_constraints(solver, input_values):
    for var, value in input_values.items():
        solver.add((var + p) % p == value)
        

def add_different_output_constraints(solver, output_values):
    output_constraints = [(var + p) % p != value for var, value in output_values.items()]
    solver.add(Or(*output_constraints))


def add_different_target_var_constraints(solver, target_var, target_val):
    target_var_constraint = (target_var + p) % p != target_val
    solver.add(target_var_constraint)
    

    
 
def reconstruct_and_try_full_counter_examples(r1cs_circuit):
    # verifies counter-examples
    # for those variables that were eliminated due to inference rules, we reconstruct their value
    # the verification is conducted on the original system, not the simplified version, to guarantee the full correctness.
    
    global linear_subs_back_solving_constants
    global linear_subs_back_solving
    global counter_example
    
    # adding in solved/eliminated vars
    # directly use constants, solve linear variables
    for v in counter_example:
        one_solution, alternative_solution = counter_example[v]
        
        for one_var in linear_subs_back_solving_constants:
            assert one_var not in one_solution and one_var not in alternative_solution
            one_solution[one_var] = linear_subs_back_solving_constants[one_var]
            alternative_solution[one_var] = linear_subs_back_solving_constants[one_var]
            
            
        progress_made_here = True
        while progress_made_here:
            progress_made_here = False
            for one_var in linear_subs_back_solving:
                # goal: find the value of one_var
                if one_var not in one_solution:
                    assert one_var not in alternative_solution

                    for exp_variable, expression in linear_subs_back_solving[one_var]:
                        if exp_variable in one_solution:
                            assert exp_variable in alternative_solution
                            # current expression: replaced_var <- exp_constant_val + exp_variable_coef * exp_variable
                            exp_constant_val = expression[()]
                            exp_variable_coef = expression[(exp_variable,)]

                            one_solution[one_var] = mul_constants(one_solution[exp_variable], exp_variable_coef)
                            one_solution[one_var] = add_constants(one_solution[one_var], exp_constant_val)

                            alternative_solution[one_var] = mul_constants(alternative_solution[exp_variable], exp_variable_coef)
                            alternative_solution[one_var] = add_constants(alternative_solution[one_var], exp_constant_val)
                            progress_made_here = True
    
        assert len(alternative_solution) == r1cs_circuit.header_vars.nwires - 1 # accounting for constants
        assert len(one_solution) == r1cs_circuit.header_vars.nwires - 1
        
    
        # ensuring the definition of underconstrainedness
        for one_input_var in r1cs_circuit.input_list:
#             one_input_var = f"x{one_input_var}"
            assert one_solution[one_input_var] == alternative_solution[one_input_var]
            
        has_different_output = False
        for one_output_var in r1cs_circuit.output_list:
#             one_output_var = f"x{one_output_var}"
            if one_solution[one_output_var] != alternative_solution[one_output_var]:
                has_different_output = True
                break
        assert has_different_output
        
        
        # ensuring they are valid solutions
        one_solution_r1cs = get_r1cs_circuit_copy(r1cs_circuit)
        alternative_solution_r1cs = get_r1cs_circuit_copy(r1cs_circuit)
        
        for one_sol_var in one_solution:
            one_solution_expression_sdp = SingleDotProduct({():one_solution[one_sol_var]})
            alternative_solution_expression_sdp = SingleDotProduct({():alternative_solution[one_sol_var]})
            
            assert isinstance(one_sol_var, int)
            for cur_sdp in one_solution_r1cs.all_constraints:
                cur_sdp.plug_in_expression_of_var_IN_PLACE(one_sol_var, one_solution_expression_sdp)
                
            for cur_sdp in alternative_solution_r1cs.all_constraints:
                cur_sdp.plug_in_expression_of_var_IN_PLACE(one_sol_var, alternative_solution_expression_sdp)
                
        for cur_sdp in one_solution_r1cs.all_constraints:
            assert cur_sdp.terms == {():0}, cur_sdp.terms
            
        for cur_sdp in alternative_solution_r1cs.all_constraints:
            assert cur_sdp.terms == {():0}, cur_sdp.terms
        
        
             
            
    

def handle_sat_result(cur_solver, model_solution, target_var, contribution_info, r1cs_circuit):
    # handles the addition of counter examples.
    global counter_example

    found, found_alternative_sols = try_alternative_solution(cur_solver, model_solution, target_var, r1cs_circuit)
    if found:
        model_solution, found_alternative_sol = found_alternative_sols
        for v1 in model_solution:
            assert v1 in found_alternative_sol
            if model_solution[v1] != found_alternative_sol[v1]:
                assert v1.startswith('x')
                add_contribution_count(contribution_info, int(v1[1:]))
                set_underconstrained((int(v1[1:]),))
        assert target_var not in counter_example
        
        new_model_solution = dict()
        for k in model_solution:
            assert k.startswith('x')
            new_model_solution[int(k[1:])] = model_solution[k]
            
        new_found_alternative_sol = dict()
        for k in found_alternative_sol:
            assert k.startswith('x')
            new_found_alternative_sol[int(k[1:])] = found_alternative_sol[k]
            
        counter_example[target_var] = (new_model_solution, new_found_alternative_sol)
        return found_alternative_sol
    return None        
    
    
    
    
    
    
############################## Sympy ##############################


def get_symbol_for_var(var_name):
    global sympy_symbols
    symbol_name = f'x{var_name}'
    
    if symbol_name not in sympy_symbols:
        sympy_symbols[symbol_name] = symbols(symbol_name)
    
    return sympy_symbols[symbol_name]



def get_sympy_expression_of_term(var_pair, coef):
    # Get the sympy expression of a single term, with coefficient
    cur_sympy_term = get_constant_gf_p(coef)

    for i in var_pair:
        cur_sympy_term *= get_symbol_for_var(i)
    return cur_sympy_term
    

def get_sympy_expression_of_sdp(sdp):
    # converts a sdp into sympy symbolic expression, where variables are converted according to var_dict.
    
    final_expression = 0
    for var_pair, coef in sdp.terms.items():
        cur_sympy_term = get_sympy_expression_of_term(var_pair, coef)
        
        final_expression += cur_sympy_term
    
    poly = Poly(final_expression, domain=GF(p))
    return poly




def sdp_is_binary_indicator(sdp):
    # checks if an sdp is a binary expression.
    # returns is_binary, binary_var
    
    global binary_vars
    
    target_terms = [t for t in sdp.terms if t != ()]
    if len(target_terms) != 2:
        return False, None
    
    if len(target_terms[0]) == 1 and len(target_terms[1]) == 1:
        return False, None
    
    if len(target_terms[0]) == 2 and len(target_terms[1]) == 2:
        return False, None
    
    if sdp.terms.get((), 0) != 0:
        return False, None
    
    target_var = set()
    for item in target_terms:
        for i in item:
            target_var.add(i)
            
    if len(target_var) != 1:
        return False, None
    
    target_var = list(target_var)[0]
    
    # only a single var
    assert (target_var,) in target_terms and (target_var,target_var) in target_terms
    
    if sdp.terms[(target_var,)] == p-1 and sdp.terms[(target_var,target_var)] == 1:
        if target_var in binary_vars:
            # NOTE:  we use this to check for skipping of encoding of binary constraints, under ablation testing this might not work as expected since the binary vars might not have already been encoded in binary_vars. As a result, only skip if it is already encoded.
            return True, target_var
    
    
    if sdp.terms[(target_var,)] == 1 and sdp.terms[(target_var,target_var)] == p-1:
        if target_var in binary_vars:
            return True, target_var
    
    # coef did not match
    return False, None
    
    

def ensure_domain(poly):
    # to address the serilization issue with multiprocessing
    return Poly(poly, domain=GF(p))
    

def grobner_basis_helper(polynomials, grobner_method):
    polynomials = [ensure_domain(poly) for poly in polynomials]
    # 1: grlex order. 2: decreasing order. 
    try:
        if grobner_method == 1:
            G = groebner(polynomials, order='grlex', domain=GF(p), method='f5b')
        else:
            assert grobner_method == 2
            var_order = [sympy_symbols[f"x{k}"] for k in sorted([int(k[1:]) for k in sympy_symbols.keys()], key = lambda x:-x)]
            G = groebner(polynomials, var_order, order='lex', domain=GF(p), method='f5b')

        return G
    except Exception as e:
        return None
        

def handler(signum, frame):
    raise TimeoutError()
    

def get_grobner_basis(r1cs_circuit, with_hypothesis = None, timeout=5, grobner_method = None):
    assert grobner_method in [1,2]
    
    # Convert constraints into sympy equations
    polynomials = []
    zero_cons = {():0}
    for constraint in r1cs_circuit.all_constraints:
        # ignore zero constraints.
        if constraint.terms == zero_cons:
            continue
            
        # ignore binary constraints, and instead use logical encoding later on.
        cur_is_binary, _ = sdp_is_binary_indicator(constraint)
        if cur_is_binary:
            continue
            
        polynomials.append(get_sympy_expression_of_sdp(constraint))
        
    if with_hypothesis is not None:
        polynomials.append(get_sympy_expression_of_sdp(SingleDotProduct(with_hypothesis.condition)))

    var_order = None
    grobner_found = False

    # Thread to run the groebner computation
    signal.signal(signal.SIGALRM, handler)
    signal.alarm(timeout)  # Set the timeout
    try:
        G = grobner_basis_helper(polynomials, grobner_method)
        signal.alarm(0)  # Disable the alarm
    except TimeoutError:
        G = None
    
    if G is None:
        return None, None
    grobner_found = True
 
    if grobner_found:
        G = [ensure_domain(p) for p in G]
        l_expr = [str(poly.as_expr()) for poly in G]
        return G, l_expr
    
    else:
        return None, None
        




############################## Z3 ##############################


    
# Dictionary to store Z3 variables
z3_variables = {}


def get_variable(name):
    global z3_variables
    if name not in z3_variables:
        z3_variables[name] = Int(name)  # Using Int as we are dealing with integers
    return z3_variables[name]




def parse_one_term(one_term):
    one_term = one_term.strip()
    
    # Case 1: Variable with exponent (e.g., x2**2)
    if '^^' in one_term:
        base, exp = one_term.split('^^')
        exp = int(exp)
        base = base.strip()
        assert base.startswith("x")
        base = get_variable(base)
        exp_result = None
        while exp > 0:
            if exp_result == None:
                exp_result = base
            else:
                exp_result *= base
            exp -= 1
        return exp_result

    
    # Case 2: Single variable (e.g., x2)
    elif one_term.startswith('x'):
        assert "*" not in one_term
        return get_variable(one_term)
    
    # Case 3: Constant (e.g., 3)
    else:
        assert 'x' not in one_term
        return (int(one_term)+ p) % p
            
            

def parse_term(term):
    # Function to parse a single term like '6466240853085493210806464488708044363640544067432981142930249539095013620696*x2**2*x4'

    term = term.strip()
    term = term.replace('**', '^^')
    
    product = None
    if term.startswith("-"):
        term = term[1:]
        product = p-1
    parts = term.split('*')
    parts = [i.strip() for i in parts]
    for one_term in parts:
        parsed = parse_one_term(one_term)
        if isinstance(parsed, int):
            if product is None:
                product = parsed
            else:
                assert isinstance(product, int)
                product = mul_constants(product, parsed)
        else:    
            if product is None:
                product = parsed
            else:
                product *= parsed
    return product




def sympy_to_z3(expr):
    # Function to parse and construct Z3 expressions from terms
    # NOTE: Zero constraints already ignored while we were building the grobner basis
    expr = expr.strip()
    assert expr != ''
    
    z3_expr = None
    expr = expr.replace('-', '+-')  # Handle subtraction as negative addition
    parts = expr.split('+')

    for part in parts:
        if part.strip():  # Ensure no empty strings are processed
            if z3_expr is None:
                z3_expr = parse_term(part.strip())
            else:
                z3_expr += parse_term(part.strip())
    
    assert z3_expr is not None
    return (z3_expr + p) % p



def terms_d_to_z3_expr(terms_d):
    z3_expr = 0
    for terms, coeff in terms_d.items():
        coeff = (coeff + p) % p
        if coeff == 0:
            continue
        if len(terms) == 0:  # Constant term
            z3_expr += coeff
        elif len(terms) == 1:  # Linear term
            z3_expr += coeff * get_variable(f'x{terms[0]}')
        elif len(terms) == 2:  # Quadratic term
            z3_expr += coeff * get_variable(f'x{terms[0]}') * get_variable(f'x{terms[1]}')
        else:
            raise Exception("Higher order terms should not occur.")
    return (z3_expr + p) % p


def get_all_terms_d_of_r1cs(r1cs_circuit):
    return [i.terms for i in r1cs_circuit.all_constraints]


def list_of_terms_d_to_z3(terms_d_l):
    # ignoring 0 polys
    l_z3_expr = []
    for one_set_term in terms_d_l:
        if one_set_term == {(): 0}:
            continue
        z3_expr = terms_d_to_z3_expr(one_set_term)
        l_z3_expr.append(z3_expr)
    return l_z3_expr


def add_0_constraints_to_solver(solver, z3_expr_l):
    for z3_expr in z3_expr_l:
        solver.add(z3_expr == 0)
    

def add_all_finite_field_range_restrictions(solver):
    global z3_variables
    global p
    
    for var in z3_variables.values():
        solver.add(0 <= var, var < p)
        
        

def add_binary_restrictions(solver):
    global binary_vars
    for var in binary_vars:
        var = get_variable(f"x{var}")
        solver.add(Or(var == 0, var == 1))
        

def check_results(solver, timeout_solver = 5000):
    global z3_variables
    solver.set("timeout", timeout_solver)
    
    
#     solver.set("simplify.algebraic_number_evaluator", True)
    


    result = solver.check()
    if result == sat:
        m = solver.model()
        model_dict = {d.name(): get_constant_gf_p(m[d].as_long()) for d in m.decls() if not isinstance(m[d], FuncInterp)}
        return "SAT", model_dict
    elif result == unsat:
        return "UNSAT", None
    else:
        return None, None
  
    
    

def push_state(solver):
    solver.push()

  
def pop_state(solver):
    solver.pop()
    
    

def get_active_variables(poly):
    # Returns a list of variables that actually appear in a given sympy Poly object.
    assert isinstance(poly, Poly)
    active_vars = set()
    poly_dict = poly.as_dict()

    for powers in poly_dict.keys():
        for power, gen in zip(powers, poly.gens):
            if power != 0:
                active_vars.add(gen)
    
    return list(active_vars)





def helper_polys_have_non_squarable_conflicts(G, inferred_assignments):
    # to handle only polynomials with exactly two terms—a single squared term and a constant
    # returns True if conflict is identified
    # records inferred assignments in inferred_assignments

    for poly in G:
        active_vars = get_active_variables(poly)
        poly = Poly(poly, active_vars, domain = GF(p))
        if poly.as_expr().is_constant():
            continue
        # Poly("9 + x1*x2 + x3**3*x4").monoms() -> [(1, 1, 0, 0), (0, 0, 3, 1), (0, 0, 0, 0)]
        # Poly("9 + x1*x2 + x3**3*x4").coeffs() -> [1, 1, 9]

        monomials = poly.monoms()
        coefficients = poly.coeffs()

        # polynomials with exactly two terms
        if len(monomials) == 2:
            squared_term_index = None
            constant_term_index = None
            squared_term_coef = None
            constant_term_coef = None

            for i, monomial in enumerate(monomials):
                # constant term
                if all(m == 0 for m in monomial):
                    constant_term_index = i
                    constant_term_coef = int(coefficients[i])
                # squared term
                elif all(m % 2 == 0 for m in monomial) and sum(monomial) > 0:
                    squared_term_index = i
                    squared_term_coef = int(coefficients[i])
        
            if squared_term_index is not None and constant_term_index is not None and squared_term_coef != 0:
                
                c = negate_constant(get_constant_gf_p(constant_term_coef))
                c = mul_constants(c, invert_constant(squared_term_coef))
                
                if not has_sqrt_mod_p(c):
                    return True
                if len(active_vars) == 1:
                    var = str(active_vars[0])
                    assert var.startswith('x')
                    var_num = int(var[1:])
                    
                    solved_val_for_var = sqrt_mod_p(c)
                    if var not in inferred_assignments:
                        inferred_assignments[var] = []
                    inferred_assignments[var].append(tuple(solved_val_for_var))


    return False



def clause_learning(G, solver, preconditions_l, start_time, timeout_seconds):
    # learns clauses under the condition of preconditions_l. encodes any new info into the SMT solver.
    # preconditions_l: A list of preconditions up to now, including with_assigned
    # returns if conflict found (i.e., UNSAT) or no conflict SAT (with SAT, learned solutions are added as clauses into solver)
    if NO_CLAUSE_LEARNING:
        return False
    
    if start_time is None:
        start_time = time.time()
    
    inferred_assignments = dict() # var name to a list of tuples, each tuple indicate an OR relationship. 
    # We refer conflict finding or merging among different tuples to the SMT solver. Over here, we just record and encode information into SMT problem.

    if time.time() - start_time > timeout_seconds:
        return False
        
    G = [Poly(poly, domain=GF(p)) for poly in G]
    
    
    if clause_learning_degree_0(G, inferred_assignments):
        solver.add(Not(And(preconditions_l)))
        add_contribution_count("clause_learning", 1)
        return True
    
    if clause_learning_degree_1(G, inferred_assignments):
        solver.add(Not(And(preconditions_l)))
        add_contribution_count("clause_learning", 1)
        return True
    
    if clause_learning_degree_2(G, inferred_assignments):
        solver.add(Not(And(preconditions_l)))
        add_contribution_count("clause_learning", 1)
        return True
    
    if len(inferred_assignments) == 0:
        # no further round needed, just return "no conflict"
        return False
    
    
    # no early conflict is found yet, new assignments were inferred, we conduct another recursive round of value inferring
    for one_var in inferred_assignments: # all from this round
        assert one_var.startswith('x')
        for one_pair in inferred_assignments[one_var]:
            one_pair = tuple(set(one_pair))
            assert len(one_pair) in [1,2]
            valid_sols = []
            

            for cur_sol in one_pair:
                # recursive check on the feasibility of the current solution. preconditions_l already encodes this layer
                if time.time() - start_time > timeout_seconds:
                    return False
                
                sub_preconditions_l = preconditions_l + [get_one_equal_condition(one_var, cur_sol)]
                with_assigned = {one_var:cur_sol}
                sub_G = [substitute_vars(poly, with_assigned) for poly in G]
                
                conflict_found_cur_var = clause_learning(sub_G, solver, sub_preconditions_l, start_time, timeout_seconds)
                if not conflict_found_cur_var:
                    valid_sols.append(cur_sol)
                # we assume conflicts are already handled in the recursive call
            if len(valid_sols) == 0:
                continue
            elif len(valid_sols) == 1:
                if len(preconditions_l) > 0:
                    solver.add(Implies(And(preconditions_l), get_one_equal_condition(one_var, valid_sols[0])))
                    add_contribution_count("clause_learning", 1)
                else:
                    solver.add(get_one_equal_condition(one_var, valid_sols[0]))
                    add_contribution_count("clause_learning", 1)
                continue
            else:
                assert len(valid_sols) == 2
                or_conditions = []
                or_conditions.append(get_one_equal_condition(one_var, valid_sols[0]))
                or_conditions.append(get_one_equal_condition(one_var, valid_sols[1]))
                
                if len(preconditions_l) > 0:
                    solver.add(Implies(And(preconditions_l), Or(or_conditions)))
                    add_contribution_count("clause_learning", 1)
                else:
                    solver.add(Or(or_conditions))
                    add_contribution_count("clause_learning", 1)
                    
    return False



def clause_learning_degree_1(G, inferred_assignments):    
    for poly in G:
        active_vars = get_active_variables(poly)
        if len(active_vars) != 1:
            continue
        poly = Poly(poly, active_vars, domain=GF(p))
        
        if poly.as_expr().is_constant():
            continue
        
        monomials = poly.monoms()
        coefficients = poly.coeffs()

        # one variable term and one constant term
        if len(monomials) == 2:
            variable_term_index = None
            constant_term_index = None
            variable_term_coef = None
            constant_term_coef = None

            for i, monomial in enumerate(monomials):
                # constant term
                if all(m == 0 for m in monomial):
                    constant_term_index = i
                    constant_term_coef = coefficients[i]
                # single variable
                elif sum(monomial) == 1 and all(m <= 1 for m in monomial):
                    variable_term_index = i
                    variable_term_coef = int(coefficients[i])

            if variable_term_index is not None and constant_term_index is not None and variable_term_coef != 0:
                # c1 * x + c2 = 0
                c = negate_constant(get_constant_gf_p(constant_term_coef))
                c = mul_constants(c, invert_constant(variable_term_coef))


                var = str(active_vars[0]) # guaranteed to have only one active var
                assert var.startswith('x')
                var_num = int(var[1:])

                if var not in inferred_assignments:
                    inferred_assignments[var] = []
                inferred_assignments[var].append((c,))

    return False


def clause_learning_degree_0(G, inferred_assignments):
    for poly in G:
        poly_expr = poly.as_expr()
        if poly_expr.is_constant() and poly_expr != 0:
            return True
    return False





def clause_learning_degree_2(G, inferred_assignments):
    # ax**2 + bx + c = 0: either solvable or UNSAT.
    if helper_polys_have_non_squarable_conflicts(G, inferred_assignments):
        return True
    
    
    x = symbols('x') 
    
    for poly in G:
        active_vars = get_active_variables(poly)
        poly = Poly(poly, active_vars, domain = GF(p))
        if len(active_vars) == 1 and poly.degree() == 2:  
            var = str(active_vars[0]) 
            assert var.startswith('x')
            var_num = int(var[1:])
            
            # ax**2 + bx + c
            coeffs = poly.all_coeffs()
            if len(coeffs) == 3:
                a, b, c = coeffs
                a, b, c = get_constant_gf_p(int(a)), get_constant_gf_p(int(b)), get_constant_gf_p(int(c))
                if a == 0 or b == 0:
                    # former: not quadratic; latter: not handled here
                    continue
                solvable, solution = solve_quadratic_equation_in_Fp(a, b, c)
                if not solvable:
                    return True
                else:
                    if var not in inferred_assignments:
                        inferred_assignments[var] = []
                    inferred_assignments[var].append(tuple(solution))

    return False





def get_one_equal_condition(one_var, cur_sol):
    assert one_var.startswith('x')
    return (get_variable(one_var) + p) % p == int(cur_sol)








def substitute_vars(poly, substitutions):
    # substitutions: A dictionary where keys are variable names (as strings) and values are the numbers to substitute.
    # returns a new sympy Poly object with the substitutions done.
    substituted_poly = poly.subs(substitutions)
    return Poly(substituted_poly, poly.gens, domain=GF(p))


    

    
    
    
# NOTE: below is the sdp version

def helper_polys_have_non_squarable_conflicts_sdp(r1cs_circuit, inferred_assignments):

    for cur_sdp in r1cs_circuit.all_constraints:
        # only take squares of the form c1 * x1 * x1 + c2 = 0
        if len(cur_sdp.terms) != 2:
            continue
        assert () in cur_sdp.terms
        other_term = [i for i in cur_sdp.terms if i != ()]
        assert len(other_term) == 1
        other_term = other_term[0]
        
        if len(other_term) != 2 or other_term[0] != other_term[1]:
            continue
        c1 = cur_sdp.terms[other_term]
        c2 = cur_sdp.terms[()]
        other_var = other_term[0]
        
        c = negate_constant(get_constant_gf_p(c2))
        c = mul_constants(c, invert_constant(c1))
    
        
        if not has_sqrt_mod_p(c):
            return True
        solved_val_for_var = sqrt_mod_p(c)
        if f"x{other_var}" not in inferred_assignments:
            inferred_assignments[f"x{other_var}"] = []
        inferred_assignments[f"x{other_var}"].append(tuple(solved_val_for_var))


    return False



def clause_learning_degree_1_sdp(r1cs_circuit, inferred_assignments):

    for cur_sdp in r1cs_circuit.all_constraints:
        # only take linears of the form c1 * x1 + c2 = 0
        if len(cur_sdp.terms) != 2:
            continue
        assert () in cur_sdp.terms
        other_term = [i for i in cur_sdp.terms if i != ()]
        assert len(other_term) == 1
        other_term = other_term[0]
        
        if len(other_term) != 1:
            continue
        c1 = cur_sdp.terms[other_term]
        c2 = cur_sdp.terms[()]
        other_var = other_term[0]
        
        c = negate_constant(get_constant_gf_p(c2))
        c = mul_constants(c, invert_constant(c1))
    
        
        if f"x{other_var}" not in inferred_assignments:
            inferred_assignments[f"x{other_var}"] = []
        inferred_assignments[f"x{other_var}"].append((c,))


    return False


def clause_learning_degree_0_sdp(r1cs_circuit, inferred_assignments):
    
    for cur_sdp in r1cs_circuit.all_constraints:
        assert () in cur_sdp.terms
        if len(cur_sdp.terms) == 1 and cur_sdp.terms[()] != 0:
            return True
    return False






def clause_learning_degree_2_sdp(r1cs_circuit, inferred_assignments):
    
    if helper_polys_have_non_squarable_conflicts_sdp(r1cs_circuit, inferred_assignments):
        return True
    
    for cur_sdp in r1cs_circuit.all_constraints:
        # a * x1 * x1 + b * x1 + c = 0
        assert () in cur_sdp.terms
        
        if len(cur_sdp.terms) != 3:
            continue
        
        double_terms = [i for i in cur_sdp.terms if len(i) == 2]
        single_terms = [i for i in cur_sdp.terms if len(i) == 1]
        
        if len(double_terms) != 1 or len(single_terms) != 1:
            continue
            
        single_term = single_terms[0]
        double_term = double_terms[0]
        
        if double_term[0] != double_term[1] or double_term[0] != single_term[0]:
            continue
        
        a = cur_sdp.terms[double_term]
        b = cur_sdp.terms[single_term]
        c = cur_sdp.terms[()]
        
        other_var = single_term[0]
        assert double_term == (other_var, other_var)
        
        if a == 0 or b == 0:
            continue
        # Solve the quadratic equation
        solvable, solution = solve_quadratic_equation_in_Fp(a, b, c)
        if not solvable:
            # if not solvable, conflict found
            return True
        else:
            if f"x{other_var}" not in inferred_assignments:
                inferred_assignments[f"x{other_var}"] = []
            inferred_assignments[f"x{other_var}"].append(tuple(solution))

    return False



def clause_learning_sdp(r1cs_circuit, solver, preconditions_l, start_time, timeout_seconds):
    if NO_CLAUSE_LEARNING:
        return False
    
    inferred_assignments = dict() # var name to a list of tuples. Each tuple indicate an OR relationship. We refer conflict finding or merging among different tuples to the SMT solver. Over here, we just record and encode information into SMT problem.
    
    if time.time() - start_time > timeout_seconds:
        return False
    
    if clause_learning_degree_0_sdp(r1cs_circuit, inferred_assignments):
        solver.add(Not(And(preconditions_l)))
        add_contribution_count("clause_learning", 1)
        return True
    
    if clause_learning_degree_1_sdp(r1cs_circuit, inferred_assignments):
        solver.add(Not(And(preconditions_l)))
        add_contribution_count("clause_learning", 1)
        return True
    
    if clause_learning_degree_2_sdp(r1cs_circuit, inferred_assignments):
        solver.add(Not(And(preconditions_l)))
        add_contribution_count("clause_learning", 1)
        return True
    
    if len(inferred_assignments) == 0:
        # No further round needed, just return no conflict
        return False
    

    # If we have not returned yet, that means no early conflict is found yet, we conduct another round of value inferring
    for one_var in inferred_assignments:
        assert one_var.startswith('x')
        for one_pair in inferred_assignments[one_var]:
            one_pair = tuple(set(one_pair))
            assert len(one_pair) in [1,2]
            valid_sols = []
            
            for cur_sol in one_pair:
                if time.time() - start_time > timeout_seconds:
                    return False
    
                cur_with_assigned_val = SingleDotProduct({():cur_sol})
                with_assigned = {one_var:cur_with_assigned_val}

                sub_r1cs_circuit = get_r1cs_circuit_copy(r1cs_circuit)
                for cur_sdp in sub_r1cs_circuit.all_constraints:
                    cur_sdp.plug_in_expression_of_var_IN_PLACE(one_var, cur_with_assigned_val)
                sub_preconditions_l = preconditions_l + [get_one_equal_condition(one_var, cur_sol)]
                
                
                conflict_found_cur_var = clause_learning_sdp(sub_r1cs_circuit, solver, sub_preconditions_l, start_time, timeout_seconds)
                
                if not conflict_found_cur_var:
                    valid_sols.append(cur_sol)
                # we assume conflicts are already handled in the recursive call
            if len(valid_sols) == 0:
                continue
            elif len(valid_sols) == 1:
                if len(preconditions_l) > 0:
                    solver.add(Implies(And(preconditions_l), get_one_equal_condition(one_var, valid_sols[0])))
                    add_contribution_count("clause_learning", 1)
                else:
                    solver.add(get_one_equal_condition(one_var, valid_sols[0]))
                    add_contribution_count("clause_learning", 1)
                continue
            else:
                assert len(valid_sols) == 2
                or_conditions = []
                or_conditions.append(get_one_equal_condition(one_var, valid_sols[0]))
                or_conditions.append(get_one_equal_condition(one_var, valid_sols[1]))
                
                if len(preconditions_l) > 0:
                    solver.add(Implies(And(preconditions_l), Or(or_conditions)))
                    add_contribution_count("clause_learning", 1)
                else:
                    solver.add(Or(or_conditions))
                    add_contribution_count("clause_learning", 1)
                    
                    
    return False


    

def get_r1cs_circuit_copy(r1cs_circuit):
    new_r1cs_circuit = R1CS(r1cs_circuit.magic_number, r1cs_circuit.version, r1cs_circuit.num_sections, 
         r1cs_circuit.header_vars, r1cs_circuit.constraint_section, r1cs_circuit.wireid2label_section, 
         r1cs_circuit.input_list, r1cs_circuit.output_list)
    
    new_r1cs_circuit.all_constraints = []
    for cur_sdp in r1cs_circuit.all_constraints:
        new_sdp = SingleDotProduct(initial_terms = cur_sdp.terms) # a copy is taken by default
        new_sdp.original_terms_copy = cur_sdp.original_terms_copy.copy()
        new_r1cs_circuit.all_constraints.append(new_sdp)
        
    return new_r1cs_circuit
        
    
    

def clear_and_rerun_cons_id(cons_id, r1cs_circuit):
    global edge_id_to_edge
    global var_to_cons_id
    global cons_id_to_edges
    global DO_SIMPLIFICATION_PHASE
    global DO_BPG_PHASE 
    
    cur_sdp = r1cs_circuit.all_constraints[cons_id]
            
    for edge_id in cons_id_to_edges[cons_id]:
        assert edge_id in edge_id_to_edge, (cons_id, edge_id)
        edge_to_pop = edge_id_to_edge[edge_id]
        edge_id_to_edge.pop(edge_id)
        # From this point, old edge ids won't appear in the look up. i.e., they can't be found in the lookup dict.

    cons_id_to_edges[cons_id] = []
    
    constraint_info = get_constraint_info(r1cs_circuit, cons_id, cur_sdp)
    if DO_SIMPLIFICATION_PHASE:
        run_simplification(cur_sdp, constraint_info) # assumptions are also extracted here
    if DO_BPG_PHASE:
        construct_BPG_edges(cur_sdp, constraint_info)



        

def address_backwards_dependency(affected_vars, new_replacement, r1cs_circuit, cur_cons_id):
    # addresses the dependency of newly identified information
    
    # one of affected_vars, new_replacement must be None
    global edge_id_to_edge
    global var_to_cons_id
    global linear_subs_back_solving
    global linear_subs_back_solving_constants

    # addresses all constraints that are before this, and replaces all constraints regardless.
    if new_replacement is None:
        assert affected_vars is not None
        new_affected_vars = set()
        for one_var in affected_vars:
            new_affected_vars.update(equivalent_cons_vars.get_group_members(one_var))
        affected_vars = new_affected_vars
        set_affected_cons_id = set()
        for var in affected_vars:
            # we might be updating old (substituted) vars, so need to account for new vars as well
            for eq_var in equivalent_cons_vars.get_group_members(var):
                if eq_var in var_to_cons_id:
                    set_affected_cons_id.update(var_to_cons_id[eq_var])
                    
        for cons_id in set_affected_cons_id:
            if cons_id >= get_cur_processing_cons_id():
                # no need to rerun the current/future constraints.
                continue
            clear_and_rerun_cons_id(cons_id, r1cs_circuit)
        
    else:
        assert affected_vars is None
        # we only replace one replacement and one var at a time
        assert len(new_replacement) == 1
        
        replaced_term = list(new_replacement.keys())[0]
        assert len(replaced_term) == 1
        replaced_var = replaced_term[0]
        
        replacing_expression = new_replacement[replaced_term]
        exp_constant_val = replacing_expression.terms[()]
        exp_terms = [i for i in replacing_expression.terms if i != ()]
        if len(exp_terms) == 1:
            exp_term = exp_terms[0]
            assert len(exp_term) == 1 # only sub single vars with single vars
            exp_variable = exp_term[0]
            exp_variable_coef = replacing_expression.terms[exp_term]
            
            # record the reversed substitution
            # current expression: replaced_var <- exp_constant_val + exp_variable_coef * exp_variable
            # reversed expression: exp_variable <- (replaced_var - exp_constant_val) * exp_variable_coef**(-1)
            inv_coef = invert_constant(exp_variable_coef)
            this_rev_const = mul_constants(inv_coef, negate_constant(exp_constant_val))
            
            if replaced_var not in linear_subs_back_solving:
                linear_subs_back_solving[replaced_var] = []
            linear_subs_back_solving[replaced_var].append((exp_variable, {(): exp_constant_val, (exp_variable,):exp_variable_coef}))
            
        else:
            assert len(exp_terms) == 0 # a single var is replaced by a constant. no sub expression
            exp_variable, exp_variable_coef = None, None
            
            assert replaced_var not in linear_subs_back_solving_constants
            linear_subs_back_solving_constants[replaced_var] = exp_constant_val
        
         
            
        all_affected_cons_id = set()
        for cons_id in list(var_to_cons_id[replaced_var]):
            cur_sdp = r1cs_circuit.all_constraints[cons_id]

            all_affected_vars = set()
            for one_term in cur_sdp.terms:
                all_affected_vars.update(one_term)
            assert all([cons_id in var_to_cons_id[onevar1] for onevar1 in all_affected_vars])
            assert replaced_var in all_affected_vars, replaced_var
            cur_sdp.plug_in_expression_of_var_IN_PLACE(replaced_var, replacing_expression)

            post_vars = set()
            for one_term in cur_sdp.terms:
                post_vars.update(one_term)

            for one_affected_var in all_affected_vars:
                if one_affected_var not in post_vars:
                    # the var has disappeared
                    var_to_cons_id[one_affected_var].remove(cons_id)

            for one_affected_var in post_vars:
                if one_affected_var not in all_affected_vars:
                    var_to_cons_id[one_affected_var].append(cons_id)

            for one_term in cur_sdp.terms:
                is_constrained(one_term)
                if len(one_term) == 2 and is_constrained((one_term[0],)) and is_constrained((one_term[1],)):
                    if not is_constrained(one_term):
                        set_constrained(one_term)

            # rerunning of affected edge must handled, but no need to worry about rerunning current edge
            if cons_id == cur_cons_id:
                continue
            all_affected_cons_id.add(cons_id)


        for cons_id in all_affected_cons_id:
            if cons_id >= get_cur_processing_cons_id():
                # no need to rerun the current constraint.
                continue
            clear_and_rerun_cons_id(cons_id, r1cs_circuit)

        assert len(var_to_cons_id[replaced_var]) == 0 # should all be gone
    return

          
    
    
############################## Data structure utils ##############################


def add_edge_to_BGP_graph(key, value, cur_cons_id, edge_condition = []):
    # Note: this automatically handles the unconditional transitions
    all_conditions = edge_condition + [key]
    add_contribution_count("bpg_edges", (tuple(all_conditions), value))
    
    for one_condition in all_conditions:
        if len(one_condition) != 0:
            add_one_edge_to_BGP_graph(one_condition, value, cur_cons_id, edge_condition = [i for i in all_conditions if i != one_condition and len(i) != 0])
        


def add_one_edge_to_BGP_graph(key, value, cur_cons_id, edge_condition = []):
    # Note: by default we assume no edge condition if not supplied.
    global bpg_graph
    global edge_id_to_edge
    global cons_id_to_edges
    
    if len(edge_condition) > 0:
        assert len(key) != 0
    
    cur_new_edge = BPG_Edge(key, value, cur_cons_id, edge_condition)
    cur_id = cur_new_edge.edge_id
    assert cur_id not in edge_id_to_edge
    edge_id_to_edge[cur_id] = cur_new_edge
    
    if cur_cons_id not in cons_id_to_edges:
        cons_id_to_edges[cur_cons_id] = []
    
    cons_id_to_edges[cur_cons_id].append(cur_id)
    
    add_contribution_count("bpg_nodes", key)
    add_contribution_count("bpg_nodes", value)

    
    if len(key) == 0:
        # unconditional edge
        bpg_graph.invariant_properties.add(cur_id)
        return 

    first_level_key = key[0]
    if first_level_key not in bpg_graph.graph:
        bpg_graph.graph[first_level_key] = dict()

    if key not in bpg_graph.graph[first_level_key]:
        bpg_graph.graph[first_level_key][key] = dict()

    if value[0] not in bpg_graph.graph[first_level_key][key]:
        bpg_graph.graph[first_level_key][key][value[0]] = []

    bpg_graph.graph[first_level_key][key][value[0]].append(cur_id)

    
    
    

def run_simplification(cur_sdp, constraint_info):
    # NOTE: we also address the addition of CDG edges, plus candidate hypotheses, when no preliminary simplifications are possible.
    global DO_SIMPLIFICATION_PHASE
    global DO_ASSUMPTION_PHASE
    
    constraint_id = constraint_info['constraint_id']
    r1cs_circuit = constraint_info['r1cs_circuit']
    
    if DO_SIMPLIFICATION_PHASE:
        # Repeat until no progress at all.
        overall_progress_made = True
        while overall_progress_made:
            overall_progress_made = False
            for consideration in ["current", "original"]:
                if consideration == "original":
                    if cur_sdp.original_terms_copy == cur_sdp.terms:
                        continue
                    else:
                        get_original_info = True
                else:
                    get_original_info = False

                constraint_info = get_constraint_info(r1cs_circuit, constraint_id, cur_sdp, get_original_info)
                progress_made = simple_solve(cur_sdp, constraint_info, get_original_info)
                if progress_made:
                    constraint_info = get_constraint_info(r1cs_circuit, constraint_id, cur_sdp, get_original_info)
                overall_progress_made = overall_progress_made or progress_made

                progress_made = multi_con_to_single_uncon(cur_sdp, constraint_info, get_original_info)
                if progress_made:
                    constraint_info = get_constraint_info(r1cs_circuit, constraint_id, cur_sdp, get_original_info)
                overall_progress_made = overall_progress_made or progress_made

                progress_made = check_binary_expression(cur_sdp, constraint_info, get_original_info)
                if progress_made:
                    constraint_info = get_constraint_info(r1cs_circuit, constraint_id, cur_sdp, get_original_info)
                overall_progress_made = overall_progress_made or progress_made

                
    if DO_ASSUMPTION_PHASE:
        constraint_info = get_constraint_info(r1cs_circuit, constraint_id, cur_sdp, get_original_info = False)

        common_vars = constraint_info['all_vars']
        all_not_sure_var_appearance = dict()
        for one_term in constraint_info['NOT_SURE_terms']:
            # record the common terms for identifying hypothesis targets
            common_vars = common_vars.intersection(set(one_term))
            for one_var in one_term:
                if one_var not in all_not_sure_var_appearance:
                    all_not_sure_var_appearance[one_var] = 1
                else:
                    all_not_sure_var_appearance[one_var] += 1

        all_not_sure_vars = list(all_not_sure_var_appearance.keys())   
    
        # track assumption vars
        if len(constraint_info['NOT_SURE_terms']) > 1 and len(common_vars) == 1:
            # If there is one common term, take it.
            common_var = list(common_vars)[0]
            if (common_var, common_var) not in constraint_info['NOT_SURE_terms']:
                # NOTE: we need all other vars to be constrained
                if all([is_constrained((v,)) for v in all_not_sure_vars if v != common_var]):
                    target = common_var
                    condition = {tuple([one_v for one_v in one_term if one_v != common_var]):cur_sdp.terms[one_term] for one_term in constraint_info['NOT_SURE_terms']}
                    assert len(condition) > 1 # since we took from each term
                    add_edge_to_assumptions(target, condition, constraint_info['constraint_id'])


        if len(constraint_info['NOT_SURE_terms']) == 1 and len(all_not_sure_vars) == 2:
            # one var not sure, one var cons. the cons var will be the condition
            assert all_not_sure_vars[0] in constraint_info['NOT_SURE_terms'][0] 
            assert all_not_sure_vars[1] in constraint_info['NOT_SURE_terms'][0] 
            this_cons_var = [v for v in all_not_sure_vars if is_constrained((v,))]
            this_uncon_var = [v for v in all_not_sure_vars if not is_constrained((v,))]
            
            if len(this_cons_var) == 1 and len(this_uncon_var) == 1:
                condition_var = this_cons_var[0]
                common_var = this_uncon_var[0]
                
                # NOTE: we need all other vars to be constrained
                assert is_constrained((condition_var,))

                target = common_var

                condition = {(condition_var,):cur_sdp.terms[constraint_info['NOT_SURE_terms'][0]]}
                add_edge_to_assumptions(target, condition, constraint_info['constraint_id'])


        if len(constraint_info['NOT_SURE_terms']) == 0:
            # try constrained doubles that can be separated
            for one_term in constraint_info['C_terms']:

                if len(one_term) != 2:
                    continue

                cons_var = None
                uncon_var = None
                if not is_constrained((one_term[0],)) and is_constrained((one_term[1],)):
                    cons_var = one_term[1]
                    uncon_var = one_term[0]
                if not is_constrained((one_term[1],)) and is_constrained((one_term[0],)):
                    cons_var = one_term[0]
                    uncon_var = one_term[1]
                if cons_var is None or uncon_var is None:
                    continue

                target = uncon_var
                condition = {(cons_var,):cur_sdp.terms[one_term]}
                add_edge_to_assumptions(target, condition, constraint_info['constraint_id'])
        
    return 



            
            

def get_constraint_info(r1cs_circuit, constraint_id, cur_sdp, get_original_info = False):
    # to save repeated computation across the construction of different graphs
    constraint_info = dict()
    constraint_info['single_terms'] = []
    constraint_info['double_terms'] = []
    constraint_info['C_terms'] = []
    constraint_info['UNDER_terms'] = []
    constraint_info['NOT_SURE_terms'] = []
    constraint_info['constant'] = 0
    constraint_info['all_vars'] = set()
    constraint_info['constraint_id'] = constraint_id
    constraint_info['input_vars'] = r1cs_circuit.input_list
    constraint_info['output_vars'] = r1cs_circuit.output_list
    constraint_info['r1cs_circuit'] = r1cs_circuit

    if get_original_info:
        target_terms = cur_sdp.original_terms_copy
    else:
        target_terms = cur_sdp.terms
        
    for one_term in target_terms:
        if len(one_term) == 1:
            constraint_info['single_terms'].append(one_term)
        elif len(one_term) == 2:
            constraint_info['double_terms'].append(one_term)
        else:
            constraint_info['constant'] = target_terms[one_term]
        constraint_info['all_vars'].update(one_term)

        if is_constrained(one_term):
            constraint_info['C_terms'].append(one_term)
        elif is_undetermined(one_term):
            constraint_info['NOT_SURE_terms'].append(one_term)
        elif is_undeterconstrained(one_term):
            constraint_info['UNDER_terms'].append(one_term)

    return constraint_info







def add_edge_to_assumptions(target, condition, cur_cons_id):
    global assumptions
    global edge_id_to_edge
    global cons_id_to_edges
    
    add_contribution_count("add_edge_to_assumptions", (target, tuple(condition.items())))
    if cur_cons_id not in cons_id_to_edges:
        cons_id_to_edges[cur_cons_id] = []
        
    if target not in assumptions.assumptions:
        assumptions.assumptions[target] = []
            
    cur_new_edge = Hypothesis_Edge(target, condition, cur_cons_id)
            
    cur_id = cur_new_edge.edge_id
    assert cur_id not in edge_id_to_edge
    edge_id_to_edge[cur_id] = cur_new_edge
    # NOTE: edge_id_to_edge serves as an intermediate station to look up all edges. Removing edge id from it suffices for removing edges

    cons_id_to_edges[cur_cons_id].append(cur_id)

    assumptions.assumptions[target].append(cur_id)
    return cur_id
    
    
    




    
    
    
    
    
############################## Some IRs ##############################

def multi_con_to_single_uncon(cur_sdp, constraint_info, get_original_info):
    # This is IR3: Single Unknown
    
    # replacements is not handled here
    
    progress_made = False

    if len(constraint_info['UNDER_terms']) > 0:
        # only to cons
        return progress_made
    if len(constraint_info['NOT_SURE_terms']) != 1:
        # we only handle solving one unknown to known
        return progress_made
    
    target_term = constraint_info['NOT_SURE_terms'][0]
    if not is_constrained(target_term):
        set_constrained(target_term)

        add_contribution_count("multi_con_to_single_uncon", target_term)
        address_backwards_dependency(set(target_term), None, constraint_info['r1cs_circuit'], constraint_info['constraint_id'])
        progress_made = True
    
    return progress_made
    

    

def simple_solve(cur_sdp, constraint_info, get_original_info):
    # This is IR2: Simple Solve

    global equivalent_vars
    global equivalent_cons_vars
    global binary_vars
    
    
    if get_original_info:
        # solving does not need to be repeated 
        return False

    progress_made = False
    
    if get_original_info:
        target_terms = cur_sdp.original_terms_copy
    else:
        target_terms = cur_sdp.terms
        
        
    if len(constraint_info['single_terms']) == 1 and len(constraint_info['double_terms']) == 0:
        # a single var can be solved to a constant.
        
        cur_target_term = constraint_info['single_terms'][0]
        assert len(cur_target_term) == 1
        if not is_constrained(cur_target_term):
            set_constrained(cur_target_term)
            
        add_contribution_count("simple_replacement_constant", cur_target_term[0])
        
        target_solved_var = negate_constant(constraint_info["constant"])
        target_solved_var = mul_constants(invert_constant(target_terms[cur_target_term]), target_solved_var)
        
        target_solved_var_sdp = SingleDotProduct({(): target_solved_var})
        new_replacement = {cur_target_term:target_solved_var_sdp}
        # plugging in is delegated to address_backwards_dependency
        
        address_backwards_dependency(None, new_replacement, constraint_info['r1cs_circuit'], constraint_info['constraint_id'])
        progress_made = True
        
        
    if len(constraint_info['single_terms']) == 2 and len(constraint_info['double_terms']) == 0:    
        # one single term solves to another, could directly substitute to all equations
        # this models where a var is replaceable by another var
        
        # NOTE: larger var is removed/replaced
        
        removed_var = constraint_info['single_terms'][0][0]
        remaining_var = constraint_info['single_terms'][1][0]

        if removed_var < remaining_var:
            removed_var, remaining_var = remaining_var, removed_var
            
        assert removed_var > remaining_var # Impossible for them to be equal
        
        r1cs_circuit = constraint_info["r1cs_circuit"]

        if removed_var in r1cs_circuit.output_list or removed_var in r1cs_circuit.input_list:
            # we only handle the case where intermediates are being substituted/removed, otherwise complications are introduced.
            return progress_made
            
        
        all_affected_vars = set()
        add_contribution_count("simple_replacement_expression", removed_var)
        
        # propogate constrainedness
        if is_constrained((removed_var,)) and not is_constrained((remaining_var,)):
            set_constrained((remaining_var,))
            all_affected_vars.add(remaining_var)
            
        if is_underconstrained((removed_var,)) and not is_underconstrained((remaining_var,)):
            set_underconstrained((remaining_var,))
            all_affected_vars.add(remaining_var)
        
        target_solved_dict = dict()
        target_solved_dict[(remaining_var,)] = negate_constant(target_terms[(remaining_var,)])
        target_solved_dict[()] = negate_constant(constraint_info['constant'])
        target_solved_var_sdp = SingleDotProduct(target_solved_dict)
        target_solved_var_sdp.mul_by_const_IN_PLACE(invert_constant(target_terms[(removed_var,)]))
        
        if target_solved_var_sdp.terms[()] == 0 and target_solved_var_sdp.terms[(remaining_var,)] == 1:
            # identity replacement, also handle binary propogation
            equivalent_vars.union(remaining_var, removed_var)
            all_eq_vars = equivalent_vars.get_group_members(remaining_var)
            if any([i in binary_vars for i in all_eq_vars]):
                for i in all_eq_vars:
                    if not is_binary(i):
                        set_binary(i)
                        all_affected_vars.add(i)
                        add_contribution_count("simple_replacement_expression_binary_propogation", removed_var)
                        
        equivalent_cons_vars.union(remaining_var, removed_var)
        
        all_eq_vars = equivalent_cons_vars.get_group_members(remaining_var)
        if any([is_constrained((i,)) for i in all_eq_vars]):
            for i in all_eq_vars:
                if not is_constrained((i,)):
                    set_constrained((i,))
                    all_affected_vars.add(i)
                    add_contribution_count("simple_replacement_expression_CONS_propogation", removed_var)    
    
        new_replacement = {(removed_var,):target_solved_var_sdp}
       
        address_backwards_dependency(None, new_replacement, constraint_info['r1cs_circuit'], constraint_info['constraint_id'])
        
        address_backwards_dependency(all_affected_vars, None, constraint_info['r1cs_circuit'], constraint_info['constraint_id'])
        
        progress_made = True
    return progress_made
            
    
    

def check_binary_expression(cur_sdp, constraint_info, get_original_info):
    progress_made = False
    
    if get_original_info:
        target_terms = cur_sdp.original_terms_copy
    else:
        target_terms = cur_sdp.terms
    
    if len(constraint_info['UNDER_terms']) > 0:
        # Not applicable
        return progress_made
    
    if len(constraint_info['NOT_SURE_terms']) < 2:
        # ONE term case is handled by multi_con_to_single_uncon
        return progress_made
    
    
    coef_to_term_d_NOT_SURE = dict()
    for one_term in constraint_info['NOT_SURE_terms']:
        # if there are non-binary (or non-double) variables in the NOT SURE part, then we are done here.
        if len(one_term) != 1:
            return progress_made
            
        if not is_binary(one_term[0]):
            return progress_made
    
        cur_coef = target_terms[one_term]
        if cur_coef not in coef_to_term_d_NOT_SURE:
            coef_to_term_d_NOT_SURE[cur_coef] = []
            coef_to_term_d_NOT_SURE[cur_coef].append(one_term)
        else:
            # not a good idea to deal with forms like x1 + 2x2 + 2x3 + 4x4
            return progress_made 
    
    # up to now, we are guaranteed that:
    #.    1. all NOT SURE (not guaranteed to be constrained) variables are in coef_to_term_d_NOT_SURE
    #.    2. coef_to_term_d_NOT_SURE are all binary variables
    #.    3. coef_to_term_d_NOT_SURE are all single-term (e.g., x1 not x1x2) variables
    #.    4. each coef have only one correspondance
    
    
    for expected_coefficient in [1, p-1]:
        processed_coefficient = []

        while expected_coefficient in coef_to_term_d_NOT_SURE:
            # we should enforce single correspondance (one coef to one term)
            #   since otherwise the combined variable would not be binary, since range changed.
            target_terms = coef_to_term_d_NOT_SURE[expected_coefficient]
            if len(target_terms) != 1:
                return progress_made

            # now, coef is as expected, is single term, and the current term is binary  
            target_term = target_terms[0]
            processed_coefficient.append(expected_coefficient)
            expected_coefficient = mul_constants(expected_coefficient, 2)


        if len(processed_coefficient) == len(coef_to_term_d_NOT_SURE):
            # all terms are binary and have expected coefs.
            for one_term in constraint_info['NOT_SURE_terms']:
                assert len(one_term) == 1
                add_contribution_count("binary_expression", one_term[0])
                if not is_constrained(one_term):
                    set_constrained(one_term)
                    
                    
            
            # NOTE: if the side term is simply a constant, then we could solve for the value of each of the binaries
            if constraint_info['constant'] == 0 and len([t for t in constraint_info['C_terms'] if len(t) > 0]) == 0:
                # sum of all of the NOT SURE terms add to 0
                
                target_solved_dict = dict()
                target_solved_dict[()] = 0 # all solve to 0
                target_solved_var_sdp = SingleDotProduct(target_solved_dict)
                
                for one_term in constraint_info['NOT_SURE_terms']:
                    new_replacement = dict()
                    new_replacement[one_term] = target_solved_var_sdp
                    add_contribution_count("binary_expression_propogated_solvable_int", one_term[0])
                    # plugging in is automatically addressed here
                    address_backwards_dependency(None, new_replacement, constraint_info['r1cs_circuit'], constraint_info['constraint_id'])
                    progress_made = True
        
        
            else:
                # not solving to 0
                address_backwards_dependency([t[0] for t in constraint_info['NOT_SURE_terms']], None, constraint_info['r1cs_circuit'], constraint_info['constraint_id'])
                progress_made = True

    return progress_made









        
def handle_BPG_case_1(cur_terms_map, cur_terms, constant_val, cur_cons_id, pre_conditions):
    # Case 1: c1 * v1 * v2 + c2 = 0
    progress_made = False
    if len(cur_terms) != 1:
        return False
    
    # Note: we might get single terms as a result of recursive calls, so handle those as well
    if len(cur_terms[0]) == 1:
        # this means the var is unconditionally constrained
        if constant_val == 0:
            add_edge_to_BGP_graph((), (cur_terms[0][0], 1), cur_cons_id, pre_conditions)
        elif constant_val == 'dirty':
            add_edge_to_BGP_graph((), (cur_terms[0][0], 3), cur_cons_id, pre_conditions)
        elif constant_val == p-1 and cur_terms_map[cur_terms[0]] == 1:
            add_edge_to_BGP_graph((), (cur_terms[0][0], 2), cur_cons_id, pre_conditions)
        else:
            add_edge_to_BGP_graph((), (cur_terms[0][0], 3), cur_cons_id, pre_conditions)
        return True
    
    
    if cur_terms[0][0] == cur_terms[0][1]:
        # Not handling squares here, handled in SMT phase
        return False
    
    
    if len(cur_terms[0]) == 1:
        # a single variable c1 * v1 + c2 = 0, handled in substitution
        # NOTE: won't survive to the BPG step
#         v1 = cur_terms[0][0]
        return False

    if constant_val == 'dirty':
        # a dirty constant value: one whose value we cannot be sure about
        return False
    
    assert len(cur_terms[0]) == 2

    c1 = cur_terms_map[cur_terms[0]]
    c2 = constant_val
    v1v2 = cur_terms[0]
    v1, v2 = cur_terms[0]

    assert c1 != 0

    # no need to add to pre_conditions here, since the edges are unconditional
    if c2 == 0:
        add_edge_to_BGP_graph((v1, -1), (v2, 1), cur_cons_id, pre_conditions)
        add_edge_to_BGP_graph((v2, -1), (v1, 1), cur_cons_id, pre_conditions)
        progress_made = True
    else:
        # NOTE: if preconditions exist, the from node is still empty and the conditions are encoded as edge condition
        # 
        add_edge_to_BGP_graph((), (v1, -1), cur_cons_id, pre_conditions)
        add_edge_to_BGP_graph((), (v2, -1), cur_cons_id, pre_conditions)
        progress_made = True
    return progress_made  
    

    
    
    
def handle_BPG_case_2(cur_terms_map, cur_terms, constant_val, cur_cons_id, pre_conditions):
    # Case 1: c1 * v1 * v2 + c2 * v3 * v4 + c3 = 0
    progress_made = False
    if len(cur_terms) != 2:
        return False
    
    c3 = constant_val
    all_vars_lst = []
    for one_term in cur_terms:
        assert len(one_term) > 0
        for one_var in one_term:
            all_vars_lst.append(one_var)
    all_vars_set = set(all_vars_lst)
    has_reps = len(all_vars_lst) != len(all_vars_set)
    if not has_reps:
        # checks for reps with preconditions
        # NOTE: also need to check if there are any repetitions of variable specifications in the conditions

        has_reps = any([cond_tuple[0] in all_vars_set for cond_tuple in pre_conditions])
    
    
    at_commutative = False    
    for cur_order in [[cur_terms[0], cur_terms[1]], [cur_terms[1], cur_terms[0]]]:
        # encodes the order-independency and commutativity of our method
        v1v2, v3v4 = cur_order
        c1 = cur_terms_map[v1v2]
        c2 = cur_terms_map[v3v4]
        
        if not has_reps:
            if len(v1v2) == 1:
                # reduces to lower level case.
                # it is possible to have multiple edges that flow from one node to another, but with different conditions. That is totally normal.
                # if 2 conditions appear in a condition tuple, that means conjunction of them.
                cur_progress_made = handle_BPG_case_1({v3v4: c2}, [v3v4], "dirty", cur_cons_id, [(v1v2[0], -1)] + pre_conditions)
                progress_made = progress_made or cur_progress_made
                cur_progress_made = handle_BPG_case_1({v3v4: c2}, [v3v4], constant_val, cur_cons_id, [(v1v2[0], 1)] + pre_conditions)
                progress_made = progress_made or cur_progress_made

                if not at_commutative:
                    # since this part is inherently symmetric
                    if len(v3v4) == 1:
                        # This was previously the variable cons. equivalency edge
                        add_edge_to_BGP_graph((v1v2[0], 3), (v3v4[0], 3), cur_cons_id, pre_conditions)
                        add_edge_to_BGP_graph((v3v4[0], 3), (v1v2[0], 3), cur_cons_id, pre_conditions)
                        progress_made = True
                    else:
                        assert len(v3v4) == 2
                        # note:complications are introduced if some variables are identical
                        # This was previously the variable cons. equivalency edge
                        add_edge_to_BGP_graph((v1v2[0], 3), (v3v4[0], 3), cur_cons_id, [(v3v4[1], -1)] + pre_conditions)
                        add_edge_to_BGP_graph((v3v4[0], 3), (v1v2[0], 3), cur_cons_id, [(v3v4[1], -1)] + pre_conditions)

                        add_edge_to_BGP_graph((v1v2[0], 3), (v3v4[1], 3), cur_cons_id, [(v3v4[0], -1)] + pre_conditions)
                        add_edge_to_BGP_graph((v3v4[1], 3), (v1v2[0], 3), cur_cons_id, [(v3v4[0], -1)] + pre_conditions)
                        progress_made = True
                        

            else:
                assert len(v1v2) == 2
                cur_progress_made = handle_BPG_case_1({v3v4: c2}, [v3v4], "dirty", cur_cons_id, [(v1v2[0], -1), (v1v2[1], -1)] + pre_conditions)
                progress_made = progress_made or cur_progress_made
                cur_progress_made = handle_BPG_case_1({v3v4: c2}, [v3v4], constant_val, cur_cons_id, [(v1v2[0], 1)] + pre_conditions)
                progress_made = progress_made or cur_progress_made
                cur_progress_made = handle_BPG_case_1({v3v4: c2}, [v3v4], constant_val, cur_cons_id, [(v1v2[1], 1)] + pre_conditions)
                progress_made = progress_made or cur_progress_made

                if not at_commutative:
    
                    if len(v3v4) == 1:
                        # This was previously the variable cons. equivalency edge
                        add_edge_to_BGP_graph((v1v2[0], 3), (v3v4[0], 3), cur_cons_id, [(v1v2[1], -1)] + pre_conditions)
                        add_edge_to_BGP_graph((v3v4[0], 3), (v1v2[0], 3), cur_cons_id, [(v1v2[1], -1)] + pre_conditions)
                        
                        add_edge_to_BGP_graph((v1v2[1], 3), (v3v4[0], 3), cur_cons_id, [(v1v2[0], -1)] + pre_conditions)
                        add_edge_to_BGP_graph((v3v4[0], 3), (v1v2[1], 3), cur_cons_id, [(v1v2[0], -1)] + pre_conditions)
                        progress_made = True
                        
                    else:
                        assert len(v3v4) == 2
                        # note:complications are introduced if some variables are identical
                        # This was previously the variable cons. equivalency edge
                        add_edge_to_BGP_graph((v1v2[1], 3), (v3v4[1], 3), cur_cons_id, [(v1v2[0], -1), (v3v4[0], -1)] + pre_conditions)
                        add_edge_to_BGP_graph((v3v4[1], 3), (v1v2[1], 3), cur_cons_id, [(v1v2[0], -1), (v3v4[0], -1)] + pre_conditions)
                        
                        add_edge_to_BGP_graph((v1v2[0], 3), (v3v4[1], 3), cur_cons_id, [(v1v2[1], -1), (v3v4[0], -1)] + pre_conditions)
                        add_edge_to_BGP_graph((v3v4[1], 3), (v1v2[0], 3), cur_cons_id, [(v1v2[1], -1), (v3v4[0], -1)] + pre_conditions)
                        
                        add_edge_to_BGP_graph((v1v2[1], 3), (v3v4[0], 3), cur_cons_id, [(v1v2[0], -1), (v3v4[1], -1)] + pre_conditions)
                        add_edge_to_BGP_graph((v3v4[0], 3), (v1v2[1], 3), cur_cons_id, [(v1v2[0], -1), (v3v4[1], -1)] + pre_conditions)
                        
                        add_edge_to_BGP_graph((v1v2[0], 3), (v3v4[0], 3), cur_cons_id, [(v1v2[1], -1), (v3v4[1], -1)] + pre_conditions)
                        add_edge_to_BGP_graph((v3v4[0], 3), (v1v2[0], 3), cur_cons_id, [(v1v2[1], -1), (v3v4[1], -1)] + pre_conditions)
                        progress_made = True
                        
        at_commutative = True
    
    
    if c3 != 'dirty':
        for cur_order in [[cur_terms[0], cur_terms[1]], [cur_terms[1], cur_terms[0]]]:
            # encodes the order-independency and commutativity of our method
            v1v2, v3v4 = cur_order
            c1 = cur_terms_map[v1v2]
            c2 = cur_terms_map[v3v4]

            if len(v3v4) != 1 or add_constants(c1, c2) != 0 or len(v1v2) != 2 or v3v4[0] not in v1v2 or len(set(v1v2)) == 1:
                continue

            common_var = [i for i in v1v2 if i == v3v4[0]]
            assert len(common_var) == 1
            common_var = common_var[0]
            other_var = [i for i in v1v2 if i != common_var]
            assert len(other_var) == 1
            other_var = other_var[0]
            

            # 𝑐1𝑣1 (𝑣2 − 1) + 𝑐3
            # i.e., c1 * common_var (other_var − 1) + c3
            # v1 is common var, v2 is other var. c1 is c1, c2 is -c1
            if c3 == 0:
                add_edge_to_BGP_graph((common_var, -1), (other_var, 2), cur_cons_id, pre_conditions)
                add_edge_to_BGP_graph((other_var, -2), (common_var, 1), cur_cons_id, pre_conditions)
                progress_made = True
            else:
                # NOTE: if preconditions exist, the from node is still empty and the conditions are encoded as edge condition
                # 
                add_edge_to_BGP_graph((), (common_var, -1), cur_cons_id, pre_conditions)
                add_edge_to_BGP_graph((), (other_var, -2), cur_cons_id, pre_conditions)
                progress_made = True
        
        
        for cur_order in [[cur_terms[0], cur_terms[1]], [cur_terms[1], cur_terms[0]]]:
            # encodes the order-independency and commutativity of our method
            v1v2, v3v4 = cur_order
            c1 = cur_terms_map[v1v2]
            c2 = cur_terms_map[v3v4]

            if len(v3v4) != 1 or add_constants(c1, c2) != 0 or len(v1v2) != 2 or v3v4[0] not in v1v2 or len(set(v1v2)) != 1 or c3 != 0:
                continue

            common_var = set([v3v4[0], v1v2[0], v1v2[1]])
            assert len(common_var) == 1
            common_var = list(common_var)[0]
            add_edge_to_BGP_graph((common_var, -1), (common_var, 2), cur_cons_id, pre_conditions)
            add_edge_to_BGP_graph((common_var, -2), (common_var, 1), cur_cons_id, pre_conditions)
            progress_made = True
        
        
    return  progress_made
        
    




def handle_BPG(cur_terms_map, cur_terms, constant_val, cur_cons_id, pre_conditions, max_depth = 4):
    # this is the entry point handling the general case.
    progress_made = False 
    
    if len(cur_terms) > max_depth:
        return False
    
    if len(cur_terms) == 1:
        cur_progress_made = handle_BPG_case_1(cur_terms_map, cur_terms, constant_val, cur_cons_id, pre_conditions)
        progress_made = progress_made or cur_progress_made
    elif len(cur_terms) == 2:
        cur_progress_made = handle_BPG_case_2(cur_terms_map, cur_terms, constant_val, cur_cons_id, pre_conditions)
        progress_made = progress_made or cur_progress_made
    elif len(cur_terms) <= max_depth:
        for cur_one_term in cur_terms:
            # encodes the order-independency and commutativity of our method
            this_map = cur_terms_map.copy()
            this_map.pop(cur_one_term)
            this_terms = [i for i in cur_terms if i != cur_one_term]

            if len(cur_one_term) == 1:
                cur_progress_made = handle_BPG(this_map, this_terms, "dirty", cur_cons_id, [(cur_one_term[0], -1)] + pre_conditions, max_depth)
                progress_made = progress_made or cur_progress_made
                
                
                cur_progress_made = handle_BPG(this_map, this_terms, constant_val, cur_cons_id, [(cur_one_term[0], 1)] + pre_conditions, max_depth)
                progress_made = progress_made or cur_progress_made
            else:
                assert len(cur_one_term) == 2
                
                cur_progress_made = handle_BPG(this_map, this_terms, "dirty", cur_cons_id, [(cur_one_term[0], -1), (cur_one_term[1], -1)] + pre_conditions, max_depth)
                progress_made = progress_made or cur_progress_made
                
                cur_progress_made = handle_BPG(this_map, this_terms, constant_val, cur_cons_id, [(cur_one_term[0], 1)] + pre_conditions, max_depth)
                progress_made = progress_made or cur_progress_made
                
                cur_progress_made = handle_BPG(this_map, this_terms, constant_val, cur_cons_id, [(cur_one_term[1], 1)] + pre_conditions, max_depth)
                progress_made = progress_made or cur_progress_made
            
    return progress_made



def construct_BPG_edges(cur_sdp, constraint_info):
    global BPG_DEPTH
                      
    progress_made = False  
    for consideration in ["current", "original"]:
        if consideration == "original":
            if cur_sdp.original_terms_copy == cur_sdp.terms:
                continue
            else:
                cur_terms = cur_sdp.original_terms_copy
        else:
            cur_terms = cur_sdp.terms

        cur_terms_map = {k: cur_terms[k] for k in cur_terms if cur_terms[k] != 0 and len(k) != 0}
        assert all([v!=0 for v in cur_terms_map.values()])
        cur_terms = list(cur_terms_map.keys())
        constant_val = constraint_info['constant']
        cur_cons_id = constraint_info['constraint_id']

        pre_conditions = []
        cur_progress_made = handle_BPG(cur_terms_map, cur_terms, constant_val, cur_cons_id, pre_conditions, max_depth = BPG_DEPTH)
        progress_made = progress_made or cur_progress_made
        
    return progress_made








def collect_all_consequences(graph, invariant_properties, start_node, pre_knowledge = None):
    if pre_knowledge is None:
        all_consequences = set()
    else:
        all_consequences = pre_knowledge # pre_knowledge is already a deep copy
    
    
    # NOTE: invariants will not have conditions, since if they do, we would have handled them while constructing bpg edges
    for prop_edge_id in bpg_graph.invariant_properties:
        # add all invariant properties that are always true
        if prop_edge_id not in edge_id_to_edge:
            continue
        prop_edge = edge_id_to_edge[prop_edge_id]
        assert prop_edge.from_node == ()
        prop = prop_edge.to_node

        # invariant properties are guaranteed to hold, regardless.
        all_consequences.add(prop)
         
    old_length = len(all_consequences)
    while True:
        visited = set()
        dfs_traverse_BPG(graph, start_node, visited, all_consequences, is_initial = True)
        if len(all_consequences) == old_length:
            break
        else:
            old_length = len(all_consequences)
    return all_consequences
    
    
def dfs_traverse_BPG(graph, start_node, visited, all_consequences, is_initial = False):
    # returns all of the consequences of the start node (i.e., a var taking some property)
    
    # one node is a tuple
    global edge_id_to_edge
    global cons_id_to_edges

    if start_node in visited:
        return
    visited.add(start_node)
    
    # offload the checking of visited for simultaneously subsequent nodes to recursed DFS.

    implicit_map = {1: [1, -2, 3], 2: [-1, 2, 3], -1: [-1, 3], -2: [-2, 3], 3: [3]}
    
    start_var = start_node[0]
    start_property = start_node[1]
    
    # if a var is non 0/1 and it is known to be binary, then we could also infer it to be 1/0
    if start_property == -1 and start_var in binary_vars:
        all_consequences.add((start_var, 2))
    if start_property == -2 and start_var in binary_vars:
        all_consequences.add((start_var, 1))
    
    for this_property in implicit_map[start_property]:
        start_node = (start_var, this_property)
        if not is_initial:
            # not adding the very initial assumption
            all_consequences.add(start_node)

        # If the node has no further neighbors
        # if not this node, or this node has no next steps, then the path terminates
        if start_node[0] not in graph or start_node not in graph[start_node[0]] or not graph[start_node[0]][start_node]:
            continue
            
        # Continue DFS for all neighbors
        for one_var in graph[start_node[0]][start_node]:
            for next_step_neighbor_edge_id in graph[start_node[0]][start_node][one_var]:
                if next_step_neighbor_edge_id not in edge_id_to_edge:
                    # has already been removed from reruns
                    continue
                    
                next_step_neighbor_edge = edge_id_to_edge[next_step_neighbor_edge_id]
                assert next_step_neighbor_edge.from_node == start_node
                assert isinstance(next_step_neighbor_edge, BPG_Edge)
                # all edge conditions must be satisfied.
                if next_step_neighbor_edge.to_node not in visited:
                    if all([one_condition in all_consequences for one_condition in next_step_neighbor_edge.edge_condition]):
                        dfs_traverse_BPG(graph, next_step_neighbor_edge.to_node, visited, all_consequences)
                        # note: the to nodes will be added in the next dfs call.

                        
                        

        
def BPG_check_binary(p1_d, p2_d):
    global binary_vars
    global contribution_count
    
    # Inputs are dicts
    affected_vars = set()
    replacements = []
    
    # binary: 
    for v1 in p1_d:
        if v1 in binary_vars:
            # skip if already binary
            continue

        if v1 not in p2_d:
            # skip if not common to both
            continue

        if (v1, 1) in p1_d[v1] and (v1, 2) in p2_d[v1]:
            # in one path, it is 0. In the other path, which collectively determin all cases, is 1. 
            #.   then its value range is constrained to must be 0/1
            binary_vars.add(v1)
            affected_vars.add(v1)
            add_contribution_count("BPG_binary", v1)

        elif (v1, 2) in p1_d[v1] and (v1, 1) in p2_d[v1]:
            binary_vars.add(v1)
            affected_vars.add(v1)
            add_contribution_count("BPG_binary", v1)

            
            
        solved_val = None
        if (v1, 1) in p1_d[v1] and (v1, 1) in p2_d[v1]:
            solved_val = 0
        elif (v1, 2) in p1_d[v1] and (v1, 2) in p2_d[v1]:
            assert solved_val is None, "conflict found: both 1 and 0 at the same time, overconstrained!"
            solved_val = 1
        
        if solved_val is not None:
            target_solved_var_sdp = SingleDotProduct({(): solved_val})
            new_replacement = {(v1,):target_solved_var_sdp}
            replacements.append(new_replacement)
            add_contribution_count("BPG_constrained_solved_to_one_val", v1)

    return affected_vars, replacements

            

            
def check_proved_constrainedness(p1, p2, p1_d, p2_d):
    global contribution_count
    affected_vars = set()
    cur_constrained_vars = set()
    for v1 in p1_d: 
        if len(p1_d[v1]) > 0 and v1 in p2_d:
            if len(p2_d[v1]) > 0:
                cur_constrained_vars.add(v1)

    for i in cur_constrained_vars:
        if not is_constrained((i,)):
            set_constrained((i,))
            affected_vars.add(i)
            add_contribution_count("BPG_constrained_both_solved", i)

    return affected_vars, []




def traverse_BPG_for_info(start_time, timeout_reasoning_engine):
    global bpg_graph
    global edge_id_to_edge
    global cons_id_to_edges
    
    
    bpg_start_time = time.time()
    
    complementary_variables_1 = [((var, 1), (var, -1)) for var in bpg_graph.graph if (var, 1) in bpg_graph.graph[var] and (var, -1) in bpg_graph.graph[var]]
    complementary_variables_2 = [((var, 2), (var, -2)) for var in bpg_graph.graph if (var, 2) in bpg_graph.graph[var] and (var, -2) in bpg_graph.graph[var]]
    complementary_variables_3 = [((var, -1), (var, -2)) for var in bpg_graph.graph if (var, -1) in bpg_graph.graph[var] and (var, -2) in bpg_graph.graph[var]]
    
    complementary_variables = complementary_variables_1 + complementary_variables_2 + complementary_variables_3
    
    complementary_variables_1 = [((var, 1), (var, -1)) for var in bpg_graph.graph if (var, 1) in bpg_graph.graph[var] and (var, -1) in bpg_graph.graph[var] and is_constrained((var,))]
    complementary_variables_2 = [((var, 2), (var, -2)) for var in bpg_graph.graph if (var, 2) in bpg_graph.graph[var] and (var, -2) in bpg_graph.graph[var] and is_constrained((var,))]
    complementary_variables_3 = [((var, -1), (var, -2)) for var in bpg_graph.graph if (var, -1) in bpg_graph.graph[var] and (var, -2) in bpg_graph.graph[var] and is_constrained((var,))]
    
    complementary_variables_cons = complementary_variables_1 + complementary_variables_2 + complementary_variables_3
    
    # this cannot infer cons.
    complementary_variables_not_cons_only = [i for i in complementary_variables if i not in complementary_variables_cons]
    

    affected_vars = set()
    replacements = []
    # traverses all edges that is reacheable from constrained vars.
    # this is preknowledge that are guaranteed to be true, regardless of whichever hypothesis.
    pre_knowledge = set()
    for one_var in bpg_graph.graph:
        if (one_var, 3) in bpg_graph.graph[one_var] and is_constrained((one_var,)):
            # all of the known constrained variables could be used to infer more variables to be constrained.
            cur_knowledge = collect_all_consequences(bpg_graph.graph, bpg_graph.invariant_properties, (one_var, 3))
            pre_knowledge.update(cur_knowledge)
    
    for var_pair in complementary_variables_cons:
        
        if time.time() - start_time > timeout_reasoning_engine:
            raise TimeoutError("timeout")
        
        
        var = var_pair[0][0]
        pair_1 = var_pair[0]
        pair_2 = var_pair[1]
        # note: two var pairs share the same var
        p1 = collect_all_consequences(bpg_graph.graph, bpg_graph.invariant_properties, pair_1, pre_knowledge.copy())
        p2 = collect_all_consequences(bpg_graph.graph, bpg_graph.invariant_properties, pair_2, pre_knowledge.copy())
        p1_d = dictify_all_paths(p1)
        p2_d = dictify_all_paths(p2)
        
        cur_affected_vars, cur_replacements = check_proved_constrainedness(p1, p2, p1_d, p2_d)
        affected_vars.update(cur_affected_vars)
        replacements.extend(cur_replacements)
        
        cur_affected_vars, cur_replacements =  BPG_check_binary(p1_d, p2_d)
        affected_vars.update(cur_affected_vars)
        replacements.extend(cur_replacements)
    
    for var_pair in complementary_variables_not_cons_only:
        
        if time.time() - start_time > timeout_reasoning_engine:
            raise TimeoutError("timeout")
            
        var = var_pair[0][0]
        pair_1 = var_pair[0]
        pair_2 = var_pair[1]
        # note: two var pairs share the same var
        p1 = collect_all_consequences(bpg_graph.graph, bpg_graph.invariant_properties, pair_1, pre_knowledge.copy())
        p2 = collect_all_consequences(bpg_graph.graph, bpg_graph.invariant_properties, pair_2, pre_knowledge.copy())
        p1_d = dictify_all_paths(p1)
        p2_d = dictify_all_paths(p2)
        
        cur_affected_vars, cur_replacements =  BPG_check_binary(p1_d, p2_d)
        affected_vars.update(cur_affected_vars)
        replacements.extend(cur_replacements)
       
    add_contribution_count("bpg_time_records", time.time() - bpg_start_time)

    return affected_vars, replacements


 
    
    
############################## Some utils ##############################

        
def clear_data_structures():
    global equivalent_vars
    global equivalent_cons_vars
    global edge_id_to_edge
    global cons_id_to_edges
    global replacement_dict
    global binary_vars
    global var_to_cons_id
    global term_to_constrainedness
    global contribution_count
    global cur_processing_cons_id
    global z3_variables
    global sympy_symbols
    global counter_example
    global assumptions 
    global bpg_graph
    global linear_subs_back_solving
    global linear_subs_back_solving_constants
    
    

    linear_subs_back_solving_constants.clear()
    linear_subs_back_solving.clear()
    z3_variables.clear()
    sympy_symbols.clear()


    counter_example.clear()

    cur_processing_cons_id = 0
    Edge.next_edge_id = 0
    equivalent_vars.parent.clear()
    equivalent_cons_vars.parent.clear()
    edge_id_to_edge.clear()
    cons_id_to_edges.clear()
    replacement_dict.clear()
    binary_vars.clear()
    var_to_cons_id.clear()
    term_to_constrainedness.clear()
    contribution_count.clear()

    bpg_graph.invariant_properties.clear()
    bpg_graph.graph.clear()
    
    assumptions.assumptions.clear()
    
    contribution_count.clear()
    
    set_constrained(())

def is_binary(var):
    global binary_vars
    
    return var in binary_vars
    
def set_binary(var):
    global binary_vars
    binary_vars.add(var)
    
    
def is_constrained(var_pair):
    global term_to_constrainedness
    check_var_pair(var_pair)
     
    if var_pair not in term_to_constrainedness:
        term_to_constrainedness[var_pair] = 2
        return False
    else:
        return term_to_constrainedness[var_pair] == 1

    
def is_underconstrained(var_pair):
    global term_to_constrainedness
    check_var_pair(var_pair)
    
    if var_pair not in term_to_constrainedness:
        term_to_constrainedness[var_pair] = 2
        return False
    else:
        return term_to_constrainedness[var_pair] == 3 
    

        
def is_undetermined(var_pair):
    global term_to_constrainedness
    check_var_pair(var_pair)
    
    if var_pair not in term_to_constrainedness:
        term_to_constrainedness[var_pair] = 2
        return True
    else:
        return term_to_constrainedness[var_pair] == 2   
    
    
def set_constrained(var_pair):
    global term_to_constrainedness
    global contribution_count
    global equivalent_cons_vars
    
    check_var_pair(var_pair)
    
    if var_pair not in term_to_constrainedness:
        term_to_constrainedness[var_pair] = 1
    else:
        assert term_to_constrainedness[var_pair] == 2, "only supporting setting false to true"
        term_to_constrainedness[var_pair] = 1
    
    if len(var_pair) == 1:
        # need to also propogate to related double terms
        new_con_var = var_pair[0]
        candidate_doubles = [t for t in term_to_constrainedness if new_con_var in t and term_to_constrainedness[t] == 2 and len(t) == 2]
        for candidate_double in candidate_doubles:
            if is_constrained((candidate_double[0],)) and is_constrained((candidate_double[1],)):
                add_contribution_count("single_to_double", candidate_double)
                if not is_constrained(candidate_double):
                    set_constrained(candidate_double)
       
        # also need to propagate to related terms
        
        for eq_var in equivalent_cons_vars.get_group_members(new_con_var):
            term_to_constrainedness[(eq_var,)] = 1
    elif len(var_pair) == 2:
        eq_1 = equivalent_vars.get_group_members(var_pair[0])
        eq_2 = equivalent_vars.get_group_members(var_pair[1])
        for v1 in eq_1:
            for v2 in eq_2:
                if v1 > v2:
                    v1,v2 = v2,v1
                term_to_constrainedness[(v1,v2)] = 1
        
        
    
    


def set_underconstrained(var_pair):
    global term_to_constrainedness
    check_var_pair(var_pair)

    if var_pair not in term_to_constrainedness:
        term_to_constrainedness[var_pair] = 3
    else:
        assert term_to_constrainedness[var_pair] == 2, "only supporting setting false to true"
        term_to_constrainedness[var_pair] = 3
            
    
    
    


def check_termination(r1cs_circuit):
    # returns is_all_variables_determined, constrainedness_dict
    # constrainedness_dict is None if not all determined
    
    has_UNDER = False
    has_NOT_SURE = False
    
    status_dict = dict() # 1: constrained, 2: NOT SURE, 3: UNDERconstrained

    for output_var in r1cs_circuit.output_list:
        if is_constrained((output_var,)):
            status_dict[output_var] = 1
        elif is_underconstrained((output_var,)):
            status_dict[output_var] = 3
            has_UNDER = True
        else:
            status_dict[output_var] = 2
            has_NOT_SURE = True
    # constrained only if all output vars are constrained.
    
    if has_UNDER:
        return True, "\n\nUNDER-CONSTRAINED!"
    else:
        if has_NOT_SURE:
            return False, "NOT SURE"
        else:
            return True, "\n\nCONSTRAINED!"
        


       
            

############################## Parsing R1CS ##############################
   
def parse_r1cs_circuit_constraints(r1cs_circuit, timeout_time, starting_time):

    def parse_single_r1cs_constraint(block_a, block_b, block_c, timeout_time, starting_time):
        def parse_constraint_block(block):
            sdp = SingleDotProduct()

            for w, f in zip(block.wids, block.factors):
                if w == 0:
                    # constant wire has 0 id
                    var_pair = ()
                else:
                    var_pair = (w,)
                coefficient = f
                
                if len(var_pair) != 0:
                    assert var_pair not in sdp.terms
                
                sdp.add_term_IN_PLACE(var_pair, coefficient)

            return sdp

        a = parse_constraint_block(block_a)
        b = parse_constraint_block(block_b)
        c = parse_constraint_block(block_c)

        return {'A': a, 'B': b, 'C': c}


    
    r1cs_circuit.parsed_constraints_ABC = []
    i = 0
    for cur_constraint in r1cs_circuit.constraint_section:
        i += 1
        if i % 50 == 0:
            if timeout_time is not None:
                assert starting_time is not None
                if time.time() - starting_time > timeout_time:
                    raise TimeoutError("The operation took too long")
                
                
        block_a = cur_constraint.block_a
        block_b = cur_constraint.block_b
        block_c = cur_constraint.block_c
        r1cs_circuit.parsed_constraints_ABC.append(parse_single_r1cs_constraint(block_a, block_b, block_c, timeout_time, starting_time))
        



        
        
def combine_all_constraints(r1cs_circuit, timeout_time, starting_time):
    def combine_linear_combinations(Ax, Bx, Cx):
        result = SingleDotProduct()

        for a_var_pair in Ax.terms:  
            for b_var_pair in Bx.terms:  
                new_coefficient = mul_constants(Ax.terms[a_var_pair], Bx.terms[b_var_pair])
                
                if len(a_var_pair) == 1:
                    if len(b_var_pair) == 1:
                        a_var = a_var_pair[0]
                        b_var = b_var_pair[0]
                        new_var_pair = (min([a_var, b_var]), max([a_var, b_var]))
                    else:
                        a_var = a_var_pair[0]
                        new_var_pair = (a_var,)
                else:
                    if len(b_var_pair) == 1:
                        b_var = b_var_pair[0]
                        new_var_pair = (b_var,)
                    else:
                        new_var_pair = ()
                
                result.add_term_IN_PLACE(new_var_pair, new_coefficient)
                
                
        result.sub_sdp_IN_PLACE(Cx)
            
        return result
    
    

    r1cs_circuit.all_constraints = []

    i = 0
    for constraint in r1cs_circuit.parsed_constraints_ABC:
        i += 1
        if i % 50 == 0:
            if timeout_time is not None:
                assert starting_time is not None
                if time.time() - starting_time > timeout_time:
                    raise TimeoutError("The operation took too long")
                    
        A = constraint['A']
        B = constraint['B']
        C = constraint['C']

        combined_constraint = combine_linear_combinations(A, B, C)
        combined_constraint.original_terms_copy = combined_constraint.terms.copy()
        r1cs_circuit.all_constraints.append(combined_constraint)

    
          
def parse_entire_circuit(filename, print_all_circuits = False, timeout_time = None, starting_time = None):
    global p
    global var_to_cons_id
    global cons_id_to_edges
    

    r1cs_circuit = read_r1cs(filename, timeout_time, starting_time)
    print("read done")
    nwires = r1cs_circuit.header_vars.nwires
    mconstraints = r1cs_circuit.header_vars.mconstraints
    field_size = r1cs_circuit.header_vars.field_size
    if print_all_circuits:
        print(f'{filename} has {mconstraints} constraints, {len(r1cs_circuit.input_list)} inputs, {len(r1cs_circuit.output_list)} outputs. Inputs are {r1cs_circuit.input_list} and outputs are {r1cs_circuit.output_list}')
        
    print(f'{mconstraints} constraints, {len(r1cs_circuit.input_list)} inputs, {len(r1cs_circuit.output_list)} outputs.')

    parse_r1cs_circuit_constraints(r1cs_circuit, timeout_time, starting_time)
    print("parse done")

    combine_all_constraints(r1cs_circuit, timeout_time, starting_time)

    integer_value = int.from_bytes(r1cs_circuit.header_vars.prime_number, 'little')  # Use 'little' for little-endian
    if p != integer_value:
        print(f"Setting the prime field to {integer_value}")
        p = integer_value
    
    for i in range(len(r1cs_circuit.all_constraints)):
        
        if i % 50 == 0:
            if timeout_time is not None:
                assert starting_time is not None
                if time.time() - starting_time > timeout_time:
                    raise TimeoutError("The operation took too long")
        
        
        cons_id_to_edges[i] = []
        
        cur_sdp = r1cs_circuit.all_constraints[i]
        
        set_vars = set()
        for one_term in cur_sdp.terms:
            set_vars.update(one_term)
            
            # adds default non-constrained label to all terms
            is_constrained(one_term)
                
        for one_var in set_vars:
            if one_var not in var_to_cons_id:
                var_to_cons_id[one_var] = []
            var_to_cons_id[one_var].append(i)
    
    
    r1cs_circuit.input_list.remove(0)
    constrained_vars = set(r1cs_circuit.input_list)
    for single_var in constrained_vars:
        set_constrained((single_var,))
        
    
    if print_all_circuits:
        print("\n\nbelow are full terms:\n\n")
        for i in range(len(r1cs_circuit.all_constraints)):
            cur_sdp = r1cs_circuit.all_constraints[i]
            print(cur_sdp.get_sdp_str(include_constrained_terms=True, include_constant_term=True))

    
    return r1cs_circuit            
            
            
            
def check_var_assignment_SAT(constraints_terms_d_lst, variable_assignments):
    # check if all cons evaluate to 0, used to verify counter-examples

    violation_inds = [] # good for debugging

    for i in range(len(constraints_terms_d_lst)):
        constraint = constraints_terms_d_lst[i]

        constraint_value = 0
        for terms, coefficient in constraint.items():
            
            if terms == ():
                # a constant term
                term_value = coefficient
            else:
                # a product of variables
                term_value = coefficient
                for var in terms:
                    term_value = mul_constants(term_value, variable_assignments[var])
                    
            constraint_value = add_constants(term_value, constraint_value)
        
        if constraint_value != 0:
            violation_inds.append(i)

    return len(violation_inds) == 0, violation_inds


############################## SDP representation and operations ##############################

class SingleDotProduct:
    # Used to represent a dot product of two vectors in R1CS, which is in the form of a linear combination of double-variable terms
    # unless specified, we assume by default that modifications to be not in-place

    def __init__(self, initial_terms = None):
        if initial_terms:
            self.terms = initial_terms.copy()
            if () not in self.terms:
                self.terms[()] = 0
        else:
            self.terms = {(): 0}  # a dict to store terms with their coefficients.
        
    def contains_single_var(self, single_var):
        # returns if any of the terms, binary or unary, contains the given single var.
        for var_pair in self.terms.keys():
            if len(var_pair) == 0:
                continue
                
            if single_var in var_pair:
                return True
        return False
    
    def contains_term(self, var_pair):
        check_var_pair(var_pair)
        return var_pair in self.terms
    
    
    
    def clear_zero_terms(self):
        # removes all terms with a coefficient of zero from self.terms.
        zero_coefficient_var_pairs = [var_pair for var_pair, coefficient in self.terms.items() if coefficient == 0]
        
        for var_pair in zero_coefficient_var_pairs:
            if var_pair != ():
                del self.terms[var_pair]
    

    def get_terms_deep_copy(self, include_constrained_terms = False, include_constant_term = True):
        # a deep copy is returned
        if include_constrained_terms:
            if not include_constant_term:
                return {k: v for k, v in self.terms.items() if len(k) != 0}
            else:
                return self.terms.copy()
        else:
            if not include_constant_term:
                return {k: v for k, v in self.terms.items() if len(k) != 0 and not is_constrained(k)}
            else:
                # Use the is_constrained function to check constraint status
                return {k: v for k, v in self.terms.items() if not is_constrained(k)}




    def add_term_IN_PLACE(self, var_pair, coefficient):
        check_var_pair(var_pair)
        check_constant(coefficient)
        
        if var_pair in self.terms:
            self.terms[var_pair] = add_constants(coefficient, self.terms[var_pair])
        else:
            self.terms[var_pair] = coefficient
            
        self.clear_zero_terms()
        
        
    def sub_term_IN_PLACE(self, var_pair, coefficient):
        check_var_pair(var_pair)
        check_constant(coefficient)

        negated_coefficient = negate_constant(coefficient)
        self.add_term_IN_PLACE(var_pair, negated_coefficient)

    def add_sdp_IN_PLACE(self, other_sdp):
        for var_pair, coefficient in other_sdp.terms.items():
            self.add_term_IN_PLACE(var_pair, coefficient)
        
    def sub_sdp_IN_PLACE(self, other_sdp):
        for var_pair, coefficient in other_sdp.terms.items():
            self.sub_term_IN_PLACE(var_pair, coefficient)    




    def get_common_variable(self, include_constrained_terms=True):
        non_constant_terms = self.get_terms_deep_copy(include_constrained_terms=include_constrained_terms, include_constant_term = False)
        
        if non_constant_terms:
            first_term_vars = set(list(non_constant_terms.keys())[0])
            common_vars = set(first_term_vars)
        else:
            return False, None

        for var_pair in list(self.terms.keys()):
            common_vars.intersection_update(var_pair)

        assert len(common_vars) <= 1, "there cannot be multiple terms that are the same"
        if len(common_vars) == 1:
            return True, next(iter(common_vars))
        else:
            assert len(common_vars) == 0, "there cannot be multiple terms that are the same"
            return False, None



    
    
    def get_sdp_str(self, include_constrained_terms=False, include_constant_term=True, entire_constraint_level=False):
        # Returns a string representation of the dot product.
        terms_str_list = []
        all_terms_str_list = []
        for var_pair, coefficient in self.terms.items():
            if not include_constant_term and len(var_pair) == 0:
                continue
            if var_pair == ():
                term_str = f"{coefficient}"
            else:
                if len(var_pair) == 2:
                    variables_str = ' * '.join([f"x{var}" for var in sorted(var_pair)])
                elif len(var_pair) == 1:
                    variables_str = f"x{var_pair[0]}"
                
                term_str = f"{coefficient} * {variables_str}"
                
            all_terms_str_list.append(term_str)   
            if (not include_constrained_terms) and is_constrained(var_pair):
                continue
                
            terms_str_list.append(term_str)
            
        if entire_constraint_level and len(terms_str_list) > 0:
            return ' + '.join(all_terms_str_list)
            
            
        return ' + '.join(terms_str_list)

    
    
    def negate_term_IN_PLACE(self, var_pair):
        assert var_pair in self.terms, "the term does not exist in the SDP."
        # this naturally handles the case where the coefficient could be constant
        self.terms[var_pair] = negate_constant(self.terms[var_pair])

    def mul_by_const_IN_PLACE(self, const):
        assert const != 0, "should not zero out the entire sdp"
        for var_pair in self.terms:
            self.terms[var_pair] = mul_constants(self.terms[var_pair], const)
        

    
    def get_involved_single_vars(self):
        set_single_vars = set()
        for t in self.terms:
            set_single_vars.update(t)
        return set_single_vars
    
    
    
    
            
        
    def __str__(self):
        return self.get_sdp_str(include_constrained_terms=False, include_constant_term=True)
        
    def __eq__(self, other):
        assert isinstance(other, SingleDotProduct)

        self.clear_zero_terms()
        other.clear_zero_terms()

        return self.terms == other.terms    
        
            
       
    def plug_in_expression_of_term_IN_PLACE(self, term, expression_sdp):
        # plugs expression_sdp into self.terms
        check_var_pair(term)
        assert isinstance(expression_sdp, SingleDotProduct)
        assert term in self.terms, "the given term does not exist"
        term_coefficient = self.terms[term]
        del self.terms[term]


        terms_to_sub = expression_sdp.get_terms_deep_copy(include_constrained_terms = True, include_constant_term = True)
        assert terms_to_sub == expression_sdp.terms, "should be an exact copy"

        sdp_to_sub = SingleDotProduct(initial_terms = terms_to_sub)

        sdp_to_sub.mul_by_const_IN_PLACE(term_coefficient)

        # plugging in the sdp is the same as removing the term, and adding term_coefficient * sdp_to_sub to the sdp.

        self.add_sdp_IN_PLACE(sdp_to_sub)

    def plug_in_expression_of_var_IN_PLACE(self, var, expression_sdp):
        
        assert all([len(term) < 2 for term in expression_sdp.terms]) # cannot involve double terms
        assert len(expression_sdp.terms) <= 2 # only supporting assigning c or x1 + c to single vars 
        all_double_terms = [term for term in self.terms if len(term) == 2 and var in term]
        assert isinstance(expression_sdp, SingleDotProduct)
        made_sub = False
            
        if (var,) in self.terms:
            self.plug_in_expression_of_term_IN_PLACE((var,), expression_sdp)
            made_sub = True
        
        
        if len(all_double_terms) > 0:
            # the expression to plug in is m + n * xi -> xj (xj is what's in the current constraint)
            m = expression_sdp.terms[()]
            if len(expression_sdp.terms) != 1:
                assert len(expression_sdp.terms) == 2
                non_constant_term = [t for t in expression_sdp.terms if t != ()]
                assert len(non_constant_term) == 1 and len(non_constant_term[0]) == 1
                i = non_constant_term[0][0]
                n = expression_sdp.terms[non_constant_term[0]]
            else:
                i = None
                n = 0

            for cur_double_term in all_double_terms:
                if var not in cur_double_term:
                    continue
                    
                # 2 cases. A square, or a multiplication by a different var. 
                if len(set(cur_double_term)) == 1:
                    # a square
                    # cur_coef * (xj) ** 2 -> cur_coef * (m + n * xi) ** 2 -> cur_coef * m * m + cur_coef * n * n * xi * xi + 2 * cur_coef * m * n * xi
                    assert cur_double_term[0] == var
                    cur_coef = self.terms[cur_double_term]
                    self.terms.pop(cur_double_term)
                    made_sub = True
                    
                    if m != 0:
                        # cur_coef * m * m 
                        cur_const_part = mul_constants(mul_constants(m, m), cur_coef)
                        self.add_term_IN_PLACE((), cur_const_part)
                        
                        if n != 0:
                            # 2 * cur_coef * m * n * xi
                            cur_const_part = mul_constants(mul_constants(mul_constants(m, n), cur_coef), 2)
                            self.add_term_IN_PLACE((i,), cur_const_part)
                        
                    if i is not None and n != 0:
                        # cur_coef * n * n * xi * xi
                        cur_const_part = mul_constants(mul_constants(n, n), cur_coef)
                        self.add_term_IN_PLACE((i, i), cur_const_part)
                        
                else:
                    # cur_coef * xk * xj -> cur_coef * xk * (m + n * xi) -> cur_coef * m * xk + cur_coef * n * xi * xk 
                    k = [one_var for one_var in cur_double_term if one_var != var]
                    assert len(k) == 1
                    k = k[0]
                    cur_coef = self.terms[cur_double_term]
                    self.terms.pop(cur_double_term)
                    made_sub = True
                    
                    if m != 0:
                        # cur_coef * m * xk 
                        cur_const_part = mul_constants(cur_coef, m)
                        self.add_term_IN_PLACE((k,), cur_const_part)
                        
                    if n != 0:
                        # cur_coef * n * xi * xk 
                        cur_const_part = mul_constants(cur_coef, n)
                        if i < k:
                            self.add_term_IN_PLACE((i,k), cur_const_part)
                        else:
                            self.add_term_IN_PLACE((k,i), cur_const_part)
        
############################## Parsing the R1CS ##############################


    
def print_entire_r1cs(r1cs_circuit):
    for c in r1cs_circuit.parsed_constraints_whole_line:
        print(c)    

