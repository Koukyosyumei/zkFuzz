## Tips

### Test

```bash
cargo test
```

### Performance Stats

**Example command:**

```bash
perf stat -- ./target/release/tcct ./tests/sample/iszero_vuln.circom --search_mode="ga"
```

**Example output:**

```bash
 Performance counter stats for './target/release/tcct ./tests/sample/iszero_vuln.circom --search_mode=ga':

           2443.80 msec task-clock:u              #    0.995 CPUs utilized
                 0      context-switches:u        #    0.000 /sec
                 0      cpu-migrations:u          #    0.000 /sec
               691      page-faults:u             #  282.756 /sec
        3970688350      cycles:u                  #    1.625 GHz
       14853648990      instructions:u            #    3.74  insn per cycle
   <not supported>      branches:u
          16513206      branch-misses:u

       2.456600004 seconds time elapsed

       2.434860000 seconds user
       0.000000000 seconds sys
```

### Profiling Information

**Example command:**

```s
perf record -g -- ./target/release/tcct ./tests/sample/iszero_vuln.circom --search_mode="ga"
perf report
```

**Example output:**

```bash
# To display the perf.data header info, please use --header/--header-only options.
#
#
# Total Lost Samples: 0
#
# Samples: 4K of event 'cycles:u'
# Event count (approx.): 3608619584
#
# Children      Self  Command  Shared Object          Symbol                                                                                                    >
# ........  ........  .......  .....................  ..........................................................................................................>
#
    10.56%    10.56%  tcct     tcct                   [.] tcct::solver::utils::evaluate_symbolic_value
            |
             --9.89%--tcct::solver::utils::evaluate_symbolic_value

     9.67%     9.63%  tcct     tcct                   [.] <num_bigint_dig::bigint::BigInt as num_integer::Integer>::div_rem
            |
            |--8.35%--<num_bigint_dig::bigint::BigInt as num_integer::Integer>::div_rem
            |
             --0.57%--0
                       <num_bigint_dig::bigint::BigInt as num_integer::Integer>::div_rem

     8.19%     8.19%  tcct     tcct                   [.] num_bigint_dig::bigint::BigInt::from_biguint
            |
       .
       .
       .
```

### Coverage

**Example command:**

```bash
cargo tarpaulin
```
