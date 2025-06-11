unconstrained fn vuln_iszero(x: Field) -> Field {
    let mut inv: Field = 0;
    if x == 0 {
        inv = 0;
    } else {
        inv = 1 / x;
    }

    let out = -x * inv + 1;
    out
}

fn main(x: Field) -> pub Field {
    let z = unsafe { vuln_iszero(x) };
    assert(z * (z - 1) == 0);
    z
}

#[test]
fn test_main() {
    let z = main(3);
    assert(z == 0);
}