# Task 8.5 re-review, fix round 1

Reviewed `69d8504eb..b431fd81d` (code `aeb913cff`, `074ada1da`). HEAD was built from the checkout (only
`progress.md` modified) into `/tmp/claude-1000/t85-rr-target-head`. BASE `6e0bf5e5b` was built from a
`git archive` (`rust/`, `interpreter/`) into `/tmp/claude-1000/t85-rr-target-base`. Both used
`CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 --bin rexx` and exited 0. Every probe ran three ways
(oracle under the standard wrapper, base and head under `memcap 2G timeout 20`), each engine from its own fresh
`mktemp -d` run directory under `t85-rereview-scratch/`. stdout, stderr and rc were compared separately, with
the run directory's name normalised in stderr. Probe paths below are relative to `t85-rereview-scratch/`, and
the deciding lines are quoted.

## Verdict

**CHANGES REQUESTED.** Critical none, Important 2, Minor 1. Queue candidates 1, new oracle defects 1.

## Verdicts on the fix brief

1. **Important 1 (the hook's unwitnessed paths): ADDRESSED.**
   * `wit/local_novalue_hook/` matches the oracle on stdout, stderr and rc at head, and differs at base on
     stdout and stderr.
   * I built both mutants myself from a `git archive` of `b431fd81d`, each in its own target dir. Mutant A is
     `stem.rs:386` `if HOOK` made `if false && HOOK`. Mutant B is `run.rs:1777` `stem_get_at(&compound.stem,
     None, &key)` made `stem_get(&compound.stem, &key)`.
   * Against the oracle, mutant A loses `  hook A8.2` and prints `s3a A8.2` for `s3a NV_A8.2`. Mutant B loses
     `  hook C5.1`. On both mutants stderr and rc are the same as the oracle's. This is the report's
     `r1/mutants/*.txt` line for line.
2. **Minor 1 (EXPOSE and PROCEDURE EXPOSE create the stem): ADDRESSED.**
   * `wit/compound_use_creates_stem/`: at head it matches the oracle on all descriptors. At base it prints
     `p3 in LIT`, `after p3 LIT`, `after p4 LIT` and `x LIT`, where the oracle prints `VAR` for each.
   * `p09/ex.rex` matches the oracle at head on all descriptors. It covers:
     * a method's `expose s.`, read by a second method (`a VAR 1`, `b VAR x`), then `drop s.` (`c VAR LIT`);
     * a method's `expose t. u` (`e VAR LIT`);
     * `procedure expose c.` with `.context~variables~hasIndex` (`p1 in VAR 1`);
     * a two-level `procedure expose e.` (`p2 after p2b VAR deep`);
     * a selector list `(lst)` holding `g. h.1` (`p3 in VAR VAR LIT`);
     * `procedure expose j. j.` then `drop j.` (`p5 dropped VAR`);
     * the caller's `.context~variables` (`vars  C. E. G. H. J.`).

     Base differs on `a LIT 0`, `e LIT LIT`, `p1 in LIT 0`, `p3 in LIT VAR LIT` and on `G.`, which is missing
     from `vars`.
   * `p11/alias2.rex` covers a method's `expose s.` followed by an internal `procedure expose s.` (`ip VAR`,
     `m deep VAR`, `n deep`), where head matches the oracle and base prints `ip LIT`.
   * `p13/bare.rex` covers VALUE, DROP, a read, PARSE VAR, a CALL argument, `>f.`, INTERPRET DROP and SYMBOL
     of a bare stem. Head matches the oracle on every line (`h LIT` for SYMBOL alone). So the
     `create_stem_variable` doc (`stem.rs:235-241`) stays true.
3. **Minor 2 (set sizes): ADDRESSED.** Report lines 41 and 100 now read "Each of the item's shapes" and "the pins
   pass at base". `grep -n "five\|Both shapes"` over the report finds nothing.
4. **Minor 3 (candidate A's citations): ADDRESSED.** Block A cites instrc.xml:1075, :1228, :2483 and :2515, and
   each line holds the quoted sentence (`sed -n`).
5. **The variable reference string value: PARTIAL.** I found no position where the code departs from the rule.
   Every position the ruling names gives the referenced value's `request('STRING')` (`p01/pos.rex`,
   `p03/ns2.rex`). Here is what I checked:
   * The item file's first line is `**CLOSED 2026-10-27 by Phase 9d Task 8.5 (074ada1da).**`.
   * Its `queued-inventory.tsv` row reads `closed:074ada1da` in columns 4 and 6, and the reason column is kept.
   * The new unit tests pass at head (`cargo test --release -p rexx-exec --lib`: exit 0, `1193 passed`), and
     `r1/witnesses/unit-tests-base-6e0bf5e5b.txt` shows both FAILED at base.
   * Every probe that block D cites exists. The `r1/probes/` ones reproduce byte for byte on all three
     engines. The `t8.5/probes/` ones reproduce on the oracle, and their recorded `.t` outputs equal my base
     run (they were recorded before the fix).
   * The C++ and rexxref citations land on the lines they name: VariableReference.cpp:258-261 and :364-370,
     provide.xml:747, :750, :758 and :769, fundclasses.xml:2683, utilityclasses.xml:12631-12632.

   The witnesses and block D still fall short of what the change does (Important 1 and 2).

