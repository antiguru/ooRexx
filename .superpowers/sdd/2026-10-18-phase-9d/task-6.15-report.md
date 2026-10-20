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
  lines print.
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
