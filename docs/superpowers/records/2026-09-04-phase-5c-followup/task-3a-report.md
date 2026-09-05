# Task 3a report — the byte cores leave the builtins

BASE `3c2ae6975` (`git log -1` at start). Worktree `/home/moritz/dev/repos/ooRexx-rust-rewrite`,
work in `rust/`, every build in the worktree's own `rust/target` (warm at start; the `--lib`
control ran without compiling). `$S` =
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/ae5f3c99-11ad-4b29-9dfe-f3d991a1a398/scratchpad/task3a`.
`$B` = `/home/moritz/dev/repos/claude-build-scratch/5c-followup-task3a` was created and nothing
was put in it (section 5).

Every claim below is **measured** (the command that printed it is named, its descriptors are under
`$S`) or **inferred** (from reading code, and said so).

## 1. What this commit is

A refactor of `rexx-exec/src/builtin/string.rs` and `word.rs` with no behaviour change: the byte
algorithm inside each builtin the brief names is lifted into a `pub(crate)` function that takes no
`Interp`, and the builtin is its argument prologue plus one call. **No `MutableBuffer` method is
bound**: no `NATIVE_METHODS` row, no `corpus/method-bodies.txt` row moves, no new corpus program
(measured: `git status --short` after the fast checks names the two builtin files, `builtin.rs`
for the one visibility word of section 6.5, and this report; `--test method_bodies` 16 passed,
`--test builtin_status` 19 passed with `corpus/builtin-status.txt` unchanged).

### 1.1 The shapes, and why the sized results are not the brief's `-> Vec<u8>`

**Inferred from `builtin.rs:1146` (`buffer`) and `value.rs:388` (`text_built`)**: `buffer(interp,
len)` is `take_result_buffer()` -- the pooled result `Vec` -- plus a fallible `try_reserve(len)`
that raises 5.1, and `text_built` hands a small result's `Vec` back to that pool. A core answering
a fresh `Vec<u8>` would make `text_built` replace the pooled buffer with a tighter one on every
call, which is the cost `changestr`'s own comment measured on `bench-programs/strings.rex`. So the
sized results keep the pool and the core reserves on it:

* **Appended into a caller's `out`, `fn(out: &mut Vec<u8>, …) -> Result<(), Raised>`**:
  `substr_bytes`, `insert_bytes`, `overlay_bytes`, `changestr_bytes`, `space_bytes`. The builtin
  does `let mut out = interp.take_result_buffer();`, the core does `reserve(out, total)?` (a new
  private helper beside `push_pad`: `try_reserve` mapped to `Raised::system_resources()`), which is
  the same `take` + `try_reserve` sequence `buffer()` ran, and `changestr_bytes` keeps its
  per-growth `reserve`. The only failure a core has is allocation, so the error type is `Raised`,
  not `Failure`; `?` in the builtin converts. `refusal_sites` scans only definitions whose return
  type *starts with* `Raised` or `Loud` (`refusal_sites.rs:224`-`:230`, `rest.starts_with(kind)`),
  so `-> Result<(), Raised>` adds no row -- measured, `--test refusal_sites` 5 passed after.
* **In place, over `&mut [u8]` or `&mut Vec<u8>`** -- `delete_range`'s shape: `translate_bytes`
  and `case_shift_bytes` (both through a private `window(bytes, start, range) -> Option<&mut [u8]>`
  that owns the start-past-end and range-capping rules), and `delword_bytes` (a `drain`). The
  builtin already owns a copy from `required_string`, so it edits that copy and hands it to
  `text_built`, as `delstr` and `reverse` already do. Consequences, **inferred from the diff**:
  `case_shifted` no longer makes a second copy (`string.to_vec()` is gone) and takes its `Vec<u8>`
  by value; `translate` no longer `clone`s; `delword` no longer takes the pooled buffer and its one
  `buffer(interp, …)` reservation -- of at most the input's own length -- is gone, so its 5.1 path
  is gone (a `drain` cannot fail; OOM divergence is licensed). The no-op early returns
  (`interp.text(string)` in `case_shifted`, `interp.text(&string)` in `delword`, `interp.text(b"")`
  in `substr`/`space`) now reach `text_built`, which answers the same value: `text_built` calls
  `text(&bytes)` at or under `INLINE_BYTES` and `text_owned` above it, and D15 makes the bytes the
  identity -- inferred from `value.rs:388`, and every byte-comparing test and both corpus modes
  agree (section 3).
* **Value-returning**: `verify_bytes -> usize`, `word_count -> usize`, `word_range ->
  Option<Range<usize>>`, `subword_range -> Range<usize>`, `wordpos_bytes -> usize`; `word_slices`
  widened from `pub(super)` to `pub(crate)` so `dispatch.rs` can reach the scanner without `Words`
  itself leaving the module.

Conventions: a byte `start` is 0-based in every core (as `delete_range` and `find_forward` already
take it; the builtin does `position_of(v)? - 1`), a word `position` stays the 1-based ordinal the
scanner skips by. An omitted length whose default depends on the subject arrives as `Option<usize>`
and the core applies the default (`substr`/`translate`/`case_shift`/`verify`: to the end;
`insert`/`overlay`: `new.len()`; `subword`/`delword`: `ALL_REMAINING_WORDS`); `changestr`'s limit
stays a `usize` with `usize::MAX` for none, matching `count_occurrences`.

**`overlay_bytes` is the one core whose arithmetic was rewritten rather than lifted**, from
1-based `start` with two guarded subtractions to 0-based with `front = start.min(len)`,
`front_pad = start - front`, `span_end = start.saturating_add(overlay_len)`,
`back = len.saturating_sub(span_end)` and a tail of `target[len - back..]`. The two inline comments
about `span_end` went with the code they described. Its witness is M3 and the splicing test's
measured oracle rows, all of which pass (section 3).

## 2. Cores extracted

Line numbers are `fn` lines; BASE from `git show 3c2ae6975:…`, after from the committed files.

| builtin | core | BASE (builtin) | after (core / builtin) | shape |
|---|---|---|---|---|
| `substr` | `substr_bytes(out, string, start, length: Option, pad)` | string.rs:462 | :465 / :486 | append |
| `insert` | `insert_bytes(out, target, new, start, length: Option, pad)` | :515 | :525 / :563 | append |
| `overlay` | `overlay_bytes(out, target, new, start, length: Option, pad)` | :561 | :583 / :618 | append |
| `space` | `space_bytes(out, string, gap, pad)` | :712 | :734 / :763 | append |
| `changestr` | `changestr_bytes(out, haystack, needle, replacement, limit)` | :833 | :869 / :919 | append |
| `translate` (table form) | `translate_bytes(bytes, out_table, in_table: Option, pad, start, range: Option)` | :932 | :1019 (+ `window` :1043) / :981 | in place |
| `verify` | `verify_bytes(string, reference, option, start, range: Option) -> usize` | :993 | :1095 / :1069 | value |
| `lower`, `upper`, `translate` (no-table form) via `case_shifted` | `case_shift_bytes(bytes, start, range: Option, shift)` | :1029, :1037, :1056 | :1165 / `case_shifted` :1147 | in place |
| `words` | `word_count(text) -> usize` | word.rs:181 | :177 / :259 | value |
| `word`, `word_index`, `word_length` | `word_range(text, position) -> Option<Range>` | :211, :240, :258 | :188 / :285, :306, :319 | value |
| `subword` | `subword_range(text, position, count: Option) -> Range` | :294 | :200 / :350 | value |
| `delword` | `delword_bytes(bytes, position, count: Option)` | :333 | :217 / :376 | in place |
| `word_pos` | `wordpos_bytes(phrase, string, start) -> usize` | :376 | :236 / :401 | value |

Also: `reserve` (string.rs:139, private); `word_slices` (word.rs:167) `pub(super)` -> `pub(crate)`.

### 2.1 Left where they were, and why

`pos`, `lastpos`, `countstr` already call `find_forward`, `find_backward`, `count_occurrences`;
`strip`, `reverse`, `center`, `left`, `right`, `copies`, `abbrev`, `compare` are not
`MutableBuffer` methods. Untouched, per the brief.

**No extracted body read `interp` mid-way.** Measured by the compiler: every core's signature has
no `Interp` and the crate builds; the only `interp` uses inside the old bodies were the fallible
reservation and the result allocation, which stayed in the builtins.

**One thing was deliberately not moved into a core: `verify`'s past-the-end answer.** At BASE the
builtin returns `interp.text(b"0")` for a start past the end and `interp.counted(answer)` for
everything else, and the two are different handles -- `Interp::text` (`value.rs:198`) answers an
inline text and `counted` (`value.rs:294`) a `SmallInt` tag, which `a_counted_answer_is_tagged_
rather_than_a_heap_string` distinguishes through `decode()`. Routing that path through the core
would have changed the representation, so the builtin keeps the early return ahead of the core,
with a one-line comment stating the property. Section 6 carries what this means for the next
commit.

## 3. Controls

### 3.1 The builtin tests, before and after

* **Before, measured** at BASE: `cargo test --release -p rexx-exec --lib` -> rc 0,
  `test result: ok. 779 passed; 0 failed; 0 ignored` (`$S/fastchecks/base-lib.{out,err,rc}`).
* **After, measured** on the committed tree: the same command -> rc 0, `779 passed; 0 failed`
  (`$S/mut/after-refactor/lib.out`, again `$S/mut/restored/lib.out`, again
  `$S/fastchecks/final/lib.out`).

### 3.2 The corpus differential, before and after

* **Before, measured** at BASE: `cargo test --release -p rexx-exec --test corpus --test
  refusal_sites` -> rc 0, `18 passed; 0 failed; 1 ignored` and `5 passed`
  (`$S/fastchecks/base-corpus.out`).
* **After, measured**: plain `--test corpus` -> rc 0, `18 passed; 1 ignored`, report line
  `359 of 359 matching -- REPORT MODE, NOT THE GATE`; `REXX_CORPUS_GATE=1 cargo test --release -p
  rexx-exec --test corpus` -> rc 0, `18 passed; 1 ignored`, `359 of 359 matching`
  (`$S/fastchecks/final/corpus*.{out,err}`).

**A correction to the brief's reading of this control, measured during the mutation round**: the
plain `--test corpus` binary is **report mode** and cannot fail on a divergence --
`corpus_differential` asserts `!gate || mismatches.is_empty()` (`corpus.rs:~900`), so its exit
status is the wrong thing to read; the `N of 359 matching` line it prints to stderr is informative
in both modes, and `REXX_CORPUS_GATE=1` is what turns a divergence red. M1's plain run exited 0
while reporting `358 of 359` (section 3.3). From M6s on, `check.sh` ran the corpus under
`REXX_CORPUS_GATE=1` and read the report line as well.

### 3.3 One mutation per core, predicted before it ran

Predictions were written to `$S/mut/predictions.md` before any mutation ran (its addendum, before
M6s-M14). Driver `$S/mut/mutate.sh`: restore the file from `$S/mut/pristine/`, apply exactly one
replacement (refusing unless the old text occurs once), run `$S/mut/check.sh` (`memcap 8G cargo
test --release -p rexx-exec --lib`, then `--test corpus` -- plain through M8, `REXX_CORPUS_GATE=1`
from M6s), sha256 both test binaries, restore, `touch`. Each row's `lib.out`/`corpus.err`/`sha.txt`
is under `$S/mut/<id>/`. **Every mutant's two binary shas differ from the pristine pair**
(`10de31ae7c40…` lib, `67b34360c207…` corpus, from `$S/mut/after-refactor/sha.txt`), so each was
built; M6 and M6s -- the same source twice -- produced identical shas (`f4302bd8…`, `4c9fc613…`),
which is the reproducibility the protocol rests on. **After the last mutation, `check.sh restored`
rebuilt the restored tree to exactly the pristine pair** and ran `779 passed` / STRICT
`359 of 359` (`$S/mut/restored/`).

| id | core | mutation | predicted | observed | verdict |
|---|---|---|---|---|---|
| M1 | `substr_bytes` | delete the `push_pad` | lib RED `the_extraction_builtins…`; corpus green | lib 778/1 RED that test; plain corpus rc 0 **but `358 of 359`**: `lang/required_string_builtin_arguments.rex`, `abc------` vs `abc` | lib confirmed; **corpus falsified** -- a subset program pads with `substr` |
| M2 | `insert_bytes` | delete the lead `push_pad` | lib RED `the_splicing_builtins…`; corpus green | 778/1 RED that test; `359 of 359` | confirmed |
| M3 | `overlay_bytes` | `front_pad = 0` | lib RED `the_splicing_builtins…`; corpus green | 778/1 RED that test; `359 of 359` | confirmed |
| M4 | `changestr_bytes` | `changes < limit` -> `changes + 1 < limit` | lib RED `the_rewriting_builtins…` (count-2 case); corpus green | 778/1 RED that test; `359 of 359` | confirmed |
| M5 | `translate_bytes` | `unwrap_or(pad)` -> `unwrap_or(*byte)` | lib RED `the_case_shifting_builtins…`; corpus green | 778/1 RED that test; `359 of 359` | confirmed |
| M6 | `case_shift_bytes` | `start` -> `start.saturating_add(1)` | lib RED `the_case_shifting_builtins…`; **corpus RED** via `translate("Hello World")` | 778/1 RED that test; `359 of 359` | lib confirmed; **corpus falsified**: `"Hello World"` begins with an upper-case byte, so the shifted window changes nothing -- and the plain binary could not have gone red anyway |
| M6s | `case_shift_bytes` | M6 again under `REXX_CORPUS_GATE=1` | lib RED same test; STRICT corpus GREEN, `359 of 359` | 778/1 RED that test; rc 0, `359 of 359`; shas identical to M6's | confirmed |
| M7 | `verify_bytes` | `start + 1 + offset` -> `start + offset` | lib RED `the_option_taking_builtins…`, `a_counted_answer_is_tagged…`; corpus green | 775/4 RED those two plus `a_counting_builtin_answers_text…`, `a_null_option_byte_is_accepted…`; `359 of 359` | confirmed (two more catchers than named) |
| M7b | `verify_bytes` | delete its `start >= len` guard | lib GREEN, corpus GREEN -- the builtin refuses that input first | 779/0, rc 0; `359 of 359` | confirmed: **the core's guard is unreachable from the builtin** (section 6) |
| M8 | `space_bytes` | `index > 0` -> `index > 1` | lib RED `the_rewriting_builtins…`; corpus green | 778/1 RED that test; `359 of 359` | confirmed |
| M9 | `word_count` | count starts at 1 | lib RED `a_byte_string_alphabet…`, `only_blank_and_tab…`, `a_counted_answer_is_tagged…`; STRICT corpus RED `358 of 359` `string_builtins.rex` | 774/5 RED those three plus `a_counting_builtin_answers_text…` (word.rs), `separators_outside_the_words…`; corpus rc 101, `358 of 359`, `lang/string_builtins.rex` | confirmed |
| M10 | `word_range` | `start + 1..end` | lib RED `the_positional_builtins…`; STRICT corpus RED `358 of 359` `string_builtins.rex` | 774/5 RED that test plus four others; corpus rc 101 **`357 of 359`**: `string_builtins.rex` and `environment_object_operands.rex` (`rray` vs `Array`) | lib confirmed; corpus red as predicted, **count falsified** -- a second subset program calls `word` |
| M11 | `subword_range` | delete `count == 0 \|\|` | lib RED `subword_answers_a_slice…`; STRICT corpus GREEN | 776/3 RED that test plus `a_negative_count_is_an_invalid_length`, `the_position_is_range_checked…`; rc 0, `359 of 359` | confirmed |
| M12 | `delword_bytes` | delete `scan.skip_blanks()` | lib RED `delword_answers_the_oracles_own_bytes`; STRICT corpus GREEN | 777/2 RED that test plus `a_byte_string_alphabet…`; rc 0, `359 of 359` | confirmed |
| M13 | `wordpos_bytes` | `start..=` -> `start + 1..=` | lib RED `wordpos_matches_words…`; STRICT corpus GREEN | 778/1 RED that test; rc 0, `359 of 359` | confirmed |
| M14 | `window` (shared by `translate_bytes`, `case_shift_bytes`) | `available` one short | lib RED `the_case_shifting_builtins…`; STRICT corpus RED `358 of 359` `string_builtins.rex` | 778/1 RED that test; rc 101, `358 of 359`, `lang/string_builtins.rex` | confirmed |

Every core has a catcher in `--lib`, so no `datadriven` row was added. Three predictions about the
*corpus* were falsified (M1, M6, M10's count) and none about `--lib`; two of the three were the
prediction under-reading what the subset programs call, one was the data.

## 4. Corrections to prose met on the way

* `space`'s inline comment said "the seven word builtins"; moved into `space_bytes` as "the word
  builtins" -- a set size in prose.
* `changestr`'s "lent buffer" paragraph describes the builtin's choice of the pooled buffer, so it
  stayed with `take_result_buffer` in the builtin; the "one search" and "haystack is the floor"
  paragraphs moved into `changestr_bytes` with the loop they describe.
* `verify` gained one sentence at its early return (section 2.1).

## 5. Files created outside the tree

Under `$S`: `fastchecks/{base-lib,base-corpus,clippy1}.{out,err,rc}`, `fastchecks/final/*`
(eight checks, three descriptors each); `mut/check.sh`, `mut/mutate.sh`, `mut/predictions.md`,
`mut/pristine/{string,word}.rs`, `mut/{after-refactor,M1..M14,M6s,M7b,restored}/`
(`lib.{out,err,rc}`, `corpus.{out,err,rc}`, `sha.txt`); `gates/` (this commit's gate run). The
auto-backgrounded batch M5-M8 left
`…/ae5f3c99-…/tasks/bddt49veo.output`. `$B` exists and is empty. Nothing was deleted.

## 6. Open questions

1. **`verify`'s past-the-end zero is the one counted answer that is not tagged.** Measured on the
   oracle's source, `interpreter/classes/support/StringUtil.cpp:1283` onward: `StringUtil::verify`
   returns `IntegerZero` at `startPos > stringLen` (`:1295`-`:1297`) and, on every other `return`
   the function has, `IntegerZero` or `new_integer(…)`, so the oracle builds an integer object
   everywhere. This
   crate's builtin answers `interp.text(b"0")` on that path and `counted(…)` on the others, and the
   difference is a representation the tag test can see and no byte can (inferred from
   `Interp::text` vs `Interp::counted`). Left as it was because this commit changes no behaviour.
   The readers commit binds `MutableBuffer~verify` over `verify_bytes` and will answer `counted` on
   every path; whether the builtin should follow is a one-line change whose witness is a row
   `(b"VERIFY", &[b"abc", b"abc", b"N", b"9"], 0)` in `a_counted_answer_is_tagged_rather_than_a_
   heap_string` -- inferred to fail today, not measured.
2. **`verify_bytes`'s own `start >= len` guard is unreachable from the builtin** (M7b, measured:
   deleting it leaves `--lib` and the STRICT corpus green). It exists for the core's totality; the
   readers commit's `MutableBuffer~verify` is the first caller that can reach it and should carry
   the case.
3. **`delword` no longer has a 5.1 path** (section 1.1). At BASE a `buffer(interp, n)` with `n` at
   most the input's length could raise 5.1 under memory pressure; a `drain` cannot. Inferred; no
   test reaches it; OOM divergence is licensed.
4. **The brief's first corpus control reads the wrong signal.** "`cargo test --release -p
   rexx-exec --test corpus` green" is true of any tree whose programs finish, because that binary
   is report mode (section 3.2). The controls that can go red are `REXX_CORPUS_GATE=1` and the
   report's own `N of 359` line. The later Task 3 briefs should say so; this report does not edit
   the brief.
5. Nothing in this commit is the next commit's `native_mutable_buffer_*`. The cores are reachable
   from `dispatch.rs` as `crate::builtin::string::*_bytes` and `crate::builtin::word::*`:
   measured, `builtin.rs:92` was already `pub(crate) mod string;` (and `dispatch.rs:7793` calls
   `crate::builtin::string::delete_range` through it), while `builtin.rs:93` was a private
   `mod word;`, so every `pub(crate)` word core would have been unreachable from `dispatch.rs`.
   This commit changes that one line to `pub(crate) mod word;` -- a visibility word with no
   behaviour behind it -- and the fast checks were re-run after it (table below).

## Fast checks before the commit

Run from `rust/` on the restored tree, each unpiped into `$S/fastchecks/final/<check>.{out,err,rc}`,
run counts read from each binary's own `test result` line:

| check | exit | run count |
|---|---|---|
| `cargo fmt --all --check` | 0 | (no output) |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | `Checking rexx-exec` then `Finished`, no warning (the linter re-examined the changed crate) |
| `cargo test --release -p rexx-exec --lib` | 0 | `779 passed; 0 failed` |
| `cargo test --release -p rexx-exec --test corpus` | 0 | `18 passed; 1 ignored`, report `359 of 359 matching` |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | `18 passed; 1 ignored`, `359 of 359 matching` |
| `cargo test --release -p rexx-exec --test refusal_sites` | 0 | `5 passed` |
| `cargo test --release -p rexx-exec --test builtin_status` | 0 | `19 passed`; `git diff --stat -- corpus/builtin-status.txt` empty |
| `cargo test --release -p rexx-exec --test method_bodies` | 0 | `16 passed` |

Re-run after the one-word `pub(crate) mod word;` change in `builtin.rs` (section 6.5), into
`$S/fastchecks/final2/`:

| check | exit | run count |
|---|---|---|
| `cargo fmt --all --check` | 0 | (no output) |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | `Checking rexx-exec`, no warning |
| `cargo test --release -p rexx-exec --lib` | 0 | `779 passed; 0 failed` |
| `cargo test --release -p rexx-exec --test refusal_sites` | 0 | `5 passed` |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | `18 passed; 1 ignored`, `359 of 359 matching` |

`Cargo.lock` is unchanged (`git diff --stat -- Cargo.lock` empty) and is not staged.

## Gates

Run from `rust/` by `$S/gates/run.sh` in the background, writing each status unpiped to
`$S/gates/status.txt` as it goes -- the commit sha as its first line, `finished` as its last -- with
the pidfile `$S/gates/pid` beside it and each gate's descriptors in `$S/gates/g<N>.{out,err}`.

| # | command | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **G1** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **G2** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **G3** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **G4** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **G5** |
| 6 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **G6** |
| 7 | `REXX_PHASE_GATE=5d REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **G7** |
