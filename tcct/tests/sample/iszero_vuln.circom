pragma circom 2.0.0;

/**
 * @template IsZero
 * @description This circuit attempts to check if a given input is zero or non-zero. The output is expected to be:
 *              - `1` if the input is zero
 *              - `0` if the input is non-zero
 *              However, this circuit contains a vulnerability where the inverse (`inv`) can be manipulated, allowing 
 *              incorrect behavior under certain conditions.
 *
 * @input {signal} in - The input signal to be checked if it is zero or non-zero.
 * @output {signal} out - The output signal:
 *                        - `1` if `in == 0`
 *                        - `0` if `in != 0`
 *
 * @vulnerability The circuit uses the constraint `out == -in * inv + 1` to define the output. While `inv` is intended 
 *                to be set to `1/in` when `in != 0`, the `inv` signal is **free** to be manipulated. By modifying 
 *                `inv <-- 0`, the circuit would incorrectly output `1` for all inputs, regardless of whether `in` is 
 *                zero or not. This leads to a vulnerability where the behavior of the circuit can be tampered with.
 *
 * @example 
 * - Correct behavior:
 *   - `in == 0` → `inv == 0`, `out == 1`
 *   - `in != 0` → `inv == 1/in`, `out == 0`
 * 
 * - Vulnerable behavior:
 *   - By setting `inv = 0` for all cases, the output will always be `1`, even when `in != 0`.
 */
template VulnerableIsZero() {
    signal input in;    // Input signal to check if it's zero or non-zero.
    signal output out;  // Output signal: 1 if `in == 0`, 0 if `in != 0`.
    signal inv;         // Intended inverse of `in`, but vulnerable to manipulation.

     // Vulnerable inverse calculation: `inv` is set to `1/in` when `in != 0`, but this is manipulable.
    inv <-- in!=0 ? 1/in : 0;

    // Constraint to calculate `out`: vulnerable due to the free nature of `inv`.
    out <== -in*inv +1;
    out*(out-1) === 0;
}


/**
 * @component main
 * @description The main component using the `IsZero` template to check whether a public input is zero or non-zero. 
 *              However, this component is vulnerable to manipulation of the `inv` signal, allowing incorrect results.
 *
 * @input {signal} in - A public input to be checked for zero or non-zero.
 * @output {signal} out - Expected to be `1` if `in == 0`, and `0` if `in != 0`, but this behavior can be manipulated.
 */
component main = VulnerableIsZero();