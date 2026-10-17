# Phase 9c Task 5 report: `rexx` beside `rexx-run`, and the A/B

Implementer: c9c-impl-5. Base `6391ead7c`. Commits: `629f2d79b` (`rexx`, tests, Step 1
records), `860fff6fc` (the first A/B and this report's first version), `9994f5c1c` (the
program-path chain as bytes, under ruling (A)), and the record commit after it.

## Status

Done. Main ruled (A) on both widenings and on Concern 1's default. The A/B was re-run at
`9994f5c1c`: no `OVER`. One expected-set test stays red, `TEST_PASSINGARGS7`, which Task 7 now
owns (Concern 1).

## Step 1: oracle measurements

Transcripts are in `docs/superpowers/records/2026-10-15-phase-9c/task-5/step1/`:

* `fixture.sh` writes the programs, `measure.sh BIN DIR` runs every shape, `run1.sh` runs one
  shape under the oracle wrapper and prints stdout, stderr and rc separately.
* `oracle.txt` is the oracle's run from a fresh directory. `crate.txt` is the crate's `rexx`
  over the same shapes at a build before `9994f5c1c`, so its three `'ff'x` names still show
  U+FFFD. `rexx_command.rs` is the current comparison.
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
* `resolve_program` is `require::search` with no caller (fix round 2): every input is bytes and
  every candidate is `stat`ed as bytes.
* `src/bin/rexx.rs` keeps `rexx-run.rs`'s `main` (signal handlers, `ProgramInput::Stdin`, the
  transient flags, the sinks, `REXX_SWITCH_MODE` with its `rexx-sim:` lines and status 2, the
  8-bit truncation). It reports the path the search found, not its canonical form (fix
  round 2). Arguments stay bytes (`OsStringExt::into_vec`)
  and the program runs through `run_program_named`. When `-o` is given and the program is not
  found or not readable, an empty program runs first so that the options' failure is the one
  reported, as on the oracle.
* `Invocation` gains `with_default_options`, `without_syscargs` and `with_syscargs(words,
  index)`. The last carries `.SYSCARGS`' words separately from the joined argument, because
  `rexx.cpp` builds the two from different ranges of `argv`.
* `run_program_named(path: &[u8], ...)`. `run_program(&str, ...)` delegates through
  `as_bytes()`, so its callers are untouched. `execute_on` takes `&[u8]`, and the five unit-test
  call sites that call it directly now pass byte literals.
* The program-path chain is bytes (`9994f5c1c`, ruling (A)): `Interp::program_path: Vec<u8>`,
  `required_paths: Box<[u8]>`, `Activity::requires_installing: Box<[u8]>`, `package_path -> &[u8]`,
  `ClauseSite::path: &[u8]`, and `Raised::cyclic_inheritance`, `circular_requires` and
  `namespace_not_found` taking bytes. `Raised::report` builds its ` running <name> line <n>` span
  as bytes, so the old `String::from_utf8_lossy(package)` there is gone. `resolve_search`,
  `resolve_requires`, `external_program` and `resolve_program_name` answer `Vec<u8>`, and the
  requires load and the external call read the file through `OsStr::from_bytes`. The search
  itself is `require::search` since fix round 2. The rest was what the compiler listed: `run/settings.rs`, `run.rs`,
  `dispatch.rs`, `condition.rs`, `dispatch/package.rs`, `dispatch/library/surface.rs`,
  `environment/identities.rs`, and test `ClauseSite` literals in `error.rs`, `builtin/word.rs`,
  `builtin/convert/tests.rs` and `builtin/string/tests.rs`. No conversion at a Rust-side display
  point was needed.

## Step 3: tests

* `tests/rexx_command.rs` with `tests/rexx_command/cases` (datadriven). Each case runs the
  oracle and `CARGO_BIN_EXE_rexx` with the same words from fresh fixture directories, and the
  three descriptors must agree. The recorded block is the oracle's, with the fixture as `DIR`
  and the banner's `Build date:` and copyright year masked. The cases are the Step 1 shapes
  plus the unreadable-file pair and `-o 'bogus 9' nonexist`. `.SYSCARGS` items and size come from
  `showargs.rex`. `.local~items` and the sorted `.local~allIndexes` are pinned under `-e` (9
  entries, no `SYSCARGS`), under a file run, and under `-x` with an argument (10 entries). The
  `'ff'x` names `p'ff'x.rex`, `p'ff'x` and `q'ff'x` are byte-for-byte cases like the rest.
* Mutation, by hand: `_ => argc_base += 0` in `parse` reddens the case set at every `-x`, `-`,
  `--` and `-?` shape. Restored by exact edit and checked with `cmp` against a copy.
