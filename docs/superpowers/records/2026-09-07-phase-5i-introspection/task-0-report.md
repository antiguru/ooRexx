# Task 0 — extend the arity instrument to this phase's classes

BASE `f4b3c64b4`, confirmed `HEAD` and a clean tree at the start. Two commits: `031b7a6f0`, whose
seven gates all read 0, and a second carrying the controller's two rulings. No method body added.

This report was written under `.superpowers/`, which is in `.gitignore`. The fix round moved it
here, where this project's committed SDD records live; that is the only reason this paragraph is
not the first implementer's.

Delivered: `crates/rexx-exec/tests/support/arity.rs` (the probe, shared),
`crates/rexx-exec/tests/support/setup_cpp.rs` (the `Setup.cpp` join, shared),
`crates/rexx-exec/tests/collection_arity.rs` (now a thin driver),
`crates/rexx-exec/tests/introspection_arity.rs`, `crates/rexx-exec/tests/introspection_scopes.rs`,
and `corpus/introspection-{receivers,arguments,scopes,arity}.tsv`.

---

## 1. Pre-flight: the brief read against the tree

Six findings, all measured. Two changed a file's shape and were put to the controller before any
code was written; four were brief defects I corrected.

**(1) The class arm is missing from the brief's design, and five of this phase's own rows live
there.** `corpus/collection-arguments.tsv` is instance-arm only — it has 44 `Array` rows, which is
exactly `class-methods.txt`'s 44 `Array` **instance** rows, and `Array~of`/`Array~new` are absent.
The brief inherits that shape by naming the same four paths. But `method-bodies.txt`'s class-arm
`loud` rows for this phase's classes are `Package~defaultOptions`, `Method~loadExternalMethod`,
`Method~newFile`, `Routine~loadExternalRoutine` and `Routine~newFile` — and plan Task 4 names four
of them verbatim, Task 7 the fifth. An instance-only table would leave Tasks 4 and 7 unsized for
exactly those five. **Resolution: the three introspection files carry an `arm` column**, receivers
keyed by `(class, arm)`; `corpus/collection-arity.tsv` keeps its four columns unchanged, the shared
module taking the arm column as a layout flag.

**(2) `Pointer` and `Buffer` excluded by name means the instrument does not size Task 2.** Their
class-arm `new` row is measurable — receiver `.Pointer`, no construction needed. Raised as a
decision rather than a silence, and **the controller changed the ruling**: their class arm is now
measured and only their instance arm is excluded, which §2a covers.

**(3) The scopes derivation cannot be copied from `collection_scopes.rs`, in two independent ways.**
Its `constructions()` reads `corpus/docs/class-set.txt` and *asserts* every in-scope class has a
construction expression; `StackFrame` is a `not-covered` row there with `-`. And it builds every
receiver at the top of a single program, which cannot work here: the per-class `::class`/`::method`/
`::routine` directives would collide in one file, and — measured on the oracle 2026-09-07 — a
`RexxContext` built inside a call is **dead** in its caller, `Error 98.981: Target RexxContext is no
longer active`. `introspection_scopes.rs` therefore reads its receivers from
`corpus/introspection-receivers.tsv` and runs **one program per (class, arm)**, fifteen oracle runs.

**(4) The brief names `StackFrame` as the receiver needing the send inside a call; measured, it is
arguably not one and `RexxContext` is.** A `StackFrame` *survives* its own frame's exit and still
answers `~name`, `~line`, `~traceLine` and `~arguments` afterwards — measured in the same run as
(3). It is the `RexxContext` that cannot leave the activation it describes.

**(5) Both of the brief's sketched `Method` receivers are defective.** `.methods['M']` on a floating
`::method M` answers a `Method` whose `~scope` is `.nil` (measured), so every `~scope`-shaped row
would say nothing — the thing the brief asks the receiver to avoid. `.K~instanceMethod('MM')` sent
to the **class object** answers `.nil` even for `K`'s own instance method (measured), which is
`collection_scopes.rs`'s own documented trap. What works on the oracle *and* on this crate is
`r = .K~method('MM')`, a `Method` whose `~scope~id` is `K`.

**(6) `Object~(abuttal)` and `Object~(blank)` are display names, not message names.** Measured:
`o~'(abuttal)'('x')` raises `97.1` while `o~''('x')` and `o~' '('x')` both answer. The oracle never
reaches `SENT` for the documented spelling, so the harness rule fires. Both are `EXEMPT` with that
measurement as the reason, and both are excluded by name from the scopes table so that `NOMETHOD`
stays a failure everywhere else. `Object~'||'` is a real name and needs no exemption.

---

## 2. Decisions taken, and why

**Directives and the wrapper — the mechanism the brief left to me, and it is both.**
`corpus/introspection-receivers.tsv` carries a `wrapper` column (`call` or `--`) and a `directives`
column (` | `-separated directive text, appended after the program). `wrapper = call` puts the
setup **and** the send inside an internal routine called as `call probe 'a1', 'a2'`, with the
`SYNTAX` trap left in the main program, where the condition arrives after the routine unwinds
(measured: a trap in the caller does catch a syntax condition raised in an internal routine).
`wrapper = --` with no directives reproduces today's program text byte for byte, which is what
protects `collection-arity.tsv`. It is used by `RexxContext` and `StackFrame`.
**This is stated in the receiver file's own header**, with the two measurements behind it.

