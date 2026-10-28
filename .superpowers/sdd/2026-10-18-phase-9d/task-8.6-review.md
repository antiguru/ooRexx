# Task 8.6 review: streams, paths, commands

Reviewed `0fd10a074..8ed7774e1` (code `36866b601..b53b1de0c`, records after). Verdicts: spec
**COMPLIANT**; quality **CHANGES REQUESTED** (0 Critical, 2 Important, 3 Minor).

## What I ran

* Builds, each `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 --bin rexx`, exit 0:
  BASE `0fd10a074` from a `git archive` (rust and interpreter) into
  `/tmp/claude-1000/t86-review-target-base`; HEAD `8ed7774e1` from the checkout into
  `/tmp/claude-1000/t86-review-target-head`; perf controls from archives, BASE plus
  `#[inline(never)]` on `fragment_plan` (`base-ni`) and HEAD without it (`head-noni`). All deleted at
  the end.
* Probes under `t86-review-scratch/probes/`, each run in a fresh directory per binary
  (`t86-review-scratch/run3.sh`): oracle under the standard wrapper, crate under `memcap 2G timeout
  -k 5 20`, stdin `/dev/null`, stdout piped (and to a file where named), stdout, stderr and rc
  compared separately. Run outputs were deleted; the deciding lines are quoted below.
* `cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test stdout_write_errors --test
  program_end --test rexxutil_process --test signals --test rexx_command --test stream_model --test
  stream_handle --test input_oracle --test stdin_contention --test prompt_before_read`: every binary
  `ok` (lib 1200, program_end 8, rexxutil_process 3, signals 42, stdout_write_errors 2, stream_model
  26).
* `REXX_CORPUS_GATE=1 ... --test corpus`: exit 101, the one mismatch the known
  `lang/rexxutil_file_date_time.rex`.
* `bench-programs/callgrind.sh -r 3 -j 4` on `interpret fibfunc rexxcps dispatchclass` over
  base, base-ni, head, head-noni, and on `pingguard pingsem` over base, head, base-ni (raw in
  `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.6-review/`).

## Spec compliance

COMPLIANT. Each closed item's witness is red at BASE and matches the oracle at HEAD on all three
descriptors:

| witness | BASE vs oracle | HEAD vs oracle |
|---|---|---|
| `path_names_not_utf8` | stdout differs | identical |
| `stream_writes_held` | stdout, stderr differ | identical |
| `stream_standard_open` | stdout differs | identical (also with stdin a file and a pipe) |
| `command_background_child` | stdout, stderr, rc differ (timeout) | identical |

