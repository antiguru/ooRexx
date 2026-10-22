# Phase 9d Task 8.3 re-review, fix round 1

Range `1a9cbc4bf..9963948e3`, code commits `262b6c72c` and `3d9cc2da0`, records `54db40612` and
`9963948e3`. `4d5132093` is out of scope. Reviewer builds: BASE `efe8d3f40` and HEAD `9963948e3`,
each a scratch worktree with its own `CARGO_TARGET_DIR`, release, `CARGO_INCREMENTAL=0`. Probes ran
from one fresh directory under the scratch dir, oracle then base then head, with stdout, stderr and rc
compared separately. Crate runs used `memcap 2G timeout 60` and the oracle the standard wrapper. The
scratch dir and its worktrees are deleted.

**Verdict: APPROVED.** Critical 0, Important 0, Minor 1. Queue candidates: 1.

## 1. I1, M1, M2

* **I1: ADDRESSED.** `rust/bench-programs/callgrind.sh:47` lists `define`. At HEAD,
  `head/rust/bench-programs/callgrind.sh -B 0.5 -j 4 -p "intcall extcall fibfunc sendloop dispatch
  dirread interpret parse" base=<base rexx> head=<head rexx>` passes its PROGRAMS-versus-disk check and
  exits 0, where at `99fd7ae2b` it exited 2. That check runs before `-p` is applied, so it covers the
  whole list.
* **M1: ADDRESSED.** The report now limits the claim to primitives entered through a `NativeTail`
  begin half, and names RUN, SORT, HASITEM, REQUEST and PUT as still missing. My probe ran the live
  `.context~stackFrames` under `o~send`, `o~sendWith`, `Message~send`, `r~call`, `r~callWith`,
  `r~[]`, `NEW` and `sortWith`: HEAD matches the oracle on all of them, and BASE differs. The
  witnessed list in the sentence (NEW, SEND, SENDWITH, CALL, CALLWITH, `[]`) is true.
* **M2: ADDRESSED.** `git diff efe8d3f40..9963948e3 -- 'rust/crates/*.rs' 'rust/bench-programs/*' |
  grep '^+' | awk 'length > 101'` finds one line, a Markdown table row in
  `bench-programs/README.md` added by the task's first round. It is not Rust and was not one of the
  two named lines. `executable.rs:596-599` and `install.rs:2724-2727` are within 100 columns.
  `cargo fmt --all --check` exits 0. HEAD has since moved to `a1d15d34f`, which touches nothing under
  `rust/`.

## 2. `3d9cc2da0`: the parked call's keep in `Activity::parked_keeps`

**The design.** A keep is filed at `parked_calls.len()` when its call enters
(`run/call.rs:585-588`). Two things can claim it:

* `take_entered_keep` (`run/call.rs:617`), on the Rust-stack paths (`run_entered`, and the
  `native_park` refusals). This runs before the callee's body.
* `end_parked_keep` (`ir/drive.rs:4293`), once the parked call at that index has been popped. This
  runs in `resume_call` and in `resume_region`'s `Register` arm, and in `resume_woken` for `Register`
  or `Flow`.

Between the filing and `park_call` the driver does nothing more (`break 'entered` leads straight to
`park_call`). `parked_calls` is popped in exactly two places (`drive.rs:2906`, `2931`), and both go
to those resume functions.

**Invariant check (scratch build, release).** I added three `assert!`s to the scratch copy of HEAD:

* at filing, every filed index is `< at`, so no stale keep sits at or above the new one;
* at `park_call`, no filed index is above `parked_calls.len()`;
* at each resume, a `Send` or `Exec` parked call never has a keep at its index. Those are the arms
  where `3d9cc2da0` stopped looking.

**Results:**

* All 1185 corpus `.rex` programs, each from a fresh copy with its `.d` fixtures, under
  `timeout 30`: no assertion fired and nothing panicked.
* Every probe below, including the concurrent ones run 3 times each: no assertion fired.
* **Liveness:** with `resume_call`'s `end_parked_keep` deleted, `k1` and `k5` panic with
  `RRCHECK keep above park`. The check can therefore see the defect class.

**Behaviour (HEAD against the oracle):**

| probe | shape | BASE | HEAD |
|---|---|---|---|
| 10 witnesses of `t8.3/witnesses/summary.tsv` | as recorded | differs (as recorded) | identical, all 10 |
| k1 | `.ROUTINES` rebound between calls, function and CALL; first call raises 42.3, rebound, then succeeds; recursive `fact`/`fib` | differs (`g 2 raised 42.3`) | identical |
| k2 | `EXIT` in an external callee, `SIGNAL` inside a callee, a raise unwinding 4 internal levels, later sites at the same depth, all rebound | differs | identical |
| k3 | raising pinned call inside the callee of another pinned call, trapped between, both rebound | identical | identical |
| k4 | 4 activities, parked calls (callee on `guard on when`), `.ROUTINES` rebound mid-run; 5 runs | `165` x4, x5 | `165` x4, x5 |
| k5 | 3 activities: first call parks on a guard, then raises (forgets) or returns (keeps), function and CALL forms, rebound mid-run; 5 runs | never forgets (`raised` throughout) | forgets after the raise and keeps `old`, as the oracle does on every run (the exact row is schedule-dependent on the oracle) |
| k12 | first calls inside a `sortWith` comparator (nested drive under a native) raising past the native, then INTERPRET, rebound | differs | identical |
| m1 | live frames (section 1) | differs | identical |

