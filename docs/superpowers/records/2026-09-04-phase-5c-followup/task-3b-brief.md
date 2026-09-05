# 5c follow-up Task 3b — the readers, over the cores

You are the implementer for the **second Task 3 commit** of
`docs/superpowers/plans/2026-09-04-phase-5c-followup.md`: the readers. Read the plan's head, Global
constraints, Task 2 and Task 3; then, in `docs/superpowers/records/2026-09-04-phase-5c-followup/`:
`task-1-report.md` §4.1 (the method-body discipline), §4.2 and §5 (the family table);
`task-2-report.md` §1 and §2.3 (how the seven existing methods are built, and how each refusal was
measured first); `task-3a-report.md` §1.1, §2 (the cores and their signatures — 0-based byte
`start`, 1-based word `position`, `Option<usize>` for an omitted length) and §6 (what it hands you).
`rust/CLAUDE.md` governs; read its Gates and Method sections. **BASE is `git log -1` when you
start.** You work in the worktree `/home/moritz/dev/repos/ooRexx-rust-rewrite`, in `rust/`; it is
yours until the hand-back.

## What you bind

`NATIVE_METHODS` rows for `MutableBuffer`, in the table's alphabetical block between `Method` and
`MutexSemaphore` where Task 2 put the first seven, each at `Setup.cpp:1416`-`:1479`'s declared count:

| name (count) | C++ (`classes/MutableBufferClass.cpp`) | core | answers |
|---|---|---|---|
| `SUBSTR` (3) | `substr` | `string::substr_bytes` | fresh `Text` |
| `[]` (2) | `brackets` (~`:787`) — `substr` with no pad | `substr_bytes` | fresh `Text` |
| `POS` (3) | `posRexx` | `string::find_forward` | `counted` |
| `LASTPOS` (3) | `lastPos` | `string::find_backward` | `counted` |
| `COUNTSTR` (1) | `countStrRexx` | `string::count_occurrences` | `counted` |
| `VERIFY` (4) | `verify` | `string::verify_bytes` | `counted` **on every path** — see below |
| `SUBWORD` (2) | `subWord` | `word::subword_range` | fresh `Text` |
| `WORD` (1) | `word` | `word::word_range` | fresh `Text` |
| `WORDINDEX` (1) | `wordIndex` | `word_range` | `counted` |
| `WORDLENGTH` (1) | `wordLength` | `word_range` | `counted` |
| `WORDS` (0) | `words` | `word::word_count` | `counted` |
| `WORDPOS` (2) | `wordPos` | `word::wordpos_bytes` | `counted` |
| `CONTAINS` (3) | `containsRexx` | `find_forward != 0` | `counted` 0/1 |
| `CONTAINSWORD` (2) | `containsWord` | `wordpos_bytes != 0` | `counted` 0/1 |
| `STARTSWITH` (1) | `startsWithRexx` | prefix compare, `primitiveMatch`'s empty rule | `counted` 0/1 |
| `MATCH` (4) | `match` (`:1590`-ish, `primitiveMatch` `:1615`) | slice compare at an offset | `counted` 0/1 |
| `MATCHCHAR` (2) | `matchChar` | one byte against a set | `counted` 0/1 |
| `SUBCHAR` (1) | `subchar` | one byte | fresh `Text` (empty past the end) |

