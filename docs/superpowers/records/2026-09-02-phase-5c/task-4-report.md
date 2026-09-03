# Phase 5c Task 4 — the `>x` silent divergence

**BASE:** `b4adf8a1e`, tree clean. **Committed at `989c307e2`.** Everything below was measured on
this machine on 2026-09-03
against the pinned 5.3 oracle at `/home/moritz/dev/repos/ooRexx/build`, three descriptors read
separately, from a fresh empty directory per program, on `REXX_ENGINE=ir` and
`REXX_ENGINE=tree-walker` both.

---

## The one-paragraph version

**`>x` now answers a `VariableReference`.** The term used to evaluate to the referenced variable's
*value*, so `(>vr)~class~id` was `String` where the oracle answers `VariableReference` — rc 0 and
empty stderr on both sides, which is why no harness went red. The fix is a new object kind
(`Body::VarRef`) bound to the variable's storage, a new `RootSet` operation that moves a referenced
variable out of its frame into a cell so the reference can outlive the activation, five native
methods, and the removal of `Argument` — `USE ARG >` now reads the reference out of the argument's
*value*, which is what the oracle does. **Fifty-six edge probes and three controls were run against the
oracle on both engines**; every edge that is about the reference agrees byte for byte, and every one that does not
is a pre-existing gap this crate refuses loudly, each with a control run at BASE showing the same
refusal without a `>` anywhere in the program. Table C's 5c not-yet-agreeing count is **237 → 233**.

---

## What the oracle actually does, measured before anything was built

Task 3 measured row 1 and the row set's four `hasMethod` readbacks, and stopped there. Everything
below row 1 is this task's, and rows 11, 46, 47 and 51 each changed the implementation after it was
first written. Each row is one program run from a fresh directory; `crate` is this branch's answer.

| # | program | oracle | crate now |
|---|---|---|---|
| 1 | `vr = 5; o = >vr; say o~class~id` | `VariableReference` | same |
| 2 | `say >vr` | `5` | same |
| 3 | `o = >vr; say o` | `5` | same |
| 4 | `o~value` | `5` | same |
| 5 | `o~name` | `VR` | same |
| 6 | `o~value = 7; say vr` | `7` | same |
| 7 | `o = >vr` over an **unassigned** `vr`; `~name`, `~value` | `VR`, `VR` | same |
| 8 | `s. = 0; o = >s.; o~name` | `S.` | same |
| 10 | `call sub >vr` into a **plain** `use arg q`; `q~class~id` | `VariableReference` | same |
| 11 | `ref: procedure; v = 42; return >v` — read and write **after** the return | `42`, then `99` | same |
| 12 | `o + 1`, `o \|\| 'x'`, `o = 5`, `o == 5`, `o~length` | `6 5x 1 1 1` | same |
| 13 | `arg(1)~class~id` inside the callee | `VariableReference` | same |
| 14 | `~request('STRING')~class~id`, `~string`, `~objectName`, `~isA`, `~hasMethod('VALUE')` | `String 5 "a VariableReference" 1 1` | same |
| 15 | `vr = 'changed'` then `o~value`; `drop vr` then `o~value` | `changed`, `VR` | same |
| 17 | `(>vr)~class~id` with no intermediate variable | `VariableReference` | same |
| 20 | `o = >vr; p = >o; p~name; p~value~class~id` | `O`, `VariableReference` | same |
| 21 | `>v` on an **`EXPOSE`d** instance variable, read and written | `V`, `iv`, then `set` | same |
| 23 | `o = >vr; call sub o` into `use arg >q` | aliases | same |
| 27 | `trace i` over `zz = >pq` and `call sub >pq` | `>O>   ">" => "PQ"` | same |
| 32 | `>v` on a method's own local, carried out through `REPLY` | `LOC`, `first`, `written` | same |
| 35 | the `<` spelling throughout | identical to `>` | same |
| 40 | `t. = 0; p = >t.; p~value = 'replaced'; say t. t.9` | `replaced replaced` | same |
| 46 | `'p' \|\| >zz` and `'p' >zz` over a two-item array | `pan Array`, `p an Array` | same |
| 47 | `'p' >k` where `K` defines `makeString` **and** `string` | `p a KK` | same |
| 51 | `o~"VALUE="()` | rc 168, `88.901 Missing argument; argument VALUE is required.` | same |
| 54 | `o + 1`, `o * 2`, `o < 8` over a numeric referent, then the same after `DROP` | `8 14 1`, then 41 | same |
| 55 | `PARSE VAR o`, `PARSE VALUE o WITH`, `PARSE ARG` over a passed reference | `alpha`/`beta` on all three | same |

