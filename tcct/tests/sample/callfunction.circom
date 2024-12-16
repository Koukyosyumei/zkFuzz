pragma circom 2.0.0;

function f(x)
{
    if (x == 0) {
        return 0;
    } else {
        return f(x - 1) + x;
    }
}

template Main() {
    signal input in;
    signal output out;
    out <== in + f(5);
}

component main = Main();