**One receiver per (class, arm), and the setup is kept minimal on purpose.** The brief asks for
`Object` to have both `.Object~new` and "something with methods on it"; the format holds one
receiver per arm, so `Object`'s is `.K~new`, an instance of a public subclass carrying `::method
MM`, and every `Object` row is inherited so all of them still apply. More generally, anything a
single row needs is written **inline in that row's argument list** rather than into the setup —
`.StringTable~new` for `Class~defineMethods`, `.Method~new('X9','return 1')` for `Object~run` — so
that a construction this crate cannot do costs one row rather than a whole class's thirty.

**`Class`'s receiver is `.K` with `inherit MX` in its own directive**, so `~uninherit(.MX)` has
something to remove without the setup itself sending a documented method. `MX2` exists for
`~inherit`. Measured: `::class K public subclass Object inherit MX` works with `MX` defined later
in the file.

**The probe directory holds one fixture file, `source.rex`, containing `return 1`.** The probe
writes one file and runs it; `Method~newFile`, `Routine~newFile`, `Package~findProgram` and
`Package~loadPackage` all take a file name, and the oracle raises `3.1` for one that is not there.
Naming the running program instead would re-execute it. The fixture is written by the harness into
the temp probe directory, not into the repository, and it is a `pub const FIXTURE` with the reason
at the site. It is written for the collection driver too, and the byte-identity control in §5 is
what proves that costs nothing. [**The fix round gated the write on the layout**, so it is no
longer written for the collection driver at all and the control is no longer the only thing holding
that -- §9(6).]

**Three `Object` rows are exempt because the oracle refuses the send from outside a method.**
Measured: `r~setMethod(...)` answers
`97.2  Object "a K" cannot accept private message "SETMETHOD" from this context.`, and the same for
`run` and `unsetMethod`; `self~setMethod(...)` from inside a method works. A third wrapper mode
(`self`, putting the send inside a `::method` on the receiver's own class) would measure them; none
of the three is a Phase 5i row, so I exempted rather than built it. **The mode is the fix if a
later task needs those rows.**

**`Object~new` (class arm) is exempt.** Measured: `.Object~new('x')` raises `93.902` while
`.Object~new` answers, so `Setup.cpp`'s `A_COUNT` is a maximum and not a minimum, and
`every_row_is_sent_something_its_arity_needs` would otherwise demand a list the oracle refuses.

**I did not change the probe to compare the send's RESULT** — see §7, which is where that decision
is measured rather than asserted.

**Also factored: `support/setup_cpp.rs`.** The brief asks only for the arity probe to be shared, but
`introspection_scopes.rs` needs the identical `Setup.cpp` `(scope, name) -> (token, arity)` join,
and a second copy is the "one quantity, two formatters" defect `support/mod.rs`'s own doc records.
`corpus/collection-scopes.tsv` byte-identity after the move is the control, in §5.

---

## 2a. The controller's two rulings, and what carrying them cost

Both arrived after `031b7a6f0` and all seven of its gates had read 0. The tree was frozen for that
run and both were carried afterwards.

### `Pointer` and `Buffer`: class arm in, instance arm out

Ruled so that plan Task 2 has a row that moves when it lands. Carrying it needed a mechanism the
harness did not have, because **the harness rule and this row are in direct conflict**: measured
2026-09-07, `.Pointer~new` answers `SYNTAX 93.967` on the oracle and never reaches `SENT`, so
`the_oracle_completes_every_send` calls it a harness failure — and no argument list would change
that, because the refusal *is* the documented behaviour Task 2 has to reproduce.

An `EXEMPT:` row cannot move, so it would not have satisfied the ruling. The mechanism instead is a
second marker, `REFUSED:`, meaning **the oracle refuses this send by design and the refusal is the
measurement**. The row is still compared on all three descriptors, and it lands where it should:

```
Buffer  new class send-differs oracle rc0 SYNTAX 93.967; crate rc120 rexx-exec: method "NEW" of class "Buffer" is not implemented (Phase 5)
Pointer new class send-differs oracle rc0 SYNTAX 93.967; crate rc120 rexx-exec: method "NEW" of class "Pointer" is not implemented (Phase 5)
```

**The rule is inverted, not waived**, which is what stops the marker being an escape hatch for a bad
list: `every_refused_row_is_really_refused` fails on a `REFUSED:` row the oracle *does* complete.
Controls G and H in §6 are that claim tested in both directions.

The exclusion is now per `(class, arm)` rather than per class — `NO_INSTANCE_ARM` in
`introspection_scopes.rs`, with the reference citation as its reason — which is only expressible
because finding (1) had already put an arm column in the files. Their scope rows derive cleanly:
`Pointer new class Pointer native PointerClass::newRexx A_COUNT`, and `Buffer` likewise.

### Assert the byte-identical program shape rather than state it

`support/arity.rs` now carries four `#[cfg(test)]` assertions, run by every test binary that
declares `mod support`: the unwrapped, directive-free program is spelled out byte for byte
(`a_receiver_with_no_wrapper_and_no_directives_writes_the_original_program`), `--` sends no
parentheses, the `call` wrapper's whole text is pinned, and directives append after the program.
`corpus/collection-arity.tsv`'s every byte is downstream of the first of those, so it is asserted
rather than left to a diff someone remembers to run. [**The fix round adds one more**, pinning the
value-comparing program text the same way, so the count in the sentence above is no longer the
number of them.]

### Two header notes the controller asked for, re-measured here rather than copied

Both reproduce exactly, and both are now in `corpus/introspection-receivers.tsv`'s own header:

* `class-set.txt`'s `Method` receiver is `.Object~method('objectName')`, a **native** method whose
  `~source` is an empty `Array` where `.K~method('MM')`'s has one line. **The seven `is*` flags do
  not discriminate** — measured, both receivers answer `0` to all of them except `isGuarded`, which
  is `1` on both — so `~source` is the only column that tells a body reading the source from one
  answering an empty Array, and that is why the committed receiver is a Rexx method.
* `class-set.txt`'s `Package` receiver is `.Class~package`, the REXX package: measured `name REXX`,
  `sourceSize 0`, `source Array(0)`, and empty tables for `routines`, `publicRoutines`,
  `resources`, `definedMethods` and `namespaces`; only `classes` (67) and `publicClasses` (62) have
  content. Every `Package` row in `method-bodies.txt` is measured against that, which is why this
  table's receiver is a program's own package.

---

## 2b. For Task 6: what outlives its frame, measured

Plan Task 6 says it must settle "what happens when a `StackFrame` outlives the frame it describes"
before choosing a mechanism. It is settled here, and **it goes the surprising way round: the frame
survives and the context does not.** One program, `f = mkframe(...)` and `c = mkcontext(...)` where
each helper returns `.context~stackFrames[1]` and `.context` respectively, read after the helper has
returned:

```
StackFrame after its frame returned:
  class      StackFrame
  name       MKFRAME
  type       INTERNALCALL
  line       15
  arguments  Array 2
  traceLine      15 *-*   return .context~stackFrames[1]
  context    RexxContext
RexxContext after its frame returned:
  class      RexxContext
```

and then, on stderr, at rc 158:

```
       *-* Compiled method "NAME" with scope "RexxContext".
    13 *-* say '  name      ' c~name
Error 98 running <file> line 13:  Execution error.
Error 98.981:  Target RexxContext is no longer active.
```

Read precisely: the dead `RexxContext` still answers `~class~id`, because that is `Object`'s method
and not the context's; it is the context's **own** readers that raise. A `StackFrame` is a snapshot
and answers everything, including a `~context` that is itself already dead.

That is also why the receiver file has a `wrapper` column at all — `RexxContext` must be sent to
from inside the call, and `StackFrame` need not be but is, for the same frame.