**Three of those changed the design after it was first written, and each was found by running:**

* **11 — a reference outlives its frame.** A `procedure` returning `>v` answers a reference whose
  `~value` reads `42` and whose `~value =` writes, after the frame is gone. A reference holding a
  frame slot position would name whatever the next activation put there. This is what
  `RootSet::promote` exists for.
* **46/47 — a reference's string value is the referent's `stringValue()`, not its string
  *conversion*.** `'p' >zz` over a two-item array is `p an Array` where `'p' zz` is `p one` and
  `two`; and over an instance whose class defines `makeString`, `'p' >k` is the default name where
  `'p' k` runs the method. `RexxObject::requestString` looks `MAKESTRING` up in the *receiver's own*
  behaviour rather than sending it, and a reference's behaviour holds no such name, so the
  conversion falls through to `VariableReference::stringValue`. **The first version of this fix
  chased the referent's conversion and was wrong on both rows.**
* **39 — `FORWARD ARGUMENTS (>v)` is 98.946 where `FORWARD ARGUMENTS (v)` is rc 0.** Same reason,
  on `requestArray`: the `MAKEARRAY` lookup fails on the reference's behaviour, so `requestArray`
  answers `.nil`. `~request('ARRAY')` **does** forward, because `VariableReference::request` sends
  `REQUEST` explicitly — the two routes disagree and the code now says so.

---

## What was committed

### `rexx-core`

* **`Body::VarRef(Box<VarRef>)`**, with `VarRef { name, home }` and
  `VarRefHome::{ Cell(SlotRef), Instance { owner, scope } }`. `Body::trace` gains its arm: the
  `Instance` home pushes the owning object and the scope, the `Cell` home pushes nothing because
  `RootSet::iter` roots cells directly.
* **`RootSet::promote`**, and the `cells` vector behind it. Promotion moves a variable's storage out
  of the frame arena into a cell that is never truncated, and leaves the frame slot redirected to
  it, so the variable and every reference to it share one piece of storage. It is **idempotent**,
  which is what two `>p` terms on one variable need, and it promotes the *resolved* position, so a
  reference to an exposed name promotes the storage it was exposed from.
* `SlotRef` gains a tag bit separating a frame position from a cell index, `resolve` walks to a
  fixed point rather than stepping once (promotion is the one operation that redirects a position
  other slots may already name), and the aliased path goes through one read/write pair that tests
  the tag. **The `alias_count == 0` path stays what it was**: `frame_slot` and `set_frame_slot` take
  it before any storage test, so a program with no `EXPOSE` and no `>name` reads a slot through the
  same add-and-index it did before. The first version of this change wrote
  `self.at(self.resolve(..))` for every read, which put one masked test on the hot path; LLVM cannot
  fold it away, because it cannot prove an arbitrary `frame.start + index` has the tag bit clear.
* `take_frame_aliases`/`put_frame_aliases`, for `REPLY`. A parked frame's values are copied out and
  its redirects go with the frame; without these a `>name` taken on a method's own local would leave
  the reference reading the cell and the resumed variable reading a copy. `frame_aliases`, which
  `park_reply` asserts on, now counts slot-target redirects only, because those are the ones a copy
  cannot carry.

**Nothing frees a cell.** A program taking a reference to a fresh local on each of many calls grows
that vector without bound. This is stated at the field and is the one cost of the design; the
alternative — a reference that refuses once its frame is gone — cannot be built here, because
`to_text` is infallible and is called from everywhere, so a dead reference would have to *render* as
something, and every candidate is a silent wrong answer.

### `rexx-exec`

* `eval_node`'s `VariableReference` arm builds the object through `Interp::variable_reference`,
  which resolves the home exactly where the old `eval_argument` did and promotes when it is a slot.
