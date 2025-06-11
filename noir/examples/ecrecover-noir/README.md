# Under-Constrained Vulnerability in `to_eth_address` (ecrecover-noir)

This repository demonstrates a vulnerability in the `to_eth_address` function of the `ecrecover-noir` project. The issue results in under-constrained behavior, meaning that the circuit may accept incorrect witnesses as valid proofs.

🔍 Original bug report: https://gist.github.com/olehmisar/4cfe6128eaac2bfbe1fa8eb46f0116d6.

To reproduce the vulnerability using `zkfuzz-noir`, run the following:

```bash
nargo compile --skip-brillig-constraints-check

zkfuzz-noir --artifact-path ./target/ecrecover.json --prover-file ./Prover.toml 2>/dev/null
🧬 Generation: 52/1000  Under-Constrained
      Original Return Value: Field(1390849295786071768276380950238675083608645509734)
       Mutated Return Value: Field(104207971572578014366191844895147407837455278660)
```