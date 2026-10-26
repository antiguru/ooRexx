# Phase 9d Task 8.3 re-review, fix round 2

Range `9963948e3..ec13c3792`. Code commits `2afe1e16c`, `3e6dc0426`, `1bb283e1a`, `03a2a1e1a`.
`2041eb468` and `d6dee4ef4` are superseded in place, and `03a2a1e1a..ec13c3792` touches nothing
under `rust/`. Reviewer builds, each a scratch worktree with its own `CARGO_TARGET_DIR`, release,
`CARGO_INCREMENTAL=0`:

* RED `9963948e3`
* BASE `efe8d3f40`
* HEAD `03a2a1e1a`
* INST: HEAD plus invariant checks (below)
* MUT: HEAD with mutations
* debug builds of HEAD and of a MUT

Probes ran from fresh directories under the scratch dir, in the order oracle, red, head, with stdout,
stderr and rc compared separately. Crate runs used `memcap 2G timeout 60` (`timeout 120` for growth)
and the oracle the standard wrapper. The scratch dir and its worktrees are deleted.

**Verdict: APPROVED.** Critical 0, Important 0, Minor 2. Queue candidates: 1.

## 1. Q-A: a site's kept routine is traced for as long as the site holds it

**The design.** A site keeping a `Routine` of another program (`installed.program != holder`,
`run/call.rs:570-579`) records a `SiteHold`. The holder's package then counts the held program's
package in `PackageData::site_held`, which `trace` follows as an edge (`package_state.rs:141`). The
count drops when the site keeps something else (`CallSite::set`, `ir.rs`) or when an `INTERPRET`
fragment's chunk ends (`run/interpret.rs:89`, `unpin_chunk_copies`). There is one engine
(`run.rs:533-534`, "no second engine"). "Both engines" is therefore read as the IR driver's parked
path plus the Rust-stack nested drive (a `sortWith` comparator, `Routine~call`). Both are covered.

**Forms, oracle against RED and HEAD.** Each probe removes its routine's `.ROUTINES` entry in the
routine's first call, then churns 200000 to 400000 allocations between calls:

| probe | site | RED | HEAD |
|---|---|---|---|
| qa | r1's probe, internal function | `f 1`, then "an activation's body selector names no routine body", rc 120 | identical (`f 1/f 2/f 3`, rc 0) |
| main | function call in the main program | rc 120 | identical |
| call | `CALL f n` | rc 120 | identical |
| method | a `::METHOD`'s site | rc 120 | identical |
| sortw | comparator under `sortWith` (nested drive) | rc 120 | identical |
| interp | a site in the main program reached after `INTERPRET` | rc 120 after `g 1` | identical |
| interp2 | **the fragment's own site**, looping inside the fragment | `f 1 1`, then rc 120 | identical |
| other | `::ROUTINE h` of a `.package~new` package, `pkg` dropped | `h 1`, then rc 120 | identical |
| cycle | report's `forms/cycle.rex` | rc 120 after `a 0 / a 0` | identical |
| cyclepkg | two `.package~new` packages whose `::ROUTINE`s' sites keep each other's, both packages and table entries dropped, only `.ra` kept | `a0 b0`, then rc 120 | identical (6 lines) |
| viartn | the site in a `Routine~new` routine's code | rc 120 | identical |
| viameth | the site in a `Method~new` method `define`d on a class | rc 120 | identical |
| viaext | the site in a `::ROUTINE` reached through `findRoutine~call` | rc 120 | identical |

The `interp2` shape (it corrects the report's concern 1, which says the fragment's own site has no
red witness):

```rexx
.local~rr = .routines
do n = 1 to 3
  call setf
  interpret 'do k = 1 to 3; say f(n k); call churn; end'
end
exit
setf: .routines['F'] = .routine~new('f', 'use arg n; .rr~remove("F"); return "f" n'); return
churn: procedure
  do i = 1 to 400000; x = .array~new(20); end
  do i = 1 to 20; x = copies('a', 1000000); end
  return
::routine dummy
```

The shape needs the routine to be made inside a `CALL`. Made in the main body, as in the report's
`iv` and the corpus witness's `I` form, it matches at RED as well. Something in the main body's
frame holds it there.

**Under collection at every allocation.** A scratch test (`tests/rr_forms.rs`, copied into RED and
HEAD) runs each probe's small variant through `run_program_collect_every_alloc`:

* RED: all 13 give rc 120. For example
  `RRFORM interp2.rex rc=120 collections=28 out="f 1 1\n"` and
  `RRFORM cyclepkg.rex rc=120 collections=79 out="a0 b0\n"`.
* HEAD: all 13 give rc 0 with the oracle's stdout. For example
  `RRFORM cyclepkg.rex rc=0 collections=199`.

The committed witnesses are the corpus program, plus collect_stress
`a_routine_only_a_call_site_keeps_survives_collection` and
`routines_whose_sites_keep_each_other_survive_collection`. The corpus run and collect_stress pass
at HEAD (section 5). The report says RED fails only on each test's first form, because
`assert_stress_prints` stops there. The per-form table above covers the rest.

