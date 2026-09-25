# Task 9 report: `rexx-parse`

BASE `72f2bc3d8`. There are seven code commits. Every artifact cited is
under `docs/superpowers/records/2026-09-15-file-split/task-9-files/`
(`files/` below). The tooling is Task 8's, adapted, in `files/tools/`.

## Commits

| # | commit | what moved | from, into |
| --- | --- | --- | --- |
| c1 | `f3664efd4` | the variable-reference walker (`for_each_variable_name`, `visit_*`) | `block.rs` to `block/references.rs` |
| c2 | `6eb85a36a` | symbol interning (`SymbolId`, `SymbolTable`) | `token.rs` to `token/symbols.rs` |
| c3 | `5c9406d50` | the keyword tables (`KeywordSet`, `Keywords`, the table constants and their comment) | `token.rs` to `token/keywords.rs` |
| c4 | `4b7db8f03` | the parse context and cursor (`ParseCtx`, `TokenCursor`) | `token.rs` to `token/cursor.rs` |
| c5 | `ee6b0b1ba` | the loop header (`create_loop` to `loop_conditional`) | `instruction.rs` to `instruction/loops.rs` |
| c6 | `5833f4a03` | the ADDRESS grammar (`address` to `redirect_target`) | `instruction.rs` to `instruction/address.rs` |
| c7 | `393b94fac` | the PARSE grammar (`parse_instruction_body` to `trigger_position`) | `instruction.rs` to `instruction/parse.rs` |

The order follows the brief. There was no tests-only move to make first:
the two test files in scope are ruled out, and `token/tests.rs` was already
a file. c1 is the leaf with one caller (`Block::add_clause`); the `token.rs`
children follow, then the `instruction.rs` clusters. Each commit message
says `git blame -w -C -C -C` recovers the moved lines.

Line counts, with the plan's `find crates -name '*.rs' | xargs wc -l`:

| file | BASE | final |
| --- | --- | --- |
| `rexx-parse/src/instruction.rs` | 2490 | 1733 |
| `rexx-parse/src/block.rs` | 1023 | 807 |
| `rexx-parse/src/token.rs` | 648 | 265 |
| new: `instruction/{loops,address,parse}.rs` | | 398, 184, 250 |
| new: `block/references.rs` | | 234 |
| new: `token/{symbols,keywords,cursor}.rs` | | 80, 242, 113 |

## What moved, and what stayed, per file

* **`token.rs` (c2, c3, c4), as ruled.** `token.rs` keeps `ParseError`,
  `Operator`, `SymbolClass`, `TokenKind`, `Tag`, `Token` and its test
  module. `token/symbols.rs` takes `SymbolId` and `SymbolTable`;
  `token/keywords.rs` takes `KeywordSet`, `Keywords`, the six table
  constants and the plain comment above them (outside any unit, so it was
  moved by line range and compared with `cmp`, `files/c3/c3-floating.txt`);
  `token/cursor.rs` takes `ParseCtx` and `TokenCursor`. Each `impl` block
  moved whole with its type. `token.rs` declares the three modules and
  re-exports them: `pub use symbols::{SymbolId, SymbolTable}` and
  `pub use keywords::{KeywordSet, Keywords}`, since `lib.rs` re-exports all
  four at the crate root and a `pub(crate) use` cannot be re-exported
  publicly; `pub(crate) use cursor::{ParseCtx, TokenCursor}`, since both
  are `pub(crate)`. Every `crate::token::...` path in the crate keeps
  working, and `token/tests.rs`'s `use super::{..}` is unchanged.
* **`instruction.rs` (c5, c6, c7), as ruled.** Each sub-grammar is a
  further `impl<'a> Inst<'a>` block in a child, its members at their
  original indentation: the loop header (`create_loop`, `plain_loop`,
  `count_loop`, `controlled`, `do_over`, `do_with`,
  `for_and_conditional`, `loop_conditional`), ADDRESS and its redirections
  (`address`, `address_with`, `output_option`, `redirect_target`), and
  PARSE (`parse_instruction_body`, `parse_option`, `parse_template`,
  `trigger_position`; Departure 2). Each set is exactly the members its
  entry point reaches, and each sat contiguously in `instruction.rs`. The
  dispatch (`parse_instruction`, `dispatch`, `keyword`), the cursor
  primitives, every other instruction's members, and every keyword index
  constant (`KW_*`, `COND_*`, `POPT_*`, `SUB_*`, which
  `tests::keyword_indices_still_name_their_own_spellings` pins) stay.
