# Task 5 report: parse errors as SYNTAX, with the traceback line

Status: DONE_WITH_CONCERNS. Commits `59eb57f37`, `bef52e1b6`, `7824d57df`, and the commit carrying
this report and the gate record. `whole_groups` OOM-killed at the 8G cap with `-j 1`; stopped as
instructed, expectation lines not updated.

## Changes

- `rexx-parse/src/token.rs:59-71`: `ParseError` gains a private `u32` end; `clause()` is the echoed
  range `byte..end`; `echoing` resolves it once and moves `byte` to the clause's start (the line
  the oracle reports, `clauseLocation`'s start). The error stays two words (perf round 1).
- `rexx-parse/src/scanner.rs`: `scan` echoes a scanner error from the clause start to the scan
  position; 13.1 runs to the end of the offending UTF-8 sequence (`utf8_width`, `:973`); 99.943
  echoes its `::RESOURCE` clause.
- `rexx-parse/src/clause.rs:134` `ClauseCursor::echo`: a grammar error echoes the physical clause
  holding its byte; a label's runs on to the end of the clause the label starts.
- `rexx-parse/src/block.rs:356` `block_error`: echoes the last instruction's span, except a label
  and a THEN the source ends after (whole clause), each measured.
- `rexx-parse/src/lib.rs:88-140`: `Rejected`, `program_from`, `fragment_from` hand the source back
  with the error; `parse_program`/`parse_lines`/`parse_interpret` keep their signatures.
- `rexx-exec/src/error.rs:1864` `Raised::parse_failure`: the one function, error plus source plus
  line to a SYNTAX condition and its clause text. `push_trace_line` (`:2043`) keeps a clause's own
  trailing line end (i2 case 3).
