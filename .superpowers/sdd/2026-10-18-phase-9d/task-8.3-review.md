# Phase 9d Task 8.3 review

Range `efe8d3f40..99fd7ae2b`. Reviewer builds: BASE `efe8d3f40` in a scratch worktree, HEAD
`99fd7ae2b`, both `--release` with their own target dirs, plus a HEAD debug build (and a BASE debug
build for one control). Probes ran from fresh directories under
`t83-review-scratch/runs/` (deleted at the end), each side's stdout, stderr and rc compared
separately. Crate runs `memcap 2G timeout 60`; oracle under the standard wrapper.

**Spec compliance: COMPLIANT.**
**Quality: CHANGES REQUESTED** (one Important finding, a broken bench script; small fix).

## 1. Closed items: witnesses red at BASE, green at HEAD

All ten new `corpus/lang` witnesses run on all three sides (not a sample):

```
call_site_reentry_reads_after_arguments  base: out          head: identical
call_site_reentry_raise_keeps            base: out          head: identical
call_site_routine_raise_forgets          base: out          head: identical
routine_copy_internal_executable         base: out          head: identical
main_executable_annotations              base: out err rc   head: identical
instance_method_after_define             base: out          head: identical
enhanced_unguarded_copy_executable       base: out          head: identical
raise_syntax_string_additional           base: out err rc   head: identical
routine_call_frame_names (+ .d/sub)      base: out          head: identical
context_name_install (+ .d files)        base: out          head: identical
```

Those columns match `t8.3/witnesses/summary.tsv`. I read the oracle stdout for the paths each one
claims. Re-entry: `fn 0 x answered 2` against base `raised 98.941`. Q6: `fn-e 1 3 answered 3`
against base `raised 98.941`, in all four forms. Raise-forgets: `2 answered good` against base
`2 raised 41.1`. Install: `9 ::CONSTANT 0`, `prolog 0 [] 0 q.cls` and `pk 0 pk` against base `9  0`,
`prolog 1 [zz] 1   SUB` and `pk 0 DIR/...`.

`collect_stress` and `compiled` at HEAD: `72 passed; 0 failed` and `2 passed; 0 failed`
(`cargo test -j 4 --release --no-fail-fast -p rexx-exec --test collect_stress --test compiled`,
exit 0).

