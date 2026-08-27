# Fix round -- Phase 5a gate close, findings F1 to F5, plus F6 by ruling

Applied at `743dd4109`. Two commits: `fca800c98` for F1 to F5, and
`c3491b2e6` for F6, which was raised here as an open item and ruled in scope.
Every change is prose or a printed row label; no behaviour changed.

Every finding's fix is a deletion. Where a deletion left a sentence saying
less, that is where it was left -- nothing was substituted for a deleted
claim except in the two places recorded under "New claims" below.

---

## F1 (medium) -- `method_scope.rex`'s `~defineMethods` bullet

**Re-measured before deleting**, because the deletion turns on the review's
claim being right rather than on the review saying it. Oracle, from a fresh
empty directory, absolute path:

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  timeout -s KILL 10 /home/moritz/dev/repos/ooRexx/build/bin/rexx $P/dm.rex )
```

```rexx
m = .methods~z
say 'a' m~scope
.k~defineMethods(.methods)
say 'b' m~scope
say 'c' .methods~z~scope
::METHOD z
  return 'z'
::CLASS k
```

rc 0, empty stderr:

```
a The NIL object
b The K class
c The K class
```

A scope-less object handed to `~defineMethods` is filled **in place**. The
review is right.

**Deleted**: `which copies for the reason its two newScope calls give` and
`, leaving the object it was handed alone`.
**Remains**: the bullet and its citation --
`~defineMethods (createMethodDictionary at classes/ClassClass.cpp:1265 and
replaceMethods at MethodDictionary.cpp:233);`.

The two bullets above it still carry the `newScope` conditional for `~define`
(`fills its scope in place` / `which copies`), which is where that mechanism
belongs and where it is stated correctly.

**The row label was renamed**, `define-methods-copies` ->
`define-methods-does-not-rescope`. Taken because the file had to be edited and
its `sourceline_oracle` expectation regenerated for the comment fix anyway, so
the rename's stated cost -- regenerate, and re-verify the row against the
oracle on both engines -- was already being paid in full. See "New claims".

Nothing outside the corpus file and its expectation named the old label:

```
/bin/grep -rn 'define-methods-copies' .   ->  corpus/lang/method_scope.rex,
   crates/rexx-parse/tests/sourceline_oracle/method_scope.txt, and three
   lines of this plan's own SDD records, which are history and were not edited
```

## F2 (medium) -- the task-number citations

**Ruling taken as given: deleted, not renumbered.** The substance of every
`authority` and `control` field is kept; only the ownership pointer goes. No
new citation was invented and no attempt was made to work out which task each
line meant.

```
/bin/grep -ac 'Task [0-9]' crates/rexx-exec/tests/gate_table_c.rs   ->  0
/bin/grep -ac 'Task [0-9]' crates/rexx-exec/tests/gate_table_d.rs   ->  0
```

`grep` on this file misses a citation split across a `\`-continued string
line, so the count above was checked a second way, over the whole file
collapsed to one line with comment markers stripped, case-insensitively:

```
tr '\n' ' ' < FILE | tr -s ' ' | /bin/grep -aoi '.\{0,60\}task[ ]*[0-9].\{0,60\}'
```

That is what found the one in `gate_table_d.rs`, which the line-wise grep also
saw, and it reports nothing in either file now.

What was deleted, field by field: the trailing `-- Task N` from `typcla`,
`xmixin`, `usingcl`, `xscope`, `xmeths`, `chsrod`, `pubpri`, `concurr`,
`methna` and `chi`'s controls and from `xcremet`'s; the whole
task-assignment tail from `xmetac`'s control (`which is where this probe's
~id and ~class land; ::CLASS ... METACLASS itself installs from Task 8`);
`which are Task 18's and Task 15's` from `xcremet`'s authority; `and which
are Task 21's and Task 9's` from `methna`'s; `which is Task 9's protocol over
Task 21's registry` and `-- Task 9, and again for the deferred classes at
Task 21` from `classmeth`'s; `Task 9's` from `chi`'s authority; `so **Task
12** owes it and carries it in its own "Done when"` from `unkno`'s control;
`**Task 14** owes both and carries them in its own "Done when"` from
`reqstr`'s; and `and the plan makes Task 9 the task that moves it and Task 21
the one that finishes it for the deferred classes` from `WIRING_PHASE`'s doc.

Every control still names the change that would falsify its row, which is the
whole of what a `control` field is for now that the rows gate.

**Three sentences elsewhere in `gate_table_c.rs` described the control field
as naming its task, and my own deletion is what made them false.** The review
did not name them; they are the shape it warns about, arriving from the
correction rather than surviving it. All three fixed in the same commit:

* `Concept`'s doc: `So each control names the change that would falsify the
  row and the task that can first run it, which is the task where the row
  first reads agree.` -> `So each control names the change that would falsify
  the row.` The following sentence lost `and are carried in the "Done when" of
  the tasks that owe them`, leaving `Two of them are D50's required controls.`
