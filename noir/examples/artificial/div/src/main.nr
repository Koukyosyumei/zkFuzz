unconstrained fn vuln_div(x: Field, y: Field) -> Field {
    if y == 0 {
        let z = 0;
        z
    } else {
        let z = x / y;
        z
    }
}

fn main(x: Field, y: Field) -> pub Field {
    let z = unsafe { vuln_div(x, y) };
    assert(y * z == x);
    z
}

#[test]
fn test_main() {
    let z = main(3, 5);
    assert(z * 5 == 3);
}