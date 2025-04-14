pragma circom 2.0.0;
// https://github.com/forestlv/zksnark-sudoku-master/blob/main/packages/circuit/circuits/utils.circom

include "../circomlib-cff5ab6/comparators.circom";

template AND() {
    signal input a;
    signal input b;
    signal output out;

    out <== a*b;
}

template MultiAND(n) {
    signal input in[n];
    signal output out;
    component and1;
    component and2;
    component ands[2];
    if (n==1) {
        out <== in[0];
    } else if (n==2) {
        and1 = AND();
        and1.a <== in[0];
        and1.b <== in[1];
        out <== and1.out;
    } else {
        and2 = AND();
        var n1 = n\2;
        var n2 = n-n\2;
        ands[0] = MultiAND(n1);
        ands[1] = MultiAND(n2);
        var i;
        for (i=0; i<n1; i++) ands[0].in[i] <== in[i];
        for (i=0; i<n2; i++) ands[1].in[i] <== in[n1+i];
        and2.a <== ands[0].out;
        and2.b <== ands[1].out;
        out <== and2.out;
    }
}

template IsValidSolutionOfPuzzle() {
    signal input solution[4];
    signal input puzzle[4];
    signal output result;

    component allNumbersAreEqualCheck = MultiAND(4);
    component equalCheck[4];
    for (var i = 0; i < 4; i ++) {
        equalCheck[i] = IsEqual();
        equalCheck[i].in[0] <== solution[i];
        equalCheck[i].in[1] <== puzzle[i];
        allNumbersAreEqualCheck.in[i] <-- (puzzle[i] == 0) ? 1 : equalCheck[i].out;
    }

    result <== allNumbersAreEqualCheck.out;
}