# Task 8.5 review: variables

Reviewed `6e0bf5e5b..42df2784e` (code `b69a1babc`, `2589e88b3`; `git diff --stat 2589e88b3 42df2784e -- rust/` is empty).
BASE was built from a `git archive` of `6e0bf5e5b` (`rust/` and `interpreter/`, which `rexx-classes/build.rs` reads) into
`/tmp/claude-1000/t85-review-target-base`, HEAD from the checkout (only `progress.md` modified) into
`/tmp/claude-1000/t85-review-target-head`, both `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 --bin rexx`,
exit 0. Every probe ran three ways (oracle under the standard wrapper, base and head under `memcap 2G timeout 20`)
from fresh empty run directories under `t85-review-scratch/`, stdout, stderr and rc compared separately. Probe paths
below are relative to `t85-review-scratch/`; the deciding lines are quoted.

## Verdicts

* Spec compliance: **COMPLIANT.**
* Quality: **CHANGES REQUESTED** (Critical none, Important 1, Minor 3; queue candidates 2; new oracle defects none).

## Spec compliance

Every item of the 8.5 batch is handled as the dispatch asks. Each one is closed and witnessed, or (for the
concatenation item) stopped with its derived rule and departure reported. Moritz has since ruled that one
(`progress.md`: "Rule everywhere, Deviation"), and it goes to fix round 1. The extra item m1 has its status line, and
its file stays open. The Deviation candidate blocks A, B and C are written with the rule, the C++ cause, the
measurement and the witnesses. No numbered Deviation was written.

* **Witnesses, red at BASE and matching at HEAD.** I re-ran the new corpus programs myself. My three-way result is the
  same as `t8.5/witnesses/summary.tsv`, cell for cell. `compound_use_creates_stem` differs at base on stdout.
  `local_novalue_hook` and `variable_reference_through_alias` differ at base on stdout and stderr. At head every one
  match the oracle on all descriptors, and `local_novalue_hook_syntax` matches at base too (it pins the order, as the
  report says). I read the oracle stdout of each witness, and the paths the report lists all print (`s15` twice,
  `nilhook`, `nothing`, `s20 NV_YYY2`, `NOVALUE cond YYY3`; `cell LOC`, `again set through q`, `inst V`, `r= V`,
  `refLocalStem: A.1`; `n 1 1 1 0`).
* **Unit tests.** `t8.5/witnesses/unit-tests-base-6e0bf5e5b.txt` shows the table test FAILED at base and the pins ok.
  At head `cargo test --release -p rexx-exec --lib`: exit 0, `1191 passed`.
