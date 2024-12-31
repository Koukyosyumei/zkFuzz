pragma circom 2.0.0;

template IsZero() {
    signal input in;    // Input signal to check if it's zero or non-zero.
    signal output out;  // Output signal: 1 if `in == 0`, 0 if `in != 0`.
    signal inv;         // Inverse of the input when `in != 0`, or 0 when `in == 0`.
    
    // Compute the inverse: if `in` is non-zero, `inv` is set to `1/in`, otherwise it's 0.
    inv <-- in!=0 ? 1/in : 0;

    // Constraint 1: Ensures that if `in != 0`, `out` is 0. If `in == 0`, `out` is 1.
    out <== -in*inv +1;

    // Constraint 2: Ensures that `in * out == 0`, forcing the output to 1 only when `in == 0`.
    in*out === 0;
}

template Main() {
    signal input x;
    signal input y;
    signal output z;
    component c = IsZero();
    c.in <== y != 0 ? x + 1 : x + 2;
    z <== c.out;
}


component main = Main();