* `interpreter_bin` (`tests/support/group_runner.rs`) and `check_path` and
  `the_crate_side_finds_its_own_rexx_and_no_rexxc` (`tests/suite.rs`) now name
  `CARGO_BIN_EXE_rexx`. `group_runner.rs`'s sim-mode `rexx-run` process is Task 6's and was left.

## Step 4: checks

Every command from `rust/`, statuses unpiped into
`docs/superpowers/records/2026-10-15-phase-9c/task-5/checks-statuses-9994f5c1c.txt` at
`9994f5c1c`. `checks-statuses.txt` holds the same set at `629f2d79b`, with the same results.

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

`corpus/refusal-sites.tsv` was re-derived with `REXX_REFUSAL_SITES_REFRESH=1` in both code
commits. `git diff 6391ead7c 9994f5c1c -- rust/corpus/refusal-sites.tsv` shows only `.rs:` line
numbers moving.

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

Status 101 at both commits. Expected 30, oracle 30 of 30 passed, crate 29 of 30. The one red is
`TEST_PASSINGARGS7` (Concern 1). The tables are in `task-5/suite/` (`629f2d79b`) and
`task-5/suite2/` (`9994f5c1c`, with `REXX_SUITE_TABLE` pointing there).
Scout C measured 22 failures with `rexx` linked to `rexx-run`, so this task turns 21 of them
green.

## Step 5: the A/B

From `rust/` at `9994f5c1c`, after
`CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 --bin rexx --bin rexx-run` and the two
copies:

```
bench-programs/callgrind.sh -B 0.5 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9c/ab-9994f5c1c \
  pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run \
  base=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9c-9994f5c1c \
  head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-9c-9994f5c1c
```

Exit 0, no `OVER`, no `SPREAD`. sha256s from `binaries.txt`: pin `c4b4e8c4...523c67` (the one
the 9b1 records cite), base `05b34aba...ff4c081`, head `8b1a8346...d438c35a`. The table,
`summary.tsv`, `binaries.txt` and `head-minus-base.tsv` are in
`docs/superpowers/records/2026-10-15-phase-9c/ab/`. The first run, at `629f2d79b` into
`cg-9c/ab`, is kept in `ab/629f2d79b/`, and also had no `OVER`.

Whole-process Ir against the pin. `base d%` is `rexx-run` at this commit and `head d%` is
`rexx`. `head - base` is the front end's share:

| program | base d% | head d% | head - base Ir | head - base % |
|---|---|---|---|---|
| alloc | -1.0868 | -1.0867 | 29339 | +0.0001 |
| alloc4c | -0.1103 | -0.1096 | 28940 | +0.0008 |
| arith | -0.1866 | -0.1863 | 29402 | +0.0002 |
| assign | +0.0002 | +0.0004 | 29959 | +0.0002 |
| churn | +0.1621 | +0.1414 | -130969 | -0.0207 |
| compound | +0.1036 | +0.1039 | 29289 | +0.0003 |
| decloop | +0.0609 | +0.0620 | 28889 | +0.0012 |
| decrender | -0.2401 | -0.2394 | 30476 | +0.0007 |
| dirread | -2.2431 | -2.2422 | 29747 | +0.0010 |
| dispatch | -0.2938 | -0.2937 | 29394 | +0.0001 |
| dispatchclass | -3.4683 | -3.4682 | 29939 | +0.0002 |
| emptyloop | +0.0006 | +0.0010 | 30138 | +0.0004 |
| extcall | -13.2014 | -13.2011 | 29764 | +0.0004 |
| fibcall | -3.4730 | -3.4727 | 29735 | +0.0003 |
| fibfunc | -3.4834 | -3.4831 | 29055 | +0.0003 |
| hashnew | -0.4809 | -0.4924 | -269451 | -0.0115 |
| heapshape | -0.6515 | -0.6506 | 22703 | +0.0009 |
| intcall | -10.1087 | -10.1052 | 29120 | +0.0039 |
| interpret | -2.1953 | -2.1969 | -67125 | -0.0016 |
| nop | +0.0004 | +0.0007 | 28683 | +0.0003 |
| parse | -0.1138 | -0.1120 | 30123 | +0.0018 |
| pkglocal | -6.5459 | -6.5452 | 29134 | +0.0007 |
| sayloop | +0.0132 | +0.0297 | 29672 | +0.0165 |
| sendloop | -0.1368 | -0.1366 | 29135 | +0.0002 |
| startup | -0.0073 | +0.0174 | 29512 | +0.0248 |
| strings | -0.2701 | -0.2699 | 29419 | +0.0002 |
| textnum | -0.0642 | -0.0617 | 29487 | +0.0024 |
| varlookup | +0.0001 | +0.0003 | 30039 | +0.0002 |
| rexxcps | -0.0918 | -0.0801 | 2269828 | +0.0118 |
| pingguard | -0.1737 | -0.1713 | 31288 | +0.0024 |
| pingmsg | -1.0292 | -1.0289 | 23598 | +0.0003 |
| pingsem | -0.2351 | -0.2328 | 29401 | +0.0023 |

