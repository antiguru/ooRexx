# Task 14 -- re-review 2, scoped to `1d87d90cc..e8717a8b7`

**Verdict: CHANGES REQUESTED.** One must-change, and it is a false sentence in a committed comment
that this diff itself made false. Everything the brief asked me to verify held, and two of the
verifications came out stronger than the report claims. Five further findings are parkable.

Scope: `04cc77f3d` and `e8717a8b7` only. Findings A-H, the `emptyloop`/`varlookup` layout swing and
finding D's closure are taken as ruled and are not revisited. The working-tree changes to
`docs/superpowers/plans/2026-08-17-phase-5a.md` and `rust/corpus/oracle-crashes.txt` are the
controller's and were not touched.

The release binary was mutated four times during this review and is restored:
`sha256sum rust/target/release/rexx-run` is `54b1610d8093bfd0d277dfe563993ca9ff94f96bec5d5b6d25acfa16448f3d13`
before and after, and `git status --porcelain` shows only the controller's two files.

---

## Findings

### F1 -- `raw_argument_positions`'s doc names the wrong instrument set, and this diff is why

**behaviour-adjacent prose, in a committed comment. Must change before this task closes.**

`crates/rexx-exec/src/builtin.rs`, the doc on `raw_argument_positions`:

> **The regression instrument is `corpus/lang/required_string_builtin_raw_argument.rex`**, which arms
> the latch and stores an object through `VALUE`; nothing else in the corpus reddens for this, because
> the value only shows up when something reads the stored object back.

`04cc77f3d` edited that sentence -- "nothing else in the tree" became "nothing else in the corpus" --
in the same commit that added `corpus/lang/required_string_nostring_no_directive.rex`, whose last rows
store an object through `VALUE` and read it back with `~isA`. So the sentence is false, and its own
stated reason ("because the value only shows up when something reads the stored object back") is now
an argument against it.

Measured, not read. `RAW_ARGUMENT_POSITIONS` set to `&[(b"VALUE", &[9])]` so nothing is exempted,
release build, both programs through `cmp.sh`:

```
########## required_string_builtin_raw_argument (VALUE exemption REMOVED)
=== oracle rc=0
=== crate/ir rc=1          DIVERGE
=== crate/tree-walker rc=1 DIVERGE
########## required_string_nostring_no_directive (VALUE exemption REMOVED)
=== oracle rc=0
=== crate/ir rc=1          DIVERGE
=== crate/tree-walker rc=1 DIVERGE
```

Restored and rebuilt to the byte-identical binary afterwards.

This also incidentally confirms fix round 2's finding-H claim that the corpus reddens for an orphaned
row, by a different mutation than the one the report used.

The fix must not enumerate the two programs, since the set-size and enumeration constraint binds
comments. Naming the set does it: the corpus programs that read a stored raw position back are what
redden for this.

### F2 -- every message the new test emits carries a stray backslash

**test-instrument, cosmetic. Should ride along with F1's edit; parkable on its own.**

`builtin.rs:1412`-`1413`, `:1502`-`:1503` and `:1527`-`:1528` end their string-literal lines with
`\\` where a line continuation `\` was meant, so the literal keeps a backslash and the source's own
wrapping indentation. All three of the new test's messages are affected, which is all of them.
Witnessed in the liveness runs below:

```
assertion `left == right` failed: the set of blocks that pass positions on without a constant has moved, and this \
             derivation cannot classify those positions -- run the probes this test's doc names \
             against the new member before widening the list