* **Closing lines and inventory rows** follow the batches header. Each closed file's first line is `**CLOSED
  2026-10-27 by Phase 9d Task 8.5 (2589e88b3).**`. Columns 4 and 6 read `closed:2589e88b3`, and the reason column is
  left as it was. `t8.5/closed.tsv` holds `gate:phase-9c-gate.md:138 closed:2589e88b3`, and its inventory row is
  closed the way 8.3 and 8.4 closed their gate rows. The suite tables in `t8.5/final/suite/` (`tests.tsv`,
  `table.tsv`, `verdicts.tsv`, `oracle-differences.tsv`) are byte-identical to `t8.4/r1/suite/` (`cmp`).
* **Candidate blocks, checked.** Every probe the blocks cite exists (`t8.5/probes/*.rex`, `t6.8/probes/p5`, `p9`,
  `p14`, `p16`). I re-ran the t8.5 probes, and their outputs are the ones quoted:
  * `drop_list_order`: oracle `LIT LIT`, head `VAR LIT`.
  * `selector_order_trace`: the oracle traces `B` before `A` for each instruction, head `A` before `B`.
  * `procedure_expose_handler`: oracle `n1 NOLIST2 c=C d=D` / `after pf 3 4`, head `n1 NOLIST2 c=3 d=4` /
    `after pf h h`.
  * `stem_name_identity`: on the computed name, oracle `98.995` and head `bound`.
  * `context_variables_alias`: oracle `stem 0 1 1`, head `stem 1 0 1`.

  Every C++ citation lands on the line it names:
  * `InstructionParser.cpp:4495` `pushSubTerm` and `:4523` `subTerms->queue`.
  * `DropInstruction.cpp:68`, `RexxInstruction.hpp:201-207`, `ExposeInstruction.cpp:62`, `ProcedureInstruction.cpp:63`.
  * `RexxVariable.cpp:270` `stem->getValue() == variableName`.
  * `VariableDictionary.cpp:656` `result->put(..., variable->getName())`.
  * `RexxLocalVariables.cpp:544` `aliasVariable`.
  * `VariableReference.cpp:245-249` `processUnknown`, `:258-261` `stringValue`, `:364-370` `request`.

  The rexxref quotes are verbatim: intro.xml:2562, collclasses.xml:8033, instrc.xml:1075, intro.xml:2159.
* **Perf.** `t8.5/final/callgrind.txt` covers every program and has no SPREAD or OVER flag. Libc subtracted, no
  figure is above `-0.0000`, and the lowest is define at `-0.0023`. Whole process, hashnew is the largest at
  `+0.0147`. The measured head is 2589e88b3 code:
  * `cg-9d/t8.5/rexx-head-2589e88b3` holds the `novalue_answer` symbol (`nm -C`, count 1), and
    `rexx-head-b69a1babc` lacks it (count 0). `novalue_answer` first appears in 2589e88b3.
  * The binary's mtime (05:46) matches the commit time (05:46:45).
  * It is byte-identical to `rexx-try11`, the variant the bisect names.
  * Its output on `local_novalue_hook`, `variable_reference_through_alias` and `compound_use_creates_stem` is identical to my HEAD build's.

## What I checked by running (all head = oracle unless said)

* **The NOVALUE hook.**
  * Positions: `length(zz)`, `zz~length`, IF and WHEN conditions, a message argument, an object answer (`ob~class` ->
    `The Array class`), `ar[1]`, a `DO` limit, `WHILE`, arithmetic (`num + 1` -> `42`), both operands of `||`
    (`p07c`).
  * A hook whose body reads another unset variable, so the hook is re-entered (`p01`).
  * A hook raising 42.3: untrapped, the traceback is identical (`p02`). Under `SIGNAL ON SYNTAX` the handler prints
    `caught SYNTAX 42 42.3 3` (`p17`).
  * A hook that REPLYs early (`a RZZ`, then `after reply ZZ`, `p18`), and a hook reached from a started message
    (`res got VUNSETVAR`, `p22`).
  * A string as the hook gives 97.1 `does not understand message "NOVALUE"` (`p09`).
  * TRACE I over hooked simple, compound and PARSE VAR reads (`p03`, stderr identical).
  * The settings around it:
    * `SIGNAL ON NOVALUE` with `::OPTIONS NOVALUE SYNTAX` traps without the syntax error, on base too (`p04a`).
    * Adding a hook to that sends the hook (`hook ZZ` / `a VZZ`, `p04b`), which follows condtra.xml:309-311 ("NOVALUE
      is not trapped").
    * `::OPTIONS NOVALUE ERROR` gives 98.986 and no hook (`p19`).
    * `SIGNAL ON ANY` (`p20`).
  * DROP selectors read through the hook (`p23`), PROCEDURE EXPOSE compound selectors, reachable and not (`p16`), an
    existing stem's unset tail (`p24`), and tail pieces (`p26`: `hook A.B`, then `cond A.C` after a `.nil` answer).
* **Compound use creating the stem.**
  * `PROCEDURE EXPOSE a.` then `drop a.1`, `drop (list)`, `drop e.` after a tail, a method's `expose s.` then
    `drop s.1`, INTERPRET `drop h.1`, `symbol('I.1')`, `.context~variables` (`p05`; the one difference is minor 1).
  * `procedure expose a. (a.1)` and `procedure expose (b.1)` (`p16`).
  * A stem reference to an unset object variable, and one from INTERPRET (`p21b`: `rp S.1 S.`, `m VAR`, `n VAR S.1`,
    `p ZZ.1 ZZ.`, `zz VAR`).
  * Drops through a stem alias (`p25`).
* **Alias references.**
  * A chain of three procedures: `c2 LOC`, `c3 LOC L`, `loc via c3`.
  * A stem alias passed on: `d1 Q.`, `d2 Q. stemdefault`.
  * A reference whose callee has returned: `e2 W`, `e1 w W` (`p06`).
  * The 88.930 text and the `>R>` line (`"LOC" => "S"`) through a second alias (`p10`).
  * An object-variable alias in a `::ROUTINE`: `(>z)~name` gives `OV` on both. `.context~variables` gives `p 1 0 1`
    against the oracle's `p 0 1 1`, the ruled candidate C (`p10`).
* **Memory.** I ran an alias-, reference- and hook-heavy loop (`mem/nodrop*.rex`: `call c1 >loc` two levels deep,
  `(>s)~name`, an object-variable reference, a hooked simple and compound read) under `memcap 2G`. Peak RSS at
  20000/80000 iterations: head 29836/30488 KB, base 31408/31096 KB, oracle 20612/20312 KB, so it is flat. Dropping
  unset tails alone (`mem/droponly*.rex`) grows on head 23124 -> 30664 KB and on the oracle 14940 -> 27764 KB. That
  growth is the tombstones the new stem keeps, as on the oracle, and not a leak.
* **Tests at HEAD.**
  * `cargo test --release -p rexx-exec --lib`: exit 0, 1191 passed.
  * `-p rexx-parse --test sourceline_oracle`: exit 0.
  * `-p rexx-exec --features pinning --test concurrency_tests a_park_under_each_frame_kind`: exit 0, 1 passed.
* **The `novalue_hooked` cell.** Every caller of `read_at`, `read_slot`, `unset_read` and `stem_get_at` calls
  `novalue_check` straight after the read, with no allocation between: `eval.rs:267/301`,
  `parse_template.rs:655/685`, `run/loops.rs:2662/2680`, `ir/drive.rs:775-780`, `run.rs:1754-1759`. So no stale hooked
  outcome can reach a later read. I found no probe that leaks one.

## Issues

### Critical

None.

### Important

1. **The hook's new paths for an unset tail of an existing stem, and a reachable PROCEDURE EXPOSE
   compound selector, are unwitnessed.** I built a mutant from a `git archive` of `42df2784e` with these changes:
   * `stem.rs:386`: `if HOOK` in `unset_tail` becomes `if false && HOOK`.
   * `run.rs:1754`: the reachable branch goes back to `self.stem_get(&compound.stem, &key)`.

   Every witness stays identical to the oracle on stdout, stderr and rc under it. Every compound read in
   `local_novalue_hook.rex` is of a stem never used before (`a.7`, `b.i`), so it takes `tail_of_unused_stem`, and its
   only PROCEDURE EXPOSE selector is simple (`(yyy)`). No unit test sets `.local~novalue` (a search of `crates/` and
   `corpus/` finds only the two corpus programs and `concurrency_tests.rs`). Two probes turn red under the mutant:
   * `p24`: `a.1 = 1` then `say 'tail' a.2`. The oracle and head print `tail VA.2`, the mutant `tail A.2`.
   * `p16`: `procedure expose c. (c.1)`. The oracle and head print `  hook C.1`, and the mutant prints no hook line.

   Fix: add both shapes to `local_novalue_hook.rex` (and regenerate its `sourceline_oracle` file).

### Minor

1. **EXPOSE and PROCEDURE EXPOSE of a stem still create nothing, while `create_stem_variable`'s doc states the rule
   for every use.** This predates the task: base answers the same. The doc at `stem.rs` reads "Every use of a stem
   variable or of a compound variable naming it creates the stem variable". In `p08`:
   * `p3: procedure expose c.` then `symbol('C.')`: oracle `p3 in VAR` / `after p3 VAR`, head `p3 in LIT` /
     `after p3 LIT`.
   * `procedure expose (lst) d.`: oracle `after p4 VAR`, head `LIT`.
   * A method's `expose s.` then `symbol('S.')`: oracle `m VAR`, head `m LIT`.

   The same shape appears in `p05`. Inside the procedure, head answers `p3 in LIT LIT` against the oracle's
   `VAR LIT`, and after the return both answer `p3 VAR`, because head's `symbol('C.1')` created the caller's stem.
   It is the item's rule (intro.xml:2562, "the first time you use the stem variable"), and the task did not reach it.
   Either narrow the doc sentence to the uses the crate implements, or take EXPOSE as queue candidate 1 below.
2. **Report prose.** Line 100 says "the five pins pass at base", which is a set size. Line 41's "Both shapes" is
   another.
3. **Candidate A cites only DROP's sentence for every instruction it covers.** rexxref states the rule for each one:
   EXPOSE at instrc.xml:1228 ("Variables are exposed in sequence from left to right"), and PROCEDURE EXPOSE at
   instrc.xml:2483 and, for subsidiary lists, :2515 ("The variables in a subsidiary list are also exposed from left
   to right"). The block is stronger with those cited in place of "EXPOSE and PROCEDURE EXPOSE take the same list".

## Concern 2 against candidate C

They are consistent with the rule. Each answers a different question:
* A reference denotes a variable. utilityclasses.xml's `name` is "the name of the variable referenced", and the
  variable referenced through an alias is the caller's `LOC` or the object's `V`, so `LOC`/`V` is the rule's answer.
  It is also the oracle's.
* `.context~variables` lists the names the running context can reach its variables by, and there the alias is
  reachable only as `Z`.

`p10` shows both at once in one routine: `(>z)~name` is `OV` on both engines, while the listing holds `Z` on head and
`OV` on the oracle.

## Concern 4: the extra `Compiled method "+"` traceback line

This is a crate gap, not an oracle bug. In `p11` (`t = >s; say t + 0` with `s = 'abc'`), the oracle prints
`       *-* Compiled method "+" with scope "String".` above `     3 *-* say t + 0`, and head and base leave it out. The
controls show the line is the oracle's normal answer whenever the operator reaches String's `+` as a method:
* In `p13`, `s~'+'(0)` prints the line on every engine.
* In `p14`, `t~'+'(0)` sent to the reference prints it on every engine too.
* In `p15`, a Rexx `UNKNOWN` that forwards the operator to `'abc'` prints it on every engine, above the FORWARD line.
* In `p12`, a plain `s + 0` prints no such line, on every engine.

Operators are messages, and a VariableReference forwards a message it does not know to the referenced value. So the
crate's operator path on a reference is missing the frame its explicit-message path already prints. This predates
the task (base is the same). Queue candidate 2.

## Queue candidates

1. EXPOSE / PROCEDURE EXPOSE of a stem creates the stem variable on the oracle and not here (`p08`, minor 1).
   Proposed owner: variables (fix round 1 or a queued item).
2. An operator on a VariableReference that raises inside the referenced value's method leaves out that method's
   `Compiled method` traceback line (`p11` against `p13`, `p14`, `p15`). Proposed owner: tracebacks.

## New oracle defects

None found by this review. `p04b` measured the oracle's NOVALUE SYNTAX setting under `SIGNAL ON NOVALUE`, and it
agrees with condtra.xml:309-311.
