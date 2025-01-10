pragma circom 2.1.0;

include "../../include/circomlib/circuits/bitify.circom";
include "../../include/circomlib/circuits/comparators.circom";

// From https://pps-lab.com/blog/fhe_arithmetization/
template Modulus(p, n) {
  signal input x;
  signal q;
  signal output y;

  y <-- x % p;
  q <-- x \ p;
  x === q * p + y;

//   component ltP = LessThan(n);
//   ltP.in[0] <== p;
//   ltP.in[1] <== y;
//   ltP.out === 0;

  component ltQ = LessThan(n);
  ltQ.in[0] <== x;
  ltQ.in[1] <== q;
  ltQ.out === 0;
}

// Imagine the coefficients are like trains passing in opposite directions
// For example:
//   n=4
//   a=1,2,3,4 (4x^3 + 3x^2 + 2x + 1)
//   b=6,5,4,3 (3x^3 + 4x^2 + 5x + 6)
//
// 4,3,2,1 --->
//         <--- 6,5,4,3
//
// 4,3,2,1
//       6,5,4,3
//       ^ coeff #0 = 6*1
//
// 4,3,2,1
//     6,5,4,3
//     ^ ^ coeff #1 = 1*5 + 2*6
//
// 4,3,2,1
//   6,5,4,3
//   ^ ^ ^ coeff #2 = 1*4 + 2*5 + 3*6
//
// 4,3,2,1
// 6,5,4,3
// ^ ^ ^ ^ coeff #3 = 1*3 + 2*4 + 3*5 + 4*6
//
//   4,3,2,1
// 6,5,4,3
//   ^ ^ ^ coeff #4 = 2*3 + 3*4 + 4*5
//
//     4,3,2,1
// 6,5,4,3
//     ^ ^ coeff #5 = 3*3 + 4*4
//
//       4,3,2,1
// 6,5,4,3
//       ^ coeff #6 = 4*3
//
// Output: 6, 17, 32, 50, 38, 25, 12
// (12x^6 + 25x^5 + 38x^4 + 50x^3 + 32x^2 + 17x + 6)

template MultiplyPolynomials(n) {
  var newSize = n + n - 1;
  signal input a[n];
  signal input b[n];

  signal output result[newSize];

  // First and last coefficients are simple constraints
  result[0] <== a[0] * b[0];
  result[newSize-1] <== a[n-1] * b[n-1];

  component sums[newSize - 2];
  for(var k = 1; k < newSize - 1; k++) {
    var sumSize = 2 * n - k - 1;
    if(k < n) {
      sumSize = k + 1;
    }
    sums[k-1] = Sum(sumSize);
    for(var i = 0; i < sumSize; i++) {
      var aIndex = k - n + 1 + i;
      if(k < n) {
        aIndex = i;
      }
      var bIndex = k - aIndex;
      if(aIndex >= 0 && aIndex < n && bIndex >= 0 && bIndex < n) {
        sums[k-1].in[i] <== a[aIndex] * b[bIndex];
      }
    }

    result[k] <== sums[k-1].out;
  }
}

template PolyMod(n, p, np) {
  signal input in[n];
  signal output out[n];

  component modulus[n];
  for(var i = 0; i < n; i++) {
    modulus[i] = Modulus(p, np);
    modulus[i].x <== in[i];
    out[i] <== modulus[i].y;
  }
}

template MultiplyPolynomialsMod(n, p, np) {
  var newSize = n + n - 1;
  signal input a[n];
  signal input b[n];

  signal output result[newSize];

  component product = MultiplyPolynomials(n);
  product.a <== a;
  product.b <== b;

  component modulus = PolyMod(newSize, p, np);
  modulus.in <== product.result;
  result <== modulus.out;
}

template Sum(n) {
  signal input in[n];
  signal output out;

  if (n == 1) {
    out <== in[0];
  } else {
    signal partialSums[n];

    partialSums[0] <== in[0];

    for (var i = 1; i < n; i++) {
      partialSums[i] <== partialSums[i - 1] + in[i];
    }

    out <== partialSums[n - 1];
  }
}

component main = MultiplyPolynomialsMod(2, 7, 7);