---

## 3. Per class: `agree` and `send-differs` under a real argument list

**This is the deliverable that sizes every later task.** `corpus/introspection-arity.tsv`, both arms:

| class | rows | agree | send-differs | setup-differs | exempt |
| --- | --- | --- | --- | --- | --- |
| `Buffer` (class arm only) | 1 | 0 | 1 | 0 | 0 |
| `Class` | 32 | 20 | 12 | 0 | 0 |
| `Method` | 20 | 4 | 16 | 0 | 0 |
| `Object` | 32 | 23 | 3 | 0 | 6 |
| `Package` | 40 | 7 | 33 | 0 | 0 |
| `Pointer` (class arm only) | 1 | 0 | 1 | 0 | 0 |
| `RexxContext` | 15 | 1 | 14 | 0 | 0 |
| `RexxInfo` | 28 | 0 | 28 | 0 | 0 |
| `Routine` | 11 | 2 | 9 | 0 | 0 |
| `StackFrame` | 10 | 0 | 0 | 10 | 0 |
| `WeakReference` | 2 | 1 | 1 | 0 | 0 |
| **total** | 192 | 58 | 118 | 10 | 6 |

**This table is the measurement before value comparison landed.** The fix round turned it on for
this driver and ten of these rows moved; §10 below carries the split as it now stands.

`StackFrame`'s ten are `setup-differs` and not a harness fault: the receiver is
`.context~stackFrames[1]` and this crate refuses `STACKFRAMES`, so the row says nothing about its
own method until Task 6 lands. That is the phase's headline measurement working as designed.

### Against the scope note's `loud` counts — no re-slice, and four named moves

The new table has a row per **documented** row and the scope note counts **`loud`** rows, so the
per-class totals differ by construction and that difference is not a re-slice. The comparable
number is `send-differs` + `setup-differs`, which is **128** against the **131** the scope note
gives these eleven classes — 124 for the nine with instance receivers, plus Pointer's 6 and
Buffer's 1. Over the nine it is 126 against 124, **a surplus of two**, and both are the `~new` rows
named below; the rest of the difference is Pointer's five instance rows, which have no receiver and
which the scope note itself says cascade off `new`. Set-differenced row by row against `method-bodies.txt`:

> **CORRECTED in the fix round, 2026-09-07.** This paragraph read "**128** against the scope note's
> **139**: 126 over the nine classes with instance receivers against its 133". Both figures were
> wrong and in the same way: the scope note's 133 and 139 include the eight `of` rows on the mapped
> classes, which this instrument does not cover at all. The nine classes' own figure is 124 —
> `method-bodies.txt`'s `loud` rows for them are 11+16+3+33+14+28+8+10+1, which this report's own
> per-class prose two paragraphs down already sums to. So it was stated as a seven-row shortfall
> where it is a two-row surplus, wrong in magnitude and in sign, on the number the brief's "say so
> and stop" trigger keys on. The conclusion — no re-slice — is unchanged and correct.
>
> **The finding that caught it named only the inner figure.** Correcting 133 to 124 and stopping
> would have left "128 against the scope note's 139" standing, and 139 is the same set eight rows
> wider — a correction that fixes one of two wrong numbers in one sentence, which is the shape this
> project keeps hitting. The controller agreed the fuller rewrite was the right call.

**Three rows `method-bodies.txt` calls `answers` are `send-differs` under a real argument list.**
All three are class-arm constructors that the zero-argument probe never gets past:

| row | `method-bodies.txt` | `introspection-arity.tsv` |
| --- | --- | --- |
| `Class~new` (class) | `answers` | `send-differs` |
| `Package~new` (class) | `answers` | `send-differs` |
| `Routine~new` (class) | `answers` | `send-differs` |

**One row `method-bodies.txt` calls `loud` reads `agree` here**: `Package~publicClasses`. This is
the row plan Task 8 singles out as "the odd one", whose `loud` evidence is
`the REXX package's class table` rather than a `native_method` refusal. With Task 0's receiver —
`.context~package`, a package with two public classes — the send **completes on both sides**. Task 8
should read this before starting: the name is bound and reachable for this receiver, and what is
unknown is whether it answers the right *contents*, which §7 explains this instrument cannot see.

Everything else lines up on the total: `RexxInfo` 28, `RexxContext` 14, `StackFrame` 10, `Object` 3,
`WeakReference` 1 and `Method` 16 match the scope note. `Package` also totals 33 either way, but the
split moves -- 32 instance + 1 class in `method-bodies.txt` against 31 + 2 here, because
`publicClasses` left the instance side and `new` joined the class side. `Class` is 12 against 11 and
`Routine` 9 against 8, the extra in each being its `~new` row above.

---

## 4. The refresh environment variables

Exactly:

* `REXX_INTROSPECTION_ARITY_REFRESH` — `REXX_INTROSPECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec --test introspection_arity`
* `REXX_INTROSPECTION_SCOPES_REFRESH` — `REXX_INTROSPECTION_SCOPES_REFRESH=1 cargo test --release -p rexx-exec --test introspection_scopes`

`REXX_COLLECTION_ARITY_REFRESH` and `REXX_COLLECTION_SCOPES_REFRESH` are unchanged.

---

## 5. Red control for the refactor: the two collection tables, byte for byte

**Before the refactor**, at BASE, to establish that the committed table is reproducible at all:

```
$ REXX_COLLECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec --test collection_arity
exit=0
$ sha256sum corpus/collection-arity.tsv
d532e4e2927ae5ff7f54b31f3fe3a03e37d3c4dba0c5f5423847e56bd5dd4042  corpus/collection-arity.tsv
```

which is the committed file's own sha256, taken before anything was touched.

**After the whole refactor** — the probe moved into `support/arity.rs`, the `Setup.cpp` join moved
into `support/setup_cpp.rs`, the `arm`/`wrapper`/`directives` mechanisms added, and the `source.rex`
fixture written into every probe directory:

```
$ REXX_COLLECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec --test collection_arity
ca-exit=0
$ diff <BASE copy> corpus/collection-arity.tsv
collection-arity-diff=0

$ REXX_COLLECTION_SCOPES_REFRESH=1 cargo test --release -p rexx-exec --test collection_scopes
cs-exit=0
$ diff <BASE copy> corpus/collection-scopes.tsv
collection-scopes-diff=0
```

Both diffs empty, both tables byte-identical, and `git status` shows neither file modified.

**Re-run after the second round**, once the `arm` column, the `REFUSED:` marker and the four program
assertions had landed: `ca-diff=0` and `cs-diff=0` again. **The control the controller said it cared
most about still holds after the column landed.**

