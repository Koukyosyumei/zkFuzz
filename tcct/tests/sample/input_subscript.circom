pragma circom 2.0.0;

function get_elem(i) {
    var array[3] = [12, 13, 14];
    return array[i];
}

template Test() {
    signal input in;
    signal output out;

    out <-- get_elem(in);
}

component main = Test();