`head - base` was derived from `table.txt` by
`awk -F'\t' 'NR>1 && $1!~/^#/ {d=$4-$3; printf "%s\t%d\t%+.4f\t%d\t%+.4f\n", $1, d, 100*d/$3, $9-$8, 100*($9-$8)/$8}' table.txt`,
saved with its libc-subtracted columns as `ab/head-minus-base.tsv`.

* The front end's cost is a fixed 28-30 thousand Ir per run (11-18 thousand with libc and ld
  subtracted): the search's `stat`s and the option scan. The largest relative share is
  `startup`'s +0.0248%.
* Four rows move more than that, all in libc: `churn` -130969, `hashnew` -269451, `interpret`
  -67125 and `rexxcps` +2269828. Their libc-subtracted differences are 17194, 17153, 17468 and
  18036, in line with every other row.
* The largest head-vs-pin deltas are `churn` +0.1414%, `compound` +0.1039%, `decloop` +0.0620%,
  `sayloop` +0.0297% and `startup` +0.0174%. `base` carries the same deltas, so they come from
  the tree since the pin and not from `rexx`.

## Shared paths

1. **Done under ruling (A):** the program-path chain, outside `lib.rs`, in the files Step 2
   lists. Against `.superpowers/sdd/queued/2026-10-13-lossy-path-sites.md` this closes none of
   the listed sites outright. The closest is "external file read: install.rs:1859/1866"
   (`new_file_executable`, now `install.rs:1921`): its read still goes through
   `String::from_utf8_lossy(name)` and `paths::normalize` and stays open, but the package name it
   records is now the bytes as given rather than the lossy copy. DIRECTORY, SysFileTree, `.File`,
   `.Stream`, CD, the `cwd_text` sites, and a `::REQUIRES` or external-call *name* that is not
   UTF-8 (`resolve_search` still does `from_utf8(name).ok()?`, and `self.cwd.to_str()?`) are
   untouched, as ruled.
2. **Done under ruling (A):** `.SYSCARGS` needed `environment.rs`. `mint_local_entries` stores
   the words from the `Invocation`'s index with empty slots below it, or removes the owed
   `SYSCARGS` entry for `without_syscargs`. The removal needed
   `dispatch/hash.rs::directory_remove`, because `store_remove` reads the item it takes and the
   owed placeholder refuses that read.
3. **Done, mechanical:** the five unit tests calling `crate::execute_on` directly
   (`dispatch/tests.rs:351`, `scheduler/tests/mutants.rs:49`, `native.rs:55`, `pool.rs:275`,
   `sim/tests.rs:692`) pass byte literals.

## Concerns

1. **`TEST_PASSINGARGS7` stays red. Main ruled that it is reported as such here and that Task 7
   owns it**: Task 7 Step 2 hands pending stdout over before each stderr hand-over and must
   make it pass. The test
   merges stdout and stderr into one array through `ADDRESS WITH OUTPUT/ERROR`. With the
   descriptors separate, the crate's output equals the oracle's byte for byte. Swapping parent
   and child (`step1/probes.txt`): oracle parent with crate child fails, and crate parent with
   oracle child passes. So the cause is the crate child writing all of stdout and then all of
   stderr at exit. Ruled 2 as first written (stderr per line end, redirected stdout buffered)
   would put the six trace lines before the three `SAY` lines. I did not touch buffering.
   `merge.rex` and the oracle-parent runs with either child in `step1/probes.txt` are the
   evidence Task 7 can reuse. The crate-parent runs gave the same split (crate child fails,
   oracle child passes) and were not saved.
