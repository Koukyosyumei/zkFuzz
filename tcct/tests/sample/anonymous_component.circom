pragma circom 2.0.0;

template A(n){
   signal input a, b;
   signal output c;
   c <== a*b;
}

template B(n){
   signal input in[n];
   signal out <== A(n)(in[0],in[1]);
}

component main = B(2);