### That control is green, so here are the two that make it mean something

Each prediction was written down before the run.

**Control B — emit the send *before* `say 'SETUP-OK'`.** *Predicted:* the table moves; the two
`send-differs` rows become `setup-differs`; the 430 `agree` rows stay; exactly 2 changed lines.
**CONFIRMED**, exactly:

```
261c261
< Properties	save	send-differs	oracle rc0 SENT; crate rc120 rexx-exec: the LIBRARY REXX entry point "stream_init" is not implemented (Phase 7)
---
> Properties	save	setup-differs	crate rc120: rexx-exec: the LIBRARY REXX entry point "stream_init" is not implemented (Phase 7)
263c263
< Properties	setLogical	send-differs	oracle rc0 SENT; crate rc120 rexx-exec: ARG option "A" answers an Array, which is not implemented
---
> Properties	setLogical	setup-differs	crate rc120: rexx-exec: ARG option "A" answers an Array, which is not implemented
```

**Control C — make `program` ignore the argument list and always send none**, the defeat the
harness rule exists for. *Predicted:* `the_oracle_completes_every_send` goes RED for every method
needing arguments, while the refreshed table stays **largely `agree`** — the table alone cannot see
the defeat and the harness rule can.

*Half confirmed, half falsified, and the falsified half is worth more than the prediction was.*
`the_oracle_completes_every_send` **FAILED** naming **286** rows, the first four being
`Array~[]`, `Array~[]=`, `Array~append` and `Array~appendAll`, each `the oracle stopped at "SYNTAX
93.901"` — **CONFIRMED**. But the table did *not* stay largely `agree`: it went from 430 `agree` /
2 `send-differs` to **146 `agree` / 286 `send-differs`**, 572 changed lines — **FALSIFIED**. The
two engines refuse differently from the oracle often enough that the table sees this particular
defeat too.

The third reading, which I did not predict and which is the one to keep:
`every_row_is_sent_something_its_arity_needs` **passed** through control C. It polices the
*argument file*, and control C corrupted the *probe*; the two tests are not redundant and neither
covers the other.

---

## 6. The controls on the new instrument

Predictions written before each run.

**Control D — delete `Package~findClass` from `corpus/introspection-arguments.tsv`.** *Predicted:*
`every_row_is_sent_something_its_arity_needs` fails naming it, and `the_table_matches_the_three_sides`
fails on the row count. **CONFIRMED, both:** exit 101, 2 failed,
`"Package~findClass (instance) has no argument list at all"` and
`introspection-arity.tsv has 190 rows against the interpreters' 189`.

**Control E — flip `RexxInfo~platform`'s committed verdict from `send-differs` to `agree`.**
*Predicted:* `the_table_matches_the_three_sides` fails naming that row. **CONFIRMED:** exit 101,
1 failed, the diff naming `method: "platform"` on both sides.

**Control F — remove the `StackFrame` row from `corpus/introspection-receivers.tsv`.** *Predicted:*
`every_class_in_scope_has_a_receiver` fails naming `StackFrame (instance)`. **CONFIRMED, and it went
one further than predicted:** exit 101, **2** failed — `every_class_in_scope_has_a_receiver` with
`"StackFrame (instance)"`, and `the_table_matches_the_interpreter` with
`introspection-receivers.tsv has no setup for StackFrame (instance)`.

**Control G — drop the `REFUSED:` marker from `Pointer~new`, leaving an ordinary `--` send.**
*Predicted:* `the_oracle_completes_every_send` fails naming it, and the table stays green because
the program text is unchanged; **exactly 1 test red**. *Half confirmed, and the falsified half is
the better news:* the named failure is exactly
`"Pointer~new (class): the oracle stopped at \"SYNTAX 93.967\""` — **CONFIRMED** — but **2** tests
went red, not 1: `every_row_is_sent_something_its_arity_needs` also fires, with
`"Pointer~new (class) is native at arity A_COUNT and is sent nothing"` — **FALSIFIED**. Two
independent rules catch the same removal, which is what I want and not what I predicted.

**Control H — mark `RexxInfo~platform`, which the oracle *does* complete, as `REFUSED:`.**
*Predicted:* `every_refused_row_is_really_refused` fails naming it; `the_oracle_completes_every_send`
stays green because it now skips that row; exactly 1 test red. **CONFIRMED, all three parts:**
exit 101, 1 failed,
`"RexxInfo~platform (instance) is marked REFUSED: and the oracle completes the send, so the refusal
is not the measurement -- give it a real argument list"`. **That pair is what makes `REFUSED:` a
narrowing of the harness rule rather than a hole in it.**

**Determinism.** Both new tables were refreshed twice and diffed; both diffs empty. The committed
tables carry no temporary path — `grep -a '/tmp\|rexx-introspection' corpus/introspection-arity.tsv`
matches nothing — which matters because the probe directory's name carries a pid and a nanosecond
timestamp.

**No registration control was run, because this task adds no method body.** The plan's 5f/5g
registration control belongs to a task that binds a name in `NATIVE_METHODS`; Task 0 binds none, and
recording a green registration cell here would be a cell over nothing.

**All 190 rows the oracle is expected to complete do complete.** Iterated with an oracle-only mirror of `program()` in the
scratch directory; the final run reports `0 rows the oracle does not complete`, and the harness's own
`the_oracle_completes_every_send` is green.

---

## 7. What makes a later task bigger than its row count suggests

