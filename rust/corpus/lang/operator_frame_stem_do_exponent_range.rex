/* Fix round 2, finding 1: the same forwarded frame reached at a DO header's
 * own range check, past a conversion that already succeeded -- `s.`'s
 * default parses as a number and only `round_via_unary_plus`'s own exponent
 * range check fails. Measured, rc 214, stderr opening
 * `       *-* Compiled method "+" with scope "String".`.
 */

numeric digits 1
s. = '9.9E999999999'
do i = s. to 5
end
