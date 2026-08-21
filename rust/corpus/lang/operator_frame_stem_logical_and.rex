/* Fix round 2, finding 1: the same forwarded frame reached through `&`,
 * whose own check is textual rather than numeric -- `s.`'s default "abc" is
 * not a logical value, and that check (not a numeric conversion) is what
 * fails inside the forwarded method. Measured, rc 222, stderr opening
 * `       *-* Compiled method "&" with scope "String".`.
 */

s. = 'abc'
say s. & 1
