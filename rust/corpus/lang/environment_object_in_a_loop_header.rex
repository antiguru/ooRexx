/* A DO header's numeric positions, with one of these objects in them.
 * Phase 5a Task 6, fix round 2.
 *
 * The initial, TO and BY positions are rounded through a real unary + on the
 * oracle, so it sends + to the object and answers 97.1 for each of the three.
 * This crate refuses them, and a refusal cannot be witnessed by a corpus file
 * -- eval.rs's object_operand_tests carries that half.
 *
 * What is here is the boundary either side of it. FOR, a bare DO's repeat
 * count and NUMERIC DIGITS read the value's TEXT through whole_nonneg rather
 * than converting it to a number, so both implementations answer 26 from the
 * same bytes; a builtin argument does the same and answers 40. Making the
 * whole header loud would take every line below from an agreement to a
 * refusal.
 *
 * Each syntax trap signals to the label that starts the next case, which is
 * what lets one program carry four raises: a syntax condition is not
 * resumable, so without the chain each case would end the program. The last
 * line is what says all four fired.
 *
 * Measured, rc 0.
 */

signal on syntax name after_for
do i = 1 to 5 for .ARRAY
end

after_for:
say 'for      rc' rc errortext(rc)

signal on syntax name after_count
do .ARRAY
end

after_count:
say 'count    rc' rc

signal on syntax name after_digits
numeric digits .ARRAY

after_digits:
say 'digits   rc' rc

signal on syntax name after_builtin
say substr('abcdef', .ARRAY)

after_builtin:
say 'builtin  rc' rc
say 'all four raised'