* **`block.rs` (c1): split.** It held two responsibilities: assembling a
  code body (the control stack, the chain, END matching and
  `translate_block`), and a walk over every instruction and expression kind
  naming the variables each references, which reads only `ast` and is
  called from one place. The walker went to `block/references.rs`.
* **Out, as ruled, untouched:** `instruction/tests.rs`,
  `directive/tests.rs`, `ast.rs`, `scanner.rs`, `expr.rs`,
  `directive.rs`, `tests/scanner.rs`.

## Visibility

Every item keeps its visibility except the three entry points and one
walker function, each read from the parent only. Nothing needed
`pub(in crate::<subtree>)`.

| item | file | visibility | read by |
| --- | --- | --- | --- |
| `for_each_variable_name` | `block/references.rs` | private to `pub(super)` | `Block::add_clause`, through `use references::for_each_variable_name` |
| `visit_opt`, `visit_list`, `visit_slot`, `visit_refs`, `visit_expr` | `block/references.rs` | private, unchanged | the walker only |
| `Inst::create_loop` | `instruction/loops.rs` | private to `pub(super)` | `keyword`'s `KW_DO` and `KW_LOOP` arms |
| `Inst::address` | `instruction/address.rs` | private to `pub(super)` | `keyword`'s `KW_ADDRESS` arm |
| `Inst::parse_instruction_body` | `instruction/parse.rs` | private to `pub(super)` | `keyword`'s `KW_PARSE`, `KW_ARG`, `KW_PULL` arms |
| every other moved `Inst` member | its child | private, unchanged | members of the same child |
| the `token.rs` types and their members | `token/*.rs` | unchanged (`pub`, `pub(crate)` or private as before) | through `token.rs`'s re-exports |

The cursor primitives the `Inst` children call (`peek_real`, `next_real`,
`nth_real_index`, `seek`, `sub_keyword`, `error`, `expr`, ...), `Inst`'s
fields and the keyword constants stay private in `instruction.rs`: a child
module may read its parent's private items (Departure 1). Clippy
`--workspace --all-targets -D warnings` passed at every commit, so no
import is unused.

## Pinned items

* **`unsafe`** (`rexx-core/tests/unsafe_sites.rs`, read in full). It names
  `crates/rexx-parse/src/lib.rs` only in `the_scan_reaches_the_whole_workspace`,
  as a path the scan must reach; `lib.rs` is untouched, so that stays true.
  `tools/unsafe_count.sh` applies its two predicates to the lines each
  commit took from its parent and to each new file: `unsafe: NONE` at all
  seven commits (`files/c<N>/c<N>-unsafe.txt`), and `verify9.sh` refused a
  commit otherwise.
* **Path pins** (`files/path-pins-base.txt`, the brief's two `grep`s per
  file before any move, and a wider one): no test, source, bench or
  example names `src/token.rs`, `src/instruction.rs` or `src/block.rs` as
  a path. `corpus/keyword-exempt.txt:56` matches `block.rs` only through
  the regex dot ("blockers"). Comments in `rexx-exec` name `instruction.rs`
  with `message`, `numeric`, `trace` and `if_instruction`, and `block.rs`
  for the IF/ELSE assembly: none of those moved, so each still means what
  it meant (the comment passes rule each). `corpus/errors/parse-errors.tsv:53`
  and `corpus/gate-tables/README.md:106` name `instruction/tests.rs`,
  which stays.
* **`refusal-sites.tsv`**: each commit re-derived it with
  `REXX_REFUSAL_SITES_REFRESH=1`; every column but column 4, and the header
  lines, identical to the committed table, 247 rows, and column 4
  unchanged too (`files/c<N>/c<N>-refusal-sites.txt`). `git status` showed
  no change to the table at any commit.
