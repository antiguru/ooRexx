# Phase 6.1 Task 12 review

Reviewed range `67dafe88a..dfd65490d` (HEAD `9512f083c` adds only the ledger line and the review
diff). Rulings in `progress.md` from "Task 12 dispatched" onward were applied: the parse overrun, the
TSan exclusion, the SysSleep floor, the R6 self-test filter and the harness-race fix are not findings.

## Verdicts

* **Spec compliance: needs fixes.** 0 Critical, 2 Important, 5 Minor.
* **Task quality: approved.** 0 Critical, 0 Important, 3 Minor.

## What was checked, and how

* Code commits, read in full from the diff and the tree at HEAD.
  * `fce0bd4c1`: `Bytes::heap_len` returns `v.len()` for `Repr::Heap` and 0 for inline. That is what
    the old `!is_inline() => as_slice().len()` arm gave. `Body::held_bytes` answers 0 for
    `Body::Class` through `_ => 0`, so moving Class to its own match arm, where it adds no bytes,
    leaves `freed_bytes` unchanged for every body kind. Text, Array, Instance/Buffer and the rest are
    still summed through `held_bytes`. The `survivor_bytes == held_bytes` debug assertion in
    `Heap::collect` still runs, and G6 (debug) is green at `97cb37712`. The running figure stays exact.
  * `01c7d7a85`: `traced_mode` now reads `TraceCache::traced()`. Every place that writes the pause
    flag or the running activation also rebuilds the cache with the flag: `replace_debug_pause`,
    `push_activation`, `pop_activation`, `set_trace_mode` and the `dispatch.rs:3391` resume. The one
    direct `set_debug_pause` call is inside `replace_debug_pause`. `lib.rs:2352` writes `trace_mode`
    on an activation that has not been pushed yet, and the push then rebuilds the cache. A debug
    assertion compares the cache with the old derivation.
  * `0f159f674`: `from_secs_f64(seconds).max(SLEEP_FLOOR)`. I ran
    `sim::tests::time_e_after_any_sleep_is_never_zero` in my own target dir: 1 passed. I wrote a
    real-mode probe (2000 × `time 'R'` / `SysSleep 0` / count `time('E') = 0`) and ran it from a fresh
    directory under memcap and timeout. Oracle `zero 0`, ours `zero 0`.
  * `e075f1d36`, `a68c735fd`, `97cb37712`. `GUARD_PASSING` now holds the group's full set of tests:
    the sorted list equals the sorted `::method test*` names of `GUARD.testGroup`, with no diff. The
    nine additions match `guard-bisect-flip.txt`.
* Gate record. All four `bg/<sha>/status.txt` files exist, and their exit codes match the table. At
  `97cb37712` the summed `test result` lines give G4 3141/0/4, G6 3145/0/4, G8 16 and G9 15, each
  matching the record. `g4-test-release.txt:1513` and `g6-test-debug.txt:1524` are the cited `ok` lines.
  Also matching the record: TSan `tsan3.out` (api/lib/int exit 0, no tsan log), `lib.txt` 1063
  passed with 14 filtered, and `int.txt` 49/2/36/1. The rest agree too:
  * `whole-groups-run.txt`: exit 0, 6:50, 2,585,996 KB.
  * sim-gate summaries: 13720 runs in 131 s and 1792 in 12 s.
  * `cg-close2/table.txt`: parse +1.1979, rexxcps +0.2694, fibfunc +0.4303.
  * The base61 sha256 `2ea19b3e…` matches the file on disk.
  * `wall/table.txt` and `perf-stat.txt`.

  I found no result line written ahead of its output.
* Queue mapping. Every `- Task 12 queue add:` line has a row: l.84, 101, 108, 119, 121, 127, 129,
  135, 138, 142, 144 and 151. I re-ran four "fixed" rows at HEAD, each from a fresh directory under
  `memcap 2G timeout 20`, against `/home/moritz/dev/repos/ooRexx/build/bin/rexx`:
  * row 17, `w_ustr1.rex`: `while 3` on both.
  * row 26, `i7a.rex`: identical, 10 lines.
  * row 23, `t4d.rex`: `U` on both.
  * row 46, `x8.rex`: stdout and stderr identical.

  Row 40's probes (`cs.rex`, `e103.rex`) are translation-time 24.1 and 10.2, as the row says. A
  run-time `trace value x` with `x = 's'` substitutes its insert on both engines, so R3 does cover the
  item.
* "For Moritz". I listed the `Ruling` lines from the ledger at `5c83d9250` and the deferred and parked
  lines, then compared them with the report (findings S-I2 and S-m4).
* Roadmap. Row 6.1 reads "CLOSED … but for criterion 7: `parse` is +1.1979% … waits on Moritz's
  ruling". Nothing claims criterion 7 is met, and the parse overrun is stated with its figure. For row
  9, I re-ran the grep commands the row cites.

