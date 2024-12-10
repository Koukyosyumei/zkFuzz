use num_bigint_dig::BigInt;
use num_traits::{One, Zero};
use std::ops::{Div, Rem, Sub};

pub fn extended_euclidean<F>(a: F, b: F) -> (F, F, F)
where
    F: Clone + PartialEq + Sub<Output = F> + Div<Output = F> + Rem<Output = F> + Zero + One,
{
    let mut r0 = a;
    let mut r1 = b;
    let mut s0 = F::one();
    let mut s1 = F::zero();
    let mut t0 = F::zero();
    let mut t1 = F::one();

    while !r1.is_zero() {
        let q = r0.clone() / r1.clone();
        let r = r0.clone() % r1.clone();
        r0 = r1;
        r1 = r;
        let new_s = s0.clone() - q.clone() * s1.clone();
        s0 = s1;
        s1 = new_s;
        let new_t = t0.clone() - q * t1.clone();
        t0 = t1;
        t1 = new_t;
    }

    (r0, s0, t0)
}

pub fn modpow(base: &BigInt, exp: &BigInt, modulus: &BigInt) -> BigInt {
    let mut result = BigInt::from(1);
    let mut base = base % modulus; // Reduce base mod modulus initially
    let mut exp = exp.clone();

    while exp > BigInt::from(0) {
        // If exp is odd, multiply base with result
        if &exp % 2 == BigInt::from(1) {
            result = (result * &base) % modulus;
        }
        // Square the base and halve the exponent
        base = (&base * &base) % modulus;
        exp /= 2;
    }
    result
}

pub fn italic(text: &str) -> String {
    format!("\x1b[3m{}\x1b[0m", text)
}
