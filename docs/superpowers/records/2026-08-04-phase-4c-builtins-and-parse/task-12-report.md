# Task 12 report: `builtin/datetime.rs` (`DATE`, `TIME`)

## Scope taken

Both option letter sets were re-measured against the oracle rather than trusted from the brief, per its own instruction. `DATE_Min`/`DATE_Max` are 0/5 and `TIME_Min`/`TIME_Max` are 0/3, read directly from `BuiltinFunctions.cpp`. Both builtins were implemented in full -- every output letter (13 for `DATE`, 11 for `TIME`) and every input letter (10 for `DATE`, 9 for `TIME`), plus `DATE`'s `osep`/`isep` separator arguments -- not only the conversion-form subset the brief's own examples name, because the brief's Step 1 ("probe every one") binds for both names and because implementing the full surface was the only way to get the arity table and the argument-position quirks right.

## Where the brief was wrong, measured

The brief's own Step 0(b)-adjacent claim -- "Host- or locale-dependent: `L`, `M`, `W` (names), `T`, `F` (absolute clocks). Deterministic given a fixed date: `B`, `D`, `E`, `I`, `N`, `O`, `S`, `U`" -- does not survive measurement. `DATE.testGroup`'s own transcripts (`date('L', date, format)`, `date('T', date, format)`, `date('F', date, format)`, all against a fixed `indate`) show `L`/`M`/`W`/`T`/`F` are exactly as deterministic as the other eight *once an input is supplied*. Read directly from `RexxDateTime.cpp`: `monthNames`/`dayNames` are hardcoded English arrays, not a locale lookup, so `L`/`M`/`W` never vary by host or locale at all in this build. The only non-deterministic case, for *any* of the thirteen letters, is the no-argument ("today") form -- confirmed for `B`/`D` too (a no-argument `date('B')` changes every day exactly like a no-argument `date('T')` would). This is stated and corrected in `datetime.rs`'s own module doc rather than left as a silent divergence from the brief.

## Probe tables

All probes wrapped `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx FILE )`, run from fresh directories under the scratchpad, never the scratchpad root.

### Option lists (measured, both rc 216)

```
say date('J')   ->  Error 40.904:  DATE argument 1 must be one of BDEFILMNOSTUW; found "J".
say time('Z')   ->  Error 40.904:  TIME argument 1 must be one of CEFHLMNORST; found "Z".
call date , , 'zz'    -> Error 40.5 (argument 2 required) -- option2 empty check never reached
call date , , ''      -> same 40.5
```

Alphabet crossed against the option-letter checks: every byte in `A-Z`/`a-z` (26+26), the empty string, a control byte (`0x01`), and a high byte (`0xFF`) -- 54 cases, all through `every_date_output_letter_is_accepted`/`every_date_input_letter_is_accepted`/`the_empty_option_renders_as_the_empty_string_not_a_placeholder`/`a_high_byte_round_trips_raw_and_a_control_byte_becomes_a_placeholder`. `datatype`'s neighbouring 93.915 spells its empty case `found "?"`; `DATE`/`TIME`'s 40.904 spells it `found ""` -- measured both ways, and pinned as two different answers rather than normalised.

### Conversion-form determinism (measured)

```
say date('S','2026-08-04','I')         -> 20260804     (rc 0, the status harness's own probe)
say time('S','12:34:56','N')           -> 45296         (rc 0, the status harness's own probe)
say date('B','20070922','S')           -> 732940
say date('D','20070922','S')           -> 265
say date('L','20070922','S')           -> 22 September 2007
say date('M','20070922','S')           -> September
say date('W','20070922','S')           -> Saturday
say date('B','1 Jan 1970')             -> 719162        (confirms UNIX_BASE_DATE)
say date('B','31 Dec 9999')            -> 3652058       (confirms MAX_BASE_DATE)
say date('B','3652059','B')            -> Error 40.19 (one past the maximum)
```
All twenty-three of `DATE.testGroup`'s `test_standard`/`test_base`/`test_european`/... transcripts were checked against my own implementation and match; a representative subset is what `date_every_output_letter_from_a_fixed_standard_input` and `date_every_input_letter_round_trips_to_standard` pin as unit tests.

