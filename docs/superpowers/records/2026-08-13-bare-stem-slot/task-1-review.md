# Task 1 review -- a bare stem's slot

Range `2b7e78450..866d07d1b`, read from `.superpowers/sdd/2026-08-13-bare-stem-slot/review-2b7e78450..866d07d1b.diff` and from the working tree.
No file was modified, no benchmark was run, no cargo command was run.

## Spec compliance

**Compliant, with the Step 2 route changed and the change argued in the plan file itself.**

* Step 1: the workload was written before the fix and each axis's own spread was recorded first.
  The decision to keep both programs out of `rust/bench-programs/` is argued rather than taken quietly, which is what the step asked for.
* Step 2: taken by the entry route rather than the parameter route the step describes.
  The deviation is measured, is recorded in the plan file's own Status block and in entry 32, and is the more valuable of the task's results.
* Step 3: answered by running, with a control.
  Not verifiable from the tree.
* Step 4: partially as specified; see Important 4 below.
  The accessor probe is `self.activation().plan.slot_of(name)`, which is the plan's name map alone rather than the full three-source resolution the step names.
  That substitution is stricter and not weaker for a `Some` slot: a slot reaches `CompoundName::stem_at` only because `Plan::slot_for` put that name in the plan's own map, so a precomputed slot that the plan cannot reproduce is already the mismatch worth failing on.
  What it does not cover is `read_stem_at`, which is the accessor the task's new fourth site feeds.
* Step 5: done, including the finding that the sampled share cannot resolve what is left and the direct `slot_of` count that replaces it.
* Step 6: entry 32 is a pure append.
  `git diff --numstat` reads `122 0`, the hunk is `@@ -3014,3 +3014,125 @@`, and the entry attributes rather than restates: its wrapper paragraph cites entry 27's reason and entry 28's way, both of which say what it says they say, and its no-attribution paragraph applies entry 31's rule correctly.
  Nothing in it contradicts entries 26 to 31.

The fourth site, the controlled loop's own re-test read at `rust/crates/rexx-exec/src/run.rs:6619`, is beyond what the plan named.
It is in the same loop as site 3, is measured, and halves the win if left out, so it is not scope creep.

## Where the wrong-slot risk was chased, and what I found

The first-priority question is whether `stem_at` from `Plan::bind` is ever a slot that is not the bare stem's own.
I could not construct a program where it is, and the invariant is held by the type system at one end and by one hash key at the other.

* Every `ExprKind::Stem(id)` carries a stem-shaped spelling.
  `rexx-parse/src/expr.rs:1275` builds that node only from `SymbolClass::Stem`, and the scanner assigns that class at `rexx-parse/src/scanner.rs:871` by exactly the predicate `run.rs:8589`'s `shape_of` uses (`dot_count == 1 && last byte is '.'`).
  So the arm that reads `stem_at` and the `bind` arm that writes it agree about the shape by construction, and `A..`, `A.B.` and `.` cannot reach it.
* Both routes that can fill `stem_at` for a stem-shaped name hash the identical bytes.
  `Plan::bind` (`plan.rs:759`) stores `slot_for(name)`; `note_compound_name` (`plan.rs:617`) stores `slot_for(&entry.stem)`, and `compound_parts` (`rexx-parse/src/ast.rs:574`) returns the whole spelling as the stem for a stem-shaped name.
  `bind` uses `get_or_insert_with` and `note_compound_name` overwrites, so either order produces the same number.
  `drop zs.` before or after `zs. = 'v'` therefore both land on the slot `by_symbol` holds for the id.
* `PROCEDURE EXPOSE` of a bare stem cannot move it.
  `exec_procedure` resolves each exposed name through `Interp::slot_of` (`run.rs:2389`) and aliases the callee's frame at that same index, and only names the plan does not hold go into `extra` (`run.rs:2432`).
  A plan-bound stem is therefore aliased at the slot `stem_at` names.
* An `INTERPRET`ed or `VALUE`-created stem cannot route a fragment's slot.
  `Code::plan` is `None` for a fragment (`run.rs:7382`), so `Code::compound` answers `None` and the write falls back to resolving the name.
