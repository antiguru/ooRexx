# Task 10 fix round 1 -- re-review

Commit under review: `8b4fd867e643d6f13541688c68531555afa0be94` on top of `3d6d68d776a2df8d754c601c28c4113c4b046256`.
Working tree confirmed clean and at this exact commit before and after all probing (`git status --short` empty, `git diff --stat` empty throughout).

## Per-finding disposition

| Finding | Disposition |
|---|---|
| Critical 1 -- two of three combinations were not new coverage | **Closed.** Ran. |
| Critical 2 -- I19 sweep's "exactly one call site" was false | **Closed.** Ran. |
| Important -- two headers' SIGL claim was backwards | **Closed.** Ran. |

### Critical 1 -- closed, ran

Independently reproduced the frame-leak mutation the report claims already catches both deleted programs: backed up `run.rs` (`md5sum`-verified), skipped `self.roots.pop_slots(callee.frame)` in `resolve_and_run_call`'s `owns_frame` arm (`run.rs:3428-3429`), rebuilt, ran `call_procedure_expose.rex`. Result: `thread 'rexx-interp' panicked ... grow_slots on a frame that is not the top one`, exit 101 -- matches the report's claim exactly. Restored from the byte-identical backup (`md5sum` re-confirmed, `git diff --stat` empty).

Read `run_repeating` (`run.rs:4568-5160`) and `exec_procedure` (`run.rs:1939-2020`) in full, plus `LoopState` (`run.rs:470-...`), `loop_advance`'s `Controlled` arm (`run.rs:5014-5158`), `bind_control` (`run.rs:5284-...`) and `Plan::slot_of` (`plan.rs:528,561`). `LoopState` (`current`/`to`/`by`/`for_remaining`/`stepped`) is confirmed a plain Rust local, passed by value into `run_repeating`, never copied to `Interp`/`Activation` -- the report's premise is true.

The deletions themselves are clean: grepped the whole repository for both removed filenames -- the only surviving hits are in `corpus/phase-4b.txt`'s own explanatory prose about the removal, never a live subset line, sourceline fixture, or test reference. Both `.rex` files and both `sourceline_oracle/*.txt` fixtures are gone from disk. `cargo test -p rexx-exec --test coverage` still passes 9/9 including `every_in_scope_variant_is_witnessed_by_the_phase_subsets` (non-zero run count, confirmed). `cargo test -p rexx-exec --test collect_stress` still 2/2.

See the structural-claim verdict below for the one qualification found.

### Critical 2 -- closed, ran

`grep -rn "\.pop_frame(" crates/ --include="*.rs"` finds exactly seven runtime call sites: `eval.rs:591,651,701,764,804,891` and `run.rs:3749` (two further hits are in `rexx-core/tests/roots.rs`, unit tests of `pop_frame` itself, not runtime consumers -- irrelevant to the I19 concern). Matches the redone count exactly.

Read all six `eval.rs` bodies directly (`eval_prefix`, `eval_arithmetic`, `concat`, `eval_compare`, `eval_logical`, `eval_logical_list`). Every one matches the claimed shape with no exception: the function's own return value is allocated (`self.number(...)`/`self.text(...)`) and bound to a local on the line immediately before `self.roots.pop_frame(frame)`, and `Ok(value)` is the very next line after `pop_frame` -- zero intervening statements in either direction, in all six cases.

Read `RootSet::pop_frame` (`rexx-core/src/roots.rs:141-143`): `self.temps.truncate(frame.0)`, a bare `Vec::truncate`, allocation-free. Confirmed as claimed.

### Important (SIGL) -- closed, ran

Read the committed `call_on_trap_rearms.rex` (`cat -n`): line 54 is the first `call raiser`, line 60 the second, line 66 `raise user zx return 'V'`. Ran the file against the live oracle from a fresh directory: stdout `S[H54]1:VK[H60]2:V`, stderr trace shows SIGL sourced from lines 54 and 60 (the calling clauses), never 66. Matches the corrected header exactly. Built `rexx-run` and ran the same file: stdout and stderr byte-identical to the oracle, exit 0 both sides.

Also ran the header's own claimed mutation (`deliver_pending_trap`'s reinsertion skipped) against the corrected, currently-committed file: `S[H54]1:VK2:V`, matching the header's stated mutation output exactly.

## Structural claim: does a loop-plus-call combination witness remain owed?

**No.**

I built and ran the counterexample the dispatch asked for, in the specific direction the report's own architectural paragraph is weakest: does a nested `CALL` ever reach state a loop's own continuation depends on? Yes, empirically:

```rexx
trace r
do zi = 1 to 5
  call bump
end
say 'done' zi
exit

bump: procedure expose zi
zi = zi + 100
return
```

Oracle: `done 102` (the loop stops after one pass because `bump`'s write to the exposed `zi` is read back on the next re-test). `rexx-run`: byte-identical stdout, stderr and exit code. This is real: reading `loop_advance`'s `Controlled` arm (`run.rs:5086-5121`) shows the re-tested pass deliberately does **not** trust the Rust-local `current` -- it re-reads the control variable through `self.read(code, *control)`, i.e. through the ordinary interpreter variable pool, specifically because a loop body's own write must be visible (Task 9/11's own F2 fix, documented in the surrounding comment). That is exactly the slot a `PROCEDURE EXPOSE zi` aliases into (`exec_procedure`/`alias_slot`), so a nested `CALL` that exposes the loop's own control variable by name absolutely can, and does, change what the loop does next.

This is real state-sharing between a loop and a call, and it contradicts the literal wording of the report's architectural paragraph ("nothing a nested CALL does to interpreter state can reach it" -- see the new finding below). But it does **not** reopen Critical 1, because:

