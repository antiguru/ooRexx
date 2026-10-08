# Task 4a review: one truth judgment

Reviewer: t4a review seat. Range `24394ca34..7219f1377` (HEAD at review `a672759fa`; nothing under
`rust/` changed after `08876442c`). Probes: `S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t4arev`,
`$S/cmp.sh BIN FILE...` runs the oracle (global-constraints wrapper) and the crate, each from its own
fresh empty directory under `$S/run/`, and `cmp`s stdout, stderr and status separately. Crate binary
for behaviour probes: `/tmp/claude-1000/p61/t4a/target-head2/release/rexx-run` (built from
`d27a9d441`; `git diff --stat d27a9d441 HEAD -- rust` touches only `refusal-sites.tsv`). Debug
builds and mutants: a `git archive HEAD rust interpreter` copy at `$S/tree`, target dir
`$S/target` (deleted after), every mutated file restored and `cmp`-checked against `git show HEAD:`.

### Spec Compliance

- ✅ Spec compliant. `Interp::truth` (`eval.rs:1460`) is the one judgment; `logical_value` is
  module-private (`eval.rs:1424`); every site the brief lists routes through `truth` or its fast half
  `truth_without_conversion` (`eval.rs:673,1160-1161,1347`, `run.rs:3684,3709`,
  `run/loops.rs:963,2756,2824`, `dispatch/collection.rs:216`, `dispatch/string.rs:1254`,
  `security.rs:122`, `dispatch/library.rs:1499`, `ir/drive.rs:1477,3915`); `loop_truth` and
  `condition_holds` are deleted; `same_item` raises; `test_case_when` sends `==` with the case value
  as receiver, as `WhenCaseInstruction.cpp:157` does; Deviation 25 drops the WHILE/STRING sentence.
- Deviation from the letter of Step 3 (accepted, see Minor 3): no `debug_assert_eq!` at
  `drive.rs:1477` or `register_holds`; both call the fast half of the one function instead, and
  `eval/tests.rs:446` checks that half against the slow half.
- ⚠️ Rooting sensitivity unproven (focus 6): see the rooting check below; recommend the controller not
  read my green stress run as evidence the pushes are load-bearing.

### Checks run

1. **Completeness (focus 1).** Oracle side enumerated by
   `grep -rn -E 'truthValue\(|logicalValue\(' --include=*.cpp --include=*.hpp interpreter/`:
   IF/WHEN, WHILE, UNTIL, GUARD, WHEN CASE, logical list, Supplier `loopAvailable`, `isEqual`
   (Object and String), Integer/String `&|&&\?` methods, security manager, native `logical_t`
   argument and `ObjectToLogical`. Crate side:
   `grep -rn -E 'logical_value\(|is_true_object|LOGICAL_TRUE|LOGICAL_FALSE|unwrap_or\(false\)|== b"1"|== b"0"|b"1" =>|b"0" =>|truth\(|truth_without_conversion' rust/crates/rexx-exec/src`,
   `grep -rn -E 'apply_binary\(|StrictEqual|send_operator\(' ...`, `grep -rn same_item ...`,
   `grep -rni '"available"' ...`. Every oracle site has a crate counterpart routed through `truth`;
   hash lookups (`dispatch/hash.rs:585`, `relation.rs:95`), stem/array/list/hash `hasItem`/`index`/
   `removeItem` all go through `same_item`; GUARD WHEN goes through `eval_condition`
   (`run.rs:2093`); IR IF without a native shape goes `EvalExpr` -> `eval_if_condition`
   (`run.rs:3541`); IR WHILE/UNTIL go through `eval_condition` (`run/loops.rs:1449,1570,1995,2162`).
   The three sites left alone (`time_support.rs:250`, `builtin/stream.rs:155`, `semaphores.rs:272`)
   are not truth judgments. No bypassing site found. Probes, all SAME on all three descriptors:
   `g_guard` (GUARD WHEN on `.s1~new`, `.array~of(1)`, `'banana'` -> 34.902), `g_interp` (IF,
   WHILE, SELECT CASE, `&` inside INTERPRET), `g_qmark` (`~"?"`), `g_strops` (`'1'~"&"(o)` and
   friends), `n_native` and `n_trace` (native-shaped `if o = 1`, `when o > 1`, `while o < 1`,
   `until o == 1` with user operators answering `.s1~new`, `.array~of(1)`, `.s0~new`, heap `'1'`,
   `0+1`, `'banana'`; plain and `trace r`).
