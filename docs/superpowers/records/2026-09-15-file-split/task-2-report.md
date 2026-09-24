# Task 2 report: `rexx-exec/src/dispatch.rs`

BASE `d6dd60d90`. Seven code commits, then this report's commit. Every artifact
cited is under `docs/superpowers/records/2026-09-15-file-split/task-2-files/`
(`files/` below).

## Commits

| # | commit | what moved | `wc -l` now |
| --- | --- | --- | --- |
| c1 | `afa268640` | the test module, to `dispatch/tests.rs` | 1797 |
| c2 | `907bbad0d` | the required-string protocol, to `dispatch/reqstr.rs` | 413 |
| c3 | `37d30fd13` | array slot/subscript arithmetic and `Array`'s natives, to `dispatch/array.rs` | 702 |
| c4 | `71d95fcd0` | `MutableBuffer` (natives and rows) and the shared argument and search helpers, to `dispatch/buffer.rs` | 2206 |
| c5 | `0ffc72de0` | the native constructors, `Pointer`'s and `WeakReference`'s natives and rows, to `dispatch/construct.rs` | 552 |
| c6 | `d18398cfb` | `Class`'s own methods, to `dispatch/class_protocol.rs` | 1065 |
| c7 | `79b530f6f` | `Object`'s own methods, `run`/`send`/`start` and the `Message` readers, to `dispatch/object_protocol.rs` | 1023 |

`dispatch.rs` went from 10710 lines to 3166. Each commit message says
`git blame -w -C -C -C` recovers the moved lines.

## What stays in `dispatch.rs`

The seam and the object model: `mod seam`, `ObjectModel`, `resolve`/`lookup`/
`invoke`, the access checks with `method_is_protected` and
`check_protected_method`, the one `seam::clear(` in `Interp::invoke`, the
send, reply and uninit machinery. Also the message-name constants,
`NATIVE_CLASS_METHODS`, `SETUP_METHODS` and most of `NATIVE_METHODS`
(departure 1), and a residue of natives no ruled child is about (departure 3).

## Departures from the ruled proposal