* `DROP` with an indirect list and `USE ARG` never carry a slot.
  Both reach `assign_by_name`/`drop_by_name` from a run-time byte string, and `assign_by_name`'s stem arm (`run.rs:3114`) calls `stem_assign`, which is now the `at: None` wrapper.
* The activation and the entry come from one plan.
  `run_activation` builds `Code` from `Rc::clone(&self.activation().plan)` (`run.rs:1125`, `run.rs:1139`), so `code.compound(id)` and `Interp::slot_of` read the same map.

A wrong slot would also not be silent: the implementer's P2 mutation with the tripwire deleted still reddens fourteen test names, so behaviour catches it as well as the assertion does.

## Do both engines still take one path

Yes, and the compiled side is untouched.

* `write_slot` (`ir/compile.rs:1283`) still answers only for `ExprKind::Variable`, `push_read` (`ir/compile.rs:1293`) is unchanged, and the `Op::Store` arm with its tripwire (`ir/drive.rs:968` to `ir/drive.rs:1003`) is unchanged.
* Both engines reach the bare stem write through `Interp::assign_expr_target` with `at: None`, and the new slot is read inside that shared arm from `Code::compound`, which is engine-independent state.
* A controlled loop is not compiled to ops, so the re-test at `run.rs:6619` is one implementation for both engines too.
* The one asymmetry left is a read: `ir/compile.rs:1114` gives a compiled bare stem read the plan's slot while `eval.rs:481` passes `None`.
  That is a work difference and not an answer difference, since `push_read` reads `by_symbol` and `slot_of` reads `names`, which `Plan::bind` sets together.
  The implementer names it as left undone, correctly.

## Strengths

* The rejected route was measured and rejected instead of shipped, and the axes that caught it were written before the fix existed.
  `emptyloop` and `varlookup` moving by exactly two instructions a pass is a result the plan did not ask for and is worth more than the optimisation.
* The design that shipped leaves `control_slot`, `write_slot`, `Op::Store` and the `drive.rs` tripwire untouched, which I verified line by line rather than from the report.
* `plan::tests::bind_keeps_a_stem_shaped_names_own_slot_as_its_stems` (`plan.rs:1493`) is a real guard on the slot's source: it pins the stem-shaped and compound-shaped answers side by side, and it fails under both the "record nothing" and the "record a wrong number" mutations.
* The mutation battery differences the tripwire in and out (P2 against P2', G against G') rather than importing the earlier task's figure, which is the only way that number means anything.
* P3 and P4 are reported as catching nothing rather than presented as coverage.
* `Code::compound` rather than `Code::stem` at `run.rs:3062` is the right call and the comment's reason is true: `compound_parts` collects a `Vec<Tail>` that `Code::stem`'s fallback discards.

## Issues

### Critical

None.

### Important

1. **`run.rs:3018`, `assign_expr_target`'s doc: "The three arms write three different slots, which is why one number could not serve them."**
   False, and falsified by its own next two sentences, which say a simple variable writes the symbol's own slot and "A bare stem writes the symbol's own slot too".
   It restores in a new sentence the misconception this whole task exists to remove, and it is the sentence a later reader consults before deciding whether `write_slot` may answer for a stem.
   The true reason one number does not serve them is the measured cost at `bind_control`'s call site plus the compound's different name.

2. **`run.rs:8556`, `control_slot`'s doc: "the inlining that turns on adds 2 instructions to every pass of every controlled loop".**
   The mechanism is asserted where the task established only the effect.
   Entry 32 says the opposite in the same commit range: "Why the generated code changes was not established; the effect was."
   Drop the clause or replace it with the partial-revert evidence, which is what actually supports the claim.