* Exposing a **scalar** by `PROCEDURE EXPOSE` is already witnessed without any loop, by `call_procedure.rex` (`exposed: procedure expose g`) and `call_procedure_expose.rex` (`sub: procedure expose w`, `bee`/`cee: procedure expose n [m]`).
* The code path this counterexample exercises (`alias_slot`'s aliasing, `self.read`'s slot resolution) is generic: it does not distinguish "the aliased name happens to be a loop's control variable" from "the aliased name is any other scalar." No branch in `run_repeating`, `loop_advance`, `exec_procedure` or `alias_slot` special-cases this combination, so there is no mutation site there that only a loop+call program could reach and a sequential-call program could not.

I also independently re-ran the frame-leak mutation (see Critical 1 above) and read the `set_sigl` redundancy the report cites (`resolve_and_run_call` sets `SIGL` again at `run.rs:3282` for every call, including the handler's own dispatch inside `deliver_pending_trap`) -- both check out as described. I did not independently rebuild-and-run the `indent_offset`-restore-skip mutation a second time; I read the surrounding code and found the claim ("inert, all seven programs identical") consistent with what the mutation touches (a trace-indentation field with no effect on stdout/exit code, only on `TRACE`'s own indent bookkeeping across a return), and treat that one claim as reasoned rather than independently re-run.

No further mutation site combining Tasks 5/6/7 was found. The one genuinely unexplored avenue is the same one the report itself names and declines to pursue (`INTERPRET` + `PROCEDURE EXPOSE`, since an `INTERPRET`-introduced name has no parse-time slot and so does force `frame_len`'s growth path to matter) -- that is `INTERPRET` (Task 8), not Tasks 5/6/7, so it is out of this task's own scope, not a gap in it.

## `call_on_trap_rearms.rex` -- confirmed sole witness, ran

Backed up `run.rs`, applied the exact mutation the header describes (skipped the `if let Some(trap) = removed { ... }` reinsertion in `deliver_pending_trap`), rebuilt, and ran **all twelve** programs in the current 4b subset (`corpus/phase-4b.txt`) against both the mutated and the unmutated binary, diffing stdout, stderr and exit code for each:

```
same: interpret_dynamic
same: interpret_error_echo
same: call_return
same: call_expression
same: call_procedure_expose
same: use_arg_forms
same: signal_forms
same: condition_traps
same: push_queue
same: raise_array_substitution
same: loop_retest_blame
DIVERGES: call_on_trap_rearms (exit 0 -> 0, stdout S[H54]1:VK[H60]2:V -> S[H54]1:VK2:V)
```

`condition_traps.rex` -- the program originally credited with catching the *other* combination's mutation -- is confirmed unaffected by this one, because it only fires its `CALL ON` trap once. `call_on_trap_rearms.rex` remains the sole program in the subset this mutation discriminates. Restored `run.rs` from the byte-identical backup, `git diff --stat` empty.

## Gates, re-run independently

| gate | result |
|---|---|
| `cargo test --workspace` | **990 passed / 0 failed** (summed programmatically over every `test result:` line, not read from a tail) |
| `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` | **42 of 42**, mode STRICT |
| `cargo test -p rexx-exec --test assertions` | **4224 / 4259** (unchanged, exempt set intact) |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |

Corpus count independently recomputed, not read from the report: `phase-4a.txt` 30 non-comment lines + `phase-4b.txt` 12 non-comment lines = 42, matching the STRICT gate. `find corpus -name '*.rex' | wc -l` = 62 (64 after round 1's three additions, minus the two removed here). All **ran**.

## New findings

### Minor -- load-bearing (comment correction owed, does not reopen Critical 1)

`corpus/phase-4b.txt`'s new "architectural reason" paragraph overstates its case in one clause. It reads: *"`DO`/`LOOP`'s own control-variable bookkeeping lives in a Rust local on `run_repeating`'s own stack frame, never in interpreter state a nested `CALL` could reach."* The first half is true (`LoopState` is a private Rust local, confirmed by reading). The second half is not, taken literally: a `Controlled` loop's re-tested pass reads the control variable's *live value* through the ordinary interpreter variable pool (`run.rs:5113`, `self.read(code, *control)` -- deliberately not the Rust local, per Task 9/11's own F2 fix, so that a body's write is visible), and that is exactly the slot `PROCEDURE EXPOSE` aliases into. A nested `CALL` that exposes the loop's own control variable by name does reach and change it -- measured on both engines, byte-identical (see the structural-claim section above). This is the same true-premise/missing-second-premise shape `rust/CLAUDE.md`'s own Method section names ("six confident 'X cannot be reached' claims have been wrong here"). It does not change any gate, test, or the deletion decision -- the shared pathway is generic and already witnessed without a loop -- but the sentence as shipped is falsifiable and should say so precisely (e.g., "the loop's own iteration counters -- `by`/`to`/`for_remaining` -- are Rust locals no `CALL` can reach; the control variable's *value* is an ordinary variable like any other, visible to whatever exposes it, and that path is already covered by `call_procedure.rex`/`call_procedure_expose.rex`") rather than the current blanket claim.

No other new findings, Critical or Important. No new prose-only findings (comment with no behavioural claim) beyond the one above, which does make a checkable behavioural claim and is filed accordingly rather than parked.

## Labels

Every claim above is marked **ran** except: the `indent_offset`-restore-skip mutation's "inert" result, which is **reasoned** (read the code, did not independently rebuild and re-run it a second time); and the plausibility of the `set_sigl` redundancy claim, which is **ran** for site existence (`run.rs:3282` read directly, confirmed reachable from `deliver_pending_trap`'s own call path) but **reasoned** for the specific claim that `clause_state.line()` is unchanged between the two call sites in this exact scenario.
