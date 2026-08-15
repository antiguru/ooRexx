STATUS: DONE

# Claim-by-claim audit of coordinator prose

Scope: commits a6abb0d7, 56e60d96, ef683ab2, 34f2a4d6, 9495c974, 3a405228, c67dd343, 63a4092e, 521195d1, e0e57825, 6ff4e93a, 610642a3; phase-4-exclusions.txt rows; design-spec 342-line annotation.
Method: never verify a citation by reading the named line; grep for the symbol and check containment. Re-derive numbers. Re-measure oracle claims.

## Findings

### F1: 34f2a4d6 overstates the "measurement gap" — the measurements were not blind to `<<=`

The commit body says "It began as a measurement gap: the fourteen oracle measurements behind that test have the same blind spot", and the comment added to `backslash_negated_and_synonym_forms` (rust/crates/rexx-exec/src/eval.rs) says the fourteen measurements have "no equal-operand case for any negated form".
Re-derived against task-8-report.md's fourteen-line block: it contains `'9' <<= '9' -> 1` and `'9' >>= '9' -> 1`, both equal-operand, and `'9' <<= '9'` distinguishes StrictLessEqual from StrictLess (`'9' << '9'` is 0), so the `<<=`-to-StrictLess mutation IS visible to the measurement set.
The test at the parent carried only `'10' <<= '9'`, dropping the measured equal-operand row, so for `<<=` this was a transcription gap between report and test, not a measurement gap.
Additionally `'9' \== '9' -> 0` is an equal-operand case of a negated form, so "no equal-operand case for any negated form" is false as written; it holds only for the four order-negated forms `\>` `\<` `\>>` `\<<` and for non-strict `>=`/`<=` (absent entirely).
Consequence: low-moderate. The seven added rows themselves are correct (re-measured, all 1), but the provenance story in both the commit body and the in-tree comment misattributes a copying loss to the measurement session.

### F2: e0e57825's D17 annotation says a case-insensitive scan "gives 37 blocks", but the scan it names gives 40

docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md, the recount note: "A case-insensitive scan therefore gives 37 blocks and 374 lines."
Re-derived against ootest/ooRexx/base/keyword/TRACE.testGroup: `grep -icE '^[[:space:]]*::resource'` gives **40** directives (34 lowercase + 6 uppercase), and counting all lines inside case-insensitively matched blocks gives **437**.
37 and 374 are reachable only by additionally excluding the three uppercase `.rex` program blocks (40-3=37; 437-63=374), a filtering step the sentence attributes to the scan itself.
The note exists precisely to stop the next recounter from concluding the spec is wrong; a recounter who runs the named case-insensitive scan gets 40/437, matches neither 342 nor 37/374, and is back where the note tried to prevent.
The underlying figures all reproduce: 34 blocks, 342 lines, 128 `*-*`, 214 other (lowercase anchored scan), and the three uppercase `*_expected` blocks carry exactly 32 lines, all prefixed `>I>` (16) or `<I<` (16). 342+32=374 and 34+3=37 are correct as "expected-output blocks/lines", not as what a case-insensitive scan gives.
Consequence: moderate. This is one of the two documents the coordinator flags as about to be built from (the recount instruction), and its stated scan does not produce its stated numbers.

### F3: a6abb0d7 anchors the enumeration's falsification to the wrong task, in the commit body and twice in lib.rs

