pragma circom 2.0.0;

template Main() {
    signal input in;
    signal output out;

    var X[2][2] = [[1,2],[3,4]];
    
    out <-- (in + X[0][0] + X[0][1]) / X[1][0] + X[1][1];
    out * (X[1][0] + X[1][1]) === in + X[0][0] + X[0][1];
}

component main = Main();
