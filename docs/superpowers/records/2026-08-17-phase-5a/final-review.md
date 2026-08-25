# Final whole-branch review — plan `2026-08-17-phase-5a`, HEAD `433ac60fc`

Scope: what falls **between** task boundaries. No task was re-reviewed and no task finding was
re-derived. Read-only on the tree apart from this file; nothing was rebuilt.

**Verdict: CHANGES REQUIRED.** One high, four medium, four low.

The high finding is a **silent wrong answer reachable by a three-line program that also corrupts
interpreter state**, and it is exactly the shape a single-task review cannot see: Task 21 built the
mechanism, Task 23 created a new population the mechanism must cover, and neither review looked at
the other's neighbourhood. Three sentences in the tree assert that the gap does not exist.

Everything else is prose falsified by a later task, plus one instrument that now suppresses the
Phase 5 exit evidence it was written to produce.

---

## H1 — the `REXX_DEFINED` lock is never set on the 34 classes the library bootstrap declares

**Severity: high.** rc 0, wrong stdout, and the mutation lands.

`CoreClasses.orx` and `StreamClasses.orx` declare **34 public `::CLASS` directives** between them
(`/bin/grep -acinE "^\s*::class\b" … | grep -icE "\bpublic\b"` answers 34; the five non-public ones
are `SupplierMixin`, `ManyItemMixin`, `SetMixin`, `BagMixin`, `LocalServer`). Task 23's prologue puts
every one of them into `.environment`. **None of them is given `REXX_DEFINED`**, so all five class
mutators are open on them where the oracle raises `98.985 User additions are not allowed to the REXX
language classes.`

