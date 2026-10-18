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

## Fix round 1

Review: `task-6.4-review.md`. Code commit `aa9bc799c`, then the records commit that follows it.
Records are in `docs/superpowers/records/2026-10-18-phase-9d/t6.4/r1/`.

### Fixes

* **I1.** In `propagate_unreportable` (`run/condition.rs`), an untrapped HALT is now `Raised::halt()`,
  searched from the top. An untrapped NOMETHOD is 97.1, built from the object's `ADDITIONAL` and
  the description, and 97.5 without them. This is the `if (!trapped)` block,
  `RexxActivation.cpp:1866`-`:1890`. It applies whether or not a caller exists. The probe
  `g2/nested_halt_none` now matches: Error 4.1 at line 18, rc 252.
* **I2.** When the first activation that is not an internal call has replied, the propagate looks
  for no caller trap, because the sender already has its answer. A `RESULT` then raises
  `Raised::return_after_reply()` (98.936), which is the `returnFrom` check. A condition with no
  `RESULT` (NOVALUE, USER) returns quietly. The probes `g2/reply_{error,failure,notready}_{sig,call,none}`
  now match.
* **M3 (partly fixed).** A transient stream is now written without a seek, so the first write to a
  FIFO answers 0 READY. A write position on a transient stream is 93.958 in both `LINEOUT` and
  `CHAROUT`.
  * **Not fixed, and not local: the second write.** In the oracle, every write after a write
    that is still buffered fails with `ERROR:29`. `SysFile::getPosition` answers the buffer
    offset (`SysFile.cpp:1038`-`:1041`). `StreamInfo` never advances `charWritePosition` for a
    transient stream. So `writeSetup` (`StreamNative.cpp:918`) finds the two unequal and calls
    `setPosition`, and the `lseek` fails on the pipe. The next write then succeeds. `p/st2.rex`
    still differs at `CHAROUT y` and at the CHARS after it, and a close after a buffered write
    raises NOTREADY the same way. I read this as an oracle defect rather than a rule to copy, and
    I did not emulate it.
  * **Not fixed, and not local: CHARS and LINES on a FIFO.** Emulating `SysFile::hasData` (read
    one byte, which is consumed) matched the review's probe, where the buffer had already been
    flushed. It did not match the case where the earlier write is still buffered: the oracle
    answered 0 there, then blocked on the next LINES and was HALTed by the timeout. The answer
    depends on that same write buffer, so I reverted the change. CHARS and LINES on a FIFO still
    answer from the size.
* **M4.** `query_exists` on an open stream answers without a lookup, as `StreamInfo::streamExists`
  does: a character or block device answers the name as written, and anything else its
  qualified name.
* **M5.** The `LINES` method's option is read by its first letter. Anything but N or C is
  `method_option_not_recognised("CN", option)` (93.915). That covers the empty string, which the
  oracle also rejects.
* **M6.** `read_from` reads through `take(len).read_to_end`, so the buffer grows only as bytes
  arrive. `charin(f, 1, 1E10)` no longer allocates 10 GB up front. `p5/v4` still dies at the cap.
  Its `linein` of a 10 GB sparse line goes through `read_line_from`, which this round did not
  touch (licensed OOM area).
* M7 is left to Task 6.9.

### Witnesses, red then green

The red binary was built from `8b0400019` (`git archive`, in its own target dir), and the green
one at `aa9bc799c`. Outputs are in `t6.4/r1/witnesses/{red,green}/`. I read each green stdout.

* `rust/corpus/lang/raise_propagate_unreportable.rex` (Review Focus 4). Each case is a `::routine`
  whose SIGNAL ON ERROR handler runs `interpret 'raise propagate'`, called first under the
  caller's CALL ON, then untrapped (the answer is the object's RESULT, 3), then under SIGNAL ON.
  The witness adds a replied method that propagates a command's ERROR (98.936 on stderr, while
  the main program carries on), and a replied method that propagates NOVALUE, which has no
  RESULT and stays quiet. It ends with a HALT propagated through two routines to no trap
  (4.1 at the outer `raise propagate`, rc 252).
  * Red: stdout prints `not reached` after the HALT, both errors are missing from stderr, and rc
    is 0 against 252.
  * Green: stdout, stderr and rc are identical.
* `rust/corpus/lang/stream_transient_queries.rex`. It covers the first write to a FIFO, LINEOUT and
  CHAROUT with a position on a FIFO, QUERY EXISTS on `/dev/null` and on a file opened WRITE, and
  the LINES method's options `n`, `Count`, `2.5` and `''`. Each FIFO sees exactly one write, and
  none is closed, because of the oracle defect above.
  * Red: `lineout 1 ERROR:29`; the positions raise NOTREADY instead of 93.958; `exists` is
    empty; `2.5` and `''` answer 1.
  * Green: stdout, stderr and rc are identical.
* The four witnesses from the first round are still identical on green.

### Review probes

`t6.4/r1/probes/review-probes.tsv` compares every review probe under `rev64/{g2,p,p3,p4,p5}` on the
oracle and on `aa9bc799c`. Every `g2` program matches. The differences that remain:
* `p/st2` (M3, above);
* `p3/b_nomethod` (M7);
* `p5/v4` and `p4/b_args` (the 10 GB `linein`, OOM-killed under `memcap`);
* `p5/v5`, `v6`, `v9` and `v13` (the bare-93 `MESSAGE`, Concern 3).

### Checks

| command (from `rust/`, task target dir, `CARGO_INCREMENTAL=0`, `-j 4`, `memcap 8G`) | exit |
| --- | --- |
| `cargo fmt --all --check` | 0 |
| `cargo clippy -p rexx-exec -p rexx-parse --all-targets -- -D warnings` | 0 |
| `cargo test --release -p rexx-exec --lib --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test stream_handle --test loud --test licensed_divergences` | 0 |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` (29 passed, 1 ignored) | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t6.4/r1/suite/table.tsv cargo test --release -p rexx-exec --test suite l3_core_suite` | 101 (expected) |
| `bench-programs/callgrind.sh -B 0.5 -p "startup interpret parse churn sayloop" -o .../cg-9d/t6.4-r1 pin=.../rexx-run head=.../cg-9d/t6.4-r1/rexx-head` | 0 |

The whole suite shows no changed row against `t6.4/suite/tests.tsv`: the sorted `diff` is empty.

Perf (`t6.4/r1/callgrind.txt`, whole process against the pin; the delta is against `t6.4/callgrind.txt`):

| program | t6.4 vs pin | r1 vs pin | delta |
| --- | --- | --- | --- |
| startup | +0.0145 | +0.0134 | -0.0011 |
| interpret | -2.1898 | -2.1783 | +0.0115 |
| parse | -0.0878 | -0.0879 | -0.0001 |
| churn | +0.1266 | +0.1262 | -0.0004 |
| sayloop | +0.0309 | +0.0298 | -0.0011 |
