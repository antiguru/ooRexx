# Adversarial review — `docs/superpowers/plans/2026-08-27-phase-5b.md`

Subject at `d7bec3ae3` (`Plan Phase 5b`), read end to end, every line. Binding spec:
`docs/superpowers/specs/2026-08-27-phase-5b-instances.md` at `01e024157`, D57-D69. The spec's own
adversarial review (`.superpowers/sdd/2026-08-27-phase-5b/spec-review.md`) was read first and its
findings are not re-reported against the spec; where the plan inherited one it is named as such.

All probes run from `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/planrev5b/`, process
CWD a fresh directory, three descriptors read separately, never `2>&1`. Oracle under
`( ulimit -v 1048576; LD_LIBRARY_PATH=… timeout -s KILL 10 … )`, crate under
`( memcap 1G env REXX_ENGINE=… timeout -s KILL 20 … )` on **both** engines. `rust/corpus/oracle-crashes.txt`
was read before any probe was written and no listed shape was constructed. No `cargo` was run and no
repository file was edited except this report.

## Verdict

The plan's citations are unusually clean: every C++ line (`ClassClass.cpp:860`-`:862`, `:531`-`:533`,
`:962`-`:964`, `:1361`, `:1413`, `:1036`, `:1882`; `ObjectClass.cpp:697`) is exact, every Rust
citation (`dispatch.rs:1123`, `run.rs:3077`, `lib.rs:6446`, `body.rs`, `gate_tables/mod.rs:344`,
`gate_table_d.rs:317`) is exact, every measured oracle transcript I re-ran reproduces byte for byte,
and the two counts I could check independently (23 of 59 classes refusing a bare `~new`; 264 corpus
entries) come out exactly right. What fails is the layer above: **the acceptance criteria**. Six
findings are high, and five of the six are the same shape the plan's own Rule 2 was written to
prevent — a criterion that is satisfiable while the mechanism it names is unbuilt. Task 7's named
`~copy` control cannot redden a witness written from Task 7's own measured paragraph (F2, verified by
running the discriminator). Task 1's rooting instrument runs over a subset file Task 1 does not own,
and no ordinary program assigns over `SELF`, so the use-after-free it names cannot reach it (F4).
Task 1 adds six instance methods with no acceptance criterion at all, one of which (`~defaultName`)
does not exist for *any* receiver in the crate, beside a paragraph implying it does (F5). Task 5's
`obdes` row cannot see the inherited class-`UNINIT` arm, and the three `UNINIT` programs 5a's Task 7
handed to 5b in writing — one of them a live silent wrong answer at HEAD, reproduced here — are owned
by no task (F6). Nobody owns creating `corpus/phase-5b.txt` or wiring it into the **five** duplicated
`SUBSET_FILES` literals, one of which has no directory guard and fails silently; 5a had a dedicated
task for exactly this and said so in the file's own header (F3). And "Tasks 2 to 8 are independent"
is false in the direction that costs work: Task 5's delivery changes the crate's answer for the exact
program Task 6 must commit as an asserted divergence (F1). The plan is salvageable everywhere; no
decision in it is wrong, and the fixes are almost all one added sentence naming a witness or an owner.

Twenty-four findings: **6 high, 13 medium, 5 low.**

---

## F1 — Task 5 changes the crate's answer for Task 6's committed witness; "Tasks 2 to 8 are independent" is false

**SEVERITY: high**

Line 14-16:

> **Delivery order is not free.** Task 1 unblocks every other task: no instance exists until `~new`
> does, so nothing after it can be measured before it lands. **Tasks 2 to 8 are independent of each
> other once Task 1 is in.**

and Task 6, line 251-254:

> The licence covers four consequences and this task owns the one 5b makes reachable: a class's
> `UNINIT` running in the termination sweep rather than at a driven collection. The witness runs both
> interpreters and **asserts the expected divergence**, so it reddens if the crate ever starts matching
> or starts diverging differently.

**What is wrong.** Task 6's witness is D59a's first row, whose program is the spec's licence transcript.
Its crate side has **three** lines today and **four** after Task 5's delivery 3 lands, in a different
position from the oracle's. A DEVIATION written before Task 5 pins today's three-line answer; Task 5
then makes it "start diverging differently", which is precisely the condition Task 6 says must redden.
The plan's own rule ("Prefer deleting to rewriting") and the DEVIATION vocabulary (`phase-4-exclusions.txt`:
"Adding a row to either is a plan amendment, not a file edit") both push against the edit that would
then be needed.

**Evidence.** The witness program, run at `d7bec3ae3`:

```rexx
say 'start'
k = .Object~subclass('K', .Meta)
say 'built' k~id
drop k
call gc 'force'
say 'after-gc'
exit 0
::CLASS Meta SUBCLASS Class
::METHOD uninit
  say 'CLASS UNINIT RAN'
```

```
oracle rc 0        start / built K / CLASS UNINIT RAN / after-gc     stderr empty
crate  rc 0 (ir)   start / built K / after-gc                        stderr empty
crate  rc 0 (tw)   start / built K / after-gc                        stderr empty
```

Under D60 ("Every class object that defines one gets one", sweep at termination) the crate's stdout
after Task 5 is `start / built K / after-gc / CLASS UNINIT RAN`. The divergence does not disappear —
it moves — so the assertion's expected crate bytes change.

**Smallest correct replacement.** In the delivery-order paragraph: *Task 6 depends on Task 5 and must
follow it: the divergence it asserts is "the class's `UNINIT` runs in the termination sweep", which
the crate does not do until Task 5's delivery 3 lands. Written before that, the DEVIATION pins
"never runs" and Task 5 reddens it.* And amend line 15 to "Tasks 2, 3, 4, 7 and 8 are independent of
each other once Task 1 is in; Task 6 follows Task 5."

---

## F2 — Task 7's named `~copy` control cannot redden a witness written from Task 7's own measured paragraph

**SEVERITY: high**

Line 270-271 and line 290-292:

> **Measured, oracle, all rc 0 unless noted.** `~copy` is shallow, copies the object's own scope and
> its exposed variables, and `(o == c)` is `0`.

> **Done when** each of the four has a witness that agrees on both engines, and **the control is
> recorded as run**: making `~copy` share the receiver's scope pools instead of copying them reddens the
> `~copy` witness at rc 0.

**What is wrong.** A witness built from that measured paragraph reads the copy's exposed variables and
asks `(o == c)`. **Both answers are identical under shared pools.** The copy's variables have the
receiver's values whether the pools were copied or shared, and identity is a separate fact. The
control cannot redden such a witness, so the acceptance is met with `~copy` sharing pools — a silent
wrong answer at rc 0 with empty stderr. Only a *write through one and read back the other* separates
them.

**Evidence.** Oracle, rc 0:

```rexx
o = .K~new ; o~set('orig') ; c = o~copy
say 'a' c~get          -- a orig      <- same under shared pools
say 'b' (o == c)       -- b 0         <- same under shared pools
c~set('changed')
say 'c' o~get          -- c orig      <- THE discriminator
say 'd' c~get          -- d changed
::CLASS K
::METHOD set
  expose v
  use arg v
::METHOD get
  expose v
  return v
```

```
a orig | b 0 | c orig | d changed      rc 0, stderr empty
```

Lines `a` and `b` are exactly what line 270-271 states; lines `c` and `d` are what the control needs
and what the plan does not ask for.

**Smallest correct replacement.** Extend line 270-271: *`~copy` is shallow, copies the object's own
scope and its exposed variables, and `(o == c)` is `0`. **The copy's initial values are the same
whether the pools were copied or shared, so the witness must write through the copy and read the
receiver back**: after `c~set('changed')` the oracle answers `o~get` `orig` and `c~get` `changed`.*

---

## F3 — no task owns creating `corpus/phase-5b.txt` or wiring it into the five `SUBSET_FILES` literals, one of which fails silently

**SEVERITY: high**

Tasks 2, 3, 5 and 7 each say "`corpus/phase-5b.txt` carries …" (lines 127, 166, 234, 290's
neighbourhood). No task creates the file, and none mentions the wiring.

**What is wrong.** `SUBSET_FILES` is duplicated in **four** integration-test binaries, each guarded
against a non-recursive `read_dir` of `corpus/` filtered to `phase-*.txt`, plus a **fifth**
hand-maintained literal with no guard at all:

```
crates/rexx-exec/tests/corpus.rs:653          const SUBSET_FILES   (guarded, :689)
crates/rexx-exec/tests/coverage.rs:671        const SUBSET_FILES   (guarded, :709)
crates/rexx-exec/tests/collect_stress.rs:137  const SUBSET_FILES   (guarded, :383)
crates/rexx-exec/tests/ir_dual.rs:1181        const SUBSET_FILES   (guarded, :1471)
crates/rexx-exec/tests/trace_oracle.rs:686    inline literal       NO GUARD
```

`trace_oracle.rs:674`-`:679` states the consequence itself: *"**This literal has no directory-listing
guard** … a phase subset file added and forgotten *here* would silently keep this check measuring the
union as it stood before."* `coverage.rs` additionally pins each file's contents against a per-phase
literal (`EXPECTED_SUBSET_5A`, `:884`, test at `:1210`); nothing forces an `EXPECTED_SUBSET_5B` to
exist, so its absence is silent too.

