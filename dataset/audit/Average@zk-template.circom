pragma circom 2.1.6;
// https://github.com/tea2x/zk-template/blob/0a813fd8701a6bc7d7debb754880769398699f62/circuits/average/circuit.circom

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
