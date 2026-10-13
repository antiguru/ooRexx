# Phase 9b Task 1 review: the 9a carry-in

Reviewed `f9105f68f..0368229d1` from the review package. Inputs read: the brief, the report, the
global constraints, and the carry-in file `.superpowers/sdd/queued/2026-10-13-phase-9b-carry-in.md`.

## Spec Compliance

✅ Spec compliant, with one finding where the brief's wording leads to a regression (I1 below).

* Step 1: `callgrind.sh` reports whole-process beside `-libc` and judges by whole-process
  (`rust/bench-programs/callgrind.sh:541-590`). No `rexx-bench` test exists for the table, so the
  brief's conditional test requirement does not apply.
* Step 2: pin present. `sha256sum /home/moritz/dev/repos/ooRexx-p9-pin/rexx-run` gives `c4b4e8c4...3c67`,
  58,197,968 bytes, the only file in the directory. Both `/tmp` build directories are gone (`ls`:
  no such file). Recorded in `phase-9a-gate.md:229-242`.
* Step 3: N1 done (`activity.rs:65,72`, `scheduler.rs:306,451,456`, `library.rs:651-692,705-735`).
  extcall whole-process is -13.40% against the pin (`cg-extcall2/summary.tsv`, 3 rounds, spread 0).
  Full set worst case is dirread +0.39%, so nothing is over +0.5%.
* Step 4: m1 assertion restored (`hash.rs:732-737`) with a witness. m2 done to the letter of the
  brief (`environment.rs:943-988`, `surface.rs:842-843`) and witnessed. m3 is in the README. m4 is
  in `dispatch/tests.rs:893-912`.
* ⚠️ Cannot verify from the diff: carry-in item 6 (the samples rows in `suite-9d-set.txt`, owned by
  9b/9c) is not in the brief's Task 1 steps. The controller should confirm that a later task owns it.

## Controller questions

**Q1: is m2's `.local` `Owed` refusal a regression? Yes.** Checked by running both sides and the pin:

```
t = .T~new
say 'q:' t~findc('STDQUE')
::class T public
::method findc external "LIBRARY orxmethod TestFindClass"
```

* Oracle: `Error 91.999: Message "FINDC" did not return a result.`, rc 165.
* Pin (`/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run`): the same, rc 165.
* HEAD (`rust/target/release/rexx-run`): `rexx-exec: directory entry "STDQUE" is not implemented
  (Phase 10 Rxapi)`, rc 120.

Can an owed `.local`/`.environment` entry be a class?
* Each name in `ORACLE_ENVIRONMENT` and `ORACLE_LOCAL` was run as `x = .NAME` on HEAD, one process
  per name, from a fresh directory. Only `.STDQUE` refuses. So after bootstrap the only reachable
  owed entry is `.local`'s `STDQUE`, which is a `RexxQueue` instance on the oracle and never a class.
* `NativeActivation::findClass` (`NativeActivation.cpp:3094-3112`) filters its answer to
  `isInstanceOf(TheClassClass)`, so the queue can never be returned. A refusal here therefore guards
  nothing.
* An `Owed(None)` placeholder (an entry the bootstrap fills) could in principle stand for a class.
* The package-local `Owed` arm cannot be reached from Rexx, because reading the placeholder refuses.

Contrast `package_find_class`: oracle `.context~package~findClass('STDQUE')` answers the queue
(`SESSION`, rc 0), so the refusal there is correct. That is why "as `package_find_class` does" is
the wrong model for a search whose caller keeps only classes.

Recommended behaviour:
* In `class_in_program`'s `.local`/`.environment` step, an owed entry the oracle holds as a non-class
  answers `Ok(None)`. Today that means the Rxapi-owed `.local` entry, `STDQUE`, a queue.
* An `Owed(None)` (bootstrap-fill) entry stays loud.
* The package-local `Owed` arm stays loud.
* Flip the `STDQUE` arm of `a_class_search_from_a_program_refuses_an_owed_entry` to `Ok(None)`.
* Add a corpus witness for `findc('STDQUE')` against the oracle's 91.999.

**Q2: is N1 sound? Yes. No `unsafe` was added.**
* The only `unsafe` in the diff is the report's line naming the `unsafe_sites` test, so the diff adds
  no `unsafe` code.
* Rooting is unchanged: `activity.rs:694` iterates `native_handles` and `object_roots` auto-derefs
  through the box. The `abandoned` list holds the same frames, now boxed.
* Frame lifetime is unchanged: push, take, pop and abandon move ownership exactly as before.
  `end_native_frame` takes `&mut NativeFrame` through deref. Boxing makes frame addresses stable
  across `Vec` growth, which is strictly safer.
* No raw pointer into a frame exists (`native_handles` uses checked by grep across `src/`).

**Q3: whole-process column. Yes on both counts.**
* `cgsum.py` takes callgrind's `summary:` line, which covers every object (and asserts it equals the
  per-object sum). The `-libc` column is `summary - libc - ld`. Checked: pin
  8975966708 - 1438625601 - 421349 = 7536919758, matching `summary.tsv`.
* The table uses column 4 (`summ`) as the "whole" column, and `OVER` tests `deltas["whole"]`
  (`callgrind.sh:586-589`). SPREAD is also judged on whole-process.

**Q4: can the tests fail?**
* m1 can. A missing `store_writes` bump would make the post-fill read print `.PLMRNAME` where the
  oracle prints `filled`, so release fails as well as the debug assert. The witness's debug-binary
  stdout was read here and matches the report.
* The implementer's two m1 mutants are credible. I did not re-run them: editing source is barred to
  me, and a copy build exceeds the 5-minute bound.
* The m2 UNKNOWN witness distinguishes before from after (91.999 versus 93.900, per the report).
* The m2 unit test is red under its package-local mutant. Its `STDQUE` arm pins the regression in I1.
* The m4 shape was confirmed here: oracle `XX` / `Array` rc 0; crate `XX`, the refusal, rc 120.

## Strengths

* The budget change is small and correct. `-T` re-tables old runs, the table judges by
  whole-process, and the commit carries a negative control (`before` flagged OVER at +3.05%).
* N1 is the minimal fix (box plus spare reuse), with the memcpy reduction measured at the site.
* The report flags I1 itself (concern 1) instead of hiding it.
* The m1 report corrects its own premise (concern 2).
* Callgrind run outputs are copied into `docs/superpowers/records/`, as the durable-artifacts rule
  requires.

## Issues

### Critical
None.

### Important
* **I1** (`environment.rs:984-985`, `environment/tests.rs:1029-1037`): m2 turns the oracle-matching
  NULL for a native `FindClass('STDQUE')` into rc 120. The unit test pins the regression as intended
  behaviour. Fix as recommended under Q1. This is brief-mandated wording ("as `package_find_class`
  does"), so the human decides; the observable is a regression.

### Minor
* **M1** (`surface.rs:846`, report concern 3): `class_named`'s `dot_variable(...).ok()` still drops
  a raised condition for a non-program scope. This is out of m2's scope and should be queued, not
  fixed here.
* **M2**: `cg-extcall2/binaries.txt` names `before` and `after` binaries under `/tmp`, which will not
  survive a reboot. The sha256 values are recorded, so the run remains attributable. Acceptable.

## Assessment

**Task quality:** Needs fixes (I1 only).

**Reasoning:** The instrument, the pin and N1 are correct and measured, and N1 adds no `unsafe` and
does not change rooting. m2's `.local` `Owed` propagation regresses one oracle-matching answer,
reachable from any native method, and its unit test enshrines it.
