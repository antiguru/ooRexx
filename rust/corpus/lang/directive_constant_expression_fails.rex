/* A ::CONSTANT directive's parenthesised expression fails at install time,
 * and the program's own main body never runs. Phase 5a Task 4, the
 * discriminating witness `phase-4-exclusions.txt`'s KNOWN GAP row asked for:
 * "the witness must make the expression fail".
 *
 * `1/0` is deliberately not a message send -- `.NoSuchClass~m`, the
 * expression the oracle probe that found this gap actually used, needs
 * message dispatch (Task 5), which does not exist yet and would make this
 * program fail for the wrong reason (a loud "not implemented" rather than
 * the oracle's own condition). Division by zero is fully implemented today
 * on both engines, so it exercises exactly this task's own new code: eager
 * evaluation of the expression, and propagating the raised condition with
 * the class's own clause blamed alongside the constant's.
 *
 * Measured, rc 214, stdout EMPTY (the failure is at install time, before
 * "prolog" is ever said), stderr two clause echoes innermost first:
 *
 *     28 *-* ::constant c (1/0)
 *     27 *-* ::class K
 * Error 42 running <path> line 28:  Arithmetic overflow/underflow.
 * Error 42.3:  Arithmetic overflow; divisor must not be zero.
 */

say 'prolog'
exit

::class K
::constant c (1/0)