### `TIME('R')` reset semantics (brief's own transcript, reproduced)

```
first time('R')            0
time('E') after ~0.11s     0.109014
time('R') after another    0.219483   = sum of BOTH burns
time('E') immediately      0.000023   so that R did reset
```
Pinned by `time_r_resets_relative_to_the_last_reset_not_program_start` and `time_e_does_not_reset_the_anchor_time_r_does`, driving `Interp::elapsed_anchor` directly (a fabricated anchor offset) rather than the wall clock, since two nearby reads cannot separate "reset" from "not reset" reliably. The reset semantics pinned, not a value, per the brief's own instruction.

### The clock is cached per clause (brief's own transcript, reproduced -- and the mechanism it initially broke)

```
say time("L") burn() time("L")   ->  identical               (one clause)
n1 = time("L"); zz = burn(); n2 = time("L")  -> different     (two clauses)
```
**This defect was caught by my own test, not assumed correct.** The first implementation put the clock cache on `Interp` (one field, matching `random_seed`'s own precedent). It reproduced every measured shape except this exact one: `burn()` is a real internal routine, and its own body's instructions (a `DO` and a `RETURN`, each its own `step_in_temps_frame` call) invalidated the *shared* `Interp`-wide cache mid-clause, so the second `time("L")` after `burn()` returned read fresh instead of cached -- `two_clock_reads_in_one_clause_answer_identically_across_a_real_burn` failed with `n1 != n2` on the first attempt. The fix moves the cache to `Activation::cached_clock` (per-activation, matching the oracle's own `ActivationSettings::timeStamp` exactly), so a callee's own instructions invalidate only its own cache. Re-measured after the fix: all three clock-caching tests pass, including the adjacent negative control (`the_same_burn_between_two_clauses_changes_the_answer`).

The burn size was tuned from an initial 2,000,000-iteration loop (~22s per test, three tests ~68s) down to 20,000 iterations (~0.19s per test) after confirming ten consecutive runs stayed green at the smaller size -- fast and, so far, not flaky.

### Separator errors (measured)

```
call date , , , 'a'                    -> 40.43  (not a single non-alphanumeric char)
call date 'b', , , ''                  -> 40.44  (B incompatible with any osep)
call date , '20070922', 'w', , '-'     -> 40.44  (isep incompatible with style2=W, before the style2 switch even runs)
call date , '1 May 2022', , , '-'      -> 40.44  (parse failed WITH an isep -> incompatible-format shape, naming the whole indate)
call date , '1 Zzz 2022'               -> 40.19  (the same kind of malformed input, with NO isep -> plain conversion error)
```

### `TIME`'s own errors (measured)

```
call time 'r', '12:00:00'   -> 40.29  TIME conversion to format "R" is not allowed.
call time 'e', '12:00:00'   -> 40.29  (naming the OUTPUT style, not the input style)
call time , , 'n'           -> 40.5   (option2 present, intime absent)
call time , '99', 'h'       -> 40.19  TIME argument 2, "99", is not in the format described by argument 3, "H".
```

## Divergences, declared rather than silent

1. **UTC-only "now".** This crate has no time-zone database; every no-argument clock reading uses UTC with a fixed zero offset, where the oracle uses the host's local zone. Barred from every differential comparison by D11, so unobservable to the gate; documented in the module doc rather than hidden.
2. **A defined answer where the oracle's own is undefined.** `TIME`'s time-only input styles (`N`/`C`/`L`/`H`/`S`/`M`) leave `year`/`month`/`day` at whatever `RexxDateTime::clear()` sets (`0`), and asking a date-shaped output (`F`/`T`/`O`) for such a result indexes the oracle's own month table at `-1` -- measured, `time('F','12:34:56','N')` on the real interpreter returns a value that depends on adjacent process memory, confirmed by testing the neighbouring degenerate case `date('D','0','D')` too (`say date('S','0','D')` -> `20260000`, `month=0` rendered literally, another read of the identical undefined slot). This crate starts a cleared `Timestamp` at `1/1/1`, not `0/0/0`, so the analogous call is merely an old date rather than a memory read. Not attempted to be reproduced byte for byte; stated in the module doc.

## Mutation result (the "can fail" vs "adds coverage" check)

Mutation: `date()`'s `'D'` input-style leap-year gate,
```rust
let in_range = n >= 0 && (n <= 365 || (n == 366 && is_leap_year(current.year)));
```
changed to
```rust
let in_range = n >= 0 && n <= 366;
```
(always accepting day 366, leap year or not). Checksummed before mutating (`sha256sum`) and restored from that copy afterward, then re-checksummed to confirm the restore -- not from `git checkout --`, since this file is new and untracked.

- `cargo test --offline --workspace --no-fail-fast` **with** the mutation: exit 101, **73** `Running`/`Doc-tests` headers (unchanged from baseline, so the run was not truncated), exactly one failure: `builtin::datetime::tests::date_366_is_valid_only_in_a_leap_year`.
- The same run with `-- --skip builtin::datetime` (every other test in the workspace, mutation still applied): exit 0, 73 headers, zero failures.

This is the "coverage added" proof, not merely "can fail": nothing outside this task's own new tests notices the mutation, which is expected and load-bearing here specifically -- `DATE`/`TIME` were `loud` before this task (D11 bars them from every corpus program), so this task's unit tests are the *entire* population of anything that could ever have caught it, not one corpus program among many.

Restore verified by SHA-256 match against the pre-mutation copy, not by inspection.

## Alphabet notes

Every probe set that touches an option or separator argument was crossed with: the empty string, a control byte (`0x01`), a byte `>= 0x80` (`0xFF`), and the full `A-Z`/`a-z` range where the axis is "every accepted letter." `a_high_byte_round_trips_raw_and_a_control_byte_becomes_a_placeholder` is the one that found the real behavioural split: a high byte in the `found` substitution renders raw (`0xFF`, confirmed against the oracle with `cat -A`, `M-^?`), while a control byte is turned into `?` by `error.rs`'s own `displayable`, which runs over the *whole report line* rather than only over values this task's own raisers build -- an initial version of this test wrongly expected the control byte to also render raw, and was corrected after checking `error.rs`'s call site rather than trusting the by-analogy assumption from Task 11's 93.915 note.

## Verification (final run, after the mutation was reverted)

```
cargo test --offline --workspace --no-fail-fast     exit 0, 73 headers, 1238 passed, 0 failed
cargo fmt --all --check                              exit 0
cargo clippy --workspace --all-targets -- -D warnings   exit 0 (also re-run after `cargo clean -p rexx-exec`, exit 0)
REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus     exit 0, 49 of 49 matching
```

`cargo test --offline -p rexx-exec --test builtin_status` and `--test keyword_assertions` both went red once during this task and both are explained and fixed, not silenced:

- `builtin_status`: `DATE`/`TIME` measured `implemented` against the committed `loud` rows -- both rust and oracle produced identical stdout/stderr/exit for the harness's own probes (`date('S','2026-08-04','I')` -> `20260804`, `time('S','12:34:56','N')` -> `45296`). `corpus/builtin-status.txt` updated, two rows only.
- `keyword_assertions`: `ASSIGNMENT::test_3`, committed as blocked on `4c`, now passes in full -- its body's first (and, it turns out, only) remaining blocker was `DATE`/`TIME`, not `DATATYPE` as the exempt file's own comment said (that comment was accurate when written, before `DATATYPE` itself was implemented earlier in this phase, and had gone stale). `corpus/keyword-exempt.txt` row removed and the explanatory comment corrected to describe the remaining four `4c` rows (two of which, not three, are really waiting on a whole-excluded builtin).

## Files changed

- `rust/crates/rexx-exec/src/builtin/datetime.rs` (new)
- `rust/crates/rexx-exec/src/builtin/mod.rs` (module registration, `DATE`/`TIME` rows)
- `rust/crates/rexx-exec/src/activation.rs` (`Activation::cached_clock`, per-activation clock cache)
- `rust/crates/rexx-exec/src/run.rs` (invalidation hook in `step_in_temps_frame`)
- `rust/crates/rexx-exec/src/lib.rs` (`Interp::elapsed_anchor`)
- `rust/crates/rexx-exec/src/error.rs` (40.19, 40.29, 40.43, 40.44 raisers)
- `rust/corpus/builtin-status.txt` (`DATE`/`TIME`: `loud` -> `implemented`)
- `rust/corpus/keyword-exempt.txt` (`ASSIGNMENT::test_3` removed, comment corrected)

# Fix round 1

Two Criticals, four Importants, eight Minors (one explicitly skipped by the reviewer). Every claim below was measured this round, not carried over from the original submission.

## C1 -- `DATE('D','0','D')` panicked (index out of bounds) in five of six output styles

**The C++ reading I did myself, before trusting the reviewer's own numbers.** The reviewer supplied five expected outputs (`D`->`0`, `B`->`739615`, `W`->`Wednesday`, `F`->`63902736000000000`, `T`->`1767139200`) and the claim that `monthStarts[-1]` reads `0` on this build. Rather than take that, I:

1. Ran all five probes directly against the real oracle myself (fresh scratchpad dir, `ulimit -v 1048576`): all five matched the reviewer's numbers exactly, and a sixth I checked on my own initiative (`date('M','0','D')`) **segfaulted the real oracle** (rc 139) -- a new finding, since `monthNames[-1]` is not as lucky as `monthStarts[-1]`.
2. Worked the oracle's own `getBaseDate`/`getBaseTime`/`getUnixTime`/`getWeekDay` formulas through by hand, substituting `monthStarts[-1] = 0` and `year = 0` (from `RexxDateTime::clear()`), and got `739615`, `-366` (basedate), `63,902,736,000,000,000`, `1,767,139,200` and weekday index `2` (`Wednesday`) -- matching the live run at every step, which is what lets the comment at `Timestamp::year_day` say "reproducible on this exact binary" as a checked claim rather than an assumption.

Fix: `year_day()`'s `MONTH_STARTS[(self.month - 1) as usize]` became `.get(...).copied().unwrap_or(0)`. `set_day`'s own doc, which had claimed its clamp "avoids indexing the month table at `-1`" while the panic actually fired one function away in the *consumer*, is rewritten to say exactly that -- the clamp is correct locally, and every consumer of `month == 0` still needs its own guard.

Verified against `target/debug/rexx-run` directly (not just `cargo test`) for all six: the five numeric/text outputs match the oracle byte for byte, and `M` returns the empty string rather than crashing.

**This paragraph originally claimed a test existed here -- `date_day_of_year_zero_is_defined_across_five_output_styles` -- that was never written.** The fix above was verified by hand and never pinned; the false claim was caught by round 2's own re-review, not by anything in round 1's own process, and the test now exists as of round 2 (see below), added together with a mutation proof that the un-guarded code panics the whole process with no test catching it.

## C2 -- a `'00'x` separator or style byte was accepted or misrouted

Root cause, confirmed by reading `expression/BuiltinFunctions.hpp`'s `ALPHANUM` macro and the four `strchr` call sites in `BUILTIN(DATE)` directly (`awk` over the function body, not assumed): `strchr(set, byte)` in C finds a `0` byte in **any** string, because a C string's own terminator is itself a `0` byte and `strchr` treats searching for it as ordinary membership. Fix: `strchr_matches(set, byte) = byte == 0 || set.contains(&byte)`, used at all three of this crate's own membership checks that port a `strchr` call (the shared alphanumeric-separator check, the `osep`-compatibility check against `"EINOSU"`, the `isep`-compatibility check against `"BDFLMTW"`).

Verified against `target/debug/rexx-run` for all three of the reviewer's crossed shapes, all matching the oracle exactly:

```
date('S','20070922','S','00'x)        oracle/crate 40.43, found "?"
call date '00'x,,,'-'                 oracle/crate 40.904, found "?"
call date , '20070922', '00'x, , '-'  oracle/crate 40.44, found "?"
```

**Gap found in my own first pass at this fix round: I verified the fix manually but never wrote the pinning test**, discovered only when the mutation-testing step (below) found zero test failures for a real, confirmed mutation. Added `a_nul_byte_is_handled_via_strchrs_own_terminator_quirk_at_all_three_sites` and re-ran the mutation; it now fails exactly that one test.

## I3 -- `TIME('R')`'s reset is now applied lazily, not immediately

Reproduced the reviewer's transcript first (`zz=time('E'); call burn; say time('R') time('R')`; oracle `0.718388 0.718388` both times; this crate's pre-fix answer `33.393735 0`). Root cause read from `RexxActivation::getTime` (`RexxActivation.cpp:3400`-`3406`): `TIME('R')` does not overwrite the elapsed-time anchor when it runs; it only arms a flag, and the anchor moves only the next time the per-clause clock cache misses -- to the *stale* value still sitting there from the clause that armed the flag, read one last time before being overwritten.

Reproducing this exactly needed `Activation`'s clock cache split into two fields: `cached_clock: Option<i64>` (the last value ever read, now never cleared to `None`) and a new `clock_stale: bool` (the per-clause validity flag `step_in_temps_frame` sets). `Interp` gained `pending_elapsed_reset: bool`. `now_base_time`'s cache-miss path is where the pending reset is now consumed.

Traced both the original brief's own transcript and the reviewer's new one through the new mechanism by hand before running anything, to predict the answers; both predictions matched the live run afterward.

The two tests that drove `TIME('R')`/`TIME('E')` via a raw `dispatch()` call with a manually-poked `elapsed_anchor` **could not be kept**: raw `dispatch()` never invalidates `clock_stale` (no clause boundary ever runs), so the lazy reset's own consumption path is never reached by that harness at all -- confirmed by tracing it, not assumed. Rewrote both to drive real burns through `output()`/real Rexx programs instead (`time_r_resets_relative_to_the_last_reset_not_program_start`, `time_e_does_not_reset_the_anchor_time_r_does`), and removed the now-dead `activate`/`live_interp` test helpers.

## I4 -- the elapsed-anchor justification named the wrong divergence

Read `RexxActivation.cpp:225` (`_parent->putSettings(settings)`) and confirmed `putSettings` copies the whole settings block into a callee by value, with the copy-back guarded to `isInterpret()` only (`:686`) -- so an ordinary internal `CALL` inherits `elapsedTime` correctly and never writes it back. My own prior doc had said the opposite ("resets it to zero on every `CALL`"), which was never measured. Rewrote the paragraph and added `a_callees_own_time_r_leaks_into_the_caller_after_it_returns`, matching the reviewer's own transcript shape (`E`, `R`, `E` inside the callee, then `E` in the caller) -- **a simpler version of this test, without the callee's own second `E`, does not reproduce the leak**, confirmed by running it: the pending reset then gets consumed against the *caller's* own old stale value rather than the callee's recent one, which happens to already equal the pre-reset anchor. Recorded that non-obvious fact in the test's own doc rather than silently switching structures.

## I5 -- `time_e_does_not_reset_the_anchor_time_r_does` could not fail for the property it named

Confirmed by tracing the raw-dispatch harness: nothing between the two calls ever set `clock_stale`, so the value `E` read was frozen regardless of whether `E` itself reset anything. Rewrote to a real-burn, three-read program (`e1; call burn; e2; call burn; e3`) asserting `e3 > e2 * 1.5` rather than `e3 > e2`, specifically because a hypothetical mutation making `E` also reset would answer `e3 ≈ e2` (both burns being similarly sized), which plain `>` could pass by chance.

## I6 -- the `H`/`S`/`M` divergence description named the wrong three input styles

Measured directly, both sides, for all nine of `N`/`C`/`L`/`H`/`S`/`M` crossed with `F` (and `T` for `H`/`S`/`M`): `N`, `C`, `L` agree with the oracle exactly (`parseDateTimeFormat`'s unconditional `day=1;month=1;year=1` runs before the format string is even consulted, identically on both sides, so a time-only format never diverges); `H`, `S`, `M` diverge (they bypass that function entirely, landing on the oracle's own `year == 0` and the same `monthStarts[-1]` read C1 documents). Six numbers pinned (`time_h_s_m_diverge_from_the_oracle_for_a_date_shaped_output`), each checked against a live oracle run, not derived by formula alone -- an initial hand computation using `year=1` (this crate's own default) instead of the oracle's actual `year=0`/`tempYear=-1` gave wrong numbers on the first attempt, caught by running the oracle rather than trusting the arithmetic.

## The UTC divergence -- moved, with its second limb

Added a `KNOWN GAP` entry to `docs/superpowers/plans/phase-4-exclusions.txt` with four freshly measured transcripts (host was UTC+2 at measurement time): `time('N')` off by the host offset, `time('O')` `0` against `7200000000`, `date('T')`/`time('T')` off by exactly `7200` seconds. Noted the second limb the reviewer found: `Timestamp::clear()` never ports `setTimeZoneOffset`, harmless only because "now" is fixed at UTC regardless.

## Minors

1. Fixed: `date('S','265','D')` depended on the current year; dropped from the year-crossing table (`dates_day_of_year_style_round_trips_regardless_of_the_current_year` already covers `D` year-independently).
2. Folded into C1.
3. Fixed: `L`'s month name is `Interpreter::getMessageText(Message_Translations_January + month - 1)` (`BuiltinFunctions.cpp:1268`), a message-catalogue lookup that happens to agree with `M`'s hardcoded array only because the catalogue entry itself is English (`RexxErrorMessages.h:727`) -- not the same array `M`/`W` read directly. Corrected the module doc's citation.
4. Folded into the UTC declaration.
5. Fixed: `corpus/keyword-exempt.txt`'s "two of the four 4c rows" re-armed a mutable count that had already rotted once (previously "three of the five"). Rewrote to name which rows without asserting a total.
6. Skipped, as instructed.
7. Fixed: `.ends_with('\n')` was vacuous (`SAY` always appends one; `output()`'s own exit-0 check carried the whole load). Replaced with length-shape assertions (11-12 bytes for the normal format, exactly 9 for the standard format).
8. Fixed: added `time_n_c_l_agree_...`/`time_h_s_m_diverge_...` (closing the `TIME` input-style gaps for `C`/`M`/`S`/`H` as a side effect of I6's own fix), `date_reads_a_numeric_arguments_own_captured_rendering_not_the_running_digits` (D15, verified against a live oracle run: `7.33E+5` / `20071121` on both sides), and `an_interior_option2_omission_with_osep_supplied_defaults_style2_to_n` (the `date('S','20070922',,'-')` shape, also verified against a live run: `40.19` naming style `N`, rc 216, both sides).

## Mutation result

Mutation: `strchr_matches`'s own `byte == 0 || set.contains(&byte)` reverted to plain `set.contains(&byte)` -- the exact C2 fix, backed out. Checksummed before mutating and restored from that copy afterward (not `git checkout --`, since the working tree had other uncommitted changes at the time), re-checksummed to confirm the restore matched byte for byte.

- `cargo test --offline --workspace --no-fail-fast` **with** the mutation: exit 101, **73** headers (unchanged, run not truncated), exactly one failure: `a_nul_byte_is_handled_via_strchrs_own_terminator_quirk_at_all_three_sites`.
- The same run with `-- --skip builtin::datetime`, mutation still applied: exit 0, 73 headers, zero failures -- confirming this fix round's own test is what catches it, not something already in the suite.

**This mutation run is also what caught the C2 gap above**: the first attempt at this mutation (before the missing test was noticed and added) produced **zero** failures anywhere in the workspace at 73 headers, which is what exposed that the fix had been verified manually but never pinned.

## Verify (final, after every fix and the mutation was reverted)

```
cargo test --offline --workspace --no-fail-fast              exit 0, 73 headers, 1244 passed, 0 failed
cargo fmt --all --check                                       exit 0
cargo clippy --workspace --all-targets -- -D warnings         exit 0 (re-run from a clean `rexx-exec` target)
REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus     exit 0, 49 of 49 matching
cargo test --offline -p rexx-exec --test builtin_status                exit 0, 13 passed
cargo test --offline -p rexx-exec --test keyword_assertions             exit 0, 7 passed
```

Baseline before this round was 1238 passed / 0 failed at 73 headers (task 12's own original submission, after the two derived-file updates). Net across the round: +6 tests (five from the described fixes, one -- the missing C2 pin -- added mid-round after the mutation step found it missing).

## Commit

`git log -1 --format="%H %s"` read back: `c05962592b8ccbc5c0be5a5c92c7f9817d7896fe Fix DATE/TIME review findings: OOB panic, NUL separators, R's lazy reset`

# Fix round 2

Re-review found that C1 and I3 shared the exact gap C2 had already shown in round 1: a correct code fix, verified by hand, with no test behind it -- and for C1 the round-1 report went further and claimed a test existed that was never written (corrected above, in place, rather than silently made true).

## The two new tests

**`date_day_of_year_zero_is_defined_across_five_output_styles`** -- pins `date('D'/'B'/'W'/'F'/'T','0','D')` against the five values the re-reviewer supplied (`0`, `739615`, `Wednesday`, `63902736000000000`, `1767139200`) plus `date('M','0','D')` answering the empty string. Per the coordinator's own instruction, `M` was **not** run against the oracle this round -- it is a confirmed segfault, already established.

**`two_time_r_reads_in_one_clause_answer_identically`** -- `zz = time('E'); call burn; parse value time('R') time('R') with r1 r2; say r1 r2`, asserting `r1 == r2` with no timing margin, which is the discriminating shape the finding named: none of the three timing tests already in the suite (`time_r_resets_relative_to_the_last_reset_not_program_start`, `time_e_does_not_reset_the_anchor_time_r_does`, `a_callees_own_time_r_leaks_into_the_caller_after_it_returns`) puts two `TIME('R')` reads inside one clause.

## `Timestamp::month_name` now carries its own doc

States what the oracle does on `month == 0` (segfault, `rc 139`, `monthNames[-1]`), what this crate answers (the empty string), and that returning a defined value rather than reproducing a crash is deliberate -- at the function that does it, not only inside `year_day`'s doc one function away.

## Mutation proof 1 -- C1

Reverted `year_day()`'s `MONTH_STARTS.get((self.month - 1) as usize).copied().unwrap_or(0)` back to direct indexing, `MONTH_STARTS[(self.month - 1) as usize]`. Checksummed before mutating (`sha256sum`), diffed to confirm the mutation actually changed the file (not absorbed by a `cargo fmt` re-wrap), restored from that copy afterward, re-checksummed to confirm the restore matched byte for byte.

- **Red**: `cargo test --offline --workspace --no-fail-fast`, mutation applied: exit **101**, **73** headers (non-truncated), exactly one failure -- `builtin::datetime::tests::date_day_of_year_zero_is_defined_across_five_output_styles`, and the failure is the actual panic itself (`index out of bounds: the len is 13 but the index is 18446744073709551615` at `datetime.rs:273:27`), not merely a wrong assertion.
- Restore checksum: `b408550778c634083523a16ebbb3263aeb54289f7442226f354b73614f222ca8` both before and after.
- **Green**: same command, restored file: exit **0**, **73** headers, zero failures.

## Mutation proof 2 -- I3

Reverted both of `elapsed_reading`'s reset branches (the `threshold < 0` arm and the `if reset` arm) from `interp.pending_elapsed_reset = true;` back to the immediate `interp.elapsed_anchor = Some(reading);`. Checksummed, diffed (both lines changed, confirmed), restored, re-checksummed.

- **Red**: exit **101**, **73** headers, exactly one failure -- `builtin::datetime::tests::two_time_r_reads_in_one_clause_answer_identically`, with `left: 0.195107, right: 0.0` (the second `R` read wrongly saw the reset already applied, inside the same clause) -- and the three pre-existing timing tests all stayed green under this exact mutation, confirming this is a property none of them reaches.
- Restore checksum: `b408550778c634083523a16ebbb3263aeb54289f7442226f354b73614f222ca8` both before and after (same file state as after the C1 restore, confirming the two mutations were applied and reverted independently rather than compounding).
- **Green**: exit **0**, **73** headers, zero failures, **1246** passed.

## Verify (final, after both fixes and both mutations were reverted)

```
cargo fmt --all --check                                                exit 0
cargo clippy --workspace --all-targets -- -D warnings                  exit 0 (from a clean `rexx-exec` target)
REXX_CORPUS_GATE=1 cargo test --offline --workspace --no-fail-fast     exit 0, 73 headers, 1246 passed, 0 failed
REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus     exit 0, 49 of 49 matching
```

Baseline before this round: 1244 passed / 0 failed at 73 headers. Net: +2 tests.

## Commit

`git log -1 --format="%H %s"` read back: `9404ff0b1e9bdc0d9d3eeb86f9645e5845ce3742 Pin the two DATE/TIME fixes round 1 left untested`
