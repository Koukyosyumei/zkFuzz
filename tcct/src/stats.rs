use crate::symbolic_execution::ConstraintStatistics;


pub fn print_constraint_summary_statistics_pretty(constraint_stats: &ConstraintStatistics) {
    println!("Total_Constraints: {}", constraint_stats.total_constraints);
    println!("Constant_Counts: {}", constraint_stats.constant_counts);
    println!(
        "Conditional_Counts: {}",
        constraint_stats.conditional_counts
    );
    println!("Array_Counts: {}", constraint_stats.array_counts);
    println!("Tuple_Counts: {}", constraint_stats.tuple_counts);

    let avg_depth = if !constraint_stats.constraint_depths.is_empty() {
        constraint_stats.constraint_depths.iter().sum::<usize>() as f64
            / constraint_stats.constraint_depths.len() as f64
    } else {
        0.0
    };
    println!("Avg_Depth: {}", avg_depth);
    println!(
        "Max_Depth: {}",
        constraint_stats
            .constraint_depths
            .iter()
            .max()
            .unwrap_or(&0)
    );

    for op in &[
        "Mul", "Div", "Add", "Sub", "Pow", "IntDiv", "Mod", "ShL", "ShR", "LEq", "GEq", "Lt", "Gt",
        "Eq", "NEq", "BoolOr", "BoolAnd", "BitOr", "BitAnd", "BitXor",
    ] {
        println!(
            "Count_{}: {}",
            op,
            constraint_stats.operator_counts.get(*op).unwrap_or(&0)
        );
    }

    let var_counts: Vec<usize> = constraint_stats.variable_counts.values().cloned().collect();
    let var_avg = if !var_counts.is_empty() {
        var_counts.iter().sum::<usize>() as f64 / var_counts.len() as f64
    } else {
        0.0
    };
    println!("Variable_Avg_Count: {}", var_avg);
    println!(
        "Variable_Max_Count: {}",
        var_counts.iter().max().unwrap_or(&0)
    );

    let func_counts: Vec<usize> = constraint_stats
        .function_call_counts
        .values()
        .cloned()
        .collect();
    let func_avg = if !func_counts.is_empty() {
        func_counts.iter().sum::<usize>() as f64 / func_counts.len() as f64
    } else {
        0.0
    };
    println!("Function_Avg_Count: {}", func_avg);
    println!(
        "Function_Max_Count: {}",
        func_counts.iter().max().unwrap_or(&0)
    );
}

pub fn print_constraint_summary_statistics_csv(constraint_stats: &ConstraintStatistics) {
    let headers = vec![
        "Total_Constraints",
        "Constant_Counts",
        "Conditional_Counts",
        "Array_Counts",
        "Tuple_Counts",
        "Avg_Depth",
        "Max_Depth",
        "Count_Mul",
        "Count_Div",
        "Count_Add",
        "Count_Sub",
        "Count_Pow",
        "Count_IntDiv",
        "Count_Mod",
        "Count_ShiftL",
        "Count_ShiftR",
        "Count_LesserEq",
        "Count_GreaterEq",
        "Count_Lesser",
        "Count_Greater",
        "Count_Eq",
        "Count_NotEq",
        "Count_BoolOr",
        "Count_BoolAnd",
        "Count_BitOr",
        "Count_BitAnd",
        "Count_BitXor",
        "Variable_Avg_Count",
        "Variable_Max_Count",
        "Function_Avg_Count",
        "Function_Max_Count",
    ];
    println!("{}", headers.join(","));

    let mut values = Vec::new();
    values.push(constraint_stats.total_constraints.to_string());
    values.push(constraint_stats.constant_counts.to_string());
    values.push(constraint_stats.conditional_counts.to_string());
    values.push(constraint_stats.array_counts.to_string());
    values.push(constraint_stats.tuple_counts.to_string());

    let avg_depth = if !constraint_stats.constraint_depths.is_empty() {
        constraint_stats.constraint_depths.iter().sum::<usize>() as f64
            / constraint_stats.constraint_depths.len() as f64
    } else {
        0.0
    };
    values.push(format!("{:.2}", avg_depth));
    values.push(
        constraint_stats
            .constraint_depths
            .iter()
            .max()
            .unwrap_or(&0)
            .to_string(),
    );

    for op in &[
        "Mul", "Div", "Add", "Sub", "Pow", "IntDiv", "Mod", "ShL", "ShR", "LEq", "GEq", "Lt", "Gt",
        "Eq", "NEq", "BoolOr", "BoolAnd", "BitOr", "BitAnd", "BitXor",
    ] {
        values.push(
            constraint_stats
                .operator_counts
                .get(*op)
                .unwrap_or(&0)
                .to_string(),
        );
    }

    let var_counts: Vec<usize> = constraint_stats.variable_counts.values().cloned().collect();
    let var_avg = if !var_counts.is_empty() {
        var_counts.iter().sum::<usize>() as f64 / var_counts.len() as f64
    } else {
        0.0
    };
    values.push(format!("{:.2}", var_avg));
    values.push(var_counts.iter().max().unwrap_or(&0).to_string());

    let func_counts: Vec<usize> = constraint_stats
        .function_call_counts
        .values()
        .cloned()
        .collect();
    let func_avg = if !func_counts.is_empty() {
        func_counts.iter().sum::<usize>() as f64 / func_counts.len() as f64
    } else {
        0.0
    };
    values.push(format!("{:.2}", func_avg));
    values.push(func_counts.iter().max().unwrap_or(&0).to_string());

    println!("{}", values.join(","));
}