The commit body says the Loud enumeration "was true of the spike and false by Task 6", and the same anchor is in the tree at rust/crates/rexx-exec/src/lib.rs:24 (module doc) and :388 (`Loud::expression`'s doc: "true of Task 3's spike and false by Task 6").
Re-derived: Task 6 is the resolution plan (ca005497). At that commit, `eval_node` still evaluated exactly Literal, Constant, Variable and `Binary { op: Concatenate }` — the enumerated list — and assignment still took only `ExprKind::Variable` targets, so both Loud messages were still true after Task 6.
The expression list first became false at Task 7 (3a9d6446, arithmetic and prefix operators); the instruction list ("executes SAY, EXIT, assignment...") first became false at Task 9 (43e18462, the seven non-branching instructions).
"None of the three tasks that shipped since did [edit the string]" counts three only under the wrong anchor; under the correct one, Task 7 falsified it and Tasks 8 and 9 shipped past it.
Consequence: low for implementation (the enumeration is deleted either way, which is the right fix), but the stale-by-example story is the stated justification for the deletion pattern, and it names the wrong example in two committed doc comments.

### F4: 56e60d96's verification list names "42.11", a condition that does not exist

The commit body: "Verified byte for byte against build/bin/rexx on eleven programs ... 41.1, 42.3 from three different divisions, 42.11, a continued clause ...".
Error 42's subcodes in interpreter/messages/rexxmsg.xml are exactly 1, 2, 3, 900, 901, 902 and 903; there is no 42.11, so no program can have produced that report and the eleven-program set as listed cannot be replicated.
Likely a typo for 42.1 or a mis-copied 41.1 (already listed separately).
Everything else in that sentence reproduces: `2 & 1` gives 34.901, `'x' + 1` gives 41.1, `/` `%` `//` by zero all give 42.3 (exit 214), the continued clause echoes joined with the comma retained, and line 12 renders as a six-wide `    12 *-*` field.
Consequence: low (commit message only), but it is a claimed oracle measurement that cannot be re-measured.

## Checked and reproduced

Oracle measurements re-run 2026-07-31 against build/bin/rexx, all matching the prose:

* Indentation rule (521195d1, Task 11 body): clause echo gains 2 spaces per enclosing DO, measured at 1/2/3 levels (`*-*` + 3, 5, 7 spaces vs base 1).
* IF's THEN counts as two levels: failing THEN body echoes with +4 spaces.
* `do i = 1 to 'x'` echo gets no indentation (error 41.1 at the DO clause itself).
* `exit 'x'` does not raise: exit code 0, empty stdout/stderr.
* `if 'x', 1 then` raises 34.6 (not 34.1); `if 'x' then` raises 34.1; `when 'x'` raises 34.2.
* `select case 2` with `when 1, 2` prints `hit`; plain `when 1, 2` raises 34.6 (element "2").
* Error 7.3 echoes the END clause, not the SELECT.
* `say 2 & 1` raises 34.901; exit code 222 = 256-34. 7.3 gave exit 249 = 256-7.
* Seven comparison rows (34f2a4d6): `'9' \> '9'`, `\<`, `\>>`, `\<<`, `<<=`, `>=`, `<=` all print 1 on the oracle.
* `'01' == '1'` is 0 where `'01' = '1'` is 1.
* LEAVE/ITERATE (521195d1, Task 11 body): ordinary clause label on a DO is 28.3 (leave) / 28.4 (iterate); `do label` works; `leave i`/`iterate i` reach the outer loop and unwind the inner; labelled simple block leavable, bare block 28.1.
* `do until 'x'; end` raises 34.4 and echoes `end`; `do while 'x'` raises 34.3 and echoes the `do` line.
* DO OVER on a string and a number each iterate once yielding themselves.
* `select case '007'` does not match `when 7`.
* True-condition absorbed WHEN (`when 1 = 1 then` + `when 2 = 2 then nop`) is accepted, rc 0.
* DROP (v) (63a4092e): newline gives 20.928 with the raw newline inside the substitution (`a\nb`); space and tab separate (both names dropped); CR, FF, VT behave as the newline does. Exit 236 = 256-20.
* Oracle prints the absolute dot-normalised path: `./sub/../sub/rel.rex` and bare `rel.rex` run from its own directory give the identical absolute string.
* INTERPRET failure gives two clause echoes, innermost first, both on the enclosing line.
* Line field is six wide, right-aligned: line 12 echoes as `    12 *-*`.
* Continued clause echoes joined on one line (`x = 1 + ,2 + (1/0)`), newline removed, comma retained.

More reproduced, second batch:

* Comma list under WHILE and UNTIL also raises 34.6 (completes "measured across all four keywords").
* Short-circuit (34f2a4d6): `if 0, (1/0) then nop` and `if 1, 0, (1/0) then nop` both print `reached`, exit 0; `if 1, (1/0)` exits 214 with 42.3.
* `if 'x' then nop` echoes `if 'x' ` with the trailing space (error.rs ClauseSite doc claim).
* 256-major across all nine listed majors: 7→249, 24→232 (24.901), 25→231 (25.11), 26→230 (26.5), 33→223 (33.1), 34→222, 41→215 (41.1), 42→214, 98→158 (98.913, `do x over .nil`).
* 11.1 is in the catalogue source (rexxmsg.xml) as "Insufficient control stack space; cannot continue execution."
* DROP oracle behavior byte-matches the test added in 63a4092e: 20.928 with additional `a\nb`.

Code-side numbers re-derived, all matching:

* form_name (a6abb0d7): 15 arms, no `_` arm, exactly two call `format!` (Prefix, Binary), 13 return statics; unit-test expected strings match the arms.
* Witness history: the loud-size test was added in 2838974d on `+`, moved to `=` in 3a9d6446 (Task 7), moved to `~` in a6abb0d7 — "began on `+`, moved to `=` when Task 7 landed" reproduces from `git log -S`.
* compare_op has exactly 18 comparison rows; compare_decoded returns on `op.is_strict()` before touching either Number (rexx-num/src/compare.rs:154-161).
* "59 tests" (34f2a4d6) = the 59 `#[test]` fns in rexx-exec/src at that commit's parent (8+28+1+6+8+8); "79 tests" (63a4092e) = 79 at its parent (run.rs adds 20); "102 pre-existing" = 104 src tests at a9997e55 minus the 2 attribution tests the mutation targets. All three counts consistent as lib-suite counts.
* Fourteen oracle measurements: task-8-report.md's block has exactly 14 rows (but see F1).
* Six eval.rs functions opening a temps frame then using `?` (9495c974): exactly `eval_prefix`, `eval_arithmetic`, `concat`, `eval_compare`, `eval_logical`, `eval_logical_list` (push_frame at 241, 295, 360, 401, 456, 513).
* pop_frame truncates (`self.temps.truncate(frame.0)`); slot side of roots.rs asserts (two assert_eq) and temps side does not; a deeper-than-top handle is a Vec::truncate no-op (3a405228's two shapes are sound as stated).
* c67dd343: task-9-review.md:122 says verbatim "EXIT's result value is unrooted from the wrapper's pop to exit_code_for" — transcription faithful; the leak-site comment exists in run.rs and Heap::collect's doc points at it.
* D17 claim (e0e57825): zero `.trace` writes in eval.rs/run.rs/error.rs at that commit; both writers sit in execute()'s error match arms in lib.rs.
* eval.rs:70-71 at 521195d1 carries "Task 11 adds the / limit check to this function" across exactly those two lines, in the depth-owning wrapper's doc.
* ast.rs:776 at 521195d1 is exactly the "measured rc 0" absorbed-WHEN line; the WhenCase value-list doc sits at 802-811 inside the cited 801-815.
* Global constraints do forbid integration tests for private subjects, and Task 9/10/11's old git adds named tests/run_basic.rs, tests/run_select.rs, tests/run_loops.rs — 521195d1's premise holds; "Step 2 to 5 as before" was the pre-image verbatim.
* ef683ab2: Select has exactly one run_bounded call site (run.rs:609); the two production construct sites are If's (:502) and Select's; run_fragment's own doc carries the jump-stays-in-range argument; error.rs at 56e60d96^ had exactly three dead_code allows; NOT_IMPLEMENTED_EXIT is 120, outside 157..=253; Task 12's plan section says nothing about parse errors (supports "never did").
* 56e60d96: Raised::condition carries #[expect]; rexx-run canonicalises with the symlink caveat written at the call site as unobserved; the KNOWN GAPS row's two-echo shape reproduces on the oracle.
* Task 16 claims (e0e57825): phase-4a-gate.md does not exist; the spec's "4a exit gate" section holds exactly seven criteria; the exclusions file predates the step and carries the KNOWN GAPS section.
* The 19 trace prefixes are the enum entries at RexxActivation.hpp:92-110 (cited as 90-110, the enum block).
* Our side: `say 1~a` under rexx-run prints "rexx-exec: a message send is not implemented", exit 120; `cargo test -p rexx-exec -p rexx-core` fully green at HEAD (104 lib tests).

### F5: 521195d1's `leave sel` bullet reads as the ordinary-label form, which raises 28.3

Task 11 body: "`leave sel`, where `sel` labels a `SELECT`, exits the `SELECT`."
Measured: `sel:` as an ordinary clause label before `select` raises 28.3; only `select label sel` is leavable (exits cleanly, `after` prints).
The preceding bullet establishes that ordinary labels do not name a *loop*, but nothing says the same for SELECT, and the bullet's own phrasing "labels a SELECT" describes the `sel:` form more naturally than the LABEL keyword form.
The fact as most plausibly intended (SELECT LABEL) reproduces, so this is a misleading wording rather than a false statement — but it sits in the exact document an implementer is about to build the block stack from, and the coordination-point paragraph beneath it inherits the ambiguity.

## Not reached

* Mutation demonstrations, all of which need source edits this audit was forbidden: a6abb0d7's "{kind:?} regression fails the unit test and leaves the spike test green"; 34f2a4d6's "all seven mutations now fail" and the evaluate-all-check-to-first-false mutant; 63a4092e's "confirmed to kill that mutant"; a9997e55's None-for-source mutation (only its arithmetic was re-derived).
* 63a4092e's "measured byte for byte" is reproduced for the newline case only at the substitution level; the full stderr bytes were compared for shape, not archived.
* The eleven-program verification set of 56e60d96 is not recorded anywhere, so only its named figures were re-measured (all reproduce except 42.11, F4).
* e0e57825's "two earlier recounts gave 239 and 393": 239 is the parent plan's figure per the spec; the 393 recount is another agent's unrecorded run, unverifiable.
* 9495c974's "only test helpers do that today" (loop-outside-instruction-context eval callers) was spot-checked, not exhaustively enumerated.
* The false-condition absorbed-WHEN segfault (SF #2018) was deliberately not reproduced, per instruction.
* 610642a3's provenance narrative was not audited, per instruction.
* The oracle-cliff figures in the exclusions file's DEVIATION 2 (39,900 parens, [34,500, 34,760] calls, 100,000/150,000 flat terms) predate the in-scope commits and were not re-measured.
* Em-dash consistency: not raised as a defect anywhere, per instruction; nothing further to say.

## Summary

Four real errors (F1-F4), one misleading wording in a to-be-built-from document (F5).
Everything else checked reproduced exactly, including all of the oracle-attributed claims the coordinator listed as suspect: the DROP separator family, the seven comparison rows, the nine majors, the indentation rule with IF-THEN as two and `do i = 1 to 'x'` as none, the absolute dot-normalised path, the two-echo INTERPRET shape, `exit 'x'` not raising, and `if 'x', 1 then` being 34.6.