**Evidence that this is a known task-sized job, not incidental plumbing** — `corpus/phase-5a.txt`'s
own header, lines 10-13:

> Task 1 built the harness plumbing this file needs (an unnormalised stderr comparison mode, a
> directive walker admitting `::CLASS`/`::METHOD` without panicking, and **this file wired into every
> harness that reads a phase subset list**) and added no corpus program of its own.

5b's plan has no such task. Whichever of Tasks 2/3/5/7 lands first inherits four red tests and one
silent narrowing, which also falsifies their claimed independence.

**Smallest correct replacement.** Give Task 1 (or a new Task 1b) the sentence: *Create
`corpus/phase-5b.txt` in `phase-5a.txt`'s shape and wire it into every harness that reads a phase
subset list — `SUBSET_FILES` in `corpus.rs`, `coverage.rs`, `collect_stress.rs` and `ir_dual.rs`, the
unguarded literal in `trace_oracle.rs:686`, and a new `EXPECTED_SUBSET_5B` with its
`phase_5b_subset_matches_the_committed_list` test. Four of the six are guarded and go red on the
commit that creates the file; two are silent, which is why they are named.*

---

## F4 — Task 1's rooting instrument cannot see the hazard it is named for

**SEVERITY: high**

Line 96-107:

> **The rooting hazard is this task's** … a running send's receiver is rooted only by the `SELF` slot,
> which a method body may assign over … it is a use-after-free rather than a wrong answer. The
> instrument is `run_program_collect_every_alloc`, which collects on every allocation; a watermark run
> cannot see a missed root.
>
> **Done when** both rows agree on both engines, **the collect-on-every-allocation stress mode passes
> over the new corpus programs**, …

**What is wrong, three ways.**

1. **"the new corpus programs" has no referent.** Task 1's Done-when names no corpus program, and
   Task 1 is not the task that creates `corpus/phase-5b.txt` (F3).
2. **The subset-wide stress harness reads `SUBSET_FILES` only.** `collect_stress.rs`'s
   `the_l0_subset_passes_again_under_collect_on_every_allocation` (`:394`-`:440`) walks the union of
   the phase subset files. Task 1's two rows, `corpus/gate-tables/concepts/creo.rex` and `abscla.rex`,
   are gate-table probes, which are **not** in any subset file — `gate_table_d.rs:30`-`:34`: *"These
   probes are not corpus programs. They live under `corpus/gate-tables/`, not in a phase subset file."*
   So the instrument as invoked sees neither of Task 1's rows.
3. **Even if they were listed, no ordinary program provokes the hazard.** The hazard is a body that
   assigns over `SELF` and then allocates. That construct is legal and reachable, measured on the
   oracle, rc 0:

```rexx
o = .K~new
say 'a' o~go
::CLASS K
::METHOD init
  expose v
  v = 'kept'
::METHOD go
  expose v
  self = 'clobbered'
  t = ''
  do i = 1 to 100
    t = t || i
  end
  return v length(t) self
```
```
a kept 192 clobbered        rc 0, stderr empty
```

Nothing in `creo.rex`, `abscla.rex` or any plausible construction witness assigns over `SELF`, so the
stress run passes with the root missing. `collect_stress.rs` already holds five *targeted* cases for
exactly this class of hazard (`:511`, `:572`, `:611`, `:691`, `:791`), which is the shape the plan
should have named.

**Smallest correct replacement.** Replace the Done-when clause with: *a committed program that assigns
over `SELF` inside a method body and then allocates enough to force a collection agrees with the
oracle on three descriptors and passes under `run_program_collect_every_alloc`, added both to
`corpus/phase-5b.txt` and as a targeted case beside `collect_stress.rs`'s existing five. The
subset-wide stress run cannot see this hazard: it walks `SUBSET_FILES`, which contains no gate-table
probe, and no ordinary program assigns over `SELF`.*

---

## F5 — Task 1 builds six instance methods with no acceptance criterion, and one of them does not exist for any receiver

**SEVERITY: high**

Line 74-78 and line 86-89:

> **What is already there, probed separately.** … `~objectName`/`~objectName=`/`~string`/`~class`/`~isA`
> all answer for a class receiver. **Re-measure all of this.**

> Also in this task, because they are the same receiver kind and **each is a silent-wrong-answer risk on
> its own**: an instance's `~string`, **`~defaultName`**, `~objectName` and `~objectName=`, its `~class`
> and `~isA`.

**What is wrong.**

*First*, the "already there" list omits `~defaultName`, and `~defaultName` **is not implemented for any
receiver**. Re-measured as instructed:

```rexx
say 'a' .K~objectName        -- a The K class
say 'b' .K~string            -- b The K class
say 'c' .K~class~id          -- c Class
say 'd' .K~isA(.Class)       -- d 1
.K~objectName = 'zed'
say 'e' .K~objectName        -- e zed
say 'f' .K~string            -- f zed
say 'g' .K~defaultName
::CLASS K
```
```
oracle rc 0:   a..g all answer, g = The K class
crate  rc 120 (both engines), after printing a..f:
               rexx-exec: method "DEFAULTNAME" of class "Class" is not implemented (Phase 5)
```

`/bin/grep -rn "DEFAULTNAME" crates/rexx-exec/src/` returns **nothing**. So five of the six named
methods need only a second receiver kind and the sixth needs building from zero, and the paragraph
directly above implies otherwise.

*Second*, **none of the six has any acceptance criterion.** Task 1's Done-when is `creo` + `abscla` +
the stress run + two controls. `creo.rex` sends `~type`, `~balance`, `~rate`; `abscla.rex` sends
`.ab~id .ab~class~id` to a **class**. Neither sends `~string`, `~defaultName`, `~objectName`,
`~objectName=`, `~isA`, or `~class` to an instance. Nor does any other gate row: the Object instance
method row is 31 `hasMethod` calls and nothing else —
`corpus/gate-tables/methods/object__instance.rex`, `/bin/grep -c "o~string" ` is `0` — and those rows
are `METHOD_PHASE = "5c"`, reported and not gated. The plan calls each of these a silent-wrong-answer
risk and then gives them no witness.

*Third*, `~isA` on an instance appears nowhere in the spec (`/bin/grep -in "isA"` over the spec: two
hits, neither this), so it is plan-added scope.

**Smallest correct replacement.** Fix line 77-78 to `~objectName`/`~objectName=`/`~string`/`~class`/`~isA`
*answer for a class receiver; `~defaultName` answers for no receiver at all and is built here from
zero*, and add to the Done-when: *`corpus/phase-5b.txt` carries the instance-naming witness — the
program above with an instance receiver, whose oracle answer is `a an Object / b a K / c a K / d a K /
e zed / f zed / g a K` at rc 0 — since no gate row sends any of these six messages to an instance.*
(Measured here, oracle rc 0, exactly those bytes.)

---

## F6 — the three `UNINIT` programs 5a handed to 5b are owned by no task, one is a live silent wrong answer, and `obdes` cannot see the inherited arm

**SEVERITY: high**

Task 5, line 234-239, whose Done-when is `obdes` + a one-instance collection delivery + a one-instance
termination delivery + the order question + the control-text edit.

**What is wrong.** 5a's Task 7 report records three class-`UNINIT` programs and states in terms that
they travel to 5b (`docs/superpowers/records/2026-08-17-phase-5a/task-7-report.md:333`-`:348`):

> **The third row is a debt this task creates.** Its shape changed from a loud refusal, which cannot be
> mistaken for an answer, to a silent wrong answer at rc 0. 5b owns the firing … **All three programs
> travel to 5b together.**

Neither the spec's "Handover from 5a" (four items, none of them these) nor any task in this plan names
them. `obdes.rex` has exactly one class with a `UNINIT` **that the class itself declares**, so Task 5's
only gate row cannot see either of the two arms these programs pin.

**Evidence, both re-measured at `d7bec3ae3`, three descriptors, both engines.** The inherited arm —
`K` declares no `UNINIT` and fires anyway, *before* the class that does:

```rexx
say 'main'
::CLASS P
::METHOD uninit CLASS
  say 'uninit on' self~id
::CLASS K SUBCLASS P
```
```
oracle rc 0:  main / uninit on K / uninit on P     (10 runs of 10, identical)
crate  rc 0:  main                                 (ir and tree-walker, stderr empty both)
```

and the `MIXINCLASS`/`INHERIT` arm, the one 5a's report calls the debt it created:

```rexx
say 'main'
::CLASS M MIXINCLASS Object
::METHOD uninit CLASS
  say 'uninit on' self~id
::CLASS K INHERIT M
::METHOD uninit CLASS
  say 'uninit on' self~id
```
```
oracle rc 0:  main / uninit on K / uninit on M
crate  rc 0:  main                                 (ir and tree-walker, stderr empty both)
```

Both are silent wrong answers today. The first also bears on Task 5's order question: `P` is declared
first and fires second, which constrains any rule the task proposes.