* **The public API.** `tools/docapi.sh` builds `cargo doc --no-deps -p
  rexx-parse` into an emptied doc directory and writes a manifest: every
  page's sha256 with source links normalised, each redirect stub by its
  target, and the `.js` indexes with defining-module paths normalised
  (its header says what is left out and why). The manifest is identical to
  BASE's at every commit (`files/c<N>/c<N>-docapi-diff.txt`, all empty;
  `verify9.sh` refused otherwise), so every public item, path, signature
  and doc is unchanged. Its controls are under "Tooling". Other crates
  name `rexx_parse::` paths only at the crate root (`rexx_parse::SymbolId`,
  `rexx_parse::{..}`); clippy over the whole workspace compiled them at
  every commit.
* `environment_seam.rs` and `dispatch_seam.rs` name none of these files.

## Departures from the brief

1. **No cursor primitive widened.** The brief says the sub-grammars'
   cursor primitives widen to `pub(super)`. A private item is visible in
   its own module and every descendant, so the children, being modules
   under `instruction`, call `peek_real`, `next_real`, `sub_keyword` and
   the rest, and read `Inst`'s fields and the `SUB_*`/`POPT_*` constants,
   with no widening; the narrowest visibility that compiles is private.
   Only the three entry points widen, because the parent calls into a
   child (Task 8 made the same observation for `values/convert.rs`).
2. **The PARSE child takes the whole PARSE grammar, not only the template.**
   The ruling names "the PARSE template". `parse_instruction_body` (the
   options and the source) is PARSE's entry point, and the other two
   children each take their entry point with the members it reaches;
   leaving it in `instruction.rs` would have widened `parse_template` to
   `pub(super)` for one caller and split one C++ function's (`parseNew`)
   port across two files. `parse_option` is called only by it. The child is
   `instruction/parse.rs`. No ruling was asked for; moving
   `parse_instruction_body` and `parse_option` back is a one-commit
   follow-up if the controller reads it the other way.
3. **`block.rs` was split** (the brief left it to this task, one sentence
   either way; the sentence is above).
4. **Module docs** on the seven new files, one or two lines each, as Task
   8's `convert.rs`. No existing comment was edited: the comment passes
   found none made false.
5. **Instrument 4's test worktree needed the read-only `build/`,
   `oodocs/` and `ootest/`.** The first BASE run in a fresh `git worktree`
   failed 27 tests in five result blocks, `dispatch::library::tests`
   among them, which reads `build/lib` three directories above the crate
   (`files/instrument4/i4-base-nobuild.meta` and `.results`). Symlinks to the main checkout's directories were added
   to the worktree (untracked, outside `rust/`) and BASE re-run: 1991 test
   lines, 63 result blocks, exit 0. Every commit ran on the same worktree.

## The instruments

Per-commit outputs are in `files/c<N>/`, made by `tools/checks9.sh` over
the working tree before each commit (`tools/prod9.sh`: `verify9.sh`, then
`commit_task9.sh`). `commit_task9.sh` refused to commit unless the staged
`rust/` tree hash equalled the one instrument 4 ran on; all seven matched
(`files/instrument4/i4-c<N>.meta`). `verify9.sh` also refused a commit
unless fmt and clippy (`--workspace --all-targets -D warnings`) passed;
the three `cargo doc` runs had BASE's warning signatures; the public-API
manifest was BASE's; the refusal-sites columns were identical; the
test-module doc run printed `Generated`; the unsafe count was `NONE`; no
file outside the parent and destination changed; `/tmp` had 10 GB free;
and `files/c<N>/comment-pass.md` existed.

