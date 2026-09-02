# Task 4: `FORWARD` and `DELEGATE` (D62)

**Goal.** Gate table D's two 5b rows agree **over probes that exercise the mechanism**. They agree
today over probes that do not, which is the point of the task.

**BASE:** the commit named in your dispatch. Read `global-constraints.md` and the 5a constraints it
points at, `rust/CLAUDE.md`, the plan's Task 4 section, and D62 in the spec.

## Measured by the controller at `e30055b37` -- re-measure it, it is a claim and not a premise

**The two committed rows are green over nothing.** Both are `say 'main'` plus directives and never
send the delegated message, so both sides print `main` at rc 0 whether or not `DELEGATE` exists:

```
attribute__delegate__subkeyword.rex     say 'main' / ::class k / ::attribute d / ::attribute at delegate d
method__delegate__subkeyword.rex        say 'main' / ::class k / ::attribute d / ::method m delegate d
```

A probe that actually sends is rc 120 on both engines against the oracle's rc 0:

```
o~d = .Helper~new ; say 'attr' o~at ; say 'meth' o~m
  oracle          rc 0    attr helper-at / meth helper-m
  ir, tree-walker rc 120  a ::ATTRIBUTE with no body of its own is not implemented (Phase 5)
```

**The gap is the bodiless forms, not attributes.** Plain `::ATTRIBUTE v` with no body already works --
`o~v = 7 ; say o~v` is rc 0 printing `7` on both sides. What is missing is `::ATTRIBUTE at DELEGATE d`
and `::METHOD m DELEGATE d`, which refuse as "a ::ATTRIBUTE / ::METHOD with no body of its own".
The first probe's message names `::ATTRIBUTE` only because the delegating attribute is itself
bodiless; do not read it as attributes being unimplemented.

**`FORWARD` as an instruction**, oracle rc 0 `a base-cm` against rc 120 on both engines:

```rexx
say 'a' .K~cm
::CLASS Base
::METHOD cm CLASS
  return 'base-cm'
::CLASS K SUBCLASS Base
::METHOD cm CLASS
  forward class (super)
```

## `keyForward`'s six options, measured -- the plan asked for this before building

The plan says `instrc.xml`'s `keyForward` names `TO`, `CLASS`, `MESSAGE`, `ARGUMENTS`, `ARRAY`,
`CONTINUE`, that two were measured and four were not, and that this phase must **say which arms it
owes**. Measured against the oracle, one probe each, a `K SUBCLASS Other` forwarding out of `m`:

| option | oracle |
|---|---|
| `TO` | rc 0, reaches the named target |
| `CLASS` | rc 0, reaches the superclass method |
| `MESSAGE` | rc 0, reaches the renamed method on the same receiver |
| `ARGUMENTS(.Array~of(1,2))` | rc 0, the callee sees the replaced arguments |
| `ARRAY(1,2)` | rc 0 **when a target or message is named** |
| `CONTINUE` | rc 165, execution resumes after the FORWARD and the result is available |

An earlier reading of `ARGUMENTS` as rc 245 was a malformed probe (`.array~of`), not a finding.

## The shape you must not run, and the ruling that goes with it

**A `FORWARD` without `CONTINUE` that resolves back to the method it is in SIGSEGVs the oracle** --
rc 139, deterministic, both descriptors empty. The minimal form is a bare `forward` in a method.
`forward array(1,2)` and `forward message('M')` crash identically, because `FORWARD` fills an
unspecified target with `receiver` and an unspecified message with `settings.messageName`
(`execution/RexxActivation.cpp:1335`-`:1347`), so the send returns to the method it left. The
non-continuing path makes the activation a phantom **before** it sends -- `setForwarded`,
`stopExecution(RETURNED)`, then `messageSend` (`:1365`-`:1391`) -- so the recursion never grows the
activation depth the resource guard counts, and the C++ stack goes instead. One keyword bounds it:
the same program with `forward continue` is a clean rc 245, and so is ordinary unbounded Rexx
recursion. The full entry is in `rust/corpus/oracle-crashes.txt`.

**The ruling, so you do not have to invent one.** The oracle's answer here is a **signal, not a
behaviour**. Do not reproduce the SIGSEGV and do not build toward it. Answer this shape the way the
oracle answers the *bounded* one -- a clean resource-exhaustion refusal, rc 245 -- and record it as a
licensed divergence in the terms Task 9's list uses for the oracle's `Error 5` stack overflows.
Matching an oracle crash is not a target. **Never run any program of that shape**, including while
exploring.

## Build

`FORWARD` as an instruction, then `DELEGATE` as the equivalence `dire.xml` states it: `expose
delegateName` plus `forward to(delegateName)`, and for `::ATTRIBUTE` the pair `name` and `name=`.
Measured: a plain `forward class (super)` **returns from** the forwarding method rather than
continuing it, which is the non-continuing path above.

`FORWARD` needs no instance and could have landed before Task 1, which is why it gets its own corpus
witness rather than riding on the `DELEGATE` rows.

## Done when

* Both table D 5b rows agree **over replacement probes that send the delegated message**, and the
  replacements are in `corpus/phase-5b.txt` and `EXPECTED_SUBSET_5B` in the same commit;
* `FORWARD` has its own corpus witness, separate from the `DELEGATE` rows;
* **every option you build has a witness, and the report says which of the six this phase owes and
  which it does not** -- that is the plan's explicit instruction, and four of them currently have no
  acceptance criterion anywhere;
* the licensed divergence for the self-forward shape is recorded with its measurement;
* **controls**: for each replacement row, delete the behaviour it witnesses and show it reddening.
  The old probes are the negative case worth stating -- show that the *replacement* reddens under a
  mutation the *current* probe survives, since that is the whole reason for replacing them.

## Hazards

* **A witness that cannot fail** stands at six instances this phase, and **the two rows you are
  replacing are themselves that shape** -- committed gate rows, green over an unimplemented
  mechanism. Do not produce a third.
* **A witness that passes for the wrong reason.** When a row agrees, construct the control separating
  your explanation from the nearest wrong one.
* **"Can fail" is not "adds coverage."** Run each mutation against the corpus without your new row.
* No comment states the size of a set or a mutable in-repo aggregate. ASCII only, no em-dashes.

## Gates and records

Five gates from `rust/`, each status read unpiped from its own file, never chained with `&&`, plus the
phase-gate command, which exits 101 by design while `methodsbyclass` is red. Gate 4 is about three
minutes, gate 5 about ten. A `src/` change owes a sitting against
`bench-baselines/pinned/rexx-run-f558ea501` with `--baseline bench-baselines/phase-5b-arms.tsv`;
`rexx-arms` **appends** to whatever `--baseline` names, and `rexxcps` needs a path axis, not a stem.
Report to `task-4-report.md`, file first, append as you go. Message the controller when you finish.
