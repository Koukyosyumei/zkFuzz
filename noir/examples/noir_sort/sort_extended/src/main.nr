use dep::sort::sort_extended;

unconstrained fn __sort_u32(a: u32, b: u32) -> bool {
    a <= b
}

// unconditional_lt will cost fewer constraints than the `<=` operator
// as we do not need to constrain the case where `a > b`, and assign a boolean variable to the result
fn unconditional_lt(_a: u32, _b: u32) {
    let a = _a as Field;
    let b = _b as Field;

    let diff = b - a;
    diff.assert_max_bit_size::<32>();
}

pub fn main(arr: [u32; 7]) -> pub [u32; 7] {
    sort_extended(arr, __sort_u32, unconditional_lt)
}