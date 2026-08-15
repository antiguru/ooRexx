# Task 1 review: a real `INTERPRET`, replacing the Task 3 spike

Range reviewed: `38fed3a1..a9420630`, minus `4ccfae88` (controller plan correction,
`docs/` only, ignored as instructed).

## Verdicts

**Spec compliance: MET.** Every requirement in the brief is implemented, including
the two escapes the brief demanded be measured rather than asserted. One brief
statement is itself false and was propagated into the tree (I2 below).

**Task quality: PASS WITH FINDINGS.** No correctness defect in the implemented
behaviour -- eight `LEAVE`/`ITERATE` boundary probes, the `RETURN` probes, seven
extra fragment shapes and the 30-program differential all re-measured here and all
agree with the oracle on stdout and exit code. Two Important findings: an
undisclosed second divergence class (TRACE inside `INTERPRET`), and a false
coverage claim in the new corpus subset file.

Counts: **Critical 0, Important 2, Minor 11.**

---

## Gates, re-run here (not taken from the report)

```
cargo fmt --all --check                                -> 0
cargo clippy --workspace --all-targets -- -D warnings   -> 0
cargo test --workspace                                  -> 0   (852 passed, 0 failed, 4 ignored)
REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus corpus_differential -> 0
                                                           "mode: STRICT (the gate)" / "30 of 30 matching"
```

`rexx-exec` lib tests 188, `tests/spike.rs` 7, assertion table `4224 of 4259` --
all three match the report. No `unsafe` added; no file outside `rust/` and `docs/`
touched; the tree was restored after every experiment and `git status --porcelain`
is empty.

The report used the correct `cargo fmt --all --check` spelling. The invalid
`cargo fmt --edition 2024 --check` spelling appears nowhere except the plan's own
warning against it.

---

## The six claims, re-measured

### 1. `LEAVE` never crosses an `INTERPRET` boundary -- **CONFIRMED, all four numbers, bare case included**

Eight probes written from scratch, each run under
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx FILE )`
with stdout, stderr and rc read as three separate descriptors, then under
`target/release/rexx-run`:

| probe | fragment text | enclosing construct | oracle | this crate |
|---|---|---|---|---|
| L1 | `leave outer` | `do label outer while 1` | 28.3 `("OUTER")`, rc 228 | same |
| L2 | `leave idx` | `do idx = 1 to 3` | 28.3 `("IDX")`, rc 228 | same |
| L3 | `leave` (bare) | `do kk = 1 to 3` | **28.1**, rc 228 | same |
| L4 | `iterate outer` | `do label outer idx = 1 to 3` | 28.4 `("OUTER")`, rc 228 | same |
| L5 | `iterate` (bare) | `do kk = 1 to 3` | **28.2**, rc 228 | same |
| L6 | `leave` (bare) | nothing, top level | 28.1, rc 228 | same |
| L7 | `leave choose` | `select label choose` | 28.3 `("CHOOSE")`, rc 228 | same |
| L8 | `do jj = 1 to 5; ...; leave; end` | control | rc 0, `frag 1/frag 2/after-interpret` | same |

stdout MATCH and rc MATCH on all eight. stderr differs by exactly the one missing
innermost `*-*` echo, on the seven that raise. The bare rows (L3, L5, L6) are the
decisive ones and the implementer's claim that the code previously did the opposite
is borne out by the diff: `run_fragment`'s old `match &flow` forwarded
`Flow::Leave(None, _)` outward with `_ => Ok(flow)`.

Two further shapes I added: a named `ITERATE` matching a `SELECT LABEL` **inside**
the fragment gives 28.5 both ways (so the 28.5 family is still raised where the
match is found, not at the boundary), and `interpret "interpret ""leave"""` inside a
`DO` gives 28.1 rc 228 both ways.

The four-row table test `a_fragments_leave_or_iterate_never_reaches_the_enclosing_loop`
is live: I applied its own stated mutation (restore `Ok(flow)` for the `None` arms)
and it failed. `a_fragments_named_leave_is_resolved_against_the_fragments_own_table`
is live too: replacing `fragment.symbols.name(id)` with a literal `b"BAR"` fails it.

### 2. `RETURN` crosses outward like `EXIT`, left failing loudly -- **CONFIRMED, and nothing half-implements it**

Oracle, main body (`say 'before'` / `interpret "return 7"` / `say 'after'`):
stdout `before\n`, empty stderr, **rc 7**. Oracle, inside a routine reached as a
function: `main-start\nsub-start\nrv= 11\n`, rc 0 -- the fragment's `RETURN`
returned from the routine.

This crate on the same main-body program: stdout `before\n`, stderr
`rexx-exec: RETURN is not implemented (4b)`, rc 120. That is what a reader gets --
loud, named, and outside the `157..=253` Rexx-error band.

Nothing half-implements it: `grep -rn 'InstructionKind::Return' src/` finds only
`plan.rs:146` (name pre-registration, which `Plan::build`'s doc says is deliberate
for every variant) and `lib.rs:433`/`:643` (the loud-owner tables). There is no
`Flow::Return` variant and no `Return` arm in `step`. `tests/owners.rs:115` still
tags it `Owner::Phase("4b")`.

### 3. Corpus `30 of 30`, and the 30th is genuinely compared -- **CONFIRMED by my own negative control**

`REXX_CORPUS_GATE=1` exits 0 and prints `mode: STRICT (the gate)` / `30 of 30
matching`. `phase-4a.txt` has 29 entries, `phase-4b.txt` has 1, so all 29 original
4a programs are in the compared set and all still match.

My own check, not the implementer's: I mutated `InstructionKind::Interpret`'s arm to
`Ok(Flow::Next)` (never run the fragment) and re-ran the gate. It **failed**:

```
29 of 30 matching
mismatches (1):
  [UNCLASSIFIED] lang/interpret_dynamic.rex: stdout differ
      rust:   stdout="0\n" ...
      oracle: stdout="interpreted\n42\nloop 1 \nloop 2 \nloop 3 \n" ...
