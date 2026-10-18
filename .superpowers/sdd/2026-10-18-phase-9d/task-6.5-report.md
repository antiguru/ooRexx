# Task 6.5 report: binary and write-only streams

Clusters `stream-binary-records`, `stream-open-write-only` and `stream-command-position`. Status:
DONE. All 18 cluster tests leave the 9d set: 17 pass, and `LINEOUT TEST_BINARY` fails only at its
final `@@wrong` assertion and carries a `Deviation 46` licence row. The whole suite changed no row
outside the cluster.

## Commits

* `06f0ce3af` Phase 9d Task 6.5: binary records, write-only opens and stream positions. It holds
  the code (`rust/crates/rexx-exec/src/dispatch/stream.rs` only), the three witnesses, their
  `sourceline_oracle` files, `phase-9.txt`, `suite-9d-set.txt` without the cluster's lines, the
  `suite-licences.txt` row, and the witness and probe records under `t6.5/`.
* The records commit that follows it holds `t6.5/suite/` and this report.

## Root causes

### `stream-binary-records`

1. **A `BINARY` stream had no record model.** The crate parsed `BINARY RECLENGTH n` into
   `OpenMode` and used it only for line positioning. `LINEIN`, `ARRAYIN`, `LINES` and `LINEOUT`
   ran the line-stream paths. `StreamInfo::linein`, `::arrayin`, `::lines` and `::lineout`
   (`interpreter/streamLibrary/StreamNative.cpp`) each have a record branch, and the crate now
   follows them:
   * `LINEIN` reads `reclen - (charReadPosition - 1) % reclen` bytes. A short read is
     `NOTREADY:EOF` answering what it got, and an empty one is `ERROR:0`.
   * `ARRAYIN` appends records the same way, the first one shortened by a prior `CHARIN`, and ends
     on the empty read.
   * `LINES` is `ceil(size / reclen) - (charReadPosition - 1) / reclen`, for `N` too.
   * `LINEOUT` raises the bare 93 when the data does not fit what is left of the record, and
     otherwise pads it with blanks to the record's end (`writeFixedLine`).
   * Line positioning for `LINEIN(name, n)` goes through `seek_line`, which has the record branch.
     Before, it walked newlines whatever the mode.
   `CHAROUT TEST_POSITIONING_1_BIN` and `CHARS TEST_MANYCHARS_BIN` failed for the `LINEIN` cause.
2. **A `LINEOUT` with no data (Deviation 46).** On a record stream the oracle pads
   `reclen - (charWritePosition - 1) % reclen` blanks before positioning or closing. On a record
   boundary that is a whole blank record. That is the `@@BUG` in `TEST_BINARY`. `complete_record`
   writes nothing on a record boundary. It pads a partial record, kept as the oracle does,
   pending Moritz's ruling on Deviation 46's scope.

### `stream-open-write-only`

3. **`OPEN WRITE` had no write-only retry.** `streamOpen` retries a failed read-write open as
   `WR_CREAT` when `WRITE` was given (bug 3274050). The crate answered `ERROR:13`. The retry opens
   `O_WRONLY|O_CREAT` without `O_TRUNC`, as `WR_CREAT` does, and leaves the mode write-only. The
   91.999 that `clusters.tsv` records for the `~command` and `STREAM()` variants predates Task
   6.4. At `93063dff5` they answered `ERROR:13`, as the red witness shows.

### `stream-command-position`

Probed one by one, it is five causes, all in `dispatch/stream.rs`, so the cluster did not split
across tasks.

4. **An implicit read open left the write position at 1.** `implicitOpen` puts it at the end for
   any stream that is not read-only. This caused `QUERYFILE_EXISTS_OPENED_03`.
5. **`QUERY HANDLE` answered the null string for an open file.** `queryHandle` answers the
   descriptor of an open stream. The crate now answers `as_raw_fd()`. The test checks only for a
   whole number above 0, and the witness prints `datatype(h, 'W')` and `h > 2`. This caused
   `QUERYFILE_EXISTS_OPENED_01`.
