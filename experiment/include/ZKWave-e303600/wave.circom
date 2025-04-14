pragma circom 2.1.4;

template VerifyPolynomial(degree) {
    signal input coeffs[degree + 1];
    signal input x;                
    signal input y;                

    signal computed_y;              
    signal power_results[degree + 1];
    signal terms[degree + 1];        

    signal debug_powers[degree + 1]; 
    signal debug_terms[degree + 1];  
    signal debug_computed_y;         
    signal debug_provided_y;         

    power_results[0] <== 1; 
    debug_powers[0] <== power_results[0];
    for (var i = 1; i <= degree; i++) {
        power_results[i] <== power_results[i - 1] * x; 
        debug_powers[i] <== power_results[i];
    }

    for (var i = 0; i <= degree; i++) {
        terms[i] <== coeffs[i] * power_results[i];
        debug_terms[i] <== terms[i];
    }

    var acc = 0;
    for (var i = 0; i <= degree; i++) {
        acc += terms[i];
    }
    computed_y <== acc;

    debug_computed_y <== computed_y;
    debug_provided_y <== y;

    computed_y === y;
}

// component main = VerifyPolynomial(5);