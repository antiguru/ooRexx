# 5c follow-up Task 2 — the constructor carries state, and the witness proves it

You are the implementer for **Task 2** of `docs/superpowers/plans/2026-09-04-phase-5c-followup.md`.
Read the plan (its head, Global constraints, Task 2, The gate) and the spec
`docs/superpowers/specs/2026-09-04-phase-5e-mutablebuffer.md` first. Then read
`docs/superpowers/records/2026-09-04-phase-5c-followup/task-1-report.md` — **its section 4 is the
representation you build, and 4.1/4.2 are the API and the rendering rules**. Do not re-derive them.

**BASE is the commit this brief lands in** (`git log -1` when you start). You work in the worktree
`/home/moritz/dev/repos/ooRexx-rust-rewrite`, in `rust/`. The tree is yours until you hand it back
under the protocol at the end. `rust/CLAUDE.md` governs everything; read its Gates and Method
sections.

## What you build

### The representation (decided by Task 1, measured)

`rexx-core`: `Body::Instance` gains one field, `native: Option<Box<BufferState>>`, with
`pub struct BufferState { bytes: Vec<u8>, capacity: usize, default_size: usize }` exported from
`rexx-core`. **`size_of::<Body>()` stays 80** — Task 1 measured it (`task-1-report.md` §2.5); the
assertion at `rexx-core/src/body.rs:602` is your check that it did. `Body::trace` needs nothing new:
the state holds no `ObjRef`. Every `Body::Instance { .. }` construction gains `native: None`; the
compiler enumerates the sites, not a grep.

`rexx-exec`: `Interp::buffer(&self, ObjRef) -> Option<&BufferState>` and
`buffer_mut(&mut self, ObjRef) -> Option<&mut BufferState>` beside `array_slots`/`array_body`
(`value.rs:737`, `:757`). A `None` is the refusal path (`Loud::native_method`), never a panic.

**Rendering, exactly as §4.2 says**: `redirect_of` gets `Body::Instance { native: Some(_), .. } =>
Redirect::None` **ahead of** the `name` arm (a named buffer still renders as its contents — oracle:
after `buf~objectName = 'named'`, `say buf~string` is `abc` and only `~objectName` is `named`);
`to_text`, `try_text` and `text_len_inner` each get an arm answering `state.bytes`. This fixes the
infallible renderings — `trace i`'s `>V> BUF => "abc"` and error-message substitutions — and **does
not** make `say buf` answer: the value protocol sends `MAKESTRING`, which stays unbound in this
task. Measured by Task 1 on both prototypes: `say buf` and `'x' || buf` are rc 120 `MAKESTRING`-loud
with the `to_text` arms present. Confirm that on your build; it is the property that keeps a
half-built buffer from ever answering wrongly.

### The constructor

`native_mutable_buffer_new` (`dispatch.rs:7546`) keeps what it is given: `default_size` is the
second argument or 256; `capacity` is `max(default_size, initial.len())`
(`MutableBufferClass.cpp:100`-`:127`); `bytes` is the initial string, `try_reserve_exact` up to
`capacity`. Its doc comment currently says the contents are not kept — **that sentence becomes false
and you correct it**; so does the `MutableBuffer` row of
`a_constructor_taking_arguments_answers_an_instance_and_refuses_its_state` (`dispatch.rs:9331`),
which becomes the assertion that `o~length` answers `3`. Keep that test's
`say .MutableBuffer~new('abc')` rc 120 assertion: it is still true and still load-bearing until
Task 3's conversion commit.

### The methods, as `NATIVE_METHODS` rows beside `("String", "LENGTH", …)` (`dispatch.rs:550`)

`LENGTH`, `STRING`, `ENDSWITH`, `APPEND`, `DELSTR`, `GETBUFFERSIZE`, `SETBUFFERSIZE`. Argument
conversion uses the **method-layer** family — `whole_method_argument`, `refuse_method_argument`,
`optional_length_argument`, `required_string_argument` (`dispatch.rs:7946`, `:7966`, `:7493`,
`:3904`) — which raises 93.9xx at rc 163, **not** the builtin layer's 40.x at rc 216. Task 1
measured the difference (`task-1-report.md` §1.1). **Measure every refusal message on the oracle
before writing it**, three descriptors, from a fresh empty directory.