```

The neighbouring empty-table assertion added by the same commit uses a single `\` and prints as one
clean paragraph, which is the control that says this is a slip and not a choice. `fmt` and `clippy`
cannot see it; the messages the report quotes as evidence are the ones degraded.

### F3 -- "every directive installer" is wider than true

**prose, in a committed comment. Parkable.**

`lib.rs:3073` now reads "The writes are `Interp::arm_reqstr_for`, called from every directive
installer, and `Interp::exec_condition_trap`'s `NOSTRING`/`ANY` arm". `arm_reqstr_for` has two call
sites, `install_method` (`lib.rs:3986`) and `install_attribute` (`lib.rs:4036`). `install_class` and
`install_class_at` are directive installers and do not call it -- correctly, since they add no name to
a class dictionary. `arm_reqstr_for`'s own doc at `lib.rs:4054` carries the qualifier that makes the
claim true: "called from every directive install that adds a name to a class's dictionary". Round 1's
finding was the set size ("both directive installers"); dropping the size also dropped the scope.
Carrying `4054`'s qualifier up to `3073` fixes it without naming a size.

`::CONSTANT` is not a counter-example: `install_directives` only evaluates a constant's expression and
installs no method, so there is no third arming site to miss here. Whether the oracle's
`::constant makeString` should install a method at all is another task's gap.

### F4 -- rc 137 does not have three causes under these bounds

**prose, in the report and in the scratchpad harness. Parkable.**

The report says rc 137 is what "a `timeout -s KILL`, a `ulimit` kill and a kernel OOM all produce",
and `cmp.sh`'s own header comment says the same. Under the bound `cmp.sh` actually applies, a
`ulimit -v 1048576` breach produces neither 137 nor a SIGKILL:

```
crate  rc=134   memory allocation of 32000000 bytes failed        (Rust abort, SIGABRT)
oracle rc=251   Error 5 ... System resources exhausted            (a clean interpreter error)
```

So `note_kill`'s sub-timeout branch -- "the 1048576KB ulimit or a kernel OOM" -- names a cause its own
bound cannot produce, and the harness separates two classes (timeout, not-timeout), not three. That
is a smaller claim than the brief's "distinguishes the three", and the answer to the brief's question
is no.

Nothing rests on it. The report's own hedge is accurate ("no measurement it rests on carries one"),
and I reproduced the self-test on `do forever; end` at rc 137 on all three sides with the timeout
label correct. `cmp.sh` is not a repo artifact, so the only fixable surface is the report sentence.

### F5 -- "the widest movement in the whole sitting" is a per-pass figure stated over the sitting

**prose, in the report and in `e8717a8b7`'s commit message. Parkable.**

`0.033585` is exact and is the widest `per_pass` movement, verified row by row. But the sitting also
carries a `fixed` scope, where `head` over `base` moves up to `51487` instructions on
`dispatchclass`/tw (0.104%) and `998` on `emptyloop`/tw. The commit message's "widest movement
anywhere" has the same reach.

This is looseness and not a wrong result. The same projection over the two earlier sittings shows the
intercept moving far more -- `dispatchclass`/ir moved `59842148` in the round-0 sitting and
`-52069182` in round 1's -- so `fixed` is established noise on this bench and round 2's is the
best-behaved of the three. "The widest per-pass movement" would be exactly true.

### F6 -- the corrected comments narrate their own edit history

**prose, project-wide rather than task-14. Park.**

Three comments in this diff record what an earlier version of themselves said:
`required_string_builtin_raw_argument.rex`'s "and an earlier version of this comment claimed they did"
and its trailer "Phase 5a Task 14, fix round 1; the arming claim corrected in fix round 2", and
`builtin.rs`'s "**What this adds over the corpus, which catches more than an earlier version of this
comment claimed.**". `global-constraints.md:172`'s strike test deletes all three: strike the framing
and each sentence still says the same thing about the code as it is.

I am not raising this against Task 14, because the shape is pervasive and pre-existing --
`/bin/grep -rn "an earlier version of this"` over `crates/` hits `eval.rs`, `ir.rs`, `lib.rs`,
`stem.rs`, `run.rs`, `activation.rs`, `builtin/state.rs`, `run/tests.rs`, `ir/golden_tests.rs`,
`corpus.rs` and more, from tasks and phases before this one. Enforcing it here alone would be
inconsistent. It is a question for the ledger about whether the constraint or the practice moves.

---

## What I verified, and what came out stronger than claimed

### 1. The descriptor over-claim, its fix, and its siblings

The header now says the inversion diverges "**on stdout alone** -- rc 0 and empty stderr on both
sides". Confirmed against the mutation run in section 3 below: two `not reached` lines become
renderings, `--- stderr` empty on all three sides, rc 0 everywhere.

**The regenerated `SOURCELINE()` expectation is right, not merely green.** I re-ran the driver the
harness's own doc prescribes -- `.Package~new(f)~source` through the `%SRCG%` capture, oracle-side,
fresh empty directory, `ulimit -v 1048576`, `timeout -s KILL 10`, `</dev/null` -- and the output is
byte-identical to the committed
`crates/rexx-parse/tests/sourceline_oracle/required_string_nostring_no_directive.txt`. The body is
also byte-identical to the `.rex`, and `count 58` matches `wc -l`. Same two checks on the other two
touched expectations: `required_string_builtin_raw_argument.txt` at `count 60` and
`required_string_operator_argument.txt` at `count 90`, both bodies identical to their `.rex` and both
counts equal to `wc -l`.

**No sibling over-claim.** `/bin/grep -n "exit status\|stderr\|descriptor\|diverge-status\|all three"`
over the review diff hits exactly two lines, and both are the corrected sentence (the `.rex` and its
copy inside the expectation file). No verdict cell anywhere in the range reads `diverge-status`.

The other two header rewrites in this diff are improvements and are accurate against their programs.
`required_string_operator_argument.rex`'s new "the `&` row inside the SELECT against the arithmetic
clause after `done`" points at `when n = 2 then say ('1' & .three)` (line 62) and
`say ('2' + .notanumber)` (line 76), and "a trapped row cannot show a message at all" is right: the
`trapped` handler prints `n 'raised' rc'.'condition('E')`, so a trapped row carries the number and
never the message text.

### 2. Finding A's set, derived rather than taken

```
grep -vhE '^\s*#|^\s*$' phase-5a.txt phase-4a.txt phase-4b.txt phase-4c.txt | sort -u | wc -l   -> 171
/bin/grep -alE 'signal on (nostring|any)' <those 171>                                           -> 4
```

The four are the four the report names. I then widened the pattern past the one the report used, in
case the manifest restriction or the spelling hid a fifth: case-insensitive
`\b(signal|call)[[:space:]]+on[[:space:]]+(nostring|any)\b` over all 189 `corpus/*/*.rex` -- the 171
plus the 18 outside any manifest -- returns the **same four**. So `phase-5a.txt`'s stronger claim
("the only corpus program that reddens when that site is disabled") survives the wider set too, and
no program outside the manifests can reach `exec_condition_trap`'s arm.

### 3. The contrast, re-run

`run.rs:3953` changed to `if false && matches!(&trap.condition[..], b"NOSTRING" | b"ANY")`, release
build, all four programs through `cmp.sh`:

| program | ir | tree-walker |
|---|---|---|
| `lang/required_string_nostring_no_directive.rex` | **DIVERGE** (stdout only, rc 0, stderr empty) | **DIVERGE** (identical) |
| `lang/required_string_nostring.rex` | MATCH | MATCH |
| `lang/required_string_builtin_raw_argument.rex` | MATCH | MATCH |
| `lang/condition_nomethod.rex` | MATCH | MATCH |

Byte-for-byte the report's transcript, including which lines change. Restored: `git checkout`, rebuild,
`sha256sum` back to `54b1610d…`, which independently confirms the report's byte-identity claim.

The contrast is what makes this a witness, and it holds. The "what this check could not see" paragraph
discriminates: it names `MATCH` as the observable a false claim would have produced, which is the
observable the three controls actually produced, and it names the residual (a second arming site
elsewhere, covered only by the debug-only `debug_assert`).

### 4. The raw-position derivation, re-derived independently

I reimplemented the derivation from scratch outside the crate and ran it against
`interpreter/expression/BuiltinFunctions.cpp`:

```
blocks 81
extra ['MAX', 'MIN']
derived [('VALUE', [2])]
NEITHER (classified by no family):   <empty>
```

Three things follow that the report does not say:

* the derived set is `{VALUE: [2]}` **before** the `IMPLEMENTED` restriction, so that filter hides
  nothing at this revision;
* no position constant in the file is classified by neither family, so concern 1's "silently
  excluded" set is empty today rather than merely unmeasured;
* `blocks` is 81, so the brace matcher is finding the whole file and not a prefix.

**The blind spot is correctly bounded, and more tightly than the report argues.** The report defends
the bound by naming `stack->arguments` as "the one shape known to exist". The stronger statement is
checkable: `BuiltinFunctions.hpp`'s position-taking accessor macros are exactly the eleven in the
test's two lists, and `/bin/grep -n "stack->" BuiltinFunctions.cpp` returns only the six
`stack->arguments(argcount - 1)` lines inside `MAX` and `MIN` (the four other hits are
`haystack->posRexx` and friends, method calls on objects, not stack fetches). So every route from a
`BUILTIN` body to an argument position is either one of the eleven macros or the `stack->arguments`
set that the test asserts. A builtin other than `MAX`/`MIN` passing a position straight on **would**
redden `EXTRA_ARGUMENT_BLOCKS`, which is the brief's question, answered yes.

The residual is what the report's concern 1 says: an accessor macro renamed or added upstream falls
into neither list, and both sides would omit the same position. That is real, currently vacuous, and
correctly parked.

### 5. Three liveness mutations, run myself

| mutation | result |
|---|---|
| `&[(b"VALUE", &[3])]` | `the_raw_argument_table_re_derives…` FAILED, `left: [("VALUE", [2])] right: [("VALUE", [3])]`; the guard test passed, which is the right division |
| `EXTRA_ARGUMENT_BLOCKS: &["MAX"]` | FAILED on the blind-spot assertion, `left: ["MAX", "MIN"] right: ["MAX"]` |
| `RAW_ARGUMENT_POSITIONS = &[]` | **both** tests FAILED; the guard names the empty table and the derivation names what it should have held, exactly the division finding H claims |

Tree clean after each. I did not re-run the "extra row" mutation; two of the three claimed plus the
empty-table one is enough to rule the instrument live, and the third is the same assertion.

### 6. Round 2's own sitting, from the TSV

`instrument` kept in the projection throughout. All fourteen `per_pass` `instructions:u` rows
reproduce the report's table to the digit, widest `|delta|` `0.033585` on `dispatchclass`/tw, twelve at
`1.000000`, the other two `1.000005` and `0.999995`. The accumulated `pinned>head` highest-ratio-per-
axis table reproduces all seven cells, and so does `pinned>base`; I checked that the table really is
*highest ratio* and not furthest-from-one, and the three axes where the two readings disagree are
`arith`, `emptyloop` and `varlookup`, whose furthest-from-one `ir` figures (`0.993016`, `0.992022`,
`0.994260`) are the ones the report already gives at line 500. `dispatchclass`/tw large moves
`1.012128` to `1.012132` between rounds, which is the fifth decimal as claimed; every other cell in
that table is unchanged. `574` rows under `14-fixround-2`, one commit label `04cc77f3d`, and the
`README.md` duplicate-key check prints `0`.

`cycles:u` is not used anywhere above.

### 7. `cmp.sh`

Both sides now carry `ulimit -v 1048576` and `timeout -s KILL 10`. **Equal bounds, confirmed by
reading the two subshells side by side** -- this is the one thing that had to be true for the sitting
to be a differential measurement, and it is. The self-test on `do forever; end` reproduces: rc 137 on
oracle, `crate/ir` and `crate/tree-walker`, all three labelled with the timeout as the cause, all three
`MATCH`. The bound demonstrably bites -- see F4's transcript -- so the failure mode that killed the
previous session is closed. F4 is only about how a 137 is *labelled*.

### 8. Neighbourhoods

Read around every insertion. Clean, apart from F1 and F3:

* `phase-5a.txt` -- the new comment block sits above its own entry and the preceding block's "its own
  program below" still points below. No doc block orphaned.
* `coverage.rs` -- the new subset row is inside the Task 14 comment's own run; the "protocol's own own
  messages" corruption is gone and the replacement is accurate.
* `dispatch.rs` -- "the limb-3 test below" is now right where "the second test below" was wrong:
  `required_string_latch_holds` gates on limb 3 first and limb 1 second.
* `lib.rs:4054` -- "every installer derives it the way `LanguageParser::methodDirective` does" is true
  of both call sites.
* the report's head -- `171 of 171`, no closed concern in the list, and it points at each round's own
  concern section. Round 0's "Files changed" carries the disclaimer that it is not the task's total,
  so it did not rot.
* `builtin.rs`'s `raw_argument_positions` doc, the `Args` doc and the collapsed-converting-family
  paragraph -- accurate; the `substr('abcdef', .array, 2)` measurement they rest on is the right shape
  for the claim they make.
* no hardcoded corpus count anywhere in `crates/`, `corpus/*.txt` or `bench-baselines/*.md` needed
  moving from 170 to 171.

### 9. "Say what a check could not see"

Both paragraphs discriminate. Finding A's names `MATCH` as what a false claim would have printed, and
that is what the three controls did print, so the discriminator is observed and not asserted. The
re-derivation's names the same-direction error (a parse missing the position the table misses) and then
explains why the blind-spot assertion is separate. Neither is "the same thing".

---

## A correction to the brief, not a finding

The brief attributes the accepted layout swing to `emptyloop` reading `0.989284`. The TSV says that
figure is `varlookup`/tw in the round-0 sitting; `emptyloop`/ir read `0.992021`. The report has it
right at line 490-493 ("`varlookup` is not a second control"), and `varlookup`/tw is indeed the widest
`per_pass` movement in that sitting on either reading. The ruling is unaffected -- I am only flagging
the axis name so the wrong one does not reach the ledger.