2. **`--lib` has one failure I believe predates this task:**
   `tests::a_fragment_that_does_not_parse_raises_the_oracles_condition` (`src/tests.rs:409`),
   at both commits,
   expects `found "&1"` for 27.901. The crate now prints `found "THEN"`, which is the oracle's
   text (Tasks 1-4's insert work). Task 4's report ran only `--lib error::`, which does not
   reach this test. I did not run it at `6391ead7c`, so this attribution is by content.
3. The empty-program run for "`-o` given and the program missing" is a second interpreter
   start on a failure path only. It is the smallest way I found to keep the oracle's order, the
   options before the search, when the search lives in the binary.
4. Closed in fix round 2: the binary canonicalised the reported path, and the oracle reports
   the path that reached the program.

## Fix round 2

Review: `task-5-review.md`. Code commit `1acd33e22`.

* **Critical 1.** `resolve_search` widened the searching program's path but kept the name, `cwd`
  (`to_str`) and `REXX_PATH`/`PATH` (`shadow_var`) as UTF-8, and `stat`ed each candidate as UTF-8.
  `require::search` now takes every input as bytes, widens each one, narrows and normalises each
  candidate before its `stat`, and answers bytes. `launcher::resolve_program` and
  `Interp::resolve_search` both call it, so there is one representation. `narrow` maps a char
  above 255 to `?` rather than panicking. No such char can arise, because every candidate is
  widened bytes plus ASCII. `shadow_var` had no other caller and is deleted. A side effect is
  that a `::REQUIRES` or external-call name that is not UTF-8 is now searched too, where
  `from_utf8(name).ok()?` used to answer not found. This is the "external call, REXX_PATH" half
  of the review's fix, not one of the queue's listed sites.
* **New cases, against the oracle:** `d<é>/req.rex` and `d<é>/ext.rex` (`::REQUIRES` and `CALL` of
  a sibling), the same in `d'ff'x/`, `ext.rex` and `req.rex` from a cwd `d<€>`, and `call
  farcall` through `REXX_PATH` set to `rp<é>` and to `rp'ff'x`. All agree on the three
  descriptors. The 9994f5c1c binary gives 43.901 rc 213 on `d<é>/req.rex`, where this commit
  gives the oracle's rc 0 (checked by hand from a scratch copy of the shape).
* **Important 1.** `rexx.rs` reports `found` as the search returns it, and the comment is gone.
  New cases `link.rex -> sub/ps.rex`, `link` (found through `.rex`) and `lsub/ps.rex` with
  `lsub -> sub` report `DIR/link.rex` and `DIR/lsub/ps.rex`, as the oracle does.
* **Minors.** The `interpret` explanation in Step 5 is deleted, and the reviewer's measurement
  stands in for it: the move is codegen inside an unchanged `SymbolTable::intern`. `rexx.rs`'s
  interleaving sentence is deleted. `Run.syscargs` is now two named fields, `syscargs` and
  `syscargs_base`.

Checks at `1acd33e22`, statuses unpiped in
`docs/superpowers/records/2026-10-15-phase-9c/task-5/checks-statuses-1acd33e22.txt`:

| Command | Status | Result |
|---|---|---|
| `cargo fmt --all --check` | 0 | |
| `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | 0 | |
| `... --test rexx_command` | 0 | 1 passed |
| `... --test suite` | 0 | 52 passed |
| `... --test refusal_sites`, `owners`, `closed_phases`, `phase_ten_reasons`, `representation_refusals` | 0 each | 5, 6, 8, 5, 3 passed |
| `REXX_CORPUS_GATE=1 ... --test corpus` | 0 | 1065 of 1065 matching |
| `... --lib` | 0 | 1142 passed (`dc16e526e` fixed the 27.901 test; `launcher::widening_a_name_round_trips_every_byte` deleted with the private `widen`/`narrow`, covered by the oracle-compared cases; re-review count) |
| the rexx_command group (command as in Step 4, table in `task-5/suite3/`) | 101 | oracle 30/30, crate 29/30, `TEST_PASSINGARGS7` only |

`REXX_REFUSAL_SITES_REFRESH=1` left `corpus/refusal-sites.tsv` unchanged.

`resolve_search` is reachable from the call resolver (`run/call.rs:302`, `external_program`) in a
program that raises nothing, so it is not only an error path, and the A/B was re-run at
`1acd33e22` (below).

The A/B at `1acd33e22`, from `rust/`: the same `callgrind.sh -B 0.5` command as Step 5, with
`-o .../cg-9c/ab-1acd33e22`, `base=.../rexx-run-9c-1acd33e22` and `head=.../rexx-9c-1acd33e22`.
It exited 0 with no `OVER` and no `SPREAD`. The table now in `ab/` is this run, and the
`9994f5c1c` run moves to `ab/9994f5c1c/`.

* Largest head-vs-pin deltas: `churn` +0.1411%, `compound` +0.1039%, `decloop` +0.0620%.
* `head - base` per program is in `ab/head-minus-base.tsv`. It is 22-24 thousand Ir per run (17-18
  thousand with libc subtracted). Four rows differ on whole-process only, by moves in libc:
  `alloc` -20976962, `hashnew` +74795, `interpret` -328568 and `pingmsg` -15598. Their
  libc-subtracted values are 17167, 17120, 17426 and 18215.
* This `head` against `9994f5c1c`'s `head`, per program: whole-process between -0.0116%
  (`rexxcps`, -2.24M Ir) and +0.0001%. Libc-subtracted, every program moves by 42 Ir or fewer.
  The search change costs nothing measurable.