| # | 1 (a) unmoved | 1 (b) moved units | 2 tokens | 3 literals | 4 tests | load before / after instrument 4 |
| --- | --- | --- | --- | --- | --- | --- |
| c1 | 803 lines equal; 1 import narrowed, 3 inserted | 6 of 6 identical (1 widened) | 48 of 48 | 48 of 48; 119 literals; 0 control mismatches | identical | 8.51 18.52 31.38 / 10.15 15.15 22.92 |
| c2 | 581 equal; 2 declared deletions, 4 inserted | 7 of 7 | 46 of 46 | 46 of 46; 294; 0 | identical | 9.25 12.31 20.30 / 20.53 20.24 20.41 |
| c3 | 357 equal; 2 inserted | 13 of 13 | 41 of 41 | 41 of 41; 278; 0 | identical | 16.28 19.22 20.05 / 14.32 12.16 15.36 |
| c4 | 263 equal; 5 declared deletions, 2 inserted | 10 of 10 | 26 of 26 | 26 of 26; 112; 0 | identical | 10.93 10.31 12.92 / 19.27 19.10 15.79 |
| c5 | 2112 equal; 1 import narrowed, 2 inserted | 8 of 8 (1 widened) | 181 of 181 | 181 of 181; 118; 0 | identical | 17.67 18.48 15.85 / 53.24 39.73 28.22 |
| c6 | 1953 equal; 1 import narrowed, 1 inserted | 4 of 4 (1 widened) | 174 of 174 | 174 of 174; 97; 0 | identical | 57.71 45.45 31.52 / 18.31 28.29 29.14 |
| c7 | 1730 equal; 1 import narrowed, 1 inserted | 4 of 4 (1 widened) | 171 of 171 | 171 of 171; 89; 0 | identical | 21.53 27.36 28.76 / 11.15 17.10 22.22 |

Every moved unit is byte-identical to its PRE text once a widened
declaration's `pub(super) ` is removed; no unit was reflowed. Units counted
in instrument 1 (b) are item-tool's: an `impl` block counts one per member.
The "declared deletions" are whole `use` lines the move left unneeded, and
at c4 the blank line between two import groups that collapsed into one,
each named by `--expect-gone` in `files/c<N>/args`. Instrument 4 ran the
brief's `cargo test -j 4 --release -p rexx-parse -p rexx-exec
--no-fail-fast` with the worktree's default target directory: 1991 test
lines in 63 result blocks, identical per test and per block to BASE at
every commit (`files/instrument4/`).

**`cargo doc --no-deps -p rexx-parse`**: 0 warnings at BASE and at every
commit, in all three runs (public, `--document-private-items`, and that
with `--cfg test`); the signatures are BASE's (`files/c<N>/c<N>-doc-*-sigdiff.txt`,
all empty). `tests_links.sh` (the `#[test]` lines stripped, private items,
`--cfg test`) gives 0 warnings in each parent and its directory at BASE
and at every commit (`files/base/base-tests-links-*.txt`,
`files/c<N>/c<N>-tests-links.txt`). The intra-doc links in the moved code
are enumerated with the command in `files/final/intra-doc-links.txt`:
there are none.

**The comment pass**, per commit (`files/c<N>/comment-pass.md`, from
`comments7.py`'s `c<N>-comments.txt` and `positional.py`'s
`c<N>-positional.txt`), rules on every hit. It found no false comment, so
nothing was corrected. Two corrections to the passes themselves:

* c5's pass omitted one listed hit, `instruction.rs:576` (a position in
  the Rexx clause); c6's pass rules on it.
