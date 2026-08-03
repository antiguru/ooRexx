/* CALL and RETURN (4b Task 3): the shared variable pool, RESULT, and the
   NUMERIC settings a callee inherits and does not write back.

   Under `trace r` throughout, but the indent this header used to claim as
   pinned here is not (review finding I1, Task 4 fix round 1): corpus.rs's
   DEVIATION 0 collapses the run of spaces between a trace line's marker and
   its content, exactly what D2r's indent rule produces, so `call inner`
   (inside a DO inside `outer:`) echoing at 6 -- the case that tells D2r's
   rule apart from `2 x depth` -- is pinned by `run.rs`'s own unit tests
   instead (`a_callees_clauses_echo_at_the_calling_clauses_indent_plus_two`),
   which compare raw bytes outside DEVIATION 0's reach, not by this
   program's differential run. What this program's differential run does
   pin: stdout byte for byte (the RESULT reads after each call below) and
   stderr's clause sequence and line numbers, which normalisation does not
   touch.

   The DO block here is a plain block and never a repetitive one,
   deliberately: phase-4-exclusions.txt's KNOWN GAP row is about a
   repetitive DO/LOOP that completes a body pass and then ends on a failing
   control test, which moves every later clause's indent by two on this
   crate's own implementation but not the oracle's -- an unrelated
   divergence this witness does not exist to exercise, and one a repetitive
   DO/LOOP here would silently introduce into the comparison.

   No PROCEDURE anywhere, so every callee shares the caller's pool: `outer:`
   reads CALLER_V and writes CALLEE_W, and both are visible from the main body
   after the return. A witness with no variables in it passes against an
   implementation that isolates every callee, which is why they are here. */
trace r
caller_v = 'caller-v'
call outer
say 'caller sees:' caller_v callee_w
say 'result after outer:' result
call bare
say 'result after bare:' result
target = 'DYNAMIC'
call (target)
say 'result after dynamic:' result
say 'caller digits still:' 1 / 3
exit

outer:
say 'outer sees:' caller_v
callee_w = 'callee-w'
do
  call inner
end
return 'outer-result'

/* NUMERIC is inherited at call time and never written back: this sets 7 for
   itself, and the main body's own division above still runs at 9. */
inner:
numeric digits 7
say 'inner digits:' 1 / 3
return

/* A bare RETURN drops RESULT rather than leaving the previous call's value,
   so the caller reads back the derived name RESULT. */
bare:
return

dynamic:
return 'dyn'