**(a) `agree` says the send completed the same way, not that the two sides answered the same
value.** The probe sends a bare statement and never prints the result, so `agree` means neither side
raised. **Measured, not asserted:** I re-ran all 58 `agree` rows with `vv~string` printed as well.
48 still agree, 3 do not, and 7 have no value to compare -- they return no result, so the
assignment the variant needs raises `91.999 Message "DELETE" did not return a result` (measured;
my throwaway variant's handler was written for `91.1` and so reported them as unsent):

* `Class~enhanced` — oracle `enhanced K`, crate `a K`. **A real divergence hiding under an `agree`
  row, and Task 5 owns it.**
* `Object~hashCode` and `Object~identityHash` — oracle `-139865136211569`, crate `516`. Necessarily
  different; if a task implements them the divergence needs licensing, not fixing.

The limitation is recorded in `support/arity.rs`'s module doc and in the new table's own header, and
it applies to `corpus/collection-arity.tsv` identically. [**The fix round turned value comparison on
for this driver**, so the limitation now holds only where the flag is off, which is
`corpus/collection-arity.tsv` -- §10. The three rows named below are what the instrument itself now
reports.] **Every task in this phase still owes each
row it closes a witness that reads what the row answered** — that is the plan's second NEW
constraint, and this is the measurement showing why it is not optional.

**(b) `Package~importedPackages` cannot be made non-empty by this instrument, so six of Task 8's
rows are measured against an empty container on both sides.** A `::requires` needs a file on this
build's search path and nothing is on it: `rxregexp.cls`, `json.cls` and `socket.cls` all answer
`43.901` at rc 213, and the only `.cls` files on the machine are under `build/bin` and `build/samples`,
neither of which is searched. Everything else in the `Package` receiver is genuinely plural —
measured `classes 4, publicClasses 2, routines 2, publicRoutines 2, definedMethods 2, resources 2`
(**corrected in the fix round from `classes 3`**, which was wrong in this report and in
`corpus/introspection-receivers.tsv`'s header; the receiver defines `K`, `L`, `MX` and `MX2`, and
re-running its own directive set on the oracle answers 4) —
which is what plan Task 8 asks Task 0 for. `importedPackages`, `importedClasses`, `importedRoutines`
and `namespaces` are the ones it could not deliver, and a body answering an empty container will
agree with the oracle on all of them.

**(c) `Package~publicClasses` already reads `agree`**, per §3. It is one of Task 8's twenty-one rows
and it is not a fresh implementation; what it needs is (a)'s witness.

**(d) `Package~options` takes an argument, and the plan's measurement of it is incomplete.** Task 7
records `options=String` rendering the whole `::OPTIONS` line and asks for a re-measurement.
Measured 2026-09-07: `p~options` answers the whole `::OPTIONS ...` string, `p~options('DIGITS')`
answers `9`, and `p~options('DIGITS', 1)` also answers `9`. So it is a per-option reader with a
no-argument summary form, and the table sends it `'DIGITS'`.

**(e) `Object~run`, `~setMethod` and `~unsetMethod` are private and are not measured**, per §2. None
is a Phase 5i row, but a later phase that wants them needs the `self` wrapper mode described there.

**(f) Six of `Method`'s and three of `Routine`'s rows are `set*`/`newFile`/`loadExternal*` whose
argument list this table proves the oracle accepts, and nothing more.** Measured, both
`loadExternalMethod('M9','LIBRARY rexxutil SysCurPos')` and
`loadExternalRoutine('R9','LIBRARY rxmath RxCalcPi')` complete on the oracle, and `newFile` needs a
real file — `.Method~newFile('nope.rex')` raises `3.1`. Task 4's "if the answer needs a stream this
phase does not have" question is therefore still open; the table sizes those rows but does not
answer it.

---

## 7a. Runtime, because eight later tasks re-run this

From the gate run at `031b7a6f0`, read off its own output:

* `introspection_arity` — **10.60s** for 190 rows, which is three sides per row twice over (the
  table and the harness rule each call `measured`), about 1140 process launches.
* `collection_arity` — **25.07s**.
* `introspection_scopes` — **0.16s**, because it is nine oracle runs and one file read.

Nowhere near the "few minutes" that would have been worth flagging. The second round adds a third
`measured` call to `introspection_arity` (`every_refused_row_is_really_refused`) and its own two
rows, measured **10.57s** — no material change, because the cost is dominated by process startup
and the three calls run in parallel test threads.

## 8. Gates

### First commit, `031b7a6f0` — all seven zero

Read from `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/t0/gate-status.txt`, whose first line is the
commit sha and whose last is `finished 2026-09-07T22:57:17+02:00`:

| gate | command | result |
| --- | --- | --- |
| G1 | `cargo fmt --all --check` | exit 0 |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| G3 | `cargo test --release --workspace --no-fail-fast` | exit 0 |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | exit 0 |
| G5 | `cargo test --release -p rexx-exec --test collection_arity` | exit 0, 18 passed |
| G6 | `cargo test --release -p rexx-exec --test introspection_arity` | exit 0, 18 passed |
| G7 | `cargo test --release -p rexx-exec --test introspection_scopes` | exit 0, 17 passed |

`memcap` is present in this session (`/home/moritz/.local/bin/memcap`), so G4 ran under it rather
than the `ulimit` substitute. The suite started after the commit, so the gated tree is the committed
tree by construction, and the tree was left untouched until the status file said `finished` —
the controller's two rulings were carried only afterwards.

### Second commit — fast checks by me, gates started after it

* `cargo fmt --all --check` — exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
* `cargo test --release --workspace --no-fail-fast` — exit 0, no `test result: FAILED` line anywhere.
* `REXX_COLLECTION_ARITY_REFRESH=1 ... --test collection_arity` then `diff` — empty.
* `REXX_COLLECTION_SCOPES_REFRESH=1 ... --test collection_scopes` then `diff` — empty.

Read from `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/t0/gate-status-2.txt`, whose first line is
`61c5fb1eaa774f3e7fe4bafc01cd64bcbe899607` and whose last is
`finished 2026-09-07T23:20:28+02:00`:

| gate | command | result |
| --- | --- | --- |
| G1 | `cargo fmt --all --check` | exit 0 |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| G3 | `cargo test --release --workspace --no-fail-fast` | exit 0, no `test result: FAILED` line |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | exit 0, no `test result: FAILED` line |
| G5 | `cargo test --release -p rexx-exec --test collection_arity` | exit 0, 22 passed |
| G6 | `cargo test --release -p rexx-exec --test introspection_arity` | exit 0, 23 passed |
| G7 | `cargo test --release -p rexx-exec --test introspection_scopes` | exit 0, 21 passed |

All fourteen gate readings across the two commits are zero. G5's count rises from 18 to 22 and G7's
from 17 to 21 because `support/arity.rs`'s four program assertions compile into every binary that
declares `mod support`; G6's 18 to 23 is those four plus
`every_refused_row_is_really_refused`.

---

# The fix round, 2026-09-07

A second implementer, from `8dec6df4d`. Everything above this line is the first implementer's and
is left as written, except the two figures the review falsified, which are corrected in place with
a note beside each. Nothing in the two commits above was reverted.

The round applies the nine findings of the review at
`…/scratchpad/r0/review.md`; its §9 ❌ (`Pointer` and `Buffer` per (class, arm)) was already carried
by `61c5fb1ea` and `8dec6df4d` and is not re-done here.

## 9. The nine, and what each cost

**(1) `classes 3` is 4.** `corpus/introspection-receivers.tsv`'s header and §7(b) above. Re-measured
independently with the Package receiver's own directive set: the oracle answers
`classes 4 publicClasses 2 routines 2 publicRoutines 2 definedMethods 2 resources 2
importedPackages 0`. The receiver defines `K`, `L`, `MX` and `MX2`, so 4 is the arithmetic as well
as the reading, and the conclusion the figure supported — the receiver is genuinely plural — is
unaffected.

**(2) The re-slice check was measured against the wrong denominator.** Corrected in §3 with the
arithmetic beside it. `method-bodies.txt`'s `loud` rows for the nine classes are Class 11,
Method 16, Object 3, Package 33, RexxContext 14, RexxInfo 28, Routine 8, StackFrame 10,
WeakReference 1 = **124**; the scope note's 133 and 139 both include the eight `of` rows on the
mapped classes, which this instrument never covers. The correction changes the sign as well as the
magnitude: a two-row surplus, not a seven-row shortfall.

**(3) The module doc's `agree` caveat overclaimed and counted a set.** `support/arity.rs`. The
sentence "three of them answer differently, and the other rows there hold" was false for the rows
that return no result — they were unobservable, not held — and named the size of a mutable in-repo
aggregate. It is replaced by a sentence with no count and no false half, which also states the
thing the review asked for and that nothing committed said: **the caveat is a property of the probe
and holds for every table it drives with value comparison off, `corpus/collection-arity.tsv`
included**, whose own header cannot be edited without breaking the byte-identity control.
`collection_arity.rs`'s module doc says it too, because that is the file the table's own provenance
line points a reader at. The measurement the old sentence carried is not restated anywhere: item 9
moved it into the table, where it cannot rot.

**(4) "Five of" deleted**, in `crates/rexx-exec/tests/introspection_arity.rs` and
`corpus/introspection-arguments.tsv`. Both sentences keep their enumeration, which is the part that
cannot rot.

**(5) The fabricated `Layout` is gone.** `introspection_scopes.rs`'s `receivers()` built a `Layout`
whose `table` — the write target — was `corpus/introspection-receivers.tsv`, a committed input, and
whose `refreshing()` answered true whenever the scopes refresh variable was set. Receiver reading
is now the free function `arity::read_receivers(file, arm_column)`, called by both
`Layout::receivers` and that file, so no layout with an input as its write target exists to be
passed to `table_disagreements`.

**(6) The `source.rex` fixture is gated on the layout.** `Layout::fixture`, off for the collection
driver. **Witnessed in the environment itself rather than argued**: the probe directories survive
their run, and the ones this session's *pre-fix* collection runs left behind hold `source.rex`
(`/tmp/rexx-collection-arity-125831-1788813654814260214` and its siblings) while every collection
directory written after the fix holds `probe.rex` and nothing else. The introspection directories
still hold it, which is the half that has to keep working.

**(7) Three guards no longer read green over a missing table.** `engine_splits` and the two
`introspection_scopes.rs` tests assert the committed table is non-empty when not refreshing, which
is the assertion `table_disagreements` already made. Controls M and N below.

**(8) The two guards with no red control have one each.** Controls I and J below.

**(9) Value comparison, as a `Layout` flag: on for the introspection driver, off for the
collection driver.** This is the substantial half of the round and §10 is its own section.

## 10. Value comparison

**The mechanism.** Under `Layout::compare_values` the probe assigns the send —
`vv = r~'name'(args)` — and prints `say 'VALUE' vv~string` after `SENT`. A method that returns no
result raises `91.999` at the assignment, so the handler reads that one code as a completed send
with nothing to compare and prints `SENT` then `NORESULT`, which keeps such a row inside the
harness rule rather than making it look refused. The handler builds the code from `rc` and
`condition('E')` rather than `condition('O')~code`: measured on both sides, a `91.999` gives
`rc=91` and `condition('E')=999` on the oracle and on this crate alike, where `condition('O')` is
an object one side declines to build — **a probe whose own handler a side cannot run measures the
probe**. That was not a hypothetical, and it is the best catch of the round: **the probe was measuring
itself.** The first refresh, written with `condition('O')~code`, turned all seven no-result rows
into `send-differs` whose evidence read
`crate rc120 rexx-exec: CONDITION option "O" answers a Directory, which is not implemented` — the
verdict on `Class~define` was a report of *this crate's own gap in the probe's handler*, not of
anything `Class~define` does. Both sides run `Class~define` correctly.

The collection driver's flag-off path still calls `condition('O')~code` and is **left alone
deliberately**: the byte-identity control needs that path untouched, and the path is dead by
construction rather than by data — `the_oracle_completes_every_send` guarantees the oracle raises on
no non-exempt row, so this crate raising is the only way to reach the handler, and nothing there
does today. The controller has ledgered it for the whole-branch review.

**Two verdicts were added rather than folding the cases into `agree`.**

* `no-value` — both sides completed the send and neither returned a result. It is deliberately not
  `agree`: a task sizing itself from such a row learns that the send is reachable and nothing about
  what the method answers.
* `unstable` — `corpus/method-bodies.txt`'s own word, for a row marked `UNSTABLE:` in the argument
  file. The value is not printed and only the three descriptors are compared.

**`Object~hashCode` and `~identityHash` are classified by measurement, and the measurement is
asserted rather than written down.** Three oracle runs of one program answer three different byte
strings for `hashCode`; two runs answer `-140458910717729` and `-140371917186849` for
`identityHash`. So they are `unstable`, not a divergence needing a licence.
`every_unstable_row_is_really_unstable` re-runs the **oracle** twice on every marked row and fails
if its two answers agree — the same inversion that keeps `REFUSED:` honest, and control K is it
tested in the other direction. `hashCode`'s answer is also raw binary rather than a printable number —
three runs put three different unprintable octet strings on stdout — which is a second reason it
has no business in a TSV, and `corpus/introspection-arguments.tsv`'s header now says so, so that
nobody later writes a probe that prints one. The controller re-measured the premise independently
on three further runs and reports the same: the values are address-derived and ASLR moves them.
What a red on that test would mean is at the assertion: either the answer stopped being
address-derived, or two addresses collided.

**What it found, which is the point of the exercise.** The row moved, and it is the row the flag
was turned on for:

```
corpus/method-bodies.txt      Class enhanced instance answers        rc 163
corpus/introspection-arity.tsv (before)                agree         rc0
corpus/introspection-arity.tsv (after)                 send-differs  oracle rc0 VALUE enhanced K; crate rc0 VALUE a K
```

rc 0 on both sides, the send completing on both, and **the method itself works**: measured here on
all three sides, `d['EXTRA'] = .Method~new('EXTRA','return 99')` then `o = .K~enhanced(d)` answers
`extra= 99` on the oracle, on `ir` and on `tree-walker` alike, and only `o~string` differs —
`enhanced K` against `a K`. So the divergence is in the enhanced instance's own name and nothing
else, which is what Task 5 inherits. No instrument in this tree could see it before: one table
calls the row `answers`, and the other called it `agree`.

**Two defects of the instrument that only value comparison exposed**, both fixed here:

* `Package~findProgram` answers the **absolute path of the fixture**, so the table rewrote itself on
  every refresh — the probe directory's name carries a pid and a nanosecond timestamp. Both streams
  of every side now have that directory replaced by `<probe directory>` before the sides are
  compared; the substitution is applied to all three, so it cannot make two sides that answered
  differently compare equal. Two refreshes now `diff` empty.

  **This is the second instance in this task of one class, and the class is worth the name.** The
  review found the first: the `source.rex` write was inert for the collection driver, but inert
  *because of what today's rows happen to do* rather than because the code could not reach it. The
  `findProgram` path is the same shape found by a completely different route — nothing in the design
  said an answer could not quote the harness's own directory, and one row did. A third sits in the
  code as shipped and is recorded rather than fixed: `scrub` and `printable` are **not** layout-gated
  and run for every table the probe drives, so their being no-ops for `corpus/collection-arity.tsv`
  is a property of what that table's rows answer and not of the code. They are deliberately left
  ungated — the collection driver has exactly the latent non-determinism they cure — and
  `support/arity.rs`'s module doc says so, with that table's byte identity as what would notice if
  it stopped being true.
* An answer containing a tab would put a second tab in its row and break the table's own shape.
  Evidence now renders control characters as spaces. No committed row needs it today, which is
  exactly the "inert, but data-dependently so" the review named for the fixture.

**The split, per class, as it now stands** (`corpus/introspection-arity.tsv`, both arms):

| class | rows | agree | send-differs | setup-differs | no-value | unstable | exempt |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `Buffer` (class arm only) | 1 | 0 | 1 | 0 | 0 | 0 | 0 |
| `Class` | 32 | 13 | 13 | 0 | 6 | 0 | 0 |
| `Method` | 20 | 4 | 16 | 0 | 0 | 0 | 0 |
| `Object` | 32 | 20 | 3 | 0 | 1 | 2 | 6 |
| `Package` | 40 | 7 | 33 | 0 | 0 | 0 | 0 |
| `Pointer` (class arm only) | 1 | 0 | 1 | 0 | 0 | 0 | 0 |
| `RexxContext` | 15 | 1 | 14 | 0 | 0 | 0 | 0 |
| `RexxInfo` | 28 | 0 | 28 | 0 | 0 | 0 | 0 |
| `Routine` | 11 | 2 | 9 | 0 | 0 | 0 | 0 |
| `StackFrame` | 10 | 0 | 0 | 10 | 0 | 0 | 0 |
| `WeakReference` | 2 | 1 | 1 | 0 | 0 | 0 | 0 |
| **total** | 192 | 48 | 119 | 10 | 7 | 2 | 6 |

Against §3's pre-fix table: `agree` 58 → 48, and those ten are 7 `no-value` + 2 `unstable` +
`Class~enhanced`, which is §7(a)'s 48/3/7 split reproduced by the instrument instead of by a
throwaway variant.

**The re-slice check, re-run on the new figures.** `send-differs` + `setup-differs` is now **129**
against the **131** the scope note gives these eleven classes, and **127 against 124** over the
nine. The surplus grows from two to three, the third being `Class~enhanced` moving out of `agree`.
Three rows on 124 is not a re-slice and the phase's shape does not change.

**The third row is a different kind of thing from the other two, and a later reader has to be able
to tell them apart.** The two `~new` rows §3 names are rows `method-bodies.txt` classified
differently because it sends no arguments — they were always going to need work and the scope note
would have counted them under a sharper instrument. `Class~enhanced` is **a defect this phase
found**: it was not a `loud` row, nothing counted it, and it left `agree` because the instrument got
sharper rather than because the work got bigger. A surplus of that kind is the instrument earning
its keep, not scope creep.

**A `no-value` row is not a closed row**, and neither is an `agree` one: `~string` renders `a
Supplier` for a `Supplier` and `an Array` for an `Array`, so the value comparison sees the class of
an answer and not its contents. The plan's second NEW constraint — a second send into anything this
phase builds — is unchanged by this and is still what closes a row.

## 11. Controls

Every prediction was written to
`…/scratchpad/t0fix/predictions.md` **before** its run, and each part is
marked below.

**The control the controller cares most about — `corpus/collection-arity.tsv` and
`collection-scopes.tsv` byte for byte.** Refreshed at `8dec6df4d` before any edit, and again after
the whole round:

```
$ REXX_COLLECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec --test collection_arity
ca-exit=0
$ REXX_COLLECTION_SCOPES_REFRESH=1 cargo test --release -p rexx-exec --test collection_scopes
cs-exit=0
$ diff <before copy> corpus/collection-arity.tsv  -> ca-diff=0
$ diff <before copy> corpus/collection-scopes.tsv -> cs-diff=0
$ sha256sum corpus/collection-arity.tsv
d532e4e2927ae5ff7f54b31f3fe3a03e37d3c4dba0c5f5423847e56bd5dd4042
$ sha256sum corpus/collection-scopes.tsv
022be97aef23a171318000c4ecefb2100b614d4df5245ebc54649d62ade5858f
```

Both shas are the committed files' own and the arity one is the figure the dispatch quoted. The
control was run three times across the round — after the layout flags landed, after the trap
changed, and after the probe-directory substitution — because each of those three touches the
program text or the streams the collection driver reads. `corpus/introspection-scopes.tsv` also
refreshes byte-identical; nothing in this round touches the scope probe.

**Determinism.** `corpus/introspection-arity.tsv` refreshed twice, `diff` empty. It was **not**
empty before the probe-directory substitution — that failure is the finding in §10, and the first
two attempts differed in exactly the `Package~findProgram` row each time.

| control | prediction | reading |
| --- | --- | --- |
| **I** — truncate `Object~(blank)`'s `EXEMPT:` reason to `too short` | `every_exemption_says_why` red naming it; `the_table_matches_the_three_sides` red as well, since an exempt row's evidence *is* its reason; exactly 2 | **CONFIRMED, all three.** exit 101, 2 failed, `"Object~(blank) (instance) is exempt with no reason worth reading: \"too short\""` |
| **J** — blank `Object~hasMethod`'s list, native at `Setup.cpp` arity 1 | `the_oracle_completes_every_send` red with a `93.9xx`; `every_row_is_sent_something_its_arity_needs` red; **`the_table_matches_the_three_sides` stays green**, because both sides agree about the arity error; exactly 2 red | **CONFIRMED, all four.** exit 101, 2 failed, `"Object~hasMethod (instance): the oracle stopped at \"SYNTAX 93.903\""` and `"Object~hasMethod (instance) is native at arity 1 and is sent nothing"`. The table test was green, which is the defeat the harness rule exists for, measured on this driver rather than inherited from the collection one |
| **K** — mark the reproducible `Object~objectName` `UNSTABLE:` | `every_unstable_row_is_really_unstable` red naming it and quoting the repeated answer; the table test red on the verdict; exactly 2 | **CONFIRMED, all three.** exit 101, 2 failed, `"Object~objectName (instance) is marked UNSTABLE: and the oracle answers \"VALUE a K\" on two runs of its own, so the value is comparable -- drop the marker"` |
| **L** — `compare_values: false` for the introspection driver, no refresh | the table test red naming **8** rows — `Class~enhanced` and the seven `no-value` — each measured `agree`; `every_unstable_row_is_really_unstable` stays green; exactly 1 red | **CONFIRMED except the count.** exit 101, 1 failed; the unstable test green; exactly 8 rows moved *verdict* and they are exactly the eight predicted. **FALSIFIED:** 124 rows were named, not 8 — the other 116 moved only their evidence, because the flag also changes what the evidence quotes (`oracle rc0 SENT` against `oracle rc0 VALUE a Supplier`). The flag is a bigger change to the table than the verdict column shows |
| **M** — move `corpus/introspection-arity.tsv` away | the table test red on its own assert; `the_engines_agree_with_one_another` red with the new message; exactly 2 | **CONFIRMED, all three.** exit 101, 2 failed, `introspection-arity.tsv is missing, so this reads green over nothing`. Before this round that second test was green |
| **N** — move `corpus/introspection-scopes.tsv` away | `the_table_matches_the_interpreter`, `every_documented_row_resolves_to_a_scope` and `the_kind_column_agrees_with_the_two_it_governs` all red; exactly 3 | **CONFIRMED, all four.** exit 101, 3 failed, the new message on the two that had none |

Every edit was restored from a `cp` copy taken before it, never with `git checkout --`, and the
driver and the argument file were re-run green afterwards: 25 passed, 0 failed.

**No registration control**, for the reason §6 gives: this round binds no name in `NATIVE_METHODS`
and adds no method body.

## 12. Runtime

`introspection_arity` is **10.80s** for 192 rows, unchanged within noise from the 10.57s §7a
records, even though the round adds a fourth `measured()` caller and two extra oracle runs for the
unstable rows: the cost is process startup and the callers run in parallel test threads.
`introspection_scopes` is 0.19s. Eight later tasks re-run both.

## 13. Fast checks, in the working tree, before the commit

* `cargo fmt --all --check` — exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
* `cargo test --release --workspace --no-fail-fast` — exit 0, no `test result: FAILED` line
  anywhere, and 116 `test result: ok` lines.
* `git status` before staging: the seven files of this round and nothing else; `Cargo.lock`
  untouched.

## 14. Gates

Run in the gate worktree `/home/moritz/dev/repos/ooRexx-5i-gates`, detached at **`598fd722a`**, the
last of this round's three commits. The worktree's own `HEAD` is recorded beside the sha in the
status file, so the gated tree is the committed tree by construction. Four of the controller's plan
commits -- `e0646e5ff`, `b37180058`, `81aa7255d` and `6f5a60db2` -- are interleaved among this
round's on the branch and are therefore covered by this run as well; a reader counting commits
between the run and the task would otherwise wonder what they were.

| gate | command | result |
| --- | --- | --- |
| G1 | `cargo fmt --all --check` | exit 0 |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| G3 | `cargo test --release --workspace --no-fail-fast` | exit 0, 116 `test result: ok`, no `FAILED` |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | exit 0, 116 `test result: ok`, no `FAILED` |
| G5 | `cargo test --release -p rexx-exec --test collection_arity` | exit 0, 23 passed |
| G6 | `cargo test --release -p rexx-exec --test introspection_arity` | exit 0, 25 passed |
| G7 | `cargo test --release -p rexx-exec --test introspection_scopes` | exit 0, 22 passed |

**G3 and G4 are the second reading of those two commands. The first was void, not failed.**
`ootest/` and `oodocs/` are gitignored SVN working copies that live inside the working tree, and
`git worktree add` takes tracked files only, so the pinned worktree had neither: both runs came back
101 with 30 failed tests and exactly 30 panics, from two sites, every one naming an absent path
under those directories -- `crates/rexx-extract/tests/extract_docs.rs:54` says so in its own message.
Failed-test count equalling panic count is what rules out any other failure hiding among them, and
no binary of this task's was red inside either. The controller symlinked both directories into the
worktree and re-ran G3 and G4 on this same pinned sha.

**A symlink can produce a false green as easily as a missing directory produced a false red**, so
the re-run was read for work done rather than for its exit status: the eight binaries that had been
red -- `assertions` 5, `bif_assertions` 5, `keyword_assertions` 7, `ir_dual` 9, `extract_assertions`
19, `extract_bif` 10, `extract_docs` 9, `extract_keyword` 22 -- all passed with non-zero counts, the
four differentials taking 8.15s, 17.64s, 16.62s and 20.21s. They ran; they did not skip.

**G1 and G2 read 0 off a warm worktree target and are provisional to that extent**, per
`rust/CLAUDE.md`'s rule that a same-session green is only evidence if the linter re-examined the
code. Every per-task run in this phase is warm, so the clean-target-directory reading cannot
honestly live in one of them; the controller has put it in the phase's close instead.

## 15. What this round did not do

Two of the review's minor findings are outside the nine and are deferred to the whole-branch
review by the controller, so that a later reader does not mistake them for closed: **5.2**,
`read_table`/`corpus_root` living in the arity module rather than in a corpus one; and **5.3**,
`introspection_scopes.rs`'s `NO_EVIDENCE` doing duty as both the table's placeholder and the
directives sentinel where `arity::NONE` is the constant that means that.

**6.2 is done**, on the controller's instruction: `support/arity.rs`'s "an earlier version of the
rule asked only for oracle exit 0" is deleted. It was history, which `rust/CLAUDE.md` excludes, and
the move that carried it here had dropped the measurement behind it and kept the narrative.
