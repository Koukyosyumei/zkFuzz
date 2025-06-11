use dep::bignum::{BigNum, BN254_Fq};

pub fn main(seed1: [u8; 4], seed2: [u8; 4]) -> pub bool {
    let a: BN254_Fq = BigNum::derive_from_seed(seed1);
    let b: BN254_Fq = BigNum::derive_from_seed(seed2);
    a < b
}