**GC.**

* `cargo test -j 6 --release --no-fail-fast -p rexx-exec --test collect_stress --test compiled` at
  HEAD: `72 passed; 0 failed` and `2 passed; 0 failed`.
* A scratch test running `run_program_collect_every_alloc` over k1, k2, k3, k6 and the three
  call-site corpus witnesses gives the same output as the plain run for all except k6. That includes
  k6's first call, where the parked call's keep is the only thing holding the routine while the
  callee removes it from `.ROUTINES` and churns. k6 fails after its first call returns, and fails
  identically at BASE, so the cause is not `parked_keeps`. See Q-A.

**Growth.** `g1.rex`, maxrss at 10000 and 100000 iterations. The pinned, raise (a keep filed and
forgotten every call), nested and INTERPRET (a fresh site, so a fresh keep, every iteration) shapes
give HEAD 19.6/26.2, 37.2/37.3, 19.9/26.0 and 21.3/26.0 MB. BASE gives 19.6/26.5, 36.0/37.1,
19.6/25.2 and 21.1/26.8 MB. Nothing grows past the collector's plateau, and HEAD tracks BASE.

## 3. Perf

`callgrind.sh` from the HEAD tree, run as in section 1, against my own `efe8d3f40` build. Exit 0,
3 rounds, every spread 0.0000-0.0001%.

```
program    whole d%   -libc d%
intcall    -0.0797    -0.1002
extcall    -0.4176    -0.4389
fibfunc    -0.1803    -0.1914
sendloop   -0.0343    -0.0361
dispatch   -0.0226    -0.0242
dirread    -0.0994    +0.0455
interpret  +0.2601    +0.0235
parse      -0.0719    -0.0774
```

Every libc-subtracted delta equals `r1/callgrind-final.txt`'s to four decimals. The largest rise is
dirread at +0.0455%, which is within +0.05%. Whole-process deltas differ from the record for some
programs, because the BASE whole-process counts differ, for example fibfunc 8602M here against
8612M there. That is libc, and the libc-subtracted column is the one judged.

## Findings

### Critical
None.

### Important
None.

### Minor

**M1. The stale-keep tripwire went with `entered_keep`.** `3d9cc2da0` deleted the
`debug_assert!(entered_keep.is_none(), "an entered call's keep was left unclaimed")` in
`site_call` (`run/call.rs:585-588`) and put nothing in its place. Under the index scheme a keep left
behind is no longer just a leak. `take_entered_keep` or `end_parked_keep` would hand it to the next
call op at the same `parked_calls` depth, which would `remember` it into that op's chunk.
`end_site_keep` uses the claiming chunk's `call_site_slot(keep.site)`, so the routine would be
written into another chunk's site.

No path leaves one today: section 2's assertions never fired over the corpus and the probes, and a
mutation shows they fire when one is left. Fix: a `debug_assert!` at the filing,
`self.activity.parked_keeps.last().is_none_or(|(a, _)| *a < at)`. It is free in release and is
caught by the debug corpus gate.

## Queue candidates

* **Q-A. A site's kept routine is not rooted once nothing else holds it, so the collector frees it.**
  The next call through the site then fails loudly. The probe:

  ```rexx
  .local~rr = .routines
  .routines['F'] = .routine~new('f', 'use arg n; .rr~remove("F"); return "f" n')
  do n = 1 to 3
    say viaf(n)
    do i = 1 to 200000; x = .array~new(20); end
  end
  exit
  viaf: return f(arg(1))
  ::routine dummy
    return 0
  ```

  The oracle prints `f 1 / f 2 / f 3`, rc 0. The crate prints `f 1`, then
  `rexx-exec: an activation's body selector names no routine body`, rc 120. The plain release binary
  fails the same way at HEAD, at BASE `efe8d3f40` and at `eeab7f128` (before Task 7.1), and so does
  the site in the main program (`say f(n)`) and in the CALL form. Holding the routine in a variable
  as well (`hold = .routines['F']`) makes it pass, so the cause is the site slot that `remember`
  writes, which the collector does not trace. While the call runs the keep's pin holds the routine,
  so `parked_keeps` is not the cause. Not introduced by Task 8.3. Its outcome is a loud refusal
  rather than a wrong body in the one variant I tried, which made new routines after each collection.
