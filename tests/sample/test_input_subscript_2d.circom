pragma circom 2.0.0;

function get_elem(i, j) {
    var array[2][3];
    array[0] = [5, 6, 7];
    array[1] = [12, 13, 14];
    return array[i][j];
}

template Test() {
    signal input x;
    signal input y;
    signal output y1;
    signal output y2;

    y1 <-- get_elem(x, y);
    y2 <== x + y1;
}

component main = Test();