`rexx_classes::native_classes`'s `build()` sets the flag on every class it builds
(`native_classes.rs:391`-`:398`) — that is the `Setup.cpp` half. The `.orx` half has no counterpart:
`Interp::bootstrap_library` (`lib.rs:4323`-`:4352`) closes the bootstrap with
`rexx_classes::remove_setup_methods` and nothing else, so `Setup.cpp:1809`'s sibling is modelled and
`RexxClass::liveGeneral`'s `setRexxDefined()` under `PREPARINGIMAGE` is not. The plan's own Task 21
row names both halves of `liveGeneral` (`plan:1874`-`:1875`: "sets `setRexxDefined()` under
`PREPARINGIMAGE` **and repoints `package` to `TheRexxPackage`**"). The package half **was** carried
across the boundary — measured, `.Comparable~package~name` and `.Alarm~package~name` are both `REXX`
on all three interpreters. The flag half was not.

### Measured

Three lines, in a fresh directory, both engines:

```rexx
say .Alarm~isA(.Comparable)
.Alarm~inherit(.Comparable)
say .Alarm~isA(.Comparable)
```

* oracle: rc **158**, stdout `0`, stderr `Compiled method "INHERIT" with scope "Class".` …
  `Error 98.985:  User additions are not allowed to the REXX language classes.`
* crate, `ir` and `tree-walker`: rc **0**, stdout `0` / `0`, stderr empty.

The mutation is real, not a refusal that silently no-ops:

```rexx
s = .Alarm~superClasses
say s~items
.Alarm~inherit(.Comparable)
s = .Alarm~superClasses
say s~items
do i = 1 to s~items
  say s[i]~id
end
```

* oracle: rc 158, stdout `1`, then 98.985.
* crate, both engines: rc 0, stdout `1` / `2` / `Object` / `Comparable`.

### Sized

`.X~inherit()` (argumentless — the lock is checked before the arguments are, which is what makes this
a clean discriminator), run over all 62 classes the oracle's `.Array~package~publicClasses` names:

| | count |
|---|---|
| oracle `98.985`, crate `98.985` | **28** |
| oracle `98.985`, crate `88.901` | **34** |

The 34 are exactly the two files' public `::CLASS` declarations. A second mutator agrees:
`.X~delete('ZZZNOSUCH')` is oracle rc 158 against crate rc 0 on the same set (`.RexxQueue` answers
differently only because it declares its own `delete` class method bound to `rexx_delete_queue`, a
Phase 7 entry point — not a lock question).

### Why nothing caught it

`dispatch.rs:3332` names the instrument: *"The only instrument that can catch a regression here is
the corpus."* All five corpus rows use `.Array` —
`lang/class_rexx_defined_{define,define_methods,delete,inherit,uninherit}.rex` — and `.Array` is a
`Setup.cpp` class, which **is** flagged. The instrument covers the native half of the class set and
is blind to the half Task 23 added. `checks-blind-to-their-own-subject`, one task removed from where
the check was written.

### Sentences this falsifies

* `rust/crates/rexx-classes/src/class_graph.rs:228`-`:229` — "which is why **every class a program
  can reach through `.environment` carries it** and a `::CLASS` a program declares does not."
  False for 34 of 62.
* `rust/crates/rexx-exec/src/dispatch.rs:3333` — "**Every class in `.environment` carries the flag**
  and every `::CLASS` a program declares does not, so a build that dropped this check would let
  `.Array~define(...)` succeed at rc 0 where the oracle raises." False, and the failure it describes
  is the one shipped — for a different class.
* `rust/crates/rexx-exec/src/lib.rs:3131`-`:3132` — "the `REXX_DEFINED` lock **every class carries**
  does not refuse the mutators". Not every class carries it. `lib.rs:3138`'s "**No user program can
  see either**" is true of the *flag being open* and false of its *effect*: a program cannot run
  inside the bootstrap, but it can mutate every class the bootstrap declared.
* `class_graph.rs:234`-`:238` — "Nothing inside this crate reads it: the bootstrap replays
  `Setup.cpp` through the same graph operations after the flag is set … and the check lives where
  the oracle puts it." The library bootstrap now runs graph operations too, and it is not covered by
  "after the flag is set" — it is covered by `dispatch.rs:3346`'s `!interp.library_bootstrap`
  bypass, which this paragraph does not mention. (`ClassRegistry::is_rexx_defined`,
  `registry.rs:562`, is also a read inside `rexx-classes`, though only as an accessor.)

### Fix shape

Set the flag where the class is created rather than by a sweep. `rexx_defined_lock` already bypasses
itself while `library_bootstrap` is true (`dispatch.rs:3346`), so flagging a library-installed class
at `Interp::install_class_at` time is safe and needs no second pass — and it avoids the ordering
hazard `native_classes.rs:391`-`:397` records against a sweep ("the sweep's only source of classes is
a `HashMap`, and a loop whose order is the map's is a loop that could come to matter";
`Interp::package_classes` is a `HashMap` too).

Whatever the fix, **the corpus row set has to grow a library class**: one row on any of the 34 is
what makes the instrument `dispatch.rs:3332` names cover its own subject.

---

## M1 — the `~publicClasses` refusal's stated ground is measurably gone, in two places

`rust/crates/rexx-exec/src/lib.rs:861`-`:872` (`Loud::rexx_package_classes`):

> Measured, oracle rc 0: `.Array~package~publicClasses["ORDEREDCOLLECTION"]` is `The
> OrderedCollection class`, **a name no class this crate registers carries**. Answering the partial
> table would make that read `The NIL object` …

Measured at HEAD: `say .OrderedCollection~id` is `OrderedCollection`, rc 0, byte-identical to the
oracle on **both** engines. Stronger: I took the oracle's own 62-name `~publicClasses` list and ran
`say .NAME~id` for each on the crate — **62 of 62 answer, none refuses**. Spot-checked against the
oracle for the twelve least likely (`DateTime File Properties ArgUtil Ticker Validate Singleton`,
`CircularQueue Monitor RexxQueue StreamSupplier Orderable`): byte-identical.

The same sentence, verbatim, is at `rust/crates/rexx-exec/src/run/tests.rs:7677`-`:7679`, inside
`the_refusals_this_task_leaves_where_the_oracle_answers_still_fire`'s doc.

The refusal itself is not obviously wrong — `~publicClasses` returns a fresh table and this crate has
no ordered source for it — but the reason given for it is a false exhibit, and it is the reason a
later task will read when deciding whether the refusal can be lifted.

**Note on method**: neither instance is findable by plain grep. The phrase is hard-wrapped across
three `///` lines; `/bin/grep -rn "a name no class this crate registers carries"` answers **0** and
the collapsed-comment index answers **2**.

---

## M2 — the benchmark suite still prints that the crate has no bootstrap, and that is the Phase 5 D2 evidence

`rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs:874`-`:878` — an **emitted report paragraph**,
not a comment:

> **Not comparable, and not a pass.** This crate has no `CoreClasses.orx` bootstrap yet (Phase 5),
> so it starts fast by not doing the work the oracle does at startup.

and the `Role::Offset` doc at `:134`-`:137` says the same thing.

Task 23 landed the bootstrap. The plan's Task 24 report item says so in terms: "that section reports
the pre-5a crate as *not comparable* precisely because it has no bootstrap, so **Task 23 is what
turns this into a real comparison rather than a formality**" (plan `:2288`-`:2290`).

Measured with the project's own timer, interleaved, three rounds of `--warmup 10 --runs 50` on
`say 'x'`:

| round | oracle min / median | crate (`ir`) min / median |
|---|---|---|
| 1 | 5.988 / 6.932 ms | 26.699 / 37.634 ms |
| 2 | 4.701 / 6.087 ms | 26.297 / 30.405 ms |
| 3 | 4.585 / 5.721 ms | 25.501 / 30.295 ms |

Caveats stated rather than dropped: no CPU pinning; `build/` is a `RelWithDebInfo -O2 -g` oracle, not
`-O3`; `rexx-time`'s own binary is from 03:43 and `rexx-run`'s from 15:34, which does not affect the
numbers because `rexx-time` only spawns and times external processes. The oracle side reproduces the
roadmap's pinned 5.1 ms median (`2026-07-27-rust-rewrite.md:159`), which is the control that says the
instrument is reading the right thing.

So the crate is now roughly **21 ms slower** at cold start, not "fast by not doing the work". That is
under D2's own 50 ms image threshold (`roadmap:157`), so D2 still resolves the same way — but it now
resolves on a measurement instead of a formality, and the suite's report tells whoever runs the phase
gate not to make the comparison.

---

## M3 — the `DO OVER` `StringTable` order divergence is in no boundary list

Six lines, rc 0 on every side, different stdout:

```rexx
do i over .methods
  say i
end
::method zebra
::method alpha
::method mango
::method delta
::method kiwi
```

* oracle: `MANGO ALPHA KIWI ZEBRA DELTA`
* crate, both engines: `ALPHA DELTA KIWI MANGO ZEBRA`

**This is documented honestly and prominently at the site** — `run.rs:7799`-`:7834` opens "**The
order is this crate's and not the oracle's, and that is a divergence a program can see**", derives
what closing it would take, and records the corpus prohibition. `corpus/README.md` carries the rule.
Nothing is concealed and I am not re-raising a known item as if it were new.

What is missing is ownership. The plan's "**Name what 5a did not cover, so 5b and 5c start from a
boundary rather than an assumption**" section (plan `:2303`-`:2350`) lists 5b's items, 5c's items,
and the two the gate added on 2026-08-25 — and this is in none of them. The ledger's closing line is
"**5a is not closed**, on the five rows above", and all five of those are loud rc-120 refusals. The
one divergence 5a shipped that is a *silent wrong answer* — the class the global constraints treat as
worst — is the one with no phase against its name.

Task 24's own boundary audit added two items it found the lists had dropped; it did not find this
one, because both of the items it added came from the gate tables and no gate row sees this either.

---

## M4 — `::ATTRIBUTE EXTERNAL` is filed to 5a by the gate and to Phase 7 by the interpreter

`rust/crates/rexx-exec/tests/gate_table_d.rs:244` files `("::ATTRIBUTE", "EXTERNAL")` to **`5a`**
(the catch-all arm; `DELEGATE` is peeled off to 5b above it and `::ATTRIBUTE` is not in the 5c arm).
Task 24 measured it as one of the five 5a-owned rows that do not agree, and declined to re-file it
to Phase 7 on the grounds that doing so "would have closed a gate row by narrowing what the gate
covers".

The tree says the opposite in three places:

* `rust/crates/rexx-exec/src/dispatch/native.rs:478`-`:479` — "`::ROUTINE` and `::ATTRIBUTE` carry an
  `EXTERNAL` too and **stay Phase 7's**"
* `rust/crates/rexx-exec/src/dispatch/native.rs:491` — the assert message, "which stays Phase 7's"
* `rust/crates/rexx-exec/src/lib.rs:1527` — `gap("::ATTRIBUTE EXTERNAL", "Phase 7")`, which is the
  **user-visible** refusal: `rexx-exec: ::ATTRIBUTE EXTERNAL is not implemented (Phase 7)`

`owning_phase`'s doc opens "Every arm's authority, in the order the arms appear" and then gives an
authority for the 5c arm, the `DELEGATE` arm, the `::ROUTINE EXTERNAL` arm and the cross-reference
arm. The 5a arm's note covers `::ANNOTATE` only. The `::ROUTINE EXTERNAL` row got a five-paragraph
note recording exactly this kind of tension; the `::ATTRIBUTE EXTERNAL` row, which the plan's own
Task 24 table names as open 5a work, got none.

Both readings are defensible and the plan settled it (plan `:2228` files it to Task 22, a 5a task).
What is not defensible is that the tree holds both and records the disagreement nowhere a reader of
either side would look. `owning_phase`'s doc is where it belongs.

---

## L1 — two laziness justifications that Task 23 voided

`Interp::bootstrap_library` (`lib.rs:4330`) opens with
`self.object_model = Some(dispatch::ObjectModel::bootstrap_for_library())` and is called
unconditionally by `execute` (`lib.rs:6686`), which is the only path `run_program` and
`run_program_collect_every_alloc` both take. So:

* `dispatch.rs:545`-`:547` — "**Built on first use and not in `Interp::new`.** Measured at 5.5 ms per
  build, **which every program that never sends a message and declares no class would otherwise
  pay**." Every program pays now, at interpreter start.
* `environment.rs:383`-`:385` — "Built here rather than in `Interp::new` for the reason
  `dispatch::ObjectModel::bootstrap` is: **it forces the native class set, which a program that never
  names a `.NAME` must not pay for.**" The native class set is already forced by the time this runs.
  (The laziness may still be worth keeping for the *directory* model it builds; the reason given for
  it is not the reason.)

`ObjectModel::bootstrap` — the non-library variant `object_model()`'s `get_or_insert_with` names — is
consequently unreachable on the shipped `rexx-run` path; the `debug_assert!` at `lib.rs:4324` is what
guarantees nothing forces the model before the bootstrap does.

## L2 — an ownership sentence pointing at a task that has run

`dispatch.rs:1046` — "`CoreClasses.orx:93` and `:97` are the same `~inherit` and **the gap belongs to
whichever task runs that file**, not to one value kind."

Task 23 ran that file. Measured: `.String~superClasses` is `Object` / `Comparable` on all three
interpreters, so the `~inherit` really did run — and `say 'abc'~compareTo('abd')` is still
`rexx-exec: method "COMPARETO" of class "String" is not implemented (Phase 5)` at rc 120 against the
oracle's `-1`. The gap survived the task the sentence assigns it to. (The work itself is plausibly
covered by 5c's "the documented per-class **method** sets"; the sentence is what needs correcting.)

## L3 — a "not reachable in this phase" whose premise has changed

`dispatch.rs:1416`-`:1418` — "**The refusing arm needs a second package and so is not reachable in
this phase**: a caller in another package needs `::REQUIRES`, which is refused here."

A second package now exists without `::REQUIRES`: library methods are scoped to `Package::Rexx` and a
program's are `Package::Program`. **The conclusion still holds**, and I checked why rather than
assuming: neither embedded file declares a single `::METHOD … PACKAGE`
(`/bin/grep -acinE "^\s*::method[^;]*\bpackage\b"` answers 0 for both), so `check_package`'s refusing
arm has nothing to fire on. The reason in the doc is no longer the reason.

## L4 — `class_graph.rs:231`-`:238`'s "Nothing inside this crate reads it"

Folded into H1's list above; recorded separately because it is the paragraph a future reader of
`ClassDef::rexx_defined` reaches first, and it will still be wrong after H1 is fixed unless it is
rewritten to name the `library_bootstrap` bypass.

---

## What I checked that is sound

Stated as numbers so a later reader can tell a check that ran from one that did not.

* **Class wiring, 62 classes × 11 methods = 682 differential sends** (`~id`, `~class~id`,
  `~superClass~id`, `~metaClass~id`, `~isA(.Class)`, `~package~name`, `~objectName`,
  `~superClasses~items`, `~isSubclassOf(.Object)`, `~string`, `~hasMethod("ID")`), one program each,
  three descriptors, against the oracle: **0 divergences**, loud or otherwise.
* **`.environment` and `.local`, all 79 index names**, `say .NAME` each: **67 agree, 12 loud rc 120,
  0 silent fallbacks.** `environment.rs:1683`'s universal — "Every name the oracle's `.environment`
  and `.local` hold either resolves here or fails loudly — never the dotted-text fallback" — holds
  as stated.
* **40 adversarial programs at the task seams** (annotations on library classes, `~objectName=` on
  one, scope overrides in six positions including `~m:'x'` and `~m:.Object`, `::class K subclass
  Alarm`, `::class Alarm` shadowing, `~inherit`/`~uninherit` round-trips, `.methods` reads, method
  identity, `~baseClass` on library mixins, `DO OVER .environment`): **0 non-loud divergences and 0
  engine splits.** Every divergence found was a `rexx-exec: … (Phase 5/6/7)` refusal.
* **Setup-method removal** — `.Class~hasMethod('DEFINECLASSMETHOD')`,
  `.Class~hasMethod('INHERITINSTANCEMETHODS')`, `.String~hasMethod('DEFINECLASSMETHOD')`,
  `.Supplier~hasMethod('INHERITINSTANCEMETHODS')`, `.Class~hasMethod('DEFINE')`: `0 0 0 0 1` on all
  three interpreters.
* **Determinism across the bootstrap** — `.Alarm~identityHash .Comparable~identityHash
  .Stream~identityHash .Array~identityHash` plus a `::class K`'s, over 10 `ir` runs and 5
  `tree-walker` runs: one distinct output.
* **`run.rs:821`'s GC-rooting premise** — "nothing in this phase can put a slot of a live array out
  of reach, because `.Array` answers no method that writes one" — holds: `[]=`, `put`, `append`,
  `insert`, `remove`, `empty`, `fill`, `last`, `makeArray` are all rc 120 on a live
  `~superClasses` array; `at` and `items` answer.
* **No claim anywhere that 5a is complete.** `CLOSED_PHASES` is `&[]` as Task 24 decided, and no
  comment in `rust/crates/*/src` asserts 5a closed.
* **No claim that any of the five open mechanisms exists here.** The one that reads like it —
  `run/tests.rs:7681` "`.K~define("SRC", "say 'x'")` installs a compiled method" — is under a doc
  heading that says "The measured **oracle** answer for each"; the crate refuses at rc 120, measured.
* **`dispatch.rs:1080`'s pattern claim reproduces**: case-insensitive `\.package\b` over both `.orx`
  files answers 0 and 0, where `\.array\b` answers 11 and 5.
* **`bootstrap_files` and `rexx-lib` agree** on the three-file set, `PlatformObjects.orx` included,
  so `dispatch/native.rs:441`'s "declares no `EXTERNAL` at all" premise stands.
* **The `donate_instance_methods` / `inherit_instance_methods` split** — Task 23's producer/consumer
  pair — is described consistently at `class_graph.rs:854`, `:877`, `method_dict.rs:267`,
  `registry.rs:596` and `native_classes.rs:105`. The six-comment round at `571275f0b` closed the one
  place they disagreed.
* **Task 21's two deferrals to Task 23 are closed**: `.Stem~superClasses~items` and
  `.Queue~superClasses~items` agree with the oracle on both engines.

## Method

The needle for finding 1 was derived from the concept — *a sentence quantifying over what a program
can observe, what exists at start, or what is unreachable* — as a strong quantifier
(`no|none|nothing|never|cannot|only|every|any|neither|not|yet|un-built`) within 100 characters of a
subject naming something the bootstrap changed (`bootstrap|library|\.orx|CoreClasses|StreamClasses|
setup method|remove_setup|defineClassMethod|inherit_instance|donate|\.environment|interpreter start|
at start|before any|scope override|:super|resident|sourceless|StringTable|do over|iteration order|
prologue|REXX_DEFINED`), run over comment blocks **collapsed to paragraph granularity** — a bare
`///` ends a paragraph, a non-comment line ends the run.

**The needle was validated before it was trusted**, at `86d73d269`, the commit before the
six-comment correction round, against four sentences known to have been stale there: `error.rs`'s
"no activation this crate can put in a sourceless package is a routine", `environment.rs`'s "The two
public-class steps need `::REQUIRES`", `native_classes.rs`'s "`Table`, `StringTable`, `Set`,
`Directory`, `Relation` and `Bag` are the `InheritInstanceMethods` users", and `rexx-lib`'s "would
have been false evidence". **All four are found.** Had the needle missed one, its misses at HEAD
would have meant nothing.

At HEAD it returns 294 paragraphs across 109 source files, of which **236 are byte-identical to their
text at `5b054d4b5`** (Task 22's close) and so are the ones Tasks 23 and 24 could have falsified
without anyone re-reading them. Those 236 were read. A second, broader pass over the same collapsed
index — strong absence and universal shapes with no Task-23-specific subject filter, restricted to
`rexx-exec`/`rexx-classes`/`rexx-core`/`rexx-lib` — supplied L3 and confirmed the rest.

Collapsing first was load-bearing, not a precaution: M1's sentence is invisible to plain grep in both
of the places it appears.

## What this review could not see

* **Anything that needs a rebuild.** Every control that would establish causation by mutating a
  source and re-running was out of scope, so H1's fix shape is reasoned from the code and not
  demonstrated by a build with the flag set.
* **Any divergence in a construct still refused at rc 120.** A refusal that would become a wrong
  answer once the construct lands is invisible to a differential; there are 5b and 5c mechanisms
  behind most of the class surface.
* **Instance-side behaviour.** `~new` is 5b's, so every claim about a library class was checked on
  the class object only. H1 in particular is measured on class-side mutation; whether the same gap
  reaches instance behaviour is not something this branch can be asked yet.
* **The 52 library class-method pairs Task 23's fix round 1 measured as differing in their two
  `Error` lines.** I confirmed the mechanism is loud on both sides and did not re-derive the
  classification; it is Task 23's finding and an inherited `use strict arg` gap, not a boundary one.
* **Timing under load.** The M2 figures were taken on an unpinned machine with other work possible;
  the mins are interleaved across three rounds, which is what makes the ~21 ms gap safe to state,
  but the medians should not be quoted as the suite's own.
