# Phase 9c final whole-branch review

Range `fe4867787..bf5224d2b`, reviewed at `bf5224d2b` in a detached worktree.
Binaries were built with `cargo build --release -p rexx-exec --bin rexx --bin rexxc` into a private target dir.
Every oracle run used the standard wrapper (`ulimit -v 1048576`, `LD_LIBRARY_PATH`, `timeout -k 5 20`), from fresh directories.
No truncated `.rxo` was fed to the oracle.

## Verdict

Approve with fixes.
The two binaries agree with the oracle on every parse-error row except the licensed ones, under `rexx`, `rexxc` and `INTERPRET`, from source and from a restored image.
Two Important 9c defects remain on the restore surface: method entry and exit trace lines, and `-o` default options.
Neither is a crash or a gate failure, and both are narrow.

## Defect class hunted

The phase's own defects share one shape: an observable that a task pinned on one route and not on its neighbours.
Examples are the byte and string encodings on paths, and the secondary message checked while the rest of stderr went unchecked.
So the hunt ran each observable across the routes D5 and D6 name, comparing complete stdout, stderr and status rather than the fields a test reads.

## Sweeps run

* **`rexxc FILE -s` on every `parse-errors.tsv` row (1109 rows), full stdout, stderr and rc against oracle `rexxc`.**
  7 differ: the 5 licensed Deviation 38 and 39 rows, and the 2 exempt `REGISTERED` install rows.
  The agreement test `tests/rexxc.rs` checks only the secondary line, `" line N:  "` and the status, so this sweep also covers the echo line and the main message text it does not read.
* **`rexx FILE` on every non-`ok` row (664 rows).**
  The same 7 differ.
* **Each translation row without a directive (386 rows), run through `INTERPRET` under `SIGNAL ON SYNTAX` and then untrapped, as source and as each side's own `rexxc` image.**
  1 row differs in both modes. That row is finding 3, which predates 9c.
  The line inserts, `condition('o')~position`, `SIGL` and the `Program in "..." (no source available)` echo all agree.

## Targeted probes (all against the oracle)

* `#!` line plus a parse error on line 3: `rexx`, `rexxc -s` and `rexxc` with an output file are identical, and no output file is left behind.
* `::REQUIRES` of a file with a parse error, from source and from a `.rxo`: identical, `Program in "m.rxo" (no source available)` included.
* A runtime error after `SAY`s, from source and from a `.rxo`, with stdout and stderr on one pipe or file: the stream contents match. The interleave differs, and Deviation 40 licenses that.
* `SOURCELINE()`, `SOURCELINE(1)` and `PARSE SOURCE` from a binary and an encoded image: identical (40.34, `(0)`).
* `rexxc` given a missing name, a name found by its extension, a subdirectory name, output `./p.rex` for input `p.rex` (both overwrite), an empty program, `-e -x`, an unreadable input, a read-only output, a directory as output, and a missing output directory: identical.
* `rexx` given a directory named `sub.rex`, `sub`, `sub/nope.rex`, and a `.rxo` called by relative path from another directory: identical.
* `rexx -e` with a parse error, with `.SYSCARGS` and `.local~syscargs`, with `exit 70000` and with `call nope`, and `rexx -x -y p.rex`, `rexx -x p.rxo "a b" c`: identical apart from the Deviation 40 interleave.
* `trace i` with routine calls in a compiled program: identical.
* `.Package~defaultOptions` before `.Routine~newFile` on a `.rxo`, a `.rex` and `CALL "r.rxo"`: identical.
* The crate alone, run on its own image cut at 0, 1, 20, 21, 36, 68, 70 and 79 bytes, on a cut encoded image, and through `.Routine~new` with tag-shaped arrays: no panic. Each run answers 3.903, 13.1, 35.1 or 93.952.

## Findings

### Important