STRICT (REXX_CORPUS_GATE) mode: 1 of 30 corpus programs disagree with the oracle
```

So the 30th slot is read, run under both interpreters, and compared. `check_case`
(`tests/corpus.rs:314-330`) compares stdout, stderr and exit code as three separate
fields, so the certification covers all three.

Second control: commenting the entry out of `phase-4b.txt` makes
`every_in_scope_variant_is_witnessed_by_the_phase_subsets` fail with
`InstructionKind: 1 in-scope variant(s) unwitnessed by the phase subsets: Interpret`,
while `phase_4a_subset_matches_the_committed_list` still passes -- which
simultaneously proves the witness is load-bearing and that `EXPECTED_SUBSET` really
did stay pinned to `phase-4a.txt` alone.

### 4. Execution did not move off `step_in_temps_frame` -- **CONFIRMED by search**

`grep -rn 'self\.step(' src/ tests/` gives exactly one call site, `run.rs:1402`,
inside `step_in_temps_frame` (`run.rs:1334`), plus three occurrences inside
doc-comment prose (`:247`, `:365`, `:384`). `fn step` is defined once (`run.rs:553`).
The `Interpret` arm calls `run_fragment` -> `run_bounded` -> `step_in_temps_frame`,
the same chain `If`/`Select`/`Do` already used. I16's premise for Task 7 holds.

### 5. The `RootSet::temps_len` tripwire can fire -- **CONFIRMED, I made it fire**

`RootSet` exposes no `pop_temp`, so I added a temporary
`pub fn zz_review_pop_temp(&mut self) { self.temps.pop(); }` to `rexx-core` and
called it from `step`'s `Nop` arm, then ran `do idx = 1 to 2 / nop / end` in a debug
build:

```
thread 'rexx-interp' panicked at crates/rexx-exec/src/run.rs:1406:9:
step popped below its own temps watermark (2 -> 1), so it discarded roots it did not push
```

Both edits reverted; `git status --porcelain` empty afterwards. `pop_frame` is
unchanged, as I22 requires, and the assertion is `debug_assert!` with the `Ok`-path
guard the comment explains.

Note on reachability today: the only handle that can truncate below the entry
watermark is a `FrameId` taken *shallower* than entry, and `FrameId`'s field is
private, so no current in-crate code can produce one. The tripwire is a guard
against a future change that stores a `FrameId` across steps, not a check on
present behaviour -- which is a legitimate reason to have it, and the comment does
not overclaim.

### 6. No fragment-plan cache -- **CONFIRMED, and the hit-rate measurement reproduces**

`Interp::fragment_plan` (`plan.rs:576-599`) builds `Plan::build(&fragment.body, ...)`
fresh on every call and returns the map by value. `BodyKey` (`plan.rs:61-67`) still
has no fragment arm, and its doc already carries the "sound and useless" finding.
Nothing in the diff adds a map keyed on fragment text or id.

I reproduced the measurement independently by printing the fragment text at the top
of `run_fragment` and running `corpus/lang/interpret_dynamic.rex`: **5 executions, 5
distinct texts, 0 hits** (`say 'interpreted'`, `v = 6 * 7`, `say 'loop 1 '`,
`say 'loop 2 '`, `say 'loop 3 '`). Instrumentation reverted.

---

## The five files outside the brief's list

Confirmed exactly as reported:

```
git diff --unified=0 38fed3a1..a9420630 -- src/eval.rs src/value.rs src/stem.rs \
    src/plan.rs src/trace.rs | grep '^[+-]' | grep -v '^\(+++\|---\)' | grep -v 'Interp::new'
