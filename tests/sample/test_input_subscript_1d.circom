pragma circom 2.0.0;

function get_elem(i) {
    var array[3] = [12, 13, 14];
    return array[i];
}

template Test() {
    signal input x;
    signal output y1;
    signal output y2;

    y1 <-- get_elem(x);
    y2 <== x + y1;
}

component main = Test();