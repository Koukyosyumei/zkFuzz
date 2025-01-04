pragma circom 2.0.0;

include "../include/circom-pairing/circuits/fp2.circom";
include "../include/circom-pairing/circuits/curve.circom";

template VulnEllipticCurveAddUnequal(n, k, p) { 
    signal input a[2][k];
    signal input b[2][k];

    signal output out[2][k];

    var LOGK = log_ceil(k);
    var LOGK3 = log_ceil( (3*k*k)*(2*k-1) + 1 ); 
    assert(4*n + LOGK3 < 251);

    // precompute lambda and x_3 and then y_3
    var dy[50] = long_sub_mod(n, k, b[1], a[1], p);
    var dx[50] = long_sub_mod(n, k, b[0], a[0], p); 
    var dx_inv[50] = mod_inv(n, k, dx, p);
    var lambda[50] = prod_mod(n, k, dy, dx_inv, p);
    var lambda_sq[50] = prod_mod(n, k, lambda, lambda, p);
    // out[0] = x_3 = lamb^2 - a[0] - b[0] % p
    // out[1] = y_3 = lamb * (a[0] - x_3) - a[1] % p
    var x3[50] = long_sub_mod(n, k, long_sub_mod(n, k, lambda_sq, a[0], p), b[0], p);
    var y3[50] = long_sub_mod(n, k, prod_mod(n, k, lambda, long_sub_mod(n, k, a[0], x3, p), p), a[1], p);

    for(var i = 0; i < k; i++){
        out[0][i] <-- x3[i];
        out[1][i] <-- y3[i];
    }
    
    // constrain x_3 by CUBIC (x_1 + x_2 + x_3) * (x_2 - x_1)^2 - (y_2 - y_1)^2 = 0 mod p
    
    component dx_sq = BigMultShortLong(n, k, 2*n+LOGK+2); // 2k-1 registers abs val < k*2^{2n} 
    component dy_sq = BigMultShortLong(n, k, 2*n+LOGK+2); // 2k-1 registers < k*2^{2n}
    for(var i = 0; i < k; i++){
        dx_sq.a[i] <== b[0][i] - a[0][i];
        dx_sq.b[i] <== b[0][i] - a[0][i];

        dy_sq.a[i] <== b[1][i] - a[1][i];
        dy_sq.b[i] <== b[1][i] - a[1][i];
    } 

    // x_1 + x_2 + x_3 has registers in [0, 3*2^n) 
    component cubic = BigMultShortLongUnequal(n, k, 2*k-1, 3*n+4+2*LOGK); // 3k-2 registers < 3 * k^2 * 2^{3n} ) 
    for(var i=0; i<k; i++)
        cubic.a[i] <== a[0][i] + b[0][i] + out[0][i]; 
    for(var i=0; i<2*k-1; i++){
        cubic.b[i] <== dx_sq.out[i];
    }

    component cubic_red = PrimeReduce(n, k, 2*k-2, p, 4*n + LOGK3);
    for(var i=0; i<2*k-1; i++)
        cubic_red.in[i] <== cubic.out[i] - dy_sq.out[i]; // registers abs val < 3k^2*2^{3n} + k*2^{2n} < (3k^2+1)2^{3n}
    for(var i=2*k-1; i<3*k-2; i++)
        cubic_red.in[i] <== cubic.out[i]; 
    // cubic_red has k registers < (3k^2+1)(2k-1) * 2^{4n}
    
    component cubic_mod = SignedCheckCarryModToZero(n, k, 4*n + LOGK3, p);
    for(var i=0; i<k; i++)
        cubic_mod.in[i] <== cubic_red.out[i]; 
    // END OF CONSTRAINING x3
    
    // constrain y_3 by (y_1 + y_3) * (x_2 - x_1) = (y_2 - y_1)*(x_1 - x_3) mod p
    component y_constraint = PointOnLine(n, k, p); // 2k-1 registers in [0, k*2^{2n+1})
    for(var i = 0; i < k; i++)for(var j=0; j<2; j++){
        y_constraint.in[0][j][i] <== a[j][i];
        y_constraint.in[1][j][i] <== b[j][i];
        y_constraint.in[2][j][i] <== out[j][i];
    }
    // END OF CONSTRAINING y3

    // check if out[][] has registers in [0, 2^n) 
    // component range_check = RangeCheck2D(n, k);
    // for(var j=0; j<2; j++)for(var i=0; i<k; i++)
    //    range_check.in[j][i] <== out[j][i];
}

component main {public [a, b]} = VulnEllipticCurveAddUnequal(52, 5, [4503595332402223, 4503599627370495, 4503599627370495, 4503599627370495, 281474976710655]);