**Smallest correct replacement.** Add to Task 5: *5a's Task 7 handed three class-`UNINIT` programs to
this task (`task-7-report.md:333`-`:348`) and the last of them is a silent wrong answer this crate
carries today. `obdes.rex` has one class declaring its own `UNINIT` and sees neither the inherited arm
(a subclass with no `UNINIT` of its own fires one, before its parent) nor the `INHERIT` arm. Both must
agree. **Under D61 neither can be a corpus row until the order question is answered**, so if it is not,
record them as still open with their measured transcripts rather than dropping them.*

---

## F7 — Task 5's "what is already there" makes the task look smaller: the class-object side of the flag does not exist

**SEVERITY: medium**

Line 208-213:

> **What is already there.** … `ClassGraph::check_uninit` and `parent_has_uninit` are 5a's half of the
> propagation. **Re-measure all of this.**

**What is wrong.** Re-measured: those two flags are about a class's **instances** and serve deliveries
1 and 2 only. The predicate delivery 3 needs — "does this *class object* respond to `UNINIT`" — does
not exist, and the crate's own doc says so. `crates/rexx-classes/src/class_graph.rs:366`-`:381`:

```rust
/// Whether `class`'s instances need `UNINIT` -- oracle's `hasUninitDefined`.
pub fn has_uninit(&self, class: ObjRef) -> bool { … }

/// Oracle's `RexxClass::checkUninit` …, the half of it this crate can model: set
/// [`Self::has_uninit`] when the class's **flattened instance behaviour** answers `UNINIT` …
///
/// **The other half is not here and is not a flag.** `checkUninit` goes on to
/// `if (hasUninitMethod()) requiresUninit();` (`:1222`), which asks the class *object's own*
/// behaviour -- the class side, where a `::METHOD uninit CLASS` lands -- and enters the object
/// in the collector's uninit table. This crate has no such table.
```

`obdes.rex`, Task 5's own gate row, is a `::METHOD uninit CLASS`, i.e. exactly the side with no flag.
The spec quotes this doc comment; the plan's paraphrase drops the half that matters and leaves the
reader with "the flags are there".

**Smallest correct replacement.** *`ClassGraph::check_uninit` and `parent_has_uninit` are 5a's half and
they answer about a class's **instances**, which is deliveries 1 and 2. Delivery 3 has no predicate at
all: `class_graph.rs:371`-`:381` records that the class-object side — where a `::METHOD uninit CLASS`
lands — needs the table the oracle's `requiresUninit` fills and "this crate has no such table". That
table is this task's to build.*

---

## F8 — Task 7's `~run` witness is satisfiable with `~run` unimplemented, and `~run`'s option surface is not enumerated

**SEVERITY: medium**

Line 272-273 and line 290:

> `o~run(...)` is `97.2` at rc 159 from a program context -- Task 3's check, reached again here.

> **Done when** each of the four has a witness that agrees on both engines …

**What is wrong.** The only measured behaviour of `~run` in the whole plan is its **refusal**. A witness
built from it agrees as soon as `RUN` is registered as a restricted-private name that the D53 check
refuses at dispatch — with no body at all, because the 97.2 fires before the body runs (the plan says
so itself at line 155-157). So Task 7's acceptance is met with `~run` unbuilt. And `~run` is one of
the few mechanisms in this phase with a real option surface, which nobody has enumerated:
`fundclasses.xml` `mthObjectRun` (`oodocs/rexxref/en-US/fundclasses.xml:3160`-`:3200`) — a Method
object, **or** a source string, **or** an Array of source strings; then `Individual` or `Array`, "You
need to specify only the first letter"; with neither, the method runs without arguments.

**Evidence, the allowing arm, oracle rc 0** — the thing no witness in the plan asks for:

```rexx
o = .K~new
say 'a' o~go
say 'b' o~peek
::CLASS K
::METHOD init
  expose v
  v = 7
::METHOD peek
  expose v
  return v
::METHOD go
  return self~run('use arg x; return "ran" x', 'I', 5)
```
```
a ran 5 | b 7        rc 0
```

Also measured: the one-off's `EXPOSE` reaches the object's **float** pool, not the class's — the same
program with `self~run('expose v; return v*10')` raises 41.1 on `("V")`, rc 215, because `v` is
uninitialised there. That is a D67 interaction the plan does not mention.

**Smallest correct replacement.** *`o~run(…)` from a program context is `97.2` at rc 159 -- Task 3's
check, reached again here, **and a witness of that refusal alone is met by registering `RUN` as a
restricted-private name with no body**. The allowing arm must be witnessed too: from a method,
`self~run('use arg x; return "ran" x', 'I', 5)` answers `ran 5`, oracle rc 0. `mthObjectRun`'s option
surface is a Method object, a source string or an Array of them, times `Individual`/`Array`/neither,
first letter only; enumerate it before building and say which arms this phase owes.*

---

## F9 — Task 4 builds `FORWARD` with four of its six documented options unmeasured, unenumerated and unwitnessed

**SEVERITY: medium**

Line 182-186 and 193:

> **Build.** `FORWARD` as an instruction, then `DELEGATE` as the equivalence `dire.xml` states it as …
> Measured, a plain `forward class (super)` returns from the forwarding method rather than continuing it.

> **Done when** both rows agree on both engines with the replaced probes, and **the control is recorded
> as run** …

**What is wrong.** `FORWARD` has six documented options and the plan measures two. The spec's own
"What I could not check" says exactly this — *"`ARGUMENTS`, `ARRAY`, `MESSAGE`, `CONTINUE` and their
interactions were not [measured], and `instrc.xml` `keyForward` was located but not read in full"* —
and the plan neither repeats the warning nor tells the implementer to measure them, which it *does* do
for the `send` array form at line 281-282 ("measure it before building it"). Both replacement probes
exercise `to()` alone, so four options would be built with no acceptance criterion anywhere.

Separately, **D64 puts "`FORWARD` as an instruction" in `corpus/phase-5b.txt`** (spec line 914) and
Task 4's Done-when contains no corpus witness at all — the only 5b mechanism in D64's list with none.

**Evidence.** `awk '/id="keyForward"/,/<\/section>/' oodocs/rexxref/en-US/instrc.xml | grep -oE '<option>[A-Za-z]+</option>' | sort -u`:

```
ARGUMENTS  ARRAY  CLASS  CONTINUE  MESSAGE  TO
```

And the instruction is measurable with **no instance at all**, so this witness does not wait on Task 1:

```rexx
say 'a' .K~cm
::CLASS Base
::METHOD cm CLASS
  return 'base-cm'
::CLASS K SUBCLASS Base
::METHOD cm CLASS
  forward class (super)
```
```
oracle rc 0:  a base-cm
crate  rc 120 (both engines):  stderr  rexx-exec: FORWARD is not implemented (Phase 5)
```

**Smallest correct replacement.** Add to Task 4: *`instrc.xml` `keyForward` names six options —
`TO`, `CLASS`, `MESSAGE`, `ARGUMENTS`, `ARRAY`, `CONTINUE`. `class` and `to` are measured; the other
four are not, and the spec records that. Measure them before building, and say which this phase owes.
`corpus/phase-5b.txt` carries a `FORWARD`-as-an-instruction witness (D64's list has one and this
task's Done-when had none); the class-method shape above needs no instance and can land before Task 1.*

---

## F10 — "Task 1 unblocks every other task" is false for Tasks 6 and 8 and for half of Task 4

**SEVERITY: medium**

Line 14-15:

> **Task 1 unblocks every other task:** no instance exists until `~new` does, so nothing after it can
> be measured before it lands.

**What is wrong, measured three ways.**

* **Task 8 needs no instance.** `methodsbyclass.rex` refuses at `.array~new`, not at `Object~new`:
  `crate rc 120, stderr rexx-exec: method "NEW" of class "Array" is not implemented (Phase 5)`, both
  engines. `Body::Array` is its own body kind and `receiver_kind` maps it to `Ok(Primitive::Array)`
  (`dispatch.rs:1122`), so nothing in that row touches `Body::Instance`.
* **Task 6 needs no instance.** Its witness program runs to completion on the crate today at rc 0
  (transcript in F1).
* **Task 4's `FORWARD` half needs no instance** (transcript in F9).

**Smallest correct replacement.** *Task 1 unblocks Tasks 2, 3, 5's instance arm, 7 and 9. Tasks 6 and 8
and Task 4's `FORWARD` half need no instance and may land before it; Task 6 must follow Task 5.*

---

## F11 — Rule 2's justification is false for two of the probes it names, and names three where it lists four

**SEVERITY: medium**

Line 48-52:

> 2. **A witness must be run against a control that makes it fail.** **Three probes** in this
>    neighbourhood **were green** over mechanisms that were not implemented: gate table D's two
>    `DELEGATE` rows, `usesem.rex` for the per-object precedence step, and a replacement `DELEGATE`
>    probe drafted during the spec review.

**What is wrong.** The rule is right and the finding it rests on is real; the sentence stating it is
not. **`usesem.rex` has never been green.** Measured at `d7bec3ae3`:

```
oracle rc 0:  setmethod one-off / not-shared 0 / enhanced enhanced / still-not-shared 0
crate  rc 120 (ir and tree-walker), stdout empty
       stderr  rexx-exec: method "NEW" of class "Object" is not implemented (Phase 5)
```

Its actual defect (spec F13) is that it *would* be green over an implementation that searched the class
first — a counterfactual, not a history. The drafted `k~at` probe was likewise never run as a row.
Only the two `DELEGATE` rows were measured green, and I re-confirm both agree (`main`, rc 0, empty
stderr, oracle and both engines). The list also names **four** probes under the count "three".

**Smallest correct replacement.** *Two committed probes in this neighbourhood are green today over a
mechanism that is not implemented — gate table D's two `DELEGATE` rows — and two more would be green
over a wrong implementation: `usesem.rex`, which defines `EXTRA` with no class-level `EXTRA` to lose
to, and a `k~at` draft of the replacement `::ATTRIBUTE` probe. Each task below names its control …*
(Per `rust/CLAUDE.md`, name the set rather than its size.)

---

## F12 — Task 3's "the probe needs" paragraph omits `StringTable~[]=`, which the crate does not have

**SEVERITY: medium**

Line 140-142:

> **Measured.** rc 120 today; oracle rc 0, … The probe needs `.StringTable~new` and `Class~enhanced` as
> well as `setMethod`.

**What is wrong.** `usesem.rex` line 10 is `t["ENHANCED"] = "return 'enhanced'"`, a `[]=` send to a
`StringTable`. `/bin/grep -rn -F '[]=' crates/rexx-exec/src/` returns **nothing** — no `[]=` exists for
any receiver. The native table has `("StringTable", "[]", …)`, `"AT"`, `"PUT"` and `"UNKNOWN"`
(`dispatch.rs:443`-`:451`) and no setter.

**Evidence that the syntax parses and only the method is missing**, so this is a build item and not a
parser one:

```rexx
a = (7,8,9)
say 'r' a[2]
a[2] = 5
say 's' a[2]
```
```
oracle rc 0:  r 8 / s 5
crate  rc 120 after printing `r 8`:  rexx-exec: method "[]=" of class "Array" is not implemented (Phase 5)
```

**Smallest correct replacement.** *The probe needs `.StringTable~new`, `StringTable`'s `[]=` (which no
receiver kind has today) and `Class~enhanced` as well as `setMethod`.*

