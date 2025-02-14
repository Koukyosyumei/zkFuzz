pragma circom 2.0.0;

function get_dim() {
    return 2;
}

template Callee(){
    signal output arr[get_dim()];
    for(var i = 0; i < get_dim(); i++)
        arr[i] <== i;
}

template Caller() {
    signal input x[2];
    signal input y[2];
    signal output z;
    z <== x[0] + x[1] + y[0] + y[1];
}

template Main() {
    signal input a;
    signal input b;
    signal output c;

    c <== Caller()([a, b], Callee()());
}

component main = Main();