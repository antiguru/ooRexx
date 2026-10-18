# Task 6.4 report: stream names and states

Clusters `stream-notready-names`, `stream-eof-state` and `stream-argument-error-numbers`. Status:
DONE_WITH_CONCERNS. All 16 cluster tests pass. The shared `RAISE PROPAGATE` fix also turned the 5
tests of `raise-propagate-from-method` and `SIGNAL TEST_SIGNAL_ANY_CONDITION_LOSTDIGITS` green
(see "Concerns").

## Commits

* `a735033de` Phase 9d Task 6.4: stream NOTREADY names, EOF states and argument errors. This
  commit holds the code, the four witnesses, their `sourceline_oracle` files, `phase-9.txt`, the
  re-derived `refusal-sites.tsv` rows and the `refusal_sites.rs` shared-answer group.
* The records commit that follows it holds `suite-9d-set.txt`, the queued item's closing line,
  `docs/superpowers/records/2026-10-18-phase-9d/t6.4/` and this report.

## Root causes

1. **NOTREADY carried neither `ADDITIONAL` nor `RESULT`.** `StreamInfo::notreadyError` raises
   with the stream object as `ADDITIONAL` and the method's default result as `RESULT`. The crate's
   `Interp::raise_notready` (`run/condition.rs`) now takes both and sets `pending_additional` and
   `pending_result` for the `CALL ON` and `SIGNAL ON` arms. Every stream call site goes through
   one `#[cold]` helper, `notready` (`dispatch/stream.rs`), which passes the value the call answers.
   This closes the `*`, `non-existing-file` and `'80'x` rows (`assertSame` on
   `condition("additional")`).
2. **An implicit open accepted a directory.** `ensure_open`'s read-only fallback opened `/`, so
   `CHARS` answered its size and `LINES` answered 1. `SysFile::open` closes a directory and fails
   with ENOENT, and `ensure_open` now does the same. This also makes `::OPTIONS ... NOTREADY SYNTAX`
   raise 98.974 for `chars('/')` and `.Stream~new('/')~charIn`.
3. **`RAISE PROPAGATE` of a non-SYNTAX condition answered nothing.** `exec_raise_propagate`
   returned `Flow::Exit(None)` and ignored the caller's traps. The orx `Stream~command` traps the
   native open's NOTREADY and runs `raise propagate`, so `STREAM(n, 'C', 'OPEN ...')` raised 91.999.
   `RexxActivation::raise` (`RexxActivation.cpp:1801`-`:1805`, `:1895`) offers the same object to
   the caller of the first non-internal activation and otherwise returns the object's `RESULT`. The
   new `propagate_unreportable` does this: a `CALL ON` trap queues the object, marked `PROPAGATED`
   with `INSTRUCTION CALL`, and a `SIGNAL ON` trap takes it through `reraised_object`.
4. **CHARIN's end of file.** A partial read left `ERROR:0`. `StreamInfo::charin` calls `eof(string)`
   after a short read that got data, which gives `NOTREADY:EOF`. A read that gets nothing fails in
   `SysFile::read` and gives `ERROR:0`. A start past the size is `eof()` before reading
   (`setCharReadPosition`). `readSetup` and `writeSetup` reset the state to `READY` after the
   implicit open, which the new `read_setup` and `write_setup` helpers do. `write_setup` also raises
   `ERROR:13` for `CHAROUT` to a stream opened read-only.
5. **LINEOUT and CHAROUT with no data only closed.** The oracle runs `writeSetup` first. That
   creates a missing file, and on `/` it raises NOTREADY and answers 1, as `LINEOUT TEST_INVALID_FILE`
   expects. `close_for_write` does this.
6. **Argument errors.** The BIFs declare positions as `optional_big_integer`, which raises 40.12
   (`ExpressionStack::optionalBigIntegerArg`). `whole_positions` (`builtin/stream.rs`) checks
   LINEIN and CHARIN arguments 2 and 3, and LINEOUT and CHAROUT argument 3. The methods marshal
   positions as `int64_t` (88.907) and counts as `size_t`. Both are reported against REXX with no
   line, as the oracle's marshalling is. A position below 1 is 93.907 naming argument 1, checked
   after the implicit open. Before this, the crate parsed positions as digits only and raised
   93.907 for `.nil`, `'1.0'` and `'1E1'`.

