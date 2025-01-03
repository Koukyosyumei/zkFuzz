pragma circom 2.1.6;

template RShift1() {
  signal input x;
  signal output out;
  signal output bit;

  var o = x >> 1;
  out <-- o;
  bit <== x - out * 2;
  bit * (1 - bit) === 0;
}

component main = RShift1();