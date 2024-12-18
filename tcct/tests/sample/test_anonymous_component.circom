pragma circom 2.0.0;

template A(n){
   signal input a, b;
   signal output c;
   c <== n*a*b;
}

template B(n){
   signal input in[n];
   signal output out_1 <== A(n)(in[0], in[1]);
   signal output out_2 <== A(n+1)(in[1], in[0]);
}

component main = B(2);