**The edge is load-bearing.** In MUT, `package_state.rs:141` (`out.extend(site_held.keys()...)`)
is replaced with `let _ = site_held;`. Then qa, other, cyclepkg and viartn each print their first
line and fail with rc 120.

**Concurrency, 5 runs each side:**

* `conc.rex`: four activities call `f(i)` 300 times through one unguarded method's site, while a
  fifth rebinds `.ROUTINES['F']` 200 times, with churn throughout.
  Oracle `bad 0`, rc 0, 5/5. HEAD identical 5/5. RED rc 120 5/5.
* `conc2.rex`: three activities run `INTERPRET` fragments whose sites keep fresh routines, while a
  fourth activity's site keeps routines it rebinds.
  Oracle `total 2100` 5/5. HEAD identical 5/5. RED rc 120 4/5.

**Growth, maxrss at 10000 and 100000 iterations.** For the report's `growth.rex` shapes, each HEAD
pair differs by at most 0.5 MB, and none rises with the iteration count:

| shape | HEAD | note |
|---|---|---|
| site | 46.4/46.1 | RED rc 120 |
| raise | 64.8/64.8 | |
| nested | 56.6/57.0 | |
| interp | 57.0/56.9 | |
| cycle | 57.1/57.4 | |
| native | 46.1/46.1 | RED rc 120 |

The new shapes (`growth2.rex`), HEAD at 10000/100000, with the oracle at 100000:

| shape | HEAD | oracle | note |
|---|---|---|---|
| pkgcycle | 93.6/93.0 | 20.6 | two `.package~new` packages per iteration whose sites keep each other's, then dropped |
| frag | 56.8/57.2 | | fragment site, fresh routine per iteration |
| copycycle | 58.1/58.5 | | `Routine~copy` pair whose sites keep each other, which is the pinned-copy path |
| other | 59.2/59.3 | | another package's `::ROUTINE` per iteration; RED rc 120 |

`pkgcycle` is the shape that shows packages whose sites keep each other's routines are collected
together and do not leak. Its 93 MB is flat and equals RED's (93.1/92.8).

## 2. The hold bookkeeping

**INST checks.** INST adds three checks to HEAD:

* in `hold_site_program`, a panic when either package is missing and when a release finds no entry
  (the two silent `return`s, and the `else if` with no `else`);
* a global count of running fragments;
* at every `collect_now` while no fragment runs, an assertion that the multiset of
  `(holder package, held package)` over every cached chunk's slot holds **equals** the union of all
  packages' `site_held` maps, and that `site_hold_count >= live holds`.

**Results:** no INST check fired anywhere.

* Every probe above, the concurrency probes, and every growth shape at 20000 iterations ran with
  the check live. For example: `g1 cycle: oks=38 holds=1044`, `g2 pkgcycle: oks=51 holds=774`,
  `cyclepkg oks=19 holds=4`, `conc oks=7`.
* All 1506 top-level corpus `.rex` programs, each run from a fresh copy with its `.d` fixtures under
  `timeout 30`. 32 of them collected at least once.
* `cargo test --release -p rexx-exec --test collect_stress` built from INST: `74 passed; 0 failed`.
  This runs the check at every allocation's collection.

So, against the brief's questions:

* No count goes negative or stays above the live holds.
* A second hold for one site replaces the first (`CallSite::set` releases what `held` had).
* No hold is lost because a package is missing.
* Activities sharing a package keep the counts balanced.

