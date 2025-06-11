use dep::sort::sort_via;

fn sort_u32(a: u32, b: u32) -> bool {
    a <= b
}

pub fn main(arr: [u32; 7]) -> pub [u32; 7] {
    sort_via(arr, sort_u32)
}