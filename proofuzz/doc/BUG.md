# Typical Bugs within ZKP Circuits

The verification process must reject proofs generated from different statements.

The proof is made up of the constraints and witness generated from the circuit, and the verifier can access to the constraints and the output (and the public input).

The programmer of Circom sometimes has to separate the variable assignment and constraint, since Circom only supports quadratic equations.
In Circom, there are three operators:

- `<--`: Computation (only assign, not generate constraint)  
- `===`: Constraint
- `<==`: Computation & Constraint 

- E.x. Statement: `z = x / y`

Programmers have to write proper constraints for the non-quadratic computations

```c
z <-- x / y; // assign x/y to z
x === z * y; // constraint
```

## Under-Constrained Circuits

### [MiMC Hash](https://github.com/iden3/circomlib/pull/22/files)

MiMC is a hash function commonly used in zk-SNARK applications.

Since `outs[0] = S[nInputs - 1].xL_out` doesn't enforce any constraints, attackers can replace `outs[0]`  with an arbitrary value 

```c
template MiMCSponge(nInputs, nRounds, nOutputs) {
  signal input ins[nInputs];
  signal input k;
  signal output outs[nOutputs];
  // S = R||C
  component S[nInputs + nOutputs - 1];
  for (var i = 0; i < nInputs; i++) {
    S[i] = MiMCFeistel(nRounds);
    S[i].k <== k;
    if (i == 0) {
      S[i].xL_in <== ins[0];
      S[i].xR_in <== 0;
    } else {
      S[i].xL_in <== S[i-1].xL_out + ins[i];
      S[i].xR_in <== S[i-1].xR_out;
    }
  }
  outs[0] = S[nInputs - 1].xL_out; // This line is vulnerable! It should be outs[0] <== S[nInputs - 1].xL_out;
  for (var i = 0; i < nOutputs - 1; i++) {
    S[nInputs + i] = MiMCFeistel(nRounds);
    S[nInputs + i].k <== k;
    S[nInputs + i].xL_in <== S[nInputs + i - 1].xL_out;
    S[nInputs + i].xR_in <== S[nInputs + i - 1].xR_out;
    outs[i + 1] <== S[nInputs + i].xL_out;
  }
}
```

Suppose an attacker has data x s.t. `MiMC(x) = 110011001` Then, the attacker can create a proof claiming “I have a x such that `MiMC(x) = 010011001`”

## Over-Constrained Circuits