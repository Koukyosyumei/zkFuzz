pragma circom 2.0.0;

template Recursive(len) {
    signal input inputs[len];
    signal output out;
    signal tmp;

    var batch = 3;
    if (len < batch) {
        out <== inputs[0];
    } else {
        var t[len - batch + 1];
        t[0] = 12;
        for(var i = batch; i < len; i++) {
            t[i - batch + 1] = inputs[i];
        }
        out <== Recursive(len - batch + 1)(t);
    }
}

component main = Recursive(7);