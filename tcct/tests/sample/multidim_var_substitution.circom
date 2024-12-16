pragma circom 2.0.0;

function multidim_var_access()
{
    var lut[2][2];

    lut[0] = [1, 2];
    lut[1] = [3, 4];

    return lut[1][1];
}

template Main() {
    signal input in;
    signal output out;
    out <== in + multidim_var_access();
}

component main = Main();