## What I checked by running (head unless said)

* **Positions the ruling names** (`p01/pos.rex`, `p03/ns2.rex`), with a reference to each of these values: an
  array (`q`), an object with `makeString` (`k`, `kmake`), a numeric `makeString` (`m`, `5`), an object with
  only `string` (`n`), a Directory (`d`), a string (`s`), a stem with a default (`s.`), an unset variable (`u`),
  and a reference to a reference (`rr`).
  * Positions: either operand of `||`, abuttal and blank, SAY, LENGTH, POS, SUBSTR, D2X, PARSE VALUE and PARSE
    VAR.
  * Each gives the referenced value's string value: `q lcat: p` / `q|`, `k rabut: |kmake`, `m d2x: 5`,
    `s. len: 4`, `u say: U`, `k rr-rcat: |kmake`.
  * `n` and `d` fall back to the reference's own `string`.
  * Under `SIGNAL ON NOSTRING`, NOSTRING is raised only for `n` and `d`, with the reference as the condition's
    object (`n say:   NOSTRING a NSTR VariableReference`). The oracle raises for every reference
    (`s say:   NOSTRING plain VariableReference`).
* **Unchanged outside the ruling** (head = base = oracle):
  * `~string` (`an Array / a KMAKE / a NSTR`) and `~makestring` (`kmake`) sent to the reference;
  * a left operand of `=`/`==` (`cmpL: 0 0`; `(>s) = 'plain'` is `1`);
  * a left operand of `+` (`97.1 Object "a KNUM" does not understand message "+"`);
  * the TRACE rendering `>V>   R => "a KMAKE"` (`p07/tr.rex`).
* **The left-operand route** (`repro85/varref_concat_matrix`): `R2` to `R9` and `S1`/`S2` follow the rule. Base's
  forwarded `R4 nstrx` is now `R4 a NOSTRx`, the reference's `string`, as the report says.
* **Perf.** `r1/final/callgrind.txt` measures `074ada1da` code:
  * `rexx-head-074ada1da` has mtime 09:59:24, the commit time of `074ada1da`.
  * It holds `reference_conversion` (`nm -C`, count 1), and `rexx-head-aeb913cff` lacks it (count 0). That
    function first appears in `074ada1da`.
  * It is byte-identical to `rexx-r1try1`.

  With libc subtracted, the largest rise is fibfunc `+0.0213`, then fibcall `+0.0209`. Both are under +0.05%,
  and the table has no SPREAD or OVER flag.

## Issues

### Critical

None.

### Important

