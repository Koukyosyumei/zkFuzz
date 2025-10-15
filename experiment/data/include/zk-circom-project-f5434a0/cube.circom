pragma circom  2.0.0;

template cube() {

    // Declaration of signals.
    signal input a;
    signal input expectedCube;
    signal calc;
    signal output cube;
    calc <== a*a;
    cube <== a*calc;
    expectedCube === cube;
}

// component main {public [expectedCube]} = cube();
