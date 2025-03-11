#!/bin/bash

# Initialize
output_file="out.txt"
> "$output_file"

find ../../Picus/benchmarks/ -type f -name "*.circom" | while read -r circom_file; do
  echo "Processing file: $circom_file"
  temp_output=$(mktemp)
  ./target/debug/zkfuzz "$circom_file" > "$temp_output"

  if tail -n 1 "$temp_output" | grep -q "Everything went okay"; then
    echo "Adding output for $circom_file to $output_file"
    head -n -1 "$temp_output" >> "$output_file"
  else
    cat $temp_output
    echo "Skipping $circom_file (did not finish successfully)"
  fi
done

echo "Output written to $output_file"