6. **A line `SEEK` read the mode from before its implicit open.** `streamPosition` chooses the
   pointers by the old flags. It tests `read_write || read_only` for the line branch after
   `implicitOpen`. The crate tested the pre-open mode, which is all false on a fresh `Stream`, so
   it answered 0. This caused both `SEEK_CLOSEDFILE` tests. The `STREAM()` table forgets a closed
   stream, so its next `SEEK` builds a fresh one.
7. **A `SEEK` offset was not range-checked.** `position_offset` parses into `int64_t` and fails on
   overflow. The crate took a `u64` and clamped it. This caused `TEST_SEEK_PARSER`, at
   `9223372036854775808`.
8. **A forward line seek counted an unterminated last line.** `SysFile::seekForwardLines` counts
   only `\n`. `readForwardByLine` records the line it stopped on as `stream_line_size` when it ran
   out. The new `seek_forward_lines` does both. This caused `TEST_SEEK_USEDASCOUNT_2776442`
   (`+99999` answered 10, not 9).

A ninth change has no test of its own: a transient `BINARY` open with no `RECLENGTH` takes a
one-byte record (`checkStreamType`), where the crate raised 93.

## Witnesses, red then green

Red is the crate built from `93063dff5`, the unmodified HEAD, before any edit. Green is the crate
built from the tree committed as `06f0ce3af`. Both were run against the oracle, each side from a
fresh `mktemp -d` directory. The three descriptors are compared separately and stored under
`docs/superpowers/records/2026-10-18-phase-9d/t6.5/witnesses/{red,green}/<name>/`.

* `lang/stream_binary_records.rex` (causes 1 and 2). Red: stdout differs. `LINEOUT` did not raise
  93, the file held newline-terminated lines, `LINEIN` read `0123456789` as one line, and `ARRAYIN`
  answered 1 item. Green: all three descriptors match.
* `lang/stream_open_write_only.rex` (cause 3). It pairs the retry with its adjacent refusals:
  `open both`, `open read` and a bare `open` on the same write-only file stay `ERROR:13` on both
  sides. Red: `open write append` and `~command('open write')` answered `ERROR:13`. Green: match.
* `lang/stream_command_position.rex` (causes 4 to 8). Red: `handle 0 0`, `position write 1`,
  `seek closed 0`, `seek unopened 0`, `over 0` (no 93), `forward 4`, `back 3 1 3`, and `linein 9`
  leaving read line 9. Green: match.

I read each green stdout. Every line the witness's header comment claims is printed: the padded
records `[1    xyabcz    ]`, both `93.0` refusals, `NOTREADY:EOF` then `ERROR:0` with their
RESULTs, the per-record `CHARS` and `LINES`, both `ARRAYIN` splits, `whole [0123456789]`, the
write-only writes in the file read back after `chmod a+r`, the handle and position lines, `over
93.0`, and `forward 3 6 0`.

## The cluster's tests

All 18 rows of `clusters.tsv` for the three clusters. The changed rows are in
`t6.5/suite/changed-rows.txt`.

* Passing now: `CHAROUT TEST_POSITIONING_1_BIN`, `CHARS TEST_MANYCHARS_BIN`,
  `LINEOUT TEST_LINE_TOO_LONG`, `Stream TEST_ARRAYIN_BIN`, `Stream TEST_ARRAYIN_3_BIN`.
* Passing now: the `TEST_OPEN_WRITE_ONLY_3274050` tests, `C` and `D` in `STREAM`, and `C` to `F`
  in `Stream`.
* Passing now: `STREAM TEST_QUERYFILE_EXISTS_OPENED_01` and `_03`, both
  `STREAM TEST_SEEK_CLOSEDFILE` tests, `Stream TEST_SEEK_PARSER` and
  `Stream TEST_SEEK_USEDASCOUNT_2776442`.
* Licensed: `LINEOUT TEST_BINARY`, row `Deviation(46)`, crate `failed`, group verdict green.

## The licence row and its evidence

`rust/corpus/suite-licences.txt`:

```
base/bif/LINEOUT	TEST_BINARY	Deviation 46	the final assertFile expects the oracle's blank records 2 and 4 from a position-only and a closing LINEOUT
```

Every assertion the Deviation does not cover passes here. Evidence:

