# Task 6.15 report: the stream model in Rexx

Status: DONE_WITH_CONCERNS. `corpus/model/stream.rex` models this crate's stream semantics on a
string with separate read and write positions and no buffer. `tests/stream_model.rs` runs named
cases and generated sequences natively on real files and through the model. Writing it found
crate departures from the documentation that the NOBUFFER oracle does not share, and they are
fixed. The departures the crate and the oracle share go to Moritz (the REPORTED rules below). The
oracle's other departures are Deviations 46, 48 and 49, except one that no Deviation covers.
`every_licensing_deviation_names_this_test` stays red until the controller adds the "Witnessed
by" sentences below.

## Commits

* `8d766c638`: the model, the driver test with its case and departure files, the crate fixes
  in `dispatch/stream.rs`, and `lang/stream_append_writes_at_end.rex` with its
  `sourceline_oracle` file and `phase-9.txt` line.
* `b72267fd4`: a named case reads CHARS mid-stream (found by mutation, below).
* `99fd065db`: the test's answer dump writes one batch at a time (two tests appending at once
  interleaved lines).
* The records commit that follows holds `docs/superpowers/records/2026-10-18-phase-9d/t6.15/`
  and this report.

## Step 1: `corpus/model/stream.rex`

`stream_model(initial, transient, ops)` runs one scenario and answers, per operation, the result,
the state word and the write and read character positions, then the final content. It covers
OPEN (READ, WRITE, BOTH, APPEND, REPLACE, BINARY, RECLENGTH, NOBUFFER), CLOSE and the two closing
calls, CHARIN, CHAROUT, LINEIN, LINEOUT, CHARS, LINES (Normal and Count), SEEK by `=`, `<`, `+`,
`-`, CHAR and LINE, READ and WRITE, and QUERY POSITION. A transient stream is a queue with no
positions. A record is only addressing, and only a LINEOUT with a string pads.

Rules come from rexxref `iostrms.xml` and `funct.xml`, the programming guide's record passages,
and Deviations 46, 48 and 49, each cited at the rule by file and line. ANSI X3.274 is cited by
section only (9.7), and nothing is transcribed. Where the documentation is silent, the model takes
this crate's answer and names the function and its line in `dispatch/stream.rs`:
* a write past the end fills with NUL;
* a string longer than its record is the bare 93;
* the state word of a short read (NOTREADY) and of a read of nothing (ERROR);
* a SEEK lands unchecked, and a position below 1 transfers from character 1 with the crate's
  arithmetic (see Concern 4);
* the line a SEEK past the last line reaches;
* QUERY POSITION on a transient stream answers 1;
* a ctrl-Z at the end is overwritten by a LINEOUT.

The model and the oracle's buffering of LINEOUT until close: the model has no buffer, as the
brief says, and the queued question is not modelled (Concern 6).

## Crate departures fixed (oracle agrees with the model)

1. **APPEND wrote inside the file.** `charout(f,'XY',1)` on a stream opened `BOTH APPEND`
   overwrote characters 1-2. rexxref: "The write pointer cannot be moved anywhere within the
   extent of the file as it existed when the file was opened" (funct.xml:4309-4312). New
   `landing` makes every write on an APPEND stream land at the end, as the oracle's does.
2. **A transient stream could not be read.** `charin` and `linein` seeked first, which fails on a
   FIFO (`ERROR`). They now read a transient stream without seeking, and a line a byte at a time.
   CHARS and LINES answered 0 on a FIFO holding data. They now answer 1, since what waits cannot
   be counted without reading it (iostrms.xml:189-193).
3. **A LINES count that scans left the state alone.** The oracle's `StreamInfo::lines` calls
   `readSetup` before scanning, so the stream is READY after it. The documentation is silent, so
   the crate now matches the oracle.

Witnesses:
* `lang/stream_append_writes_at_end.rex` (fixes 1 and 3). Red: the base binary
  (`cg-9d/t6.14-r2/rexx-head`, whose stream code is that of `5698947b0`) differs on every line
  (`t6.15/witnesses/red/`). Green: stdout, stderr and rc are identical to the oracle's
  (`t6.15/witnesses/green/`, `t6.15/witnesses/oracle/`). I read the stdout: every scenario
  line prints.
* Fix 2: named cases `transient-queue` and `transient-counts`, plus probe `t6.15/probes/fifo.rex`.
  Red: the base answers `ci '' ERROR`, `li '' NOTREADY` and `chars 0 lines 0 0`. Green: it answers
  `ci ab`, `li cde` and `chars 1 lines 1 1`. The oracle reads `de`, not `ab`, which is the
  uncovered departure below.

## Step 2: `tests/stream_model.rs`

Each scenario runs in one program: natively on a real file (or a FIFO the test makes) and through
the model. The answers are compared item by item and by the final bytes.
* Named cases, `tests/stream_model/cases` (datadriven).
* Generated scenarios: `SEEDS` times `PER_SEED`, from the open forms in `OPENS` and the
  operation grammar in the module doc. A close is followed by a reopen with the same options, so every stream stays
  NOBUFFER. A READ stream's SEEK never names WRITE (Concern 5).
* ANSI 9.7 (`check_reflective`): from each native run's reported write positions, an overlay of
  every write must equal the final file at each written place (reflective), and every other place
  must hold its initial byte (nothing lost). It holds on every crate run. It is live: on the
  oracle it fails on Deviation 46 scenarios, and the oracle test asserts it fails on at least one.
* The crate tests: a departure is allowed only under a REPORTED rule, judged by the operation at
  the first differing item (`reported_rule`). The generated set is recorded in
  `departures-crate.tsv`.
* The oracle tests: a departure is allowed under a Deviation, judged by the operations up to the
  first differing item (`deviation_rule`: 46, 48, 49), or under a REPORTED rule that this crate
  shares. The generated set is recorded in `departures-oracle.tsv`, and the named oracle answers
  are in `ORACLE_DEPARTURES`. `REXX_STREAM_MODEL_REFRESH=1` rewrites both files.

Recorded sets (`cut -f2 departures-*.tsv | sort | uniq -c`):

| side | label | scenarios |
| --- | --- | --- |
| crate | reported write-only | 346 |
| crate | reported line-position | 34 |
| crate | reported line-query | 17 |
| oracle | 46 pads-on-close | 77 |
| oracle | 49 relative-seek | 164 |
| oracle | 49 seek-from-end | 75 |
| oracle | reported write-only | 279 |
| oracle | reported line-position | 4 |
| oracle | reported line-query | 13 |

Mutations (`t6.15/mutation/`):
* Model, CHARS counting from the read position itself: the generated crate and oracle tests go
  red (`model-chars.txt`). Every named case stayed green, so `separate-positions` now asks CHARS
  mid-stream (`b72267fd4`).
* Crate, `landing` ignoring APPEND: the named test goes red at `append-writes-at-end`, and the
  generated test goes red too (`crate-append.txt`).

## REPORTED: crate and oracle depart from the documentation alike (for Moritz)

The model follows the derived rule in each case. Neither the crate nor the oracle was changed,
since a change would be a new oracle departure, which needs a ruling.

* **R1 `write-only`**: `OPEN WRITE` reads. The documentation says it "opens the stream for writing
  only" (funct.xml:4302), and "If you do line positioning in a file open only for writing, you
  receive an error message" (funct.xml:4454-4455). Both interpreters open read-write, so CHARIN,
  LINEIN, CHARS, LINES and line SEEKs read the file. Case `write-only-reads`: `charin(f,3,1)`
  answers `n` on both, and the model answers the null string in the ERROR state.
* **R2 `line-position`**: line positioning counts from a tracked line number, not from the line
  the position is in. A character operation zeroes the tracker. A query sets it at a mid-line
  character, and naming that line again keeps the mid-line position.
  * Case `line-seek-relative-after-charin`: after `charin(f,6,1)` the read position is line 2's
    start, and `seek +1 read line` answers 1 on both. The model answers 3.
  * Case `linein-line-from-mid-line`: `linein(f,2)` after a query at character 9 answers `ne2` on
    both. The model answers `line2`.
  * The documentation: line positioning is on line-end characters (iostrms.xml:476-490), and
    LINEIN with `line` reads from "the start of a specified line" (funct.xml:3088-3090). Its
    allowance that a line number "could become inaccurate" covers only a rewrite ahead of the
    read position (iostrms.xml:488-489), and these cases rewrite nothing.
* **R3 `line-query`**: QUERY POSITION ... LINE answers from the tracker.
  * Case `read-line-after-positioning`: a stale 1 after `charin(f,7,0)`, where the model answers 2.
  * Case `read-line-at-end`: 3 at the end of a stream ending in a line end, where the model
    answers 4. QUERY POSITION WRITE LINE answers 4 for that same character on both interpreters
    (probe `t6.15/probes/documented-corners.rex`, `qwl-end`).
  * Case `write-line-query`: the write position inside the stream answers one line too many, 3
    for character 8, where the model answers 2.
* **R4 `lines-count`**: after a CHARIN, `lines(f,'C')` answers 3 and then 2 for the same position
  (case `lines-count-repeated`). The documentation says Count "returns the actual number of lines
  remaining in the stream" (funct.xml:3306-3307).

## Oracle departures

Covered, with the sentence each Deviation row needs (the controller adds them, and
`every_licensing_deviation_names_this_test` then passes):
* Deviation 46: `     Witnessed by tests/stream_model.rs (ORACLE_DEPARTURES and departures-oracle.tsv, rule pads-on-close).`
* Deviation 48: `     Witnessed by tests/stream_model.rs (ORACLE_DEPARTURES, rule transient-no-seek).`
* Deviation 49: `     Witnessed by tests/stream_model.rs (ORACLE_DEPARTURES and departures-oracle.tsv, rules relative-seek and seek-from-end).`

