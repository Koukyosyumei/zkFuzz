pragma circom 2.0.0;

template Test()
{
    signal input in[4][8];
    signal output out;

    var s[4][8];
    var i,j,k;

    for(i=0; i<4; i++)
    {
        for(j=0; j<8; j++) {
            s[i][j] = in[i][j];
        }
    }

    var t[8] = s[1];
    var sum = 0;
    for(k=0; k<8; k++) {
        sum += t[k];
    }
    out <== sum;
}

component main = Test();