The queued items' own probes agree: `streamflush`, `lossy` and `bgcmd` are red at BASE and match at
HEAD; `stdinq` matches except `chars 1` against `chars 0` on a `/dev/null` stdin, the oracle's
uninitialised `FIONREAD` read (Concern 4). The closed items' files carry the `**CLOSED 2026-10-28 by Phase 9d
Task 8.6 (b53b1de0c).**` first line; their inventory rows and the gate row have `closed:b53b1de0c`
in columns 4 and 6; `t8.6/closed.tsv` holds `gate:phase-9c-gate.md:146	closed:b53b1de0c`, and the
report closes the gate line. The stopped item (stdout write errors) keeps `9d` and is reported with
Deviation candidates A and B, as the brief requires (its file now carries Moritz's ruling).

## Strengths

* **Byte paths hold beyond the witness.** In `probes/bytes1.rex` and `probes/bytes2.rex` (names with
  `'ff'x` and `'e9'x`, decoys named `'efbfbd'x`) HEAD matches the oracle on `.File`'s `name`,
  `absolutePath`, `parent`, `isDirectory`, `length`, `canWrite`, `lastModified` and its setter,
  `isHidden` (BASE answered 0), `list` and `listFiles` round trips (`69E9` where BASE answered
  `69EFBFBD`), `makeDir`, `delete`, `searchPath` (also with a trailing `/` and a missing directory
  first), `QUERY EXISTS`, `QUALIFY`, `.Stream~qualify`, `QUERY DATETIME`/`TIMESTAMP`, explicit
  `open write` by name, `SysFileTree`, `SysFileExists`, `SysIsFile`, `SysIsFileDirectory`,
  `SysFileDelete` (BASE `13`), and, inside a non-UTF-8 current directory, relative queries, an external
  routine call (`ext ff`, BASE `ext decoy`), `Routine~newFile`, `pwd` and `directory()`.
* **Held writes survive every program end I tried.** `lineout` by name, `charout` and an unclosed
  object's write reach the file on a normal end, `exit 3`, a nested `exit 4`, a SYNTAX error, `raise
  halt`, SIGINT and SIGTERM, and from an activity still running after `reply`; a program-made STDOUT
  stream kept in `.local` or `.environment` is written at the end (oracle loses the `.environment`
  one).
* **Write errors to files match the oracle line for line** (`probes/devfull.rex`): `lineout` and
  `charout` to `/dev/full` answer `0 READY:`, `FLUSH` `ERROR:28`, a 5000-byte write `1 ERROR:28`,
  `CLOSE` `READY:`, and a `NOBUFFER` open `1 ERROR:28`, where BASE answered `1 ERROR:28` for each
  write.
* **Inherited command output interleaves as the oracle's does.** With stdout a pipe, a file, and
  2>&1 into one file, SAY, `.stderr`, TRACE lines and command output keep their order, and an
  unredirected `echo c1; echo ce1 >&2` now writes `c1` before `ce1` as on the oracle (BASE had them
  reversed). The remaining differences are Deviation 40's buffered SAY, the same at BASE. Killed by
  PID with SIGKILL during `'sleep 2.5; echo late'`, HEAD's child writes `late` to the inherited file
  or pipe as the oracle's does (BASE's child died); after SIGINT/SIGTERM the HALT returns at 1.0 s on
  all three and the child's `late` arrives on the oracle and HEAD. No child was left over.
* **The `fragment_plan` `#[inline(never)]` is a sound fix, not layout luck.** The control:

  | program | base-ni -libc | head -libc | head-noni -libc |
  |---|---|---|---|
  | interpret | +0.0153% | +0.0164% | +0.1561% |
  | fibfunc | +0.0000% | +0.0213% | +0.0213% |
  | rexxcps | +0.0000% | -0.0033% | -0.0033% |
  | dispatchclass | -0.0000% | +0.0000% | +0.0000% |

  The attribute costs BASE +0.0153% on interpret (the out-of-line call), and HEAD against base-ni is
  +0.0012%. So it hides no win; without it HEAD's inlined loop costs +0.156%, as the report found.
* **Memory is flat.** A loop of `.stream~new('f.txt')~lineout`, `lineout('g.txt')` and a
  program-made STDOUT `charout('')`, closing every hundredth object: 47.2 MB at 20000 and 48.0 MB at
  80000 iterations (BASE 47.4 and 49.9).
* **The reaped-child retry hides no regression.** 20 direct runs: BASE 20 of 20 `SysWait` wins, HEAD
  19 of 20, the other `cmd rc 5` and `wait -1`. Both outcomes are right for the schedule they took:
  with no pipe reader in front of it the command's own `waitpid` now races the `SysWait`'s, which the
  pipe reads used to delay. The test still requires the 127 path to occur.

## Issues

### Critical

None.

### Important

