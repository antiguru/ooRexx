# Phase 9a Task 7 review: package setting writes

BASE `e0f3ee495`, HEAD `40278c99e`. Reviewer: opus, read-only. Probes ran from fresh directories
`/tmp/claude-1000/p9/t7rv-a`, `-b`, `-c`, with the oracle under the standard wrapper and a crate
`rexx-run` built at HEAD (`CARGO_TARGET_DIR=/tmp/claude-1000/p9/t7-review-target`, a `Compiling
rexx-exec` line seen, the directory deleted afterwards). `oracle-crashes.txt` was read first. Entry 4
(DIGITS above 1000 in arithmetic) and entry 36 were kept out of every probe.

## Spec verdict: COMPLIANT

Step 1: both witnesses write every option name the setting switch takes, read the value back and
print the answered previous value. They show the effect on code started afterwards (`::routine calc`
and the called/loaded files) for digits, form, fuzz, trace and numeric inherit, and they cover the
`'C'` count of 1 that `rexx -o` sets (`utilities/rexx/platform/unix/rexx.cpp:103`, `:208`-`:215`:
count 1, or -1 for `-od`, then `D` then `C`, which matches the witness header). Step 2: the
constructor is deleted, and the only references left are in historical records and the plan. The
report claims Package_Options at 54/54. Per the brief I did not rerun the suite.

## Quality verdict: APPROVED WITH FIXES (one Important, prose only)

## Focus answers

1. **`Settings::from_parts`.** It has three callers. The `D` write runs `numberArgument`, then
   `< 1` (93.907), then `digits <= fuzz` (33.1), in the same order as `PackageClass.cpp`'s digitsFlag
   arm. The `FU` write runs `nonNegativeArgument` (93.906), then `fuzz >= digits` (26.900). Both
   witnesses print each of these on both sides. The third caller, `take_defaults`, *does* build
   `fuzz >= digits`: take a package whose own `::OPTIONS DIGITS 3` is explicit, under an override of
   `::options fuzz 5`. The C++ does the same thing (`setDigits`/`setFuzz` with no comparison,
   `PackageClass.cpp:2739`-`:2750`). `rexx-num`'s only consumer of the pair is
   `compare.rs:197`-`:204`, which already handles `fuzz > digits` and gives `fuzz == digits` working
   digits 0. Probe `t7rv-a` ran comparisons at digits 3 / fuzz 5 and digits 5 / fuzz 5, plus a bare
   `NUMERIC DIGITS` (33.1). Oracle and crate matched on stdout, stderr and rc 0. No digits of 0 can
   be reached. Large digits (`1000000000`, `123456789012`) are accepted and read back identically on
   both sides (`t7rv-c`). The corpus does not witness this state (Minor 3).
2. **Per-Interp vs process-wide statics.** One `Interp` holds every activity, so a write from one
   activity is seen by another. Probe `t7rv-c` sets `D`/`C` inside a `~start`ed method, then compiles
   on the main activity and on a second started activity: identical on both sides (`7 ENGINEERING`,
   counts -2 and -3). The C++ statics are wider only across several interpreter instances in one
   process. This crate has no way to create a second instance, and ooTest runs one per process. The
   per-Interp reading fits the isolated-interpreter design, and I found nothing ooTest can see.
3. **Effect on later-loaded programs.** I ran both witnesses myself from a fresh directory.
   `package_default_options.rex`: oracle rc 0, crate rc 0, stdout byte-identical, stderr empty on
   both. `package_option_writes.rex`: rc 0 on both, stdout identical, and stderr identical (the
   `>I>`/`<I<` lines of `TRACE L` on `CALC`). In the stdout I confirmed these lines are printed:
   `called: 6 ENGINEERING 2 N ... NOPROLOG`, then `called again: 9 SCIENTIFIC 2 N ...` (count 1
   consumed by one file); `req prologue ran` absent under the override's NOPROLOG;
   `noprolog prologue ran` present under the override's PROLOG; and the 98.986 / 98.972 escalations.
