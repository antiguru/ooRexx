# Task 6.6 re-review, round 2

Scope: `7d0660c87..0bed34005`. The code change is `install.rs` only:
* `install_executable` sets `install_caller` to the running activation's settings;
* `package_from_program` passes those settings to `run_loaded`.

I built HEAD (`rexx`, `rexxc`) into `scratchpad/rerev66r2-target`. I built round 1 (`7d0660c87`) from
a detached scratch worktree into its own target dir, to tell new divergences from old ones. Every
probe ran each side in a fresh `mktemp -d` copy, and stdout, stderr and rc were compared separately
(paths replaced by `DIR`). The probes were in `scratchpad/rr66probes.l3td/` (the round-1 set) and
`scratchpad/rr66r2.yF9x/` (new). Neither is durable, so the deciding lines are quoted.

## Round-1 findings

1. **Executables drop the requiring package's own `::OPTIONS`: closed.** r2, r4, r5, r6, r7 and r8
   now match the oracle. For example:
   * r8 (`.routine~newfile` of an options-less file requiring an inheriting file) gives `prolog 9`;
   * r5 (`.routine~new` with `::options digits 6`) gives `prolog 6` / `r 6 0.666667`;
   * r2 gives `prolog 9` / `rtn 9 0.333333333`.

   The inheriting shapes r1 and r3 still give `prolog 4`, as on the oracle.
2. **`.Package~new(name, source)` inheriting: closed.** l3 gives `z 4` / `src 4` / `made` / `z 9` /
   `src2`, identical to the oracle.
3. **Required-file `::CONSTANT` error names the requiring file: still open, as queued.** This is old
   minor 3 (t6, t6b, t6c, t8), and round 1 gives the same output.

Both new witnesses (`class_constant_expression_runtime_requires`,
`class_constant_expression_numeric_inherit`) match the oracle byte for byte when run from a copy of
their `.d/`. `cargo test --release -p rexx-exec --test directive_options --test package_requires`
exits 0, with 24 and 24 passed.

## The round-1 probe set, rerun

All of it matches the oracle except t6, t6b, t6c and t8 (finding 3 above):
* prologues p1-p7;
* caching q1 and q2;
* nested requires n1;
* external CALL at defaults e1-e4;
* INTERPRET i1;
* `.Package~new(name)` and `loadPackage`: l1, l2 and l4;
* condition and TRACE settings t1-t5 and t7;
* constants f1a-f1c;
* runtime loads r1-r8 and l3;
* the leading-label set L1-L12.

## New hunt: what "install with the running activation as caller" might leak

These match the oracle on HEAD (in stdout, stderr and rc):
* **Created under one setting, run under another.** x1: a routine created under DIGITS 4 with an
  inheriting own constant, then called under DIGITS 9. It gives `body 9 0.3333`, so the constant
  is fixed at creation and the body inherits at call time. x2: `.routine~newfile` created inside a
  procedure under DIGITS 5, then called in main under 9. It gives `prolog 5` / `body 9 0.33333` /
  `main 9`.
* **A method created in one activation and run in another.** m1: `.method~new` created under 4,
  then run from a method under `numeric digits 7`. It gives `7 0.3333`.
* **Condition traps.** The caller has `signal on novalue`, and an executable's constant reads an
  unset variable. It gives `body NV` and the trap does not fire (c1). An executable's own
  `::options novalue syntax` constant under the caller's `signal on syntax` gives `trapped syntax
  SYNTAX 98 3` (c2). A prologue's novalue under the caller's trap is not trapped (c3). A prologue's
  `1/0` under `signal on syntax` gives `trapped syntax 42 3`, with `prolog 9` (c4). In ps2, the
  caller's `signal on novalue` does not reach a `.Package~new(name, source)` prologue.
* **TRACE.** `trace r` in the caller does not trace a newFile's required prologue or constant (tr1).
  It does not trace an executable's own constant (tr2) or a `.Package~new(name, source)` prologue
  (ps2) either. The trace lines are identical.
* **Error lines.** A prologue syntax error (er1, rc 221) and a prologue 41.1 (er2, rc 215) under
  `.routine~newfile` give identical tracebacks: `*-* Compiled method "NEWFILE" with scope
  "Routine".`
* **An executable built inside a constant expression during an install** (re-entering
  `install_caller`):
  * ce1 is a package with `::options digits 5` whose constant is `.routine~newfile(...)~call`,
    under `numeric digits 3`. It gives `prolog 5` / `5 0.33333`.
  * ce2 is an inheriting callee whose constant builds and calls an inheriting `.routine~new`. It
    gives `3 0.667 3`.
* **`.Package~new(name, source)` controls.** ps1 shows inheriting, options-less and DIGITS 6
  sources. They give `src 4 0.6667`, `src2 9 0.666666667` and `src3 6 0.666667`.
* **`loadPackage`.** lp: an inheriting file that requires an inheriting file, under 4, gives `z 4` /
  `prolog 4`. lp2: an options-less file, which requires an inheriting one, gives `z 9` / `prolog 9`
  / `0.666666667`.

## New findings

### Minor (old, to queue; round 1 gives the same output)

1. **`.context~name` in a constant expression and in a required prologue.**
   * cx1: an executable's `::constant k (digits() .context~name)` gives `4 ::CONSTANT` on the
     oracle and `4 ` (empty name) on HEAD. Round 1 printed `9 ` there, so round 2 fixed the digits,
     and the name was already wrong.
   * cx2: a prologue of a file required by a `.routine~newfile` package prints
     `.context~name~right(5)`. The oracle gives `prolog 9 q.cls` and HEAD `prolog 9 n.rex` (the
     main file's name). Round 1 gives `prolog 4 n.rex`.

   Neither is caused by this round. Both name the wrong context in an install activation.
2. **An executable's own constant error names the creating file**, the same class as finding 3
   above:
   * er3 (`.routine~new("x", ...)` with `::constant k ('a' + 1)`): the oracle gives `Error 41
     running x line 4`, and HEAD `Error 41 running DIR/main.rex line 4`;
   * er4 (`.routine~newfile("r.rex")`): the oracle gives `running r.rex line 4`, and HEAD
     `running DIR/main.rex`.

   The traceback lines and rc 215 match, and round 1 gives the same output. Queue it with
   finding 3 and review Concern 4.

## Verdict

**APPROVED.** Round-1 new finding 1 and minor 2 are closed, and the round-1 probe set matches the
oracle except the queued finding 3. The leak hunt found nothing new: every divergence it found
predates round 2, and round 1 gives the same output for each.