3. **`ir/drive/tests.rs:677`: "`Plan::note_compound_name` registers the stem and each variable tail piece by name and binds the compound's own id to no slot at all, so the widened arm has nothing to answer with."**
   False.
   `Plan::bind` puts compound-shaped names into `by_symbol` from `note_loop` (`plan.rs:494`, `plan.rs:505`, `plan.rs:516`, `plan.rs:519`) and from `note_parse` (`plan.rs:538`), so `do a.b = 1 to 2 ; end ; a.b = 5` gives `A.B` a `by_symbol` entry, and a widened `write_slot` would answer with it.
   The mutation is still unobservable, but for the reason the paragraph gives for `control_slot` and not for the reason it gives here: `assign_expr_target`'s compound arm ignores the parameter.
   Under the IR engine a test with that shape would in fact redden, through `Op::Store`'s own tripwire at `ir/drive.rs:994`, so "neither reddens anything else either" is a statement about the suite's contents rather than about the code.

4. **`stem.rs:274`, `stem_slot`'s doc: "Where every `_at` accessor in this module decides it".**
   False, and it was true before the edit, which said "the `_at` accessors below".
   `read_by_name_at` (`stem.rs:200`) and `read_stem_at` (`stem.rs:249`) each write the `match at` out themselves and so carry no tripwire.
   This matters beyond the wording: the task's fourth site feeds a precomputed slot into `read_stem_at` at `run.rs:6619`, so the Step 4 accessor probe does not cover the read half of the new flow.
   It is covered only incidentally, because the write half of the same pass takes the identical slot from the identical expression and does go through `stem_slot`.

5. **`ir/compile.rs:1270`, `write_slot`'s doc: "a bare stem through `stem_assign`, which resolves the same spelling `Plan::bind` bound the symbol's id to".**
   That is the path this task removed.
   A bare stem assignment target now goes through `stem_assign_at` with the entry's slot and resolves the spelling only when there is no plan.
   The same doc also no longer says why a bare stem is `UNRESOLVED` here, having just said that it does write the symbol's own slot, so the function whose whole job is that decision documents everything about it except the decision.

6. **Entry 32's drift paragraph: "still moved, by up to -38,000,321 instructions, thousands of times their spans".**
   The multiple holds for `varlookup` and for nothing else in that group.
   `compound` moved -24,238,923 against spans of 1,599,903 and 2,598,819, `alloc4c` -6,009,303 against 117,496 and 92,743, and `strings` -15,000,396 against spans the entry gives as 1,025 and 1,354.
   `strings` is the one that matters: the task's own Step 1 base measurement recorded a same-binary span of 36,000,462 on that axis, so its move is smaller than a span this task measured, and the entry carries neither that span nor the caveat.
   The entry's spread table is also captioned "measured before any of the small figures were read" while its numbers are the Step 5 spans, not the Step 1 ones.
   Nothing in the disposition rests on the `strings` row, so this is a correction to make in a later entry rather than a reason to redo the measurement.

### Minor

* `run.rs:3063` shadows the `at` parameter inside the stem arm, so a caller-supplied slot is discarded silently.
  A `debug_assert!(at.is_none())` before the shadowing would pin the doc's "Only the first arm below reads it" and would fail loudly the day `write_slot` is widened on the tree-walker side, where no tripwire watches.
* `stem.rs:241`, `read_stem_at`'s doc still says the slot "is what `crate::ir::Op::Load` carries and what a read reached from `eval.rs` never has".
  The controlled loop's re-test is now a second source and the doc does not name it.
* "against same-binary spans under 1,500", in `control_slot`'s doc and in the plan file's Status block, is contradicted by the task's own `varlookup` spans of 1,508 and 1,576.
* The fourth site's `867,001,000` is a difference between two variants of the **rejected** design (`10,562,819,715` minus `9,695,819,133`, which is `867,000,582`, off by 418 from the figure quoted), and it is stated against the shipped design's 1.659 billion.
  Say which arm it came from, or restate it as a bound.
* Committed prose quantifies sets: entry 32's "Three sites still did it", "Two axes written for this", "Five axes execute no bare-stem operation at all".
  The vocabulary is inherited from the plan's own "The three sites", so the plan file is the place to break the habit.
* Report-only, and the committed entry gets it right: the report says "That is codegen drift, and I offer no attribution beyond 'not this change's semantics'" and, of `compound`, "moved -24 million. Drift."
  That is the two-sentence disagreement entry 31 exists to correct, reproduced in the artifact the next reader reaches for first.
