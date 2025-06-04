template A() {
    signal input data_len_bytes;

    var padding_zero_bytes_var = ((data_len_bytes + 1 + 8 + 63)\64)*64 - (data_len_bytes + 1 + 8); 
    var padding_zero_bytes_as_bits[1];
    padding_zero_bytes_as_bits[0] = padding_zero_bytes_var >> 0 & 1;
    signal pzbb[1] <-- padding_zero_bytes_as_bits;
    (1 - pzbb[0]) * pzbb[0] === 0;  // pzbb[i] is a bit
}

component main = A();