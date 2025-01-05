pragma circom 2.1.0;
// https://github.com/zkFHE/circomlib-fhe/blob/96536373df183bc1849b96b0dc82f1a0f7a8abfd/circuits/add.circom#L8

// assumes 0 <= a <= 2^252
function log2(a) {
    return logb(a, 2);
}

// assumes 0 <= a <= b^k where k is the largest integer such that b^k < p/2,
// where p is circom's prime
function logb(a, b) {
    if (a==0) {
        return 0;
    }
    var n = 1;
    var r = 0;
    while (n<a) {
        r++;
        n *= b;
    }
    return r;
}

/*
    Performs bit decomposition of n bits and returns the most significant bit.

    Based on:
    https://github.com/iden3/circomlib/blob/cff5ab6288b55ef23602221694a6a38a0239dcc0/circuits/bitify.circom#L25
*/
template GetMostSignificantBit(n) {
    signal input in;
    signal {binary} bits[n];
    signal output {binary} out;

    var lc1=0;

    var e2=1;
    for (var i = 0; i<n; i++) {
        bits[i] <-- (in >> i) & 1;
        bits[i] * (bits[i] - 1) === 0;
        lc1 += bits[i] * e2;
        e2 = e2+e2;
    }

    lc1 === in;
    out <== bits[n-1];
}

// Enforces in < ct
template parallel LtConstant(ct) {
    signal input in;
    var n = log2(ct);

    component bit = GetMostSignificantBit(n+1);
    bit.in <== in + (1<<n) - ct;
    1-bit.out === 1;
}

template parallel FastAddMod(q) {
    signal input in[2]; // both inputs need to be in Z/qZ
    signal sum <== in[0] + in[1];
    signal quotient <-- sum \ q; // quotient is either 0 or 1
    signal output out <-- sum % q;

    LtConstant(q)(out); // Check that remainder is less than q
    quotient * q + out === sum; // Check that quotient and remainder are correct
    quotient * (quotient - 1) === 0; // Check that quotient is in {0, 1}
}

component main = FastAddMod(3);