/* Two library frames at once, which is what says the sourceless frame is
 * built per activation rather than once for the innermost. Phase 5a Task 23.
 *
 * `.TimeSpan~fromDays` validates through `.Validate~number`, so the
 * traceback carries `NUMBER with scope "Validate"` above `FROMDAYS with
 * scope "TimeSpan"`, each under its own line number and **at its own
 * indent** -- the inner one is inside a `DO` and the outer one is not.
 *
 * The line the report names is the innermost frame's, and the package is
 * still `REXX`. See `library_method_traceback.rex` for the single-frame
 * case and for what is deliberately not fixed here. rc 168.
 */

say .TimeSpan~fromDays('x')
