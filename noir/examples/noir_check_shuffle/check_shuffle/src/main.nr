use dep::noir_check_shuffle::check_shuffle;

pub fn main(lhs: [Field; 5], rhs: [Field; 5]) {
    check_shuffle(lhs, rhs);
}