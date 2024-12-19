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
/// use tcct::executor::utils::generate_cartesian_product_indices;
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