* `run::tests::a_stem_control_inside_a_fragment_writes_the_enclosing_frames_slot` passes unchanged at base, since a fragment's `Code::plan` is `None` on both sides of the change.
  It is a guard against the design that was not shipped rather than coverage of the one that was, which its doc comment could say.
* Pre-existing, and outside this diff, but the task edited inside it: `push_read` has no doc comment of its own.
  The block at `ir/compile.rs:1247` opens by documenting a bare-symbol read and is attached to `write_slot`.

## What I could not verify from the tree

* Every `perf stat` and `perf record` figure, the partial reverts, and the byte-identity of the arms.
* Every mutation result, including the inverted tripwire's 38 failures and the P3/P4 zeroes.
* The instrumented `slot_of`, `stem_assign_at`/`read_stem_at` and `PARSE` target probes, and the oracle comparisons.
* The gate outputs, including that clippy's log names `rexx-exec`, which the controller is re-running.

## Assessment

**Spec compliance:** compliant, with Step 4 partial as described above.

**Task quality:** needs fixes.

The mechanism is sound and I could not find a program where the precomputed slot is the wrong one, but the doc comments the next task in this family will read misstate what the code does at four places, and one of them contradicts the record entry committed beside it.

---

# Fix round 1 re-review

Scope: `d2f46e144` and `851b18fce`.
Every finding below was checked against the working tree at `851b18fce` with `git status` clean, not against the diff or the report.
No benchmark was run and no cargo command was run.

## The six Important findings

**1. `assign_expr_target`'s "three different slots" -- addressed.**
The sentence is gone from the tree, and the replacement says what the code does: both of the first two arms write the slot their own symbol is bound to, `at` would be the right number for a bare stem and is withheld for a measured reason, and it would be the wrong number for a compound.
The claim is now enforced as well as stated: `run.rs:3073` and `run.rs:3104` each assert `at.is_none()`, both before anything shadows the parameter, and every caller passes a literal `None` for those shapes today (`run.rs:1449`, `run.rs:6890`, `run.rs:6901`, `parse_template.rs:679`) while the compiled path can only reach them through `write_slot`'s `UNRESOLVED`.
So the assertions are reachable, cannot fire on today's tree, and would fire under the successor task's own most likely first move.

**2. The asserted inlining mechanism -- addressed.**
`control_slot`'s doc now carries "Attributed by partial revert and not by reading the assembly" and "Why the generated code changes was not established", which is what entry 32 says.
The `bind_control` stem-arm comment now points at that doc rather than restating the figure.

**3. The `write_slot` widening claim -- addressed, and better than the finding asked for.**
The paragraph at `ir/drive/tests.rs:670` now separates the two widenings: `control_slot`'s is unobservable in the code, `write_slot`'s is unobservable only in this suite's contents, and it says so in those words.
Its mechanism claim checks out: `Plan::bind` inserts into `by_symbol` unconditionally, and `note_loop` (`plan.rs:494`, `plan.rs:505`, `plan.rs:516`, `plan.rs:519`) and `note_parse` (`plan.rs:538`) both reach it with spellings that may be compound-shaped, while a `PARSE` template target reaches the plan through `note`'s `ExprKind::Compound` arm into `note_compound_name` and so gets no `by_symbol` entry.
The run's four rows are consistent with that split, and the paragraph states the split rather than the aggregate.

**4. `stem_slot`'s scope and the coverage gap -- addressed.**
The doc now says "every `_at` accessor *below* this one", which matches the file: `read_by_name_at` (`stem.rs:200`) and `read_stem_at` (`stem.rs:249`) sit above it and resolve their own, the rest go through it.
The gap is closed rather than only described: `read_stem_at` carries the same comparison against the same map (`stem.rs:263` against `stem.rs:339`).
Its two supporting claims are true in the tree: `read_symbol`'s compound arm does carry the tripwire the compound-arm comment cites (`eval.rs:405`), and `join_tails` does carry the tail-piece one (`stem.rs:155`).

