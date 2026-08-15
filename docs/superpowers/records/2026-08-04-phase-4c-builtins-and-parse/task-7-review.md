# Task 7 review: the `PARSE` template engine

Reviewed `f322477f..26fb60d9` (2 commits) against
`task-7-brief.md` and `task-7-report.md`.
Verified independently: 12 differential probes against the oracle (fresh empty
directory, `ulimit -v`, three descriptors read separately), the C++ sites the
comments cite, and every count I could re-derive from the tree with
`/bin/grep -a`.
The test suite was not re-run; the report's own test evidence stands.

## Verdict 1 -- spec compliance

**PASS**, with one disclosed deviation (Step 2) and one omission (the brief was
not corrected where it was found wrong).

| Requirement | Verdict | Evidence |
|---|---|---|
| Create `crates/rexx-exec/src/parse_template.rs` | ✅ | 1117 lines, two layers |
| Modify `run.rs`, `lib.rs`, `owners.rs`, `loud.rs`, `coverage.rs`, `trace_oracle.rs`, create `corpus/phase-4c.txt` | ✅ | all seven present |
| Witness committed but inert (`tests/corpus.rs` does not read `phase-4c.txt`) | ✅ | `corpus.rs:436` still passes 4a+4b only |
| No parser code written | ✅ | `rexx-parse` untouched except two `sourceline_oracle` fixtures |
| Sources: `Var`, `Value`, `Arg`, `Source`, `Version` | ✅ | `parse_strings`' five arms |
| `Pull`/`LineIn` still fail loudly | ✅ | measured: exit 120, `rexx-exec: PARSE PULL is not implemented (4c)`; policed in both directions by `keyword-exempt.txt`'s six remaining `PARSE::` rows |
| `PARSE ARG` = current activation's arguments; top-level program argument still absent | ✅ | `call_context.arguments`; `rexx-run` passes no program arguments, so the gap is not reachable through the CLI |
| Step 0(a) `-n` vs `<n` not conflated | ✅ | see the trigger table below; A1/A2 reproduce the brief's rows byte for byte |
| Step 0(b) `Absolute`/`Plus`/`Minus` share the one backward rule | ✅ | B2/B4/B5/B6/B9 all match; `absolute` mirrors `RexxTarget::absolute` including origin-one and the `offset <= start` branch |
| Step 0(b) `>n`/`<n` do **not** follow it | ✅ | B7/B8 = `[][abcdefghij]`, `forward_length`/`backward_length` carry no no-movement rule |
| Step 0(b) errors 26.4 and 38.2 | ✅ | 26.4 in the engine (bounded by the active `DIGITS`, matching `integerTrigger`); 38.2 already parse-time |
| Step 0(c) absent pattern matches at END, empty pattern behaves as absent, non-overlapping, relative-after-pattern measures from the match START | ✅ | E1--E6, F1/F2; `find` refuses an empty needle deliberately |
| Step 0(d) `CASELESS` ASCII-only; `UPPER`/`LOWER` transform the source; order-independent | ✅ | `eq_ignore_ascii_case`; C1--C6; the `>K>` line carries the **untranslated** value and `>>>` the translated one, measured identical to the oracle |
| Step 0(e) comma fence never leaves a target unset; omitted middle holds its place; only the final target keeps leading blanks | ✅ | H1--H3, B3, G1 |
| Step 0(f) `PARSE SOURCE` field 2 varies by context, not depth | ✅ | D1/D2 |
| Step 0(g)/Step 1 the trace shape, all eight kinds, `>.>` at `trace i` | ✅ | `parse_placeholder.expected` is oracle bytes; my own probes reproduce `>.>` inside two nested `DO`s at the right indent |
| Step 1: state whether the existing `trace.rs` keyword path emits `=>` | ✅ | report §2, confirmed at the call: `trace_keyword` passes `" => "` and `quote_tag = true` |
| Step 2: measure `PARSE SOURCE`/`PARSE VERSION`; witnesses in the live corpus | ⚠️ partial, disclosed | `SOURCE`'s witness is in the corpus but the 4c subset is inert until Task 15 (the brief's own preamble); `VERSION` was deliberately kept out of the corpus (report §8, offered as a controller decision) |
| Step 3: engine source-independent; comma fence a template boundary, not a trigger | ✅ | `Cursor` holds no `Interp`; `None` entry calls `next_template` |
| Step 4: `Parse` -> `Owner::InScope`, `EXPECTED_OUT_OF_SCOPE` row removed, `lib.rs` arm loses `Parse`, `loud.rs` witness deleted; `Arg`/`Pull` stay `4c` | ✅ | all four, counts moved 31->32 and 4->3 and 12->11 |
| Step 5: `>.>` -> `Witnessed`, both prefix counts move by one, witness exists first | ✅ | 13->14 and 6->5, test renamed, `>.>` added to `CLAIMED_PREFIXES`, `parse_placeholder` row in `WITNESS_PREFIXES` |
| Global: rendering fixed at creation | ✅ | the operand's `>>>` and 26.4's substitution both use `to_text` on the value; measured `numeric digits 2; e = 1e2` searches for `1E2`, and `found "1E2"` for `+(1e2)` |
| Global: every allocation through `Interp::alloc_with` | ✅ | `Interp::text` -> `text_owned` -> `alloc_with`; no `alloc_with_uncollected` outside `lib.rs:1677` |
| Global: `NOT_IMPLEMENTED_EXIT` for what is not implemented | ✅ | 120 measured for `PARSE PULL`, `PARSE LINEIN`, `ARG`, `PULL` |
| Global: no `unsafe` | ✅ | zero occurrences in the new file |