Semantics, all measured on the oracle (Task 0 and Task 1) and in the C++:

* `append(x, ...)`: `A_COUNT`; each argument a required string; grows by `ensureCapacity`'s
  `max(needed, 2 * capacity)` (`:236`-`:248`); answers the receiver.
* `setBufferSize(n)`: `n == 0` → length 0 and, if `capacity > default_size`, capacity back to
  `default_size` (`:680`-`:695`); `0 < n < length` → **truncates** contents to `n`, capacity `n`;
  `n ≥ length` → capacity `n`. Answers the receiver. Oracle: `new('abcdef',100)` then
  `setBufferSize(3)` → `len 3 cap 3 string abc`; `new(copies('x',400))` then `setBufferSize(0)` →
  `0 256`; `new('abc',500)` grown to `1203 2000` then `setBufferSize(0)` → `0 500`.
* `getBufferSize()`: `capacity`. `length()`: `bytes.len()`. `string()`: a fresh `Text` of the
  bytes (`RexxObject::makeStringRexx` → `stringValue`, `:740`). `endsWith(s)`: a suffix compare
  (`primitiveMatch` shape, `:1615`); one required string argument.
* `delstr(n, [len])`: `mydelete` (`:400`-ish; read it). **Extract the byte core from
  `builtin/string.rs::delstr` (`:479`) into a plain `fn(&[u8], …) -> …` (or over `&mut Vec<u8>`)
  that the builtin and the method both call.** This is the first instance of the extraction Task 3
  repeats for every shared core (`task-1-report.md` §1.2, §5); the builtin's existing tests are the
  control that the extraction changed nothing. Keep it to `delstr` — no other core moves in this task.

Method bodies follow §4.1's discipline: **convert every argument first**, copying argument bytes
into `take_result_buffer()` where needed, **then** take `buffer_mut(receiver)` and do the byte work;
while the `&mut BufferState` is live no `interp` call can allocate. Mutators answer the receiver;
readers answer `interp.text(...)`/`text_built` or `counted`.

### The witness: `rust/corpus/lang/mutablebuffer_state.rex`

Nine lines the oracle prints, rc 0, stderr empty — **write the program against these and confirm on
the oracle before touching the crate**:

```text
6
abcdef
1
9
abcdefghi
abcdefgh
999
400
8 500
```

They are, in order: `length` of `new('abcdef')`; its `string`; `endsWith('ef')`; `length` after
`append('ghi')`; `string`; `string` after `delstr(9)`; `getBufferSize` of `new('x', 999)`;
`getBufferSize` of `new(copies('x',400))`; and `length getBufferSize` after `setBufferSize(500)`
on a buffer of eight. **Add the truncation case** (`new('abcdef',100)` then `setBufferSize(3)`,
printing `length getBufferSize string`) and the `setBufferSize(0)` shrink case, each confirmed on
the oracle first, and record the final expected text in your report. Use `buf` as the variable —
**never `b`**: a symbol `b` followed by a quoted string parses as a binary literal (error 15.4).
**No `say buf`, no concatenation of the buffer, no `makeString`** — those stay loud until Task 3.

Against BASE the program must fail (it does: rc 120 at the first `~length`); against your build it
agrees with the oracle byte for byte on all three descriptors on **both engines**,
`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` (that exact spelling; `tree` alone is rc 2). Add the
line `lang/mutablebuffer_state.rex<TAB>Task 4 files it into phase-5c.txt` to `rust/corpus/unfiled.txt`
— `corpus.rs`'s `every_lang_program_is_run_or_named_unfiled` reddens otherwise.

**Also add a witness for the two silent divergences Task 1 found**, in the same program or a second
one you name: `trace i` around `x = buf` must show `>V> BUF => "abcdef"` (A's shape printed
`"a MutableBuffer"`), and a `::class sub subclass MutableBuffer` whose `init` does `expose n; n =
42` must answer `~n` and `~length` (B-as-built refused `EXPOSE`). Both are oracle rc 0 today.