2. **User STRING (focus 2).** SAME: `s_raise` (STRING raises 40.1, rc 216), `s_raise2` (STRING
   divides by zero under SIGNAL ON SYNTAX), `s_nil` (answers `.nil`), `s_noret` (answers nothing),
   `s_numstr` (answers `0+1`), `s_trace` (`trace r`), `s_count`/`s_count2` (STRING send count per
   judgment, printed after each of IF, WHEN, WHILE, UNTIL, list, `&`, DO WITH, `hasItem`, CASE:
   identical counts to the oracle, one send per judgment). `s_rec` (STRING recursing through IF): rc
   245 on both, traceback length differs, the licensed `MAX_EVAL_DEPTH` divergence. DIFF: `s_obj`
   and `s_obj2`, Minor 1.
3. **SELECT CASE (focus 3).** SAME: `c_num` (`'1'` vs `'1.0'`, `' 1'`, `1`; `1.0` vs `1`;
   `0+1`; under `trace r`), `c_whenobj` (string CASE, user-object WHEN whose STRING answers the
   case text: `'x' == w` matches and W's own `==` is never sent, as in the oracle; trace lines
   `"a W"` then `"1"`), `c_casenil` (CASE `.nil`; `==` answering `.nil` -> 34.905 found "The NIL
   object"), `c_noans` (`==` answering nothing -> 91.999), `c_arr`, `c_ir` (CASE in a loop, IR
   path, send order), `c_traceall` (`trace i`). Not testable: a String subclass WHEN or CASE value
   (`c_strsubcase`, `s_strsub`): `.mystr~new` refuses "method NEW of class MYSTR is not implemented
   (Phase 9)", rc 120, at the base too.
4. **Collections (focus 4).** 120 programs (`$S/probes/k_*.rex`, results `$S/k_results.txt`):
   `hasItem`/`index`/`removeItem`/`hasIndex`/`[]` over Array, List, Queue, CircularQueue, Table
   (item and key), Set, Bag, Relation (item and key), Directory, Stem, IdentityTable, each with an
   `==` answering `'banana'`, `.nil`, `'1'||''`, `1+0`, `.s1~new`, `.array~of(1)`: 120 of 120
   SAME, stdout, stderr (including the traceback line) and status. `o_orient`: receiver orientation
   (wanted object is the receiver of `==`; a base-class string wanted sends nothing) SAME on all
   fifteen shapes.
5. **Fast-path checks (focus 5).** Mutants in `$S/tree`, debug build:
   - `truth_without_conversion` with `SmallInt(0) => Some(true)`:
     `cargo test -p rexx-exec --lib the_unconverted_truth` red, `eval/tests.rs:465`
     "assertion `left == right` failed: ObjRef(1, ...)".
   - `eval_logical_list` answering `self.text(b"yes")` when it holds:
     `cargo test -p rexx-exec --test truth` red with the panic
     "a checked condition answered something other than a logical constant" at `run.rs:3680`, the
     new `debug_assert!` executing in the debug run.
   - `drive.rs:1477` rewritten as a local fast path answering `LOGICAL_FALSE` true: no assert
     exists there to fire; the table goes red on `'0'` only through the `until` context's auxiliary
     `if n > 1` (Minor 3).
6. **Rooting (focus 6).** Scratch test `$S/tree/rust/crates/rexx-exec/tests/t4arev.rs` runs
   `rootprobes/r1.rex` (heap-allocating STRING, `==`, AVAILABLE; long non-logical answers read back
   from `condition('o')~additional`), `r2.rex` (DO TO/BY with heap answers, GUARD WHEN), `r3.rex`
   (fresh unvariabled values in IF/WHILE/WHEN/hasItem), the six new corpus witnesses, `s_count2`,
   `c_whenobj`, `o_orient` with `run_program` and with `run_program_collect_every_alloc` both plain
   and `SwitchMode::EveryOpportunity`, asserting equal stdout/stderr/rc and `collections > 0`: pass
   (`$S/t4arev.log`; e.g. r1 354 and 634 collections). r1 and r2 were first confirmed against the
   oracle (r1 SAME) or read (r2, Deviation 25 territory) to print every path. Controls: removing the
   push in `truth_of_string_value` (R1), then also `condition_value`'s (R4), stays green; so does a
   positive control that unroots `required_string_dispatch`'s answer (`reqstr.rs:139`) and forces an
   allocation before `truth_of_string_value` reads it. The values are rooted redundantly by their
   producers, so no defect is visible, and this harness has not been shown able to see one on these
   paths.
7. **Records (focus 7).** Deviation 25 read in full at HEAD: Minor 6. Gate record: callgrind,
   wallclock, mutation and per-task commands quoted; the Step 1 cell counts are not (Minor 2). The
   counts themselves check: 14 non-DO contexts x 16 values = 224 identical; 32 `doto`/`by` cells, 6
   agreeing (`q0`, `a0`, `s0` in each), 26 differing.

### Strengths

