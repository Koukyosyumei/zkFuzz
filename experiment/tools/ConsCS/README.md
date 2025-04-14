
# Structure of the package:

## Folders:
- ./benchmarks: contains the benchmarks along with the ".r1cs" files compiled by the Circom compiler.
- ./logs: includes the logs obtained by running both the existing tools and our proposed framework. Our tables and figures are made from these logs.
- ./scripts: contains the scripts we used to obtain results from both our proposed framework and the existing tools, as well as the codes for obtaining the tables, figures, and statistics reported in the paper.
 

## Files:
- ./analyze_circuit.py: the main entry point of our proposed framework for circuit analysis.
- ./data_structures.py: holds helper data structures. 
- ./utils.py: utility functions.
- ./misc.py: miscellaneous operations such as file I/O and arithmetics.
- ./r1cs_utils.py: codes for parsing the ".r1cs" files, translated from the Racket code originally developed by the Picus project (https://github.com/chyanju/Picus), under the MIT license.


# License
The MIT License is adopted for this proposed framework, to facilitate reuse and adaptation by other researchers with minimal restrictions.

# Usage
To run the codes, use the following command:

```bash
python3 analyze_circuit.py <target_file_path> <log_path> <log_path_with_contributions> <flags> <max_depth>
```

The parameters are specified as follows:
- target_file_path: path to the ".r1cs" file to be analyzed.
- log_path: path where the logs should be appended to.
- log_path_with_contribitions: path where the logs, including detailed contribution counts, should be appended to.
- flags: 3-bit flag to enable or disable different stages (1 to enable, 0 to disable). The first bit is for stage 4, second bit is for stage 5, third bit is for stage 6. For example, 111 would run the full framework.
- max_depth: maximum depth for the BPG phase.

An example run is as follows:
```python
python3 analyze_circuit.py ./benchmarks/utils/AND@gates.r1cs ./logs/test_log.log ./logs/test_log_with_contributions.log 111 4
```

Our codes have been tested on sympy 1.11.1, z3-solver 4.12.1.0, and python 3.9.12.

