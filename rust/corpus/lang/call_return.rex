/* CALL and RETURN (4b Task 3): the shared variable pool, RESULT, the
   settings a callee inherits, and the indent a callee's clauses echo at.

   Under `trace r` throughout, because half of what this program pins is in
   the trace and not in the output. Each activation's clauses echo two spaces
   further in than the clause that called it -- `call inner` sits inside a DO
   inside `outer:`, so its callee echoes at 6, which is the case that tells
   D2r's rule apart from `2 x depth`.

   The DO block here is a plain block and never a repetitive one, deliberately:
   phase-4-exclusions.txt's KNOWN GAP row is about a repetitive DO/LOOP that
   completes a body pass and then ends on a failing control test, which moves
   every later clause's indent by two. A program pinning indents cannot also
   contain one.

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