---

## F13 — `Message` is assigned to Task 7 and to 5c in adjacent sentences

**SEVERITY: medium**

Line 91-94:

> **5b owes only the constructions its own rows make** … `Object` and user classes here, `StringTable`
> in Task 3, `Array` in Task 8, **`Message` in Task 7**.
> **Twenty-three of the fifty-nine** `*__instance.rex` classes refuse a bare `~new` on the oracle and
> **are 5c's; do not build them** and do not treat their rows as this phase's business.

**What is wrong.** `Message` is one of the twenty-three. I ran all fifty-nine bare constructions on the
oracle; the count is exactly 23 and the classification is exactly the plan's, and `Message` refuses
`93.901`:

```
Message rc=163 Error 93.901        Class rc=163 Error 93.901       String rc=163 Error 93.903
(23 refusals total: 93.901 x11, 88.901 x3, 93.903 x3, 93.967 x4, 97.1 x2; 36 answer at rc 0)
```

So one paragraph tells the implementer to build `Message` construction in Task 7 and the next tells
them not to build it. The two obligations are actually different — 5b owes the object `~start`
*answers*, 5c owes `.Message~new` — and the spec has the same tension (its split section names
`Message`, its "could not check" says `.Message~new` was not settled). The plan makes it sharper by
putting the two sentences three lines apart.

**Smallest correct replacement.** *… `Array` in Task 8, and the `Message` object `~start` answers in
Task 7 — **not** `.Message~new`, which refuses `93.901` on the oracle and is 5c's with the other
twenty-two.*

---

## F14 — "the spec prints both" is false: there is no `~uninherit` program in the spec

**SEVERITY: medium**

Line 127-129:

> `corpus/phase-5b.txt` carries the `~inherit` and `~uninherit` witnesses (**the spec prints both**, and
> neither exists in the corpus today -- no committed `.rex` combines `~inherit` with `~new`)

**What is wrong.** The spec prints the `~inherit` program (its lines 202-212) and never prints a
`~uninherit` one. `/bin/grep -in "uninherit"` over the spec gives four hits — lines 227, 921, 990,
1031 — all prose, no program. The implementer sent to the spec for it will not find it.

The rest of the parenthesis **holds**: over `rust/corpus/`, of the fourteen `.rex` files matching
`~inherit|~uninherit`, none also matches `~new` (`gate-tables/methods/class__instance.rex` writes
`hasMethod("inherit")`, not `~inherit`, so it is correctly excluded).

**Evidence for the missing arm, measured here so the plan can carry it**, oracle rc 0:

```rexx
k = .Object~subclass('K')
k~inherit(.Mx)
o = k~new
say 'a' o~hasMethod('MXM')
say 'b' o~mxm
k~uninherit(.Mx)
say 'c' o~hasMethod('MXM')
signal on syntax name h
say 'd' o~mxm
h: say 'e trapped' rc
exit 0
::CLASS Mx MIXINCLASS Object
::METHOD mxm
  return 'mixin-ran'
```
```
a 1 | b mixin-ran | c 0 | e trapped 97        rc 0
```

**Smallest correct replacement.** Replace "the spec prints both" with *the spec prints the `~inherit`
arm and states the `~uninherit` arm without a program; measured, `k~inherit(.Mx)` / `o = k~new` /
`k~uninherit(.Mx)` gives `a 1 / b mixin-ran / c 0 / e trapped 97` at rc 0.*

---

## F15 — no invocation in the plan can make a 5b row red before Task 10

**SEVERITY: medium**

Eight tasks use "agrees" and "reddens" as their acceptance verb (lines 104, 127, 166, 193, 234, 263,
290, 310).

**What is wrong.** A gate-table verdict mismatch is an exit status *only* under `REXX_CORPUS_GATE` and
*only* for a row whose phase is closing (`REXX_PHASE_GATE`) or already closed (`CLOSED_PHASES`) —
`gate_tables/mod.rs:365`-`:370`, and `gate_table_c.rs:45`-`:51`: *"A verdict failure is an exit status
only under `CORPUS_GATE_ENV` and only for a row whose owning phase is closing … or already closed."*
`CLOSED_PHASES` is `&["5a"]` (`:344`) until Task 10. So under the five gate commands, **every 5b row
can be red and all five exit zero**, and a control "recorded as run" as reddening a 5b row has to name
an invocation the plan never gives. `/bin/grep -n "PHASE_GATE"` over the plan, the spec and
`docs/superpowers/records/2026-08-17-phase-5a/global-constraints.md` returns **nothing**.

5a did not have this gap: `docs/superpowers/plans/2026-08-17-phase-5a.md:2365` reads *"**Done when**
every 5a row in both tables reads `agree` **under `REXX_PHASE_GATE=5a`**"*, and `:592` names the full
command.

**Smallest correct replacement.** In the global constraints: *A 5b row's verdict reddens nothing until
Task 10, because `CLOSED_PHASES` holds only `"5a"`. Every "agrees" and "reddens" below is read under
`REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test
gate_table_d`, and that command is quoted beside the figure like any other.*

---

## F16 — Task 8's acceptance cannot see the index mapping, and the equivalence arm does not close it

**SEVERITY: medium**

Line 301-312:

> **Build.** `.Array~new(2, 3)`, `~dimension`, and `[]`/`[]=` with several indexes. …
> **Add the arm the section states and the probe omits.** … **Done when** the row agrees on both
> engines with the equivalence arm in the probe, and **the control is recorded as run**: the row's
> committed control (route `matrix[2, 3] = 0` to a single-index `[]=`) reddens it.

**What is wrong.** `methodsbyclass.rex` writes **one** cell and reads **the same** cell. Any injective
index mapping — row-major, column-major, transposed, off by a constant — passes, because the write and
the read agree with each other whatever the mapping is. The equivalence arm writes and reads that same
cell again, so it adds an arity check and no mapping check. The committed control catches only the
collapse to one subscript. So "5b owes multidimensional `Array` construction and indexing" (D63) closes
over an acceptance that cannot see the indexing.

**Evidence** — the discriminator is one asymmetric second cell, oracle rc 0:

```rexx
m = .array~new(2, 3)
m[1, 2] = 'a12'
m[2, 1] = 'a21'
m~"[]="('b23', 2, 3)
say 'x' m[1, 2] m[2, 1] m[2, 3]
say 'y' m~dimension m~dimension(1) m~dimension(2)
say 'z' m~size m~items
```
```
x a12 a21 b23 | y 2 2 3 | z 6 3        rc 0
```

`m[1,2]` and `m[2,1]` are distinguishable only under the right mapping; `~dimension(1)`/`~dimension(2)`
pin the shape, which a hardcoded `~dimension` answering `2` also passes today.

**Two smaller notes on the same paragraph.** (a) "the committed probe does the first twice and never
the second" — it does `matrix[2, 3] = 0` **once** and `matrix[2, 3]` (a `[]`, not a `[]=`) once.
(b) `Concept { id: "methodsbyclass", …, oracle_lines: 4 }` (`gate_table_c.rs:508`) is checked as
`OracleShape::Exactly` (`:1556`) and a structural failure is red in **every** mode (`gate_table_c.rs:53`),
so an added output line must move that literal.

