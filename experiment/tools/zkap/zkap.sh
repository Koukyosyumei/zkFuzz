#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Ensure the script is run with an argument
if [ $# -ne 1 ]; then
    echo "Usage: $0 <input_circom_file>"
    exit 1
fi

# Input and output file variables
input_circom_file="$1"
base_file_name=$(basename "$input_circom_file" .circom)
output_ll_file="${base_file_name}.ll"

# Process the Circom file with circom2llvm
circom2llvm --input "${input_circom_file}" --instantiation > /dev/null

# Apply ZKAPPass optimizations using opt
opt -enable-new-pm=0 -load /usr/local/lib/libZKAPPass.so --All -S "${output_ll_file}" -o /dev/null

# Cleanup
rm "${output_ll_file}"
