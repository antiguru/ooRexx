# Task 14 fix round 1 -- scoped re-review

**Verdict: CHANGES REQUESTED.**

Scope: `127fb74b8..1d87d90cc`, two commits. Everything below is my own running at `1d87d90cc`, from a
fresh empty directory with absolute paths and three descriptors read separately. The five gates are
taken as established by the controller (`fmt` 0, `clippy` 0, corpus 170 of 170, 98 `test result: ok`,
no `FAILED`, no `panicked`), as is finding 1's own witness.

Both behaviour defects are fixed and I reproduced the proof of each. What is still open is prose, and
it is the same distribution the previous thirteen tasks show: the fix round corrected the sentences the
review named and left the sentences beside them, and it asserts one correction that was not made.

---

## Per finding

| # | kind | verdict |
|---|---|---|
| 1 | behaviour | **CLOSED** in the code; the corpus comment is corrected; **the report body is not**, and the fix round says it is |
| 2 | prose | **CLOSED**, and the enumeration re-derives independently |
| 3 | test/instrument | **CLOSED** at the check; both inversions reproduce |
| 4 | prose | **CLOSED**, both corrected counts re-derive |
| 5 | prose | **PARTLY OPEN** -- size column and `cycles:u` row closed, the projection's label still wrong |
| 6 | prose | **CLOSED**, citations exact |
| 7 | prose | **PARTLY OPEN** -- the sweep relocated the shape rather than removing it, and corrupted one comment |

---

## Attention item 1 -- the 81-builtin enumeration, re-derived

I wrote my own extractor rather than reading the report's result. It brace-matches every
`^BUILTIN\((\w+)\)` block in `/home/moritz/dev/repos/ooRexx/interpreter/expression/BuiltinFunctions.cpp`,
reads each block's own `const size_t <NAME>_<arg> = N;` constants, classifies every
`MACRO(NAME, arg)` call against the two families `BuiltinFunctions.hpp:59`-`:86` defines, and reports
both halves:

```
BUILTIN blocks: 81

builtins with a position fetched ONLY through a raw accessor: 1
   VALUE [(2, 'newValue')]

builtins whose converting fetches are NOT in non-decreasing position order: 0

macro-shaped calls with an unrecognised family name: {}
```

Both halves reproduce. `/bin/grep -rln "^BUILTIN(" .` from `interpreter/` names that one file and no
other, with no extension filter, so the file bound holds too. The two named spot checks also hold, run
on both engines:

* **`XRANGE`'s double fetch.** `xrange('a', .K, 'x', 'z')` and `xrange('alpha', .K)` with a saying
  `makeString`: `K asked` exactly once per call, oracle rc 0 and both engines rc 0, stdout
  byte-identical on all three.
* **`VALUE`'s position 3.** `value('zz', 'stored', .S)` prints `S asked` on the oracle and on both
  engines, so the selector is a converting position. (The crate then refuses the external-selector
  form, rc 120 against oracle rc 0 -- a pre-existing loud refusal, not this task's, and the report's
  sentence claims only the conversion.)

**Two blind spots in the method, both checked by running rather than by parsing.**

* **Positions no position constant names are invisible to this pass.** `BUILTIN(MAX)` and
  `BUILTIN(MIN)` declare only `<NAME>_target = 1` and then hand positions 2..N to
  `RexxString::Max`/`Min` as `stack->arguments(argcount - 1)`
  (`expression/BuiltinFunctions.cpp:2007`, `:2018`, `:2036`, `:2046`). Neither my extractor nor the
  report's can classify those. Ruled by running: `max('1', .K)`, `max(1+0, .K)` and `min('9', .K)` with
  a saying `makeString` all print `K asked` before the answer, and oracle, `ir` and `tree-walker` agree
  byte for byte at rc 0 on all three. So the trailing positions are converted and the exemption table
  is right not to name them -- but the report's "exactly one position in the whole set" is a statement
  about the positions a constant names, and it should say so.
