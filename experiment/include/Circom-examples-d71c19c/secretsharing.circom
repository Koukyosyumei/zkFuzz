pragma circom 2.0.0;

include "../circomlib-cff5ab6/comparators.circom";

template ModP() {
    signal input num;
    signal input p;
    signal output out;

    /* 
    error: operator '<==' not allowed

    var _num_mod_p = num;
    while (_num_mod_p >= p) { // TODO add constrains
        _num_mod_p <== _num_mod_p - p; 
    }
    */

    // var _num_mod_p = num;
    // while (_num_mod_p >= p) { // TODO add constrains
    //     _num_mod_p = _num_mod_p - p; 
    // }

    // num = p * quotient + remainder
    signal remainder <-- num % p; // TODO "mod" needs more constraints
    signal quotient <-- num \ p ;

    num === p * quotient + remainder;
    out <== remainder;
}

template InField(field_bits) {
    signal input value;
    signal input p;

    // must 0 <= value
    component greater_eq_than = GreaterEqThan(field_bits);
    greater_eq_than.in[0] <== value;
    greater_eq_than.in[1] <== 0; 
    greater_eq_than.out === 1;

    // must value < p
    component less_than = LessThan(field_bits);
    less_than.in[0] <== value;
    less_than.in[1] <== p; 
    less_than.out === 1;

}



template SecretSharingForParty1(field_bits) {
    signal input s;
    signal input p;
    signal input k;

    signal input s1;
    signal input s2;
    signal input s3;
    
    // k is in the field (0 <= k < p)
    // s is in the field (0 <= s < p)
    // s1 is in the field (0 <= s1 < p)
    // s2 is in the field (0 <= s2 < p)
    // s3 is in the field (0 <= s3 < p)

    // secret sharing correctness
    // ->   every 2 share combination can open the secret
    // ->   s3 + s1 - 2*s2 = 0

    // s1 == s + k mod p
    // s2 == s + 2*k mod p
    // s3 == s + 3*k mod p


    //  k is in the field (0 <= k < p)
    component in_field1 = InField(field_bits);
    in_field1.value <== k;
    in_field1.p <== p;

    //  s is in the field (0 <= s < p)
    component in_field2 = InField(field_bits);
    in_field2.value <== s;
    in_field2.p <== p;


    //  s1 is in the field (0 <= s1 < p)
    component in_field3 = InField(field_bits);
    in_field3.value <== s1;
    in_field3.p <== p;

    //  s2 is in the field (0 <= s2 < p)
    component in_field4 = InField(field_bits);
    in_field4.value <== s2;
    in_field4.p <== p;

    //  s3 is in the field (0 <= s3 < p)
    component in_field5 = InField(field_bits);
    in_field5.value <== s3;
    in_field5.p <== p;


    
    // Constructing share s1
    // s1 == s + k mod p
    component s1_mod_p = ModP(); // num % p
    s1_mod_p.num <== s + k;
    s1_mod_p.p <== p;
    signal s1_calculated_in_circuit <== s1_mod_p.out;
    s1 === s1_calculated_in_circuit;

    // Constructing share s2
    // s2 == s + 2*k mod p
    component s2_mod_p = ModP(); // num % p
    s2_mod_p.num <== s + 2*k;
    s2_mod_p.p <== p;
    signal s2_calculated_in_circuit <== s2_mod_p.out;
    s2 === s2_calculated_in_circuit;

    // Constructing share s3
    // s3 == s + 3*k mod p
    component s3_mod_p = ModP(); // num % p
    s3_mod_p.num <== s + 3*k;
    s3_mod_p.p <== p;
    signal s3_calculated_in_circuit <== s3_mod_p.out;
    s3 === s3_calculated_in_circuit;


    // secret sharing correctness: s3 + s1 - 2*s2 = 0
    signal weighted_sum <== s3 + s1 - 2*s2;
    component s_mod_p = ModP();
    s_mod_p.num <== weighted_sum;
    s_mod_p.p <== p;
    signal zero <== s_mod_p.out;
    zero === 0;

    signal weighted_sum2 <== s3_calculated_in_circuit + s1_calculated_in_circuit - 2*s2_calculated_in_circuit;
    component s_mod_p2 = ModP();
    s_mod_p2.num <== weighted_sum2;
    s_mod_p2.p <== p;
    signal zero2 <== s_mod_p2.out;
    zero2 === 0;
}

// component main {public [p, s2, s3]} = SecretSharingForParty1(9);