## Spec compliance findings

### Important

* **S-I1. Spec criteria 1-4 are not recorded in the gate record.** The brief's Step 4 and the carry
  ("Record each spec section 8 criterion … with its evidence") require every section 8 criterion.
  `## Task 12` has sections for criteria 5, 6, 7 and 8 only. No earlier section of
  `phase-6-1-gate.md` records them either. `grep -n -i "cond1\|pakinds\|interpreply\|closed_phases"`
  finds only an incidental `closed_phases.rs` mention at l.765. The evidence probably exists:
  `closed_phases` and `refusal_dispositions` run in G4/G6, and the D1/D5/D6 witnesses are corpus
  programs. But the record never maps criterion 1 (closed_phases + dispositions), 2 (IMPLEMENT groups
  through corpus), 3 (the D1 scope witnesses, five runs for the concurrent ones, `reply` as predicates)
  or 4 (scout B's D5/D6 probes) to a command and its output. Roadmap row 6.1 says CLOSED, and with it
  "every loud refusal naming Phase 5 or no phase … is implemented, or re-homed …", which is
  criterion 1. Fix: add a short "Criteria 1-4" subsection naming, for each criterion, the tests or
  corpus programs and the `bg/97cb37712` log lines that show them green.
* **S-I2. "For Moritz" leaves out the rulings Task 12 itself made.** The report's pointers are frozen
  at `5c83d9250`, the dispatch line, so the list stops before every ruling made during the task. These
  are all rulings made on Moritz's behalf, and each changes behaviour or coverage:
  * the GUARD list rewrite;
  * the self-test skipping `clock=real` rows (narrows its row pool);
  * the SysSleep root-cause ruling and the 1 µs floor on every delay, 0 included;
  * `refusal_sites` refreshed as its own commit;
  * the seeded-gate harness keeping the shared run directory (leaves an empty directory behind);
  * `quick_native_calls_at_the_smallest_bound_run_alike` excluded under TSan (one sim-bound native
    test goes unchecked under TSan);
  * the parse-round widening and its "items (1) and (4) accepted" clause.

  Only the parse residual appears, and only as an open question. Fix: add one line each with its
  `progress.md` line.

### Minor

* **S-m1. Rows 40 and 49 end as "neither" with no file.** The carry gives three outcomes and requires
  a new file for "neither". Row 40 leans on R3 and is flagged to Moritz, which is defensible. Row 49
  (rexxcps +0.09% between Task 6 rounds) gets no file and no flag, only "inside the budget".
* **S-m2. Row 23 claims "fixed in Task 11a Step 6b" without a commit hash.** The carry requires one.
  The ledger's l.229-231 point at `8b95afe79` and its successors.
* **S-m3. Row 20 claims "resolved" with no probe rerun.** The evidence is a reading of
  `Activation::object_roots`. That evidence is reasonable, but the carry requires a probe for a fixed
  row. The row should say "no probe; resolved by inspection" as its outcome, not "resolved".
* **S-m4. The parked list leaves out two Task 11a queue files**: `2026-10-10-call-on-nostring-insert.md`
  (ledger l.238 "Queue add") and `2026-10-10-security-manager-command-rc-requires.md`. It lists the two
  from 11b. Neither 11a file is marked RESOLVED. The deferred minor at ledger l.44 is covered through
  row 13's file, but the parked list does not point to it.
* **S-m5. Roadmap row 9 assigns Phase 9 an item already fixed.** Its exclusions list includes "the
  `native_method` re-home", which is the `phase-4-exclusions.txt` row "Method~new AND Routine~new
  REFUSE ANY THIRD ARGUMENT" (l.5615-5629). I re-ran the probe at HEAD.
  `say .Method~new('mm', 'return 43', .context~package)` and its Routine twin answer `a Method` and
  `a Routine`, rc 0, on both engines. The report flags the stale row. The roadmap still names it as
  Phase 9 work.

## Task quality findings

### Minor

* **Q-m1. The pingsem/pingguard wall-clock "attributed to layout" claims more than the data shows.**
  The data shows that a pad moves HEAD's cycles by up to 4.1 points (cpad8 against close). It does
  not show that layout produced close's +4.5-4.6%: cpad48 is still +4.19% for pingsem. The memory
  rule in force says a figure under about 4% on one axis is not evidence either way. "Not
  distinguishable from layout" is the claim the data supports.
* **Q-m2. `fce0bd4c1` assumes, without saying so, that a Class body holds no bytes.** The `Body::Class`
  arm skips `held_bytes`, which is exact today only because `held_bytes` answers 0 for Class through
  its catch-all arm. Only the debug assertion catches a future Class byte charge. A short comment on
  the arm, or a `debug_assert_eq!(body.held_bytes(), 0)` inside it, would make the assumption local.
* **Q-m3. The report's queue rows 33 and 34 use timing from single runs** (`append.rex` 0.03 s against
  0.02 s, `copies.rex` 0.44 s against 0.77 s) with no run count. The gap to the scout figures (5.4 s,
  1.0 s) is large enough that the conclusion holds. The run count should still be stated.

