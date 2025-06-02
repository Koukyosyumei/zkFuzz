use dep::nodash::field_to_hex;

pub fn main(x: Field) -> pub str<64> {
    field_to_hex(x)
}