* **A converting fetch is not always the *string* protocol.** `required_integer` and friends reach
  `ExpressionStack::requiredIntegerArg`, which calls `requestNumber` -> `numberValue`
  (`classes/ObjectClass.cpp:1489`), not `requestString`. 54 positions across the set are reached only
  through a numeric accessor. Ruled by running: the existing corpus program already exercises two of
  them and matches, and `signal on nostring name h; say substr('abcdef', .Array, 2)` traps NOSTRING on
  the oracle and on both engines at rc 0. So the collapse of "converting" into one family is sound
  here; the doc comment describes it as running "the whole protocol", which for the numeric half is
  true only via `numberValue`.

Neither is a defect. Both are places where the claim is narrower than its wording, and the wording is
what a later task will read.

## Attention item 2 -- the exemption table, and the implementer's own concern

**The orphaned-row test fires, and I did not take it on faith.** Both of its arms, run with
`--no-fail-fast`:

| mutation | result |
|---|---|
| `&[(b"VALUEX", &[2])]` | `FAILED`, `builtin.rs:1360`, `RAW_ARGUMENT_POSITIONS names VALUEX, which is not an implemented builtin` |
| `&[(b"VALUE", &[4])]` | `FAILED`, `builtin.rs:1371` (position past `VALUE_Max` 3) |

**But the division of labour the test's doc claims is wrong.** It says "The corpus program is what
catches the exemption being *wrong*; this catches it being *absent*." I built the release binary with
the row orphaned to `b"VALUEX"` and ran the corpus program: it **differs from the oracle**, rc 1
against rc 0. So the corpus catches the orphaned row as well, and the empty table -- the actual
"absent" case -- passes the guard test vacuously. The test's real value is that it runs without the
oracle gate and names the row; the sentence describing it should say that instead.

**The concern is well founded, and the stated blocker is not.** Concern 1 says committing the
extractor "would need a decision about depending on the C++ tree at test time, which the plan's D56
makes for `oodocs`/`ootest` and not for `interpreter/`". That decision has already been taken and is
running in this crate's test suite: `crates/rexx-exec/tests/gate_tables/orx.rs:79` returns a hardcoded
`PathBuf::from("/home/moritz/dev/repos/ooRexx/interpreter/RexxClasses")`, scanned at test time by
`crates/rexx-exec/tests/gate_table_d.rs:489`, and its own doc comment already writes the justification
("Hardcoded for the same reason the oracle binary's path is"). A re-derivation test over
`interpreter/expression/BuiltinFunctions.cpp` is that shape exactly, under that precedent, and my
extractor is about forty lines. So the honest answer is not that a mechanism is infeasible or blocked;
it is that none was written. Failing that, the table needs a named owner and a re-read rule stated
where the table lives, because concern 1 is otherwise a recorded answer with nothing to notice when it
goes stale.

## Attention item 3 -- both latch inversions

**Inversion B reproduced myself**, debug build, `run.rs:3953` changed to `if false && matches!(...)`,
`signal on nostring name h; say .array`:

```
ir            rc 101   thread 'rexx-interp' panicked at crates/rexx-exec/src/dispatch.rs:1792:9:
                       the required-string latch is off where the protocol would answer differently or raise
tree-walker   rc 101   (identical)
```

and under that same inversion the `makeString` probe still answered `K says hello` at rc 0 on both
engines, so the assert is not simply always firing. Byte for byte what the report records.

**The pre-fix silence reproduced too.** With inversion B still applied *and* the new trap test in
`required_string_latch_holds` disabled (`dispatch.rs:1834`, `if false && self.trap_for(...)`), the same
program is rc 0, stdout `The Array class` / `after`, stderr empty, on both engines, where the oracle
prints `trapped NOSTRING`. That is the "say what a check could not see" answer for the old check: it
would have done nothing, and now it panics. All files restored from copies and rebuilt; `git status
--porcelain` shows only the two files the brief reserves, and the restored release binary hashes back
to `aa15a11d428c34d552131695fe10dd44e41f4f558905965bb42bd06aba2f9dda`.

