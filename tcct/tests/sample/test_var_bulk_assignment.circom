pragma circom 2.0.0;

template Test()
{
    signal input in[2][2];
    signal output out;

    var s[2][2];
    var i,j,k;

    for(i=0; i<2; i++)
    {
        for(j=0; j<2; j++) {
            s[i][j] = in[i][j];
        }
    }

    var t[2] = s[1];
    var sum = 0;
    for(k=0; k<2; k++) {
        sum += t[k];
    }
    out <== sum;
}

component main = Test();
