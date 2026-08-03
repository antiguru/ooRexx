/* ExprKind::Call, the internal-function expression form (4b Task 4): f(...)
   evaluated for its own value inside a larger expression, not run as a CALL
   clause of its own.

   Under trace r throughout, because part of what this program pins is in
   the trace and not in the output. eval_call (eval.rs) reuses
   resolve_and_run_call, the same nested-activation machinery CALL already
   uses (call_return.rex, Task 3), so f's own clauses below echo at the
   calling clause's own indent plus two (D2r) exactly as a CALLed routine's
   do, even though this is an expression term and not an instruction.

   The indent is NOT what this program's own differential run against the
   oracle pins (review finding I1, Task 4 fix round 1): corpus.rs's
   DEVIATION 0 collapses the run of spaces between a trace line's marker and
   its content, exactly what D2r's indent produces, so a version with no
   indent bookkeeping at all for the expression form still matches here.
   What the differential run does pin: stdout byte for byte (42, in f,
   result: before -- the last one below discriminates a wrongly-settled
   RESULT) and stderr's clause sequence and line numbers, which
   normalisation does not touch. The indent itself is pinned by run.rs's own
   unit tests (current_value_indent_is_restored_after_a_nested_expression_
   call and its neighbours), which compare raw bytes outside DEVIATION 0's
   reach.

   RESULT is untouched by the expression form (measured, eval_call's own
   doc): result is set before the call and read back unchanged after it,
   which a version that wrongly settled RESULT the way CALL does would
   break -- and it is this program's stdout, not its trace, that catches
   that. */
trace r
result = 'before'
say f(1) + 1
say 'result:' result
exit

f:
say 'in f'
return 41
