unconstrained fn vuln_rshift1(x: u32) -> u32 {
    let y = x >> 1;
    let b = x - y * 2;
    b
}

fn main(x: u32) -> pub u32 {
    let b = unsafe { vuln_rshift1(x) };
    assert(b*(1 - b) == 0); 
    b
}

#[test]
fn test_main() {
    let z = main(7);
    assert(z == 3);
}