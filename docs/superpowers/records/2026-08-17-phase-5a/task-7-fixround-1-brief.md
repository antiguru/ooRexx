# Task 7, fix round 1

The review verified a great deal by running it, and found the class-graph work sound. The shapes the
committed programs do not cover were measured and are byte-identical on the oracle and both engines: a
subclass of an inheriting class, a mixin of a mixin, `SUBCLASS P INHERIT M` where `P` already inherits
`M` (98.944 with the frame), `INHERIT M ZZZNOSUCH` (98.909, no frame), `INHERIT C M` (98.942 with the
frame), and cycles through inherit edges and through mixinclass edges alike (98.911 blaming the first
directive). `ClassGraph::inherit`'s check order maps one to one onto `ClassClass.cpp:1298-1332`, and
M10 was verified in the code rather than from your report.

Two Important findings, both about UNINIT. Your own disclosure named the first; the review found it is
worse than disclosed, and found a second.

## 1. The flag is not merely unset through the directive path -- it can never be set

Checked in code, not reasoned: `install_directives` (`rexx-exec/src/lib.rs:3437-3467`) creates every
class in one loop and attaches every method in the next, and `UNINIT` occurs zero times in the
generated `setup_classes.rs`. **So `parent_has_uninit` is `false` for every class a program can
declare, always.** The only writer is a direct `ClassGraph` call, which is what the in-crate test
makes -- in an order (`define` before `define_class`) the directive path never produces.

And the part that decides the handover: **the brief's three programs use `::METHOD uninit CLASS`,
which routes through `add_class_method` to `class_define` and sets nothing**, so `has_uninit` is
unreachable for them too. The test passes over a layer no directive-installed class reaches, and 5b
would inherit a flag it cannot read.

Your root cause checks out -- the two-pass install is confirmed in code, and
`phase-4-exclusions.txt:519-541` carries the `::CONSTANT` row with the same one.

**Ruling: try the narrow fix, and stop if it turns into the install reordering.** The narrow shape is
to set `has_uninit` where a class method named `UNINIT` is attached, and to run the
`parent_has_uninit` propagation as a pass after methods are attached rather than during class
creation. If that works, the flags become true of declarable classes and the task delivers what it was
given. **If it cannot be done without reordering install itself, stop and say so precisely** -- that
reordering is a different subsystem, it is the standing phase-4 exclusion's own root cause, and it is
not this task's. A precise report is the acceptable outcome there; what is not acceptable is the
current state, where a test passes and the thing it is named for is unreachable.

**Whichever branch you take, three things are owed:**

* the in-crate test must say in its own name or its doc that it exercises the **graph API** and not the
  directive path, so it cannot be read as evidence about programs;
* the handover record must carry that `parent_has_uninit` is false for every declarable class today,
  with the two-pass install as the reason and the `phase-4-exclusions.txt` row as the precedent;
* the three handover programs must carry the finding that `::METHOD uninit CLASS` reaches
  `class_define` and sets nothing, so 5b starts from that rather than rediscovering it.

## 2. `ClassDef::has_uninit` is documented as the oracle's `HAS_UNINIT` and is not it

The oracle also sets that flag in `checkUninit` (`ClassClass.cpp:1210-1218`) from the **flattened**
behaviour, called from `subclass` (`:1626`), so an inheriting class has it set there where ours does
not. The site the flag exists for reads exactly it -- `:1892`,
`if (hasUninitDefined()) obj->requiresUninit()` -- so 5b following your doc under-registers silently.

**And a test pins the divergent value:** `behaviour_wiring.rs`'s `assert!(!g.has_uninit(child))`
asserts our answer is correct where the oracle's differs.

**Ruling: correct the doc, and unpin the assertion.** The doc must say what our flag is and name where
the oracle's diverges, with the `checkUninit` citation. The assertion must not encode a divergence as
expected: either drop it, or keep it with the oracle's value recorded beside it and the divergence
named as a divergence. An assertion that quietly freezes a wrong answer is worse than no assertion,
because it will be cited as a decision.

## Minors

* `behaviour_wiring.rs:566` says `98.943` for a recursive-inherit case; the twin sentence in
  `class_graph.rs` was corrected to `98.944` in this same commit.
* Two comments name a set size: `class_references`' "exactly these three", and `error.rs`'s "on three
  shapes".
* `dispatch.rs:862-867`'s `pub(crate)` rationale for `blame_native_method` names only the two `eval.rs`
  situations, and `lib.rs`'s directive send is a third caller from a third module. Do not extend the
  list -- state the rule that admits a caller, the way the helper's own doc does.
* `class_names_a_namespace` also swallows `METACLASS ns:`, and the new `phase-4-exclusions.txt`
  paragraph names only `SUBCLASS`/`MIXINCLASS`/`INHERIT`, with no in-crate row covering the fourth
  spelling. Loud either way, so this is a record defect rather than a behaviour one.

## One thing to clarify rather than fix

The reviewer could not reproduce control 1 from your description: a literal same-polarity substitution
would not produce the flip you report, and the observable you describe requires building a
`MIXINCLASS` directive as a plain subclass, which is what the plan's own line describes. It judged the
control genuine. Rewrite the description so it names the mutation someone else can apply and get your
result.

## Verification this round owes

* The five gate commands, each status read **unpiped**.
* If you take finding 1's narrow fix: the three handover programs measured on both engines against the
  oracle, before and after, and the in-crate test shown failing without the change.
* If you do not: the precise account of what reordering would take, and why it is not this task's.
* Write the report section **before** you commit.
