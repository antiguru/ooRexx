# Phase 9c Task 5 report: `rexx` beside `rexx-run`, and the A/B

Implementer: c9c-impl-5. Base `6391ead7c`. Code commit `629f2d79b`. This report and the A/B
record are the commit after it.

## Status

Done, with three items for the controller (Shared paths 1, Concerns 1 and 2). No `OVER` in the
A/B. The rulings asked of main were not answered while I worked, so each default is stated
where it applies.

## Step 1: oracle measurements

Transcripts are in `docs/superpowers/records/2026-10-15-phase-9c/task-5/step1/`:

* `fixture.sh` writes the programs, `measure.sh BIN DIR` runs every shape, `run1.sh` runs one
  shape under the oracle wrapper and prints stdout, stderr and rc separately.
* `oracle.txt` is the oracle's run from a fresh directory. `crate.txt` is the crate's `rexx`
  over the same shapes, built before the `lossy` marking and the final refusal-constructor
  change.
* `probes.txt` holds `.local` under a file run and under `-e`, and the `merge.rex` interleave
  probe (see Concern 1).

What the measurements settled (all in `oracle.txt` unless noted):

* No arguments, `-e` as the last word, `-o`/`-od` as the last word, a lone `-x` or `-`: the usage
  text on stderr, rc 255.
* `-v`, `-V`, `-vx`, `-? -v`, `-v -?` and `-o 'digits 12' -v` all print the banner, rc 0. After
  the program name or after `-e TEXT`, `-v` is an argument.
* `-e`: `PARSE SOURCE` is `LINUX COMMAND INSTORE`, errors read `Error N running INSTORE line 1`,
  `.local` has no `SYSCARGS` entry (`.local~items` 9 against 10, `probes.txt`), so `.syscargs`
  renders `.SYSCARGS` and `.syscargs~items` is 97.1. `-o` before or after `-e` is ignored. The
  status is RexxStart's `short`: `return 70000` and `return 2147483648` give 0, `return -3` 253,
  `return 300` 44 (`RexxStartDispatcher.cpp:174-179`).
* `-o STR`: the directive is `STR` when the byte after its first non-blank is `:`, otherwise
  `::OPTIONS STR`. `:options ...` therefore gets the prefix (25.924 found `:`), `x:options ...`
  does not (93.900 not an ::OPTIONS directive), and one or two leading blanks before `::` work.
  The byte can lie past `STR`: `-o '' ':x'` sends the empty string (93.900, argument 2 must not
  be empty) and `-o '  ' ':x'` sends the blanks (no error, then 3.901). A 600-blank string is
  cut at 511 bytes, dropping a trailing `form engineering`. A failing directive is reported
  before the program is looked for, with no position when no site has a line.
* `-o` sets count 1: the program takes the override and a `::REQUIRES` does not. `-od`, `-oD`,
  `-Od` set -1, and the required package takes it too. A program's own `::OPTIONS` wins over
  both.
* `.SYSCARGS` is indexed from `argc_base - 1`: `-x p a b` gives size 3 with index 1 empty,
  `-o 'digits 12' p a b` size 4 with 1-2 empty, `-x -y` size 4. With no words after the program
  the array is empty whatever the shift.
* Search: `t` finds `t.rex`, `UP` finds `UP.REX`, `sub/t`, `./sub/t`, an extensionless file,
  `REXX_PATH` and `PATH` entries. A directory named `dironly.rex` is not a program. Not found
  is `Error 3:` then `Error 3.901:` with the typed name, rc 253, no `running` line. An
  unreadable file is `Error 3.1:` naming the resolved absolute path, rc 253.
* The return table of scout C section 1 reproduces: `'abc'` 0, `-3` 253, `exit 300` 44, `1.5` 0,
  `2147483648` 0, `-2147483649` 0, `''` 0, none 0, `255` 255, `256` 0, `' 4 '` 4, `1e1` 10, an
  array 0.
* Review Focus 4: `my prog.rex` and `my prog` are found and reported with the blank.
  `p'ff'x.rex`, `p'ff'x` and `q'ff'x` are found and the oracle reports the `'ff'x` byte in
  `PARSE SOURCE` and the `running` line. Arguments are bytes: `a'ff'xb` reaches `ARG(1)` and
  `.SYSCARGS[1]` as `61FF62`.