- One function, with the fast half shared rather than re-written: `truth` is
  `truth_without_conversion` then `truth_of_string_value` (`eval.rs:1437-1490`), and both IR fast
  paths call the same half, so a separate fast path cannot drift. The unit test at
  `eval/tests.rs:446` is exhaustive over the four handles the fast half answers for, and asserts
  that count.
- The datadriven invariant (`tests/truth.rs:2035`) checks agreement across sixteen contexts and one
  sub-number per context; both recorded mutants and two of mine turn it red.
- `test_case_when` (`run.rs:3693-3714`) matches `RexxInstructionCaseWhen::execute` line for line:
  receiver, trace of value then answer, 34.905.
- The DO WITH change fixes a second bug in passing (`string_value_text` rendered an Array as
  "an Array" and sent no STRING), witnessed by `do_with_available_string.rex`.
- Every behaviour probe I ran outside Minor 1 agrees with the oracle byte for byte.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

None.

#### Minor (Nice to Have)

1. **A STRING method answering a non-string is not converted as `requestString` converts it.**
   `$S/probes/s_obj.rex` (`::method string; return .array~of(1)`, `if .r~new`): oracle prints
   `T`, rc 0; crate raises 34.1 found "an Array", rc 222. `s_obj2.rex` (STRING answers an object
   whose own STRING answers 1): both rc 222 34.1, oracle found "The NIL object", crate "a Q". The
   oracle applies `primitiveMakeString` to the STRING answer (`ObjectClass.cpp:1285`); the crate's
   `required_string_value` (`dispatch/reqstr.rs:60`) does not. Pre-existing and not truth-specific:
   `$S/probes/rs_len.rex` `length(.r~new)` with STRING answering `.array~of(1,2)` prints oracle `3`,
   crate `8`. It now reaches every truth judgment through R9's "by requestString". Queue against
   `reqstr.rs`, not this task.
2. **The Step 1 extent figures cite uncommitted scripts.** `phase-6-1-gate.md:311,343` and
   `task-4a-report.md:12-13` derive "224 cells" and "26 cells" with
   `/tmp/claude-1000/p61/t4a/gen/{gen.py,run.sh,table.py}`, which are not in the repository; the
   report's "every truth decision in `rexx-exec/src` goes through `truth`"
   (`task-4a-report.md:53`) quotes no command. The global constraint says extent claims are derived
   with the command committed. Commit the generator and the grep, or quote them in the record.
3. **No refinement assert at `drive.rs:1477` or `register_holds` (`drive.rs:3912`), and the table's
   IF row never reaches the IF quick path.** Calling `truth_without_conversion` makes the brief's
   `debug_assert_eq!` redundant there, which I accept. But my third mutant shows the datadriven
   table reaches `Op::ConditionJump`'s quick path only through the `until` program's own
   `if n > 1`: `if v` with `v` a variable is not native-shaped, so the `if`, `when` and `while`
   rows exercise `condition_value` only. A future edit that inlines a quick path again is caught by
   accident. A row whose condition is native-shaped (`if v = 1` over a user `=`) would witness it
   directly.
4. **`current_case` is never cleared when a `SELECT CASE` ends** (`run/select.rs:53`, set only by
   `Op::SelectCaseText`, `drive.rs:3578`). As text it pinned nothing; as `Option<ObjRef>`, rooted
   at `activation.rs:994`, it keeps the CASE object alive until the activation's next `SELECT` or
   its end. GC timing is a licensed divergence, so this changes no gated answer.
5. **`test_case_when` copies two texts per WHEN value whether or not tracing is on**
   (`run.rs:3703,3707`: `to_text(value).to_vec()` and `to_text(answer).to_vec()`). Before this task
   it copied one. Report concern 3 covers the send; the copies are separate, and
   `condition_value` already shows the trace-gated pattern (`run.rs:3655`).
6. **Deviation 25's "Text exactly `1` ends the loop"** (`phase-4-exclusions.txt:1798`) now means
   the answer's string value (a user STRING, an array's joined items), which the paragraph below it
   says ("read as this crate's WHILE and UNTIL read theirs"). "Text" reads as the rendering, which
   for `.array~of(1)` is "an Array". Imprecise rather than false. "String value exactly `1`" fixes
   it.

### Assessment

**Task quality:** Approved

**Reasoning:** Every truth judgment I could find on either side goes through one function, and
every probe agrees with the oracle byte for byte except a pre-existing `requestString` gap in
`reqstr.rs` (Minor 1). That covers 120 collection shapes, SELECT CASE under trace, GUARD,
INTERPRET, native-shaped conditions and STRING send counts. The refinement checks fire under
mutation. The remaining findings are record hygiene and witness reach.

Counts: Critical 0, Important 0, Minor 6.
