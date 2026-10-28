# Phase 9d Task 8.6 report: streams, paths, commands

Status: DONE_WITH_CONCERNS. Base `0fd10a074`. Code `36866b601`, `b74e6a848`, `6fdcb7d7a`,
`19e478b98`, `b6cf1c525`, `7596ced8d`; records and this report in the commit after. The closing lines
and inventory rows name `7596ced8d`. Records: `docs/superpowers/records/2026-10-18-phase-9d/t8.6/`
(`probes/` holds the probes, `witnesses/` each witness's oracle `.o.*`, base and head outputs).

## What changed, per item

**2026-10-13-lossy-path-sites.md and gate:phase-9c-gate.md:146 (closed; the gate line points at the
same file, `t8.6/closed.tsv`).** Rexx strings are bytes, so every path reaches the file system through
`paths::os_path` (moved from `rexxutil.rs`), never a lossy UTF-8 conversion: `DIRECTORY` (enters and
answers the bytes), a command's working directory, `.Stream` queries, explicit and implicit opens,
`.File`'s path argument, `renameTo`, `searchPath` (the list is split as bytes), and
`Routine~newFile`'s read. `Interp::cwd_text` and the `&str` `paths::normalize` are gone. The SysFileTree
sites the item lists had already been made byte-preserving. Witness `path_names_not_utf8.rex`: every
file is made by a shell command, each site has a decoy named `'efbfbd'x` where the lossy conversion
would land, and at base every line of the witness differs.

**2026-09-27-stream-write-visibility.md (closed).** Rule: rexxref streamclasses.xml, `NOBUFFER` "Turns
off buffering of the stream. All data written to the stream is flushed immediately to the operating
system"; a stream is otherwise buffered, and `FLUSH` and `CLOSE` write the buffer. The crate wrote
every byte at once. `rexx_core::OpenFile` now holds a stream's writes as `SysFile::write` does
(`common/platform/unix/SysFile.cpp:520-590`, `DEFAULT_BUFFER_SIZE` 4096): held while they follow on and
fit, written in buffer-sized pieces as the buffer fills, written at once after the held bytes when
larger than the buffer, and flushed before anything reads, measures or positions the descriptor
(`OpenFile::flushed`; the `file` field is private, so every site chose). A terminal or `NOBUFFER` open
is unbuffered. `QUERY SIZE` on an open stream answers its descriptor after a flush
(`StreamInfo::getStreamSize`); `FLUSH` writes and answers `NOTREADY` `ERROR:n` on a failure
(`StreamInfo::streamFlush`); a drop writes what is held except during a panic. A program's own
`.Stream~new('STDOUT'/'STDERR')` holds its writes the same way (`StreamState::held`) until full,
`FLUSH`, `CLOSE`, a reopen or `UNINIT`, where `.STDOUT` and `.STDERR` are opened `NOBUFFER`
(`CoreClasses.orx` `LocalServer~init`). Witness `stream_writes_held.rex` (the item's probe,
redirection target included, by-name and object views, commands reading the file, size by name and by
object, NOBUFFER, FLUSH, a write past the buffer, a filling buffer, a positioned write, close by name,
and the program-made STDOUT/STDERR ordering against SAY and `.STDOUT`). Measured with stdout a pipe:
with a regular-file stdout the oracle's program-made STDOUT object seeks fd 1 to its own position
(`SysFile::write`'s `lseek64` on switching to write mode) and overwrites earlier output; this crate
does not copy that (see New oracle defects).

**2026-10-13-stdin-stream-object-queries.md (closed).** Rule: a stream nothing has opened has no
stream type (`QUERY STREAMTYPE` "UNKNOWN") and no handle, and an I/O operation opens it implicitly
(`StreamInfo::implicitOpen`), after which a standard stream is `READY` and its qualified name is its
name as written (`StreamInfo::openStd`). The crate answered from the descriptor before any open. Now a
program's own `STDIN`/`STDOUT`/`STDERR` (with or without the colon) is unopened until `CHARIN`,
`LINEIN`, `CHAROUT`, `LINEOUT`, `CHARS`, `LINES`, `ARRAYIN`, `SEEK`/`POSITION` or `OPEN`; until then and
after a `CLOSE` the stream type is `UNKNOWN` and the handle empty; once open `QUERY EXISTS` and
`QUALIFY` answer the name as written (`.STDIN`/`.STDOUT` included, which answered empty for `EXISTS`).
A closed standard stream's bare-name qualified path is resolved against the interpreter's directory,
not the process's. `HANDLE:n` streams are unchanged (Deviation 35). Witness
`stream_standard_open.rex` (reads and queries only). Residuals in Concerns 3.

**2026-10-18-command-waits-for-background-child.md (closed).** Rule: rexxref intro.xml, "Commands to
External Environments": "The environment then processes the command and returns control to the
language processor after setting a return code"; the command completes when its process ends. The
crate read the child's output pipes to end-of-file, which a background process the command started
keeps open. Now `collect` waits for the child on the calling thread; a pipe carrying output to this
interpreter's own standard output or error stops being read when the child ends: what it holds then
(by `FIONREAD`) is taken, and the rest is read and dropped on a detached thread, so the background
process's writes do not fail. A redirected output or error is still read to its end, as the oracle
reads it (measured: `'(sleep 1; echo late) &' with output stem s.` waits and captures `late` on both).
In `rexx` itself (`Sinks::own_descriptors`), an unredirected output or error is inherited by the child
after the hand-over, as the oracle's child inherits it, so a background process's later output reaches
the terminal as on the oracle (`probes/bgred.rex`: `late3` between the two lines on both). Witness
`command_background_child.rex` (the item's FIFO probe, an unwaited background process, both
redirected forms); at base the crate times out. `tests/signals.rs`'s thread census during a command
now expects one waiting pool thread (no pipe readers in `rexx`).

**2026-10-13-stdout-write-errors-swallowed.md: STOPPED, partly changed, not closed.** Derived rule:
a stream write the system refuses answers the residual count and `NOTREADY` with `ERROR:errno`
(rexxref funct.xml LINEOUT/CHAROUT). Changed: `Sinks` callbacks answer an `io::Result`; a refused
hand-over is kept per descriptor (`Interp::sink_errors`), and every stream write to `STDOUT`,
`.STDOUT`, `STDERR` or `.STDERR` (unbuffered) answers it from then on, the write whose own hand-over
fails included. Witness `tests/stdout_write_errors.rs`, spawning `rexx` with a closed pipe and with
`/dev/full`. Two departures remain; Deviation candidates below.

## Witnesses

New `corpus/lang` programs, in `phase-9.txt`, each with its `sourceline_oracle` file generated by the
test's driver from a copy outside the repository: `path_names_not_utf8`, `stream_writes_held`,
`stream_standard_open`, `command_background_child`. `witnesses/summary.tsv`, stdout piped, from fresh
directories: at base (`rexx-base-0fd10a074`, built from a `git archive`) they differ on stdout
(`path_names_not_utf8`, `stream_standard_open`), stdout and stderr (`stream_writes_held`), and all three
descriptors (`command_background_child`, timeout); at head (`rexx-head-b6cf1c525`) each is identical to
the oracle on all three. The corpus gate runs them in process, which exercises the pipe path of the
command change. I read each oracle stdout for its claimed paths: every lossy-site line answers the
real file (`directory: 1 FF`, `pwd: 1`, `newFile: from ff`); `redirected: 0 0`, `by name: 1 other
object: 0`, `size by name: 0 size of the object: 4`, `nobuffer: 1`, `flush: READY: 1`, `past the
buffer: 5000`, `buffer filled: 4096`, `out of order: 4`, `closed: 7`, `o1`/`o2` after `.stdout`, `at the
end too` after `last say`, `x1` after `x3`; the STDIN rows `UNKNOWN`, `[]`, `NOTREADY`, `[STDIN]`,
`[Stdin:]`; `v=[abc] READY`, `not waited 0 1`, `redirected: 2 out late 1`, `redirected error: 1 late 1`.

`tests/stdout_write_errors.rs`: `0 READY: | 1 ERROR:32 Broken pipe | 1 ERROR:32 Broken pipe | 0 READY:`
and the same with `ERROR:28 No space left on device`; the base binary answers `0 READY:` for every
write (`probes/stdoutclosed.rex` at base, and the oracle's answers in Deviation candidate A).

## Deviation candidates

**A. A buffered standard output reports a write error from its next hand-over (Deviation 40
sentence).** Oracle: `.STDOUT` is `NOBUFFER`, so every write reaches `write(2)` and the first `LINEOUT`
to a closed pipe or `/dev/full` answers `1 ERROR:32`/`ERROR:28` (`probes/stdouterr.rex`). Here
standard output to a pipe or file is buffered (Deviation 40), so a write before the first hand-over
answers `0 READY:` and the error is answered by every stream write after the hand-over that found it.
Proposed sentence: "A write error on a buffered standard output is answered by the stream writes
after the hand-over that met it." The alternative, a hand-over after every stream write to `.STDOUT`,
matches the oracle and changes Deviation 40's buffering of stream writes.

**B. A standard descriptor closed at exec is `/dev/null` here.** With stdout closed (`>&-`) the oracle
answers `1 ERROR:9 Bad file descriptor` to every write; Rust's runtime reopens a closed fd 0-2 on
`/dev/null` (read-write) before `main`, so here every write succeeds. Detecting it means reading fd 1
as "`/dev/null` opened read-write", which a shell's `1<>/dev/null` also gives. Proposed: "A standard
descriptor that is closed when `rexx` starts is open on `/dev/null`."

## Perf

`callgrind-final.txt` (`t8.6/`): `bench-programs/callgrind.sh -B 0.5 -j 4 -o
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.6/final2 base=.../rexx-base-0fd10a074
head=.../rexx-head-7596ced8d`, every program, 3 rounds, exit 1 on one OVER flag. Libc subtracted, every program is at most
+0.0341% (pingsem) except pingguard +0.0521%, a schedule-dependent program reported per the batches
header; interpret +0.0164%, fibfunc +0.0213%, rexxcps -0.0033%. Whole process, interpret is OVER at
+0.5054% and fibfunc +0.1399%, dispatchclass +0.1465%: malloc, since the `StreamState::held` field
alone moves interpret +0.75% whole process and 0.0000% libc subtracted (`perf-bisect/try-held.txt`).

The first full run, at `b6cf1c525` (`callgrind-b6cf1c525.txt`), was over on `interpret`: +0.1562% libc
subtracted, +0.5644% whole process. A dead-code control in `stream.rs` was flat; bisect builds from
`git archive` trees, each in a fresh target dir (tables in `perf-bisect/`, raw in
`/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.6/try-*`):
the path changes alone flat, head without the command or sink changes still +0.156%, the
`StreamState::held` field alone +0.75% whole process and 0.0000% libc subtracted (the whole-process
part is malloc). The libc-subtracted part was one instruction more in the loop of `fragment_plan`'s
slot translation, inlined into `run_fragment` (a pointer spilled to the stack, 128 iterations a pass;
`perf-bisect/run_fragment-loop-*.txt`), so `7596ced8d` keeps `fragment_plan` out of line
(`plan.rs`, outside the item's files): `interpret` +0.0164% libc subtracted in a one-round run
(`try-noinline`).

## Commands

All from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t86-target`; records in
`t8.6/`.

| command | result |
|---|---|
| `cargo fmt --all --check` (`checks-fmt.txt`) | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-core -p rexx-exec --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test signals --test dispatch_seam --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test stream_model --test stream_handle --test rexx_command --test prompt_before_read --test input_oracle --test stdin_contention --test no_process_state --test stdout_write_errors --test rexxutil_process` (`checks-touched.txt`, at `6fdcb7d7a`) | 101: `refusal_sites` (a line moved in `command.rs`, re-derived in `19e478b98`) and `signals`' thread census (updated in `19e478b98`); both 0 after (`refusal_sites` 5, `signals` 42). Lib 1200 passed. `checks-touched-1.txt`, at `36866b601`, has the lend-panic test failing, fixed by `b74e6a848` |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress` (`checks-collect-stress.txt`) | 0, 74 passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus-1.txt`, at `36866b601`) | 101: the one mismatch the known `rexxutil_file_date_time.rex` |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-parse --test sourceline_oracle --test errors` (`checks-parse.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test coverage` (`checks-coverage.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites` (`checks-unsafe.txt`) | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t8.6/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`suite/`, at `b74e6a848`) | 0; `tests.tsv`, `table.tsv`, `verdicts.tsv`, `oracle-differences.tsv` identical to `t8.5/r6/suite/` |
| WORKSPACE-PLACEHOLDER | |

## Concerns

1. **The stopped item** (stdout write errors) needs Moritz's ruling on candidates A and B; its file
   stays open.
2. **`plan.rs`** is outside the item's files: an `#[inline(never)]` to keep the budget (Perf).
3. **Standard-stream residuals, not queued.** An open program-made `STDIN` answers `QUERY SIZE`,
   `QUERY POSITION`/`SEEK` and `QUERY DATETIME` with the null string where the oracle answers from
   descriptor 0 (`probes/stdpos.rex`, `probes/stdinpos.rex`: size 18, position 7 after a line, on a
   file). The oracle's positions are per object (each standard stream object buffers descriptor 0 on
   its own, so a `.Stream~new('STDIN')` read leaves `.STDIN` at end of file), where this crate's
   readers share one position. Proposed owner: a queued item for the streams area.
4. **`CHARS`/`LINES` on a `/dev/null` standard input** answer 1 on the oracle from an uninitialized
   `int` after a failed `FIONREAD` (`SysFile::hasData`, `SysFile.cpp:1310-1315`); here 0. Unchanged.
5. **In-process runs drop a background process's late output.** Without `Sinks::own_descriptors` (the
   corpus harness, an embedding) output a command's background process writes after the command ends
   is read and dropped; the oracle's child writes it to the inherited descriptor.
6. **Held writes and a crash.** Held stream data is lost if the process dies (as on the oracle) and
   is not written during a panic's unwinding.

## New oracle defects

* A program-made `STDOUT`/`STDERR` stream on a regular-file descriptor overwrites earlier output:
  `SysFile::write` seeks the shared descriptor to the object's own position when it switches to
  writing (`SysFile.cpp:533-543`); `probes/stdbuf.rex` with stdout a file loses `nb1`, `nb2` and
  `after colon`.
* `SysFile::hasData` reads an uninitialized `bytesWaiting` when `FIONREAD` fails (Concern 4).
* `.Stream~new('HANDLE:1')~lineout` with stdout closed or redirected to a file segfaults (rc 139);
  this is entry 12a of `oracle-crashes.txt`, not new.
