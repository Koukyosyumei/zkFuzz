# Design

```c
// Initialize population and symbolic trace
input_population <- Input_Generation()
symbolic_trace_population <- Trace_Mutation(original_symbolic_trace)
symbolic_trace_population.append(original_trace)
symbolic_trace_fitness = [-inf, -inf, ...]

// Main loop for trials
for trial in 1..Max_Trial:
    // Update input population at specified intervals
    if trial % Input_Update_Interval == 0:
        input_population <- Input_Update(input_population)
    
    // Perform crossover and mutation on symbolic traces
    symbolic_trace_population <- Trace_Evolution(symbolic_trace_population, symbolic_trace_fitness)

    // Initialize maximum score
    let max_score = -inf

    // Evaluate fitness of each symbolic trace against input population
    for i, symbolic_trace in enumerate(symbolic_trace_population):
        for input in input_population:
            score, flag = evaluate_trace_fitness(original_symbolic_trace, symbolic_trace, side_condition, input)
            
            // If score is zero, return flag indicating a condition met
            if score == 0:
                return flag
            
            // Update maximum score
            max_score = max(score, max_score)
        symbolic_trace_fitness[i] = max_score

// Function to evaluate trace fitness
Func evaluate_trace_fitness(original_symbolic_trace, symbolic_trace, side_condition, input_population):
    max_score = -inf
    
    // Evaluate each input against the symbolic trace
    for input in input_population:
        is_success, output = emulate_trace(symbolic_trace, input)
        error_of_side_condition = error_func(side_condition, input)
        score = -error_of_side_condition
        
        if score == 0: // Check if side-conditions are met
            if is_success:
                // Check original program's success and output consistency
                original_is_success, original_output = emulate_trace(original_symbolic_trace, input)
                
                if !original_is_success:
                    max_score = 0 // Original program crashes while side-conditions are met
                    return max_score, UnderConstrained::UnexpectedInput
                
                if original_output != output:
                    max_score = 0 // Outputs differ despite meeting conditions
                    return max_score, UnderConstrained::NonDeterministic            
                else:
                    score = -inf
            else:
                if symbolic_trace == original_trace:
                    max_score = 0 // Original program crashes while side-conditions are met
                    return max_score, UnderConstrained::UnexpectedInput
        else:
            if symbolic_trace == original_symbolic_trace && is_success:
                max_score = 0 // Trace-conditions met but side-condition violated
                return max_score, OverConstrained 
        
        max_score = max(score, max_score) // Update maximum score if current score is higher

    return max_score, None  
```