**5. `write_slot`'s doc -- addressed, and the `push_read` Minor with it.**
The stale `stem_assign` sentence is gone, the function now documents the decision it makes and why the entry rather than the op is the source, and the bare-symbol-read block has been moved down onto `push_read` where it belongs.

**6. Entry 32's drift paragraph -- addressed by entry 33.**
It follows entries 29 and 31: it corrects by appending, names the commit entry 32 is left at, withdraws the quantifier rather than the movement, carries the `strings` span of 36,000,462 with the interference caveat, states `strings` as a bound and not a difference, corrects the caption and lists the Step 1 spans the caption was reaching for.
Its arithmetic reproduces: 38,000,321/1,576 is 24,112, 15,000,396/1,354 is 11,079, 2,499,761/924 is 2,705, 6,009,303/117,496 is 51.
It also correctly restricts the group to the axes entry 32's accessor probe counted zero bare-stem operations on, which is where my own finding was loose: I put `compound` in that group, and entry 32 does not, since it counts one operation there.
Entry 33 did not adopt my error.

## New findings from this diff

* **Minor.** Entry 33, the paragraph after the multiples table: "those axes still moved by more than the instrument's resolution" covers `strings`, which the entry's own next section then withdraws as a bound against a span of 36,000,462.
  Scope that sentence to the axes it survives on, in a later entry.
* **Minor.** "So the quantifier holds for three of the five" is a fresh set count in committed record prose, in the round that removed set counts from the plan file.
  The table above it names the axes, so the count carries nothing.
* **Minor.** The plan file's set counts are not all gone: `2026-08-13-bare-stem-slot.md:26` still reads "The first two were named by the implementer; the third by its reviewer", in the section this round renamed from "The three sites".
  "Set counts in the plan file ... are gone" is therefore a completeness claim the file does not support.
* **Minor.** "undoing that one line and nothing else puts `emptyloop` back on base exactly", in `control_slot`'s doc and in the plan file, overstates an equality.
  The partial-revert table reads 27,150,813,214 against a base of 27,150,813,500, a difference of 286 and inside that axis's own span.
  "Indistinguishable from base" is what was measured, and this round is the one that made the surrounding sentence precise.
* **Nit, not a finding.** "A simple variable and a bare stem write the same slot" is the same topic-sentence-then-qualify construction that produced Important 1.
  It does not mislead, because the next sentence defines it as the slot each one's own symbol is bound to rather than contradicting it, but it is the construction to stop reaching for.

## Checks that came back clean

* The new `read_stem_at` tripwire cannot fire on an `INTERPRET`ed stem read: `run_fragment` runs every fragment with `BodyEngine::TreeWalker` (`run.rs:7432`), so no chunk ever carries a fragment-local slot into `Op::Load`, which is the shape the round-1 report found firing against `stem_slot`.
* Both new arm assertions and the read-side tripwire are in the tree, not only in the report: `run.rs:3075`, `run.rs:3106`, `stem.rs:263`.
* All six sentences the previous round flagged are absent from the tree, and all six replacements are present in it.

## My Minor findings

Landed: the arm assertion, `read_stem_at`'s doc, the span figure, the `867,001,000` restatement as a bound, the fragment test's doc, `push_read`'s doc.
Left with reasons: entry 32's set counts, which the append-only rule protects and entry 33 does not restate; and the report body's two-sentence disagreement about `compound`.

On the second: stating the correction in a later section of the same file is adequate here.
The report body opens the fix section with "Everything above this line is left as it was written; corrections to it are below", the correction names both sentences and says what the measurement supports instead, and the durable artifact -- the record -- never carried the defect.
The reservation is that a reader who stops at "Things I am not sure about" never reaches the pointer, and the body's own sentences still read as claims when quoted alone.

## Verdict

**Task quality: approved.**

Every Important is addressed in the committed tree, the two that were only prose are now backed by assertions that can fire, and the one measurement correction is recorded in the shape this project's record requires.
What is left is Minor and belongs in a later entry rather than another round on this task.