### `corpus/method-bodies.txt`

Your methods move rows. **Predict first, in your report**: which of `MutableBuffer`'s 51 instance
rows move `loud` → `answers` under the receiver `.MutableBuffer~new('abc')` with a zero-argument
send, and what evidence each carries — `length` → `rc 0`, `string` → `rc 0`, `getBufferSize` →
`rc 0`, and `endsWith`/`delstr`/`setBufferSize`/`append` → whatever the oracle's zero-argument
refusal is (measure it: likely `rc 163` `93.901`, but `append` is `A_COUNT` and may not refuse).
Then `REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies`, read
`git diff -- corpus/method-bodies.txt`, and compare row by row. **The gate is: no row moves to
`diverge`.** A row that moves that you did not predict is a finding, and goes in the report.

## Controls, each predicted before it is run

* The witness against BASE's binary (a pristine build in a `git archive` extract under
  `/home/moritz/dev/repos/claude-build-scratch/5c-followup-task2/`, own `CARGO_TARGET_DIR`): rc 120,
  stderr names `LENGTH`. This is the negative control that says the program tests the change.
* Delete the `native` write in the constructor (keep the field): the witness must fail at line 1
  (`length` 0 against 6) — and **run the full suite without your new assertions first**, to see
  what else catches it ("can fail" is not "adds coverage").
* `a_constructor_taking_arguments_answers_an_instance_and_refuses_its_state` red before your edit
  to it, green after — read both.
* `size_of::<Body>()`: the assertion at `body.rs:602` compiles. If it does not, stop and report;
  do not raise the bound.

## Rules that bite

* **No `unsafe`.** Workspace lint is `deny`; a site is Moritz's decision. Stop and say so.
* **Never `git checkout -- <path>`** on a file you edited; copy to your scratch dir and restore from
  the copy, then `touch` it. **No `rm` with a glob or a computed path; do not delete anything you
  did not create this task.** Never `git add -A`; never amend; `git commit -F <file>` naming paths;
  `Cargo.lock` is not staged unless you changed a dependency (you should not).
* Oracle runs from a fresh empty directory, wrapped
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
  Three descriptors, never `2>&1`, `$?` captured immediately. Never run anything in
  `rust/corpus/oracle-crashes.txt`. Never `NUMERIC DIGITS` above 1000.
* **Comments**: one sentence of overview; params/returns/panics; properties not visible in the
  code. No history, no task numbers, no set sizes. Correct the constructor doc that becomes false;
  do not hedge it.
* `cargo fmt --all --check` (not `--edition`). `grep` is `ugrep -I`; use `/bin/grep -a`.
* Scratch under `/tmp/claude-1000/.../scratchpad/task2/` (small files) and
  `/home/moritz/dev/repos/claude-build-scratch/5c-followup-task2/` (builds).

## Report and hand-back — the commit-before-gating protocol, no exceptions

Report: `docs/superpowers/records/2026-09-04-phase-5c-followup/task-2-report.md`. Write its
skeleton first; fill each section from output you are looking at; every claim **measured** (with
the command) or **inferred**. Include: the witness's final text and the oracle's three descriptors;
the predicted and measured `method-bodies.txt` row moves; each control's prediction and reading;
the constructor-doc and test corrections; every file you created outside the tree.

Then: **fast checks yourself** — `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
-- -D warnings`, `cargo test --release -p rexx-exec --test corpus --test method_bodies --test
coverage` plus the `dispatch` unit tests (`cargo test --release -p rexx-exec --lib`), assert
non-zero run counts. **Commit code and report together**, the report's Gates table carrying
`**G1**`–`**G7**` placeholders (copy the table from `task-0-report.md`). **Then** start the seven
gates in the background from `rust/`, each status written unpiped to
`<scratch>/task2/gates/status.txt` as it completes, the commit sha as that file's first line, a
pidfile beside it, `finished` as its last line. Then your final message is exactly: `committed at
<sha>, gates running, statuses at <path>` — and **stop**. Do not edit anything after the commit;
the tree belongs to the gate run until the controller reads `finished`.
