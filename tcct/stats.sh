#!/bin/bash

TARGET_DIR="../../Picus/benchmarks/"
BASE_COMMAND="./target/release/tcct"

counter=1
for circom_file in "$TARGET_DIR"/*/*.circom; do
    echo "Processing: $circom_file"
    output_file="output/${counter}.csv"
    $BASE_COMMAND "$circom_file" --show_stats_of_ast > "$output_file"
    echo "Output saved to: $output_file"
    ((counter++))
done