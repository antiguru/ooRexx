**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** N1 native-frame memcpy cost is phase:9.1; the unchecked minors m1, m3, m4 are 9d.

# Phase 9b carry-in from the Phase 9a final review

Source: `.superpowers/sdd/2026-10-11-phase-9a/final-review.md`, `final-rereview.md`, ledger tail.

1. **N1, native-frame memcpy.** `push_native_frame`/`take`/`pop_native_frame` still copy the bigger
   native frame through memcpy: 1.110e9 Ir on the pin, 1.374e9 at 1f068bcca, about +88 Ir per native
   call, plus a fifth memcpy site in `library.rs` `push_native_frame` the pin lacks. Whole-process
   extcall is +3.05% against the pin (9.249e9 vs 8.976e9) while `callgrind.sh`'s libc-subtracted
   table shows +0.28%. Data: the review's callgrind rows (re-derive; /tmp is not kept).
2. **The budget instrument.** `callgrind.sh` subtracts libc.so.6 and ld-linux; libc work called from
   crate code (memcpy, calloc zeroing) is then invisible to the +0.5% budget. Moritz to rule which
   instrument the Phase 9 close judges by.
3. **m1.** The `.NAME` miss memo's only check is a `debug_assert!` on a hit that no gate reaches
   (no witness repeats a miss; `package_local_get` dropped `directory_get`'s view assertion).
4. **m2.** `class_in_program` (`environment.rs:593-597`) drops an `Err` and an `Owed` from
   `package_local_entry` and falls through to `.local`; `package_find_class` propagates both.
5. **m3, m4.** Records: `objname-table.rex`'s README row for `::ROUTINE`; the metaclass pin lacks
   the "a subclass already took" shape.
6. The three samples tests in `rust/corpus/suite-9d-set.txt` are owned by 9b (SysTempFileName) and
   9c (front-end `.rex` resolution).

## Moritz 2026-10-13

* Item 2 ruled: "Whole process in the limit, but we have to account for timing noise on this
  machine. Callgrind might avoid this, though." The Phase 9 close judges the +0.5% budget on
  callgrind's whole-process instruction count (libc and ld included); callgrind is deterministic, so
  no timing noise enters. `callgrind.sh` already records the libc and ld columns; the comparison
  must add them back.
* Timing tests: "timing tests under load are hard to get right, or we run them using DST." SysSleep
  TEST_SLEEP_DURATION in `concurrency_tests`' S2 list runs under the deterministic simulation's
  clock, or is licensed there; owner 9d.
* "/tmp is not durable here": the perf pin and any artifact a later phase needs live outside /tmp.
  The pin is rebuilt from a98238be5 into /home/moritz/dev/repos/ooRexx-p9-pin/ (outside the
  worktree, not tracked) before the Phase 9 close.