1. **A restored program drops `>I>` and `<I<` method entry and exit trace lines.**
   `rust/crates/rexx-exec/src/environment/route.rs:158` truncates every trace line while `untraceable_code()` holds, `route_trace_line_as(.., entry)` included.
   The oracle suppresses a compiled program's clause and result lines but still writes the method entry and exit lines.
   Measured with `t.rex`, a class method that runs `trace i` and then `raise syntax 40.1`, compiled by each side's `rexxc` and run as `rexx t.rxo`.
   The oracle writes `>I> Method "M" with scope "C" in package "DIR/t.rxo".` and the matching `<I<`, and the crate writes neither.
   `tests/compiled/programs/observe.rex` traces only the main program and a routine, so no witness reaches a method.
   D5 says trace lines are measured rather than assumed. 9c defect (Task 9).
2. **`rexx -o` and `-od` options reach a restored top-level program, and the oracle's do not.**
   `rust/crates/rexx-exec/src/lib.rs:3927` sends `defaultOptions` before translating, whatever the source is, and a restored image is re-translated from its source.
   The oracle restores the image without translating it, so the options never apply.
   Measured: `rexx -o "digits 5" p.rxo` answers `DIGITS()` 9 on the oracle and 5 on the crate, the same for the encoded `p.rxe`.
   `rexx -od "digits 7 form engineering" d.rxo`, where `d.rex` has `::options digits 12`, answers `12 SCIENTIFIC` on the oracle and `12 ENGINEERING` on the crate.
   The `.rex` runs agree. 9c defect (Tasks 5 and 9 meet here: `-o` is Task 5's, the restore is Task 9's).

### Minor

3. **When an earlier clause has a parse error and a later one holds an invalid character, the crate reports the invalid character.**
   `rust/crates/rexx-parse/src/scanner.rs:961` (`invalid_character`) is reached by a scan that runs ahead of the parse. The oracle scans clause by clause and reports the first clause's error.
   Program `say 1` / `say (` / ``say ` ``: the oracle answers 35.1 on line 2 and the crate 13.1 on line 3.
   The same holds in `INTERPRET "trace -;" || "0a"x` (oracle 35.1, crate 13.1 on the line feed), and for `say (;`, `x = ;` and `say 1;` followed by `'0a'x` in an `INTERPRET` string.
   The pre-9c binary `ooRexx-p9-pin/rexx-run-9c-t2-cdbbc1568` makes the same selection, so it is pre-existing. A search of `.superpowers/sdd/queued/` and `rust/corpus/` found no record of it.
   It is outside D6, which covers inserts and not which error is chosen, and it is worth queueing.
4. **`Msg<N>`'s arity guarantee can be bypassed by assignment.**
   `rust/crates/rexx-parse/src/token.rs:39-42` keeps `code` and `sub` `pub` on `ParseErrorFields`, and `:63` gives `ParseError` a public `DerefMut`.
   A caller in any crate can raise with one message and then rewrite `code` and `sub` to a pair whose insert count differs.
   No caller does today (`grep -rn '\.code = \|\.sub = '` over `rexx-parse` and `rexx-exec/src` finds none on a `ParseError`).
   The spec's condition for the reversal (D6, "cannot be called without the inserts its message needs") is met by construction only. 9c (Task 2's boxing).
5. **`rust/corpus/README.md:347` lists `parse-errors.tsv`'s fields as major, sub-number and line.**
   Task 1 added the secondary message column, and `tests/rexxc.rs` now reads the file besides `rexx-parse/tests/errors.rs`. The sentence is incomplete, not false. 9c (Task 1).
6. **`rust/crates/rexx-exec/tests/rexxutil_process.rs:24` still names its helper `rexx_run`**, though it spawns `CARGO_BIN_EXE_rexx`. This is a naming leftover from Task 6.
   The other `rexx_run` hit, `src/internal_routines.rs:81`, is the REXX-package row constructor and unrelated. 9c (Task 6), cosmetic.

## Not findings

* The `git grep` for `rexx-run` outside `PINNED.md` finds only the two identifiers above. `PENDING` survives only as `signal.rs`'s unrelated static. No "not yet" sentence near compiled or `rexxc` code is false.
* `ParseError::new` is gone. Every constructor goes through `ParseError::raise::<N>` (`token.rs:74`).
* Stream buffering and `rexxc`: `rexxc` flushes its banner before translating, and a banner-plus-error run redirected to one file is byte-identical to the oracle's.