**Smallest correct replacement.** *Add the equivalence arm **and a second, asymmetric cell**: one
subscript pair and one message-form write cannot see the index mapping, since the write and the read
miss together. Keep the probe's oracle output at four lines or move `oracle_lines` with it.*

---

## F17 — Task 9's method names no enumeration and its Done-when cannot fail

**SEVERITY: medium**

Line 324-332:

> **Method.** Walk 5a's landed error and refusal sites, and for each, send the same message to an
> instance. … **Done when** the walk is complete and recorded, every divergence it found is closed or
> has a named owner, and the report says which sites were checked … **Name the enumeration the walk was
> derived from and commit it.**

**What is wrong.** "5a's landed error and refusal sites" names no artifact. The plan asks the
implementer to *name* the enumeration in the report, which means the criterion is satisfied by any
enumeration, including a small one: a walk over three sites is "complete", "recorded", and names its
enumeration. This is the vacuity shape the plan's own framing warns about one sentence earlier, and
the *fix* the plan applies (name the enumeration) does not remove it, because the plan does not say
the enumeration must be derived from the tree rather than chosen.

Candidate enumerations do exist and none is cited: `/bin/grep -rho "Loud::[a-z_]*" crates/rexx-exec/src/ | sort -u`
gives **40** distinct refusal constructors; `crates/rexx-exec/src/error.rs` holds 110
`pub(crate) fn` constructors; and `corpus/phase-5a.txt`'s 5a rows are a third.

**Smallest correct replacement.** *Derive the enumeration from the tree rather than choosing it: the
distinct `Loud::` constructors reachable from `rexx-exec/src` plus the `Raised::` sites 5a added, both
listed by a command quoted in the report. Commit the derived list, and assert in a test that the walk
covers it, so a site added later is red rather than unwalked.* (`docs/.../prose-cannot-review-a-procedure`
is the standing reason for the assertion rather than the prose.)

---

## F18 — "three parked bench programs turn live" is false for `alloc.rex` and only partly true for `heapshape.rex`

**SEVERITY: medium**

Line 352-355 and 363-368:

> `bench-programs/dispatch.rex`, `alloc.rex` and `heapshape.rex` all exit 120 on this crate today and
> **all need `~new`** …
> **`dispatchclass.rex` and `alloc4c.rex` were written as stand-ins and say so.** … **Decide, and
> record, whether each stand-in stays an axis beside the program it stood in for or is retired**

**What is wrong.** Only `dispatch.rex` needs `Object~new`. Measured on the crate at `d7bec3ae3`:

```
dispatch.rex    rc 120  rexx-exec: method "NEW" of class "Object" is not implemented (Phase 5)
alloc.rex       rc 120  rexx-exec: method "OF"  of class "Array"  is not implemented (Phase 5)
heapshape.rex   rc 120  rexx-exec: method "NEW" of class "Array"  is not implemented (Phase 5)
```

