pragma circom 2.0.3;
include "../circomlib-cff5ab6/comparators.circom";

// verify none zero using isZero template
template IsNoneZero() {
    signal input in;
    signal output out;
    signal inv;

    component iszero = IsZero();
    iszero.in <== in;
    inv <-- (iszero.out != 0) ? 0 : 1;

    out <== inv;
}