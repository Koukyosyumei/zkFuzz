pragma circom 2.1.6;

// The solution is to compute the average using regular programming, 
// then constrain the output to be correct.

function invert(x) {
    return 1/x;
}

template IsZero() {
    signal input in;
    signal output z;

    signal inv;

    inv <-- in!=0 ? 1/in : 0;

    z <== -in*inv +1;
    in*z === 0;
}

template IsEqual() {
    signal input in[2];
    signal output y;

    component isz = IsZero();

    in[1] - in[0] ==> isz.in;

    isz.z ==> y;
}

template Average(n) {

    signal input in[n];
    signal denominator_inv;
    signal output out;

    var sum;
    for (var i = 0; i < n; i++) {
        sum += in[i];
    }

    denominator_inv <-- invert(n);

    component eq = IsEqual();
    eq.in[0] <== 1;
    eq.in[1] <== denominator_inv * n;

    out <== sum * denominator_inv;

}

component main  = Average(5);

/*
╔══════════════════════════════════════════════════════════════╗
║🚨 Counter Example:                                           ║
║    🔥 UnderConstrained (Non-Deterministic) 🔥
║    🔍 Assignment Details:
║           ➡️ main.in[0] = 2
║           ➡️ main.denominator_inv = 1
║           ➡️ main.in[1] = 5
║           ➡️ main.out = 7
║           ➡️ main.eq.isz.in = 1
║           ➡️ main.eq.isz.z = 0
║           ➡️ main.eq.y = 0
║           ➡️ main.eq.isz.inv = 1
║           ➡️ main.eq.in[0] = 1
║           ➡️ main.eq.in[1] = 2
╚══════════════════════════════════════════════════════════════╝
*/