**The release-build statement is true and the doc says it accurately.** `rust/Cargo.toml`'s
`[profile.release]` sets `debug`, `lto` and `codegen-units` and does **not** set
`debug-assertions`, so `debug_assert!` is compiled out. `required_string_latch_holds`'s doc says "This
detects; it does not protect. It runs under `debug_assert`, so a release build has nothing here", and
`reqstr_armed`'s says "what protects a release build is the arming sites and not a check". Both accurate.

One thing the check gained that the report does not claim: `trap_for` falls back to the `ANY` entry
(`run.rs:3995`), so the new test covers the `SIGNAL ON ANY` spelling of route B as well as `NOSTRING`.

## Attention item 4 -- the new corpus program

**Registered and live.** `rust/corpus/phase-5a.txt:425` and `tests/coverage.rs:1022`; not in
`oracle-crashes.txt`. With the exemption emptied (`RAW_ARGUMENT_POSITIONS = &[]`) and the release
binary rebuilt, it differs from the oracle on **stdout and exit status** on both engines -- oracle rc 0
ending `b is a string 0` / `done`, crate rc 1 ending `NOSTRING raised where the oracle stores the
object`, and the oracle's two `K asked` lines (one per render of the stored object) collapse to one.
Restored: `MATCH` on both engines, stderr empty, rc 0.

**But its stated coverage of the trap-arming route is false, and this is a finding.** The program's own
committed header (lines 12-14) says the last rows are "the protocol armed by a NOSTRING trap instead of
by a makeString, which is the arming route that needs no directive at all", and the report says "It
also carries the trap-armed rows, so the arming route that needs no directive is in it." The program
installs two `::method makeString` directives, and `arm_reqstr_for` runs at directive install, before
any clause executes -- so the latch is already set by route A when `signal on nostring` is reached.
Ruled by running rather than argued: with route B's arming disabled (inversion B, debug build) the
program still **matches the oracle** on both engines at rc 0. A program that exercised route-B arming
would have panicked, as `inv_b.rex` did.

I then checked whether anything else in the corpus covers it. Only three programs in `phase-5a.txt`
arm `NOSTRING` or `ANY` at all -- `required_string_nostring.rex`, `condition_nomethod.rex` and the new
one -- and under inversion B **all three match the oracle**. So no corpus program reddens for the
route-B arming site; the `debug_assert` is the whole instrument, which is what concern 6 says, and the
program comment and the report both claim coverage the corpus does not have.

## Attention item 5 -- the dispatchclass measurement

**Every number checks out, and I reproduced the byte identity independently.** The report does not say
how the identity was established, so I established it: `git checkout 127fb74b8 -- rust/crates` and
`cargo build --release --bin rexx-run` in this tree gives

```
fd1e44db9616429adbabed4639c1f27d0e2afc5b563d70ae42444f6f8c5f6863  target/release/rexx-run
```

which is the hash the report quotes for both the first sitting's `head` and the fix round's `base`. The
build is deterministic in this tree, so the identity claim stands on a reproducible fact rather than on
a retained artifact. (Restored, and the HEAD build hashes back to `aa15a11d...` both times.)

From `phase-5a-arms.tsv`, `dispatchclass | per_pass | ir | instructions:u`:

| task | build | median | min | max |
|---|---|---|---|---|
| `14` | head | 6417.398606 | 6404.357939 | 6421.384423 |
| `14-fixround-1` | base | 6404.359463 | 6387.410016 | 6417.424106 |

`6417.398606 - 6404.359463 = 13.039143`, so "a difference of 13.039" is right, and it exceeds the fix
round's own `+13.024245` delta (`6417.383708 / 6404.359463 = 1.002034`, as quoted). Each median sits
inside the other's range, the second only barely (`6404.359463` against a min of `6404.357939`).
Concern 2's `-13.948737` is `6417.398606 - 6431.347343`, correct. The whole fourteen-row own-contribution
table reproduces exactly, and so does every cell of both accumulated tables.