## Witnesses, red then green

Each witness was run against the oracle and against a crate binary built from `0e7e18cc4` (red;
`git archive` into the scratchpad, built in its own target dir), then against `a735033de` (green).
The parent of `a735033de` is `323a11d77`, and the two commits between it and `0e7e18cc4` change
only docs and queued files (`git diff --stat 0e7e18cc4 323a11d77`).
Outputs are in `docs/superpowers/records/2026-10-18-phase-9d/t6.4/witnesses/{red,green}/<name>/`,
with stdout, stderr and rc in separate files. I read each green stdout, and every path it names
prints.

* `rust/corpus/lang/stream_notready_names.rex`: CHARS, LINES, LINES C, LINEIN, CHARIN, LINEOUT,
  CHAROUT and the LINEOUT close on `/`, `*`, a missing file, `'bad' || '80'x || 'name'` (Review
  Focus 3) and a missing directory. Each line shows the answer, the hit count, `STREAM` S and D,
  and the condition's description, `ADDITIONAL` class and bytes, `RESULT`, `INSTRUCTION` and
  `PROPAGATED`. The witness also covers the object path, CHAROUT and the close on a read-only
  stream, and a file the close created. Red: rc 159 (97.1, `.nil` has no `RESULT`) and every row
  differs. Green: all three descriptors identical.
* `rust/corpus/lang/stream_command_open_failure.rex`: `STREAM 'C' OPEN` on a directory, for
  writing, on a missing file and on a missing path, plus `Stream~command`. It covers untrapped,
  `CALL ON`, `SIGNAL ON`, a trap in a calling routine, a method that propagates a command's ERROR,
  and `::OPTIONS NOTREADY SYNTAX`. Under that option the propagated NOTREADY is not an error, and
  the OPEN method's own raise is 98.974 in a `::routine`. Red: 91.999 on line 8, rc 165. Green:
  identical, rc 158.
* `rust/corpus/lang/stream_charin_eof.rex`: a partial read, an empty read, starts past the end,
  `charin(1, 0)`, the return to READY, the BIF form, LINEIN at EOF, and `SIGNAL ON`. Red: the
  `NOTREADY:EOF` rows read `ERROR:0`. Green: identical.
* `rust/corpus/lang/stream_position_arguments.rex`: LINEIN, CHARIN, LINEOUT and CHAROUT, as BIFs
  and as methods, over `line`, `1.5`, `' 2 '`, `1E1`, `-1`, `0`, `.nil`, a 20-digit number and
  `1.0`. It also checks the position before the open (`/`) and after it. Red: the BIF rows answer
  93.907 where the oracle answers 40.12, and the method rows differ. Green: identical. The bare 93
  rows print the code only (see "Concerns").

## The cluster's tests

All 16 pass in the whole-suite run:
* CHARS `TEST_CHARS_NOTREADY_{DIRECTORY,INVALID,NONEXISTING}`;
* LINES `TEST_LINES_NOTREADY_{DIRECTORY,INVALID,NONEXISTING}`;
* LINEOUT `TEST_INVALID_FILE` and `TEST_LINE_NONNUMERIC`;
* Stream `TEST_LINEIN_ARG_ONE_INVALID`, `TEST_NOTREADY_EOF_2792702` and `TEST_STATE_NOTREADY_2`;
* OPTIONS `TEST_ALL_SYNTAX_NOTREADY`, `TEST_ALL_SYNTAX_OVERRIDE`, `TEST_NOTREADY_SYNTAX_NOT` and
  `TEST_NOTREADY_SYNTAX_OVERRIDE`;
* SecurityManager `TEST_STREAM_PASS_BIF_STREAM_OPEN`.

## Changed rows

`diff` of `t6.3/suite-r1/tests.tsv` against `t6.4/suite/tests.tsv` (sorted, header dropped) shows
22 rows, each moving from oracle passed / crate failed to passed / passed:
* the 16 above;
* SIGNAL `TEST_SIGNAL_ANY_CONDITION_{ERROR,FAILURE,NOSTRING,NOTREADY,NOVALUE}`, which is the whole
  of `raise-propagate-from-method`, and `TEST_SIGNAL_ANY_CONDITION_LOSTDIGITS`, which Task 6.3
  attributed to it.

