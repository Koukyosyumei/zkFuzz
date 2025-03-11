use num_bigint_dig::BigInt;
use num_traits::{One, Signed, Zero};
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

pub fn moddiv(lv: &BigInt, rv: &BigInt, modulus: &BigInt) -> BigInt {
    if lv.is_zero() || rv.is_zero() {
        return BigInt::zero();
    }

    let mut r = modulus.clone();
    let mut new_r = rv.clone();
    if r.is_negative() {
        r += modulus;
    }
    if new_r.is_negative() {
        new_r += modulus;
    }

    let (_, _, mut rv_inv) = extended_euclidean(r, new_r);
    rv_inv %= modulus;
    if rv_inv.is_negative() {
        rv_inv += modulus;
    }

    let mut result = (lv * rv_inv) % modulus;
    if result.is_negative() {
        result += modulus;
    }
    result
}

/// Returns Some(x) such that x² ≡ n (mod p), or None if no solution exists.
/// Assumes that `p` is an odd prime.
/// # Examples
/// ```
/// use num_bigint_dig::BigInt;
/// use zkfuzz::executor::utils::tonelli_shanks;
///
/// let n = BigInt::from(5);
/// let p = BigInt::from(41);
/// let n_square = tonelli_shanks(&n, &p).unwrap();
/// let answer = BigInt::from(28);
/// assert_eq!(n_square, answer);
/// ```
pub fn tonelli_shanks(n_original: &BigInt, p: &BigInt) -> Option<BigInt> {
    let one = BigInt::one();
    let two = BigInt::from(2u32);

    let n = if n_original.is_negative() {
        n_original + p
    } else {
        n_original.clone()
    };

    // Handle trivial cases.
    if n.is_zero() {
        return Some(BigInt::zero());
    }
    if p == &two {
        return Some(n % p);
    }

    // Check if n is a quadratic residue mod p using Euler's criterion:
    // n^((p-1)/2) mod p should be 1.
    let exp = (p - &one) >> 1; // (p - 1) / 2
    if n.modpow(&exp, p) != one {
        return None;
    }

    // Factor p - 1 as q * 2^s with q odd.
    let mut q = p - &one;
    let mut s = 0;
    while (&q & &one) == BigInt::zero() {
        q >>= 1;
        s += 1;
    }

    // Find a quadratic non-residue z modulo p.
    let mut z = BigInt::from(2u32);
    while z.modpow(&exp, p) == one {
        z += &one;
    }

    let mut m = s;
    let mut c = z.modpow(&q, p);
    let mut t = n.modpow(&q, p);
    let mut r = n.modpow(&((&q + &one) >> 1), p);

    // Main loop: repeat until t ≡ 1 (mod p).
    while t != one {
        // Find the smallest i (0 < i < m) such that t^(2^i) ≡ 1 mod p.
        let mut i = 0;
        let mut temp = t.clone();
        while temp != one {
            temp = temp.modpow(&two, p);
            i += 1;
            if i == m {
                return None; // Should not happen if n is a residue.
            }
        }

        // Compute b = c^(2^(m-i-1)) mod p.
        let exponent = BigInt::from(1u32) << (m - i - 1);
        let b = c.modpow(&exponent, p);

        m = i;
        c = b.modpow(&two, p);
        t = (t * &c) % p;
        r = (r * b) % p;
    }

    Some(r)
}

pub fn solve_quadratic_modulus_equation(coeffs: &[BigInt; 3], modulus: &BigInt) -> Option<BigInt> {
    if coeffs[2].is_zero() && coeffs[1].is_zero() {
        None
    } else if coeffs[2].is_zero() {
        Some(moddiv(&-&coeffs[0], &coeffs[1], modulus))
    } else {
        let d = (&coeffs[1] * &coeffs[1] - BigInt::from(4) * &coeffs[2] * &coeffs[0]) % modulus;
        let root_d = tonelli_shanks(&d, modulus);
        if let Some(r) = root_d {
            Some(moddiv(
                &(-&coeffs[1] + r),
                &(BigInt::from(2) * &coeffs[2]),
                modulus,
            ))
        } else {
            None
        }
    }
}

/// Generates all combinations of indices for a given set of dimensions.
///
/// This function takes a slice of dimensions (represented as a slice of usize)
/// and produces a vector of vectors, where each inner vector represents a unique
/// combination of indices across all dimensions.
///
/// # Arguments
/// - `dims`: A slice of usize values, where each value represents the size of a dimension.
///
/// # Returns
/// A `Vec<Vec<usize>>` where each inner vector contains indices for a unique combination.
///
/// # Examples
/// ```
/// use zkfuzz::executor::utils::generate_cartesian_product_indices;
///
/// // Example 1: Two dimensions with sizes 2 and 3
/// let dims = &[2, 3];
/// let combinations = generate_cartesian_product_indices(dims);
/// assert_eq!(
///     combinations,
///     vec![
///         vec![0, 0],
///         vec![0, 1],
///         vec![0, 2],
///         vec![1, 0],
///         vec![1, 1],
///         vec![1, 2],
///     ]
/// );
///
/// // Example 2: Three dimensions with sizes 2, 2, and 2
/// let dims = &[2, 2, 2];
/// let combinations = generate_cartesian_product_indices(dims);
/// assert_eq!(
///     combinations,
///     vec![
///         vec![0, 0, 0],
///         vec![0, 0, 1],
///         vec![0, 1, 0],
///         vec![0, 1, 1],
///         vec![1, 0, 0],
///         vec![1, 0, 1],
///         vec![1, 1, 0],
///         vec![1, 1, 1],
///     ]
/// );
/// ```
///
/// # Complexity
/// This function has a time complexity of O(dims.iter().product()) due to the nested loops
/// iterating over all combinations.
///
/// # Safety
/// This function assumes `dims` does not contain negative sizes or excessively large values
/// that could cause memory overflow.
pub fn generate_cartesian_product_indices(dims: &[usize]) -> Vec<Vec<usize>> {
    let mut positions: Vec<Vec<usize>> = vec![vec![]];
    for size in dims {
        let mut new_positions: Vec<Vec<usize>> = vec![];
        for combination in &positions {
            for i in 0..*size {
                let mut new_combination = combination.clone();
                new_combination.push(i);
                new_positions.push(new_combination);
            }
        }
        positions = new_positions;
    }
    positions
}

pub fn italic(text: &str) -> String {
    format!("\x1b[3m{}\x1b[0m", text)
}
