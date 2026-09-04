# 5c follow-up Task 1 — where the bytes live, decided by measurement

You are the implementer for **Task 1** of `docs/superpowers/plans/2026-09-04-phase-5c-followup.md`.
Read the plan and the spec (`docs/superpowers/specs/2026-09-04-phase-5e-mutablebuffer.md`) first,
from the extract described below. **BASE is `fdf4c6624`.** This task answers the spec's D80 and
**builds nothing that ships**: prototypes, measurements, and a report with a recommendation.

## Where you work, and where you must not

* **The worktree at `/home/moritz/dev/repos/ooRexx-rust-rewrite` belongs to a running gate suite.
  Do not edit, build in, or run cargo in it.** Reading a file there is allowed; nothing else.
* Your tree is a `git archive` extract at BASE:
  ```
  B=/home/moritz/dev/repos/claude-build-scratch/5c-followup-task1
  mkdir -p $B/tree && git -C /home/moritz/dev/repos/ooRexx-rust-rewrite archive fdf4c6624 | tar -x -C $B/tree
  export CARGO_TARGET_DIR=$B/target
  ```
  Build and run from `$B/tree/rust`. Your prototypes, scripts, and report all live under `$B`.
  **They are kept until the phase closes; do not delete anything, including your own files.**
* Scratch text (probe programs, logs) goes under `$B/scratch/`. `/tmp` is a RAM disk; keep large
  build output under `$B` on real disk.
* **Never delete anything.** No `rm` of any kind. Name what you created in the report and the
  controller sweeps it.
* **Do not commit anything anywhere.** The extract is not a git repository.

## What to find out, in order

### 1. Where the byte machinery already is

Task 0 measured that `String`'s *method* surface in this crate is
`NATIVE_METHODS`' `LENGTH REVERSE SIGN UPPER NEW` and `'abc'~substr(2)` is rc 120 on both engines.
The implementations of `substr`, `pos`, `words`, `delstr`, `overlay`, `insert`, `changestr`,
`countstr`, `lastpos`, `verify`, `translate`, `space`, `subword`, `wordpos`, ... are the builtin
**functions** in `rust/crates/rexx-exec/src/builtin/string.rs` and `word.rs`, with
`(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure>` signatures.

Read them and answer: **do they factor over a `&[u8]` a buffer can hand them?** That is, is there
(or is there cheaply) a layer that takes the subject bytes and the parsed arguments and returns bytes,
so a `MutableBuffer` method can call it with the buffer's contents and write the result back? Or is
the subject read from an `ObjRef` inside each function so that a buffer would need to materialise a
`Text` object per call? Name the functions you read and which shape each has. If they do factor,
say so plainly: the phase gets smaller.

### 2. The representation, two candidates built far enough to measure

