use dep::bignum::{BigNum, BN254_Fq};

pub fn main(seed1: [u8; 4], seed2: [u8; 4]) -> pub BN254_Fq {
    let a: BN254_Fq = BigNum::derive_from_seed(seed1);
    let b: BN254_Fq = BigNum::derive_from_seed(seed2);
    let c = a / b;
    assert(b * c == a);
    c
}