**The "13 instructions per pass" figure is the loosest of the available bounds, not the tightest, and
the report reads it as the floor.** It is the distance between two *medians*. The same identical binary's
own per-round readings run from `6387.410016` to `6421.384423` across the two sittings -- a span of
**33.974407** instructions per pass, 0.53% of the axis -- and **30.014090** within the single sitting
`14-fixround-1 base` alone. So the resolution floor on this arm is at least 30, not 13. The conclusion
("no reading of it at that size is a measurement of work") is right and if anything understated; the
number carried into Tasks 15 and 16 as a planning fact should be the ~34 figure, or the sentence should
say 13 is a lower bound.

## Attention item 6 -- the neighbourhood

**Both corrected counts re-derive.**

```
git grep -c 'to_text(' df799fee0 -- 'rust/crates/*'   -> sum=149  files=18
git grep -c 'to_text(' 1d87d90cc -- 'rust/crates/*'   -> sum=158  files=18
git diff df799fee0 -- rust/crates | grep -c '^-.*args: &\[Option<ObjRef>\]'   -> 79
git diff df799fee0 -- rust/crates | grep -c "^+.*args: Args<'_>"             -> 79
```

149 across 18 files, 158 at the working tree, 79 and 79. The report's finding-4 table and its body at
line 148 and line 197 both say what those commands say.

**The citations in finding 6 are exact.** `ExpressionStack::requiredStringArg` is at `:142`,
`optionalStringArg` at `:167`, `replace(position, newStr)` at `:154` and `:186`; `requiredStringArgument`
matches nothing under `--include=*.cpp --include=*.hpp` from `interpreter/`. The doc comment now cites
those. (The review's own `:165` was off by two; the report's `:167` is right.)

**Constraint sweep over the diff, all clean.** No `unsafe` added, no non-ASCII byte in any added line
(so no em-dashes), no `diverge-` spelling authored anywhere, exactly one `#[test]` added and exactly one
corpus program added -- consistent with the debug count going 1832 to 1833 -- and no `TODO`/`FIXME`.
Every table in the fix report carries `instructions:u`, and the one `cycles:u` sentence names its row
and says it is not a result.

### New findings

**A. (test-instrument) The new corpus program's trap rows do not exercise the arming route they are
said to exercise, and both a committed comment and the report say they do.** Detailed under attention
item 4: `corpus/lang/required_string_builtin_raw_argument.rex:12`-`:14` and the report's finding-1
section. Proved by running -- the program matches the oracle with route-B arming disabled -- and no
other corpus program covers that site either. The rows are still worth having (a raw position must not
raise under a trap); the claim about *how the latch got armed* is what is false. Severity: this is the
kind of sentence a later task reads to decide it need not add coverage.

**B. (prose) Finding 1's report-body correction was not made, and the fix round says it was.** The fix
round ends its finding-1 section with "The report's 'Two structural consequences' paragraph is corrected
the same way." It is not. At `task-14-report.md:194`-`:195` the paragraph still reads "An unread
argument is converted too ... so converting the whole list is right rather than merely convenient" --
the exact generalisation the review flagged, and "the whole list" is now contradicted by the code,
which skips `VALUE` position 2. Two more sites carry the same stale reading: line 58's context table
("`builtin::run` ..., once over the whole list") and line 267's corpus table ("position order, **the
unread argument**, and the object-versus-conversion naming", where the file's own header was corrected
to name `SUBSTR`'s pad specifically).

**C. (prose) The set-size sweep corrupted a committed comment.** `tests/coverage.rs:1014`-`:1015` now
reads "the NOSTRING condition, the protocol's own / own messages" -- the sweep replaced "four" with
"own" beside an existing "own". `fmt` and `clippy` both pass over it.

**D. (prose) The sweep relocated finding 7's shape instead of removing it.** Three added comments still
assert a set's size, in a set the code can enumerate and that can grow:

* `lib.rs`, `reqstr_armed`: "`Interp::arm_reqstr_for`, called from **both** directive installers" -- a
  third installer is precisely the hazard finding 3 exists about, and `arm_reqstr_for`'s own untouched
  doc says "the **two** installers derive it" a few lines away;
* `builtin.rs`: "One builtin call's arguments, in **both** readings a builtin needs of them" -- the
  report calls this "softened to 'both readings'", but "both" is the count;
