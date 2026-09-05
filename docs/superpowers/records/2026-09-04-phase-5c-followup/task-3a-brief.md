# 5c follow-up Task 3a — the byte cores leave the builtins

You are the implementer for the **first Task 3 commit** of
`docs/superpowers/plans/2026-09-04-phase-5c-followup.md`: the core extraction. Read the plan's head,
Global constraints and Task 3; then `docs/superpowers/records/2026-09-04-phase-5c-followup/`
`task-1-report.md` §1.1, §1.2 and §5 (the inventory of what factors and what does not) and
`task-2-report.md` §1 (`delete_range` is the model you repeat). `rust/CLAUDE.md` governs; read its
Gates and Method sections. **BASE is `git log -1` when you start.** You work in the worktree
`/home/moritz/dev/repos/ooRexx-rust-rewrite`, in `rust/`; it is yours until the hand-back.

## What this commit is, and is not

**A refactor with no behaviour change.** Every string algorithm a `MutableBuffer` method will need
in the later Task 3 commits is today written inline inside a builtin function in
`rust/crates/rexx-exec/src/builtin/string.rs` or `word.rs`, between "arguments converted" and
`interp.text_built(...)`. You lift each such body into a plain function over bytes —
`fn(&[u8], …) -> Vec<u8>`, `fn(&mut Vec<u8>, …)`, or a value-returning `fn(&[u8], …) -> usize` —
that takes **no `Interp`**, and make the builtin call it. `delete_range`
(`builtin/string.rs`, Task 2) is the shape: the builtin keeps its argument prologue and its
`text_built`, and the algorithm has a name.

**No `MutableBuffer` method is bound in this commit.** No `NATIVE_METHODS` row, no
`corpus/method-bodies.txt` row moves, no new corpus program. If you find yourself writing
`native_mutable_buffer_*`, stop — that is the next commit.

## The cores to extract, from Task 1's inventory

| builtin (file:line at BASE, approximate) | core to name | notes |
|---|---|---|
| `substr` (string.rs ~449) | `fn substr_bytes(&[u8], start, len: Option<usize>, pad: u8) -> Vec<u8>` | `[]` is this without pad |
| `insert` (~517) | `fn insert_bytes(target: &[u8], new: &[u8], n, len: Option, pad) -> Vec<u8>` | |
| `overlay` (~563) | `fn overlay_bytes(target: &[u8], new: &[u8], n, len: Option, pad) -> Vec<u8>` | `replaceAt`/`[]=` reuse it |
| `changestr` (~835) | `fn changestr_bytes(hay: &[u8], needle: &[u8], new: &[u8], limit: Option) -> Vec<u8>` | uses `find_forward` |
| `translate` (~934) | `fn translate_bytes(&[u8], out_table: Option<&[u8]>, in_table: Option<&[u8]>, pad, start/range) -> Vec<u8>` | the no-table form is a case shift |
| `space` (~714) | `fn space_bytes(&[u8], n, pad) -> Vec<u8>` | over `word_slices` |
| `verify` (~995) | `fn verify_bytes(&[u8], reference: &[u8], match_mode, start, range) -> usize` | already over `&[u8]`; name it |
| `lower`/`upper` via `case_shifted` (~1058) | split so the byte shift is `fn case_shift_bytes(&[u8], start, range, shift) -> Vec<u8>` and the `ObjRef` allocation stays in the builtin | |
| `delword` (word.rs ~333) | `fn delword_bytes(&[u8], n, len: Option) -> Vec<u8>` | over `Words` |
| `subword`/`word`/`wordindex`/`wordlength`/`words`/`wordpos` | the `Words` scanner and `word_slices` already are the cores; where a builtin does index arithmetic inline after scanning, name that too | |

`pos`, `lastpos`, `countstr` already have `find_forward`, `find_backward`, `count_occurrences` —
leave them. `strip`, `reverse`, `center`, `left`, `right`, `copies`, `abbrev`, `compare` are not
`MutableBuffer` methods — leave them. **The table above is Task 1's reading, not a spec**: if a
function's inline body turns out not to be a pure byte operation (it reads `interp` mid-way), say
so in the report and leave it; if two share a body, extract one core.

Every extracted core is `pub(crate)` with a one-sentence doc naming the builtin it came from, and
lives in the module it came from. Argument semantics stay exactly the builtin's — the core is the
part **after** conversion, so it takes already-validated `usize`s and bytes.

## Controls

* **The builtin tests are the first control**: `cargo test --release -p rexx-exec --lib` before
  and after, same pass count, `string.rs`/`word.rs` `mod tests` among them. Read both counts.
* **The corpus differential is the second**: `cargo test --release -p rexx-exec --test corpus`
  green, and `REXX_CORPUS_GATE=1` too. `lang/string_builtins.rex` is in `phase-5c.txt` and
  exercises many of these.
* **A mutation per extracted core, predicted first**: after extraction, break the core (off-by-one
  in one bound, or drop the pad) and confirm a builtin test or a corpus program goes red; restore
  from a copy (never `git checkout --`), `touch`, rebuild, confirm the sha256 returns. Record
  which catcher caught each. A core no test catches when broken is a finding — write it down, and
  add the builtin-level test that catches it (a `datadriven` row in the existing table, not a new
  binary).
* **`cargo test <name>` matches nothing at exit 0** — assert run counts.

## Rules that bite

No `unsafe`. Never `git checkout -- <path>` on an edited file. No `rm` with a glob or computed path;
delete nothing you did not create. Never `git add -A`; never amend; `git commit -F <file>` naming
paths; `Cargo.lock` not staged. `cargo fmt --all --check` (not `--edition`). `grep` is `ugrep -I`;
`/bin/grep -a`. Comments minimal — one sentence per core; no history, no set sizes. Oracle probes
(if any) from a fresh empty directory under the wrapper in the plan; three descriptors; both
engines `REXX_ENGINE=ir` / `REXX_ENGINE=tree-walker`. Scratch under
`/tmp/claude-1000/…/scratchpad/task3a/` and builds under
`/home/moritz/dev/repos/claude-build-scratch/5c-followup-task3a/`.

## Report and hand-back — commit-before-gating, no exceptions

Report `docs/superpowers/records/2026-09-04-phase-5c-followup/task-3a-report.md`: skeleton first;
every claim **measured** (command) or **inferred**; the table of cores extracted with file:line
before and after; each mutation's prediction, catcher and restore check; the two pass counts; files
created outside the tree. Fast checks yourself (`fmt`, `clippy`, `--lib`, `--test corpus`, **`--test refusal_sites`** —
`corpus/refusal-sites.tsv` cites constructor definitions by line, and if your edits move one you
re-derive the moved rows rather than editing them by hand — asserting run counts), **commit code and report together** with the Gates table carrying `**G1**`–`**G7**`
(copy it from `task-2-report.md`), start the seven gates in the background from `rust/` writing
`…/scratchpad/task3a/gates/status.txt` (sha first line, pidfile beside it, `finished` last), final
message `committed at <sha>, gates running, statuses at <path>`, stop. **Background jobs' completion
notifications have been lost twice this phase** — when you start one, also tell the controller its
pidfile path in your message, and do not wait on more than one background job before the commit.
