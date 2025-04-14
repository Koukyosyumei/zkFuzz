pragma circom 2.0.6;

include "../sign/circuit.circom";

// 浮点数乘法，输入无符号，输出无符号
template FloatMulti() {
    signal input l[2];
    signal input r[2];
    signal output p[2];

    p[0] <== l[0] * r[0];
    p[1] <== l[1] + r[1];
}

// 浮点数乘法，输入有符号，输出无符号
template FloatMultiSign() {
    signal input l[3];
    signal input r[3];
    signal output p[2];

    component us0 = UnSign();
    component us1 = UnSign();
    component fm = FloatMulti();

    us0.in <== l[0];
    us0.sign <== l[2];
    us1.in <== r[0];
    us1.sign <== r[2];

    fm.l[0] <== us0.out;
    fm.l[1] <== l[1];
    fm.r[0] <== us1.out;
    fm.r[1] <== r[1];

    p[0] <== fm.p[0];
    p[1] <== fm.p[1];
}

// 浮点数加法，输入无符号，输出无符号
template FloatAdd() {
    signal input l[2];
    signal input r[2];
    signal output s[2];
    signal left;
    signal right;

    component gtd = GreaterThan(5);
    component swd = Switcher();
    component swf = Switcher();

    // 计算精度差
    gtd.in[0] <== l[1];
    gtd.in[1] <== r[1];

    swd.sel <== gtd.out;
    swd.L <== l[1];
    swd.R <== r[1];

    var d = swd.outR - swd.outL;
    var fac = 10**d;

    // 计算对齐精度后的两个操作数
    swf.sel <== gtd.out;
    swf.L <-- fac;
    swf.R <== 1;
    left <== swf.outL * l[0];
    right <== swf.outR * r[0];

    s[0] <== left + right;
    s[1] <== swd.outR;
}

// 浮点数加法，输入有符号，输出无符号
template FloatAddSign() {
    signal input l[3];
    signal input r[3];
    signal output s[2];

    component usl = UnSign();
    component usr = UnSign();
    component fa = FloatAdd();

    usl.in <== l[0];
    usl.sign <== l[2];
    usr.in <== r[0];
    usr.sign <== r[2];

    fa.l[0] <== usl.out;
    fa.l[1] <== l[1];
    fa.r[0] <== usr.out;
    fa.r[1] <== r[1];

    s[0] <== fa.s[0];
    s[1] <== fa.s[1];
}

// 浮点数加法，默认精度相同，输入无符号，输出无符号
template FloatAddSimple() {
    signal input l[2];
    signal input r[2];
    signal output s[2];

    l[1] === r[1];

    s[0] <== l[0] + r[0];
    s[1] <== l[1];
}

// 浮点数加法，默认精度相同，输入有符号，输出无符号
template FloatAddSignSimple() {
    signal input l[3];
    signal input r[3];
    signal output s[2];

    component usl = UnSign();
    component usr = UnSign();
    component fa = FloatAddSimple();

    usl.in <== l[0];
    usl.sign <== l[2];
    usr.in <== r[0];
    usr.sign <== r[2];

    fa.l[0] <== usl.out;
    fa.l[1] <== l[1];
    fa.r[0] <== usr.out;
    fa.r[1] <== r[1];

    s[0] <== fa.s[0];
    s[1] <== fa.s[1];
}

// component main = FloatAdd();