* `comments7.py` (a) lists an unqualified name outside the parent and
  destination only when it is reached through the parent's module path.
  After c7 I swept every moved function's name unqualified across
  `rust/crates` (`files/final/unqualified-mentions.txt`, command inside):
  `rexx-exec/src/run.rs:1054` (`parse_instruction_body` sets `upper` from
  the implied source), `run/loops.rs:375`, `:378`, `:759` (`count_loop`'s
  and `create_loop`'s parsing), `parse_template.rs:707` (`parse_template`
  fills every trigger's `value`), `block/tests.rs:476` (`visit_refs`) and
  `instruction/tests.rs:1949` (`for_and_conditional`). Each describes
  what the function does, under its unchanged name, and none states where
  it is; all stay true. The three `address` hits name variables, not
  `Inst::address`. The `token.rs` type names are named across the
  workspace as types; no comment names them with a file path ((b) found
  none at c2, c3 or c4).

**Re-run with the final tooling** on all seven commit pairs from `git
archive` trees (`tools/rerun9.sh`, arguments from each commit's
`files/c<N>/args`, `files/final/rerun-summary.txt`): all PASS, and every
output identical to the committed one, c1 to c3 included, whose outputs
were made before the c4 tooling change.

**Cumulative** (`files/final/cumulative.txt`): for each parent, every BASE
unit is present exactly once at `393b94fac`, in the parent or a
destination, identical modulo visibility. **Comments**
(`files/final/comments.txt`, each BASE parent against the final parent
plus its new files): no comment line lost; what was added is the seven
license headers and the seven module docs.

## Tooling changes and controls

* **The positional-word list is widened** (Task 8's review, M2). It is
  now one list, `tools/poswords.py`, read by `comments7.py` (c) and
  `positional.py`: above, below, before, after, ahead, behind, beside,
  next to, `follow\w*`, `preced\w*`, earlier, later, end of, top of,
  bottom of, start of, this file, this module, here, immediately, further
  up/down. The output is noisier, and every hit is ruled on by hand.
* **`instruments.py` accepts declared deletions that difflib pairs with
  inserted lines** (c4). When two import groups collapse into one, the
  blank line between them goes, and difflib aligns the deleted group with
  the new `mod cursor;` as one replace block. Such a block now passes only
  if every PRE line in it is consumed from `--expect-gone`, counted as a
  multiset, and its POST lines are reported as insertions, which an insert
  block admits anyway. Controls B3-c4 to B5-c4 below.
* **`docapi.sh`**, new: the public-API manifest above. Its first form
  hashed every doc file, and at c2 it reported a difference that was
  rustdoc's redirect stubs and `.js` indexes recording the item's new
  defining module. It now records a stub by its target and normalises
  defining-module paths and `fragment_lengths` in the `.js` files, and
  leaves `search.index/` out (length-prefixed, and every item it indexes
  has a page). c1's check ran under the first form (stricter) and passed.
  The BASE manifest is built from a `git archive` of the whole repository
  (a `rust/`-only archive cannot build `rexx-inventory`).
* **`move_items9.py`**, the mover: named units verbatim, in source order,
  with whole `impl` blocks (`BLOCK:`), floating comments (`LINES:`), and
  members gathered into a further `impl` block (`--impl=`).
* **`perf_builds9.sh` stamps every archived file before building**; see
  Performance and Concern 1.
* The per-commit pipeline, adapted from Task 8's with `SPLIT_CRATE=rexx-parse`:
  `prep9.sh`, `checks9.sh`, `verify9.sh`, `prod9.sh`, `commit_task9.sh`,
  `run_tests9.sh`, `i49.sh`, `rerun9.sh`, `run_controls9.sh`,
  `perf_builds9.sh`, `perf9.sh`, `mkmsg9.sh`.

**Controls**, on trees fresh from `git archive`: every earlier task's
(Task 2's five, Task 3b's four, Task 4's eight, Task 5's seven, Task 6's
eight, Task 7's ten, Task 8's fifteen, and the other-edits controls of
Tasks 5 and 6), run four times: before any tooling change
(`files/controls-*-before.txt`), before the first commit
(`-before-first-move`), after the c4 change to `instruments.py`
(`-before-c4`), and at the end (`files/final/controls-*-final.txt`,
summary `files/final/controls-final.out`). Every run caught every control.
This task's own, `tools/controls_task9.py`, 61 of 61 at the end
(`files/final/controls-task9-final.txt`):

* A (29): for each word the widening adds (before, after, ahead, behind,
  follow, follows, followed, precede, precedes, end of, top of, bottom of,
  start of), a comment using it planted in Task 8's c2 parent and in its
  destination is listed by this task's `comments7.py` and not by Task 8's;
  a planted comment with no listed word is listed by neither; and
  `positional.py` lists a planted "follow" naming a moved unit, which Task
  8's does not.
* B, per commit (each commit as committed passes, 7): a token changed in
  a moved body (c1, c4, c5, c6, c7); a literal changed inside a moved
  `.expect(..)` (c2, c5); a doc swap between two moved units (c1, c4, c5);
  a moved unit deleted (c1, c2, c5, c6, c7); an unmoved parent line
  changed (c1, c5); a table entry changed, two entries swapped, a table
  deleted (c3); a doc line changed (c7); a renamed entry point (c6); a
  removed import left undeclared (c2); and for the c4 change: the blank
  line left undeclared (B3-c4), an undeclared line deleted beside the
  declared ones (B4-c4), and a second blank line deleted where one is
  declared (B5-c4). Each is caught by the instruments it names.
  One expectation was corrected after the first final run
  (`files/final/controls-task9-final-first.txt`): B3-c6, the renamed
  `address`, expected I1b and I2 and was reported by I2 alone ("unit
  vanished"), because instrument 1 (b) compares only units present on
  both sides. The rename was caught; the expectation was wrong.
* `tools/controls_docapi.sh` (4 of 4, before c3 and at the end,
  `files/controls-docapi-before-c3.txt`, `files/final/controls-docapi-final.txt`):
  a public method's doc text changed, a public method narrowed to
  `pub(crate)`, and a type removed from the crate root's re-exports each
  change the manifest; lines shifted by an inserted comment do not.

## Performance

The measure is callgrind's `summary:` minus `libc.so.6` and `ld-linux`,
two interleaved rounds (base, head per program per round, `tools/perf9.sh`,
eight runs in parallel), on `rexxcps.rex`, `nop.rex`, `startup.rex` and
`parse.rex`. Every binary was built by `tools/perf_builds9.sh` from a `git
archive` of the whole repository: BASE in its own target directory, then
c1 to c7 in turn in a second one (`files/perf/builds.txt`, which also
records that each build compiled `rexx-parse`). The `.text` sha256 at each
revision (`objcopy -O binary --only-section=.text BIN OUT && sha256sum OUT`):

| revision | `.text` sha256 | bytes | changed from the revision before |
| --- | --- | --- | --- |
| BASE `72f2bc3d8` | `bc09a49f...` | 2522699 | |
| c1 `f3664efd4` | `bc09a49f...` | 2522699 | no |
| c2 `6eb85a36a` | `811527db...` | 2522699 | yes |
| c3 `5c9406d50` | `e9947f31...` | 2522699 | yes |
| c4 `4b7db8f03` | `4f3b6cda...` | 2522699 | yes |
| c5 `ee6b0b1ba` | `e86350ff...` | 2522699 | yes |
| c6 `5833f4a03` | `e86350ff...` | 2522699 | no |
| c7 `393b94fac`, final | `e86350ff...` | 2522699 | no |

c2 to c5 change the release `.text` of `rexx-run`, with its size the same
to the byte at every revision. I did not diff the code to say what moved;
the instruction counts below are the measure the brief asks for.

| program | BASE round 1 | BASE round 2 | final round 1 | final round 2 | change of the means |
| --- | --- | --- | --- | --- | --- |
| rexxcps | 17895012176 | 17895019226 | 17895016468 | 17895024081 | +0.00003% |
| nop | 9340097263 | 9340105016 | 9340098804 | 9340102127 | -0.00001% |
| startup | 60854188 | 60855597 | 60867147 | 60853793 | +0.00917% |
| parse | 1543380482 | 1543379609 | 1543381098 | 1543377220 | -0.00006% |

No axis moves more than 0.01%, far under the brief's 0.5%. The largest,
`startup`, is inside its own run-to-run spread: the final binary's two
rounds differ by 13354 instructions, more than the difference of the
means (5578). Every program's stdout is identical between the binaries
except `rexxcps`'s wall-clock clauses-per-second line
(`files/perf/stdout-compare.txt`). Load 6.55 8.11 14.36 before the runs
and 9.09 8.87 13.99 after (`files/perf/meta.txt`).

**The first build attempt was void, and is kept** (`files/perf-stale/`).
`git archive` stamps every file with its commit's time; in the shared
target directory, c2's sources were therefore older than c1's build
outputs, and cargo called them fresh: c2 to c7 each "Finished" in
0.02-0.04 s without compiling anything, and all showed BASE's `.text`.
`perf_builds9.sh` now touches every archived file before building, and
records that `rexx-parse` compiled; the table above is from that run
only.

## Gates

Over `59b608676`, this report's first commit, whose `rust/` tree is
c7's. `tools/gates.sh` ran them after the commit with the default target
directory (`rust/target`), so the arity suites ran the binary G3 built. It
now waits, before G4 and before G6, until the one-minute load is under 15
and the five-minute load under 40 (the brief's condition), and records the
load it started at. The tree did not change from start to finish
(`tree-changes 0` before and after, `files/gates/status.txt`). Each status
was read unpiped. The counts come from `tools/test_results.py` over each
gate's own log (`files/gates/test-*.results`), and the 604 from that log's
differential report (`files/gates/corpus-differential-*.txt`).

| gate | command | result | load average (1, 5, 15 min) |
| --- | --- | --- | --- |
| G1 | `cargo fmt --all --check` | exit 0 | 7.55, 8.41, 13.50 before the run |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings`, empty target dir | exit 0 | |
| G3 | `cargo build --workspace --all-targets --release`, no cap | exit 0 | |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast` | exit 0; 135 result blocks; 2663 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 14.33, 15.07, 15.36 before; 10.52, 16.24, 16.35 after |
| G5 | `cargo build --workspace --all-targets` (debug) | exit 0 | |
| G6 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | exit 0; 135 result blocks; 2664 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 10.84, 16.04, 16.28 before; 17.35, 17.06, 17.00 after |

Every figure matches BASE's (135 binaries, 2663 / 0 / 4 release,
2664 / 0 / 4 debug, 604 of 604 STRICT). Nothing was re-run:
`introspection_arity::every_unstable_row_is_really_unstable` passed in
both test gates.

## Concerns

1. **Task 7's and Task 8's performance evidence is likely void for their
   last commit, by the stale-build mechanism above.** Their
   `perf_builds*.sh` built the last two commits in turn in one shared
   target directory from `git archive` trees, as this task first did.
   Task 8's own log shows it: `file-split-8/art/perf/build-c2.log` (in the
   scratchpad, not committed) is "Finished ... in 0.04s" with no
   `Compiling` line, so the binary called c2 (`values/convert.rs`, the
   production move) was c1's. Task 8's report says neither commit changed
   `.text` and measured "final" = c2; both rest on that binary. Task 7's
   c6 shows c5's `.text` hash exactly; its logs are gone, so that one is
   unconfirmed, not measured. Earlier tasks were not checked. A re-measure
   of those commits with `perf_builds9.sh`'s stamping would settle it.
2. **Files still over the trigger in `rexx-parse`**, all ruled out by the
   brief: `instruction/tests.rs` (2084), `directive/tests.rs` (1505),
   `ast.rs` (1352), `directive.rs` (1117), `scanner.rs` (1073), `expr.rs`
   (1043), `tests/scanner.rs` (1007). `instruction.rs` is 1733 after the
   three moves: the dispatch, the cursor primitives and the other
   instructions' grammars, each a member of one `impl Inst`, which the
   ruling keeps whole.
3. **The `.text` changed at c2 to c5** with no measurable instruction
   count change. I did not attribute it (symbol order, or inlining
   across the new module boundaries); a reviewer who wants it can diff
   `nm` of `files/perf` binaries rebuilt with the script.
4. `comments7.py`'s (a) scope misses unqualified mentions outside the
   parent and destinations; the by-hand sweep is recorded, but the tool
   was not changed.

## Fix round 1

This round addresses the task review
(`.superpowers/sdd/2026-09-15-file-split/task-9-review.md`): I1, and the
report corrections M1 to M3. It leaves the text above as it was; where
that text says otherwise, this section corrects it. Artifacts are in
`files/fix1/`.

* **I1: c4 committed four instrument outputs into the crate.**
  `rust/crates/rexx-parse/src-out-{instrument1,instrument2,instrument3,verdict}.txt`
  came from `controls_task9.py --pre 4`, which I ran on the working tree
  before committing c4. Its `instruments()` helper wrote the "as committed"
  run's output to `POST + "-out"`, and in `--pre` mode POST is the main
  checkout's `src`. `snap.sh` then took the four untracked files into the
  snapshot. The other-edits check (`git diff`) cannot see untracked files,
  and the tree-hash guard matched because instrument 4 ran on the same
  tree. `cae8c216d` removes exactly those four files. The "no file outside
  the parent and destination changed" line under The instruments was false
  for c4.
  The tooling is fixed in two places:
  * `controls_task9.py` writes every instruments output under its work
    directory in the scratchpad, and asserts the path is outside the
    repository.
  * The other-edits check is now `tools/other_edits9.sh`, called by
    `checks9.sh`. Besides the tracked diff, it lists
    `git ls-files --others --exclude-standard -- rust` outside the parent
    and destinations, and prints `untracked: N files`. `verify9.sh`
    refuses a commit unless that is 0.

  The controls are `tools/controls_untracked.sh`, run on the main checkout
  at `cae8c216d` with c7's parent and destination, each plant removed by
  path afterwards (`files/fix1/controls-untracked-fix1.txt`):
  * U0: on the clean tree, `untracked: 0`, and the check passes.
  * U1: an untracked `rust/crates/rexx-parse/src-out-verdict.txt` gives
    `untracked: 1`, and the check fails.
  * U2: an untracked `src/instruction/planted.rs` that is not a
    destination gives `untracked: 1`, and the check fails.
  * U3: the fixed `controls_task9.py --pre 7` (33 of 33 caught,
    `files/fix1/ctl-pre7.txt`) leaves `git status` under `rust/` empty.
  * U4: after U1 to U3, the check passes again.
  * U5: the committed pre-fix `controls_task9.py --pre 7`
    (`git show 9067e1fca:.../tools/controls_task9.py`, run once from the
    scratchpad) reproduces I1, leaving the same four `src-out-*.txt`
    untracked. The new check reports `untracked: 4` and fails. The four
    were then removed by path (`files/fix1/ctl-pre7-before-fix1.txt`).

  The controller has checked that no other non-`.rs` file under `rust/`
  was added by this plan.
  Checks at `cae8c216d`, as the round asks (`files/fix1/status.txt`):
  `cargo build --workspace` exits 0. `cargo test --release -p rexx-parse`
  exits 0, with 13 result blocks and 409 passed, 0 failed, 0 ignored.
  Every test line and block summary is also in c7's instrument 4 results
  (`files/fix1/test.results`, 0 of 422 lines missing). Load was 9.55 11.83
  16.19 before and 9.47 11.74 16.11 after.
* **M1: the `.text` change at c2 to c5 is attributed** (Concern 3 left it
  open). The explanation is the reviewer's, measured by them, not by me.
  The moved code carries panic `Location`s whose file-name strings are
  now the new files' names. `.rodata` grows from 0x565b0 to 0x56608
  bytes (+0x58), and `.text` starts 0x50 later. `objdump -d` of BASE and
  c7, with addresses, hex immediates and symbol names masked, is
  identical. So the instruction stream is unchanged; only RIP-relative
  displacements moved. c6 and c7 change nothing because their moved code
  adds no new path string.
* **M2: "reads only `ast`" was wrong.** It appears under What moved
  (`block.rs`). The walker also imports
  `crate::token::{SymbolId, SymbolTable}` (`block/references.rs:18`; the
  review cites `:17`, the closing line of the `ast` import above it). It
  reads the AST and resolves symbol ids to names; the rest of the
  sentence stands.
* **M3: `block/references.rs:190` and `:197` name a function that does
  not exist, `for_each_variable_in_expr`.** Both docs said the same in
  `block.rs` at BASE (`block.rs:769`, `:776`), so the move did not make
  them false.
  They are left for a later comment fix, since this plan changes no
  comment the move did not falsify.
