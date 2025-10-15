pragma circom 2.0.3;

// verify none zero without using isZero template
template IsNoneZero() {
    signal input in;
    signal output out;
    signal inv;

    inv <-- (in != 0) ? 1/in : 0;
    out <== in*inv;
}