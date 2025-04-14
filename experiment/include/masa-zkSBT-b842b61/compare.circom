pragma circom 2.0.4;

include "../circomlib-cff5ab6/mux3.circom";
include "../circomlib-cff5ab6/comparators.circom";

// Solution from https://github.com/r0qs/zkcertree/blob/main/circuits/compare.circom
// Compare compares a with b based on the given operator op
// Operator is represented as a 3 bits bitmap.
// Currently supported operations are:
// 000: ==
// 001: !=
// 010: >
// 011: >=
// 100: <
// 101: <=
// 110: not_implemented
// 111: not_implemented
// It returns out with 1 if the condition is satisfied or 0 otherwise.
template Compare() {
	signal input a;
	signal input b;
	signal input op;
	signal output out;

	component validOp = LessThan(252);
	validOp.in[0] <== op;
	validOp.in[1] <== 6;
	// Restrict to supported operators
	assert(validOp.out);

	component eq = IsEqual();
	eq.in[0] <== a;
	eq.in[1] <== b;

	component gt = GreaterThan(252);
	gt.in[0] <== a;
	gt.in[1] <== b;
	
	component gte = GreaterEqThan(252);
	gte.in[0] <== a;
	gte.in[1] <== b;

	component lt = LessThan(252);
	lt.in[0] <== a;
	lt.in[1] <== b;

	component lte = LessEqThan(252);
	lte.in[0] <== a;
	lte.in[1] <== b;

	component mux = Mux3();
	component n2b = Num2Bits(3);
	n2b.in <== op;
	mux.s[0] <== n2b.out[0];
	mux.s[1] <== n2b.out[1];
	mux.s[2] <== n2b.out[2];

	mux.c[0] <== eq.out;
	mux.c[1] <== 1 - eq.out;
	mux.c[2] <== gt.out;
	mux.c[3] <== gte.out;
	mux.c[4] <== lt.out;
	mux.c[5] <== lte.out;
	mux.c[6] <== 0;
	mux.c[7] <== 0;

	mux.out ==> out;
}