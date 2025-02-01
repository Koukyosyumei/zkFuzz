pragma circom 2.0.0;

template Test()
{
    signal input in;
    signal output out;

    var s;
    var t;

    s = in;
    t = s + 1;
    out <== t;
}

component main = Test();
