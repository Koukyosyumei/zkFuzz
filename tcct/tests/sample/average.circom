pragma circom 2.1.6;

include "../../../benchmark/circomlib/circuits/comparators.circom";

// The solution is to compute the average using regular programming, 
// then constrain the output to be correct.

function invert(x) {
    return 1/x;
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

component main  = Average(2);