**Not covered (for the controller):** CHARS and LINES on a transient stream consume data on the
oracle. Case `transient-counts`: after `abcd` and `e` are written to a FIFO, CHARS, LINES and
LINES Count answer 1 each, and the following `charin(f,,2)` answers `de` where the model, and now
this crate, answers `ab`. The derived rule: CHARS and LINES "attempt to determine if data is
present" (iostrms.xml:189-193), and nothing in the documentation consumes input. The oracle's
`SysFile::hasData` (common/platform/unix/SysFile.cpp:1322-1333) reads one byte to test for data
and backs up with `::lseek(fileHandle, SEEK_CUR, -1)`. The arguments are swapped, and the call
fails on a FIFO anyway, so the byte is lost.

## Checks

From `rust/`, `CARGO_TARGET_DIR=<scratchpad>/t615-target`, `CARGO_INCREMENTAL=0`; outputs in
`t6.15/`.

| command | exit |
| --- | --- |
| `cargo fmt --all --check` | 0 |
| `timeout 590 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test refusal_sites --test refusal_dispositions --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test licensed_divergences --test numeric_model` (`checks-tables.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-parse --test sourceline_oracle` (`checks-sourceline.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --lib` (`checks-lib.txt`) | 0, 1157 passed |
| `REXX_CORPUS_GATE=1 ... --test corpus` (`checks-corpus.txt`) | 0, 1149 of 1149 matching |
| `... --test stream_model` (`checks-stream-model.txt`, at `99fd065db`) | 101: only `every_licensing_deviation_names_this_test` fails, on `[46, 48, 49]` |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t6.15/suite/table.tsv ... --test suite l3_core_suite` | 0, 402 groups green, 4m35s |

## Suite delta

`diff <(sort t6.14/r2/suite/tests.tsv) <(sort t6.15/suite/tests.tsv)` is empty: no row changed.
`verdicts.tsv` has 402 `equal` groups.

## Perf

`t6.15/callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -p "startup interpret parse churn
sayloop" -o .../cg-9d/t6.15-run pin=.../rexx-run parent=.../cg-9d/t6.14-r2/rexx-head
head=.../cg-9d/t6.15/rexx-head`, exit 0. The parent is the last perf-checked head (6.14 round 2).
With libc subtracted:

| program | parent vs pin | head vs pin | own delta |
| --- | --- | --- | --- |
| startup | -0.1203 | -0.1207 | -0.0004 |
| interpret | -4.6331 | -4.6331 | 0.0000 |
| parse | -0.1416 | -0.1416 | 0.0000 |
| churn | +0.1068 | +0.1068 | 0.0000 |
| sayloop | -0.0638 | -0.0640 | -0.0002 |

## Concerns

1. R1-R4 need Moritz's ruling: either a Deviation each, or a change to the crate.
2. Uncovered oracle departure: transient CHARS/LINES consume input (above).
3. **Deviation 49's text names only `<n WRITE`**, but the NOBUFFER oracle lands past the end for
   any character `SEEK <n`. Case `seek-char-from-end` is `seek <3 read`, which answers 22 on an
   18-byte file where the model answers 16. Of the generated `49 seek-from-end` rows, 21 name
   only READ and 32 name neither position (`t6.15/seek-from-end-split.txt`). Either the row's text
   extends to every `<n` character SEEK, or the controller rules otherwise.
4. **A SEEK below position 1.** rexxref: SEEK "returns the new position in the stream if the read
   or write position is successfully located or an appropriate error message otherwise"
   (funct.xml:4427-4429). The crate answers the unchecked position (`<99` on a 29-byte file is
   -69, as the buffered oracle answers, per `position`'s doc comment). The NOBUFFER oracle answers 0 with NOTREADY: the
   generated `S -2 write char` answers `0 NOTREADY` where the crate answers -1. The documentation
   gives the message no form, so the model takes the crate's answer. In the generated set these
   rows fall under `49 relative-seek`, because only a relative SEEK reaches below 1 there.
5. **A SEEK naming WRITE on a READ stream** is left out of generation. A READ stream has no write
   position (funct.xml:4299), and the crate's QUERY answers 0 for it. The crate still positions
   it, and its line form answers a line. The NOBUFFER oracle answers 0 with NOTREADY and a write
   position of -1, NOTREADY in `g1s31` and ERROR in `g13s16`. Measured in the first generated
   run.
6. **LINEOUT buffered until close** (the queued question,
   `.superpowers/sdd/queued/2026-10-20-lineout-buffered-until-close.md`). Measured with probe
   `t6.15/probes/lineout-visibility.rex` (output `lineout-visibility.out`). After `lineout` and
   `charout`, the oracle shows a second stream object and `wc -c` 0 bytes on an implicitly opened
   or `OPEN BOTH` stream. It shows 12 bytes with `NOBUFFER` or after `STREAM FLUSH`. QUERY SIZE
   answers 12 in every case. This crate shows 12 everywhere. These passages bear on it:
   * NOBUFFER "turns off buffering ... all data written to the stream is flushed immediately to
     the operating system", for when "data integrity is a concern" (funct.xml:4331-4335);
   * FLUSH "forces any data currently buffered for writing to be written" (funct.xml:4390-4391);
   * the positioning optimizations "require that no other process is writing to the stream
     concurrently", with "use the system I/O functions" when one is (iostrms.xml:495-500).

   So the documentation expects buffering and gives NOBUFFER and FLUSH as the ways to make writes
   visible, but it does not require them to stay invisible. On my reading, neither interpreter
   departs from the documentation here, and the difference is one the documentation permits.
7. Deviation 12 (a halt during a blocking read) is not reached by the model. No case reads a
   transient stream that waits.

## Fix round 1

Brief: `task-6.15-fix-r1.md`, with Moritz's rulings of 2026-10-20. Review: `task-6.15-review.md`.
Status: DONE_WITH_CONCERNS. Every ruling is in the crate and the model. No ooTest row changed, so
no ruled change is reverted. The driver no longer licenses any crate departure. The new Deviation
candidates are below, for the controller to write.

### Commits

* `4064a8011`: the crate (`dispatch/stream.rs`, `rexx-core` `OpenFile`), the model, the driver,
  the cases and `departures-oracle.tsv`. `departures-crate.tsv` is deleted.
* `231fb2fde`: the stream corpus witnesses drop the lines the rulings overturn, and their
  `sourceline_oracle` files are regenerated.
* `09c39a341`: citations and doc comments follow the code.
* The records commit that follows holds `t6.15/r1/` and this section.

### What changed in the crate, per ruling

* **Transient CHARS and LINES** (ruling 1, review I1): they answer 1 only when data waits or the
  state cannot be learned, and 0 otherwise. `transient_has_data` polls with no wait. When the
  descriptor is readable, it reads one byte and keeps it in `OpenFile::pending`, and the next read
  returns that byte first. A read at the end of input answers 0. Nothing is consumed.
  `do while lines('/dev/null') > 0` ends at once (`r1/witnesses/transient-counts.*`: round 0
  looped, head and oracle do not).
* **R1, OPEN WRITE is write-only.** The descriptor is still opened for reading where it can be, so
  LINEOUT can find a line number, but CHARIN, LINEIN and ARRAYIN refuse with `ERROR:9`. CHARS and
  LINES answer 0, and a line SEEK answers 0.
* **R2-R4, lines from the position.** The line tracker and the line count cache are gone from
  `OpenFile`. A line SEEK, `LINEIN n` and `LINEOUT s, n` walk line ends from the start of the file.
  The current line is `line_of` the position: one more than the line ends before it, or the record
  holding it. QUERY POSITION ... LINE answers `line_of`, and 0 for a READ stream's write position.
  LINES Count counts from the read position every time.
* **Invalid SEEK** (ruling 3): a SEEK that would land below 1 by characters or by lines, or that
  names WRITE on a READ stream, answers 0, raises NOTREADY and moves nothing (`invalid_seek`).
  WRITE seeks on READ streams are generated again.
* **APPEND and ctrl-Z** (review I4, M4): derived from rexxref intro.xml:3322-3330. Unix-like
  systems use no end-of-file character in text files, and Rexx honours `'1a'x` only in program
  source. So a ctrl-Z is data. The crate's `CTRL_Z` overwrite is removed for every stream, not
  only APPEND streams, since the passage covers every stream (see the candidate below). With
  APPEND, the derived rule and the oracle agree on where the bytes land.
* **A null CHAROUT moves nothing.** `charout(f, '')` on an APPEND stream moved the write position
  to the end. It now leaves it, as the oracle does and funct.xml:1396-1398 says ("no characters are
  written"). The generated record APPEND streams found this.

Witness `r1/probes/rulings.rex`: one line per ruling (`r1/witnesses/{red,green,oracle}/rulings.*`).
The base (`cg-9d/t6.14-r2/rexx-head`) prints the oracle's answers on the R1-R4 lines and the READ
stream's SEEK. On `seek <99` it answers -80, the unchecked position, where the NOBUFFER oracle
answers 118, and it opens the ctrl-Z file at 4 where the oracle opens at 3. Head prints the ruled
answer on every line, and each ruled line is a candidate below.

### ooTest

`REXX_SUITE_GATE=1 ... l3_core_suite` with the table at `t6.15/r1/suite/table.tsv`: exit 0, 402
groups `equal`. `diff <(sort t6.15/suite/tests.tsv) <(sort t6.15/r1/suite/tests.tsv)` is empty:
no test row changed, so no ruled change is reverted.

### Corpus witnesses

The gated corpus failed on five stream programs, each at lines the rulings overturn. Those lines
are dropped, and each departure is pinned in `tests/stream_model.rs` instead. After the edit each
program is identical to the oracle on all three descriptors (`bin/both`, `r1/checks-corpus.txt`
1149 of 1149):
* `lang/stream_chars_lines.rex`: the second LINES Count after a CHARIN (R4);
* `lang/stream_seek.rex`: `seek <99` (invalid SEEK);
* `lang/stream_seek_line.rex`: `seek 0 read line` (invalid SEEK) and a line SEEK on a WRITE
  stream (R1). Its comments no longer state the tracker's rules;
* `lang/stream_seek_errors.rex`: `seek -1` from position 1 (invalid SEEK, not a rejected option),
  and QUERY POSITION WRITE LINE on a READ stream (R3);
* `lang/stream_write_only_reads.rex`: LINEIN on a write-only descriptor (R1: ERROR here,
  NOTREADY:EOF on the oracle).

### The driver (review I2, I3, M2, M3, M5)

* The crate tests allow no departure: this crate must equal the model on every item of every
  named and generated scenario, and pass the 9.7 check.
* An oracle departure is judged by `oracle_rule` at the first differing item. The rule must
  describe the operation at that item and the field that differs:
  * a relative or `<n` character SEEK's answer (Deviation 49);
  * the model's invalid SEEK;
  * a read on a WRITE stream answering data;
  * a line operation's answer or position;
  * a line query's answer;
  * an LC item's count or state;
  * the open's write position on a ctrl-Z file;
  * a LINEOUT on a record APPEND stream;
  * the transient rules.
* Deviation 46 is judged by a variant: the model run with the oracle's padding (`CB`) put before
  each binary `LX` or `LP`. The oracle must equal the variant, or equal it up to an item another
  rule licenses.
* Rows where the oracle answered as this crate does can no longer get a Deviation label, because
  the crate equals the model.
* Mutation, the review's LINEOUT one byte late on a WRITE stream: `stream_model` goes red in
  `the_model_answers_generated_scenarios_as_this_crate_does` (`r1/mutation/lineout-late-on-write.txt`).
* Mutation, `write_record` ignoring APPEND (review C2): red in the named and the generated crate
  tests (`r1/mutation/record-append-ignored.txt`). M2: `OPENS` has `both binary reclength 4
  append`, and the named case `append-record`.
* Mutation, an empty transient stream answering 1: red at `transient-empty`
  (`r1/mutation/transient-empty-answers-1.txt`). Each mutation was reversed by a literal `sed`
  and `git diff --stat` was empty after it.
* M3: the crate run is on its own thread with a 120 s limit, and a blocked read fails the test.
* M5: operation `P` sends the POSITION command, generated and in the named case `position-command`.
* M6: `r1/seek-from-end-split.py` (command in its header) counts the `49 seek-from-end` rows by the
  position the SEEK at the first difference names: 18 READ, 17 WRITE, 22 neither.

Recorded oracle labels (`grep -v '^#' departures-oracle.tsv | cut -f2 | sort | uniq -c`,
`r1/departure-labels.txt`):

| label | rows |
| --- | --- |
| 46 pads-on-close | 63 |
| 46 pads-on-close + 49 relative-seek | 1 |
| 46 pads-on-close + 49 seek-from-end | 1 |
| 46 pads-on-close + new invalid-seek | 1 |
| 46 pads-on-close + new write-only | 1 |
| 49 relative-seek | 41 |
| 49 seek-from-end | 57 |
| new append-record | 6 |
| new ctrlz-is-data | 59 |
| new invalid-seek | 100 |
| new line-position | 21 |
| new line-query | 19 |
| new lines-count | 2 |
| new write-only | 299 |

`every_licensing_deviation_names_this_test` passes: it checks numbered Deviations only, and the
`new` labels name none yet.

### Deviation candidates (for the controller to write)

Measurements are from `r1/probes/rulings.rex` (crate head vs oracle, both NOBUFFER) and the
named cases. "Witnessed by" is the sentence for each row, once its rule label gets a number.

1. **CHARS AND LINES ON A TRANSIENT STREAM TAKE NO INPUT (Phase 9d).**
   * Rule: they answer 1 when data waits or the state cannot be learned, and 0 otherwise, and the
     next read returns what waited.
   * Measured (`r1/probes/transient-counts.rex`): after `abcd` and `e` are written to a FIFO, the
     oracle's `charin(f,,2)` answers `de` and its LINEIN the null string. This crate answers `ab`
     and `cde`. On an empty FIFO this process holds open, the oracle's CHARS blocks
     (`transient-empty`, skipped on the oracle). The oracle's `SysFile::hasData`
     (common/platform/unix/SysFile.cpp:1322-1333) reads a byte and fails to put it back.
   * rexxref iostrms.xml:189-193, funct.xml:1475-1487.
   * `     Witnessed by tests/stream_model.rs (ORACLE_DEPARTURES, rule transient-counts-read).`
2. **A STREAM OPENED WRITE READS NOTHING (Phase 9d).**
   * Rule: CHARIN, LINEIN and ARRAYIN refuse with ERROR, CHARS and LINES answer 0, a line SEEK
     answers 0, and QUERY POSITION defaults to the write position.
   * Measured: `charin(f,3,1)` after `OPEN WRITE` on a three-line file. The oracle answers `n`
     READY, CHARS 15 and LINES Count 3. This crate answers the null string in ERROR, and 0 and 0.
   * rexxref funct.xml:4302, 4454-4455.
   * `     Witnessed by tests/stream_model.rs (ORACLE_DEPARTURES and departures-oracle.tsv, rule write-only).`
3. **LINE POSITIONS COUNT LINE ENDS FROM THE POSITION (Phase 9d).**
   * Rule: a relative line SEEK counts from the line the position is in. LINEIN and LINEOUT with
     a line number, and a line SEEK, land at that line's start, whatever an earlier operation
     left.
   * Measured: after `charin(f,6,1)`, `seek +1 read line` is 1 and reads `line1` on the oracle,
     and 3 and `line3` here. `linein(f,2)` after a query at character 9 reads `ne2` on the oracle
     and `line2` here.
   * rexxref iostrms.xml:476-490, funct.xml:3088-3090, 3201-3203.
   * `     Witnessed by tests/stream_model.rs (ORACLE_DEPARTURES and departures-oracle.tsv, rule line-position).`
4. **QUERY POSITION ... LINE ANSWERS THE LINE THE POSITION IS IN (Phase 9d).**
   * Rule: one more than the line ends before the position, and 0 for a READ stream's write
     position.
   * Measured on a three-line file:
     * write position 8: 3 on the oracle, 2 here;
     * read position at the end: 3 on the oracle, 4 here;
     * read position 7 after `charin(f,7,0)`: 3 on the oracle, 2 here.
   * rexxref funct.xml:4550-4555.
   * `     Witnessed by tests/stream_model.rs (ORACLE_DEPARTURES and departures-oracle.tsv, rule line-query).`
5. **LINES COUNT ANSWERS THE LINES THAT REMAIN (Phase 9d).**
   * Rule: counted from the read position every time.
   * Measured: after `charin(f,2,1)`, two LINES Counts answer `3 2` on the oracle and `3 3` here.
     With no cache, a count always reads, so the stream is READY after it.
   * rexxref funct.xml:3306-3307.
   * `     Witnessed by tests/stream_model.rs (ORACLE_DEPARTURES and departures-oracle.tsv, rule lines-count).`
6. **A SEEK BELOW POSITION 1, OR OF A READ STREAM'S WRITE POSITION, FAILS (Phase 9d).**
   * Rule: it answers 0, raises NOTREADY, and moves nothing.
   * Measured:
     * `seek <99 read` on an 18-byte file: 118 READY on the NOBUFFER oracle, and the unchecked
       position on the buffered one. Here it is `0 NOTREADY`, the read position still 1.
     * `seek -9 read` from 4: `0 ERROR` with the read position at -1 on the oracle (case
       `seek-invalid`).
     * `seek =2 write` on a READ stream: 2 READY on the oracle, `0 NOTREADY` with write position 0
       here.
   * rexxref funct.xml:4427-4429 ("or an appropriate error message otherwise"), 4299.
   * `     Witnessed by tests/stream_model.rs (ORACLE_DEPARTURES and departures-oracle.tsv, rule invalid-seek).`
7. **A CTRL-Z IN A STREAM IS DATA (Phase 9d).** This one goes past the brief's APPEND question;
   see Concern 2.
   * Rule: no stream operation treats a trailing `'1a'x` specially.
   * Measured on `'ab'||'1a'x`:
     * the oracle opens with write position 3 and a LINEOUT overwrites the ctrl-Z
       (`6162780A`), BOTH and APPEND alike (cases `ctrlz-is-data`, `ctrlz-append-kept`);
     * this crate opens at 4 and keeps it (`61621A780A`);
     * with APPEND the oracle's bytes also land after the ctrl-Z (O_APPEND), but it still
       reports position 3.
   * rexxref intro.xml:3322-3330.
   * `     Witnessed by tests/stream_model.rs (ORACLE_DEPARTURES and departures-oracle.tsv, rule ctrlz-is-data).`
8. **ON A RECORD STREAM OPENED APPEND, LINEOUT COMPLETES THE RECORD AT THE END (Phase 9d).**
   * Rule: the bytes land at the end (funct.xml:4309-4312), so the record a LINEOUT completes is
     the one there (funct.xml:4352-4354).
   * Measured (generated `g4s3`, `r1/dump/oracle.tsv`): after `charout(f,,15)` on a 21-byte
     `RECLENGTH 4` stream, the oracle's `lineout(f,'abc')` is the bare 93, because it sizes the
     room from position 15. This crate writes `abc_` at 21.
   * `     Witnessed by tests/stream_model.rs (departures-oracle.tsv, rule append-record).`

### Reported for Moritz (no change)

* **M1, a read of nothing is ERROR.** `charin(f)` at the end answers `ERROR:0` on both
  interpreters, while LINEIN at the same place answers `NOTREADY:EOF`. The state method's
  NOTREADY example is a read beyond the end (streamclasses.xml:1638-1652). The model takes the
  interpreters' answer and says so at `charin`.
* **M4, APPEND's reading.** The model states it at `landing`: SEEK and QUERY report the write
  position as set, and every write, including one from beyond the end, lands at the end. Both
  interpreters agree.

### Checks

From `rust/`, `CARGO_TARGET_DIR=<scratchpad>/t615-target`, `CARGO_INCREMENTAL=0`; outputs in
`t6.15/r1/`.

| command | exit |
| --- | --- |
| `cargo fmt --all --check` | 0 |
| `memcap 8G cargo clippy -j 4 -p rexx-exec -p rexx-core --all-targets -- -D warnings` | 0 |
| refusal tables + `licensed_divergences` + `numeric_model` (`checks-tables.txt`) | 0 |
| `cargo test --release -p rexx-parse --test sourceline_oracle` (`checks-sourceline.txt`) | 0 |
| `cargo test --release -p rexx-exec --lib` (`checks-lib.txt`) | 0, 1157 passed |
| `cargo test --release -p rexx-core` (`checks-core.txt`) | 0 |
| `REXX_CORPUS_GATE=1 ... --test corpus` (`checks-corpus.txt`) | 0, 1149 of 1149 |
| `--test stream_model --no-fail-fast` (`checks-stream-model.txt`) | 0, 25 passed |
| `--test ir_recorded --test collect_stress` (`checks-ir-collect.txt`) | 101: `collect_stress` fails at `lang/raise_additional_array.rex` (Concern 1) |
| `REXX_SUITE_GATE=1 ... l3_core_suite` (`suite/`) | 0, 402 groups green |

### Perf

`t6.15/r1/callgrind.txt`: parent `cg-9d/t6.15/rexx-head` (round 0), head `cg-9d/t6.15-r1/rexx-head`
(`4064a8011`'s code), exit 0. With libc subtracted, the own delta is:

| program | parent vs pin | head vs pin | own delta |
| --- | --- | --- | --- |
| startup | -0.1207 | -0.1202 | +0.0005 |
| interpret | -4.6331 | -4.6331 | 0.0000 |
| parse | -0.1416 | -0.1416 | 0.0000 |
| churn | +0.1068 | +0.1068 | 0.0000 |
| sayloop | -0.0640 | -0.0637 | +0.0003 |

### Concerns (round 1)

1. **`collect_stress` fails before this task.** `the_l0_subset_passes_again_under_collect_on_every_allocation`
   panics with `a live value` (`dispatch.rs:1799`, `not_in_arena`) on `lang/raise_additional_array.rex`,
   a program with no streams. It fails identically at `5698947b0` and at `86175cf84`, each built
   from a `git archive` (`r1/collect-stress-at-*.txt`). It is a rooting defect for the controller.
   Those builds shared the target dir, which left stale build outputs. The dir was deleted and the
   head rebuilt before the perf run and the green witnesses, and the witness output was compared
   with that rebuilt binary.
2. **The ctrl-Z candidate goes past the brief's APPEND question.** The passage the derivation rests
   on covers every stream, so the crate keeps a trailing ctrl-Z on non-APPEND streams too. If
   Moritz rules ctrl-Z for APPEND only, the overwrite comes back for the other streams, and the
   `ctrlz-is-data` rows split by open mode.
3. The new rule labels need numbers. Until then, `departures-oracle.tsv` carries them as `new`.

## Fix round 2

Brief: `task-6.15-fix-r2.md`. Re-review: `task-6.15-rereview-r1.md`. Status: DONE_WITH_CONCERNS.
I1-I3, M1, M2 and the CHARIN ruling are in. No ooTest row changed, so the CHARIN ruling stays.

### Commits

* `63c7cad28`: the crate (`dispatch/stream.rs`, `input.rs`, `rexx-core` `OpenFile`), the model,
  the driver, `input_oracle.rs`'s console rows, and seven stream witnesses with their
  `sourceline_oracle` files.
* `9584ac2ba`: the named case `line-mark-after-write` and its `ORACLE_DEPARTURES` row.
* The records commit that follows holds `t6.15/r2/` and this section.

### I1: line operations are linear

* `OpenFile` keeps `line_mark`, a (character position, line) pair from the last line operation,
  and `line_total`, the stream's line count.
* `line_of` counts forward from the mark when the mark is at or before the position. `seek_line`
  walks forward from it when it lies on an earlier line. LINES Count is the line count less the
  lines before the read position's line.
* `write_open` drops the count on every write, and the mark on a write landing at or before it.
  So each cached answer is what a recount answers. The cache rests on rexxref's condition for
  its positioning optimizations, that nothing else writes the file meanwhile
  (iostrms.xml:495-500).
* Each accessor has a `debug_assert` that the cached answer equals a recount from the start.
  `stream_model` and the gated corpus pass under a debug build
  (`r2/checks-stream-model-debug.txt`, `r2/checks-corpus-debug.txt`, 1149 of 1149).
* Inverting `line_of`'s assertion turns the debug `stream_model` red with `a line counted from
  the mark` (`r2/mutation/line-mark-assert-inverted.txt`).
* A write that never drops the mark turns the release named test red at `line-mark-after-write`
  (`r2/mutation/mark-never-dropped.txt`). The generated scenarios alone did not catch it, so the
  case was added.

Callgrind, Ir for the whole process, from `r2/lineperf/run.sh` with the programs in
`r2/lineperf/` (`table.tsv`):
* `p1`: `do while s~lines > 0; s~linein`;
* `p2`: `linein(f, i)`;
* `p4`: LINEIN, then `query position read line`;
* `p6`: `seek i read line`, then LINEIN.

The parent is the round-0 binary, whose crates equal `2b5551df3`'s.

| program | lines | parent | round 1 | head |
| --- | --- | --- | --- | --- |
| p1 | 2500 | 145,961,536 | 592,416,158 | 150,139,890 |
| p1 | 5000 | 172,778,201 | 1,950,685,786 | 181,136,271 |
| p2 | 2500 | 146,223,970 | 624,351,714 | 158,540,244 |
| p2 | 5000 | 173,422,096 | 2,074,114,705 | 198,368,635 |
| p4 | 2500 | 239,439,473 | 689,320,188 | 243,571,032 |
| p4 | 5000 | 358,572,611 | 2,150,443,276 | 366,846,420 |
| p6 | 2500 | 203,145,389 | 681,482,298 | 215,431,268 |
| p6 | 5000 | 286,289,237 | 2,187,669,798 | 311,032,017 |

Less the roughly 120M of startup, head doubles from 2500 to 5000 lines on every program, so it
grows linearly, as the parent does. Round 1 grew quadratically. Head's `p2` costs about 5,000 Ir
a line more than the parent's.

### I2: nothing is consumed

* `transient_has_data` asks `FIONREAD` first. That needs no read, and it covers pipes, FIFOs and
  terminals.
* Only a device that refuses `FIONREAD` falls back to a `poll`, then a one-byte read kept for the
  next read. `/dev/null` is one such device, and its read finds the end and keeps nothing. Such a
  device has no other way to tell whether data waits.
* Probe `r2/probes/fifo-kept-byte.rex` (`r2/witnesses/fifo-kept-byte.*`):
  * after CHARS and LINES on a FIFO holding `abcd\nef\n`, a second stream object reads `6162`
    here and `6263` on round 1;
  * after a close and reopen, head reads `6364` and round 1 `640A`;
  * the oracle's second object reads `640A`, and its next CHARS blocks.

### I3: the default input stream

* `Input::stdin_waiting`: buffered data answers 1, and the end of input 0. Otherwise a `poll` with
  no wait answers 0 when nothing is readable. When something is, a read that cannot block takes
  what is there into the buffer.
* LINES and CHARS on a live standard input answer from it. While a read is in flight they answer
  1, since nothing can be learned then.
* Witness: `input_oracle.rs` rows `console-lines-loop` and `console-chars-loop`, piped input,
  oracle-identical (`r2/checks-input-oracle.txt`, gated, exit 0). Probe
  `r2/probes/console-lines.rex` with `printf 'hello\nworld\nthird\n' |` reads one line on round 1
  and three on head and on the oracle (`r2/witnesses/console-lines.*`).
* With `/dev/null` as standard input, the oracle's first LINES and CHARS answer 1. Its
  `SysFile::hasData` reads an unset count after `FIONREAD` fails. This crate answers 0, as
  ruling 1 says for `/dev/null`. `lang/std_stream_writes.rex` drops those two lines.

### M1: every rule compares its field

* The line-position arms need the result, or the position the line operation set, to differ.
* The relative and `<n` SEEK arms need the SEEK's answer to differ.
* The append-record arm needs the LINEOUT's result or write position to differ.
* A read on a WRITE stream that moves the read position is write-only.
* After the first LINEOUT that would pad, the Deviation 46 variant is asked first.
* `g3s35` is now `46 pads-on-close`, and `g4s32` is `new write-only`.

### M2: line positioning on a WRITE stream

On a text stream opened WRITE, a line SEEK answers 0 with NOTREADY and moves nothing, in the
invalid SEEK's form. `lineout(f, s, n)` and `lineout(f, , n)` raise NOTREADY, answer 1 (LINEOUT's
failure answer, funct.xml:3222-3226), and write and move nothing.

On a record stream opened WRITE, lines are character positions (iostrms.xml:465-472), and LINEOUT
may name a line past the end of a binary stream (funct.xml:3203-3205), so record positioning
stays allowed there. ooTest's `LINEOUT TEST_BINARY` opens `write ... binary reclength 5` and
asserts that each line-numbered LINEOUT answers 0. Refusing it would break the assertions that
Deviation 46's licence relies on passing (global constraint 7). The full refusal did not change
TEST_BINARY's row (`r2/suite-streams-full-m2/`), only because that test already fails at its
licensed final assertion.

### The CHARIN ruling (charin-eof-notready)

A CHARIN that reads nothing at the end is NOTREADY:EOF. This applies to a file, a transient
stream and the default input stream. A LINEIN on a record stream that reads nothing is
NOTREADY:EOF too, as text LINEIN already was. A read that fails with an errno stays ERROR. The
whole suite has no changed row (`r2/suite/`, against `t6.15/r1/suite/tests.tsv`), so nothing is
reverted.

### Witnesses edited

Seven programs drop the lines the rulings overturn, and each is oracle-identical again (gated
corpus 1149 of 1149):
* `stream_chars_lines`, `stream_charin_eof` and `stream_notready_names`: CHARIN past the end;
* `stream_binary_records`: the record LINEIN past the end;
* `std_stream_writes`: the `/dev/null` counts and CHARIN past the end;
* `stream_write_only_reads`: line positioning on a write-only text stream;
* `stream_append_writes_at_end`: its WRITE scenario writes without a line number.

### Recorded oracle labels

From `cut -f2 departures-oracle.tsv | sort | uniq -c` (`r2/departure-labels.txt`):

| label | rows |
| --- | --- |
| 46 pads-on-close | 53 |
| 46 pads-on-close + 49 relative-seek | 4 |
| 46 pads-on-close + 49 seek-from-end | 4 |
| 46 pads-on-close + new append-record | 1 |
| 46 pads-on-close + new charin-eof-notready | 7 |
| 46 pads-on-close + new invalid-seek | 2 |
| 46 pads-on-close + new line-position | 1 |
| 46 pads-on-close + new write-only | 6 |
| 49 relative-seek | 35 |
| 49 seek-from-end | 51 |
| new append-record | 3 |
| new charin-eof-notready | 100 |
| new ctrlz-is-data | 59 |
| new invalid-seek | 91 |
| new line-position | 19 |
| new line-query | 19 |
| new write-only | 300 |

### Deviation candidates, updated

* **CHARS AND LINES ON A TRANSIENT STREAM TAKE NO INPUT** (round 1, candidate 1) now covers the
  default input stream.
  * Rule: 1 while data waits and 0 at the end, so `do while lines() > 0` reads every waiting line.
  * Measured: with `/dev/null` as standard input, the oracle answers 1 and this crate 0.
  * Measured (I2): after CHARS and LINES, a second stream object on the FIFO reads `ab` here. The
    oracle's reads have lost what its counts took.
  * Witnessed by: unchanged.
* **A STREAM OPENED WRITE READS NOTHING** (candidate 2) adds a rule: line positioning on a text
  stream opened WRITE is NOTREADY. A line SEEK answers 0, and a LINEOUT with a line number
  answers 1 and writes nothing.
  * Measured: `lineout(f,'X',2)` after `open write` answers `0 READY` and writes line 2 on the
    oracle, and `1 NOTREADY` here, writing nothing.
  * rexxref funct.xml:4454-4455.
* **A CTRL-Z IN A STREAM IS DATA** (candidate 7, ruled a Deviation by Moritz). Measured with
  `r2/probes/ctrlz.rex` on `'ab'||'0a'x||'1a'x` (`r2/witnesses/ctrlz.*`):
  * the oracle reports write position 4 on BOTH and APPEND opens, before the ctrl-Z, and this
    crate reports 5;
  * the oracle's CHAROUT `Q` at the end overwrites the ctrl-Z (`61620A51`), on BOTH and on an
    implicit open, and its LINEOUT `x` does too (`61620A780A`);
  * this crate keeps it in all three (`61620A1A51`, `61620A1A780A`);
  * with APPEND the oracle's bytes land after the ctrl-Z (`61620A1A780A`, as here) while it
    reports position 4;
  * rexxref intro.xml:3322-3330;
  * `     Witnessed by tests/stream_model.rs (ORACLE_DEPARTURES and departures-oracle.tsv, rule ctrlz-is-data).`
* **On a record stream opened APPEND, LINEOUT completes the record at the end** (candidate 8,
  ruled a Deviation by Moritz). Unchanged.
* **New: A READ PAST THE END IS NOTREADY (Phase 9d).**
  * Rule: a CHARIN that reads nothing at the end, on a file, a transient stream or the default
    input, leaves the stream NOTREADY with description `NOTREADY:EOF`, as LINEIN does. So does a
    record-stream LINEIN that reads nothing. A read that fails with an errno is still ERROR.
  * Measured:
    * named case `charin-bounds`: a bare `charin(f)` at the end of an 18-byte file answers the
      null string, `ERROR` on the oracle and `NOTREADY` here;
    * `lang/stream_charin_eof.rex` before its edit: `s~charin(, 2)` past the end left `ERROR:0`
      on the oracle and `NOTREADY:EOF` here;
    * `lang/stream_binary_records.rex` before its edit: a record LINEIN past the end left
      `ERROR:0` and `NOTREADY:EOF`.
  * rexxref streamclasses.xml:1638-1652 (the NOTREADY example is a read beyond the end).
  * `     Witnessed by tests/stream_model.rs (ORACLE_DEPARTURES and departures-oracle.tsv, rule charin-eof-notready).`

### Checks

From `rust/`, `CARGO_TARGET_DIR=<scratchpad>/t615-target`, `CARGO_INCREMENTAL=0`; outputs in
`t6.15/r2/`.

| command | exit |
| --- | --- |
| `cargo fmt --all --check` | 0 |
| `memcap 8G cargo clippy -j 4 -p rexx-exec -p rexx-core --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| refusal tables + `licensed_divergences` + `numeric_model` (`checks-tables.txt`) | 0 |
| `cargo test --release -p rexx-parse --test sourceline_oracle` (`checks-sourceline.txt`) | 0 |
| `cargo test --release -p rexx-exec --lib` (`checks-lib.txt`) | 0, 1157 passed |
| `cargo test --release -p rexx-core` (`checks-core.txt`) | 0 |
| `REXX_CORPUS_GATE=1 ... --test corpus`, release (`checks-corpus.txt`) and debug (`checks-corpus-debug.txt`) | 0 and 0, 1149 of 1149 each |
| `--test stream_model --no-fail-fast`, release (`checks-stream-model.txt`) and debug | 0 and 0 |
| `REXX_CORPUS_GATE=1 ... --test input_oracle` (`checks-input-oracle.txt`) | 0 |
| `--test ir_recorded --test collect_stress` (`checks-ir-collect.txt`) | 101: only `collect_stress`, at `raise_additional_array.rex`, as at `5698947b0` |
| `REXX_SUITE_GATE=1 ... l3_core_suite` (`suite/`) | 0, 402 groups green, no row changed |

### Perf

`t6.15/r2/callgrind.txt`: parent `cg-9d/t6.15-r1/rexx-head`, head `cg-9d/t6.15-r2/rexx-head`
(`63c7cad28`), exit 0. With libc subtracted:

| program | parent vs pin | head vs pin | own delta |
| --- | --- | --- | --- |
| startup | -0.1202 | -0.1207 | -0.0005 |
| interpret | -4.6331 | -4.6331 | 0.0000 |
| parse | -0.1416 | -0.1417 | -0.0001 |
| churn | +0.1068 | +0.1067 | -0.0001 |
| sayloop | -0.0637 | -0.0639 | -0.0002 |

### Concerns (round 2)

1. **M2 is narrower than the cited sentence.** On a record stream opened WRITE, line positioning
   stays allowed, for the binary-stream exception and the ooTest assertions above. If Moritz
   wants the error there too, `LINEOUT TEST_BINARY`'s earlier assertions fail, and Deviation 46's
   licence then needs a ruling.
2. **The line cache assumes a single writer.** Another stream object writing line ends ahead of
   the cached position leaves the mark stale, and in a debug build the assertion fires.
   rexxref's positioning optimizations make the same assumption (iostrms.xml:495-500).
3. **The console rows in `input_oracle.rs` race on startup.** The harness writes the piped input
   after spawning, so a LINES asked before the bytes arrive answers 0 on both interpreters. It
   was stable in every run here.
4. `collect_stress` still fails at `raise_additional_array.rex`, as at the task base (round 1,
   Concern 1).
5. The new rule labels still need numbers (`new` in `departures-oracle.tsv`).

## Fix round 3

Brief: `task-6.15-fix-r3.md`. Re-review: `task-6.15-rereview-r2.md`. Status: DONE_WITH_CONCERNS.

### Commits

* `01a7ca716`: the debug-only digests on the line cache (`rexx-core` `OpenFile`,
  `dispatch/stream.rs`), the persistent counts for a file console (`input.rs`), the test
  `a_second_writer_leaves_the_line_cache_checks_quiet` (`stream_model.rs`) and
  `a_file_console_counts_what_is_left` (`input_oracle.rs`).
* The records commit that follows holds `t6.15/r3/` and this section.

### Important 1: the line cache under a second writer

* The checks no longer fire on a legal program. `OpenFile` keeps `line_mark_digest` and
  `line_total_digest`, digests of the bytes the mark and the count were counted over. They are
  taken only under `cfg!(debug_assertions)`, so a release build adds no read and no syscall.
* Each check of the cache runs only while its digest still matches the file. A mark counted from
  a mark whose bytes changed gets no digest, so an answer built on stale data is never checked.
  The stream's own writes keep the rest of the scheme: they drop the count, and drop the mark
  when they land at or before it.
* The release answers are unchanged, and may go stale when another writer changes the file, by
  the controller's ruling (iostrms.xml:495-500).
* Test `a_second_writer_leaves_the_line_cache_checks_quiet` runs the re-review's three programs
  in one: two stream objects with a later line count, a second object's CHAROUT ahead of the
  read position, and an `address system` `dd conv=notrunc` overwrite. It asserts rc 0, no
  stderr, and the recorded answers. Run unoptimised it is the test that sees the checks
  (`r3/checks-stream-model-debug.txt`, exit 0, 26 passed). Round 2's debug build panicked on the
  same program (`r3/witnesses/second-writer.round2-debug.txt`).
* Mutation: making `total_intact` answer true turns that test red in a debug build with `the
  cached line count` (`r3/mutation/total-check-ungated.txt`).
* Inverting `line_of`'s check turns the debug named, generated and second-writer tests red with
  `a line counted from the mark` (`r3/mutation/line-mark-assert-inverted.txt`), so the check is
  still live. Each mutation was reversed by a literal `sed`, and `grep` confirmed the line was
  restored.
* Answers on the program (`r3/witnesses/probes.*`):
  * the release and debug crate give `before 2 / after 2 fresh 4 / A aaaa bbbb 3 11 / B 3 3 / C
    aaaa 2 / D 2 4 bbbb 3`;
  * the oracle gives the same except `B 3 1` and `D 2 2`. Both are LINES Counts after the other
    writer, where the crate's cached count and stale mark answer differently. The ruling permits
    this, and the line positions themselves (`B 3`, `D 2`) agree with the oracle.

### Minor 1: a standard input that is a regular file

* `Input::stdin_file_remaining`: when `fstat` of standard input says it is a regular file, CHARS
  answers the buffered bytes plus what the file holds after the descriptor's offset. LINES Count
  answers the lines in that, a last line with no line end included. The file is read with
  `pread`, so the offset does not move.
* Any other standard input keeps round 2's rule.
* Witness `input_oracle.rs`'s `a_file_console_counts_what_is_left`, gated, oracle-identical.
* Probe `r3/probes/stdin-file.rex` on `hello\nworld\nthird\nlast` (`r3/witnesses/stdin-file.*`):
  round 2 answers `C 1 1 1 1`, and head and the oracle answer `C 22 22 1 4`.

### Checks

From `rust/`, `CARGO_TARGET_DIR=<scratchpad>/t615-target`, `CARGO_INCREMENTAL=0`; outputs in
`t6.15/r3/`.

| command | exit |
| --- | --- |
| `cargo fmt --all --check` | 0 |
| `memcap 8G cargo clippy -j 4 -p rexx-exec -p rexx-core --all-targets -- -D warnings` | 0 |
| `cargo test -p rexx-exec --test stream_model --no-fail-fast`, debug (`checks-stream-model-debug.txt`) | 0, 26 passed |
| the same, release (`checks-stream-model.txt`) | 0 |
| `REXX_CORPUS_GATE=1 ... --test input_oracle` (`checks-input-oracle.txt`) | 0, 23 passed |
| `cargo test --release -p rexx-exec --lib` (`checks-lib.txt`) | 0 |
| refusal tables + `licensed_divergences` + `numeric_model` (`checks-tables.txt`) | 0 |
| `REXX_CORPUS_GATE=1 ... --test corpus` | 101 then 0. The first run's one mismatch was `lang/command_redirected_target_waits_elsewhere.rex`'s timed `guard ... waited` line, as in 6.14's first run (`checks-corpus.txt`). The second run is 1149 of 1149 (`checks-corpus-2.txt`) |
| `REXX_SUITE_GATE=1 ... l3_core_suite` (`suite/`) | 0, no row changed against `r2/suite/tests.tsv` |

### Perf

`t6.15/r3/callgrind.txt`: parent `cg-9d/t6.15-r2/rexx-head`, head `cg-9d/t6.15-r3/rexx-head`
(`01a7ca716`), exit 0. With libc subtracted, the head and the parent are equal on every program
(own delta 0.0000).

### Concerns (round 3)

1. **Mixing LINEIN and CHARIN on a standard input differs, before this task too.** After
   `linein()`, `charin(, 3)` answers one character here (`w`), on every binary back to the pin.
   The oracle answers `l`, apparently from a separate buffer. The probe shows it, but nothing
   here changes it. It is a queue candidate.
2. **`command_redirected_target_waits_elsewhere.rex` can mismatch on a timed line**, as in 6.14.
3. `collect_stress` still fails at `raise_additional_array.rex`, as at the task base.

## Fix round 4

Brief: `task-6.15-fix-r4.md`. Re-review: `task-6.15-rereview-r3.md`. Status: DONE_WITH_CONCERNS.

### Commits

* `3cd8cff5c`: `FileConsole` in `input.rs`, the standard-input positioning in
  `dispatch/stream.rs`'s `charin` and `linein`, and the `input_oracle.rs` rows
  `console-charin-after-linein` and `a_file_console_has_one_read_position`.
* The records commit that follows holds `t6.15/r4/`, this section, the queued item's close and
  its inventory row, and one widened rexxref citation in `input.rs`.

### Important 1: `lines()` on a file standard input is linear

* A standard input that is a regular file is `Source::File`, a `FileConsole`: a read position,
  the file read in place at it through a buffer of `STDIN_CHUNK`, the line the position lies in
  once something has counted it, and the file's line count with the size it was counted at.
  `stdin_file_remaining` is gone.
* LINES Normal answers whether a byte remains at the position, from the buffer or one read
  (rexxref funct.xml:3296-3306): no scan.
* LINES Count scans from the position once and keeps the file's line count. Reads advance the
  position's line as they pass line ends, so each later count is the kept total less the lines
  before the position's line, for as long as the file's size stays. A changed size recounts, and
  a position at or past the end answers 0.
* CHARS is the size less the position. `exhausted` plays no part for a file console, so a file
  that grows after its end was read is read on (`r4/witnesses/grow.file.*`), where round 3 kept
  answering 0 and the null string.
* Measurements in `r4/loops/`: the inputs are
  `awk -v n=N 'BEGIN{for(i=1;i<=n;i++) printf "line %d of the input, number %d\n", i, i}'` at
  5000, 10000, 20000 and 200000 lines; `loop.rex` is `do while lines() > 0`, `loopc.rex` is
  `do while lines(, 'C') > 0`, each reading one line per pass, with `loopu.rex` (`do until
  lines() = 0`) and `loopchar.rex` (`do while chars() > 0`, 100 characters per pass) beside
  them. Head is `cg-9d/t6.15-r4/rexx-head` (3cd8cff5c), base a release build of 7a7441c6c, r3
  `cg-9d/t6.15-r3/rexx-head` (01a7ca716), the oracle `build/bin/rexx`. Wall seconds
  (`wall.tsv`, one run each, the file as standard input):

  | program | lines | head | base | r3 | oracle |
  | --- | --- | --- | --- | --- | --- |
  | loop | 5000 | 0.052 | 0.057 | 0.220 | 0.030 |
  | loop | 10000 | 0.085 | 0.088 | 0.769 | 0.031 |
  | loop | 20000 | 0.124 | 0.128 | 3.061 | 0.053 |
  | loop | 200000 | 0.603 | 0.666 | not run | 0.378 |
  | loopc | 5000 | 0.078 | 0.081 | 0.248 | 0.021 |
  | loopc | 10000 | 0.096 | 0.065 | 0.769 | 0.031 |
  | loopc | 20000 | 0.139 | 0.100 | 3.047 | 0.059 |
  | loopc | 200000 | 0.681 | 0.654 | not run | 0.424 |

  Callgrind Ir with libc and ld subtracted (`callgrind.tsv`; `cgsum.py` on a run shaped like
  `callgrind.sh`'s `one`, since the script takes no standard input), at 5000 and 10000 lines,
  and the Ir per line added between them:

  | program | binary | 5000 | 10000 | per line |
  | --- | --- | --- | --- | --- |
  | loop | head | 169,629,874 | 281,876,898 | 22,449 |
  | loop | base | 170,119,083 | 282,862,364 | 22,549 |
  | loop | r3 | 2,305,768,012 | 8,837,847,837 | 1,306,416 |
  | loop | oracle | 67,291,025 | 132,318,709 | 13,006 |
  | loopc | head | 173,075,682 | 288,804,602 | 23,146 |
  | loopc | base | 170,883,550 | 284,386,831 | 22,701 |
  | loopc | r3 | 2,307,667,273 | 8,841,672,110 | 1,306,801 |
  | loopc | oracle | 77,831,545 | 153,524,394 | 15,139 |

  Head's cost per line is the base's, where r3's grows with what is left of the file. The
  base's `lines(, 'C')` never counted (round 3 added the count), so its loopc cost is the
  Normal one.
* The answers are round 3's: `a_file_console_counts_what_is_left` passes, and
  `r4/witnesses/pos.file.*` opens `A 22 1 4` and `B 16 1 3` on both sides.

### The queued item: one read position on a file standard input

* The rule (rexxref funct.xml:1314-1322 and :3080-3094, iostrms.xml:155-165): a persistent
  stream has one read position, from which CHARIN and LINEIN read and which both advance;
  CHARIN's `start` and LINEIN's `line` set it, and neither may be given for a transient
  stream. `charin(, 3)` is therefore character 3 with the default length 1, not three
  characters: the queued item's account of the oracle's answer was a misreading, and its first
  line now says so. The oracle answers `l` there (round 3's `stdin-file.oracle.stdout`,
  `r4/witnesses/pos.file.oracle.out`).
* `FileConsole::seek_char` moves to the character when it is within the file and answers
  `false` past the end, where `charin` answers the null string in `NOTREADY:EOF` and moves
  nothing (`StreamInfo::setCharReadPosition`; `r4/witnesses/past.file.*`). A start on a
  standard input that is not a regular file is 93.958, which `refuse_transient_position`
  already raised for a pipe or a terminal.
* `FileConsole::seek_line` walks line ends to the start of the line, from the position when it
  lies on an earlier line and else from the start of the file, and stops at the end of the file
  for a line past the last one. A seek to the line the position lies in goes back to its start.
* `.stdin`'s `charin`, `linein`, `chars` and `lines` go through the same console, so
  `.stdin~charin(2, 2)` and `.stdin~linein(4)` position too.
* Witnesses, gated and oracle-identical (`r4/checks-input-oracle.txt`):
  `console-charin-after-linein` (piped: `linein()`, then `charin(, , 3)` answers `wor`, then
  `charin(, 3)` is 93.958 and rc 163) and `a_file_console_has_one_read_position` (a file:
  every move and count of `FILE_POSITION_PROGRAM`, with the description after a start past the
  end). Unit tests in `input.rs` run the console over a temporary file: the same sequence,
  seeks past the end, a line seek from mid-line and over every line, the count cache against a
  fresh count after every read and after seeks either way, a file that grows after its end was
  read, and the descriptor's offset as the starting position.
* Probes in `r4/probes/`, run on `in22.txt` (`hello`, `world`, `third`, `last`, no final line
  end) as a file and as a pipe, witnesses in `r4/witnesses/`, the comparison in
  `r4/witnesses-diff.txt`: the error probes (`zero`, `neg`, `zeroline`: 93.907 on a file,
  93.958 on a pipe), `linepos`, `linepos-named` and every pipe run agree with the oracle on all
  three channels. The file runs that differ are below.

### Oracle departures seen

Each is the oracle's cached line information or cached size, each falls under Deviation
candidate 5's rule (LINES Count from the read position) or the ruling that a count may go stale
under another writer (iostrms.xml:495-500), and nothing here copies them.
* `pos.file` H and I: after `linein(, 1)` and `linein(, 3)` the oracle's Count answers 1 and
  -1 where 3 and 1 lines remain (`StreamInfo::lines` answers
  `stream_line_size - lineReadPosition + 1` from a cache `setLineReadPosition` does not
  refresh). `count.file` D and `count-named.file` D: the oracle answers 3 where 2 remain, on
  the standard input and on a named stream alike, which is candidate 5's own measurement.
* `lenzero.file`: `linein(, 3, 0)` after a CHARIN moved the position leaves the oracle at line
  2 (CHARS 16, then `world`) where line 3 starts at character 13 (CHARS 10, then `third`).
  `linepos.file`, the same seeks without the earlier `linein()`, agrees on both sides.
* `past.file`: `linein(, 4)` after a line read found the end of the file answers the null
  string on the oracle and `last` here; `charin(, 1)` right after reads `h` on both.
* `grow.file`: lines appended by `address system` after the end was read are counted here
  (CHARS 11, LINES 1, Count 2) and not by the oracle (0 0 0), which reads the appended line all
  the same.

### Checks

From `rust/`, `CARGO_TARGET_DIR=/tmp/claude-1000/t615r4-target/head`, `CARGO_INCREMENTAL=0`,
`-j 4`; outputs in `t6.15/r4/`, each exit status in `checks-status.txt` with the commit it ran
on (3cd8cff5c) as its first line.

| command | exit |
| --- | --- |
| `cargo fmt --all --check` | 0 |
| `memcap 8G cargo clippy -p rexx-exec -p rexx-core --all-targets -- -D warnings` | 0 |
| `memcap 16G cargo test -p rexx-exec --test stream_model --no-fail-fast`, debug (`checks-stream-model-debug.txt`) | 0, 26 passed |
| the same, `--release` (`checks-stream-model.txt`) | 0, 26 passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test --release -p rexx-exec --test input_oracle` (`checks-input-oracle.txt`) | 0, 24 passed |
| `memcap 16G cargo test --release -p rexx-exec --lib` (`checks-lib.txt`) | 0, 1163 passed |
| refusal tables + `licensed_divergences` + `numeric_model` (`checks-tables.txt`) | 0 |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 0, 1149 of 1149 |

### Perf

`t6.15/r4/callgrind.txt`: `callgrind.sh -B 0.5 -j 4 -p "startup interpret parse churn
sayloop"`, pin `rexx-run`, parent `cg-9d/t6.15-r3/rexx-head` (01a7ca716), head
`cg-9d/t6.15-r4/rexx-head` (3cd8cff5c), exit 0. Own deltas with libc subtracted, head less
parent: startup +0.0005, interpret 0.0000, parse +0.0001, churn 0.0000, sayloop +0.0003.

### Concerns (round 4)

1. **The queued item described the oracle wrongly**, as answering three characters where it
   answers the one at position 3. Its first line now carries the measured answer and the
   rule; the inventory row is `closed:3cd8cff5c` in `proposed` and `ruled`, with a prose
   `reason`, the file's own convention for a closed row.
2. **The oracle's LINES Count is wrong after a LINEIN line, on a named stream as on standard
   input**, and `linein(, n, 0)` after a CHARIN lands on the wrong line: both from the cache
   behind Deviation candidate 5. The `r4/probes/` programs are the measurements, should the
   controller want them on that candidate.
3. **Standard input that is neither a regular file nor a pipe or terminal** (a socket, a
   directory) is "not transient" to `rexx.rs`'s classification and not a file console here,
   so a start there is 93.958 from `input_seek_char` rather than from
   `refuse_transient_position`; the oracle says 93.958 for anything that is not a regular
   file. Not measured on a socket.

## Fix round 5

Brief: `task-6.15-fix-r5.md`. Re-review: `task-6.15-rereview-r4.md`. Status: DONE_WITH_CONCERNS.

### Commits

* `47e5ad8de`: `lines.rs`, the file console over an `OpenFile`, the descriptor sync before a
  command (`command.rs`'s `start`, `rexxutil/process.rs`'s `cls`) and on drop, the
  `input_oracle.rs` row `a_file_console_hands_a_command_what_is_unread`, and the unit tests.
* `ae39e1981`: a read on the console carries the line mark it starts on; `line_count` takes the
  size only where it needs it.
* `9a886720e`: `corpus/refusal-sites.tsv` re-derived (`REXX_REFUSAL_SITES_REFRESH`): `mod lines;`
  moved every `lib.rs` row down one line, and nothing else in the table changed.
* The records commit that follows holds `t6.15/r5/` and this section.

### Important 1: the descriptor's offset is the read position whenever another process can see it

* `FileConsole::sync_offset` seeks the descriptor, a duplicate of fd 0 sharing its open file
  description, to the read position. `Interp::sync_input_offset` calls it in `command::start`
  before any ADDRESS command spawns, redirected or not, and in `SysCls`; `impl Drop for
  FileConsole` calls it, which is the interpreter's end. The console's own reads go through
  `lines::read_from`, so the offset moves while the program runs and is put back before
  anything else can read from it.
* Measured (`r5/child-probes.txt`, per channel in `r5/witnesses/`), what the child or successor
  reads, as `wc -c` counts it:

  | probe | input | oracle | head | r4 | r3 | after the read position |
  | --- | --- | --- | --- | --- | --- | --- |
  | `childwc` (`linein()`, `address system 'wc -c'`) | `in22.txt` | 0 | 16 | 22 | 0 | 16 |
  | `childwc` | `b4096.txt` (a 4096-byte first line, then `rest`) | 5 | 5 | 4101 | 0 | 5 |
  | `childwc` | the 7,777,790-byte 200000-line file | 7773694 | 7777760 | 7777790 | 7769598 | 7777760 |
  | `exit` (`(rexx one.rex; wc -c) < in22.txt`) | `in22.txt` | 0 | 16 | 22 | 0 | 16 |

  `chars()` after the command answers the same figure on every binary (`C 16 1`, `C 5 1`,
  `C 7777760 1`), which is the documented answer. The oracle hands over what is left after its
  own 4096-byte read, round 3 what is left after its 8192-byte chunk, round 4 the whole file
  again. `child.rex` (`linein()`, `address system 'cat'`, `linein()`) prints the three unread
  lines between `L hello` and `M world` here, where round 4 printed all four. The pipe is
  unchanged (`child.pipe.*`: `L hello / M world / N 1 1` on both).
* Row `a_file_console_hands_a_command_what_is_unread`, gated and oracle-identical: the first line
  fills the oracle's read exactly, so both answer `4095 / 5 / C 5 1`. Unit test
  `a_file_console_leaves_the_descriptor_at_the_read_position`: a duplicate of the descriptor
  stands at the read position after `sync_offset` and after the console is dropped.

### Important 2: one copy of the line code

* `lines.rs` holds `CHUNK`, `as_position`, `offset_of`, `read_from`, `size_of`,
  `count_lines_from`, `line_count`, `digest_before`, `mark_intact`, `total_intact`, `set_mark`,
  `lines_after`, `seek_line`, `seek_forward_lines`, `line_of` and `line_ends_between`, moved
  from `stream.rs` as they were, `pub(crate)`; `stream.rs` imports the ones it calls.
* `FileConsole` holds an `OpenFile` (its `read_position`, `line_mark`, `line_total` and digests)
  and calls `lines_after`, `seek_line`, `read_from` and `size_of`. Its own walkers and line
  tracking are gone. What it adds is `counted_size`, the file's size at the last count: nothing
  here writes the file, so the stream's write-side invalidation does not apply, and a changed
  size drops the count instead.
* `FileConsole::advance` carries a line mark sitting at the old position along with a read, its
  digest dropped, so a count after each read walks nothing and a Count loop stays linear.
* Answers unchanged: round 4's probe set re-run on head answers byte for byte as round 4's head
  did on every probe, and differs from the oracle on the same five file-side cases
  (`r5/witnesses-diff.txt`); `grow.in22` is round 4's `B 11 1 2 / [more] / C 6 1 1`.
* Perf: `r5/callgrind.txt`, own deltas against the round-4 head 0.0000 on every program.
  Loops (`r5/loops/callgrind.tsv`, Ir with libc and ld subtracted, per line added between 5000
  and 10000 lines): `loop` 22,494 here against round 4's 22,449, `loopc` 23,385 against 23,146.
  Wall seconds at 200000 lines, two runs each (`wall.tsv`): `loop` 0.608 and 0.652 against
  0.632 and 0.663, `loopc` 0.748 and 0.858 against 0.716 and 0.716 (oracle 0.424 and 0.451).
  The Count loop pays one more `fstat` per count (`lines_left` and `line_of` each take the
  size) and a seek and read where round 4 used `pread`.

### Minor 1: the size check is witnessed

* `a_file_console_recounts_a_grown_file` counts 3, reads `a`, counts 2, has another writer append
  `x\ny\n`, and counts 4.
* Mutation `counted_at == size` to `counted_at <= size` (`r5/mutation/size-check-le.txt`): that
  test fails with `left: 2, right: 4` and the other file-console tests pass. Reversed by a literal
  `sed`; `grep` finds `==` once and `<=` nowhere.
* `grow2.abc`, the re-review's program with the append on the standard input file: `A 3 a / B 8
  4` here, `A 3 a / B 4 2` on the oracle (its stale size and count, iostrms.xml:495-500).

### Checks

From `rust/`, `CARGO_TARGET_DIR=/tmp/claude-1000/t615r5-target/head`, `CARGO_INCREMENTAL=0`,
`-j 4`; outputs in `t6.15/r5/`, statuses in `checks-status.txt` with the commit each ran on.

| command | exit |
| --- | --- |
| `cargo fmt --all --check` | 0 |
| `memcap 8G cargo clippy -p rexx-exec -p rexx-core --all-targets -- -D warnings` | 0 |
| `memcap 16G cargo test -p rexx-exec --test stream_model --no-fail-fast`, debug (`checks-stream-model-debug.txt`) | 0, 26 passed |
| the same, `--release` (`checks-stream-model.txt`) | 0, 26 passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test --release -p rexx-exec --test input_oracle` (`checks-input-oracle.txt`) | 0, 25 passed |
| `memcap 16G cargo test --release -p rexx-exec --lib` (`checks-lib.txt`) | 0, 1165 passed |
| refusal tables + `licensed_divergences` + `numeric_model` on `ae39e1981` (`checks-tables-stale.txt`) | 101, `refusal_sites` on the moved `lib.rs` rows |
| the same on `9a886720e` (`checks-tables.txt`) | 0 |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 0, 1149 of 1149 |

### Perf

`t6.15/r5/callgrind.txt`: `callgrind.sh -B 0.5 -j 4 -p "startup interpret parse churn
sayloop"`, pin `rexx-run`, parent `cg-9d/t6.15-r4/rexx-head` (3cd8cff5c), head
`cg-9d/t6.15-r5/rexx-head` (ae39e1981), exit 0. Own deltas with libc subtracted, head less
parent: 0.0000 on startup, interpret, parse, churn and sayloop.

### Concerns (round 5)

1. **The oracle's child sees what is left after the oracle's own buffer**, not after the read
   position, so the gated row uses an input where the two coincide and the general case is a
   probe (`childwc.in22`, `childwc.big`, `exit.in22`).
2. **The Count loop's wall time is up at 200000 lines** against round 4 (0.748 and 0.858 against
   0.716 and 0.716), from syscalls rather than instructions; it stays linear.
3. **A successor process sees the rest only after a normal end.** The sync on drop does not run
   when the interpreter is killed by a signal; the offset then stands wherever the last chunk
   read left it.

## Fix round 6

Brief: `task-6.15-fix-r6.md`. Re-review: `task-6.15-rereview-r5.md`. Status: DONE_WITH_CONCERNS.

### Commits

* `0da8c4c64`: the conditional sync and the adoption in `input.rs`, the adoption after every
  ended command (`command.rs`'s `command_ended`, `rexxutil/process.rs`'s `cls`), the row
  `a_command_moves_the_shared_read_position` and the shortened `FILE_CHILD_PROGRAM` in
  `input_oracle.rs`, the unit test `a_command_that_consumed_moves_the_read_position`, and one
  temporary file name per console test.
* The records commit that follows holds `t6.15/r6/` and this section.

### Important 1: the descriptor is the read position commands share

* `FileConsole::sync_offset` seeks only when the descriptor's offset is not the read position,
  which is exactly when the console's own reads moved it: after a command returns,
  `FileConsole::adopt_offset` takes the read position from the offset, drops the line mark and
  the read-ahead, and the two agree until the console reads again. So a command, or a
  successor process, continues from where the previous command stopped, and a program that
  has read hands over the bytes after its read position.
* `Interp::adopt_input_offset` runs in `command_ended`, which every spawned command reaches
  from `run_command` and `end_blocked_command`, and in `SysCls` after its child (a null
  standard input there, so nothing moves). A registered command handler spawns nothing and
  adopts nothing.
* Measured (`r6/probes.txt`, per channel in `r6/witnesses/`), on `five.txt` (`one` to `five`,
  24 bytes) unless named:

  | probe | oracle | head | r5 | r4 |
  | --- | --- | --- | --- | --- |
  | `two` (`'head -1'`, `'head -1'`, `'wc -c'`) | `one / two / 16` | `one / two / 16` | `one / one / 24` | `one / two / 16` |
  | `h1` (`'head -1'`) then `cat` in the shell | `one / two..five` | `one / two..five` | `one / one..five` | `one / two..five` |
  | `two2` (`linein()`, `'head -1'`, `'head -1'`, `linein()`) | `L one / M two` | `L one / two / three / M four` | `L one / two / two / M two` | `L one / one / two / M two` |
  | `wc1` (`linein()`, `'wc -c'`, `chars() lines()`) | `L one / 0 / C 20 1` | `L one / 20 / C 0 0` | `L one / 20 / C 20 1` | `L one / 24 / C 20 1` |
  | `childwc` on `in22.txt`, `b4096.txt`, the 200000-line file: the child | 0, 5, 7773694 | 16, 5, 7777760 | 16, 5, 7777760 | 22, 4101, 7777790 |
  | `exit` (`linein()`) then `cat` in the shell, `in22.txt` | nothing | `world / third / last` | the same | `hello / world / third / last` |

  `two` and `h1` are the re-review's required observables and answer as r3, r4 and the oracle
  do. Every child still sees the bytes after the read position. What is new is the row after
  a command that consumed: `two2` reads on from where the second `head` stopped, and `wc1`'s
  counts answer 0 once `wc` has taken everything, where the oracle's buffered position is
  untouched by its children. That is the ruling's "the program's next read must start there",
  and it is an oracle departure the ruling makes; `FILE_CHILD_PROGRAM` no longer prints the
  counts after `wc`, so its row stays oracle-identical.
* Rows, gated and oracle-identical: `a_command_moves_the_shared_read_position` (`one / two /
  16`) and `a_file_console_hands_a_command_what_is_unread` (`4095 / 5`). Unit test
  `a_command_that_consumed_moves_the_read_position`: a seek on a duplicate stands for the
  command; after `adopt_offset` the console counts and reads from there, a sync with nothing
  read since moves nothing, and a sync after a read puts the offset at the read position.

### Important 2: one temporary file per console test

* `a_file_console_leaves_the_descriptor_at_the_read_position` now uses `console("sync", ..)`
  and `a_file_console_starts_where_the_descriptor_is` writes `rexx-file-console-<pid>-start`;
  every other `console(..)` caller already had a name of its own (`adopt`, `count`, `grow`,
  `midline`, `past`, `position`, `recount`).
* `r6/lib-runs/summary.txt`: the release `--lib` binary of `0da8c4c64`, run from
  `crates/rexx-exec` with the default test threads, 40 times in full: 1166 passed every time,
  no non-zero exit; 40 times filtered to `file_console`: 7 passed every time. (A first set of
  40 runs from `rust/` failed the same three `rexxutil` and `library` tests every time, which
  resolve paths against the crate directory; the console tests passed in those too.)

### Checks

From `rust/`, `CARGO_TARGET_DIR=/tmp/claude-1000/t615r6-target/head`, `CARGO_INCREMENTAL=0`,
`-j 4`; outputs in `t6.15/r6/`, statuses in `checks-status.txt` with the commit they ran on.

| command | exit |
| --- | --- |
| `cargo fmt --all --check` (`checks-fmt.txt`) | 0 |
| `memcap 8G cargo clippy -p rexx-exec -p rexx-core --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `memcap 16G cargo test -p rexx-exec --test stream_model --no-fail-fast`, debug (`checks-stream-model-debug.txt`) | 0, 26 passed |
| the same, `--release` (`checks-stream-model.txt`) | 0, 26 passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test --release -p rexx-exec --test input_oracle` (`checks-input-oracle.txt`) | 0, 26 passed |
| `memcap 16G cargo test --release -p rexx-exec --lib` (`checks-lib.txt`) | 0, 1166 passed |
| refusal tables + `licensed_divergences` + `numeric_model` (`checks-tables.txt`) | 0 |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 0, 1149 of 1149 |

### Perf

`t6.15/r6/callgrind.txt`: `callgrind.sh -B 0.5 -j 4 -p "startup interpret parse churn
sayloop"`, pin `rexx-run`, parent `cg-9d/t6.15-r5/rexx-head` (ae39e1981), head
`cg-9d/t6.15-r6/rexx-head` (0da8c4c64), exit 0. Own deltas with libc subtracted, head less
parent: 0.0000 on startup, parse, churn and sayloop, -0.0001 on interpret.

### Concerns (round 6)

1. **The adoption is an oracle departure by ruling.** After a command that consumed standard
   input, the program reads on from where the command stopped (`two2`: `L one / two / three /
   M four`) and its counts say what is left from there (`wc1`: `C 0 0`), where the oracle's
   buffered position is untouched (`L one / M two`, `C 20 1`). The ruling asked for exactly
   this; it is not written up as a Deviation here.
2. **A command's own seek is what makes `two` answer `one / two`.** `head` leaves the
   descriptor after the line it printed on a seekable input; a reader that takes a whole
   buffer and does not seek back (`cat`, `wc`) leaves the descriptor at the end, and the next
   command or read gets nothing. That is the same on r3, r4 and the oracle.
3. **The sync on drop still does not run when the interpreter dies by a signal.**
