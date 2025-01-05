pragma circom 2.1.6;
// https://github.com/shuklaayush/circom-monolith/blob/main/circuits/goldilocks.circom

// Order of Goldilocks field
function Order() { return 18446744069414584321; }

// Range check
// Verifies x < 1 << N
template LessNBits(N) {
  signal input x;
  var e2 = 1;
  signal tmp1[N];
  signal tmp2[N + 1];
  tmp2[0] <== 0;
  for (var i = 0; i < N; i++) {
    tmp1[i] <-- (x >> i) & 1;
    tmp1[i] * (tmp1[i] - 1) === 0;
    tmp2[i + 1] <== tmp1[i] * e2 + tmp2[i];
    e2 = e2 + e2;
  }
  x === tmp2[N];
}

// Gl: Goldilocks
// range check d < 1 << N
template GlReduce(N) {
  signal input x;
  signal output out;

  var r = x % Order();
  var d = (x - r) \ Order();
  out <-- r;
  signal tmp0 <-- d;
  tmp0 * Order() + out === x;

  component c0 = LessNBits(N);
  c0.x <== tmp0;
  component c1 = LessNBits(64);
  c1.x <== out;
}

template GlAdd() {
  signal input a;
  signal input b;
  signal output out;

  component cr = GlReduce(1);
  cr.x <== a + b;
  out <== cr.out;
}

template GlSub() {
  signal input a;
  signal input b;
  signal output out;

  component cr = GlReduce(1);
  cr.x <== a + Order() - b;
  out <== cr.out;
}

template GlMul() {
  signal input a;
  signal input b;
  signal output out;

  component cr = GlReduce(64);
  cr.x <== a * b;
  out <== cr.out;
}

function gl_inverse(x) {
  //assert(x != 0);
  var m = Order() - 2;
  var e2 = x;
  var res = 1;
  for (var i = 0; i < 64; i++) {
    if ((m >> i) & 1 == 1) {
      res *= e2;
      res %= Order();
    }
    e2 *= e2;
    e2 %= Order();
  }
  return res;
}

template GlInv() {
  signal input x;
  signal output out;
  out <-- gl_inverse(x);
  component check = GlMul();
  check.a <== x;
  check.b <== out;
}

component main = GlInv();