## Re-review 1

Scope: `81b6b103b..27f3de7fa` (`review-t12-fix1.diff`) and the report's "## Fix round 1".

**Verdict: spec compliance passes, and task quality is approved with Minors.** 0 Critical, 0
Important. Q-m1 is only partly addressed. The fixes add 3 new Minor issues.

### Findings from the first review

| finding | status | checked |
|---|---|---|
| S-I1 criteria 1-4 | addressed | Each cited log line in `bg/97cb37712/logs/` reads as stated. In g4, `:1963` and `:1967` are the two closed_phases tests `ok` and `:1969` is their 8 passed, inside the `closed_phases.rs` section at `:1957`. `:2183` reads `991 of 991 matching` and `:2187` 29 passed / 1 ignored. `:2964` is ir_recorded_oracle's 21 passed. `:3441` and `:3443` are refusal_dispositions and `:3454` is refusal_sites' 5 passed. `:1498` and `:1540` are the two REPLY crate tests `ok`. g6 has `:1501`, `:1545`, `:1973`, `:2190` and `:3446`. Every corpus program named for criteria 3 and 4 exists under `rust/corpus/lang/` and is listed in `phase-6-1.txt`. `ir_recorded_cases/parse-errors-main` exists. `c3reply.txt` and `c3sl2.txt` each hold 5 runs per engine, all rc 0, and each engine gives one outcome. The c3reply output matches the record line for line. |
| S-I2 Task 12 rulings | addressed | The l.256-l.273 pointers match `progress.md` at `9512f083c`. See n4. |
| S-m1 rows 40, 49 | addressed | Both files exist in the house style. The translation-error file quotes my run-time `trace value` control correctly. |
| S-m2 row 23 commit | addressed | `8b95afe79`, with a build of the parent as control. |
| S-m3 row 20 | addressed | |
| S-m4 parked list | addressed | Both 11a files and the l.44 pointer are present. |
| S-m5 roadmap row 9 | addressed | The `native_method` re-home is removed from row 9. The three exclusions rows are CLOSED. I re-ran the probe from a fresh directory under memcap and timeout, oracle against the `close` binary (sha256 `4e8ab4c7…`, code equal to `97cb37712`), twice. `.Method~new(…, .context~package)`, `.Routine~new(…, .context~package)~call` and context-less `.Routine~new('x', 'return mainr()')~call` gave `a Method`, `43` and `new mainr`, rc 0, on both engines. |
| Q-m1 wall-clock wording | **partly addressed** | See n3. |
| Q-m2 Class arm | addressed in code | The `debug_assert_eq!` is in place. The comment and the report row each add a false sentence: n1 and n2. |
| Q-m3 run counts | addressed | |

### New findings (all Minor)

* **n1. The new comment in `heap.rs` is false.** "A Class body holds no bytes outside its slot" does
  not hold, because `Body::Class { owned: Vec<ObjRef> }` owns a separate `Vec` allocation. What is
  true is that `held_bytes` charges none for a Class, so the running figure never counted any. Reword
  to that, and reword the assertion message ("a Class body holds bytes") the same way.
* **n2. The report's Q-m2 row says "classes are never collected (D59)".** That is false. Phase 5j
  reopened D59 (`docs/superpowers/specs/2026-09-09-phase-5j-class-lifetime.md:12`), and user classes
  are collected: `class_lifetime.rs` `a_class_nothing_refers_to_is_collected`, and the sweep's own
  `freed_classes`. The conclusion still holds, because a debug-only assertion costs nothing in
  release. The reason given for it is wrong.
* **n3. "As large as the drift" is false for pingguard,** in the gate record (Criterion 7, wall clock)
  and in the report's "Wall clock". The 4.1-point pad move is pingsem's: close +4.64 against cpad8
  +0.56. For pingguard the largest pad move is 2.78 points (close +4.53 against cpad8 +1.75), below
  its +4.53 drift. So "not attributable to code" is supported for pingsem only. For pingguard the
  statement should be that the drift exceeds the largest pad move seen and is unattributed.
* **n4. The "Rulings made on your behalf" list skips l.257.** That ruling held that the parse overrun
  blocks the close. l.263 and l.268 superseded it, but it is the ruling that says why the close was
  allowed to proceed, so it belongs beside l.268.
