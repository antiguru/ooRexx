# Task 6.8 review: variable references

Reviewed `f2444c6ae..8b344309d`. I built HEAD from `rust/` into the session scratchpad (`rev68-target`, `cargo build --release --bin rexx`, exit 0). Every probe ran from one fresh `mktemp -d` (`scratchpad/rev68-probes.ugrQ/`, which is not durable, so the deciding lines are quoted below). The oracle ran under the standard wrapper and the crate under `memcap 2G timeout 20`. I compared stdout, stderr and rc separately. The parent binary is `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.7-r1/rexx-head`. `git diff bacf18d5a f2444c6ae -- rust/` is empty, so that binary carries the parent's code. I used it to sort each remaining difference into "pre-existing" or "changed by this task".

## Spec compliance

Compliant in what it claims. The three witnesses match the oracle in the gated corpus (1096 of 1096). The selector read now does NOVALUE and tracing the way an expression read does, and the 98.995 rule now covers exposed targets. But each of the two fixes over-reaches in one place, and in both places the parent matched the oracle and HEAD does not (findings 1 and 2). Neither witness reaches either place.

## Strengths

* **DROP selectors match across the value space.** These all match the oracle on stdout, stderr and rc: mixed case, surrounding blanks, an empty value, a tab separator, an empty stem, a stem with a default, a dropped stem, a stem whose value names stems and compounds (`'a.1 b.'`), compound selectors with set, unset and defaulted tails, `j = 'J'; s.J = 'm'; drop (s.j)` (the parent got this wrong), `'9'`, `'.x'`, `'1abc'`, `'a+b'`, `'(w)'`, `'a'||'0a'x||'b'`, an Array value, `drop (1)`, `drop (.nil)`, `drop (.environment)`, `drop ('v')`, `drop ()`, and the INTERPRET forms of each.
* **NOVALUE matches.** Under `::OPTIONS NOVALUE SYNTAX`, DROP gives 98.986 for simple, stem, compound and INTERPRET selectors, and none for `.nosuchenv`'s parse error. `SIGNAL ON NOVALUE` matches over DROP, over PROCEDURE EXPOSE, over a stem selector (no condition), over an unset-tail compound selector, and through INTERPRET, `condition('d')` and `sigl` included.
* **Tracing matches.** TRACE I, R, A, N and O over DROP, PROCEDURE EXPOSE, INTERPRET, compound and stem selectors match byte for byte, `>C>` before `>V>` and the procedure's own indent included (`t{i,r,a,n,o}.rex`). Method EXPOSE under `::OPTIONS TRACE I` with `(a.i)`, `(b.)`, `(vv)` and `(n.)` matches, except for the cases in finding 1 and Minor 3.
* **The exposure rule holds.** These targets all match the oracle (98.995, or the bound value): a procedure-exposed object variable (`p1: procedure expose v; use arg >v`), an internal call that shares a method's exposed variable, `USE STRICT ARG`, an exposed target that was dropped first, `use local zz` and then an auto-exposed `v`, an exposed stem dropped first, an exposed compound with a simple target, an exposed object variable under `GUARD ON`, `interpret 'use arg >ie'` on an exposed name, two INTERPRETed `use arg >it`, a `::ROUTINE` target, and write-through of a bound alias. `use arg >a = 1` gives 99.950 on both sides.
* **The fix the name check was for holds.** `q.9 = 1; drop q.9; abc. = q.; use arg >abc.`: the parent bound, the oracle and HEAD give 98.995. `abc. = .stem~new('ABC.')` and `say abc.` followed by `use arg >abc.` bind on all three. A target exposed by `procedure expose zz.` binds.
* **Perf reproduces.** `callgrind.sh -r 1 -B 0.5 -p "interpret churn"` against the pin, the parent and my build, exit 0: interpret 4116240712 to 4116415381 (+0.0042% head against the parent), churn 632192801 to 632215757 (+0.0036%). The report gives +0.0040 and +0.0028.
* **Tests.** `cargo test --release -p rexx-exec --lib`: exit 0, 1152 passed. `REXX_CORPUS_GATE=1 ... --test corpus`: exit 0, `1096 of 1096 matching`. `-p rexx-parse --test sourceline_oracle`: exit 0.

## Issues

### Critical

None.

### Important

1. **`PROCEDURE EXPOSE` reads a compound selector's tail in the caller's frame. The oracle reads it in the procedure's new frame. This regressed stdout that the parent got right.** `RexxVariableReference::procedureExpose` (`IndirectVariableReference.cpp:199`) exposes the selector and then calls `list(context)`. Here `context` is the new activation, so a tail variable that has not been exposed reads as its own name. `expose_names` (`run.rs:1595`) instead reads through `read_list_selector` before isolating, so the tail resolves against the caller's variables. The probe is `i = 2; a.2 = 'p q'; p = 'cp'` followed by `call pr2`, with `pr2: procedure expose (a.i); say symbol('p') symbol('a.2') symbol('a.I')`:
   * Oracle: `pr2 LIT VAR LIT`, with trace `>C> A.I => "A.2"`, `>C> A.I => "A.I"`, `>V> A.I => "A.I"`, `>C> A.I => "A.2"`.
   * Parent: `pr2 LIT VAR LIT`.
   * HEAD: `pr2 VAR VAR LIT`, with trace `>C> A.I => "A.2"`, `>V> A.I => "p q"`, `>C> A.I => "A.2"`. So HEAD exposes `P` and `Q`, and the oracle exposes neither.

   The same probe under `::OPTIONS NOVALUE SYNTAX` and `SIGNAL ON SYNTAX`: the oracle gives `E 98.986 Reference to unassigned variable "A.I".`, HEAD `pr VAR` / `back` (no condition), and the parent `pr LIT` / `back`. `procedure expose i (a.i)` and `procedure expose (a.i) i` regress the same way (`pr LIT VAR 2` on the oracle and the parent, `pr VAR VAR 2` on HEAD). Part of the reason is the ordering in Minor 3: on the oracle, `(a.i)` runs before `i`. The witness reaches only simple selectors in PROCEDURE EXPOSE, which read the same in either frame.
   Fix: for PROCEDURE EXPOSE, read the selector as the new frame would see it. Tail variables resolve only through names already exposed, and the rest read as unset, including the NOVALUE check. This touches Concern 1 (the crate isolates only after every name resolves), so the controller may want to rule how far to take it. At minimum, add a witness for `procedure expose (a.i)` with `i` unexposed.