* `dispatch.rs`: "wrong in **two independent ways**" (borderline: the protocol's limbs are a set the
  report's own rule exempts, and this is derived from them).

The measured counts the sweep kept -- "exactly one position in the whole set", "81 `BUILTIN(x)` blocks"
-- are correctly kept: those are measurement results over a set outside this repo.

**E. (prose) Finding 5's projection label still does not describe the projection.** The size column is
now carried and every cell reproduces, so that half is closed. But the label is still "the widest
`pinned>head` over both arms **and both sizes**", and the projection is not widest -- it is *highest
ratio*. Derived both ways over `14-fixround-1`, `pinned>head`, `across_builds`, `instructions:u`:

| axis | highest ratio (what the table shows) | furthest from 1.000000 |
|---|---|---|
| `arith` | tw small `0.999778` | ir large `0.993016` |
| `emptyloop` | tw `0.995434` | ir large `0.992022` |
| `varlookup` | tw `0.996025` | ir large `0.994260` |

For three of seven axes the two readings disagree, and for `arith` the row shown is the *narrowest* of
its four. Highest-ratio is the right projection for a "made nothing slower" guard; the word is wrong,
and the review's own worked example (`arith`/ir `0.993016` large against `0.993066` small) was reasoning
under the other rule. `emptyloop` and `varlookup` are ties between `small` and `large`, so their size
labels carry no false figure.

**F. (prose) The report head is stale about the work it now describes.** Line 4: "the corpus is 169 of
169" -- it is 170 of 170. Line 7: "The concerns are a residual ordering case inside one builtin, ..." --
that concern was closed by this round and replaced by two others, neither of which the head names. The
fix round appended a section rather than moving the boundary, which is the recurring shape on this plan.

**G. (prose, minor) The pre-fix stand-in for an inversion survives beside the real inversions.**
`task-14-report.md:178`-`:179` still says "The first version of this check compared identities and
reddened eleven tests in the debug gate, which is the check working." The review's point was that this
is a bug in the check firing, not a wrong latch being caught. Now that two real inversions are recorded
590 lines below, the sentence is redundant as well as wrong, and the section around it
(`:175`-`:176`) still describes the check as only comparing the conversion limbs' bytes.

**H. (prose, minor) The guard test's stated division of labour is wrong.** Detailed under attention
item 2: the corpus catches the orphaned row too, and the empty table passes the guard test vacuously.

## What else I checked and found clean

* The exemption's placement behind the latch: `required_string_arguments` returns early on
  `!self.reqstr_armed` before `raw_argument_positions` is called, so an unarmed program pays one bool.
  The comment saying so is accurate.
* No false positive in the new trap test: traps reach an activation only through
  `exec_condition_trap`, which is the arming site, so a clear latch beside a live non-`CALL` trap is
  always the bug the test names. The green gate at 170 of 170 and 98 `ok` corroborates.
* `collect_stress.rs` and `dispatch.rs` sweep edits read correctly after the change; `parse_template.rs`
  and `required_string_face.rex` likewise, though `phase-5a.txt`'s `required_string_face` entry has a
  two-word orphan line from the reflow (cosmetic).
* The `DO` context table row (`:49`) and the sitting's sentence (`:456`) now say the same thing, and the
  `ir_dual` sentence (`:281`) names what the 9 counts.
* The first sitting's accumulated table cells (`pinned>base` and `pinned>head`) reproduce exactly, as do
  the `emptyloop` `pinned>base` figures of `1.000000` on all four arm/size combinations, the
  `+0.784%` for `compound`/ir, and the ~0.8% layout bound.
* The unverified inlining-mechanism sentence is gone, and the layout conclusion is weakened to what
  `emptyloop` supports, with "I did not run anything at an inlining site" stated.

## What I did not verify

* The five gate commands and finding 1's own witness (established by the controller).
* The full debug suite's 1833 count; I checked only that exactly one `#[test]` was added.
* The per-site "which operand an error names" and trace-line tables in the report body, unchanged by
  this round.
* Whether any program outside `phase-5a.txt` covers the route-B arming site; I swept that manifest and
  the three trap-armed programs in it, not `phase-4a`/`4b`/`4c`.
