/*
	Common functionality for Blake3. The starting point of this code is from Blake 2: https://github.com/bkomuves/hash-circuits/blob/master/circuits/blake2/blake2_common.circom
*/

pragma circom 2.1.6;

//------------------------------------------------------------------------------


/*
	The permutation table where `iO` is the index of the permutation round
	Wait maybe not...
	This is the wrong permutation and corresponds to Blake2s
*/
template Blake3Permute() {
	// TODO: fix up
	signal input inp[16];
  signal output out[16];

  var sigma[16] =
    [2, 6, 3, 10, 7, 0, 4, 13, 1, 11,  12, 5,  9, 14, 15, 8];


  for(var j=0; j<16; j++) { out[j] <== inp[sigma[j]]; }
  // for(var j=0; j<16; j++) { out[sigma[j]] <== inp[j]; }
}


//------------------------------------------------------------------------------
// XOR 3 bits together

template XOR3() {
  signal input  x;
  signal input  y;
  signal input  z;
  signal output out;

  signal tmp <== y*z;
  out <== x * (1 - 2*y - 2*z + 4*tmp) + y + z - 2*tmp;
}

template XOR2() {
  signal input  x;
  signal input  y;
  signal output out;

  // If x = 0, then out = y and vice versa
  // If both are 1, the output 0
	out <== x + y - 2*x*y;
}

//------------------------------------------------------------------------------
// XOR 2 words together

template XorWord2(n) {
  signal input  x;
  signal input  y;

  signal out_bits[n];
  signal output out_word;

  component tb_x = ToBits(n); 
  component tb_y = ToBits(n);

  tb_x.inp <== x;
  tb_y.inp <== y;  

  component xor[n];

  var acc = 0;
  for(var i=0; i<n; i++) { 
    xor[i] = XOR2();
    xor[i].x   <== tb_x.out[i];
    xor[i].y   <== tb_y.out[i];
    xor[i].out ==> out_bits[i];
    acc += out_bits[i] * (2**i);
  }

  out_word <== acc;
}



//------------------------------------------------------------------------------
// XOR 3 words together

template XorWord3(n) {
  signal input  x;
  signal input  y;
  signal input  z;
  signal output out_bits[n];
  signal output out_word;

  component tb_x = ToBits(n); 
  component tb_y = ToBits(n);
  component tb_z = ToBits(n);

  tb_x.inp <== x;
  tb_y.inp <== y;  
  tb_z.inp <== z;

  component xor[n];

  var acc = 0;
  for(var i=0; i<n; i++) { 
    xor[i] = XOR3();
    xor[i].x   <== tb_x.out[i];
    xor[i].y   <== tb_y.out[i];
    xor[i].z   <== tb_z.out[i];
    xor[i].out ==> out_bits[i];
    acc += out_bits[i] * (2**i);
  }

  out_word <== acc;
}

//------------------------------------------------------------------------------
// XOR a word with a constant

template XorWordConst(n, kst_word) {
  signal input  inp_word;
  signal output out_bits[n];
  signal output out_word;

  component tb = ToBits(n);
  tb.inp <== inp_word;

  var acc = 0;
  for(var i=0; i<n; i++) {
    var x = tb.out[i];
    var y = (kst_word >> i) & 1;
    out_bits[i] <== x + y - 2*x*y;
    acc += out_bits[i] * (2**i);
  }

  out_word <== acc;  
}

//------------------------------------------------------------------------------
// decompose an n-bit number into bits

template ToBits(n) {
  signal input  inp;
  signal output out[n];

  var sum = 0;
  for(var i=0; i<n; i++) {
    out[i] <-- (inp >> i) & 1;
    out[i] * (1-out[i]) === 0;
    sum += (1<<i) * out[i];
  }

  inp === sum;
}

//------------------------------------------------------------------------------
// decompose a 33-bit number into the low 32 bits and the remaining 1 bit

// TODO: check me
template Bits33() {
  signal input  inp;
  signal output out_bits[32];
  signal output out_word;
  signal u;

  var sum = 0;
  for(var i=0; i<32; i++) {
    out_bits[i] <-- (inp >> i) & 1;
    out_bits[i] * (1-out_bits[i]) === 0;
    sum += (1<<i) * out_bits[i];
  }

  u <-- (inp >> 32) & 1;
  u*(1-u) === 0;

  inp === sum + (1<<32)*u;
  out_word <== sum;
}

//------------------------------------------------------------------------------
// decompose a 34-bit number into the low 32 bits and the remaining 2 bits

template Bits34() {
  signal input  inp;
  signal output out_bits[32];
  signal output out_word;
  signal u,v;

  var sum = 0;
  for(var i=0; i<32; i++) {
    out_bits[i] <-- (inp >> i) & 1;
    out_bits[i] * (1-out_bits[i]) === 0;
    sum += (1<<i) * out_bits[i];
  }

  u <-- (inp >> 32) & 1;
  v <-- (inp >> 33) & 1;
  u*(1-u) === 0;
  v*(1-v) === 0;

  inp === sum + (1<<32)*u + (1<<33)*v;
  out_word <== sum;
}

//------------------------------------------------------------------------------
// decompose a 65-bit number into the low 64 bits and the remaining 1 bit

template Bits65() {
  signal input  inp;
  signal output out_bits[64];
  signal output out_word;
  signal u;

  var sum = 0;
  for(var i=0; i<64; i++) {
    out_bits[i] <-- (inp >> i) & 1;
    out_bits[i] * (1-out_bits[i]) === 0;
    sum += (1<<i) * out_bits[i];
  }

  u <-- (inp >> 64) & 1;
  u*(1-u) === 0;

  inp === sum + (1<<64)*u;
  out_word <== sum;
}

//------------------------------------------------------------------------------
// decompose a 66-bit number into the low 64 bits and the remaining 2 bit

template Bits66() {
  signal input  inp;
  signal output out_bits[64];
  signal output out_word;
  signal u,v;

  var sum = 0;
  for(var i=0; i<64; i++) {
    out_bits[i] <-- (inp >> i) & 1;
    out_bits[i] * (1-out_bits[i]) === 0;
    sum += (1<<i) * out_bits[i];
  }

  u <-- (inp >> 64) & 1;
  v <-- (inp >> 65) & 1;
  u*(1-u) === 0;
  v*(1-v) === 0;

  inp === sum + (1<<64)*u + (1<<65)*v;
  out_word <== sum;
}

//------------------------------------------------------------------------------