No row turned red. `SysFileTree TEST_TREE_DIRECTORY` did not change this run. `verdicts.tsv`
changes only for CHARS, LINEOUT, LINES, Stream, OPTIONS, SIGNAL and SecurityManager, and LINES
turns green. All 22 lines leave `rust/corpus/suite-9d-set.txt`.

## Queued items

* `2026-10-16-stream-command-open-missing-file.md`: closed by the same mechanism (root cause 3).
  The probe `t6.4/probes/scopen` answers `ERROR:2` and `rc ok` on both sides. The file's first line
  now reads `**CLOSED 2026-10-18 by Phase 9d Task 6.4 (a735033de).**`.
* `2026-09-28-condition-object-divergences-found-in-fix-round-a.md` (owner
  `call-on-trap-argument`): the NOTREADY object now has `RESULT` and `ADDITIONAL`. It still has
  `POSITION` where the oracle has none (`t6.4/probes/coorder`), so that item stays open.

## Checks

| command (from `rust/`, `CARGO_TARGET_DIR` the task's own, `CARGO_INCREMENTAL=0`, `-j 4`, `memcap 8G`) | exit |
| --- | --- |
| `cargo fmt --all --check` | 0 |
| `cargo clippy -p rexx-exec -p rexx-parse --all-targets -- -D warnings` | 0 |
| `cargo test --release -p rexx-exec --lib --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test stream_handle --test loud --test licensed_divergences` | 0 |
| `REXX_REFUSAL_SITES_REFRESH=1 cargo test --release -p rexx-exec --test refusal_sites` (re-derivation; its own run is red by design) | 101 |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` (29 passed, 1 ignored) | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t6.4/suite/table.tsv cargo test --release -p rexx-exec --test suite l3_core_suite` (4m36s) | 101 (expected) |
| `bench-programs/callgrind.sh -B 0.5 -p "startup interpret parse churn sayloop" -o .../cg-9d/t6.4 pin=.../rexx-run head=.../cg-9d/t6.4/rexx-head` | 0 |

`refusal-sites.tsv`: `native_argument_out_of_range` is now reachable from a send (`body+send`,
agrees, 88.907). `native_argument_out_of_range_unsigned` moves from diverges to agrees, because
both are now reported against REXX with no line (`t6.4/probes/major`, `major2`). No `unsafe` was
added.

Perf (`t6.4/callgrind.txt`, whole process against the pin; the delta is against
`t6.3/callgrind-r1.txt`):

| program | t6.3-r1 vs pin | t6.4 vs pin | task delta |
| --- | --- | --- | --- |
| startup | +0.0144 | +0.0145 | +0.0001 |
| interpret | -2.1791 | -2.1898 | -0.0107 |
| parse | -0.0879 | -0.0878 | +0.0001 |
| churn | +0.1265 | +0.1266 | +0.0001 |
| sayloop | +0.0310 | +0.0309 | -0.0001 |

## Concerns

1. **`raise-propagate-from-method` closed here as a side effect.** The fix (root cause 3) is in
   shared condition code, and it turns that cluster's 5 tests green. Review Focus 4 asks that
   cluster's instance to witness INTERPRET, REPLY and a propagated condition. These witnesses
   cover only the propagated path: a native NOTREADY through `Stream~command`, and a command
   ERROR through a method. The controller should decide whether that instance still runs for the
   INTERPRET and REPLY witnesses.
2. **Remaining condition-object differences, not this cluster's.** These are in `t6.4/probes/d`.
   A NOTREADY object here has `PACKAGE` and `PROGRAM` of the program where the oracle has `REXX`,
   and it has a `POSITION` the oracle omits. The `CALL ON` `STACKFRAMES` is 1 here and 2 on the
   oracle. A propagated one carries the orx line as `POSITION` (287).
3. **The bare 93 condition object.** The oracle's has `MESSAGE` `.nil`, and this crate's has
   `Incorrect call to method.` (`t6.4/probes/e`, `linein(name, 1, 2)`). This difference predates the
   task, and `stream_position_arguments.rex` prints only the code for those rows.
4. `LINEOUT`/`CHAROUT` with no data now create a missing file, as the oracle does. A test that
   closes a name it never wrote leaves an empty file behind on both sides.
