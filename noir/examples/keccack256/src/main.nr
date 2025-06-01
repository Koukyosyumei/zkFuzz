use dep::keccak256::keccak256;

pub fn main(input: [u8; 5]) -> pub [u8; 32] {
    keccak256(input, input.len())
}