`alloc.rex`'s body is `a = .array~of(i, i+1, i+2)` and `s = .string~new("item")`. `Array~of` is named
nowhere in this plan or the spec, and `String~new` is one of the twenty-three the plan itself sends to
5c (`String rc=163 Error 93.903` on a bare `~new`; the argument form is still 5c's native constructor).
`alloc4c.rex`'s own header says the same: *"`alloc.rex` allocates a fresh `.array` and `.string` every
iteration via `.array~of`, `.string~new`, `~size` and `~length`"*. `heapshape.rex` gets past
`.array~new(1000)` with Task 8 and then needs `.directory~new` and `Directory`'s `[]=`, neither of
which is in 5b's construction list.

**Consequence for the decision the task demands.** "Whether each stand-in stays beside the program it
stood in for or is retired" is malformed for `alloc4c`, because `alloc.rex` does not go live in 5b, so
`alloc4c` cannot be retired in favour of it.

**Smallest correct replacement.** *`dispatch.rex` goes live with Task 1. `heapshape.rex` clears
`.array~new` with Task 8 and then needs `.directory~new` and `Directory`'s `[]=`, which 5b does not
owe. `alloc.rex` needs `Array~of` and `String~new`, neither of which is 5b's, and stays parked. So the
stand-in decision is `dispatchclass` versus `dispatch` only; `alloc4c` stays until the phase that
lands `Array~of` and `String~new`, and the new-baseline paragraph covers one program, not three.*

---

## F19 — two tasks' second controls assert "every gate row green", which is unevaluable until the rest land

**SEVERITY: medium**

Line 130-131 and line 169-171:

> copying the behaviour on `~inherit` too reddens the new `~inherit` witness **while leaving every gate
> row green**.

> searching the class before the object **leaves every gate row green** while reddening the new
> precedence program.

**What is wrong.** Both are the *point* of their task — the mutation no existing row can see — and both
are only observable once the rows that might have seen it are green. At the point Task 2 or Task 3
finishes, `usesem`, `obdes`, `methodsbyclass` and the two `DELEGATE` rows may all still be red (each
verified red or unbuilt at `d7bec3ae3`), so "leaving every gate row green" cannot be read off anything.
That is a third ordering constraint the independence claim at line 15 denies.

**Smallest correct replacement.** Either scope the assertion — *"leaving every gate row that agrees
before the mutation still agreeing after it"* — or state that both controls are re-run at Task 10 with
the full table green and recorded there.

---

## F20 — a mechanism-set row and a D64 item with no acceptance criterion anywhere: `~hasError`, and the `send` array form

**SEVERITY: low**

Line 278-282:

> **Build.** The four paths, plus enough of `Message` for `~result`, `~completed` and **`~hasError`**.
> `send` and `sendWith` **also accept an array whose first item is the name and whose second is a class
> to start the method search from** … measure it before building it.

Task 7's Done-when covers "each of the four", which is `~copy`, `~run`, `~send`/`~sendWith`,
`~start`/`~startWith`. `~hasError` and the array form have no criterion.

**Evidence, both measurable, both oracle rc 0** (the array form was the spec's unmeasured item):

```rexx
o = .Sub~new
say 'a' o~send('M')                            -- a sub-m
say 'b' o~send(.Array~of('M', .Base))          -- b base-m
say 'c' o~sendWith(.Array~of('M', .Base), .Array~new(0))   -- c base-m
::CLASS Base
::METHOD m
  return 'base-m'
::CLASS Sub SUBCLASS Base
::METHOD m
  return 'sub-m'
```
```
a sub-m | b base-m | c base-m        rc 0
```

and `m~hasError` is `0` after a clean `~result` (measured, rc 0). The spec's "could not check" says
`~hasError` on a *raising* method is unmeasured; the plan does not repeat that.

**Smallest correct replacement.** Add to the Done-when: *the `~start` witness asserts `~hasError` after
`~result`, and the `send` witness carries the array form's starting-class override, whose oracle
answers are above. What `~start` does with a method that raises is not measured and is not asserted.*

---

## F21 — Task 6's DEVIATION has no home and no harness, and the shape it points at is the thing the same paragraph forbids

**SEVERITY: low**

Line 246-249 and 263-264:

> **Build.** A numbered DEVIATION in the shape `docs/superpowers/plans/phase-4-exclusions.txt` already
> uses -- IMPLEMENTED / SCOPE / WHY, with pinned witnesses … **Not a prose `.txt` nobody executes** …
> **Done when** the DEVIATION exists, **the harness runs its witness**, and **the control is recorded as
> run**: making the crate match the oracle on that program reddens the DEVIATION's assertion.

**What is wrong, three small ways.** (a) The file it points at *is* a prose `.txt` nobody executes:
DEVIATION 4 (`phase-4-exclusions.txt`, the one D41 cites) carries a `MEASURED` block of sixteen
transcripts and nothing re-runs any of them. The task must therefore invent the runnable half, and the
plan names no file for it. (b) The file's own header says *"the gate asserts the SET of both sections"*
and its set is read by `builtin_status.rs:136`-`:138`, `:696`-`:707`, which is a Phase 4 builtin gate —
so whether the new row goes in that file or a Phase 5 successor is a decision the plan leaves open.
(c) "making the crate match the oracle on that program" is a mutation nobody can apply without undoing
D59; the applicable mutation is "deliver a class's `UNINIT` from the driven collection instead of the
termination sweep", which should be spelled.

The closest existing instrument, unnamed by the plan, is `ir_dual.rs`'s `KNOWN_DIVERGENCES` /
`the_known_engine_divergences_still_diverge_exactly_as_recorded` (`:1074`, `:1104`), which asserts each
side's exact bytes *and* that they differ — and whose doc states the property this task needs:
*"Red if either engine's answer moves, in either direction -- including a fix, which is what should
delete the row rather than update it."*

**Smallest correct replacement.** Name the file and the harness: *the DEVIATION row goes in
`phase-4-exclusions.txt`'s DEVIATIONS section (or a Phase 5 successor named here), and its runnable
half is a test in the shape of `ir_dual.rs`'s `KNOWN_DIVERGENCES` — the program, the oracle's expected
stdout, the crate's expected stdout, and an assertion that the two differ. The control is delivering
the class's `UNINIT` from the driven collection instead of the termination sweep.*

---

## F22 — the inherited 5a constraint sends a new corpus program to `phase-5a.txt`

**SEVERITY: low**

Line 20-21:

> `docs/superpowers/records/2026-08-17-phase-5a/` carries the constraints this project runs under and
> they **bind unchanged**.

That file (`global-constraints.md:23`-`:24`) says: *"A task that adds a corpus program adds it to
`rust/corpus/phase-5a.txt` in its own commit."* Read as binding unchanged, that contradicts D64, which
puts 5b's witnesses in `corpus/phase-5b.txt`.

**Smallest correct replacement.** *… bind unchanged, with one amendment: a 5b task that adds a corpus
program adds it to `rust/corpus/phase-5b.txt`, not `phase-5a.txt`.*

---

## F23 — the plan omits the convention that a newly-agreeing gate-table probe joins the phase subset file

**SEVERITY: low**

`gate_table_d.rs:30`-`:34` states it: *"These probes are not corpus programs. They live under
`corpus/gate-tables/`, not in a phase subset file. Most of them diverge, and `corpus/phase-5a.txt`
means 'agrees with the oracle'; **a probe moves there in the task that makes its row agree**."*
`phase-5a.txt` carries five such entries with the reason written beside them (`:672`, `:733`, and
three more). Tasks 1, 2, 3, 4, 5 and 8 each make a gate probe agree and none of them says to list it.
This is also what puts Task 1's rows under the stress harness (F4).

**Smallest correct replacement.** One line in the global constraints: *a task that makes a gate-table
row agree adds that row's probe path to `corpus/phase-5b.txt` in the same commit, which is where the
differential and the collect-on-every-allocation run then reach it.*

---

## F24 — "two documentation commits and the spec"

**SEVERITY: low**

Line 11-12:

> **Base.** `01e024157`. Corpus 264 of 264; all five gates green at `c7345f7a9`, whose only descendants
> are **two documentation commits and the spec**.

`git log --oneline c7345f7a9..HEAD` gives four commits: `538f35310` (one documentation commit, 19 lines
of `rust/bench-baselines/README.md`), `be1cf430b` and `01e024157` (the spec, two commits), and
`d7bec3ae3` (the plan itself). So it is *one* documentation commit and the spec — unless the plan's own
commit is being counted, which it could not be when the sentence was written.

**The load-bearing half holds and is worth keeping**: `git diff --stat c7345f7a9..HEAD` touches only
those three markdown files and nothing under `rust/`, so the tree the gates were green on is the tree
this plan starts from.

**Smallest correct replacement.** *… whose only descendants are one documentation commit and the spec's
two, and `git diff --stat c7345f7a9..HEAD` touches no file under `rust/`.*

---

# Pass 5 — task-pair interactions

One row per pair that shares a file, a data structure or an interface.

| pair | shared | one produces / other consumes | found |
|---|---|---|---|
| 1 & 2 | the behaviour an instance dispatches against | 1 step 2 sets the instance's behaviour at construction; 2 decides whether that is a pointer to the class's object or a copy | **Real and one-directional.** If 1 lands a live class walk, 2's `~define` arm is silently wrong. 1's Build text says "set the behaviour from the class's instance behaviour", which is the right shape; no defect, but 2 cannot be started without knowing what 1 chose |
| 1 & 3 | `Body::Instance(ScopePools)` | 1 creates the pools; 3 adds the object's own `FLOAT`/`OBJECT` pools and puts the object's dictionary in front of the class's | **Real.** Composes cleanly; 3's precedence step must sit in front of whatever 2 produced, so 2 & 3 also touch the same lookup |
| 1 & 4 | `.K~new` in both replaced table D probes | 1 produces the instance; 4's two rows consume it | **Real for the `DELEGATE` half only.** 4's `FORWARD` half is measurable with a class method and needs nothing from 1 (F9, F10) |
| 1 & 5 | `completeNewObject` step 3, `requiresUninit` | 1 registers the flag on the new instance; 5's deliveries 1 and 2 read it | **Real, and cross-referenced correctly** ("Task 5 owns what happens with them afterwards"). Step 3 has no acceptance in Task 1; 5's corpus witnesses are the only thing that would catch its omission |
| 1 & 7 | the instance receiver | 1 produces it; 7 sends `~copy`/`~run`/`~send`/`~start` to it | Real, no defect |
| 1 & 8 | — | — | **No dependency at all.** `methodsbyclass` refuses at `Array~new`, not `Object~new`; `Body::Array` is not `Body::Instance`. Falsifies line 14 (F10) |
| 1 & 9 | every 5a refusal site | 1 gives 9 its second receiver kind | Real; 9 cannot start before 1 |
| 1 & 10 | `bench-programs/dispatch.rex` | 1 unblocks it; 10 must baseline it | Real, and the only one of the three that holds (F18) |
| 2 & 3 | the send-time method lookup | 2 owns where the class behaviour lives; 3 puts the object's scope first | **Real.** Both edit the resolution path; independent in content, colliding in file |
| 2 & 3 & 5 & 8 | "leaving every gate row green" in 2's and 3's second controls | those controls read the whole table | **Real ordering constraint** the independence claim denies (F19) |
| 2, 3, 5, 7 (and 1, 4, 8 by convention) | `corpus/phase-5b.txt` and five `SUBSET_FILES` literals | each appends; nobody creates or wires | **Conflict.** First lander inherits four red guards and one silent one (F3, F23) |
| 3 & 7 | `checkRestrictedMethod` | 3 builds the predicate for `setMethod`/`unsetMethod`; 7 reaches it again for `run` | **Real and named** ("Task 3's check, reached again here"). If 7 lands first it must build the check; the plan does not say who owns it then |
| 5 & 6 | the class-object `UNINIT` delivery point | 5 makes the crate print the line; 6 commits an assertion about whether it prints | **Broken by ordering.** 6 before 5 pins a witness 5 falsifies (F1) |
| 5 & 8 | `gate_table_c.rs`'s `CONCEPTS` array | 5 edits `obdes`'s `control` text; 8 may need to move `methodsbyclass`'s `oracle_lines` | Textual collision only, in one `const` |
| 5 & 9 | instance `UNINIT` on a receiver 5a never had | 5 delivers; 9 walks 5a's limits with the same receiver | Real, benign |
| 6 & 10 | the DEVIATION's runnable witness | 6 builds it; 10's criterion 3 requires it to run | Real, and 10 is where 6's ordering error would surface |
| 8 & 10 | `bench-programs/heapshape.rex` | 8 clears `.array~new`; 10 must then baseline it | **Partly broken**: `heapshape` still needs `.directory~new` and `Directory`'s `[]=` (F18) |
| all & 10 | `CLOSED_PHASES`, D65 | every task's row; 10 flips | Real; 10 is correctly last |

**Does anything in a later task invalidate an earlier task's committed witness?** One case, and it is
F1: Task 5 invalidates Task 6's. I found no other — in particular Task 2's `~inherit` witness, Task 3's
precedence and `FLOAT`-pool programs, Task 5's two instance deliveries and Task 7's four witnesses are
all insensitive to the other tasks' mechanisms, and I checked each program against every other task's
Build list for a shared observable.

---

# Asserted about the world without being checked — every one run

Universal quantifiers, "the only", "every", "no X does Y", and counts. One row per claim, with the
line in the plan.

| line | claim | verdict |
|---|---|---|
| 11 | all five gates green at `c7345f7a9`, "whose only descendants are two documentation commits and the spec" | **FAILS as written** — one documentation commit (`538f35310`, `bench-baselines/README.md`) and the spec's two (`be1cf430b`, `01e024157`); the fourth descendant is the plan's own commit. **The load-bearing half HOLDS**: `git diff --stat c7345f7a9..HEAD` touches three markdown files and nothing under `rust/` (F24) |
| 11 | "Corpus 264 of 264" | **HOLDS** structurally: the deduplicated union of the four phase subset files is exactly 264 entries |
| 14 | "Task 1 unblocks **every** other task" | **FAILS** — Task 8 refuses at `Array~new` not `Object~new`, Task 6's witness runs to completion on the crate today, and Task 4's `FORWARD` half is measurable on a class method (F10) |
| 15 | "Tasks 2 to 8 are independent of each other once Task 1 is in" | **FAILS** three ways — Task 5 changes the crate's answer for Task 6's committed witness (F1); whichever of 2/3/5/7 lands first owns `corpus/phase-5b.txt`'s wiring (F3); Tasks 2 and 3's second controls assert "every gate row green" (F19) |
| 48-50 | "**Three probes** in this neighbourhood **were green** over mechanisms that were not implemented" | **FAILS** — the list names four probes, and only the two `DELEGATE` rows were ever green; `usesem.rex` is rc 120 against rc 0 today and the `k~at` draft was never run as a row (F11) |
| 58-60 | every 5b row of gate table C diverges; every one but `obdes` is loud; `obdes` is `diverge-stdout` with empty stderr both sides; both 5b rows of table D agree over an unimplemented mechanism | **HOLDS, all four.** Re-ran the six concept probes and both directive probes on the oracle and on both engines |
| 68-69 | `creo` and `abscla` are "both rc 120 today, refused at `method "NEW" of class "Object" is not implemented (Phase 5)`" | **HOLDS**, that exact string on both engines for both rows |
| 76-77 | `("Object", "INIT", Arity::Fixed(0), native_no_op)` is already in the native table | **HOLDS verbatim**, `dispatch.rs:375` |
| 77-78 | "`~objectName`/`~objectName=`/`~string`/`~class`/`~isA` **all** answer for a class receiver" | **HOLDS** for those five. The list is the trap: `~defaultName`, named three lines later as this task's work, answers for **no** receiver and appears nowhere in `crates/rexx-exec/src/` (F5) |
| 83 | "The registry already carries the `UNINIT` flags (`has_uninit`, `parent_has_uninit`) **that step three reads**" | **HOLDS** for step three, which is about the new instance. It does not extend to Task 5's delivery 3, whose predicate does not exist (F7) |
| 88-89 | the instance-naming measurements | **HOLDS exactly**: `an Object` / `a K`; after `~objectName =`, `~string` and `~objectName` move and `~defaultName` does not |
| 93 | "**Twenty-three of the fifty-nine** `*__instance.rex` classes refuse a bare `~new`" | **HOLDS exactly.** All 59 run: 23 refuse, 36 answer, and the five error-number groups are the plan's (F13 is about `Message` being in the 23, not about the count) |
| 99 | "Nothing can reach it today because **no** `::METHOD` body runs with a non-class receiver" | **not falsified, not proved.** Seven routes tried, all refused earlier; the most interesting, `.String~define('MINE',…)` then a send to a literal, is `98.985` at rc 158 identically on both sides. A universal over the reachable surface I could not enumerate |
| 118-121 | six `ClassClass.cpp` line citations for D58's evidence | **HOLD, all six**, to the line: `:860`-`:862`, `:531`-`:533`, `:962`-`:964`, `:1361`, `:1413`, `:1036` |
| 128 | the `~inherit` and `~uninherit` witnesses — "**the spec prints both**" | **FAILS** — the spec prints the `~inherit` program and states the `~uninherit` arm in prose only, four mentions, no program (F14) |
| 129 | "**no committed `.rex`** combines `~inherit` with `~new`" | **HOLDS.** Fourteen files match `~inherit|~uninherit` under `rust/corpus/`; none also matches `~new` |
| 141-142 | "The probe needs `.StringTable~new` and `Class~enhanced` **as well as** `setMethod`" | **INCOMPLETE** — `usesem.rex` also sends `StringTable`'s `[]=`, which exists for no receiver kind (F12) |
| 150-151 | one `FLOAT` method cannot distinguish per-object from per-method; "the spec measures the difference with two" | **HOLDS** — D67 states the two-method measurement, and the single-method transcript genuinely cannot separate them |
| 155-159 | D66's two refusals, their error numbers, exit statuses, and the frame discriminator | **HOLD exactly**, both transcripts, including "the 97.2 transcript has no method frame at all" |
| 159 | "`send` and `sendWith` are **not** in the trio and answer from a program context" | **HOLDS**, rc 0 for both |
| 162-163 | `usesem.rex` "defines `EXTRA` only in the object's scope" | **HOLDS** — `::class k` declares `addExtra` and nothing named `EXTRA` |
| 178-180 | both table D probes are "`say 'main'` plus directives", so both sides print `main` whether or not `DELEGATE` exists | **HOLDS verbatim** — the two files are four lines each, exactly as printed |
| 188-190 | `expected_oracle_lines` bounds a non-refusing row's oracle stdout at exactly one line | **HOLDS**, `gate_table_d.rs:317`-`:319` |
| 194-195 | the setter control prints `main AT` instead of `main via-set` | **HOLDS**, oracle rc 0 both ways |
| 204-205 | "`obdes.rex` is the phase's **only** silent row" | **HOLDS** — the other five are rc 120 against rc 0 |
| 208-211 | `Heap::collect`/`pending_uninit`/`set_uninit`/`clear_uninit`, and "`Heap::set_uninit` is the **only** writer of `true`" | **HOLD.** `heap.rs:322` is the only `= true`; `:346` writes `false` and `:411`/`:434` construct `false` |
| 212 | "`ClassGraph::check_uninit` and `parent_has_uninit` are 5a's half of the propagation" | **HOLDS but is the smaller half** — both are about a class's *instances*; the class-object side has no flag and `class_graph.rs:371`-`:381` says so (F7) |
| 229-231 | D60's two order witnesses (a class declared second firing after one declared third; runtime-built before directive-installed) | **not re-run**; I ran a third shape instead, which adds a constraint: a subclass inheriting a class-side `UNINIT` fires **before** the parent that declares it, ten of ten |
| 247-248 | "the parent spec's D41 already cites [the DEVIATION shape] for `~identityHash`" | **HOLDS** — `2026-08-17-phase-5-object-model.md:1143`, "licensed as deviation 4" |
| 258-260 | the OOM measurement (100,000 `~subclass`, oracle rc 0 `done 100001`, crate rc 137) | **not re-run** — deliberately, since it OOM-kills; the spec's review reproduced it |
| 270-274 | Task 7's whole measured paragraph | **HOLDS exactly**, all seven values |
| 286-287 | "`~completed` sampled before `~result` … forty runs split thirty to ten" | **not re-run.** The spec's review measured 26/14 on its own forty; both support the conclusion the plan draws, which is that it is not reproducible |
| 299 | `methodsbyclass`'s four oracle lines | **HOLDS** byte for byte |
| 302-303 | "`Body::Array` is a flat `Vec<Option<ObjRef>>` today" | **HOLDS**, `rexx-core/src/body.rs` |
| 306-307 | "the committed probe does the first twice and never the second" | **FAILS as written** — the probe does `matrix[2, 3] = 0` once and `matrix[2, 3]` (a `[]`, not a `[]=`) once (F16) |
| 340 | `CLOSED_PHASES` in `rust/crates/rexx-exec/tests/gate_tables/mod.rs` | **HOLDS**, `:344`, `&["5a"]` |
| 347 | "the eight axes" against `bench-baselines/pinned/rexx-run-15a1ffa98` | **HOLDS** — eight distinct axes in `phase-5a-arms.tsv`, and the pinned binary exists with the provenance `PINNED.md` records |
| 353-354 | "`dispatch.rex`, `alloc.rex` and `heapshape.rex` all exit 120 on this crate today and **all need `~new`**" | **first half HOLDS, second FAILS** — `alloc.rex` refuses at `Array~of` and then needs `String~new`; `heapshape.rex` needs `Directory~new` and `Directory`'s `[]=` after `Array~new` (F18) |
| 354-355 | `dispatch` and `alloc` are benchmarked; `heapshape` is in `NOT_BENCHMARKED` because it reports its own timing | **HOLDS**, `rexx-bench/src/lib.rs:41`-`:68` |
| 356-357 | that file "names Phase 5 as when `heapshape` starts running" | **HOLDS**, `:250`-`:251` |
| 364-367 | the two stand-in header quotations | **HOLD verbatim**, the second across a line wrap that defeats a literal `grep` |

---

# Verified and found CORRECT

Re-run or re-read from my own probe directory at `d7bec3ae3`, byte for byte unless noted.

**Every C++ citation is exact.** `ClassClass.cpp:860`-`:862` is the `defineMethod` copy with
`// make a copy of the instance behaviour so any previous objects / aren't enhanced`; `:531`-`:533` the
same three lines in `defineMethods` (the plan's `:531` is tighter and more accurate than the spec's
`:530`); `:962`-`:964` the `deleteMethod` copy under *"we work on a copy of the instance behaviour so
that this changed does not suddenly show up in existing instances of this class."*; `:1361` is
`updateSubClasses();` inside `inherit` and `:1413` inside `uninherit`; `:1036` is
`void  RexxClass::updateSubClasses()`; `:1882` is `void RexxClass::completeNewObject(…)` with
`checkAbstract();` as its first statement. `ObjectClass.cpp:697` is
`void RexxObject::checkRestrictedMethod(const char *methodName)`, called from `:1876`, `:1896`, `:2241`.

**Every Rust citation is exact.** `dispatch.rs:1123` is `Body::Instance(_) => Err("an instance of a
user class")`. `dispatch.rs:375` is `("Object", "INIT", Arity::Fixed(0), native_no_op),`, quoted
character for character. `body.rs` has `Array(Vec<Option<ObjRef>>)` and `Instance(ScopePools)`.
`run.rs:3077` is the `SELF`-slot sentence of `pool_owner`'s doc, and `:3081` the function.
`lib.rs:6446` is the `debug_assert!(stats.pending_uninit.is_empty(), …)` with the comment at
`:6439`-`:6445` naming itself as the site that owes delivery. `gate_tables/mod.rs:344` is
`CLOSED_PHASES: &[&str] = &["5a"]`. `gate_table_d.rs:317`-`:319` is
`fn expected_oracle_lines(row: &Row) -> usize { usize::from(!oracle_refuses(row)) }`.
`Heap::set_uninit` is the only writer of `true` (`heap.rs:322`; `:346` writes `false`, `:411` and
`:434` construct `false`) — the plan inherited the spec review's F19 correction intact.
`compile_method_source` is `dispatch.rs:3541` and reaches `~define` at `:3660`; `method_source_lines`
(`:3473`-`:3493`) already accepts a string **or** an Array of strings, so the plan's "is what the
string and string-array forms need here" holds exactly.

**Every 5b gate-row measurement holds**, all six re-run on the oracle and on both engines:

```
creo            oracle rc 0   type a savings account / balance 1000.00 / rate 6.25
                crate  rc 120 rexx-exec: method "NEW" of class "Object" is not implemented (Phase 5)
abscla          oracle rc 158 stdout declared AB Class; stderr the 98.989 transcript with the
                              *-* Compiled method "NEW" with scope "Object". frame
                crate  rc 120 stdout declared AB Class, same loud message
objcla          oracle rc 0   before 0 / after 0 / fresh 1                  crate rc 120
usesem          oracle rc 0   setmethod one-off / not-shared 0 / enhanced enhanced /
                              still-not-shared 0                            crate rc 120
obdes           oracle rc 0   main / uninit ran                             crate rc 0, main only
methodsbyclass  oracle rc 0   element 0 / dimensions 2 / methods 1 1 /
                              environment Array Directory StringTable
                crate  rc 120 method "NEW" of class "Array"
```

so "every 5b row of gate table C diverges and every one but `obdes` is loud" **holds**, and `obdes` is
rc 0 with empty stderr on both sides, differing on stdout alone — the phase's only silent row.

**Both table D probes are verbatim as the plan prints them** (`say 'main'` plus directives, no send)
and both `agree`. **Both replacement probes and Task 4's control run exactly as claimed**, oracle rc 0,
empty stderr: the getter probe prints `main 6`; the setter probe prints `main via-set`; and with
`::attribute at delegate d` replaced by a plain `::attribute at` it prints `main AT`. That control is
real and I recommend keeping the sentence that calls it not-optional.

**Both restricted-private refusals are exact, including the discriminator.** `o~setMethod(…)` from a
program context: rc 159, `Error 97.2: Object "an Object" cannot accept private message "SETMETHOD"
from this context.`, **no method frame**. From a class method of a class the object is not an instance
of: rc 158, `Error 98.991: Method SETMETHOD may only be invoked from a method of the same object or one
of its classes.`, **with** `*-* Compiled method "SETMETHOD" with scope "Object".`. `o~run('return 1')`
from a program context is `97.2 … "a K" … "RUN"` at rc 159.

**Task 1's naming measurements are exact.** `.Object~new~string` is `an Object`; an instance of `K` is
`a K` for `~string`, `~objectName` and `~defaultName`; after `~objectName = 'zed'`, `~string` and
`~objectName` answer `zed` and `~defaultName` still answers `a K`.

**Task 7's measured paragraph is exact.** `o~send('M', 3)` → 4; `o~sendWith('M', .Array~of(4))` → 5;
`m~class~id` → `Message`; `m~result` → 6; `m~completed` after `~result` → 1; `~hasError` → 0;
`(o == c)` → 0. All rc 0.

**The `23 of 59` count is exactly right.** All fifty-nine `*__instance.rex` classes run with a bare
`~new` on the oracle: 23 refuse — 93.901 ×11 (Alarm, CaselessColumnComparator, CircularQueue, Class,
ColumnComparator, File, InvertingComparator, Message, Stream, Ticker, TimeSpan), 88.901 ×3 (Method,
Package, Routine), 93.903 ×3 (String, Supplier, WeakReference), 93.967 ×4 (Pointer, RexxContext,
StackFrame, VariableReference), 97.1 ×2 (RexxInfo, StreamSupplier) — and 36 answer at rc 0. The file
count is 59.

**"no committed `.rex` combines `~inherit` with `~new`" holds.** Fourteen files under `rust/corpus/`
match `~inherit|~uninherit`; none of them also matches `~new`.

**"Corpus 264 of 264" holds structurally**: the deduplicated union of `phase-4a/4b/4c/5a.txt` is
exactly 264 entries.

**"the eight axes" holds**: `bench-baselines/phase-5a-arms.tsv` carries eight distinct axes —
`alloc4c`, `arith`, `compound`, `dispatchclass`, `emptyloop`, `rexxcps`, `strings`, `varlookup` — and
`bench-baselines/pinned/rexx-run-15a1ffa98` exists with the provenance `PINNED.md` records. The
arithmetic behind "eleven" (8 + 3) is right, though the eight include `rexxcps` (not a bench-program)
and exclude `startup` (which is one), so "eleven axes" is the sitting's eleven and not the criterion
harness's `PROGRAMS + NOT_BENCHMARKED`, which is also eleven and a different set.

**The two stand-in quotations are verbatim**, including the second, whose line-wrap makes it invisible
to a naive `grep -F`: `dispatchclass.rex` says it "does not replace dispatch.rex or relieve whichever
task unblocks it", and `alloc4c.rex` says "It is not a substitute for / a collection-forcing
measurement once message sends land" across lines 29-30. `crates/rexx-bench/src/lib.rs` has `dispatch`
and `alloc` in `PROGRAMS` (`:41`), `heapshape` alone in `NOT_BENCHMARKED` (`:68`) because it reports
its own timing, and the test's doc comment names Phase 5 as when it starts running (`:250`-`:251`).

**`obdes`'s committed control text is exactly what Task 5 says it is** —
`gate_table_c.rs:457`-`:459`, "stop running `UNINIT` before the object's storage is reclaimed, which
is **silent**…" — as are `objcla`'s, `usesem`'s, `creo`'s, `abscla`'s ("drop `checkAbstract` from the
`~new` path, so an abstract class constructs -- 5b") and `methodsbyclass`'s.

**`docs/superpowers/plans/phase-4-exclusions.txt` has the IMPLEMENTED / SCOPE / WHY DEVIATION shape**
the plan describes, and the parent spec's D41 does cite "deviation 4" for `~identityHash`
(`2026-08-17-phase-5-object-model.md:1143`). `rust/CLAUDE.md:95` carries the 2026-08-27 comment rule
in the words the plan paraphrases. `docs/superpowers/records/2026-08-17-phase-5a/global-constraints.md:11`-`:19`
carries the five gate commands.

**Three measurements the plan needs and does not have, offered because they settle open items:**

1. **A one-instance termination delivery *is* reproducible** — the spec's "What I could not check" #1.
   Twenty runs of twenty on the oracle: `start | exiting | uninit one`, rc 0, every time. So Task 5's
   required corpus program is not flaky, and D61 is satisfied by it.
2. **The one-instance form of D69's class-retained program is reproducible too**, ten of ten:
   `start | after-gc | instance uninit 1`, rc 0. **This is the program Task 5 should require**, because
   a plain one-instance termination witness does not discriminate: the hazard D69 names is an instance
   a class-scope `EXPOSE` roots for ever, which a collection can never reach, and only the retained
   shape can see a termination sweep that walks the activation's variables instead of the live set.
3. **The `send`/`sendWith` array form works and is now measured** (F20), closing the item Task 7 says
   was not measured while writing the spec.

---

# What I could not check, and why

* **The gate tables' printed phase summaries.** `5b: 6 rows, 6 not yet agree` and `5b: 2 rows, 0 not
  yet agree` come from `REXX_CORPUS_GATE=1 cargo test`, and I was told not to run cargo. I re-derived
  all eight verdicts by running each probe on all three interpreters by hand, and the counts follow,
  but I did not see the harness print them.
* **Every negative control, as a mutation.** Each is a source edit plus a rebuild. I checked each
  control's *text* against its probe and can say whether the probe could see it — F2 and F16 are the
  two that could not, and F19 is the one that cannot be evaluated when its task ends — but I executed
  none of them.
* **Line 99's negative**, "Nothing can reach it today because no `::METHOD` body runs with a non-class
  receiver". I tried `~new`, `~run`, `setMethod`, `~enhanced` via `.StringTable~new`, `Array~of`,
  `~start`, and `.String~define('MINE', …)` followed by a send to a literal — the last is the only
  interesting one and it is refused identically on both sides (`Error 98.985: User additions are not
  allowed to the REXX language classes.`, rc 158, oracle and both engines). Every route is refused
  earlier, but the claim is a universal over the whole reachable surface and I could not enumerate it.
  The spec's review reached the same place.
* **Whether the crate's `~defaultName` absence is 5b's to fix or 5c's.** I established it does not
  exist for any receiver (F5); which phase owes it is a question for the spec's owner, not a
  measurement. The plan assigns it to Task 1 and the spec does not mention it.
* **Task 5's order question.** I added one constraint any rule must satisfy — a subclass inheriting a
  class-side `UNINIT` fires *before* the parent that declares it, ten runs of ten — and deliberately
  did not go looking for the mechanism in the C++, which is the task's own work.
* **`corpus/phase-5b.txt`'s eventual contents against `read_subset`.** The file does not exist, so the
  five wiring sites and the missing `EXPECTED_SUBSET_5B` are read from the code rather than from a red
  run.