1. **Most of the native table stays in dispatch.rs.** The proposal had each
   child own "the `NATIVE_METHODS` slice that points at" its functions. The
   table is sorted by class, and its rows do not partition by child: `Class`'s
   block points into both `class_protocol` and `object_protocol`, and its
   comments place it against `Object`'s ("the same identity test `Object`'s
   rows below carry"; `Object`'s `INIT` row: "The same function `ACTIVATE`
   above names"). A class's block moved only where every row points into one
   child and no comment places it against another block: `MutableBuffer`'s
   (to `buffer.rs`), `Pointer`'s and `WeakReference`'s (to `construct.rs`).
   `Array`'s block qualifies by that rule and still stays:
   `corpus/lang/array_make_string.rex`, which cannot be edited without
   reddening `sourceline_matches_the_interpreter_for_every_corpus_program`,
   names "dispatch.rs's NATIVE_METHODS" as where an array's names are written
   down. `NATIVE_CLASS_METHODS` stays whole: its comments cross-reference rows
   of other classes ("for the reason `StringTable`'s below is one", "The one
   row here whose answer is a value rather than an instance").
2. **Chain order is load-bearing, and measured.** `put_native` overwrites, and
   at BASE four `MethodId`s are filed twice with different functions:
   `Array`'s `[]`/`AT`/`ITEMS` rows and `Queue`'s in
   `collection::NATIVE_METHODS` resolve to the same ids and `Queue`'s win
   because that chain comes later; `Package`'s `NEW` loses the same way to
   `package::NATIVE_CLASS_METHODS`. The moved slices are chained directly after
   dispatch.rs's own table, where their rows were. `tools/natives_dump.py`
   (applied to a scratch copy only) dumps what `build` files under every
   `MethodId`, resolved to function names with `nm`; the dump is identical to
   BASE's (`files/natives-base.txt`, 536 entries) after c4, c5 and c7
   (`files/c<N>/c<N>-natives-verdict.txt`). Control: chaining `collection`'s
   table first is seen by it (`files/c4/c4-natives-control-verdict.txt`).
3. **Residue natives stay in dispatch.rs**, about 420 lines: `String`'s
   `length`, `makeArray`, `makeString`, `reverse`, `sign` and `upper`;
   `Directory`/`StringTable`'s `at`/`put`/`unknown` with `hash_index`;
   `VariableReference`'s natives; `Package~local` and `~name`; and the two
   separator entry points `native.rs` names. The survey's ranges left them out
   (its constructor range ended at 8720 and its tests began at 8919, and
   `sign`, the hash and reference natives sat inside ranges it labelled
   otherwise). Each belongs to a class an existing child already serves
   (`string.rs`, `hash.rs`, `package.rs`) or to none of the ruled children;
   moving them into Task 6's files is outside this task. Named here so Task 6
   can take them.
4. **Boundaries re-derived.** The survey's "Object protocol" range held
   `Class` methods (`id`, `defaultName`, `metaClass`, `superClass(es)`,
   `baseClass`, `isSubclassOf`, `annotation(s)`); they went to
   `class_protocol`, as did `Class~copy`, `Class~enhanced` and `Class~package`,
   which sat among `Object`'s natives. A collection's `~init`
   (`native_capacity_init`) and the hash-collection constructors went to
   `construct`; `request_array` and `unconverted_array_argument` to `array`.
   The method-argument parsers (`optional_length_argument` through
   `named_pad_argument`, `whole_method_argument`, `refuse_method_argument`,
   `usize_or_refuse`, `METHOD_ARGUMENT_DIGITS`) went to `buffer` with the
   helpers `string.rs` already imported, as the survey proposed; see concern 3.
5. **No test left `dispatch/tests.rs`**, so no test's fully qualified name
   changed.

Visibility: every item another part of `dispatch` reaches is `pub(super)` in
its child and imported into dispatch.rs with a private `use`, so the existing
`super::name` paths in string.rs, collection.rs, hash.rs, package.rs,
executable.rs, introspection.rs, rexx_info.rs, stream.rs, context.rs and
library.rs work unedited. The one `pub(crate) use` is `reqstr::MAKESTRING`,
for `lib.rs`'s `dispatch::MAKESTRING`. Items that were `pub(super)` at BASE
(`hash_value`, the string-shared helpers) keep the token, which in a child now
means visible to `dispatch` rather than to the crate root: narrower, and the
crate compiles. Each widening is listed per commit in
`files/final-rerun/c<N>-instrument2.txt`.

## Pinned items

* `mod seam`, `method_is_protected`, `check_protected_method` and the
  `seam::clear(` call are untouched in `dispatch.rs`; `dispatch_seam.rs`
  passes at every commit (instrument 4).
* `CLEARANCE_CONSUMERS` gained `array.rs` (c3), `buffer.rs` (c4),
  `construct.rs` (c5), `class_protocol.rs` (c6) and `object_protocol.rs` (c7).
  Each holds native bodies whose signature names `Cleared`, is dispatch code
  the list already covered through `src/dispatch.rs`, and introduces no new
  producer of the token (`seam::clear` stays the only one). `reqstr.rs` and
  `tests.rs` name no `Cleared` and were not added.
* `refusal-sites.tsv`: re-derived with `REXX_REFUSAL_SITES_REFRESH=1` at every
  commit (`files/c<N>/c<N>-refusal-sites.txt`, each showing the table's mtime
  moved, so the refresh ran). No row changed at any commit, so no location
  column moved either: every constructor is defined outside `dispatch.rs`.
  Column 3 is asserted by diffing it for all 247 rows before and after each
  refresh: identical at c1 through c7.
  **A finding about the table's scanner:** `tests/refusal_sites.rs`'s
  `code_lines` drops only an inline `#[cfg(test)] mod x {` block, so since c1
  the code of `dispatch/tests.rs` is scanned as `send`-surface call sites. It
  changes no row today. The header prose said `send` is "dispatch.rs (outside
  its #[cfg(test)] mod tests) and dispatch/native.rs"; c5 corrected it to the
  rule the test applies, naming `dispatch/tests.rs`. That sentence became
  false at c1 and should have been corrected there.
* Path pins (`/bin/grep -rn 'src/dispatch' crates/*/tests crates/*/src`):
  only `dispatch_seam.rs`. Prose naming "dispatch.rs's" X for an X that moved
  was corrected in the commit that moved it: `tests/coverage.rs`,
  `corpus/phase-5a.txt` and `corpus/oracle-crashes.txt` (c1);
  `gate_table_c.rs`'s `methna` control text and `oracle-crashes.txt` again
  (c6). Left as they are: `run.rs:2219`'s "`dispatch.rs:1465`", a record of
  where a past measurement failed; `rust/CLAUDE.md:147`, which quotes a
  `dispatch.rs` doc that no longer exists at BASE; and the two read-only
  corpus copies named in departure 1.

## Comment and doc edits

Four intra-doc links the moves broke, each repointed in the commit that broke
it, with the one widening rustdoc needed to resolve it:

* `ObjectModel::weak_reference`: [`WEAK_REFERENT`] to [`construct::WEAK_REFERENT`] (c5)
* `WEAK_REFERENT`: [`COLLECTION_STORES`] to [`super::COLLECTION_STORES`] (c5),
  then [`super::object_protocol::COLLECTION_STORES`] (c7)
* `native_class_copy`: [`native_copy`] to [`super::native_copy`] (c6)
* `dispatch/tests.rs`: [`factory_metaclass`] to [`class_protocol::factory_metaclass`] (c6)

No `cargo doc` sees the last one: rustdoc never renders a `#[test]` fn's doc,
even with `--cfg test`, measured by breaking that link on purpose and getting
no warning. `tools/tests_links.sh` documents a copy with the `#[test]` lines
removed; it found the break, and shows 0 warnings in `tests.rs` at BASE
(`files/base-tests-links.txt`) and after the fix (`files/c7/c7-tests-links.txt`).

`files/final-rerun/comments.txt` accounts for every comment line: of BASE
`dispatch.rs`'s 2229, exactly the four link lines above are gone, and
everything added is a license header, a module doc, a `mod` comment, a moved
table's doc, or one of the four replacements.

## The instruments

Tooling in `files/tools/`; per-commit outputs in `files/c<N>/`, made as each
commit was built; all seven commit pairs re-run with the final tooling in
`files/final-rerun/` (the c1 outputs in `files/c1/` predate two fixes to the
tooling and are superseded by `final-rerun/c1-*`).

* **Units.** `tools/item-tool` (syn with span locations) lists every top-level
  item, every `impl` member and every native-table row, with its lines, its
  visibility, its whitespace-stripped token stream, and its decoded literals,
  re-tokenizing the raw lines and walking every group, so macro arguments are
  seen.
* **Instrument 1.** (a) The parent's `dispatch.rs` minus the removed lines,
  against the new `dispatch.rs`: every difference must be an inserted line or
  a declared edit, and each equal block is also `cmp`'d. (b) Each moved unit's
  text, the comments above it included, `cmp`'d against its new text after
  undoing only a visibility change (and, for c1, the four-space de-indent).
* **Instrument 2.** Token streams per unit, visibility stripped.
* **Instrument 3.** Decoded literal lists per unit, and the tool's count of
  string, char, byte and doc literals cross-checked per unit, on both sides,
  against `tools/rustlex.py`, a hand-written lexer sharing no code with syn;
  plus the whole-file literal multiset.
* **Instrument 4.** `memcap 8G cargo test -j 4 --release -p rexx-exec
  --no-fail-fast` at BASE and at each commit, reduced by
  `tools/test_results.py` to one line per test and per result block and
  compared whole (`files/instrument4/`).
* **Controls** (`files/c1/c1-controls.txt`, made with the tooling of that
  time): an unmoved line gaining a space is caught by 1(a); a space inside a
  moved test's continued string, token-blind by construction, by 3; a changed
  token by 1(b) and 2.
* **Cumulative** (`files/final-rerun/cumulative.txt`): every one of BASE's 692
  units is present exactly once after c7, identical modulo visibility, or one
  of the five declared edits (`ObjectModel`'s field doc, `build`'s chain,
  `WEAK_REFERENT`'s doc, `native_class_copy`'s doc, the `tests.rs` link).

| # | 1(a) unmoved | 1(b) moved units | 2 tokens | 3 literals | 4 tests |
| --- | --- | --- | --- | --- | --- |
| c1 | 8919 lines equal, 1 inserted | 54 of 56 byte-identical, 2 reflowed | 692 of 692 | 692 of 692; 2517 literals; 0 control mismatches | identical |
| c2 | 8525 equal, 5 inserted | 19 of 20, 1 reflowed | 637 of 637 | 637 of 637; 1863; 0 | identical |
| c3 | 7855 equal, 10 inserted | 32 of 35, 3 reflowed | 620 of 620 | 620 of 620; 1786; 0 | identical |
| c4 | 5683 equal, 18 inserted | 166 of 166 | 586 of 587, `build` declared | 587 of 587; 1673; 0 | identical |
| c5 | 5173 equal, 11 inserted, 1 declared | 31 of 32, 1 declared | 420 of 423, 3 declared | 421 of 423, 2 declared; 1223; 0 | identical |
| c6 | 4142 equal, 13 inserted | 48 of 50, 1 reflowed, 1 declared | 392 of 393, 1 declared | 392 of 393, 1 declared; 1135; 0 | identical |
| c7 | 3154 equal, 12 inserted | 46 of 46 | 345 of 345 | 345 of 345; 954; 0 | identical |

"Reflowed" is rustfmt re-wrapping a signature after `pub(super)` was added
(in c1, after the de-indent gave four more columns); each is token-identical
once the trailing comma rustfmt's vertical layout adds is dropped, and
literal-identical. c7's edit to `construct.rs` (one doc link) and c6's to
`tests.rs` (one doc link) are in files that existed before their commit, which
instruments 1-3 do not scan; each is the single line shown above, and the
cumulative check covers both. Instrument 4: 1582 test lines in 50 result
blocks, 1581 passed, 0 failed, 1 ignored, identical at BASE and at every
commit, loads recorded beside each in `files/instrument4/c<N>.meta`.

`cargo doc --no-deps -p rexx-exec`: 2 warning lines (BASE's one warning, at
`lib.rs:193`) at BASE and every commit, 0 in dispatch files. With
`--document-private-items`: 53 at BASE and at every commit as committed, 0 in
dispatch files; c5, c6 and c7 each showed their broken link before it was
fixed. With `--cfg test` as well: 52, 0 in dispatch files.

## Performance

callgrind `summary:` minus `libc.so.6` and `ld-linux`, two interleaved rounds,
BASE built in its own worktree and target directory, c7 in the main
checkout's; `.text` sha256 `597b63b4...` (BASE) and `f6840a66...` (c7), which
differ. Every program's stdout was identical across all four runs.
`files/perf/results.txt` has the commands and raw figures.

| program | BASE round 1 | BASE round 2 | c7 round 1 | c7 round 2 | change |
| --- | --- | --- | --- | --- | --- |
| rexxcps | 17897295183 | 17897293209 | 17897284566 | 17897281776 | -0.00006% |
| dispatch | 20471073882 | 20471077217 | 20471071256 | 20471080827 | +0.00000% |
| dispatchclass | 15832995341 | 15832979201 | 15832983942 | 15832983129 | -0.00002% |
| varlookup | 14842900428 | 14842900664 | 14842893859 | 14842894157 | -0.00004% |
| arith | 11519340181 | 11519340732 | 11519334215 | 11519331358 | -0.00007% |
| compound | 9234397761 | 9234398223 | 9234397236 | 9234393743 | -0.00003% |
| alloc | 25153461759 | 25153466497 | 25153457550 | 25153458420 | -0.00002% |
| strings | 17788061276 | 17788063225 | 17788058479 | 17788059194 | -0.00002% |

No axis moves more than 0.00007%, so nothing was bisected. The round-to-round
spread within one binary (up to 16140 instructions, on `dispatchclass`) is of
the same size as the BASE-to-c7 differences.

## Gates

Run over `e326811f4` (this report's first commit, code identical to c7) by
`files/gates/gates.sh`, after committing, with the tree unchanged from start to
finish (`git status --short` empty before and after). Statuses, each read
unpiped: `files/gates/status.txt`.

| gate | command | result | load average (1, 5, 15 min) |
| --- | --- | --- | --- |
| G1 | `cargo fmt --all --check` | exit 0 | 1.26, 5.45, 6.79 before the run |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings`, empty target dir | exit 0 | |
| G3 | `cargo build --workspace --all-targets --release`, no cap | exit 0 | |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast` | exit 0; 135 result blocks; 2663 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 18.86, 12.42, 9.25 before; 2.61, 8.29, 8.55 after |
| G5 | `cargo build --workspace --all-targets` (debug) | exit 0 | |
| G6 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | exit 0; 135 result blocks; 2664 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 3.86, 8.31, 8.55 before; 2.06, 6.62, 8.10 after |

All as at BASE. The G4 counts come from `tools/test_results.py` over G4's own
log (`files/gates/test-release.results`), the 604 from that log's differential
report (`files/gates/corpus-differential-release.txt`); likewise for G6. The
flaky `introspection_arity::every_unstable_row_is_really_unstable` did not
fail, so nothing was re-run. G4's 18.86 is the 1-minute figure right after
G2 and G3's builds.

## Concerns

1. **The refusal-sites scanner now reads `dispatch/tests.rs` as production
   code** (pinned items above). No row changes today, but a test added there
   that calls a constructor the send surface does not otherwise reach would
   change column 3. `code_lines` could drop a file declared `#[cfg(test)] mod
   tests;` as it drops an inline one; that is a test change, not a move, so it
   is not made here.
2. **`dispatch.rs` is 3166 lines**: about 2000 of object model, about 560 of
   native tables, about 420 of residue natives (departure 3). The residue is
   the part that is not one thing.
3. **`buffer.rs` is 2206 lines and holds two responsibilities**:
   `MutableBuffer` and the method-argument parsers every primitive class uses
   (`collection.rs` and `package.rs` as well as `string.rs`). The parsers
   (about 330 lines) could stand as their own child; the survey put them with
   `MutableBuffer`, and I kept its ruling rather than add a module it did not
   propose.
4. `class_protocol.rs` (1065) and `object_protocol.rs` (1023) sit at the
   trigger; each is one class's own methods. `tests.rs` (1797) is one test
   module and stays whole for the test-name constraint.
