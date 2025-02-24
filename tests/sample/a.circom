template Main() {
    signal input x1;
    signal input x2;
    signal output y;
    y <-- x1 / x2;
    y * x2 === x1;
}

component main = Main();