The spec's D80 offers three shapes and the width assertion `const _: () =
assert!(size_of::<Body>() <= 80);` at `rust/crates/rexx-core/src/body.rs:602` constrains all of
them. Read that module's doc around the assertion first. Also read `Body::Instance` (`:192`),
`Body::Native(Box<NativeObject>)` (`:208`, the boxed precedent), `NativeObject` (`:506`) and
`Bytes` (`rust/crates/rexx-core/src/bytes.rs`, `INLINE_BYTES = 54`).

The two live candidates:

* **(A) object variables** in `Body::Instance`'s `pools` — the buffer's text as one variable,
  capacity and default size as two more; no `rexx-core` change; every mutation reallocates a `Text`.
* **(B) a new boxed `Body` variant**, `Body::Buffer(Box<...>)` or similar on `Body::Native`'s
  precedent, carrying `Vec<u8>` (or `Bytes`) plus `capacity: usize` plus `default_size: usize`.

The third shape (`NativeObject::entries`) is the wrong shape for bytes; say so in one sentence and
do not build it.

**What the representation must carry, measured on the oracle by Task 0 and in the plan's Task 1
text**: the bytes; the length (which is the bytes' length); the **capacity** (`bufferLength`, what
`getBufferSize` reports); and **`defaultSize`**, which `setBufferSize(0)` shrinks capacity back to
(`MutableBufferClass.cpp:686`-`:691`). Growth on append is `max(needed, 2 * capacity)`
(`ensureCapacity`, `:243`). Capacity defaults to 256, the constructor's second argument sets it,
and an initial string longer than it raises it. **Do not derive capacity from `Vec::capacity()`** —
it is the oracle's own number and it is observable; carry it as a field.

Build each candidate only far enough that `.MutableBuffer~new('abc')` keeps its argument and
`~append(x)` and `~length` work — nothing else — in the extract. Two extracts or one extract with
two patches, your choice, but **each candidate's binary is a separate build with its own sha256
recorded**, and the patch for each is saved under `$B/` as a file the report names.

### 3. Measure, do not choose on taste

Two programs, run on both candidates, on both engines (`REXX_ENGINE=ir` and
`REXX_ENGINE=tree-walker` — that exact spelling; `tree` alone is rc 2 and reads like a fast
success):

* an append loop: `b = .MutableBuffer~new('')` then `do 100000; b~append('x'); end; say b~length`
  — **name the variable something other than `b`**: a symbol `b` followed by a quoted string parses
  as a binary literal; use `buf`.
* a length loop: a buffer of a few hundred bytes read with `~length` 100000 times.

Instrument: `perf stat -e instructions:u -r 5` if it runs here; if perf is unavailable or reports
`<not supported>`, `valgrind --tool=callgrind` and read `Ir`. **Say which instrument printed each
figure.** The sandbox may allow one PMU counter at a time; if a run reports a counter shortage,
retry once before believing it. Record the sha256 of each binary beside its figures. **Write your
prediction of the ratio before running** and mark it confirmed or falsified.

Also record what each candidate does to `size_of::<Body>()` — a `const` assertion trip is a
compile error, and that is data.

### 4. Recommend

One paragraph: which shape, why, with the figures. Plus the API the methods will need — how a
method body gets `&mut` access to the buffer through `Interp` (find how `native_length` reaches a
`Text`'s bytes via `interp.text_len` in `rust/crates/rexx-exec/src/value.rs:559` and what the
mutable analogue would be). Plus how `makeString`/rendering would produce a `Text` from the buffer.

### 5. Optional, if you have budget: the plan's review question 1

Should Task 3 be one task or several? Group `MutableBuffer`'s 51 instance methods into families
(caseless, word, search, mutators, capacity, conversion), say which builtin function each maps to
(from step 1), and say which families share enough to be one commit.

## Things that have gone wrong here before, each once at least

* The controller's own test binary was stale after a `cp -a` restore, and an experiment was run
  against the wrong code. **After any patch, confirm the binary's mtime is after the edit**, or
  grep the binary for a string only the new code carries.
* `cargo test <name>` exits 0 when it matches nothing. Assert the run count.
* An oracle probe run from a directory with leftover `.rex` files calls them as external routines.
  Run oracle probes from a fresh empty directory, wrapped as
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
  Never run anything listed in `rust/corpus/oracle-crashes.txt`. Never set `NUMERIC DIGITS`
  above 1000.
* Read stdout, stderr and exit status as three separate descriptors; never `2>&1`; capture `$?`
  immediately.
* No `unsafe`: the workspace lint is `deny` and a site is Moritz's decision. If a candidate seems
  to need it, stop and say so.
* `grep` here is a `ugrep` wrapper with `-I`; use `/bin/grep -a` for anything binary and `-F` for
  data patterns.
* A comment may not name the size of a set; no "the six functions". The report may carry
  measurements freely.

## Report

Write `$B/task-1-report.md` **first**, as a skeleton with every section named, then fill each
section as you measure it. Mark every claim **measured** (with the command) or **inferred**. End
with: the list of every file and directory you created under `$B`, and one line per open question
you could not settle. Your final message to the controller is the path of that report and its
recommendation in two sentences — the controller reads the file, not the message.