- `rexx-exec/src/run.rs:3205` `Interp::raise_parse_failure`: records the clause as a level of its
  own (`Named` for a loaded source, `Clause` for INTERPRET at the INTERPRET's line), a `COMPILE`
  stack frame, and seals the level.
- Callers: main program `lib.rs:3185` `parse_failure_outcome` (standard report, rc from the code);
  external call `lib.rs:2128`; `::REQUIRES` and `Package~new(file)` `install.rs:655`; `Package~new`
  over lines `install.rs:954`; `Method/Routine~newFile` `install.rs:1836`; INTERPRET
  `run/interpret.rs:29`; `Method~new`/`Routine~new` (and every `compile_*_source` user)
  `dispatch/class_protocol.rs:336`.
- `run/interpret.rs:282`: a failing line typed at a debug pause clears the failure levels. Found
  here: it leaked traceback entries into the next condition before this task (a runtime error
  typed at a pause; probe `dbg2`, oracle traceback 1 item, base 3). Crate test
  `tests.rs:434 a_failing_pause_line_leaves_no_traceback_level`; reaches the site (asserts both
  `+++ Interactive trace.  Error` lines) and goes red (`3` against `1`) with the line removed.

## Refusals deleted

`Loud::required_source` and `Loud::method_from_source`. `Loud::library_source` stays (b17).
`refusal-sites.tsv` re-derived; the `Raised from` row moved from `body+send` to `body`
(`off-send-surface`), since `From<&ParseError>` is now constructed only in `error.rs`.

## Witnesses and oracle agreement

Corpus (`phase-6-1.txt`, Task 5 block), each with a SOURCELINE expectation; second files in `.d`
directories, named `.cls` because `tiling.rs`/`variants.rs` parse every corpus `.rex`:
`parse_error_interpret` (i2 1-8 under a trap: code, position, rc, traceback), `_interpret_reply`
(interpreply), `_interpret_untrapped` (interp, with 35.929 so the report carries no insert),
`_requires` (b3 requires), `_external_call` (trapped and untrapped), `_newfile` (Method and
Routine newFile trapped, Routine untrapped), `_package_new` (trapped and untrapped), `_method_new`.
Main program (c8): `tests/ir_recorded_cases/parse-errors-main`, four stanzas (scanner error,
grammar error in a block, END after a label, a continued clause), checked by
`ir_recorded_oracle` in gate mode. Every witness's stdout and stderr were read and each path it
claims prints. All agree with the oracle on all three descriptors; untrapped witnesses use
messages without inserts (R3). Mutation: dropping the recorded site in `raise_parse_failure`
reddens all eight corpus witnesses.

Extent check: `task-5-parse-sweep.py` runs every `translation` row of `parse-errors.tsv` through
`rexx` and `rexx-run` and compares rc, stdout and stderr minus the sub-message line (inserts):

```
python3 -I .superpowers/sdd/2026-10-07-phase-6-1/task-5-parse-sweep.py rust/corpus/errors/parse-errors.tsv WORKDIR OUT
```

At the r1 tree: `rows 567 same 565`. The two others are rows 434 and 753 (`then: nop`), the
pre-existing 18.x against 35.1 code divergence `errors.rs:466` records.

Not agreeing, not in the brief's comparison: `condition('O')~program` and `~package~name` for a
loaded source name the caller's program where the oracle names the source; recorded as a KNOWN
GAP in `phase-4-exclusions.txt`. `STACKFRAMES` agree (probe `pkgprog`).

## Docs

`phase-4-exclusions.txt`: the `::REQUIRES` OWNER row marked DELIVERED; the KNOWN GAP "a callee that
does not parse" replaced by the insert row (R3) and the program/package row above.

## whole_groups

Command: `REXX_CORPUS_GATE=1 REXX_WHOLE_GROUPS_TABLE=... memcap 8G peak.sh cargo test -j 1
--release -p rexx-exec --test concurrency_tests whole_groups`. Result: OOM-killed at the 8G cap,
memcap reports peak 8.0G, about nine minutes in; no table written. Not raised, not retried. So
the expectation lines (Task 4's TRACE_TraceObject `whole` lines, Task 4a's changes, and the
`does not parse here` rows of ATTRIBUTE, CONSTANT, METHOD, CALL, GUARD, TRACE) are unchanged and
unverified.

## Perf

Gate record `## Task 5`. Task 5 against `f1202acc1`: startup +0.2257%, parse +0.0085% (round 1;
the first version was +0.49% on startup). Running totals against `e6af1198b`: startup +0.2646%,
parse +0.6090%, of which +0.6004% predates Task 5. Wall clock inside ±4% but for startup's one-step
resolution, which the pad shows identically.

## Commands and results

- `cargo fmt --all` clean; `memcap 8G cargo clippy --workspace --all-targets -- -D warnings` exit 0
  (at `7824d57df`).
- `memcap 8G cargo test -j 4 --workspace --no-fail-fast` at `7824d57df`: exit 0. (At `59eb57f37`
  it was 101, `tiling` and `variants` parsing the `.d` fixtures; fixed by `bef52e1b6`.)
- `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus --test
  ir_recorded_oracle` at `7824d57df`: exit 0 (29 passed, 1 ignored; 21 passed).
- `REXX_REFUSAL_SITES_REFRESH=1 cargo test -p rexx-exec --test refusal_sites`: exit 0.

## Concerns

1. whole_groups OOMs at 8G with `-j 1` (peak 8.0G); its rows are not verified for Tasks 4, 4a, 5.
2. `parse`'s running total is +0.609% against the 6.1 base, over +0.5%, nearly all from before
   Task 5.
3. Loaded-source parse failures: PROGRAM and PACKAGE differ from the oracle (KNOWN GAP recorded).
4. The debug-pause leak fix touches Task 6's area (one line, with its test).
5. `ParseError::byte` now moves to the echoed clause's start when a grammar error is resolved;
   `parse-errors.tsv`'s line check still passes on every row.

## Fix round 1

Commits `eb3775477` (behaviour, witnesses, exclusions, M1-M4), `496977456` (`collect_stress`
zero-collection list).

- I1. A package whose directives fail to install closes its own level with a `ROUTINE` frame
  named for the package (`install.rs` `seal_package_level`, called from `install_executable`
  and from `run_loaded` for `CallType::Requires`), so the native loader's `Compiled method` line
  and the calling clause record. A `::REQUIRES` prologue that raises captures its frame and seals
  its level too (`lib.rs` `run_main`, `own_level`); its frame is `ROUTINE` named for its package
  (`dispatch/context.rs` `read_snapshot`), and its clauses trace from the margin as a called
  program's do (measured against the oracle under `trace r` from an indented `DO` and from an
  internal routine). The runtime shape, wrong at base, agrees too.
  Witnesses: `parse_error_nested_requires.rex` (Routine~newFile, Method~newFile, Package~new,
  loadPackage; each with a nested file that does not parse and one whose prologue raises; trapped;
  traceback and `STACKFRAMES`), `parse_error_nested_requires_{rnf,mnf,pkg,lp}.rex` (untrapped, one
  per loader), `package_new_prologue_raise.rex` (the exclusions family row's own probe, untrapped).
  All identical to the oracle on the three descriptors, read.
  Exclusions: the agreement row names the nested shapes it now covers; the family row "THE
  TRACEBACK OF A RAISE INSIDE .Package~new OR loadPackage" is marked CLOSED with its measurement
  (untrapped identical, trapped traceback and `STACKFRAMES` agree); a new KNOWN GAP records that a
  prologue's live `.context~stackframes` lacks the loading levels (the program's frame during its
  `::requires`, `METHOD NEW` under `Package~new`), measured. Also seen, not changed: a prologue run
  under `trace r` prints no `>I>`/`<I<` lines here where the oracle prints them.
- M1: `token.rs` states the bound: an echo ending past `u32::MAX` is cut there, one starting past
  it is empty.
- M2: `Raised::parse_failure` `debug_assert!`s the span is inside the source and echoes nothing
  otherwise; no invented text.
- M3: `Interp::parse_file` (`run.rs`) serves the external call, `::REQUIRES` and `newFile`.
- M4: the gate record's band sentence states both pad deltas.
- `collect_stress`: the programs whose `::REQUIRES` chain fails now build the failing package's
  frame and collect under the stress mode; dropped from the zero-collection list, outputs
  unchanged under it.

Perf (`lib.rs`/`install.rs` changed): the callgrind command of the gate record with `fr1`
(`eb3775477`, sha256 `58f637e8...a24`) added: startup +0.2641%, parse +0.6089% against
`base61`, the same as r1 to 0.001%.

Checks at `496977456`: `cargo fmt --all --check` 0; clippy (at `eb3775477`) 0;
`memcap 8G cargo test -j 4 --workspace --no-fail-fast` 0; `REXX_CORPUS_GATE=1` corpus and
`ir_recorded_oracle` 0 (29 passed, 1 ignored; 21 passed). At `eb3775477` the workspace run was
101 on `collect_stress` alone, fixed by `496977456`. `refusal_sites` 0 without a refresh.
whole_groups not run (I2 stays open).