* The group run under this crate through `testOORexx.rex -f ooRexx/base/bif/LINEOUT.testGroup -U
  -V 2`, from a scratch copy of the harness layout. `TEST_QUEUE` was renamed out because it needs
  `.STDQUE` (Phase 10 Rxapi), and the `rxfuncquery` lines were dropped as `group_runner` drops them.
  It reports `TEST_BINARY` failing at line 233, the `@@wrong` `assertFile`, with
  `Actual: 1    22   three4444 55555`. That is the test's own commented `--@@lf` expectation, the
  documented result. The other failure, `TEST_LINE_TRANSIENT`, is an existing `Flaky` row. Saved as
  `t6.5/probes/lineout_group_crate/crate.{stdout,stderr}`.
* The probe `t6.5/probes/lineout_binary_dev46/` replays the test's sequence on both sides. The
  oracle and the crate agree on every line up to the five padded records `[1    22   333  4444
  55555]` and on each `0` answer. They differ only in the final file: the oracle has
  `[1         three     55555]` and the crate `[1    22   three4444 55555]`.

## Changed rows

Against `t6.4/r1/suite/tests.tsv`, computed by the command at the top of
`t6.5/suite/changed-rows.txt`. That gives 18 changed rows, exactly the cluster's tests: 17 move
from crate `failed` to `passed`, and `LINEOUT TEST_BINARY` moves from row `-` to
`Deviation(46)`. In `verdicts.tsv`, `CHAROUT`, `CHARS`, `LINEOUT`, `STREAM` and `class/Stream`
move from red to green, and no other verdict changed. `SysFileTree TEST_TREE_DIRECTORY` did not
flip in this run.

## Queued items and exclusions

None closed. No queued item is held by these three clusters. `queued-inventory.tsv` has no row
naming them. Deviation 46 was already in `phase-4-exclusions.txt`, and this task added only its
`suite-licences.txt` row.

## Checks

Commands from `rust/`, with `CARGO_TARGET_DIR=<scratchpad>/target-9d-t6` and
`CARGO_INCREMENTAL=0`, each exit status read unpiped:

* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`: exit 0. The first run
  failed on `suspicious_open_options` for the retry's `create` without `truncate`. The fix is
  `.truncate(false)`, matching `WR_CREAT`.
* `memcap 8G cargo test -j 4 --release -p rexx-parse --test sourceline_oracle`: exit 0, 1 passed.
* `memcap 8G cargo test -j 4 --release -p rexx-exec --test refusal_sites --test owners --test
  closed_phases --test phase_ten_reasons --test representation_refusals`: exit 0 (5, 6, 8, 5, 3
  passed). No `Loud` site changed, so `refusal-sites.tsv` was not re-derived.
* `memcap 8G cargo test -j 4 --release -p rexx-core --test unsafe_sites`: exit 0, 2 passed. No
  `unsafe` was added. `as_raw_fd` is safe.
* `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  29 passed, 1 ignored. The report header lists `phase-9.txt`.
* `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=docs/superpowers/records/2026-10-18-phase-9d/t6.5/suite/table.tsv
  CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite
  l3_core_suite`: exit 101 as expected, 402 groups with 28 red, in 4m36s.
* Perf check: not run. The dispatch rules it out unless the stdout path changed, and the
  standard-stream write path (`standard_write`) is untouched.

## Fix round 1

Review: `task-6.5-review.md`. Code commit `76acd13a3`. The records commit that follows it holds
`t6.5/r1/` and this section. Every change is in `dispatch/stream.rs`.

### Findings

1. **No-data LINEOUT pads a partial record.** Unchanged, as directed. Root cause 2 now says the
   padding is kept as the oracle does, pending Moritz's ruling on Deviation 46's scope. The false
   documentation claim is gone.
