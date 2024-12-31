pragma circom 2.0.3;
// https://github.com/tea2x/zk-template/blob/0a813fd8701a6bc7d7debb754880769398699f62/circuits/none-zero/circuit.circom

// verify none zero without using isZero template
template IsNoneZero() {
    signal input in;
    signal output out;
    signal inv;

    inv <-- (in != 0) ? 1/in : 0;
    out <== in*inv;
}

component main = IsNoneZero();