# S1 front-end round 1 review (2050300f4..92ac5c054)

Reviewer: p6-perf1-review. Read-only. Builds: git archive of both shas, separate target dirs,
`cargo build --release -p rexx-exec --bin rexx-run` (Compiling line present in both logs).

## Progress

- Read brief, report, diff (dispatch.rs, ir.rs, compile.rs, invariants.rs, corpus_shape_tests.rs,
  drive.rs, golden.rs, valid.rs).
- Op::Clause uses at 92ac5c054 (non-comment, non-emission): valid.rs:60, golden.rs:52/351,
  drive.rs:253 (undriven name), :1601 (region_ops cold arm), :2872 (grant_for), :3008 (ops_loop),
  invariants.rs:26/212/233/263, corpus_shape_tests.rs:67/271. Each has a CallingClause
  counterpart. No other file matches the IR `Op` enum in code (other hits are comments or
  `FailureSite::Clause`). `size_of::<Op>() == 16` is a const assert at ir.rs:61.

## Verdict

No Critical or Important findings. One Minor (a false sentence in the report). Every probe
that differs from the oracle differed the same way before the round; new == old on everything run.

## Findings

### Minor

M1. perf1-report.md, "Why round 1 did not move sendloop's wall clock", first sentence: "Round 1
removed the second region expansion and the granting instance from the per-send path". It removed
`resume_region`'s expansion, but `ops_loop` now holds two region expansions (`clause_region!` at
drive.rs:3026 for `Op::Clause`, :3156 for `Op::CallingClause` and resumed regions), and a call
whose callee body has a plain clause runs both per call: fibfunc's `if k < 2 then return k` and
dispatch's `total = total + 1` are `Op::Clause`, the calling clauses `Op::CallingClause`
(`bench-programs/fibfunc.rex`, `dispatch.rex`). sendloop's empty `ping` is the case where only
one runs. The brief's "one expansion on the per-send and per-call path" is therefore not what
landed for dispatch/fibfunc; the Batch 4 paragraph describes the design correctly. Fix: delete
"the second region expansion and" from that sentence.

## What was checked, by running

Binaries: `git archive` of 92ac5c054 (new) and 2050300f4 (old), touched, separate target dirs,
`cargo build --release -p rexx-exec --bin rexx-run`, one Compiling line each. Oracle as briefed,
from a fresh empty dir per run; stdout, stderr (temp path normalised) and rc compared separately.
Probes and runner: `$S/probes/`, `$S/cmp.sh` (S = scratchpad/p6-perf1-review).

1. `Op::Clause` sites. Every code match has a `CallingClause` counterpart (list above). All 18
   `Op::Clause` emission sites in compile.rs go through `close_region`, so classification is
   total. Script over golden_tests.rs: 149 regions, 9 CallingClause, 140 Clause, 0 where the kind
   disagrees with "region holds CallExpr/CallArgs/Send"; the 9 changed golden lines are exactly
   those 9. `Call`/`CallNamed` (Deliver::Flow) stay `Op::Clause` and still park, through
   `clause_region!`'s `'park` in either expansion; resume of those is `resume_call`, unchanged.
2. First-instruction permission, 38 probes (p00-p23, r01-r16) against oracle and old: PROCEDURE
   first after CALL / function call, labels before PROCEDURE, EXPOSE list, NOP/CALL/send/`x = f()`
   /`interpret ''`/`if 1 then procedure`/`do; procedure` before PROCEDURE, `interpret
   'procedure'`, fall-through, recursion (function and CALL), SIGNAL to a label followed by
   PROCEDURE, `signal sub` re-running PROCEDURE, SIGNAL ON SYNTAX and CALL ON USER handlers whose
   first clause is PROCEDURE, ::ROUTINE with PROCEDURE, method with PROCEDURE, EXPOSE/USE ARG
   ordering, USE LOCAL first/not-first in routine/method/main, `~new` INIT with EXPOSE first and
   INIT whose first clause is a send. All identical to old; all identical to the oracle except the
   pre-existing translation-time divergences (99.907, 47.2, 99.910 are rc 120 `rexx-exec:` here
   and 157/209 in the oracle; USE LOCAL in a method is Loud) which old has byte for byte.
   `grant_for`'s `None` fallback is reached (gdb: `ops_loop_granting` hit by q01, a `CALL` to a
   label that ends the body) and matches the oracle.
3. Calling regions: trace R/I over calls in expressions, DO header values from calls (`to`, `by`,
   count, `for`, `while`), SELECT/IF conditions, SAY/PARSE VALUE/QUEUE/`call f f(2)`/SIGNAL VALUE,
   SIGNAL ON NOVALUE after a resumed call in the same clause (SIGL), CALL ON USER raised inside a
   function, errors after a resumed call (41 line numbers), INTERPRET with calls and sends,
   ITERATE/LEAVE after calls, DO OVER. Identical to oracle and old. Interactive `trace ?r` with
   `=` re-executing a send clause (stdin piped): new == old (both differ from the oracle only in
   stdout/stderr interleaving and the temp path).
4. Differential new vs old over rust/corpus/lang and gate-tables (977 programs), plain and with
   `trace i` prepended: 0 differences in stdout+stderr+rc.
5. Stack: gdb `$sp` at `resume_region` entry, hits 1/941 at depth 950, for function recursion,
   method recursion, `~new` INIT recursion and `'abc'~length + g(n-1)`: identical at every depth
   (new 0x...2a40, old 0x...22e0). A program issuing its second call from a resumed region
   (`0 * zero() + r(n - 1)`, same for sends): `$sp` identical at hits 5/1700/2700; ops_loop_steady
   entries sampled at hits 5/1500/3000/5000 read three distinct values by call site, none growing. No regained recursion.
6. Pinning: `cargo test --release -p rexx-exec --features pinning --test concurrency_tests --
   measured::` on both trees (ootest symlinked): 8 passed each; `pinning-table.md` byte-identical
   old vs new (270 lines, arrivals 1624, 0 beyond TreeEval/TreeSend/OpExec).
7. begin_invoke: diff of old `begin_invoke` against new `begin_invoke` + `begin_invoke_other` is
   the Rexx arm hoisted above the match unchanged, the non-Rexx/non-native arms moved verbatim,
   and an unreachable `Rexx | Native => Loud::missing_body()` arm. No behaviour change.
8. Permission leak: steady now takes `procedure_permitted` where it used to assert it false. The
   only statements between the grant and the take are `count_clause_against_deadline()?`
   (Failure::Deadline, which ends the run) and `chunk_map_too_short`, so no permission can be left
   set for a later clause.
9. perf doc section: checked the Ir claim ("inside +0.3% of base everywhere except fibcall and
   fibfunc") against its table: true. Wall verdict lists sendloop, fibfunc, dispatchclass, rexxcps,
   fibcall; arith (+7.32%) and textnum (+5.56%) are also over 4% but the sentence does not claim
   completeness. Nothing false found there.

Target dirs deleted after the run.
