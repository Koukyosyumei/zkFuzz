#!/bin/bash

output_file="circom_template_counts.csv"

echo "Filename,TemplateCount,SignalAssignArrowCount,VarAssignCount" > "$output_file"

find . -type f -name "*.circom" | while read -r file; do
    template_count=$(grep -o "template" "$file" | wc -l)
    signal_assign_arrow_count=$(grep -o "<--" "$file" | wc -l)
    var_assign_count=$(grep -E "^[^c]* = " "$file" | grep -v "component" | wc -l)
    echo "$file,$template_count,$signal_assign_arrow_count,$var_assign_count" >> "$output_file"
done

echo "Output: $output_file"