Four files outside the brief's list were touched -- `trace.rs`,
`keyword-exempt.txt`, `keyword_assertions.rs`, `corpus/README.md`, plus two
`sourceline_oracle` fixtures. Each is a necessary consequence (the `>.>`
emitter; 653 bodies that now pass and would fail the set test if left exempt;
the row count in that file's prose becoming false; the sourceline test reads
every `corpus/lang/*.rex`). No scope creep.

### Trigger kind by trigger kind, against the C++ and the oracle

`Cursor` is a faithful transcription of `RexxTarget`: `move_to_end`/`forward`/
`forwardLength`/`absolute`/`backward`/`backwardLength`/`search`/`caselessSearch`/
`getWord`/`remainder`, position for position, including the two that are easy
to get wrong -- `absolute` measuring from `pattern_end` where `forward`/
`backward`/`backward_length` measure from `pattern_start`, and `getWord`'s
leading-blank skip being bounded by the string rather than the section.

A swap would not be caught by the seven `Cursor` unit tests (they call the
methods by name), which the implementer found with mutation 3 and fixed with
`every_trigger_kind_and_source_reaches_its_own_operation`. I checked that test
row by row rather than trusting it: `<2`/`-2` separate `Minus` from
`MinusLength` in both directions, the `+0`/`>0` pair separates `Plus` from
`PlusLength` (the four `+2`/`>2` rows do **not**, as the doc comment says), the
`2`/`=2` pair pins `Absolute` against all four relatives, and the `'x'`
against `caseless 'x'` pair separates `String` from `Mixed`. No remaining
dispatch swap is invisible to it.

I also constructed the cases the brief's table does not reach, where a target
sits on the trigger immediately after a string pattern -- the one shape that
distinguishes `absolute`'s `pattern_end` start from a `pattern_start` one
(`p 'c' q =4 r` -> `[ab][defghij][defghij]`). Byte-identical, as were
`p 'c' q +2/-1/<2/>2/'f' r`, `.nil` as a source, compound and stem targets
under both trace modes, `parse arg` with an omitted middle under both modes,
`NOVALUE` on `parse var`, the 26.4 path under `trace r`, and both committed
corpus programs.

## Verdict 2 -- task quality

Strong. No correctness defect found in the engine, the trace shape, the
allocation discipline or the boundary moves. Every finding below is about
prose or about what a guard actually guards.

### Important

1. **`the_version_string_is_the_measured_oracle_string` cannot detect the
   hazard the report says it detects.** Report §8: "If the oracle is rebuilt,
   this constant is the one thing in the change that goes stale, and its unit
   test is what says so." The test asserts `VERSION` equals a literal copy of
   itself, so it fires when the constant is *edited* and never when the oracle
   moves under it. The only live re-measurement of that string anywhere is
   `PARSE VERSION language +4 .` / `assertSame("REXX", language)` in
   `ootest/ooRexx/base/keyword/PARSE.testGroup:297`, which pins the first four
   bytes. The build date is unguarded in both directions. The decision to hard
   code it may still be right; the sentence claiming a guard for it is not.
2. **`corpus/README.md`'s new paragraph overstates source coverage.** "Between
   them they cover ... five of the seven `ParseSource` variants" -- the two
   programs construct four (`Value`, `Var`, `Arg`, `Source`). `parse version`
   occurs in neither program's code, only in `parse_sources.rex`'s own header
   prose, which says so explicitly, as does `phase-4c.txt`. The same commit
   therefore states a coverage claim and its refutation.
   (Scan: `/bin/grep -aoiE "parse +(upper +|lower +|caseless +)*(value|var|arg|source|version|pull|linein)"`
   over the three programs `phase-4c.txt` names, matching lines read.)
