# Stackless call-path spike (2026-09-29)

Spike for Phase 6's "activities as continuations" candidate. Throwaway worktree at 0b698dff2; the
patch is `stackless-spike.patch` beside this file (27 files, +1227/-219 with instrumentation). Tags:
[M] measured, [R] read, [I] inferred. Transcribed by the controller from the spike agent's reports.

## Feasibility [R]
Internal CALL, internal function calls, `::ROUTINE` calls and plain sends to a `::METHOD` run and
return in one Rust frame: `run_chunk` became a trampoline (`drive`) that parks the caller and
resumes it mid clause region; `invoke_call_over` and `enter_method_body` split into begin/finish
halves (the recursive path is the same halves composed, kept as the pinned route); a new `Op::Send`.
RESULT, traps, EXIT, SIGNAL and TRACE indentation unchanged: state mutations happen in the same order.
Resisted: resume mid clause region needs a per-clause test; calls in loop headers, non-flattened
loops, INTERPRET and nested-call arguments stay recursive; the parked stack lives in a `drive` local
(register frames borrow the arena; switching activities needs offset handles). A receiver left rooted
in its register broke 3 corpus programs (collection/UNINIT timing) until cleared.

## Correctness [M]
Corpus gate 655/655 on HEAD and spike (release and debug); ooTest API groups 23/23 both;
collect_stress 32/32. Five structural test failures (source scans, counters, op-shape tables), no
behaviour change.

## Performance [M] (callgrind Ir, libc/ld-linux subtracted, separate target dirs)
| | layout control | spike | spike, calls forced recursive |
|---|---|---|---|
| rexxcps | +0.77% | +4.51% | +4.03% |
| emptyloop | -0.27% | +3.49% | +3.49% |
| fibcall(22) | +0.52% | +10.3% | +5.5% |
| sendloop (2M sends) | +0.31% | +7.8% | +0.6% |

Per call, stackless minus recursive in one binary: +469 (CALL), +488 (function), +520/+585 (sends).
`Op::Send` alone is cheaper than today's send path (-157/call on fibsend). With the resume branch
compiled out emptyloop is -2.4%: the static cost is the per-clause resume test. Per-call breakdown
(fibcall): struct copies ~215 (artefact), `run_ops_from` ~180, `run_activation`/`drive` ~190.

Recursion depth, cap lifted: HEAD dies of Rust stack overflow (SIGABRT, rc 134) at 89,081 function
calls / 90,224 CALLs / 72,821 sends; the spike reaches 1,000,000 of each (~1.1 KB per call level,
~2.05 KB per send level), stopped only by a 4 GB cap. With the cap, both stop at 9,999 (error 11).

## Pinning inventory [M]
METHOD group: 9,161 method bodies entered by recursion, 687 stackless; 7,130 of the 9,161 are
instruction-form sends (`Op::Message` -> `exec_message` -> tree `message_term`), a coverage gap, not
structural. After converting them, genuinely native re-entries in METHOD: INIT 354, FORWARD 336,
UNKNOWN 319, `Object~send` 319, library-program entry 324 (~1,650). Corpus totals (607 programs):
tree-evaluated expressions 345, `~new`->INIT 343, tree-evaluated calls 241, CALLs 178, sort
comparators 176, conversions 156, UNKNOWN 73, `::ROUTINE` install 72, ADDRESS WITH 70, trap
handlers 67, operators 54, FORWARD 51; driver-nested calls 19. Site list: `stackless-reentry-sites.txt`.

## Verdict
(a) The +3.5% static cost is removable [M bound, I remedy]: one resume test per `run_ops_from`
entry via a separate `#[inline(always)]` resume entry, not per clause. Clause-boundary-only resume
does not work for expression calls (`x = f(1) + 2`). The countdown path does not help.
(b) Per-call cost is the leave-and-re-enter design [M breakdown, I estimate]: a CPython-3.11-style
in-loop frame swap (callee frame pushed, dispatch continues in the same loop iteration) is estimated
at a few dozen Ir for the swap, parity with recursion or better (the callee's driver entry disappears). Not built.
(c) Instruction-form sends are a coverage gap [R]; `~new`->INIT would then dominate pinning unless
`~new` gets the begin/finish split, else carrier fallback becomes common.
Overall: viable; as built no win; needs the in-loop design, arena-resident parked state with offset
handles, measured steps.