```

produces **no output**. Nothing but `Interp::new()` call lines.

Other files outside the brief's four are all accounted for: `rexx-core/src/roots.rs`
is I22's explicitly welcomed `temps_len()` accessor; `tests/loud.rs` is forced by
`owners.rs`'s pinned item 4; `tests/coverage.rs`, `tests/collect_stress.rs` and
`rust/corpus/phase-4b.txt` are Step 8; `rust/corpus/README.md` is its natural
adjunct; `tests/corpus.rs` is the coordinator-granted Concern 3 follow-up. Nothing
unexplained.

---

## Ruling on the dated historical figure

**Substantially right, but it is the wrong line to have defended.**

`tests/corpus.rs:476` reads "Today's expected result, at commit `e0e57825`: 9 of 26
matching, ... the fix is to re-run and record which commit was measured, not to
adjust this comment to match a stale number." Anchored to a named commit and
carrying an explicit instruction not to edit it, that is a dated record, not a live
claim. Retaining it is correct, and the project's "never delete a true comment to
ease a change" rule points the same way. So: **keep it.**

Two qualifications.

* The comment's own instruction is "re-run and **record which commit was
  measured**". This task re-ran, at `a9420630`, and got `30 of 30`. Recording that
  as a second dated line -- rather than replacing the first -- is what the comment
  asks for and would have cost one line. Leaving it untouched satisfies the letter
  and not the intent.
* The genuinely stale text is three lines away and has **no** commit anchor:
  `:26-31` says the runner "replaces the hand-run shell loop this phase has used
  after every task (3 of 26 programs matching before Task 9, 9 of 26 after), and
  each of **the two tasks still to land** should be able to see its own effect".
  4a's tasks have all landed and the phase is gated. That sentence is a live false
  claim about plan state, and the implementer rewrote the paragraph directly above
  it without touching it. See M2.

---

## Findings

### Important

**I1. TRACE inside `INTERPRET` is a second live divergence, and it is disclosed
nowhere.** `rust/crates/rexx-exec/src/run.rs:749-754` (the new `Interpret` arm),
and the report's Concern 1.

Measured. Program:

```rexx
trace r
x = 'nop'
interpret x
```

Oracle stderr (rc 0):

```text
     2 *-* x = 'nop'
       >>>   "nop"
     3 *-* interpret x
       >>>   "nop"
     3 *-* nop
```

This crate's stderr (rc 0):

```text
     2 *-* x = 'nop'
       >>>   "nop"
     3 *-* interpret x
