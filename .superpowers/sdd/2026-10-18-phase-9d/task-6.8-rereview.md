# Task 6.8 re-review: fix round 1

This scoped re-review covers `f5e9a493e..0b405bf1e`. The code commit is `a88871bb3`. I built HEAD into the session scratchpad (`rev68r1-target`, `cargo build --release --bin rexx`, exit 0). Probes ran from one fresh `mktemp -d` (`scratchpad/rev68r1-probes.9qmc/`, which is not durable, so the deciding lines are quoted below). The rules are the same as the first review, with the oracle wrapper, `memcap 2G timeout 20`, and stdout, stderr and rc compared separately. There are two comparison binaries:
* "round 0" is `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.8/rexx-head`, which is `33298eb3b`.
* "pre-task" is `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.7-r1/rexx-head`.

## Review Important 1 and 2: closed

I re-ran all fifteen of my first-review probes on HEAD.
* **Important 1 is closed.**
  * `procedure expose (a.i)` gives `pr2 LIT VAR LIT` on the oracle and HEAD, and the four trace lines are identical.
  * `procedure expose (a.i) i` gives `pr3 LIT VAR VAR` on both.
  * Under `::OPTIONS NOVALUE SYNTAX` both sides now give `E 98.986 Reference to unassigned variable "A.I".`.
* **Important 2 is closed.**
  * A second `use arg >abc.` after the caller's `say a.`, `drop c.` or a dropped tail binds, as on the oracle: `twiceStem answers set`, then `a.1 set`.
  * `abc. = q.` still refuses.

The only remaining differences in those probes are the ones the first review already sorted:
* Minor 3's list ordering: `procedure expose i (a.i)`, `drop list (list)`, `drop (w) (v)`, method `expose i (a.i)`, and the reversed `>C>`/`>V>` order in `e7`.
* The implementer's Concerns 1 to 3.

## Strengths

* **Rooting is not needed.** `reference_names` holds `(SlotFrame, usize, Box<[u8]>)`, so it holds no `ObjRef`. `object_roots` ignores it with a comment (`reference_names: _`).
* **No cost on the hot path.** The field is in the boxed `ActivationCold`. It is written only in the `VarRefHome::Cell` arm of `bind_use_target` and read only in `target_is_uninitialised`, and both are on the `USE ARG >name` path only. I ran `callgrind.sh -r 1 -B 0.5 -p "interpret churn"` with the pin, round 0 and HEAD, exit 0:
  * interpret: 4116410290 to 4116113889, -0.0072% against round 0.
  * churn: 632210430 to 632211369, +0.0001%.
* **These lifetime paths match the oracle:**
  * The outer `use arg >abc.` first, then an internal call doing the same (`outerThenInner answers ok`).
  * Two INTERPRETed binds (`interpTwice answers ok`).
  * A direct bind, then an INTERPRETed one (`interpMixed answers ok`).
  * A SIGNAL to a label inside the internal routine, then a rebind there (`signalOut`, `c.1 set`).
  * A method that binds twice (`m1 answers ok`, `m.1 m1`).
  * A method's internal call, then the method's own bind: 98.995 on both sides, and on the pre-task binary too.
  * DROP of the aliased stem, then a rebind: 98.995 on both sides. The pre-task binary bound here, so this is a fix.
* **Tests:**
  * `cargo test --release -p rexx-exec --lib`: exit 0, 1152 passed.
  * `REXX_CORPUS_GATE=1 ... --test corpus`: exit 0, `1096 of 1096 matching`.
  * `-p rexx-parse --test sourceline_oracle`: exit 0.

## Issues

### Critical

None.

### Important

1. **A reference record is per activation, but an alias belongs to the frame, so a bind made in a frame-sharing internal call is lost on return.** The implementer predicted this without measuring it, and I have now measured it. The probes call `procedure` routines with `>a.` and `>h.`, where the caller has run `say a.` and `say h.`:
   * `innerThenOuter: procedure; call inner1 arg(1); use arg >abc.; abc.1 = 'set'`, where `inner1: use arg >abc.; return` has no PROCEDURE:
     * Oracle and pre-task: `innerThenOuter answers ok`, and the caller then reads `a.1 set`.
     * HEAD (and round 0): `98.995 Unable to reference variable "ABC."; it must be an uninitialized local variable.`, and the caller reads `a.1 A.1`.
   * `innerTwiceOuter`, where the internal routine binds `>abc.` twice and the caller binds once more:
     * Oracle and pre-task: `answers ok`, `h.1 set`.
     * HEAD: 98.995, `h.1 H.1`.

   The reverse direction (outer binds first, then the internal routine binds) matches. The internal routine's own lookup misses the outer record and falls back to the target's name, but that name equals the stem's name in my probe. Fix: keep the record with the frame's slot rather than in the activation. One option is to store it alongside the alias in `roots`. Another is to carry it in the internal call's activation and hand it back when the call returns.

2. **A reference made from an alias still names the alias, so a second bind fails from two levels deep.** In `deep1: procedure; use arg >x.; call deep2 >x.` / `deep2: procedure; use arg >y.; call deep3 >y.` / `deep3: procedure; use arg >z.; use arg >z.; z.1 = 'set'`, with the caller's `say g.`:
   * Oracle and pre-task: `deep1 answers ok`, `g.1 set`.
   * HEAD (and round 0): `98.995 Unable to reference variable "Z."; ...`, `g.1 G.1`.

   The `>y.` reference carries the name `Y.`, while the stem in the shared cell is named `G.`, so the comparison in `target_is_uninitialised` fails. The oracle's `VariableReference` wraps the original `RexxVariable`, whose name stays `G.`. This is the same root cause as the implementer's queued Concern 4, but here it changes a 98.995 outcome, and that outcome is new with this task: the pre-task binary bound. Fix: compare with the name of the variable that owns the cell, not with the name the reference was spelled with. For example, the record could take the caller's own record when the caller's slot is itself an alias, or the cell could carry its owner's name. Add a witness for a reference passed on twice.

### Minor

None new.

## Assessment

**Task quality:** One more fix round.

**Reasoning:** Both review findings are closed, as measured by the original probes, and the procedure-frame read matches the oracle line for line. The new field costs nothing on the hot path and needs no rooting. But the name record does not follow the alias it describes. It is lost across a frame-sharing internal call, and it records the spelled name rather than the owning variable's name. Both cases leave a USE ARG outcome (98.995 against a bind) that the pre-task crate got right.

**Verdict: CHANGES REQUESTED** (findings 1 and 2).