* `TEST_PASSINGARGS5` asserts `rexx -? showArgs Ho Ho` gives two `.SYSCARGS` items and
  `[Ho, Ho]` (items and `toString`, so it passes with or without the index shift).
  `TEST_PASSINGARGS7` asserts nine merged lines in program order for
  `rexx -o "trace all" showArgs -o "my output file"`.

## Step 2: what was built

* `src/launcher.rs` (public): `parse`, a literal port of `rexx.cpp:81-150`; `resolve_program`
  on `require::search_entries`, `candidates` and `first_regular`; `version_banner` from
  `version.rs`'s `VERSION_NUMBER`, `BUILD_DATE` and `VERSION`; `program_not_found` and
  `program_unreadable`, positionless 3.901 and 3.1; `instore_status`; and
  `Interp::send_default_options`, which sends `DEFAULTOPTIONS` `('D', ...)` then `('C', count)`
  to `.PACKAGE` through `send_message`, so the crate's own `default_options` method does the
  checking.
* `resolve_program` widens each byte to the `char` of the same value before calling the `str`
  functions in `require.rs` and narrows the hit back, so a non-UTF-8 name is searched without
  loss and `require.rs` is unchanged. `to_ascii_lowercase` leaves chars above 127 alone, which
  is `strlower`'s rule.
* `src/bin/rexx.rs` keeps `rexx-run.rs`'s `main` (signal handlers, `ProgramInput::Stdin`, the
  transient flags, the sinks, `REXX_SWITCH_MODE` with its `rexx-sim:` lines and status 2, the
  canonical reported path, the 8-bit truncation). Arguments stay bytes (`OsStringExt::into_vec`)
  and the program runs through `run_program_named`. When `-o` is given and the program is not
  found or not readable, an empty program runs first so that the options' failure is the one
  reported, as on the oracle.
* `Invocation` gains `with_default_options`, `without_syscargs` and `with_syscargs(words,
  index)`. The last carries `.SYSCARGS`' words separately from the joined argument, because
  `rexx.cpp` builds the two from different ranges of `argv`.
* `run_program_named(path: &[u8], ...)`. `run_program(&str, ...)` delegates through
  `as_bytes()`, so its callers are untouched. `execute_on` takes `&[u8]`, and the five unit-test
  call sites that call it directly now pass byte literals.

## Step 3: tests

* `tests/rexx_command.rs` with `tests/rexx_command/cases` (datadriven). Each case runs the
  oracle and `CARGO_BIN_EXE_rexx` with the same words from fresh fixture directories, and the
  three descriptors must agree. The recorded block is the oracle's, with the fixture as `DIR`
  and the banner's `Build date:` and copyright year masked. The cases are the Step 1 shapes
  plus the unreadable-file pair and `-o 'bogus 9' nonexist`.
* Mutation, by hand: `_ => argc_base += 0` in `parse` reddens the case set at every `-x`, `-`,
  `--` and `-?` shape. Restored by exact edit and checked with `cmp` against a copy.
* `interpreter_bin` (`tests/support/group_runner.rs`) and `check_path` and
  `the_crate_side_finds_its_own_rexx_and_no_rexxc` (`tests/suite.rs`) now name
  `CARGO_BIN_EXE_rexx`. `group_runner.rs`'s sim-mode `rexx-run` process is Task 6's and was left.

## Step 4: checks

Every command from `rust/`, statuses unpiped into
`docs/superpowers/records/2026-10-15-phase-9c/task-5/checks-statuses.txt` at `629f2d79b`:

| Command | Status | Result |
|---|---|---|
| `cargo fmt --all --check` | 0 | |
| `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | 0 | |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test rexx_command` | 0 | 1 passed |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite` | 0 | 52 passed, `the_crate_side_finds_its_own_rexx_and_no_rexxc` among them |
| `... --test refusal_sites` | 0 | 5 passed |
| `... --test owners` | 0 | 6 passed |
| `... --test closed_phases` | 0 | 8 passed |
| `... --test phase_ten_reasons` | 0 | 5 passed |
| `... --test representation_refusals` | 0 | 3 passed |
| `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` | 0 | 1065 of 1065 matching, 29 passed |
| `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --lib` | 101 | 1141 passed, 1 failed (Concern 2) |

`corpus/refusal-sites.tsv` was re-derived with `REXX_REFUSAL_SITES_REFRESH=1` in the code commit.
`git diff 6391ead7c 629f2d79b -- rust/corpus/refusal-sites.tsv` shows only `lib.rs:` line numbers
moving.

Earlier in the session, before the commit, `--test suite` failed twice out of nine runs at
`a_command_a_test_leaves_running_is_killed_with_the_run` ("the command outlived the run"). Six
runs of that test in a row then passed with no `sleep 41.5` left in `/proc` after each, and the
full binary passed at the commit. Its check scans `/proc` machine-wide, so another session's
suite run can trip it. I did not establish the cause.

**The rexx_command group**, from `rust/`:

```
REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=utilities/rexx/rexx_command \
  REXX_SUITE_TABLE=/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-15-phase-9c/task-5/suite/table.tsv \
  CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite
```

Status 101. Expected 30, oracle 30 of 30 passed, crate 29 of 30. The one red is
`TEST_PASSINGARGS7` (Concern 1). The table, `tests.tsv` and `verdicts.tsv` are in `task-5/suite/`.
Scout C measured 22 failures with `rexx` linked to `rexx-run`, so this task turns 21 of them
green.

## Step 5: the A/B

From `rust/` at `629f2d79b`, after
`CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 --bin rexx --bin rexx-run` and the two
copies:

```
bench-programs/callgrind.sh -B 0.5 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9c/ab \
  pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run \
  base=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9c-629f2d79b \
  head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-9c-629f2d79b
```

Exit 0, no `OVER`, no `SPREAD`. sha256s from `binaries.txt`: pin `c4b4e8c4...523c67` (the one
the 9b1 records cite), base `f6142be3...a3aa3a`, head `a665131e...a3241c`. The table, `summary.tsv`
and `binaries.txt` are copied to `docs/superpowers/records/2026-10-15-phase-9c/ab/`.

Whole-process Ir against the pin. `base d%` is `rexx-run` at this commit and `head d%` is
`rexx`. `head - base` is the front end's share:

| program | base d% | head d% | head - base Ir | head - base % |
|---|---|---|---|---|
| alloc | -0.9814 | -1.0864 | -23972196 | -0.1061 |
| alloc4c | -0.1087 | -0.1079 | 27496 | +0.0007 |
| arith | -0.1861 | -0.1858 | 28336 | +0.0002 |
| assign | +0.0006 | +0.0007 | 27391 | +0.0001 |
| churn | +0.1466 | +0.1510 | 28015 | +0.0044 |
| compound | +0.1043 | +0.1046 | 26856 | +0.0003 |
| decloop | +0.0633 | +0.0645 | 27842 | +0.0011 |
| decrender | -0.2258 | -0.2252 | 28164 | +0.0006 |
| dirread | -2.2411 | -2.2402 | 28379 | +0.0009 |
| dispatch | -0.2935 | -0.2934 | 28478 | +0.0001 |
| dispatchclass | -3.4680 | -3.4678 | 27386 | +0.0002 |
| emptyloop | +0.0014 | +0.0017 | 27861 | +0.0004 |
| extcall | -13.2007 | -13.2004 | 27580 | +0.0004 |
| fibcall | -3.4723 | -3.4720 | 28480 | +0.0003 |
| fibfunc | -3.4816 | -3.4813 | 27764 | +0.0003 |
| hashnew | -0.4755 | -0.4748 | 17148 | +0.0007 |
| heapshape | -0.6492 | -0.6481 | 27481 | +0.0011 |
| intcall | -10.1013 | -10.0980 | 27224 | +0.0036 |
| interpret | -0.3446 | -0.3568 | -513049 | -0.0122 |
| nop | +0.0010 | +0.0013 | 28124 | +0.0003 |
| parse | -0.1100 | -0.1084 | 27711 | +0.0017 |
| pkglocal | -6.5444 | -6.5437 | 27545 | +0.0007 |
| sayloop | +0.0481 | +0.0632 | 27119 | +0.0151 |
| sendloop | -0.1364 | -0.1362 | 28223 | +0.0002 |
| startup | +0.0452 | +0.0685 | 27754 | +0.0233 |
| strings | -0.2697 | -0.2696 | 27889 | +0.0001 |
| textnum | -0.0591 | -0.0568 | 28024 | +0.0023 |
| varlookup | +0.0006 | +0.0008 | 27876 | +0.0002 |
| rexxcps | -0.0777 | -0.0776 | 27681 | +0.0001 |
| pingguard | -0.1694 | -0.1668 | 33031 | +0.0025 |
| pingmsg | -1.0252 | -1.0289 | -273823 | -0.0036 |
| pingsem | -0.2300 | -0.2281 | 24659 | +0.0019 |

`head - base` was derived from `table.txt` by
`awk -F'\t' 'NR>1 && $1!~/^#/ {d=$4-$3; printf "%s\t%d\t%+.4f\t%d\t%+.4f\n", $1, d, 100*d/$3, $9-$8, 100*($9-$8)/$8}' table.txt`,
saved with its libc-subtracted columns as `ab/head-minus-base.tsv`. The front end's cost is a
fixed 27-28 thousand Ir per run (15-22 thousand with libc and ld subtracted). That is the
search's `stat`s and the option scan. The largest relative share is `startup`'s +0.0233%. The
three negative rows (`alloc`, `interpret`, `pingmsg`) are libc: their libc-subtracted
differences are +17206, +17480 and +15886, in line with every other row. The largest head-vs-pin
deltas are `churn` +0.1510%, `compound` +0.1046%, `startup` +0.0685%, `decloop` +0.0645% and
`sayloop` +0.0632%. `base` carries the same deltas, so they belong to the tree since the pin and
not to `rexx`.

## Shared paths

1. **Not done (default (B), asked of main, no answer):** an `'ff'x` program name is found and
   reaches `call_context.name` as bytes, but `PARSE SOURCE` and the `Error N running <path>` line
   still go through `String`. `execute_on` converts lossily at one marked line (`lib.rs:3726`). The consumers needing a change outside `lib.rs`:
   `Interp::program_path: String` (`lib.rs:1707`) and its readers `install.rs` (`package_path ->
   &str`, `program_display_name`, the `cyclic_inheritance` site), `scheduler.rs:2102`,
   `condition.rs:433`; `required_paths: Box<str>` (read at `run.rs:3129`); `error.rs`'s
   `ClauseSite::path: &str` (`:2221`), the `String::from_utf8_lossy(package)` in `Raised::report`,
   and `Raised::cyclic_inheritance` and `circular_requires(&str)`. `rexx_command.rs` pins the
   divergence: its three `run lossy` cases expect the oracle's bytes with `\xFF` as U+FFFD.
   Removing the marker is the check for whoever lands the change.
2. **Done (default (A), told to main):** `.SYSCARGS` needed `environment.rs`. `mint_local_entries`
   stores the words from the `Invocation`'s index with empty slots below it, or removes the
   owed `SYSCARGS` entry for `without_syscargs`. The removal needed
   `dispatch/hash.rs::directory_remove`, because `store_remove` reads the item it takes and the
   owed placeholder refuses that read.
3. **Done, mechanical:** the five unit tests calling `crate::execute_on` directly
   (`dispatch/tests.rs:351`, `scheduler/tests/mutants.rs:49`, `native.rs:55`, `pool.rs:275`,
   `sim/tests.rs:692`) pass byte literals.

## Concerns

1. **`TEST_PASSINGARGS7` stays red, and Ruled 2 would not turn it green** (sent to main). The test
   merges stdout and stderr into one array through `ADDRESS WITH OUTPUT/ERROR`. With the
   descriptors separate, the crate's output equals the oracle's byte for byte. Swapping parent
   and child (`step1/probes.txt`): oracle parent with crate child fails, and crate parent with
   oracle child passes. So the cause is the crate child writing all of stdout and then all of
   stderr at exit. Ruled 2 (stderr per line end, redirected stdout buffered) would put the six
   trace lines before the three `SAY` lines. I did not touch buffering.
2. **`--lib` has one failure I believe predates this task:**
   `tests::a_fragment_that_does_not_parse_raises_the_oracles_condition` (`src/tests.rs:409`)
   expects `found "&1"` for 27.901. The crate now prints `found "THEN"`, which is the oracle's
   text (Tasks 1-4's insert work). Task 4's report ran only `--lib error::`, which does not
   reach this test. I did not run it at `6391ead7c`, so this attribution is by content.
3. The empty-program run for "`-o` given and the program missing" is a second interpreter
   start on a failure path only. It is the smallest way I found to keep the oracle's order, the
   options before the search, when the search lives in the binary.
4. `resolve_program` returns the normalised path and the binary canonicalises it for the report,
   as `rexx-run` did, so a program reached through a symlink would be reported by its target.
   That is unmeasured on the oracle, as before.