* **`Argument` is deleted.** `CallContext::arguments` is a `Vec<Option<ObjRef>>`, and
  `bind_use_target`'s alias branch reads the home out of the argument's *value*. That is not a
  simplification for its own sake: it is what makes row 23 work, which the old shape refused at
  88.928.
* `Primitive::VariableReference` and its `receiver_kind`/`receiver_behaviour`/`native_class` arms,
  wired to the registry's existing `VariableReference` class — whose behaviour already hid `=`, `==`,
  `\=`, `\==`, `<>` and `><`, so those names miss and reach `UNKNOWN`.
* Five native methods: `NAME`, `VALUE`, `VALUE=`, `UNKNOWN` and `REQUEST`, which are the whole of
  `memory/Setup.cpp:1298`-`:1302`. `VALUE=` on a **stem** reference assigns what the bare
  `stem. = value` assigns (`stem_assignment_value`, new in `stem.rs`), because
  `RexxVariable::setValue` "sorts out the stem vs. simple assignment bits" — **without this the
  crate wrote a `Body::Text` into a stem-named slot and the next stem read panicked**, which is
  worse than the divergence being fixed and is the reason row 40 is in the table above.
* `to_text`, `text_len`, `try_text`, `classify_string_conversion`, `heap_to_number`,
  `operator_operand_gap` and `forward_arguments_conversion` each gain a reference arm. The first
  four answer the referent's `stringValue()`; `heap_to_number` and `operator_operand_gap` chase the
  referent, which is what makes `>vr + 1` arithmetic rather than 41.1; `forward_arguments_conversion`
  refuses, which is the measured 98.946.

**The compiler found three exhaustive `Body` matches and no more, so most of this list came from
running rather than from the build.** `stem.rs`'s `body_variant_name`, `run.rs`'s
`forward_arguments_conversion` and `dispatch.rs`'s `receiver_kind` are the three; every other site
matching on `Body` has a wildcard arm that would have taken a reference silently. `to_text`,
`text_len_inner` and `heap_to_number` announced themselves as panics on the first probe that
rendered or added a reference; `try_text`, `classify_string_conversion` and `operator_operand_gap`
were found by a probe disagreeing with the oracle at rc 0; and `forward_arguments_conversion` was
the compiler's, but the arm it first got (chase the referent) was wrong and a probe is what said
so.

### The row set and the probe

`corpus/docs/class-set.txt` and `class-methods.txt` regenerated with

```
cargo run -p rexx-extract --bin rexx-extract-docs -- --oodocs ../oodocs \
    --interpreter ../interpreter --out corpus/docs
```

after adding `("VARIABLEREFERENCE", ">vr")` to `CONSTRUCTION_PROGRAMS`. The diff is the class row
plus four method rows and nothing else, checked by reducing it to `(sign, class, method, status)`
tuples. `corpus/gate-tables/methods/variablereference__instance.rex` regenerated to match; the
gate's own `check_probe_text` re-derives it on every run and compares in both directions, and it is
what told me the committed bytes were stale.

---

## The corpus witness

`corpus/lang/variable_reference.rex`, run by `crates/rexx-exec/tests/variable_reference.rs` against
the oracle on both engines, three descriptors compared separately and raw.

| | command | exit |
|---|---|---|
| **before** (BASE `b4adf8a1e`, program and test copied into a `b4adf8a1e` worktree) | `cargo test --release -p rexx-exec --test variable_reference` | **101** |
| **after** (this tree) | the same command | **0** |

The BASE failure is `stdout differs from the oracle on Ir`, `left: ""` against the oracle's eighteen
lines: the program's third clause is `o~name`, which at BASE is
`97.1 Object "5" does not understand message "NAME".` at rc 159 on both engines.

The program's blocks are `object`, `render`, `spelling`, `tracks`, `unset`, `forwards`, `escapes`,
`aliases` and `plain`, and its header says what each would print if the term still answered the
value. **`render` is the block that would not have moved** — `say` of a reference prints the
referenced value on both sides, which is the whole reason the divergence was silent — and it is in
the program to say so.

### Why it is a test binary of its own and not a line in `corpus/phase-5c.txt`