3. **The brief was found wrong twice and was not corrected.** Step 1's
   parenthetical ("4a's and 4b's `>K>` lines carry a bare value") is false, and
   Step 0(g)'s table is `trace i`-only and misses that a target's value line is
   a *choice* of prefix. Both are recorded only in the report.
   `rust/CLAUDE.md`'s Method section forbids exactly this and cites this exact
   failure mode from an earlier task; the brief file still carries both
   statements, and Task 8/9's readers will meet them there.

### Minor

4. **`parse_triggers.rex`'s "backward" block names the wrong field.** "An
   engine that assigns the null string for equal or backward movement prints
   B4's third field and B5's third field empty" -- that defect empties the
   *second* field in both rows. The third target belongs to the trailing `End`
   trigger, whose section the defect does not touch (`pattern_start`/
   `pattern_end` are unchanged by how the previous trigger sized its own
   section). Same class as the three claims commit `26fb60d9` corrected.
   (The neighbouring `-n`/`<n` claim reads correctly if "the `-2` answer" names
   the *slice rule* rather than the row; it is ambiguous, not false.)
5. **The `VERSION` pin is redundant.**
   `source_and_version_carry_their_own_strings` embeds the same literal and
   goes red on the same edit, so `the_version_string_is_the_measured_oracle_string`
   adds no failure mode the suite does not already have. It is also the
   constant-against-its-own-literal shape this project tracks.
6. **"the eleven operations that move them" is a count of an in-repo
   aggregate.** It is currently accurate for `impl Cursor` minus `new` and
   `string`, but the next operation falsifies it, and the report's own
   parenthetical enumerates a *different* eleven (it lists `new`, which moves
   nothing, and omits `match_at`, which moves four positions). One of the two
   lists is wrong and neither is asserted.
7. **The `PULL`/`LINEIN` boundary is written down in four places**
   (`lib.rs:804`, `run.rs:1874`, `owners.rs:165`, `phase-4c.txt`'s header) plus
   the module doc's design note. It is asserted -- `keyword-exempt.txt`'s six
   rows police it in both directions -- so none of the four is load-bearing,
   and each is a site Task 8 must correct.
8. **Unneeded visibility.** `Cursor`, `Cursor::new` and `Cursor::string` are
   `pub(crate)` inside a private module that nothing outside `parse_template.rs`
   references.

### Checked and clean

* No `unsafe`; allocation only through `Interp::text` -> `alloc_with`.
* `std::mem::take` of `call_context.arguments` in `parse_strings` is GC-safe:
  the argument values are `push_temp`'d in the caller's still-live temps frame
  (`run.rs:3345`), and the collector roots from `RootSet` alone. No `?` between
  the take and the restore.
* Counts re-derived: `keyword-exempt.txt` 655 rows removed (653 `PARSE`, one
  `ASSIGNMENT`, one `DO` -- exactly as the report says), 117 rows left, 111
  `4c`, 6 `defect:`. The six surviving `PARSE::` rows are `Queue`+`Parse Pull`
  or `PUSH`+`pull`, read in `PARSE.testGroup` at `:210` and `:4301`ff.
  `PARSE.testGroup` has 682 `::method`, 792 `assertSame`, 19 `expectSyntax`,
  confirming Step 0(h).
* `sourceline_oracle` counts equal `wc -l` (134 and 112).
* C++ citations spot-checked and correct within a line: `ParseTrigger.cpp:143`-`153`
  and `:271`-`286`, `ParseTarget.cpp:423`-`433`, `RexxActivation.hpp:339`,
  `RexxActivation.cpp:4788`-`4802`, `ParseInstruction.cpp`'s `SUBKEY_ARG` arm
  (no `traceKeywordResult`).
* `Loud::parse_trigger_operand`'s unreachability claim is true as written:
  `rexx-parse`'s `parse_template` fills `value` on every trigger but `End`
  (`instruction.rs:2313`-`2419`), and it is a `Loud` rather than an
  `unreachable!`, so the claim is bounded even if it later stops holding.
* The new `parse_placeholder` witness adds coverage rather than merely being
  able to fail: no other `trace_oracle` witness contains a `PARSE`, so
  mutations 1, 2 and 4 have no other guard in the workspace.
* The comma fence has live coverage after all, through `keyword_assertions`
  (e.g. `PARSE::Test_615`, `Parse Source , d`), not only through the inert
  corpus programs.

## Cannot verify from this diff

* Whether Step 2's "witnesses belong in the live corpus" is satisfied in
  spirit, given that `phase-4c.txt` is inert until Task 15 by design and
  `VERSION` was deliberately excluded. Disclosed in report §8 as a controller
  decision.
* The six mutation results and the three re-run dispatch swaps (report §5) --
  taken from the report, not re-run.
* Whether `corpus/lang/source_arg.rex`, which uses `parse source`/`parse var`
  and is in no phase subset, should now enter `phase-4c.txt`. Cross-task.
