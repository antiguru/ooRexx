/* Phase 5a Task 6: an unset stem forwards an operator it does not answer
 * itself to its default value (the same forward `~length` already reaches),
 * and the method that raises is the default's -- so the traceback names
 * scope "String", not "Stem", which is `b.~class`. Measured, rc 215, stderr
 * opening `       *-* Compiled method "+" with scope "String".`.
 */

say b. + 1
