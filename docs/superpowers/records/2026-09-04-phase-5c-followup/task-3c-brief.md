# 5c follow-up Task 3c — the mutators, over the cores

You are the implementer for the **third Task 3 commit** of
`docs/superpowers/plans/2026-09-04-phase-5c-followup.md`: the mutators. Read the plan's head, Global
constraints, Task 2 and Task 3; then, in `docs/superpowers/records/2026-09-04-phase-5c-followup/`:
`task-1-report.md` §4.1 (the method-body discipline), §4.2 and §5 (the family table);
`task-2-report.md` §1 and §2.3 (how the first seven methods are built, and how each refusal was
measured first); `task-3a-report.md` §1.1 and §2 (the cores and their signatures: a byte `start` is
0-based, a word `position` 1-based, an omitted length is `Option<usize>`); `task-3b-report.md` §1
(the argument helpers already in `dispatch.rs`), §2.3 (the measured refusal table you extend) and
§7 (argument conversion order is part of the answer). `rust/CLAUDE.md` governs; read its Gates and
Method sections. **BASE is `git log -1` when you start.** You work in the worktree
`/home/moritz/dev/repos/ooRexx-rust-rewrite`, in `rust/`; it is yours until the hand-back.

## What you bind

`NATIVE_METHODS` rows for `MutableBuffer`, in the alphabetical block where Tasks 2 and 3b put the
existing rows, each at the count `interpreter/memory/Setup.cpp:1416`-`:1479` declares:

| name (count) | C++ (`classes/MutableBufferClass.cpp`) | core | answers |
|---|---|---|---|
| `INSERT` (4) | `insert` | `string::insert_bytes` | the receiver |
| `OVERLAY` (4) | `overlay` | `string::overlay_bytes` | the receiver |
| `REPLACEAT` (4) | `replaceAt` | `overlay_bytes`'s neighbourhood, read the C++ | the receiver |
| `[]=` (3) | `bracketsEqual` | as `REPLACEAT` | the receiver |
| `CHANGESTR` (3) | `changeStr` | `string::changestr_bytes` | the receiver |
| `UPPER` (2) | `upper` | `string::case_shift_bytes` | the receiver |
| `LOWER` (2) | `lower` | `case_shift_bytes` | the receiver |
| `TRANSLATE` (5) | `translate` | `string::translate_bytes` | the receiver |
| `SPACE` (2) | `space` | `string::space_bytes` | the receiver |
| `DELWORD` (2) | `delWord` | `word::delword_bytes` | the receiver |
| `DELETE` (2) | `mydelete` | `delete_range`, as `DELSTR` | the receiver |

`DELETE` and `DELSTR` are **one C++ method under two names** (`MutableBuffer::mydelete`); `DELSTR`
is already bound from Task 2, so `DELETE` is a second row over the same body, not a second body.
**Not this commit**: the caseless family (`CaselessPos`, `CaselessChangeStr`, `CaselessMatch` and
the rest) and the conversions (`makeString string makeArray subWords`). `MAKESTRING` stays unbound,
so `say buf` stays loud and the witness must not contain it.

**What answers what.** Every method here mutates in place and, on the oracle, **answers the
receiver itself** rather than a fresh buffer or a string: measure that per name, as identity
(`buf~insert('x') == buf`, and a chained send) rather than by printing. Answering the receiver needs
no new machinery -- `Ok(Some(receiver))` is what `native_mutable_buffer_delstr` and `_append`
already do -- so the work is the measurement, not the plumbing.

**Read the C++ for each before writing it**, and **measure every refusal on the oracle first** (a
two-line program per case, fresh directory, three descriptors), as Task 3b's §2.3 table does. The
helpers are already there: `required_position_argument`, `optional_position_argument`,
`required_length_argument`, `optional_length_argument`, `required_string_argument`,
`string_method_argument`, `pad_method_argument` (93.922 `Raised::incorrect_pad`),
`option_method_argument` (93.915). Add a helper only where the C++ refuses in a shape none of them
covers, and say which C++ call it mirrors.

**Growth is the new hazard.** These methods can extend the contents past the current capacity, which
`BufferState::ensure_capacity` handles and Task 2's `mutablebuffer_state.rex` is the only program
that witnesses. Every method here that can grow the buffer needs a case in your witness that does
grow it, past `getBufferSize` and past the 256-byte default, and the report must say which methods
can grow it and which cannot.

## Method bodies

Task 1 §4.1's discipline: **convert every argument first** (copying argument bytes into
`take_result_buffer()` where a `&[u8]` is needed alongside the state), **then**
`buffer_state_mut(interp, receiver, b"NAME")?` and the core over `&mut state.bytes`. A `None` from
`buffer_state_mut` is the `Loud::native_method` refusal, never a panic. Task 3b §7 is live here:
where the C++ returns or refuses before converting a later argument, the body must do the same, so
read each method's conversion order rather than assuming the argument list's.

## The witness: `rust/corpus/lang/mutablebuffer_mutators.rex`