* The `control` field's own doc: `The change that would redden the row, and
  who can first run it.` -> `The change that would redden the row.`
* The report header the test prints: `the negative control each concept row
  carries, and who can first run it:` -> `the negative control each concept
  row carries:`. That header is printed, not compared against a committed
  file, so changing it changes no expectation.

`gate_table_d.rs`'s one citation is the only place a word was substituted
rather than deleted, and it is recorded under "New claims".

## F3 (low) -- the two "no table C row exercises the scope question" sentences

**`gate_table_c.rs`: the whole "What this table cannot see" bullet is deleted**,
not only its named clause. The bullet's headline -- `**Whether a method is
defined at a class's own scope.**` -- is the same false claim as the clause
under it: `xscope`'s first two lines read `Method~scope`, and its third,
`.sub~method("BASEONLY")`, is the inherited-name discrimination. A disclaimer
whose subject the table does see is false at the headline, so trimming the
clause and leaving the headline would have been the exact failure this fix
round exists to avoid.

Nothing true was lost that is not asserted elsewhere: the bullet's measurement
(`.Array~hasMethod("ID")` is 1 while `.Array~method("ID")` raises) is
`class_method_own_dictionary.rex`'s rows 8 and 71, executed against the oracle
on every corpus run. That is the medium `rust/CLAUDE.md` asks for -- assert it
or delete it -- and it is already asserted.

**`class_method_own_dictionary.rex`**: deleted `There is no table C row for it
-- nothing in the documentation supplies an expected answer for the scope
question -- so this program and class_method_class_side_raises.rex beside it
are the whole of the protection.`

**One word in the neighbourhood, which the review did not name.** The
surviving sentence opened `This is the guard against a build that flattens
every scope onto one class`. With `xscope` gating the same property, `the
guard` carries the same exclusivity the deleted sentence spelled out, so it
reads `a guard` now. That is a weakening, not a substitution.

The paragraph further down that opens `The pair is what pins the rule ...` was
read and left alone: its own colon defines the pair it means -- `.Array~hasMethod
is 1 for names this raises for` -- which is the refusal-and-adjacent-success
pair inside this one program, not the two-program pair the deleted sentence
named.

## F4 (low) -- `rexxinfo_entry.rex`'s cardinality claim