I wrote `corpus/phase-5c.txt`, wired it into the five `SUBSET_FILES` lists and added
`EXPECTED_SUBSET_5C`, and the differential ran at **332 of 332 matching**. It also turned
`gate_table_c.rs`'s `every_closed_phase_this_table_owns_rows_for_is_gated` red:

```
["5c"] own rows in gate table C and have a committed corpus subset file, so their programs
agree with the oracle, but CLOSED_PHASES does not name them -- a verdict of theirs can move
and every gate still exits 0
```

That assertion (added at 5b's close, `9ec812de9`) makes the existence of `corpus/phase-<id>.txt`
oblige `CLOSED_PHASES` to name the phase, and `verdict_is_gated` then makes every non-`agree` 5c row
an exit status under `REXX_CORPUS_GATE=1` — 233 of them today. **So the first 5c witness cannot go
into a phase subset file, and Task 6 owns the flip.** The whole `phase-5c.txt` change was reverted;
the test binary is the interim, and its module doc says to move the line when 5c closes. **This is
a live conflict between that assertion and the plan's Task 6, which asks for exactly such a
witness**, and it is written into the plan.

---

## What `VariableReference`'s four table C rows now do

They **agree**. `cargo test --release -p rexx-exec --test gate_table_c`, report read from stderr:

```
before (Task 3's report, at BASE):
         VariableReference  instance  loud=yes 5c   4  row(s): unanswered=4
after (this run):
         VariableReference  instance  loud=no  5c   4  row(s): agree=4
```

and the loud block loses its `method "NEW" of class "VariableReference": 4` line. Table C's totals:
`5c: 1347 rows, 233 not yet agree` where BASE was 237; `5a: 135 rows, 0 not yet agree` and
`5b: 6 rows, 0 not yet agree`, both unchanged.

Task 3's report said the cell `>vr` was committable "the day this crate builds the object", and that
is what happened: the cell is one expression over an *uninitialised* `vr`, which the oracle answers
at rc 0.

---

## The divergences that remain, and each one's control

Every one of these is **loud on this crate's side** — rc 120 or a raised condition — and every one
was run against the **BASE binary**. All but the last give the same refusal there, so all but the
last are pre-existing; the last is this task's one new loud refusal and is set out below the table.

| probe | crate | why | BASE gives |
|---|---|---|---|
| `>s.1` (a compound) | rc 120 `20.930: Symbol expected.` vs oracle rc 236 | this crate renders a parse error as a loud refusal; nothing to do with references | the identical rc 120 |
| `(>s.)~value~class~id`, `(>s.)~request('STRING')` | rc 120 `a message send to a stem is not implemented (Phase 5)` | the **stem receiver** gap. Control `q = s.; say q~class~id`, no `>` anywhere, gives the identical refusal | the identical rc 120 |
| `o~identityHash~length`, `~c2x` | `3` against `16`; `C2X` unimplemented | Task 2's `identityHash` rendering finding | the same |
| `o~copy~class~id` | rc 120 `method "COPY" of class "Object"` | Task 3's finding | the same |
| `o~value = .array~of(1,2)` | rc 120 `method "OF" of class "Array"` | a Phase 5 gap the fix now *reaches*: at BASE the program died one clause earlier at 97.1 | 97.1 on `~value` |
| `(o~value == a)` on an array | rc 120 `the operator == applied to an array` | same shape | 97.1 on `~value` |
| `condition('O')` after a trap | rc 120 `CONDITION option "O" answers a Directory` | Task 3's finding | the same |
| `o~request('ARRAY')` | rc 120 `method "MAKEARRAY" of class "String"` | a Phase 5 gap | the same |
| `(>zz) + 1` over an array | rc 120 `the operator + applied to an array` vs oracle `97.1 Object "an Array" does not understand message "+".` | control `zz + 1`, no `>`, gives the identical refusal | the identical rc 120 |
| `o~unknown('LENGTH', 'notanarray')` | rc 120 `method "MAKEARRAY" of class "String"` | the same `MAKEARRAY` gap, reached through a door that did not exist before: at BASE the send went to a `String` and raised 97.1, which the program's trap caught | rc 0, trapped |

**The last row is the one new loud refusal this task creates**, and it is the pre-existing
`String~MAKEARRAY` gap standing where `native_hash_unknown`'s own doc already records it — an
`~UNKNOWN` argument list that is not already an `Array`. Its transcript, from a fresh directory,
three descriptors separately:

```
vr = 5 ; o = >vr
signal on syntax name t6
say o~unknown('LENGTH', 'notanarray')
t6: say 'end'

oracle       rc 0    stdout "end"        stderr empty
crate ir     rc 120  stdout empty        stderr rexx-exec: method "MAKEARRAY" of class "String"
                                                is not implemented (Phase 5)
crate tw     rc 120  the same
BASE ir      rc 0    -- the send went to a String and raised 97.1, which the trap caught
```

The `FORWARD ARGUMENTS` asymmetry is the other transcript worth keeping, because it is the one place
where a reference and the value it names must **not** agree and the first implementation had them
agreeing:

```
::method go ; v = 'val' ; forward message('other') arguments (>v)

oracle  rc 158  Error 98.946: FORWARD arguments must be a single-dimensional array of values.
crate   rc 158  byte-identical (the probe harness's own file path aside)

the same instruction over `(v)` instead:
oracle  rc 0    "String val"
crate   rc 0    "String val"
```

---

## Siblings

`8b87195bc` is `PROCEDURE`, `PROCEDURE EXPOSE`, `USE ARG` and the variable reference. Its other
surfaces were swept for the same shape — a construct answering a value where the oracle answers an
object, at rc 0 on both sides:

* **A `~class~id` sweep over fifteen expression shapes** — a literal, a decimal, a string, an unset
  symbol, `.nil`, a list, `.array`, `.true`, an arithmetic result, a compound read, `date()`,
  `.methods`, `.local`, `arg()`, `.environment` — is byte-identical on both engines. No second
  decay of this shape was found.
* **`<x` is the same term and was measured separately**, not assumed: `~name`, `~value`, `~value =`
  and `call sub <vr` into `use arg >q` all agree.
* **`PROCEDURE EXPOSE` of a compound tail and `USE LOCAL`** are loud on both sides, which that
  commit's own message records and which a silent divergence cannot be.

Two things that commit left behind and this task corrected, both **false comments** rather than
divergences:

* `Argument`'s own doc said `call sub2 >p` into a plain `use arg q` "binds that value". Measured, it
  binds the **reference object** — the callee's `arg(1)~class~id` is `VariableReference`. The
  sentence survived because `say q` prints the value either way. The type is deleted and the
  sentence with it.
* `corpus/lang/use_arg_forms.rex`'s header still says `>PREF` "is worth the referenced variable's
  value, not a reference object". **That sentence is false and is not corrected here**: editing any
  byte of a `corpus/lang/*.rex` reddens `sourceline_matches_the_interpreter_for_every_corpus_program`
  unless its expectation file is regenerated with the `.Package~new` driver, and
  `loop_control_rounding.rex` is the standing precedent for leaving such a comment in place rather
  than paying that. The program's *output* is unchanged, and
  `crates/rexx-exec/tests/variable_reference.rs` now states the true property. It is one line for
  whoever next regenerates that directory.

---

## The controls

**A — the witness fails at BASE.** A `git worktree` at `b4adf8a1e` built into its own
`CARGO_TARGET_DIR`, the program and the test binary copied in, `cargo test --release -p rexx-exec
--test variable_reference` → **exit 101**, transcript above. Without this the witness would be a
program that passes, which proves nothing.

**B — the stem panic.** `t. = 0; p = >t.; p~value = 'replaced'; say t.` panicked at
`stem.rs:472`, `a stem-named slot holds only Body::Stem, got Body::Text`, before
`stem_assignment_value` existed. Found by running, not by review, and it is why row 40 is in the
edge table.

**C — the required-string latch is *not* needed, measured rather than assumed.** The first version
of this change armed `reqstr_armed` at reference construction, on the argument that a reference's
protocol answer can differ from its rendering. Once both answer the referent's `stringValue()` they
cannot, so the arming was removed and a **debug** build run over the array-referent concatenation
program: `required_string_latch_holds` does not fire and the answers are byte-identical to the
oracle. The line is gone.

**D — every remaining divergence has a control at BASE.** Ten probes re-run against the BASE binary,
table above. Seven give the identical refusal, two die one clause earlier at the old 97.1, and one
(`~unknown` with a non-array argument) is rc 0 at BASE and rc 120 now — the one new loud refusal,
listed rather than buried.

**E — `>s.1` and the stem receiver are not mine.** Two controls with no `>` in them —
`q = s.; say q~class~id` and `zz = .array~new(2); say zz + 1` — give the identical rc 120 refusals
their reference-shaped neighbours give.

**F — the hot-path restructuring changes no answer.** Splitting `resolve_aliased` out and letting
the three frame accessors take the unaliased path directly is behaviour-preserving by construction,
and it was checked the same way anything else here is: `rexx-core`'s roots tests, which include an
alias taken before a promotion and a cell read after its frame is popped, and the whole probe sweep
re-run against the rebuilt binary — the same fourteen divergences and no others, every one of them
already in the table above.

---

## Gates

Tree hash (`git diff HEAD | sha256sum`) before the first gate:
`10004fe9ec5a37d80ba88a6c1944b62510721a9c7f9b15e415e0b7985297f834`.

| # | command (from `rust/`) | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |

Each status was written to a file by the runner and read back unpiped in the turn this report was
finished. **Zero `test result: FAILED` lines across gates 3, 4 and 5**, which is the reading that
matters rather than the exit status alone, and each of the three reports **107** `test result: ok`
binaries — one more than Task 3's 106, which is `tests/variable_reference.rs`. Gate 4's corpus
harness is **331 of 331 matching**, unchanged, because the witness is not in a phase subset file;
its table C report is the 233 above and its table D report is `5c: 38 rows, 35 not yet agree`,
unchanged and Task 5's. `memcap` was present (`command -v memcap` → `/home/moritz/.local/bin/memcap`),
so gate 5 is the `memcap` form rather than the `ulimit` substitute.

Clippy was re-run after `touch`ing every changed source file, so its green is evidence the linter
looked at the new code rather than reusing a warm result: its stderr names `rexx-extract`,
`rexx-core`, `rexx-classes` and `rexx-exec`.

Tree hash after the last gate: `10004fe9ec5a37d80ba88a6c1944b62510721a9c7f9b15e415e0b7985297f834`,
unchanged. The report file is the only thing written after the runs, and it is under the git-ignored
`.superpowers/`.

**Three gate runs were started and only the third is reported.** The first was voided by the
`corpus/phase-5c.txt` experiment being reverted mid-run; the second by the `resolve_aliased` split
(control F) and by two corrections to this phase's plan, which is a tracked file. Neither of the
first two is quoted anywhere above: a gate line has to be about the bytes that ship, and re-running
is cheaper than arguing that an edit was harmless.

---

## What I did not do

* **I did not create `corpus/phase-5c.txt`.** It is written above why, with the assertion's own
  text; the witness runs from a test binary of its own instead. Task 6 owns both the flip and the
  move.
* **I did not fix the `>s.1` parse-error rendering**, the stem-receiver gap, `identityHash`,
  `Object~COPY`, `Array~of`, `String~MAKEARRAY`, `condition('O')`, or the array/instance operator
  refusals. Each is loud, each has a control at BASE, and none is a reference.
* **I did not correct `corpus/lang/use_arg_forms.rex`'s false header sentence**, for the reason
  above.
* **I did not implement `~copy` on a reference.** The oracle answers `VariableReference`; this crate
  refuses `Object~COPY` for every receiver, which is Task 2's row and not one this task should move
  for one class.
* **I did not free a cell.** Promotion leaks one cell per referenced variable instance, stated at
  the field and in the design paragraph above. Closing it needs the collector to reach a cell from
  the references that name it, which is its own subject.
* **I did not run the phase gate** (`REXX_PHASE_GATE=5c`): Task 6 turns it on and 5c is not in
  `CLOSED_PHASES`.
* **I did not touch `UNCONSTRUCTIBLE`.** `VariableReference` keeps the `Status::NotCovered` its
  sentence grounds; `coverage_of`'s `NotCovered` arm is where a committed program turns into
  `Coverage::Covered`, exactly as Task 3 left it.