**Written and confirmed on the oracle before you touch the crate** — rc 0, stderr empty, both
engines byte-identical after. Every name above sent at least once with a positive and a negative
case; each growth case; the receiver-identity check; a chained send (`buf~upper~space`); the
contents read back after each mutation with a reader Task 3b bound (`buf~string` is **not**
available, `buf~substr(1, buf~length)` is). `buf`, never `b`. **No `say buf`, no concatenation of a
buffer, no `makeString`.**

**File it in this commit**: `corpus/phase-5c.txt` and `coverage.rs`'s `EXPECTED_SUBSET_5C`, **not**
`corpus/unfiled.txt`; plus the `rexx-parse/tests/sourceline_oracle/mutablebuffer_mutators.txt`
companion, generated with the driver in that test's module comment. The pin's inversion — the line
absent from `EXPECTED_SUBSET_5C` — must redden `phase_5c_subset_matches_the_committed_list`.

## `corpus/method-bodies.txt`

**Predict first, in your report**: which `MutableBuffer` instance rows move under the receiver
`.MutableBuffer~new('abc')` with a zero-argument send, and the evidence each carries — measure the
oracle's refusal per name (`rc 163` `93.903`, `rc 168` `88.901`, or `rc 0` for one that takes no
required argument). Then `REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test
method_bodies`, read the diff row by row against the prediction. **The gate: no row moves to
`diverge`.** Note that a mutator answering the receiver makes the probe's own `say` reach
`MAKESTRING`, which is still loud: predict what that does to each row before you run it.

## Controls, each predicted before it runs

* The witness against BASE's binary (a pristine `git archive <BASE> rust interpreter` extract under
  `/home/moritz/dev/repos/claude-build-scratch/5c-followup-task3c/`, own `CARGO_TARGET_DIR`; the
  archive needs `interpreter/` as well as `rust/`, two build scripts read it): rc 120 at the first
  mutator, stderr naming it.
* **The corpus control that can go red is `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec
  --test corpus`**, read together with its `N of M matching` line. The plain `--test corpus` binary
  is report mode and exits 0 on a divergence.
* One mutation per family at least — an `insert` position, an `overlay` pad, a `changeStr` count, a
  `space` run, a `delWord` bound, and one that defeats `ensure_capacity`'s growth — each predicted
  (which test or program catches it, and by what reading), run with `--profile mutation`
  (`target/mutation/`; the release binary is untouched and its sha256 must not change), restored
  from a copy, `touch`ed, sha confirmed. Task 3b's M4 is the precedent for a mutation that goes red
  by a panic rather than a mismatch: predict which, and say so when the reading differs.
* Run the suite *without* the new witness where a catcher is claimed for it ("can fail" is not
  "adds coverage").
* `cargo test <name>` matches nothing at exit 0 — assert run counts.

## Rules that bite

No `unsafe`. Never `git checkout -- <path>` on an edited file. No `rm` with a glob or computed path;
delete nothing you did not create, your own scratch directory included. Never `git add -A`; never
amend; `git commit -F <file>` naming paths; `Cargo.lock` not staged. Oracle runs from a fresh empty
directory under the wrapper in the plan; three descriptors, never `2>&1`; both engines
`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`; never run anything in
`rust/corpus/oracle-crashes.txt`; never `NUMERIC DIGITS` above 1000. `cargo fmt --all --check` (not
`--edition`). `grep` is `ugrep -I`; use `/bin/grep -a` for counts and `-F` for data patterns.
Comments minimal: one sentence per body naming the C++ function; measurements that state a property
(`measured, oracle rc 163: …`) may stay; no history, no set sizes. Scratch under
`…/scratchpad/task3c/`, builds under `claude-build-scratch/5c-followup-task3c/`.

## Report and hand-back — commit-before-gating, no exceptions

Report `docs/superpowers/records/2026-09-04-phase-5c-followup/task-3c-report.md`: skeleton first;
every claim **measured** (with the command or file) or **inferred**; the witness's text and the
oracle's three descriptors; every refusal measured, in a table like Task 3b's §2.3; which methods
can grow the buffer; the predicted and measured `method-bodies.txt` moves; each control's
prediction and reading; files created outside the tree; open questions. Fast checks yourself —
`fmt`, `clippy`, `--lib`, STRICT corpus, `--test refusal_sites` (constructor definitions are cited
by line; an insertion in `error.rs` moves every row below it, so re-derive from the test's own panic
by script, never by hand), `--test method_bodies`, `--test coverage`, `cargo test --release -p
rexx-parse --test sourceline_oracle`, run counts asserted, **in the foreground with a bounded
timeout** where possible. **Commit code and report together** with the Gates table carrying
`**G1**`–`**G7**` (copy from `task-3b-report.md`), start the seven gates in the background from
`rust/` writing `…/scratchpad/task3c/gates/status.txt` (commit sha its first line, pidfile beside
it, `finished` its last), final message `committed at <sha>, gates running, statuses at <path>`,
stop. **If you background any job and wait on it, send the controller its pidfile and status-file
paths before you go idle** — three completion notifications were lost this phase, and one agent
died on a usage limit with the work uncommitted, which is why the tree is backed up before it is
handed to you.