4. **PROLOG/NOPROLOG.** `DirectiveParser.cpp:1311`-`:1327`: each subdirective clears the other's
   explicit flag. `options.rs`'s `apply` does the same. Both orders are witnessed
   (`NOPROLOG then PROLOG names [PROLOG]`, `PROLOG then NOPROLOG names [NOPROLOG]`).
5. **Hot path.** No activation-setup file is in the diff, so `start_from_package` is unchanged.
   `override_package_settings` runs once per package compile: `install_directives` (`install.rs:359`)
   and `record_compiled_body`/`record_compiled_routine` (`:1681`, `:1816`), which are reached only
   from `compile_method_source_in`/`compile_routine_source`. It is one `count == 0` branch.
   `settings_of`'s new REXX-package clone runs only inside the `Package` reader methods. Nothing new
   runs per clause or per call.
6. **Refusal.** The `Loud::package_option_write` row is gone from `refusal-sites.tsv`.
   `whole_number_user_defined` was added, and `method_user_defined` became `body+send`. The rest are
   line shifts. `git grep` finds the name only in historical records and the plan. The
   `phase_ten_reasons` sample was moved to a literal that still exists.
7. **Prose.** No em-dashes in added lines. See Important 1 and Minor 1 for sentences this change
   made false.

## Findings

### Critical

None.

### Important

1. **Sentences made false by the new latch write.** `rust/crates/rexx-exec/src/install.rs:263`
   says "`Interp::lostdigits_armed`'s only write". Since this change, `Interp::arm_option_latches`
   (`options.rs:543`) also writes it. `lib.rs:1655`-`:1656` documents `lostdigits_armed` as "Whether
   any program in this run installed `::OPTIONS ... LOSTDIGITS SYNTAX`", but a
   `p~options('L', 'S')` write now arms it too. **Failure scenario:** someone working on the
   arithmetic gate takes "only write" at its word and derives the gate from install-time state. The
   witness's `lost()` (`98.972` after `L S`, with no `::OPTIONS` anywhere) would then answer
   `1.2346E+6`. The global constraint says a false sentence is deleted. Fix: delete "'s only write"
   or name both writers, and widen the field doc to "installed, or a `Package~options` write set".

### Minor

1. `rust/corpus/introspection-arguments.tsv:11`-`:14` still lists `Package~defaultOptions` among
   "This phase's own `loud` rows". That is no longer true. Delete the name from that list.
2. `Settings::from_parts` is `pub` and unchecked. Today all three callers are correct (see focus 1),
   but nothing in the type stops a later caller passing digits 0. Consider `pub(crate)`-like
   narrowing. That is not possible across crates, so the doc could instead say the caller owns the
   `digits >= 1` check.
3. Not witnessed in the corpus: an override whose FUZZ is at or above an explicit package DIGITS
   (probe `t7rv-a`, which agrees with the oracle), an override that names `NUMERIC INHERIT` (the
   witness only carries the default NOINHERIT through `take_defaults`'s `NUMERIC` branch), and the
   cross-activity sharing (probe `t7rv-c`, agrees). Each is a few lines in
   `package_default_options.rex` if the controller wants them pinned.
4. Observation outside this task: concern 3's unsubstituted `&1` is wider than `Routine~new`. The
   main program `call .Package~defaultOptions('D', '::options fuzz 5')` (a parse error) prints
   `Error 35.1:  Incorrect expression detected at "&1".` on the crate's stderr, where the oracle
   prints `"~"`. The parser is not in this diff, so this predates the task. It belongs to the 9d
   ledger beside concern 3.
5. The checks overran the 5-minute budget, and the harness, sourceline and `--lib` runs were taken
   before the final edit. The ledger already records this. My witness runs at HEAD cover the
   behavioural part of that edit.