`ENDSWITH` is Task 2's and stays. **Not this commit**: the mutators (`insert overlay replaceAt []=
changeStr upper lower translate space delWord delete`), the caseless family, the conversions
(`makeString string makeArray subWords`). `MAKESTRING` stays unbound — `say buf` stays loud.

**Read the C++ for each before writing it**, and **measure every refusal on the oracle first** (a
two-line program per case, fresh directory, three descriptors), exactly as Task 2's §2.3 table did:
the method layer raises 93.9xx at rc 163 (`whole_method_argument`, `optional_position_argument`,
`optional_length_argument`, `required_string_argument`, `required_string_named_argument`,
`Raised::missing_method_argument`, `Raised::missing_named_argument`), never the builtin layer's
40.x. Pad arguments are one byte (`padArgument`); option letters are the first byte, case-folded
(`optionArgument`) — measure the refusals for a bad pad and a bad option too.

**`VERIFY` answers `counted` on every path**, including a start past the end — the oracle's
`StringUtil::verify` builds an integer everywhere (Task 3a §6.1). That start-past-the-end case is
also **the only caller that reaches `verify_bytes`'s own guard** (Task 3a M7b): put
`buf~verify('abc', 'N', 9)` → `0` in the witness, and a mutation that deletes the guard must go red
there. Whether the *builtin* `VERIFY` should also answer `counted` past the end is **not** this
commit's; leave `builtin/string.rs` alone.

## Method bodies

Task 1 §4.1's discipline, as the seven existing bodies do it: **convert every argument first**
(copying argument bytes into `take_result_buffer()` where a `&[u8]` is needed alongside the state),
**then** `buffer_state(interp, receiver, b"NAME")?` and the core over `&state.bytes`, then
`text_built`/`interp.text` or `counted`. Readers never take `buffer_state_mut`. A `None` from
`buffer_state` is the `Loud::native_method` refusal, never a panic.

## The witness: `rust/corpus/lang/mutablebuffer_readers.rex`

**Written and confirmed on the oracle before you touch the crate** — rc 0, stderr empty, both
engines byte-identical after. Every name above sent at least once with a positive and a negative
case; the `verify` start-past-the-end case; `substr`/`[]` past the end and with pad; `pos`/`lastpos`
with start and range; word readers on a buffer with leading, multiple and trailing blanks; `match`
at an offset; `matchChar` with a set; the readers answering into the same buffer afterwards
(`buf~string` unchanged — readers do not mutate). `buf`, never `b`. **No `say buf`, no
concatenation of a buffer, no `makeString`**.

**File it in this commit**: `corpus/phase-5c.txt` and `coverage.rs`'s `EXPECTED_SUBSET_5C` (append
under Task 2's two lines), **not** `corpus/unfiled.txt`; plus the
`rexx-parse/tests/sourceline_oracle/mutablebuffer_readers.txt` companion, generated with the driver
in that test's module comment (`sourceline_matches_the_interpreter_for_every_corpus_program` panics
without it). The pin's inversion — the line absent from `EXPECTED_SUBSET_5C` — must redden
`phase_5c_subset_matches_the_committed_list`.

## `corpus/method-bodies.txt`

**Predict first, in your report**: which `MutableBuffer` instance rows move under the receiver
`.MutableBuffer~new('abc')` with a zero-argument send, and the evidence each carries — `words` → `rc
0` (`1`); a name whose first argument is required → the oracle's refusal (`rc 163` `93.903` or `rc
168` `88.901`, measure which per name); `substr`/`pos`/… likewise. Then
`REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies`, read the
diff row by row against the prediction. **The gate: no row moves to `diverge`.**

## Controls, each predicted before it runs

* The witness against BASE's binary (a pristine `git archive` extract under
  `/home/moritz/dev/repos/claude-build-scratch/5c-followup-task3b/`, own `CARGO_TARGET_DIR`): rc 120
  at the first reader, stderr naming it.
* **The corpus control that can go red is `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec
  --test corpus`**, read together with its `N of M matching` line. The plain `--test corpus` binary
  is report mode and exits 0 on a divergence (Task 3a §3.2). Never cite the plain binary's exit as
  a control.
* One mutation per family at least — a `substr` bound, a `pos` start, a word count, the `verify`
  guard, a `contains` negation — each predicted (which test or program catches it), run with
  `--profile mutation` (`target/mutation/`; the release binary is untouched and its sha256 must not
  change), restored from a copy, `touch`, rebuilt, sha confirmed. Run the suite *without* the new
  witness first where a catcher is claimed for it ("can fail" is not "adds coverage").
* `cargo test <name>` matches nothing at exit 0 — assert run counts.

## Rules that bite

No `unsafe`. Never `git checkout -- <path>` on an edited file. No `rm` with a glob or computed path;
delete nothing you did not create. Never `git add -A`; never amend; `git commit -F <file>` naming
paths; `Cargo.lock` not staged. Oracle runs from a fresh empty directory under the wrapper in the
plan; three descriptors, never `2>&1`; both engines `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`;
never run anything in `rust/corpus/oracle-crashes.txt`; never `NUMERIC DIGITS` above 1000. `cargo
fmt --all --check` (not `--edition`). `grep` is `ugrep -I`; `/bin/grep -a`. Comments minimal: one
sentence per body naming the C++ function; measurements that state a property (`measured, oracle rc
163: …`) may stay; no history, no set sizes. Scratch under `…/scratchpad/task3b/`, builds under
`claude-build-scratch/5c-followup-task3b/`.

## Report and hand-back — commit-before-gating, no exceptions

Report `docs/superpowers/records/2026-09-04-phase-5c-followup/task-3b-report.md`: skeleton first;
every claim **measured** (command) or **inferred**; the witness's text and the oracle's three
descriptors; every refusal measured, in a table like Task 2's §2.3; the predicted and measured
`method-bodies.txt` moves; each control's prediction and reading; files created outside the tree;
open questions. Fast checks yourself — `fmt`, `clippy`, `--lib`, STRICT corpus, `--test
refusal_sites` (constructor definitions cited by line; re-derive moved rows from the test's own
panic, never by hand), `--test method_bodies`, `--test coverage`, `cargo test --release -p rexx-parse
--test sourceline_oracle`, run counts asserted, **in the foreground with a bounded timeout** where
possible. **Commit code and report together** with the Gates table carrying `**G1**`–`**G7**` (copy
from `task-3a-report.md`), start the seven gates in the background from `rust/` writing
`…/scratchpad/task3b/gates/status.txt` (sha first line, pidfile beside it, `finished` last), final
message `committed at <sha>, gates running, statuses at <path>`, stop. **If you background any job
and wait on it, send the controller its pidfile and status-file paths before you go idle** — two
completion notifications were lost this phase.