```

stdout matches, rc matches, stderr is short by **two** lines. The same shape with a
fragment that produces a value (`trace r` / `interpret "say 1 + 2"`) additionally
mis-attributes the fragment's own `>>>   "3"` to the enclosing `INTERPRET` clause,
because the fragment's clause line never printed.

Two distinct causes:

* **(a) The new arm never traces its own expression result.** Every other
  value-producing arm calls `self.trace_result(...)` (`run.rs:575`, `:607`,
  `:2614`); `InstructionKind::Interpret` evaluates to `text` and calls
  `run_fragment` without one, so the oracle's `>>>   "nop"` line is simply absent.
  This is an omission in the arm *this task shipped*, not inherited.
* **(b) Fragment clause echoes are suppressed.** `run_fragment` calls
  `run_bounded(&code, 0, len, None)`, and `step_in_temps_frame`'s echo is gated on
  `clause_site(source, instruction)`, which is `None` for every instruction in a
  fragment. That convention is pre-existing and documented at `run.rs:1330-1333`,
  but nowhere is it recorded that it *diverges from the oracle*.

Why this is Important rather than Minor: report Concern 1 states the bound "**Any
future corpus program that raises inside `INTERPRET` text will fail the
byte-for-byte gate until it is fixed.** `interpret_dynamic.rex` avoids it only by
raising nothing." Measurement falsifies that bound -- a program that raises nothing
also fails, if it traces, and `TRACE` has been fully in scope since 4a (there are
already `lang/trace_*.rex` corpus programs). The next task to add a 4b witness will
inherit a disclosure that does not cover the case it hits.

Fix: add the missing `trace_result` call to the arm at `run.rs:749-754`, matching
`Say`'s shape; and amend Concern 1 (and `run_activation`'s comment at
`run.rs:445-460`, which currently frames the whole gap as a `Raised::report` issue)
to record the TRACE clause-echo divergence as a second class, with the transcript
above. Whether (b) is fixed here or handed to Task 2 with the echo-stacking work is
a scoping call; leaving it undisclosed is not.

**I2. `rust/corpus/phase-4b.txt:20-21` claims coverage the witness does not have.**

The header says the subset admits

> `INTERPRET` (4b Task 1) -- a dynamic fragment, **a fragment that binds a name the
> enclosing body never mentions**, and a fragment inside a `DO` body.

The middle clause is false. Measured, not read: I instrumented `Interp::slot_of`'s
growth branch (`plan.rs:556-559`, the `Activation::extra` path) with an `eprintln!`
and ran `corpus/lang/interpret_dynamic.rex` -- **0 hits**. Positive control
`interpret "zork = 42"` / `interpret "say zork"` -- **1 hit, `ZORK`**. Every name any
fragment in that program binds is `V`, and the enclosing body binds `V` itself at
line 3 (`v = 0`), so it is already in the plan and `extra` is never reached.

The consequence is not cosmetic: the brief's own Step 6 singles out "a name the
enclosing plan never saw goes through `Plan::slot_of` into `Activation::extra`" as
the property that forces the design, and that path is now witnessed **only** by
`lib.rs`'s in-crate unit test, which never sees the oracle. The differential corpus
does not cover it at all.

The brief itself makes the same claim (`task-1-brief.md:73`), so this is an
inherited error -- but the task's job was to verify the witness, and a claim in a
file this task created is this task's claim.

Fix, verified here: append to `rust/corpus/lang/interpret_dynamic.rex`

```rexx
interpret "zork = 42"
say zork
```

I ran that exact program through both interpreters: stdout MATCH, stderr MATCH,
rc 0 MATCH. Alternatively, delete the clause from the header -- but then the
`extra` path stays unwitnessed differentially, and that should be said out loud.

### Minor

**M1. `tests/loud.rs:178` -- "19 entries" describes a list of 23.**
`INSTRUCTION_WITNESSES` has 23 rows (`Call` expands to four, `Signal` to two); 19 is
the number of `Owner::Phase` *arms* in `INSTRUCTION_TAGS`. The diff edited this line
(20 -> 19) and left the category error in place; the new comment at `:492` gets it
right ("19 coarse phase-owned tags ... so 23 expected witness tags"). Fix: make
`:178` say "23 entries, one per `Owner::Phase` arm after `Call`/`Signal` expand (19
arms)".

**M2. `tests/corpus.rs:26-31` -- "the two tasks still to land" is now false**, and
"26" is not the subset size (30). No commit anchor, unlike `:476`. The diff rewrote
the paragraph immediately above (`:17-24`). Fix: date the sentence the way `:476` is
dated, or drop the forward reference.

**M3. `run.rs:2770` and `:2778` -- `record_leave_failure(&origin)` is a provable
no-op here.** `origin.site` is `clause_site(source, instruction)` and `source` is
`None` for everything inside a fragment (`LeaveOrigin.site`'s doc, `run.rs:176-179`),
so the function returns without doing anything -- which the new doc at `:2714-2721`
states outright. Either delete the two calls, or add one line saying they are kept
so the two boundaries stay identical if a fragment ever gets a source. As written a
reader has to reconcile a call with a comment saying it does nothing.

**M4. `run.rs:2768-2782` duplicates `run.rs:485-500` almost verbatim.** The two
`Flow::Leave`/`Flow::Iterate` arms differ only in `fragment.symbols` vs
`code.symbols` and `Err(...)` vs `return Err(...)` -- sixteen lines. The comment
calls the duplication deliberate and "the same event happening at a different
boundary", which is an argument for sharing rather than against: nothing makes the
two drift detectably. Fix: extract
`fn exhausted_leave_search(&mut self, flow: Flow, symbols: &SymbolTable) -> Result<Flow, Failure>`
and call it from both.

**M5. `run.rs`, doc on `a_fragments_named_leave_is_resolved_against_the_fragments_own_table`
-- the mutation-kill instruction cannot be carried out as written.** It says
"resolve through the enclosing `code.symbols` instead"; `run_fragment(&mut self,
text: Vec<u8>)` has no `code` in scope. The test is genuinely live (I killed it with
an equivalent mutation, `raised_leave_no_match(b"BAR")`), so the defect is in the
note. Fix: name a mutation that compiles, e.g. "replace
`fragment.symbols.name(id)` with a fixed byte string".

**M6. `src/lib.rs:1443` -- "Step 2, and the reason the spike exists in the shape it
does."** Present tense about an entry point this same commit deletes. The brief
required the doc be kept verbatim, and the block comment above the tests
(`:1414-1431`) explains the move, so the fix is one bracketed note on that line, not
a rewrite. Flagged because the project's rule is that a false comment is corrected
rather than left standing, and "keep it verbatim" and "keep it true" collided here
without being reconciled.

**M7. `src/lib.rs:305-309` -- the `Outcome::collections` doc rewrite is
ungrammatical and mis-wrapped.** "Always `0` under `run_program`, which does not
enable Task 16's stress mode and nothing else in this crate calls `collect` at all"
-- the relative clause and the following independent clause are spliced, and the
paragraph now has a short orphan line. Fix: re-wrap and split the sentence.

**M8. `tests/coverage.rs:513-526` -- nothing pins `phase-4b.txt`'s line list.**
`EXPECTED_SUBSET` correctly stays `phase-4a.txt`-only as the brief instructs, but
the brief did not forbid a *sibling* pin, and its rationale ("a line cannot be
silently dropped ... or silently added") now applies to a file with no pin at all --
and will apply to every future phase file. The coverage test catches a dropped line
only when it is the sole witness of an in-scope variant. Fix: an `EXPECTED_SUBSET_4B`
beside it, or make `phase_4a_subset_matches_the_committed_list` table-driven over
(file, expected list) pairs.

**M9. `src/lib.rs` -- `interpret_binds_a_name_the_enclosing_body_never_mentions`
duplicates bullet 2 of `a_fragment_shares_the_enclosing_frames_variable_pool`.**
Both assert that a fragment binds a name the enclosing body never mentions and a
later, separate fragment reads it back. The brief mandated the new test (Step 4), so
this is not a defect -- but its doc ("the one property `INTERPRET` has that no other
instruction does") reads as though it were the only coverage of that property, and
does not acknowledge the overlap. Fix: one clause saying the older test asserts the
same property inside a longer transcript, and this one isolates it.

**M10. `run.rs:1400` -- `let temps_at_entry = self.roots.temps_len();` sits outside
the `debug_assert!`.** The comment's "absent in release" is true of the assertion and
true of the `let` only by dead-code elimination. Trivial; noted only because the
comment makes a cost claim.

**M11. Report §5 -- "852 tests pass, 0 fail, 1 ignored" for `cargo test --workspace`.**
Measured: 852 passed, 0 failed, **4** ignored. The other three are `rexx-num`'s
multi-gigabyte `--ignored` tests, pre-existing and unrelated; "1 ignored" is the
`-p rexx-exec` figure. Report-only, nothing in the tree is wrong.

---

## Things checked and found correct

* Every comment the diff adds or changes that makes a *measurable* claim was
  measured. `run_activation`'s corrected comment (`run.rs:445-460`) -- verified:
  `interpret "say 2 & 1"` matches on stdout, rc 222, both error numbers and both
  `Error ...` lines, differing only by the innermost echo; nesting two deep costs
  two echo lines, matching "one echo per nesting level". `Loud::parse`'s new doc
  (`lib.rs:474-490`) -- verified: `interpret "do forever then"` gives the oracle
  27.901 rc 229 with a two-line echo and this crate
  `rexx-exec: INTERPRET text did not parse: 27.901: Invalid DO or LOOP syntax.` at
  rc 120. `spike.rs`'s new fixture -- verified: `q~append(1)` gives exactly
  `rexx-exec: a message send is not implemented (Phase 5)\n`, and
  `ExprKind::Message`/`InstructionKind::Message` are both `Owner::Phase("Phase 5")`,
  so no task in this plan can implement it out from under the test.
* The unreachability claim in the new `Interpret` arm ("`Flow::Leave`/`Iterate` can
  no longer reach here at all") is **true**: `run_fragment` converts both to `Err`
  before returning, and `other => Ok(other)` cannot produce them.
* `run_program_collect_every_alloc`'s new doc claim "this is now the crate's only
  hidden entry point" is true -- exactly one `#[doc(hidden)]` in `src/`.
* The three moved doc comments are byte-identical to their `38fed3a1` originals
  (checked mechanically, not by eye).
* No live reference to `run_program_interpret_spike` or `interpret_spike` remains;
  all remaining mentions are historical prose in comments and `docs/`.
* Seven extra fragment shapes agree with the oracle on all three channels: `IF` in a
  fragment, `SELECT` in a fragment, a fragment inside a `DO`, a nested `INTERPRET`,
  `interpret "exit 5"` (rc 5 both ways), an empty fragment, and `iterate` inside a
  fragment inside a loop (28.4, the known echo gap only).
