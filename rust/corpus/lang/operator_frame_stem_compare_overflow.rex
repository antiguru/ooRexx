/* Fix round 3, finding 1: the goal's corrected premise (be81ce689) -- the
 * operator families split by whether the operator needs a number, not by
 * whether it is arithmetic. A non-strict comparison converts both operands
 * to a Number exactly like an arithmetic operator once both are numeric, so
 * it reaches the same forwarded frame when that conversion's own arithmetic
 * overflows past it. One program stands for the family: `>` here, and `<`,
 * `>=`, `<=`, `=` and `\=` all reach the identical `compare_values` path and
 * were measured to carry the same frame. Strict `==`/`>>` and the
 * non-numeric fallback (`compare_strings`) never reach `compare_numbers` at
 * all and stay frameless, checked separately.
 * Measured, rc 214, stderr opening
 * `       *-* Compiled method ">" with scope "String".`.
 */

numeric digits 1
s. = '9.9E999999999'
say s. > 1