2. **The stem-name check refuses a second `USE ARG >abc.` that the oracle and the parent bind.** `isAliasable` compares the stem's value with *the variable's own* `variableName` (`RexxVariable.cpp:257`). After a first `use arg >abc.`, the target slot holds the caller's variable, whose name is `A.`. So a second `>abc.` passes on the oracle. `is_uninitialised_stem(value, name)` compares against the *target's* name `ABC.` and refuses. The probe calls `procedure; use arg >abc.; use arg >abc.; ...` with the caller's stem in three states:
   * `say a.` (a vivified empty stem): the oracle and the parent give `twiceStem answers set` and the caller then reads `a.1 set`. HEAD gives `98.995 Unable to reference variable "ABC."; it must be an uninitialized local variable.` and the caller reads `a.1 A.1`.
   * `drop c.`: the oracle and the parent give `answers ok`, HEAD gives 98.995.
   * `d.1 = 'x'; drop d.1`: the oracle and the parent give `answers ok`, HEAD gives 98.995.
   * A never-touched `b.` binds on all three, because HEAD's caller slot stays `None` (Concern 3).

   Fix: compare against the name of the variable the slot resolves to, which is the caller's name when the slot is an alias, and add these three cases to `use_arg_reference_exposed`.

### Minor

3. **Queue candidate: the oracle runs a variable list out of source order, and the crate does not.** `processVariableList` (`InstructionParser.cpp:4495` and `:4523`) puts direct names into the list with `pushSubTerm` (front) and `(name)` selectors with `subTerms->queue` (back). The constructor then pops LIFO (`DropInstruction.cpp:61`). So every selector runs before every direct name, and the selectors run in reverse order. The same applies to EXPOSE and PROCEDURE EXPOSE. Pre-existing, with the same stdout on the parent:
   * `list = 'x'; x = 1; drop list (list)`: the oracle drops `X` (`A LIT LIT`). The crate drops `LIST`, reads `"LIST"` and leaves `X` (`A VAR LIT`, trace `>V> LIST => "LIST"` against the oracle's `"x"`).
   * `v = 'x'; x = 1; w = 'v'; drop (w) (v)`: the oracle gives `D LIT LIT VAR`, the crate `D VAR LIT VAR`.
   * In a method, `expose i (a.i)`: the oracle gives `LIT A.2 LIT 2`, the crate `VAR p q VAR 2`.
   * In `procedure expose (q.1) (q.k)` and `expose (a.1) (b.)`, the `>C>`/`>V>` lines come in reverse order on the oracle.

   This is an oracle defect, so it is not to be copied. But it is observable, so it is a Deviation candidate for Moritz's batch. It is also the cause of Concern 1 (next section).
4. **Pre-existing, outside these clusters: the `.local~novalue` hook is not implemented.** With `.local~novalue = .nv~new`, the oracle sends `NOVALUE` for `say zzz`, for `a.7`, and for the selectors of `drop (zzz)` and `procedure expose (yyy)` (`novalue hook ZZZ` / `qq`). The crate prints `ZZZ`. The parent is the same. I found no queued item naming it.

## The implementer's concerns

I re-ran Concerns 1, 2 and 3, and each claim holds as stated: `c=C d=D` on the oracle and `c=3 d=4` here; 98.995 with `'AB' || 'C.'` and a bind with the literal; `t3 A.1` / `caller VAR` on the oracle and `ABC.1` / `LIT` here. One addition to Concern 1, which does not make it false: the oracle's split is not an isolation artefact. It is Minor 3's ordering. In `procedure expose c d (sel)` the oracle runs `(sel)` first, so it fails before `c` and `d` are exposed. The selector is still exposed because `procedureExpose` exposes it before reading it. So the queued Deviation candidate and Minor 3 are one ruling, not two. I did not re-run Concern 4 (p14).

## Assessment

**Task quality:** Needs a fix round.

**Reasoning:** The selector read and the exposure check are right on the paths the clusters' tests reach. I found nothing to fault across DROP, TRACE, NOVALUE and the exposed-target matrix. But each fix also changed one path where the parent already matched the oracle: the PROCEDURE EXPOSE compound selector now reads its tail in the wrong frame, and the stem-name check uses the target's name where the oracle uses the aliased variable's name. Both are stdout regressions that the witnesses cannot see.

**Verdict: CHANGES REQUESTED** (findings 1 and 2; 3 and 4 are queue candidates).
