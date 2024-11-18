pragma circom 2.0.0;

/**
 * @template IsZero
 * @description This circuit checks whether a given input is zero or non-zero, and produces a corresponding output.
 *              The circuit uses a combination of constraints to ensure that:
 *                - If the input is zero, the output is 1.
 *                - If the input is non-zero, the output is 0.
 *              Additionally, an inverse (`inv`) is computed, which is `1/in` when `in != 0` and `0` when `in == 0`.
 *
 * @input {signal} in - The input signal to be checked if it is zero or non-zero.
 * @output {signal} out - The output signal:
 *                        - `1` if `in == 0`
 *                        - `0` if `in != 0`
 *
 * @remark The logic of the circuit is derived from two key constraints:
 *         - The first constraint (`out == -in * inv + 1`) ensures that if `in != 0`, then `out` is forced to 0.
 *         - The second constraint (`in * out == 0`) ensures that `out` is only 1 when `in == 0`.
 *         This behavior closely follows how an inverse operation works in algebraic systems, where dividing by zero is undefined.
 */
template IsZero() {
    signal input in;    // Input signal to check if it's zero or non-zero.
    signal output out;  // Output signal: 1 if `in == 0`, 0 if `in != 0`.
    signal inv;         // Inverse of the input when `in != 0`, or 0 when `in == 0`.
    
    // Compute the inverse: if `in` is non-zero, `inv` is set to `1/in`, otherwise it's 0.
    inv <-- in!=0 ? 1/in : 0;

    // Constraint 1: Ensures that if `in != 0`, `out` is 0. If `in == 0`, `out` is 1.
    out <== -in*inv +1;

    // Constraint 2: Ensures that `in * out == 0`, forcing the output to 1 only when `in == 0`.
    in*out === 0;
}


/**
 * @component main
 * @description The main component for checking if the public input is zero or non-zero, 
 *              using the IsZero template.
 *
 * @input {signal} in - A public input signal to be checked if it is zero or non-zero.
 * @output {signal} out - The result of the zero check: 1 if `in == 0`, 0 if `in != 0`.
 */
component main = IsZero();