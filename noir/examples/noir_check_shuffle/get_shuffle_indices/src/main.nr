use dep::noir_check_shuffle::get_shuffle_indices;

pub fn main(lhs: [Field; 5], rhs: [Field; 5]) -> pub [u32; 5] {
    get_shuffle_indices(lhs, rhs)
}