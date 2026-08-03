/* ExprKind::Call, the internal-function expression form (4b Task 4): f(...)
   evaluated for its own value inside a larger expression, not run as a CALL
   clause of its own.

   Under trace r throughout, because half of what this program pins is in
   the trace and not in the output. eval_call (eval.rs) reuses
   resolve_and_run_call, the same nested-activation machinery CALL already
   uses (call_return.rex, Task 3), so f's own clauses below must echo at the
   calling clause's own indent plus two (D2r) exactly as a CALLed routine's
   do, even though this is an expression term and not an instruction of its
   own -- measured, and the reason this program is here rather than trusting
   the unit tests alone: a version that skipped the indent bookkeeping for
   the expression form would still pass every eval.rs test (none of them
   inspect stderr this closely) and would only diverge here.

   RESULT is untouched by the expression form (measured, eval_call's own
   doc): result is set before the call and read back unchanged after it,
   which a version that wrongly settled RESULT the way CALL does would
   break. */
trace r
result = 'before'
say f(1) + 1
say 'result:' result
exit

f:
say 'in f'
return 41