Queued files: every closed item's first line is `**CLOSED 2026-10-22 by Phase 9d Task 8.3
(68e7f15d5).**`. The 9b1 minors file stays OPEN, with a line naming m1 as Task 8.5's, so its "lists
Z" minor is untouched. In `queued-inventory.tsv` columns 4 and 6 are `closed:68e7f15d5` on every
closed row, and the diff shows the reason column unchanged. `gate:phase-9c-gate.md:143` is in
`t8.3/closed.tsv`.

## 2. The three deletions from `phase-4-exclusions.txt`

The diff to that file only deletes lines. Nothing licensing was added or reworded, and no other
file under `docs/superpowers/plans/` changed.

* **Known gap beside Deviation 41.** Each side translated `f.rex` with its own `rexxc`. Then
  `.Routine~newFile('f.rxo')~call` and `call 'f.rxo'`:
  oracle `name DIR/f.rex / ROUTINE DIR/f.rex / METHOD CALL / ... / name f.rxo / ROUTINE f.rxo`,
  HEAD identical on all three descriptors, BASE `name CALL / ROUTINE CALL`. Deletion is correct.
* **"A ::routine whose call raised stays kept".** The implementer's
  `probes/routine_raise_addroutine.rex`: oracle `pass 1 raised 42.3 / pass 2 added`, HEAD
  identical, BASE `pass 2 raised 42.3`. Deletion is correct.
* **"A library routine's Routine~call traceback also lacks ...".** Trapped
  (`probes/library_routine_call_traceback.rex`) and untrapped (my
  `loadExternalRoutine('myname', 'LIBRARY orxfunction TestRaiseException0')~call('x')`, rc 168):
  BASE, HEAD and oracle are identical in both, `Compiled method "CALL" with scope "Routine".`
  included. So the entry was already false at BASE, and the deletion is correct.

## 3. Concern 6: primitive frames

**Condition frames** (`p10`): a raise under NEW, SEND, SENDWITH, SORTWITH, SORT, RUN, CALL,
CALLWITH, REQUEST, Message~SEND, HASITEM and an `enhanced` method. HEAD is identical to the oracle
in traceback text and `condition('o')~stackframes`. BASE differs only in `ROUTINE CALL` for
`BOOM`, so condition frames already listed SORT, RUN, REQUEST and HASITEM at BASE.

**`.context~stackFrames`** (`p9`): HEAD now lists `METHOD NEW` under INIT, `METHOD SEND` (Object and
Message) under a sent method, `METHOD CALL` under a routine run from inside a `send`,
`METHOD CALLWITH`/`[]` for the main program's executable (`p11`), and the frame names. All of these
match the oracle. No primitive frame appears on HEAD that the oracle lacks: `forward`, `~start`
plus `result`, and an `enhanced` method all agree. The reverse does occur, at BASE and HEAD alike.
Live frames lack five primitives the oracle lists:

```
< run METHOD | RUN | 1            < sort METHOD | SORT | 0
< eq METHOD | HASITEM | 1         < makearray METHOD | REQUEST | 1
< hash METHOD | PUT | 1
```

This predates the task (Minor M1, and queue candidate Q1).

Main-program `~call`/`~callWith`/`~[]` tracebacks, trapped and untrapped, and a main executable
called from inside a `::routine` (`p11`): identical to the oracle.

## 4. Concern 1: the pruned `NO_ALLOCATION_PROGRAMS`

In the BASE scratch worktree I changed collect_stress's divergence `assert!` into an `eprintln!`
(scratch only) and ran `the_l0_subset_passes_again_under_collect_on_every_allocation`. The zero-collection
set BASE observes (`left:`) is exactly HEAD's pruned list. So every dropped program
(`call_trace_nested`, `do_ctrl_class`, `do_to_class`, `message_assignment_form`, the four
`message_send_*`, `mutation_controlled_order`, the three `trace_*`) already collected at BASE, and
the drift predates the task. The BASE run also shows the two Task 7.1 programs red
(`call_site_routine_copy_lifetime`, `call_site_fresh_routine_raises`), which HEAD fixes. The task
did not establish when the drift began, or what the +4 collections in the traced programs are
(queue candidate Q2).

## 5. Call-site pinning, adversarial probes (HEAD identical to the oracle on every one)

| probe | shape | BASE |
|---|---|---|
| p1 | `.ROUTINES['F']` rebound between calls (oracle keeps `one` forever) | identical |
| p2 | first call raises 42.3, `.ROUTINES` rebound, then succeeds: `fn 1 two 1` | **differs** (`fn 1 1`) |
| p3 | argument re-enters the same site through recursion and rebinds it, function and CALL forms | identical |
| p4 | recursive `::routine`s (fact, fib) | identical |
| p5 | `Routine~call`/`callWith`/`[]` of a copy, `.context~executable` identity, a raising copy's frames | differs (names) |
| p6 | unresolved `foo()` (43.1), then `addRoutine`, then calls | identical |
| p7 | routine answering nothing to a function call (44.1, kept), then rebound | identical |
| p8 | external file rewritten between calls, then `.ROUTINES` entry | identical |
| p14 | 4 threads, sites in a method body, `.ROUTINES` rebound every 500 calls; 5 runs each side | identical, `sum 54030000` x5 |
| p15 | callee parks on `guard on when` (released by another activity), kept / raised; 5 runs | differs; HEAD identical x5 |
| p16 | the p2 shape under `TRACE R`, and INTERPRET re-entry | differs (out, err) |

`collect_stress` at HEAD: 72 passed (above).

A debug build of HEAD over all corpus `.rex` programs (fixtures copied, stdin `/dev/null`,
`timeout 30`) produced no panic and no `debug_assert` failure. The `entered_keep` "left unclaimed"
assertion never fired. One program could not be judged: `lang/call_depth_cap_call.rex` passes
400 s in debug on HEAD **and on BASE**. In release it takes 0.11 s at BASE and 0.14 s at HEAD.
gdb puts the debug run in `dispatch::context::executable_at` recursing through `Interp::frame_at`'s
`nth` (`context.rs:496`, `activation.rs:1193`), which is quadratic in depth and predates the task
(queue candidate Q3).

## 6. Perf

Task 8.2's binary could not be rebuilt identically here, so I compared my own BASE and HEAD builds.
Using BASE's copy of the script (see I1), `callgrind.sh -B 0.5 -j 4 -p "parse churn startup"`:

```
parse    +0.0372 whole   +0.0387 libc-subtracted
churn    -0.0166         +0.0266
startup  +0.0016         +0.0008
```

These agree with `callgrind-2.txt` (parse +0.0368 / +0.0386, churn +0.0104 / +0.0265) and are
within +0.05%.

`8d409845b` builtin path: `missed_site` answers `None` for `Resolved::Label` and
`Resolved::Builtin`, and `site_resolution_before_arguments` settles only those two kinds before the
arguments run (`resolve_fixed_call`). So a builtin never carries a keep, and the hot path is the
old one. The cold `begin_kept_call_args` lacks the builtin shortcut, but it only runs with a keep.
Builtin shapes exercised: function and CALL forms inside the probes above, and under TRACE (p16).

## 7. Memory

`p13`, maxrss at 2000 / 200000 / 600000 iterations, HEAD and BASE:

```
f / c / rexx / copycall (pinned copy site): 19.6 MB flat
raise (raising call through a pinned copy): 36.7 -> 37.8 -> 38.0 MB, base 36.7 -> 37.1 -> 38.3
rebind (new copy into .ROUTINES every call): 25.1 -> 50.4 -> 49.6 MB, base 25.4 -> 50.1 -> 49.8
```

Nothing grows past the collector's plateau, and HEAD tracks BASE.

## Findings

### Critical
None.

### Important

**I1. `bench-programs/callgrind.sh` no longer runs at HEAD.** `8d409845b` adds
`rust/bench-programs/define.rex` (and registers it in `rexx-bench` and the README), but not in the
script's `PROGRAMS` list (`rust/bench-programs/callgrind.sh:47-49`). Every invocation now exits 2
before measuring anything:

```
$ rust/bench-programs/callgrind.sh -B 0.5 -j 4 -o ... -p "parse churn startup" base=... head=...
PROGRAMS and /home/moritz/dev/repos/ooRexx-rust-rewrite/rust/bench-programs/*.rex disagree:
8a9
> define
(exit 2)
```

The per-task perf check in `global-constraints.md` runs this script, so Task 8.4 cannot run its
mandated check as the tree stands. The report's `callgrind-2.txt` was taken before the file
existed: `cg-9d/t8.3-run3` is dated 14:23-14:25, while `define.head.cg` is 14:26 and the commit
14:41. Fix: add `define` to `PROGRAMS`. `rexx-bench`'s lib test (30 passed) does not cover the
script.

### Minor

**M1. The report's frame claim is broader than what was built.** "A sent primitive method that
entered a Rexx body is now listed by `.context~stackFrames` and in a condition's frames ... This is
general, not `CALL` alone" (task-8.3-report.md, "What changed").

* Live frames still omit RUN, SORT, HASITEM, REQUEST and PUT, where the oracle lists them (section 3).
* Condition frames already listed those at BASE.

What changed is the primitives entered through a `NativeTail` begin half. The sentence should say
that, or be deleted. The items' witnesses are unaffected.

**M2. Over-width comment lines that rustfmt will not wrap.**
`rust/crates/rexx-exec/src/dispatch/executable.rs:596` (116 columns, `begin_routine`'s doc) and
`rust/crates/rexx-exec/src/install.rs:2724` (104 columns, the `::CONSTANT` comment). Found with
`awk 'length($0) > 100'` over the added lines.

## New oracle defects
None found.

## Queue candidates (all predate the task)

* **Q1.** `.context~stackFrames` omits the primitive method's frame for primitives that run a Rexx
  body nested rather than through a begin half: `Object~run`, `Array~sort` (compareTo),
  `Array~hasItem` (`==`), `Object~request` (makearray), `Set~put` (hashCode). The oracle lists
  `METHOD RUN | 1`, `METHOD SORT | 0`, `METHOD HASITEM | 1`, `METHOD REQUEST | 1`,
  `METHOD PUT | 1`. Condition frames already agree. The probe is `p9` in section 3.
* **Q2.** The collect_stress zero-collection drift: when the dropped programs began allocating
  (the implementer traced `do_to_class`'s first to `deliver_trace_line`'s `TraceObject`), and what
  the +4 collections in `call_trace_nested`, `trace_output`, `trace_results` and
  `mutation_controlled_order` are (`t8.3/probes/collections-base-head.txt`).
* **Q3.** `dispatch::context::executable_at` recurses one depth at a time, and each level calls
  `Interp::frame_at`, which walks the frame chain with `nth`. That is quadratic at the activation
  depth cap: `lang/call_depth_cap_call.rex` does not finish in 400 s in a debug build, at BASE or
  HEAD. A debug corpus gate (rust/CLAUDE.md) will time out on it.