1. **A held write on an APPEND stream lands at a stale offset and overwrites what others appended
   since.** `OpenFile::write` (`rexx-core/src/body.rs:637`) holds bytes at the offset `landing`
   (`dispatch/stream.rs:1754`) computed when the write was made, and writes them there by `seek` at
   the flush. rexxref streamclasses.xml APPEND: "Positions the write pointer at the end of the
   stream"; the oracle opens APPEND with `O_APPEND` (`SysFile.cpp:158`, `:594`), so its buffer lands
   at the end as the file stands at the flush. BASE wrote at once. `probes/append.rex` and
   `append2.rex`, files read back with `charin`, line ends shown as `|`:
   * `b~open('write append'); b~lineout('b1'); 'echo cmd1 >> b.txt'; b~lineout('b2'); b~close`:
     oracle `cmd1|b1|b2|`, BASE `b1|cmd1|b2|`, HEAD `b1|1|b2|` (the command's line is half
     overwritten).
   * the same with `open('both append')`: oracle `cmd1|b1|b2|`, HEAD `b1|1|b2|`.
   * two objects `open('write append')` on one file, `d1`, then `e1` closed, then `d2`: oracle
     `e1|d1|d2|`, BASE `d1|e1|d2|`, HEAD `d1|d2|` (`e1` lost).
   A plain `open('write')` and an implicit open are not O_APPEND on the oracle and lose the
   command's line on both, so they agree (`write b1|b2|`, `implicit c1|c2|`). Fix: on an APPEND
   stream, a flush writes at the file's end as it stands then (open with `O_APPEND`, or seek to the
   end at the flush).

