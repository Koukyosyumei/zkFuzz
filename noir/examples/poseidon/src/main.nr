use dep::poseidon::poseidon2;

pub fn main(input: [Field; 5]) -> pub Field {
    poseidon2::Poseidon2::hash(input, input.len())
}