**Concern 5 (dead program's chunks pruned with holds).** INST shows `site_hold_count` only grows
once programs die holding:

* `g1 nested` at 20000 iterations: `holds=534 count=19718`.
* `g2 pkgcycle`: `holds=774 count=39494`.
* `c5.rex held`, after all 2000 holders died: `holds=0 count=2000`.

The holder's package dies with its chunks, so the edge goes with it. That is **not a wrong trace**,
and the exact-equality check above confirms it. **Not a leak** in memory either. It costs time:
every `INTERPRET` end afterwards takes `unpin_chunk_copies`. I measured that with callgrind on
`c5.rex`: 2000 dying holders (or 2000 dying non-holders), then N × `interpret 'x = i; y = x + 1;
z = length(y)'`. With libc subtracted, the per-`INTERPRET` difference between the two warmups is
`((9366785106 - 270131271) - (9340705784 - 252989568)) / 100000` = **89 Ir**, about 0.1% of an
`INTERPRET` at roughly 91k Ir. The warmups differ, so this is an upper estimate. There is also a
correctness edge, which is Minor 1.

## 3. M1's tripwire

The tripwire is present: `run/call.rs:600-606`,
`debug_assert!(parked_keeps.last().is_none_or(|(filed, _)| *filed < at), "an entered call's keep was left unclaimed")`.

It is live. In a debug build of MUT with `resume_call`'s `self.end_parked_keep(chunk, ended.is_ok())`
(`ir/drive.rs:3269`) replaced with `let _ = ended.is_ok();`, the report's `tripwire/t2.rex` gives:

```
thread 'rexx-interp' panicked at crates/rexx-exec/src/run/call.rs:600:17:
an entered call's keep was left unclaimed
rc=101
```

The HEAD debug build prints `1`, `2`, `end`, rc 0. `tripwire-probe.rex` gives `1 2 / 3 4`, rc 0, in
both builds.

## 4. Perf

I ran `head/rust/bench-programs/callgrind.sh -B 0.5 -j 12 -p "nop assign varlookup interpret dirread
intcall fibfunc extcall emptyloop compound textnum rexxcps"` with `base=<my efe8d3f40>`,
`head=<my 03a2a1e1a>` and `pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run`, 3 rounds. Every
spread is 0.0000-0.0002%. Exit 1, from `OVER` flags only on the **pin** column. The pin is the
older Phase 9 binary: intcall +11.6% and extcall +13.9% against base. Head against base, with libc
subtracted:

| program | d% |
|---|---|
| nop | +0.0000 |
| assign | +0.0000 |
| varlookup | +0.0000 |
| interpret | +0.0447 |
| dirread | +0.0380 |
| intcall | -0.1754 |
| fibfunc | -0.2340 |
| extcall | -0.5187 |
| emptyloop | +0.0000 |
| compound | +0.0000 |
| textnum | -0.1045 |
| rexxcps | -0.0612 |

Every value equals the report's `r2/callgrind.txt` to four decimals. The largest rise is interpret
at +0.0447%, which is within +0.05%. The whole-process figures differ from the record for some
programs (fibfunc -0.3403 here against -0.4002 there), because the BASE whole-process counts move
with libc.

## 5. The report's claims against records and reruns

* **Suite tables.** `cmp t8.3/r2/suite/{tests,table}.tsv t8.2/r1/suite/` shows both identical.
* **Corpus gate.** `r2/checks-corpus.txt` reads "1182 of 1183 matching", and the one mismatch is
  `lang/rexxutil_file_date_time.rex`. Rerun today (2026-10-26) from a fresh copy, BASE and HEAD
  both print `1603585800` on line 9 where the oracle prints `1603589400`. stderr and rc (216) are
  identical, and BASE's stdout equals HEAD's. So the mismatch predates this round and depends on
  the DST date, as the report says.
* **Checks.** At HEAD:
  * `cargo test --release -p rexx-exec --lib`: 1176 passed;
  * `--test collect_stress`: 74 passed;
  * `--test compiled`: 2 passed.

  The first `--lib` run failed 67 tests on "the worktree's build/lib is three directories above
  this crate". That was my scratch worktree, which had no `build/`. With `build` linked in, all
  tests passed.
* **Concern 1 is wrong as stated.** A fragment's own site can be witnessed red at `9963948e3`
  (`interp2`, section 1). The report's shapes only missed it because the routine was made in the
  main body. This is Minor 2.

## Findings

### Critical
None.

### Important
None.

### Minor

**M1. `site_hold_count` is not an upper bound once it wraps.** `prune_program_rows`
(`lib.rs:3589-3599`) drops a dead program's chunks without subtracting their holds. So the count
grows by one for every hold any dead program took (`count=39494` against `holds=774` after 20000
`pkgcycle` iterations). `hold_site_program` takes with a plain `+ 1`
(`dispatch/library.rs:213`), and the release profile has no `overflow-checks`. After 2^32 such
holds the count wraps to a value below the live holds, possibly 0. `run_fragment`'s test
(`run/interpret.rs:89`) then skips `unpin_chunk_copies` while `pinned_copies` is empty. A fragment
site's hold then stays in the holder package's `site_held`, and the routine it names, with the
routine's package, leaks until the holder dies. For the main program, that is the run.

That is far off: about one hold per call through a fresh dynamic routine, so days of a server-style
loop. But it makes `lib.rs:1387-1388`'s "At least how many ... so zero when none is" false. It also
costs the 89 Ir per `INTERPRET` measured in section 2 for the rest of the run. Fix: in
`prune_program_rows`'s retain, also collect `chunk.site_holds()` and subtract their count (the
packages are already dead, so only the counter changes). The count is then exact, never wraps in
practice, and concern 5's cost goes away.

Command:
`RRCHECK_LOG=1 memcap 2G timeout 120 <INST>/rexx growth2.rex pkgcycle 20000`, last line
`RRCHECK ok holds=774 count=39494`.

**M2. The fragment's own site has no red committed witness.**

* collect_stress's `<site-interpret>` form and the corpus program's `I` form both make the routine
  in the main body, and that shape passes at `9963948e3` (`interp` and `iv`).
* The unit test `an_interpret_fragments_site_holds_end_with_it` witnesses the release, not the
  trace.

The `interp2` shape above fails at `9963948e3`, in release and under collection at every allocation
(`RRFORM interp2.rex rc=120 collections=28 out="f 1 1\n"`), and passes at HEAD. Adding it as a
collect_stress form (or a corpus form) would close the gap. Concern 1's record should say the
witness exists.

## Queue candidates

* **Q-1 (perf round).** Concern 2, `Calls` layout sensitivity: one `Cell<bool>` on `Calls` cost
  nop +4.36%. It was not re-measured here. I agree with the proposed owner.
