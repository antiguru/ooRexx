/* A ::CONSTANT directive's parenthesised expression with no preceding
 * ::CLASS is a translation-time refusal, checked structurally rather than by
 * evaluating anything. Phase 5a Task 4.
 *
 * This is the OTHER of the ::CONSTANT KNOWN GAP row's two shapes, and it has
 * its own exit code: measured, rc 157 (not 214, which is what a failing
 * evaluation gives), stdout EMPTY, stderr ONE clause echo -- there is no
 * enclosing ::CLASS to blame alongside it:
 *
 *     23 *-* ::constant sep (1+2)
 * Error 99 running <path> line 23:  Translation error.
 * Error 99.906:  A ::CONSTANT directive with an expression requires a
 * matching ::CLASS directive.
 *
 * The expression itself is well-formed (`1+2`), which is deliberate: this
 * program is a witness for the STRUCTURAL check alone, and would still fail
 * this way even if the expression could never raise anything.
 */

say 'prolog'
exit

::constant sep (1+2)