2. **The read side of a write-only stream.** The causes, from `StreamNative.cpp` and `SysFile.cpp`:
   * `StreamInfo::chars` and `::lines` answer 0 for a persistent stream that is neither
     `read_only` nor `read_write`. Both now check the mode.
   * `readBuffer` raises `notreadyError` with the read's errno. `charin` and the record
     `LINEIN`/`ARRAYIN` read now carry the errno (`ERROR:9`) where they answered `ERROR:0`.
   * `getLineWritePosition` and `getLineReadPosition` count lines through `countLines`, which
     fails with the errno. `line_write_position` and `line_read_position` now return
     `io::Result`. On an error `query_position` raises NOTREADY with it and leaves the tracker at
     0, as the oracle does. The state becomes `ERROR:9`.
   * `SysFile::hasData` answers false when the stream is not `readable`. So `atEof()` is true,
     `seekForwardLines` treats the failed read as the end of the file, and `LINEIN` is
     `NOTREADY:EOF`. The crate already agreed on both.
   * `lineout(f, 'x', 2)` therefore writes at line 1, and the oracle's write position is 3, after
     the bytes. The crate's measured rule, "end of the file plus what was written" after a
     line-placed write, now applies only to a stream that can read.
3. **Implicit read-only open, `QUERY POSITION WRITE LINE` 2 against 1.** This predates Task 6.5.
   The crate built from `93063dff5` (git archive, its own target dir) prints `abc 1 2 READY:`, the
   same as `06f0ce3af`. The oracle prints `abc 1 1`. `implicitOpen` zeroes `lineWritePosition` only
   for a stream that is persistent and not read-only, so a read-only or transient implicit open
   keeps `resetFields`'s 1. The fix is local: `ensure_open` starts such a stream at write line 1,
   character 1. Outputs are in `t6.5/r1/probes/readonly_write_line/`. The explicit open was left
   alone, because no probe covers its transient case.
4. **`complete_record` swallowed a failed write.** It now answers `Some` of the NOTREADY answer
   (`1`), and `lineout` and `close_for_write` return it without positioning or closing. This
   matches `completeLine`'s `notreadyError`, which unwinds. Not probed: a write has to fail on a
   persistent record stream.
5. **Device retry.** Not done, because no device could be probed. `isDevice()` there reads
   `SysFile::device`. A failed `open64` does not reset that flag (`SysFile::open` returns before
   `getStreamTypeInfo`, and `reset()` does not touch it). So the retry fires only when the same
   `Stream` last opened a character device and its read-write reopen now fails. That needs a
   character device that refuses read-write to this user, and none is at hand. `/dev/null` and
   `/dev/full` open read-write, and a FIFO is not `S_IFCHR`.

### Left divergence (Task 6.9's)

On the write-only stream, `stream(f, 'c', 'query position write line')` answers
`ERROR:9 Bad file descriptor` on the oracle, and `RESULT` is the same text. Here both are the null
string. The orx `query` handler is `raise propagate return (self~description)`, and
`exec_raise_propagate` (`run/condition.rs`) ignores the `RETURN` value. This is the
`raise-result-entry` cluster ("`RAISE ... EXIT expr` and `RETURN expr` leave the condition object's
`RESULT` unset"), which Task 6.9 holds. The witness prints only the description and whether the
condition was raised. The probe is in `t6.5/r1/probes/write_only_query_result/`.

### Witness, red then green

`lang/stream_write_only_reads.rex`, with its `sourceline_oracle` file and a `phase-9.txt` entry.
Red is the crate built from `06f0ce3af` (git archive, tree touched, `Compiling rexx-exec`
confirmed). Its stdout differs in `chars 12`, `lines 1`, `ERROR:0`, `NOTREADY:EOF 0` for the
write-line count, write positions 15 and 17, the file bytes, and `abc 1 2`. Green is the crate
built from `76acd13a3`, and all three descriptors match. The four witnesses' green runs are in
`t6.5/r1/witnesses/green/`. I read the green stdout, and every line its header claims is printed.

### Checks

Commands from `rust/` with `CARGO_INCREMENTAL=0` and the task's own target dir, each status read
unpiped:

* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`: exit 0.
* `memcap 8G cargo test -j 4 --release -p rexx-parse --test sourceline_oracle`: exit 0.
* The refusal tables (`refusal_sites`, `owners`, `closed_phases`, `phase_ten_reasons`,
  `representation_refusals`): exit 0. `rexx-core --test unsafe_sites`: exit 0.
* `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0, 29
  passed, 1 ignored.
* The whole suite, with `REXX_SUITE_TABLE` at `t6.5/r1/suite/table.tsv`: exit 101, 402 groups,
  28 red. Against `t6.5/suite/tests.tsv` no row changed (`t6.5/r1/suite/changed-rows.txt` holds
  only the command).
