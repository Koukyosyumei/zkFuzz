pragma circom 2.0.0;

template Recursive(len) {
    signal input inputs[len];
    signal output out;
    signal tmp;

    var batch = 3;
    if (len < batch) {
        var sum = 0;
        for (var j = 0; j < len; j++) {
            sum += inputs[j];
        }
        out <== sum;
    } else {
        var t[len - batch + 1];
        t[0] = 12;
        for(var i = batch; i < len; i++) {
            t[i - batch + 1] = inputs[i];
        }
        out <== Recursive(len - batch + 1)(t);
    }
}

component main = Recursive(8);