Deleted `the one` from line 1: `.RexxInfo: an .environment entry that is a
pre-built instance rather than a class object.` The paragraph below it, which
carries the property the file actually means (`EndSpecialClassDefinition`
routing the class to `addToSystem` while only the instance is
`addToEnvironment`'d), is untouched and was already correct.

**The weaker instance in `environment.rs` is deleted too.** The review listed
it as ambiguous rather than wrong, only so it would not be rediscovered;
deleting the ambiguity costs nothing and removes the rediscovery. `which is
why the entry renders as a RexxInfo and answers ~class~id RexxInfo where every
other entry built from a class renders as The X class` ->
`... and answers ~class~id RexxInfo`. The two measured sentences after it
(`.RexxInfo~isA(.Class)` is `0`, `.RexxInfo~class~superClass` is
`The Object class`) are untouched, and they are what the comment is for.

## F5 (low) -- `methna`'s control attributes lookup-upcasing to the wrong function

**Confirmed at the source before deleting.** `crates/rexx-classes/src/method_dict.rs:160`-`:164`:

```rust
pub fn replace_method(&mut self, name: &str, scope: ObjRef, method: MethodId) {
    let key = name.to_ascii_uppercase();
    self.entries
        .insert(key, vec![MethodSlot::Defined { scope, method }]);
}
```

It upcases and inserts. It performs no lookup, so it cannot upcase on one.

**Deleted**: `, which upcases every key on insert and on lookup`.
**Remains**: `Both sites have to go together, measured: dispatch.rs's own
method_name_pair, and MethodDict::replace_method. Either one alone leaves this
row agree and the corpus untouched` -- which is the measured part, and is
Task 6's run.

**A second deletion in the same sentence, which the review did not name.**
It read `Both sites that upcase have to go together`. That is a universal
quantifier over upcasing sites, and F5's own measurement falsifies it: the
lookups upcase in their own functions. `that upcase` is deleted; `Both sites`
now points forward at the two the colon names, and asserts nothing about how
many other functions upcase.

---

## New claims

Two, and both are measured.

**1. The row label `define-methods-does-not-rescope`.** It is printed by the
program, so it is in the oracle's stdout and in both engines'. Measured on the
oracle, rc 163 (the file's own untrapped last send):

```
define-methods           K4
define-methods-does-not-rescope   K2
```

`m` was given `k2`'s scope two rows earlier by `.k2~define('Y', m)`, and it
still reads `K2` after `.k4~defineMethods(.methods)`. So what the row
witnesses is that `~defineMethods` did not rescope an already-scoped object --
which is what the new label says, and is not what `copies` said.

**2. `gate_table_d.rs`: `Task 22 drew the boundary` -> `The plan drew the
boundary`.** This is the one substitution rather than a deletion, because the
sentence's subject could not simply be removed. `The plan` is resolvable: the
bullet list this sentence sits in opens with
`docs/superpowers/plans/2026-08-17-phase-5a.md`'s handover section hands ...`
and continues `The same plan puts DELEGATE in 5b`, so `the plan` already has
an antecedent naming a path. Checked in that document, which says of its own
Task 22 `which moved ::METHOD ... EXTERNAL 'LIBRARY REXX name' into 5a and
left the ::ATTRIBUTE spelling of the same LIBRARY REXX form behind`, and
elsewhere `That is the LIBRARY REXX spelling D37 moved into 5a for ::METHOD`.
The claim the sentence now makes is the plan's own.

## F6 (added by ruling) -- `class_method_class_side_raises.rex`'s task citation

Raised as "deliberately not fixed, reported for a ruling"; ruled in scope and
fixed at `c3491b2e6`. `rust/CLAUDE.md:107` reaches it, and the file gates, so a
reader arrives at this comment while reproducing a failure.

**Deleted**: `The frame line is Task 6's, and`.
**Remains**: the whole of the claim the clause carried --
`This refusal is owed a frame line by the rule blame_native_method states: the
oracle reached METHOD's body by an explicit message send. rc 159.`

Deleting the first clause orphaned the pronoun in `this refusal is owed **it**`,
so `it` became its own antecedent, `a frame line`. That is the one word that
changed and it is a recovery of the deleted subject, not a new one; the
indefinite article is weaker than the definite pronoun, which is the direction
this round wants. `blame_native_method` is at
`crates/rexx-exec/src/dispatch.rs:2534`, so the citation that remains resolves.

**One of the ruling's three reasons rests on a false premise, and the
correction is worth recording.** The reason was that both gate tables are at
zero, so "leaving exactly one behind" would read as deliberate. Measured over
the whole corpus:

```
/bin/grep -ral 'Task [0-9]' corpus/lang/*.rex | wc -l    ->  107
ls corpus/lang/*.rex | wc -l                             ->  268
```

Checked a second way for citations split across two comment lines, by
collapsing each file's comment prose to one line before matching: the same
107, so nothing hides beyond that set. This file was one of 107, not the last
one standing, and 106 remain. **The other two reasons are untouched by that**
-- `CLAUDE.md:107` reaches every one of them, and this file gates -- so the fix
stands on those. What the count changes is what the *rest* of the corpus looks
like: `corpus/lang` is not a tree with one stray citation in it, and a decision
about the other 106 is a decision about a body of work, not a tidy-up. Not
taken here, and not widened into: this round's ruling was one comment in one
file.

## Deliberately not fixed

* **The other 106 `corpus/lang/*.rex` programs carrying a task citation.**
  Measured above. Out of scope by the ruling's own terms, and named here so
  the size of the remaining question is on the record rather than rediscovered.
* **`unkno`'s `One of D50's two required controls` and `Concept`'s `Two of
  them are D50's required controls`.** Both name a set's size, both predate
  this round, and neither is a task citation. Left rather than widened into,
  and confirmed as the right call by the ruling.
* **`Two of them are D50's required controls`** kept the sentence but lost
  `and are carried in the "Done when" of the tasks that owe them`, which is
  recorded under F2 above rather than here, because my own deletion is what
  unmoored it.

## Verification

**The four edited corpus programs, against the oracle, on both engines.**
Three descriptors read separately, never `2>&1`; probes from a freshly created
empty directory with absolute paths. Oracle wrapper and crate wrapper exactly
as the constraints spell them. `command -v memcap` reported present.

| program | oracle rc | `REXX_ENGINE=ir` | `REXX_ENGINE=tree-walker` |
| --- | --- | --- | --- |
| `method_scope.rex` | 163 | stdout, stderr, status identical | identical |
| `rexxinfo_entry.rex` | 159 | identical | identical |
| `class_method_own_dictionary.rex` | 159 | identical | identical |
| `class_method_class_side_raises.rex` (F6, at `c3491b2e6`) | 159 | identical | identical |

The binary was rebuilt after the `environment.rs` edit and before these runs
(`cargo build --release --bin rexx-run`, `Finished` at 28.77s, so `rexx-exec`
did recompile). No file was restored with `cp -p`, so the stale-mtime hazard
did not arise.

`find crates -name '*.rs' -newer target/release/rexx-run` names
`gate_table_c.rs`, because three of its comments were edited after that build.
**That does not stale the binary**: it is a test file, compiled into the gate
harness and not into `rexx-run`, so no differential above ran against out of
date code -- and both gate lanes compile everything from the commit in their
own worktrees regardless.

**The regenerated `sourceline_oracle` expectations, with a control, run twice.**
The driver in `sourceline_oracle.rs`'s module comment was run against the
edited programs **and five untouched ones** -- `address_env`, `arith_digits`,
`call_return`, `no_trailing_newline`, `trace_numeric_request`, the last two
chosen because they are the files the module comment names as the ones the
driver's two paths part over. All five reproduced their committed
expectations **byte for byte** (`cmp -s`), which is what says the driver is
faithful and that the regenerated files differ only by my edits. The diff for
each edited file is exactly the source lines I changed, and nothing else.

**The control was re-run from scratch for F6 rather than assumed to still
hold**, and the same five reproduced identically the second time.

The module comment's own driver invocation reads `build/bin/rexx` relative to
the repository root. **That path in this worktree is a different interpreter**
-- `build/bin/rexx` here is 16,496 bytes, dated Jul 27, against the 62,600
bytes of `/home/moritz/dev/repos/ooRexx/build/bin/rexx`. The constraints'
oracle was used instead, checked first: `parse version` answers
`REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026`.

**Local pre-commit checks**, from `rust/`, each status read from an unpiped run:

* `cargo fmt --all --check` -> 0
* `cargo clippy --workspace --all-targets -- -D warnings` -> 0
* `cargo test -p rexx-parse --test sourceline_oracle` -> 0,
  `1 passed; 0 failed`

**The five gates, run twice** -- once at `fca800c98` and again at `c3491b2e6`
-- with `scratchpad/controller-gates.sh <sha> <logdir>`. Every status is read
from its own `.rc` file and not from any completion notification.

**Run 1, `fca800c98`.** `checked-out-sha` reads
`fca800c98e23d7803354a8492bc0767df8e0e4ae` in both lanes.

* `g1.rc` (fmt) -> **0**
* `g2.rc` (clippy) -> **0**
* `g4.rc` (release corpus gate) -> **0**
* `g5.rc` (debug corpus gate) -> **0**

`elapsed.seconds` reads 3874.

**Run 2, `c3491b2e6`.** `checked-out-sha` and `checked-out-sha-b` both read
`c3491b2e63e09b11f4de5d372b31f6ad8a1ecf4a`.

* `g1.rc` (fmt) -> **0**
* `g2.rc` (clippy) -> **0**
* `g4.rc` (release corpus gate) -> **0**
* `g5.rc` (debug corpus gate) -> **0**

`elapsed.seconds` reads 3861. G3 is deliberately omitted by the runner, which
G4 strictly covers.

**A zero status was not read as the whole answer**, in either run, because a
run that never reached the harnesses would also exit 0. All four logs were
read for what actually ran, and the two runs agree line for line on every
figure below:

* `/bin/grep -a 'test result: FAILED'` prints nothing in any of the four logs.
* Summing the `test result: ok` lines: G4 is 1905 passed / 0 failed and G5 is
  1906 passed / 0 failed, in **both** runs. The one extra in the debug lane is
  the `debug_assertions`-gated case the release profile compiles out, which is
  why that gate is run beside the release one.
* Each of the four logs contains all three of `Running tests/corpus.rs`,
  `Running tests/gate_table_c.rs` and `Running tests/sourceline_oracle.rs` --
  the oracle differential, the table this round edited, and the expectations
  it regenerated. So the harnesses that could see these changes were reached
  rather than skipped past by an earlier failure.

That the totals are identical across the two runs is itself the check that
`c3491b2e6` added no test and removed none: F6 edited a comment and the
expectation that records it, and a change of either count would have said
otherwise.