1. **The conversion change reaches positions that the report does not name and no witness covers.**
   `reference_conversion` sits in the shared classifier (`dispatch/reqstr.rs:428`). So every required-string
   position changes, not only the ones the report lists ("`SAY`, a builtin argument and a concatenation's right
   operand"). Each change I measured follows the rule's "every position". None is witnessed, and the report's
   "No corpus program and no suite row changed" is its only statement about them. Each line below is head
   against base, with the oracle after it:
   * Right operand of a comparison with a string receiver (provide.xml lists "Rexx dyadic operators when the
     receiving object ... is a string"). `p04/untrapped.rex`: `k1 1 1 1` against base `k1 0 1 0`, oracle
     `0 1 0`. `q1 1` against `0`. `k3 1` against `0`.
   * Right operand of arithmetic. `p03/ns2.rex`: `m addR: 6` against base `41.1 Nonnumeric value ("a KNUM")`,
     oracle `6`. Under `SIGNAL ON NOSTRING`, `n addR:   NOSTRING a NSTR VariableReference`: base raised nothing
     (`41.1`), and the oracle raises `NOSTRING nstr NSTR`.
   * String method arguments. `p05/meth.rex`: `'xkmakex'~pos(>k) -> 2`, `changestr(>k, '-') -> x-x`,
     `'ab'~append(>k) -> abkmake`, `compareTo(>k) -> 0`, `startsWith(>k) -> 1`, `match(1, >k) -> 1`,
     `'kmake'~'=='(>k) -> 1`. Base answers `0`, `xkmakex`, `aba KMAKE`, `1`, `0`, `0`, `0`. The oracle raises
     `88.909` for `pos`, `changestr`, `compareTo`, `startsWith` and `match`, and gives `aba KMAKE` and `0`.
   * Instructions. `p06/instr.rex`:
     * `do (>m)` runs 5 times (`ddddd`), where base and the oracle raise `26.2 ... found "a KNUM"`.
     * `interpret (>c)` prints `interp`, and the oracle and base run `a KCODE`.
     * `address command (>c)` sends `"say 'interp'"`.
     * `push (>k)` gives `push KMAKE`.
     * The tail `x.t` gives `tail X.kmake`, against the oracle's `X.a KMAKE`.
     * `signal value (>k)` gives `Label "kmake" not found`.
     * `numeric digits (>m)` gives `digits 5`. The oracle agrees, and base raised `26.5 ... found "a KNUM"`.
     * `trace value (>m)` (`p07/tr.rex`) gives `trace N`, where the oracle and base give `trace A`.
   * An explicit concatenation message to the reference. `repro85/varref_concat_matrix`: `M1 x` against base and
     oracle `M1 an Arrayx`.

   Fix: witness each of these positions red at `6e0bf5e5b` (a corpus program or the existing unit tests), and
   list them in the report and in block D's measurement. Moritz rules the Deviation on the positions it names.

2. **Block D's oracle-bug sentence is false for some positions, and D leaves the NOSTRING object open.** D says
   "the implicit conversion takes the first" (`stringValue`, VariableReference.cpp:258-261). The oracle's own
   arithmetic and NUMERIC conversions reach the referenced value's `makeString`:
   * `p03/ns2.rex` gives oracle `m addR: 6` for `1 + r`, where `r = >m` and `m~makestring` is `'5'`;
   * `p06/instr.rex` gives oracle `digits 5` for `numeric digits (>m)`.

   In those positions the oracle also raises NOSTRING on the **referenced value**, with that value's `STRING`
   answer as the description: `n addR:   NOSTRING nstr NSTR` and `d addR:   NOSTRING a Directory Directory`.
   The same holds for a comparison's right operand: `n cmpR:   NOSTRING nstr NSTR`. Head raises with the
   reference and the reference's `string` in every position: `n addR:   NOSTRING a NSTR VariableReference`.
   The unit test `a_reference_raises_nostring_only_where_its_value_has_no_string_value` fixes that choice, but
   neither D nor the report states it.

   Fix: scope D's cause sentence to the positions where the oracle takes `stringValue` (SAY, builtin arguments,
   concatenation, comparison right operand where the value has a string value, INTERPRET, DO, PUSH, tails,
   SIGNAL and TRACE VALUE). Name the positions where the oracle already forwards (arithmetic, NUMERIC DIGITS).
   Then state which object and description the NOSTRING condition carries, with the oracle's
   referenced-value answer as the measured alternative.

### Minor

1. `rust/corpus/lang/local_novalue_hook.rex:5` was extended to 134 columns instead of being re-wrapped to the
   block's width (`awk 'length > 100'` over both changed corpus programs finds only this line).

## Out of scope

* **Queue candidate 1.** `String~equals` and `~caselessEquals` with a non-string argument that has `makeString`
  answer `0` here and `1` on the oracle, at base as well as head. `p05/meth.rex`: `'kmake'~equals(k) -> 1`
  (oracle) against `0`, and `'KMAKE'~caselessEquals(k) -> 1` against `0`. The task does not touch it. A
  reference argument follows it (`equals(>k) -> 0` while `'kmake' == (>k)` is `1` at head). Proposed owner:
  string methods.

## New oracle defects

1. **PROCEDURE EXPOSE of a variable alias leaves SYMBOL and VALUE answering unset, while the expression reads
   the value.** `p12/symalias.rex`:

       a. = 'x'; v = 1
       call r2 >a., >v
       exit
       r2: use arg >w., >u
         say 'r2' symbol('W.') symbol('U') w.1 u
         call r3
         return
       r3: procedure expose w. u
         say 'r3' symbol('W.') symbol('U') w.1 u value('U') .context~variables~hasIndex('W.') .context~variables~hasIndex('U')
         return

   Oracle, rc 0: `r2 VAR VAR x 1` / `r3 LIT LIT x 1 U 0 0`. In `r3`, `u` reads `1` while `value('U')` is `U`
   and `symbol('U')` is `LIT`. Head and base print `r3 VAR VAR x 1 1 1 1`. It predates the task, and the crate
   follows the rule. I did not trace the C++ cause. It is not in `defects.json`: a search of its entries for
   EXPOSE together with an alias finds only `empty-stem-name-compared-by-identity`. It neighbours
   `variable-reference-alias-listed-under-owner-name`, which covers the listing under the owner's name, but
   this one hides the variable from SYMBOL, VALUE and the listing alike.
