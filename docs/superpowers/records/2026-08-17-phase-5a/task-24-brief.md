## Task 24: the 5a gate

**Goal.** Prove 5a rather than assert it, and hand 5b and 5c a stated boundary.

**Flip the gate.** Add `5a` to `CLOSED_PHASES` in this commit, so every 5a row in both tables is red
from here on under `REXX_CORPUS_GATE=1`. **Then run the five gate commands** -- which now include the
tables' gating arm -- and record a negative control: revert one 5a row's mechanism and confirm the
gate command exits non-zero. Without that control the flip is a change nobody has seen fail.

**Report, each a number or a named absence:**

* the five gate commands;
* both tables: every 5a row `agree`; the 5b and 5c rows reported with their verdicts, so the two later
  phases start from a measured list rather than an assumption;
* **the wiring half of the class-set criterion**: every class named by a `cls*` section across the
  reference, minus `RegularExpression` (no library on this build's path), plus `ArgUtil`, is an
  `.environment` class answering `~id`, `~class`, `~superClass`, `~superClasses`, `~metaClass` and
  `~isA(.Class)` byte-identically; each documented edge appears in that class's `~superClasses`; and
  **`RexxInfo` carries both halves of what the spec says about it** -- it is *out* of the class row
  set, because `EndSpecialClassDefinition(RexxInfo)` routes the class object to a target no
  environment symbol reaches, *and* its `.environment` entry is required to be an **instance whose
  `~class~id` is `RexxInfo`**. Writing only "minus `RexxInfo`-as-an-environment-entry" drops the
  second half and reads as dropping the row;
* **`Queue`, `Stem` and `VariableReference` are inside this set and are Task 21's**, not deferrals
  this report may note in passing. The spec put them in 5a's scope in the same commit that enumerated
  native removal and hiding, and a wiring row that cannot pass is a verdict failure at this gate --
  which is the outcome it wants. **If any deferral in `native_classes.rs` still stands when this task
  runs, this report names the classes the criterion covers and the ones it does not, and why**; a
  surviving deferral that nothing states is the failure mode the whole item was found by;
* **D56's recorded re-derivation run**: one run against a present `oodocs/` whose diff against each
  committed row set is empty, named with the revision it ran at. The spec requires the gate report to
  carry it, and it is the only place the derivations are exercised at a phase boundary -- CI's five
  platforms have no `oodocs/`;
* the corpus, **106 of 106 with no red**, and the count of programs `phase-5a.txt` gained.
  `loop_control_rounding.rex` was the one red while this plan was being written and was closed at
  `2eed4cad5`, before Task 1; this gate carries no exception for it or for anything else, so a red
  here is a finding and not a name to check against a list;
* **the unsafe-block count and the crate roots carrying `deny` rather than `forbid`**, which every
  phase exit owes (roadmap Global Constraint, `2026-07-27-rust-rewrite.md:36`);
* the guard's standing per axis against `bench-baselines/phase-5a-arms.tsv`, `instructions:u`, with
  the note that the axes call none of the new crates -- so a sitting witnesses that the classic paths
  did not get slower and nothing more;
* **cold start and D2**, measured here for the first time in this phase:
  `rexx-bench/src/bin/rexx-time.rs --warmup 10 --runs 50`, compared against the oracle side of
  `perf-baseline.md`'s **"Fixed per-process offset (`startup.rex`)"** subsection of **"The first
  counted baseline, measured 2026-08-20 at `d90de68e3`"** -- named by its exact heading because a
  citation reading "its 2026-08-20 section" no longer identifies one section, that file's other
  2026-08-20 sections being marked superseded in their own headings --
  **the number is read there and not restated here**, because it is a property of that file's
  measurement and this plan should not hold a second copy that can go stale on its own. Two things
  the comparison must carry: that section reports the pre-5a crate as *not comparable* precisely
  because it has no bootstrap, so **Task 23 is what turns this into a real comparison rather than a
  formality**; and `build/` is a `RelWithDebInfo -O2 -g` oracle rather than `-O3`, which
  `rust/CLAUDE.md` requires for a performance claim -- so the ratio is reported with that caveat
  named rather than dropped;
* every committed table this plan edited: `assertions.rs`'s `EXEMPT`, `corpus/bif-exempt.txt` and its
  attribution column, `owners.rs`'s five pinned items, `corpus/builtin-status.txt`,
  `trace_oracle.rs`'s `PREFIX_COVERAGE`, `phase-4-exclusions.txt`'s moved rows.

**Amend the roadmap.** `2026-07-27-rust-rewrite.md:453`'s Phase 5 exit clause "32 classes exist and
respond" is amended to the wiring half above, with the method half stated as 5c's (D48). Measured:
that number is exactly `/bin/grep -acE "^::[Cc][Ll][Aa][Ss][Ss]" CoreClasses.orx`, and the shipped
oracle's `.environment` holds 62 classes -- so the old clause is satisfiable with a third of the
environment missing and by a build that never runs `StreamClasses.orx`.

**Name what 5a did not cover**, so 5b and 5c start from a boundary rather than an assumption:

* **5b's** -- `~new`, `init` and `self~init:super` chaining as *instance* construction; `UNINIT`
  itself and everything that **reads** the propagation flags, the flags themselves being Task 7's and
  carried by its constructors -- **and travelling with that row, the class-side witness Task 7
  measured and could not use**: a class object is destroyed like any other object, so a
  `::METHOD uninit CLASS` fires with no `~new` anywhere, and Task 7's programs (a plain `::CLASS`, a
  `SUBCLASS` of one carrying the method, and an `INHERIT` of a mixin carrying it) are a differential
  5b can run on the day it starts -- the first two silent divergences on this crate today and **all
  three by the time this handover is read**, because the third is silent only once Task 7 lifts the
  `MIXINCLASS` refusal that currently makes it loud -- so 5b does not have to look for an instance to
  see the firing; per-object methods (`SETMETHOD`, `ENHANCED`,
  `unsetMethod`) and the object-own scope they create; `FORWARD` and therefore `DELEGATE`;
  abstract-**class** enforcement;
  `~copy`; `~run`, `~send`/`~sendWith`, `~start`/`~startWith` minus concurrency; the per-object first
  step of the method search order; **and the instance-side reading of every 5a limit measured only on
  a class object** -- the old plan's Task 7 debt, this plan's Task 13 (`PRIVATE`), Task 19
  (`::CONSTANT`) and Task 21 (D43's two orderings).
* **5c's** -- `::REQUIRES` with `LIBRARY` and `NAMESPACE` and namespace-qualified class references;
  **`>N>`**, whose only route is one of those references; `::OPTIONS` and the `OPTIONS` instruction;
  `::RESOURCE` and `.RESOURCES`; `::ROUTINE`'s option surface and whatever of `.ROUTINES` Task 17's
  widening left. **The readback of an `::ANNOTATE ROUTINE` is not among them**: Task 17 populated
  `.ROUTINES` far enough that `.routines["R"]` and `.routines~r` both resolve, and Task 20 reads the
  annotation back through both, `corpus/lang/directive_annotate_targets.rex` being the witness.
  `Package~findRoutine` is still unbuilt and is still 5c's. `Package~local` and
  environment search steps 3 and 5; **`PACKAGE`'s cross-package refusal arm**; `~identityHash`'s
  identity semantics and D41; **the documented per-class method sets**, which is the acceptance set
  Phases 6, 7 and 8 enter on.

**Done when** every 5a row in both tables reads `agree` under `REXX_PHASE_GATE=5a`, the flip's
negative control is recorded, the five gate commands pass, and the report names each item above as a
number or a named absence.
