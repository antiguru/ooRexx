# Task 23 fix round 1, re-review (base c9c7dfff2, head 4db3412ee)

Probes ran in a `git archive` of `4db3412ee` (`rust/`, `interpreter/`, `extensions/`, plus a copy of
`ootest/`) under `/tmp/claude-1000/p6-t23-rr/`, own target dir, `memcap 8G`, deleted after.

**Verdict: CHANGES NEEDED.** Eight of the ten items are fixed. C2 is partial: the collector's
post-sweep prunes still resolve through the tagged `Heap::get`, and the gate now says the opposite.
F1 is partial: the every-group population is not stated exactly.

## Items

| # | Item | Verdict | Evidence |
|---|---|---|---|
| 1 | C1 SAFETY | FIXED | `island.rs:47-55` names `Lent::interp` and `HostRef` through `Island::host`, and the abandoned call's off-baton box drop. Checked: `OffBaton` (`dispatch/library.rs:905`) is `HeldCall`, `NativeCall` (`returns: u16`, descriptors) and `CStringPool` (`Vec<(Option<ObjRef>, Box<[u8]>)>`, `Vec<(ObjRef, Vec<u8>)>`): no `Rc` or `Cell`. `drop(thread)` (`library.rs:1171`) runs while `ended` is live. On the panic paths the context and box drop during unwind after `ended`, but `Lent::drop` does not give the baton back while panicking (`island.rs:121-129`), so they still drop on the baton. |
| 2 | C2 untagged walks | PARTIAL | `clear_uninit_all` is fixed, and the witness passes at head (3 sharing witnesses ok). Not fixed: `Interp::collect_now` (`lib.rs:2855-2870`) prunes `kept_strings`, `guards` pools and watches, and `semaphores` with `heap.get(object).is_some()`, which is tagged, so whichever activity collects marks every object in those tables. Probe (scratch test, `--release --features sharing`): main makes 100 `.MutexSemaphore`s and acquires and releases each, then `.t~new~start('other')~result` where `other` does `call GC 'Force'`: program shared **102**. Same with `GC` called by main instead: 2. No `GC`: 2. Control, the prunes wrapped in `sharing_pause(true/false)`: 2. The gate sentence "The collector's walks and the UNINIT registry's resolve untagged" (criterion 6) is therefore false. Fix: resolve those three liveness checks untagged (an untagged `Heap::is_live`, say), add the 100-semaphore witness. |
| 3 | C3 profile-independent | FIXED | Corpus re-run at head, `--release` and test profile: byte-identical tables, both equal to `sharing-corpus.md` below its header. Every-group run in both profiles (`RAYON_NUM_THREADS=4`): shared columns identical; only `base/bif/TIME.testGroup`'s program objects differ by one (it ends at its deadline). |
| 4 | C4 bootstrap split | FIXED | `Tag { activity, shared, program }`, `counts: [SharingCount; 2]`, `sharing_program_starts` at the end of `bootstrap_library` (`lib.rs:2155`), after the bootstrap's objects. Every table row has bootstrap objects 272. |
| 5 | C5 wallclock usage | FIXED | `-x ORACLE_PROGRAMS` in both usage texts. |
| 6 | F1 populations | PARTIAL | All four populations are measured and the figures match their records (corpus, derived, groups, harnesses; percentages recomputed). The every-group re-run at head reproduces the record's shared counts exactly (bootstrap 66, program 157); two timing-dependent rows differ in objects or outcome (`TIME` deadline, `SysSleep` failure vs pass). Open: the gate calls it "every `.testGroup` ... run whole", but a refusal or deadline ends a group's run at that test. `grep -c '^\| [^|]*testGroup \| refused at' sharing-groups.md` gives 76 and `grep -c '^\| [^|]*testGroup \| no test ran' sharing-groups.md` gives 36, of 388. Fix: one clause in the gate that each group runs until its end or its first refusal or deadline (outcomes in the record), without a count unless quoted with its command. |
| 7 | F2 P74 | FIXED | Both producers named. Nit: `time_support.rs:201` is the `behaviour` line; the timer id is converted at `:202`. |
| 8 | F3, F4 | FIXED | `:1320` dropped; `island.rs:167` (`thread::scope`) and `signal.rs:237`, `:269` (`thread::spawn`) in the rows; "test statics". P71's command at head, sorted, equals the committed inventory below its header. |
| 9 | F5, F6 | FIXED | "(rulings P52, P72)" in gate and spec; "Spec sentences are amended". |
| 10 | F7 | FIXED | `thirty-runs.sh` takes `$1` and runs it from its own `mktemp -d`; `thirty-runs.txt` re-run; `binaries.txt` path `target-fix1` matches `sharing-off-text-hash.txt`'s `fix1` build. |

## New findings

**N1, Minor: the every-group test runs in any `--features sharing` test run and needs most of 8 GB.**
`sharing::sharing_fraction_over_every_ootest_group` is a plain `#[test]` under `#[cfg(feature =
"sharing")]`, not `#[ignore]`. It is not built by plain `cargo test --workspace`, so the P51 bar is
unaffected. But `cargo test -p rexx-exec --features sharing` runs it, and the record says the
default 32 rayon threads were OOM-killed at the 8G cap. Re-run at head with `RAYON_NUM_THREADS=4`:
64 s, and `/usr/bin/time -v` around the command reports maximum resident 7014784 kB (the largest
descendant). Fix: `#[ignore]` it (the record's command already names it with `--exact`, which
then needs `--ignored`), or bound its pool in the test.

**N2 (from item 2): false gate sentence.** "The collector's walks ... resolve untagged" goes with
item 2's fix; if the prunes are fixed, the sentence becomes true.

## Checked and holding

- Pause nesting: a depth counter; `resolved` neither counts nor re-tags while paused. Both sites
  unpause before any early return (`route.rs` holds the `Result` and applies `?` after
  `sharing_pause(false)`; `hash.rs` uses `.ok()`). A panic inside a paused read ends the run, and the
  heap goes with it. Every call is under `#[cfg(feature = "sharing")]`; the `hash.rs` change from
  `debug_assert_eq!` to a `#[cfg(debug_assertions)]` block with `assert_eq!` is the same code in each
  profile. `sharing-off-text-hash.txt`'s `fix1` hash equals `head` and `aonly` (not rebuilt here).
- `log_sharing` is called once per run (bootstrap objects per run = 272 in every harness row:
  243712/896, 2770048/10184, 1158448/4259, 156944/577).
- `cargo fmt --check` 0 and `cargo clippy --workspace --all-targets --features
  rexx-exec/sharing,rexx-exec/pinning -- -D warnings` 0 at head.
- No new `unsafe`, no new statics (the counters are `Cell`s in `Heap`), no em-dashes in added lines
  (`git diff c9c7dfff2 4db3412ee | grep '^+' | grep -c '—'`: 0).
- Criterion 7: the gate's medians equal `pingpong/table.txt`. Load before the run was 2.27 8.41
  10.09 (`binaries.txt`); the oracle's `pingsem` median moved 0.581 to 0.446 between the two
  recordings. Recorded, not gated, so no finding.
