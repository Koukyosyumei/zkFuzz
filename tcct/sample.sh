#!/bin/bash

TARGET_DIR="../../Picus/benchmarks/circomlib-cff5ab6"
BASE_COMMAND="./target/release/tcct"

TIME_LIMIT=10
for circom_file in "$TARGET_DIR"/*.circom; do
    echo "Processing: $circom_file"
    timeout $TIME_LIMIT $BASE_COMMAND "$circom_file" --search_mode="quick"
    if [ $? -eq 124 ]; then
        echo "Timeout reached for file: $circom_file"
    fi
done