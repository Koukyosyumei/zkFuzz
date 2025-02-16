pragma circom 2.0.0;

template Main(n) {
    signal input x[n];
    signal output y;

    var sum = 0;
    for (var i = n - 1; i >= 0; i--) {
        sum += x[i];
    }
    y <-- sum;
}

component main = Main(5);