2. **Reopening `.STDOUT` or `.STDERR` without NOBUFFER splits them from SAY and trace, reorders their
   writes and loses them at program end.** `open` (`dispatch/stream.rs:700`) sets a standard
   stream's `nobuffer` from the options, so `.stdout~open('write')`, `stream('STDOUT', 'c', 'open
   write')` or `.stderr~open('write')` makes the `.local` object hold its writes in
   `StreamState::held` (`:1123`), while SAY and TRACE still go to the interpreter's own buffer. SAY
   writes `.OUTPUT`, which is `.STDOUT`: one stream, so its order is the order written; and a stream
   "remains open until you explicitly close it ... or until the program ends" (rexxref iostrms.xml),
   which writes it. The `.local` objects are never UNINITed, so nothing releases what they hold.
   stdout piped:
   * `probes/reo5.rex` (`.stdout~open('write'); .stdout~lineout('x'); say 'y'; .stdout~flush; say
     'z'`): oracle `x y`, BASE `x y z`, HEAD `y x z`.
   * `reo1.rex` (the same without the FLUSH, `.stdout~lineout('z')` last): oracle nothing, BASE `x y
     z`, HEAD `y` (x and z lost).
   * `reo3.rex` (by name: `call stream 'STDOUT', 'c', 'open write'`, `lineout 'STDOUT'`): HEAD `y`.
   * `reo4.rex` (`.stderr~open('write')`, then `.stderr~lineout`, `lineout 'STDERR'` by name):
     stderr oracle empty, BASE `x y z`, HEAD empty.
   The oracle loses these too, SAY included (new oracle defect 2 below), so this is not an oracle
   answer to copy. BASE's answer is the rule's. Fix: keep the `.local` standard streams unbuffered
   on a reopen (their buffering is Deviation 40's, shared with SAY), or route SAY and TRACE through
   the same held bytes and release them at program end.

### Minor

1. **Every native-state object grew, not only streams.** `NativeState::Stream(StreamState)`
   (`body.rs:180`) is the enum's largest variant, and `held` plus `OpenFile`'s new fields grow every
   `Box<NativeState>`: MutableBuffer, Buffer, Pointer, String and Package subclass instances. 200000
   `.mutablebuffer~new('x')` kept in an array: max RSS 134.4/134.9 MB at BASE, 143.2/143.6 MB at HEAD
   (about 44 bytes each); instructions unchanged (callgrind 371.36M against 370.98M at 50000). The
   crate boxes `PackageState` in `Bound` for this reason ("so that every other native object stays
   its size", `body.rs:1185`). Fix: box the stream state.

2. **pingguard's +0.0521% libc subtracted is deterministic, not schedule noise.** My run reproduces
   it to the fourth decimal (`1151744880` against `1152344613`, spread 0.0002%, three rounds), as the
   report's final run does; pingsem +0.0340% likewise. The batches header exempts SPREAD flags, and
   neither run flags one, so pingguard is over the +0.05% limit by 0.0021%. The per-function diff
   shows about one instruction more per `resume_parked`, `resume_woken` and `exec_guard` call
   (199998 calls each), a code-generation shift like the `fragment_plan` one. The report calls it
   "schedule-dependent"; the controller rules whether it stands (Moritz's ruling 14 says the budget
   guards obvious regressions).

3. **Candidate A does not cover FLUSH.** `probes/werr.rex` under `/dev/full`, a closed pipe and
   `>&-`: for a program-made STDOUT stream, `o~lineout('f')` then `o~flush` answers on the oracle `0
   READY:` then `ERROR:28` (`ERROR:32`, `ERROR:9`); HEAD answers `READY:` for the FLUSH, because the
   held bytes go to the Deviation 40 buffer and no hand-over happens. Ruled candidate A's sentence
   is about "the stream writes after the hand-over"; FLUSH is the operation rexxref says writes to
   the operating system, so the Deviation 40 sentence should name it, or FLUSH should hand over.

### Perf judgment

* interpret +0.5055% whole process is libc only (+0.0164% subtracted), so within the rule. The
  report's attribution to `StreamState`'s size is not the whole story: in the control, fibfunc's
  and dispatchclass's whole-process deltas (+0.1399%, +0.1466% at HEAD) fall to +0.0197% and
  +0.0000% in head-noni, which differs only by the `#[inline(never)]`. The libc part moves with the
  binary's layout, so it is not a per-operation cost and the libc-subtracted figure is the one to
  judge.
* pingguard: Minor 2.

## Observed consequences of the buffering rule (no finding)

* `call lineout 'in.txt', ...` twice, unclosed, then `address system 'cat' with input stream
  'in.txt'`: HEAD feeds the command nothing (BASE `l1 l2`). The command's input reads the file through
  a stream of its own from position 1 on both (after a `linein` the oracle and HEAD still feed `l1
  l2`), so it sees only flushed bytes. The oracle hangs here (new oracle defect 1).
* 2000 unclosed `.stream~new('f.txt')~lineout` objects, every hundredth closed: the oracle and HEAD
  both leave 22 lines (each held write lands where its object opened), BASE 2000.

## Queue candidates (predate the task)

1. **LINES on a write-only stream.** `f = .stream~new('f.txt'); f~open('write'); f~lineout('f1'); say
   f~lines`: oracle `1`, BASE and HEAD `0`.
2. **A `/dev/null` standard input is transient here.** `.stream~new('STDIN')~seek('1')` with stdin
   `/dev/null`: oracle `1` and `READY`, BASE and HEAD raise 93.958 ("Positioning of transient streams
   is not valid"), since `rexx` counts a character device as transient. With stdin a file the oracle
   answers `1 READY` and HEAD `0 ERROR`, which is the report's Concern 3.

## The report's oracle defects

* Program-made STDOUT on a regular-file stdout overwrites earlier output: **confirmed**.
  `probes/stdbuf.rex` with stdout a file: the oracle's file holds `last say`, `never closed`,
  `colon`, the x run and `after big`; `nb1`, `nb2` and `after colon` are gone. `SysFile.cpp:533-543`
  seeks the shared descriptor to the object's own position.
* `SysFile::hasData` reads an uninitialised `bytesWaiting`: **confirmed**. `SysFile.cpp:1311-1313`
  ignores `ioctl`'s result, and `lines('STDIN') chars('STDIN')` with stdin `/dev/null` answers `1 1`
  on the oracle.

## New oracle defects

1. **WITH INPUT STREAM naming an empty file hangs.** `'touch e.txt'; address system 'cat' with
   input stream 'e.txt'; say 'rc' rc`: the oracle is halted by `timeout` (rc 124, `Error 4.1`), BASE
   and HEAD print `rc 0`. The same command on a non-empty file returns.
2. **A `.STDOUT` or `.STDERR` reopened without NOBUFFER buffers SAY too and drops the buffer at
   program end.** `.stdout~open('write'); .stdout~lineout('x'); say 'y'`: the oracle prints nothing.
   `say 'a'; .stdout~open('write')`, then 1000 SAYs of ten bytes: 820 lines arrive (two 4096-byte
   pieces), the tail is lost. `.stdout~close` followed by writes loses them the same way, and
   `.stderr~open('write')` loses everything after it on stderr. rexxref iostrms.xml: a stream stays
   open "until the program ends".
