pragma circom 2.1.8;

include "../../include/circomlib/circuits/comparators.circom";

template Main() {
    signal input x;
    component lt1 = LessThan(10);

    lt1.in[0] <== x;
    lt1.in[1] <== 512;
    lt1.out === 1;
}

component main = Main();

"""
```
╔══════════════════════════════════════════════════════════════╗
║🚨 Counter Example:                                           ║
║    🧟 UnderConstrained (Unexpected-Trace) 🧟
║           Violated Condition: (BoolOr (BoolAnd (Eq 1 main.lt1.out) (Lt main.lt1.in[0] main.lt1.in[1])) (BoolAnd (Eq 0 main.lt1.out) (GEq main.lt1.in[0] main.lt1.in[1])))
║    🔍 Assignment Details:
║           ➡️ main.x = 21888242871839275222246405745257275088548364400416034343698204186575808495588
║           ➡️ main.lt1.n2b.out[6] = 1
║           ➡️ main.lt1.n2b.out[1] = 1
║           ➡️ main.lt1.n2b.out[0] = 1
║           ➡️ main.lt1.n2b.out[5] = 1
║           ➡️ main.lt1.n2b.out[10] = 0
║           ➡️ main.lt1.n2b.out[3] = 0
║           ➡️ main.lt1.n2b.out[7] = 1
║           ➡️ main.lt1.n2b.out[2] = 0
║           ➡️ main.lt1.n2b.out[8] = 1
║           ➡️ main.lt1.n2b.out[4] = 0
║           ➡️ main.lt1.out = 1
║           ➡️ main.lt1.in[1] = 512
║           ➡️ main.lt1.in[0] = 21888242871839275222246405745257275088548364400416034343698204186575808495588
║           ➡️ main.lt1.n2b.in = 483
║           ➡️ main.lt1.n2b.out[9] = 0
╚══════════════════════════════════════════════════════════════╝
```
"""