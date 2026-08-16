/* A ::CONSTANT directive's parenthesised expression evaluates at install
 * time and a well-formed one changes nothing observable. Phase 5a Task 4.
 *
 * The KNOWN GAP this closes: before this task the expression form was
 * accepted and never evaluated, so a well-formed expression and an ignored
 * one produced the same bytes -- this program alone could not have told the
 * two apart, which is exactly why `directive_constant_expression_fails.rex`
 * beside it exists. Measured, rc 0, stdout "prolog" on both sides; the
 * constant's own value is discarded here, since nothing in this crate can
 * read it back before message dispatch (Task 5) and Rexx method invocation
 * (Task 7) exist.
 */

say 'prolog'
exit

::class K
::constant c (2+3)
