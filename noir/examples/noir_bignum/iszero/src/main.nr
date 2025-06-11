use std::ops::{Add, Neg, Sub};
use dep::bignum::{BigNum, BN254_Fq};

pub fn main(seed: [u8; 4]) -> pub bool {
    let a: BN254_Fq = BigNum::derive_from_seed(seed);
    a.is_zero()
}