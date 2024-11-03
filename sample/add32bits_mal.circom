template MaliciousAdd32Bits() {
    signal input a;     
    signal input b;     
    signal tmp;         
    signal output out; 
    // Maliciously hardcoded flag
    tmp <-- 0;
    // Ensure that the overflow flag (tmp) is either 0 or 1.
    tmp * (tmp - 1) === 0;
    // If overflow occurs, subtract 0x100000000 (2^32) from the result to simulate wrap-around.
    out <== (a + b) - (tmp * (0xFFFFFFFF + 1));
}

