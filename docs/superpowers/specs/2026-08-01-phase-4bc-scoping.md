STATUS: DONE — 37 inherited items (31 owned by 4b or 4c, 6 carried unowned), 14 open decisions.

# Phase 4b/4c scoping groundwork

This document is not a plan.
It is the material a plan gets written from, plus the decisions Moritz has to make before either plan can be written.
Nothing here changes code: it cites the tree and the phase record so a plan author can verify rather than trust.

Every measurement labelled "measured 2026-08-01" was taken here, against `build/bin/rexx`, under the standing `ulimit -v 1048576` wrapper.
Everything else carries a citation to a file and line, a ledger entry, or a gate criterion.

---

## 1. The inherited work list

Thirty-seven items.
Thirty-one carry an owner of `4b` or `4c`; six are carried debts with no owner that a 4b or 4c plan will nonetheless meet.
The grouping is by where the work lands, not by which document recorded it, because the scattering across documents is the thing that has cost this project the most.

Ledger citations are `progress.md:LINE` against `.superpowers/sdd/2026-07-30-phase-4a-executor/progress.md`.
Code citations are relative to `rust/`.

### 1.1 Activation, dispatch, and the interpret spike — 4b, nine items

**I1. `Activation` has no body selector, so a callee re-runs `main`.**
`run_activation` hardcodes `&program.main`.
True for every activation 4a can build; false the moment 4b calls a `::routine`, and the failure is silent and with the right program.
The missing field is a body selector beside `Activation::program`.
*Cite: `crates/rexx-exec/src/activation.rs:44-52`.*

**I2. `BodyKey::directive` is `Some(index)`-shaped and nothing ever sets it.**
The plan cache already carries the field I1 needs on the activation; `None` is the main body and "4b is the first to need `Some(index)` for `directives[index]`'s body".
The two should be decided together or they will disagree.
*Cite: `crates/rexx-exec/src/plan.rs:64-66`.*

**I3. `Activation::new` needs a sibling constructor that inherits `settings` from the caller.**
Measured during 4a: with `numeric digits 7`, an internal `call sub` sees 7, sets its own to 3, and after `return` the caller still reports 7.
`Activation::new` unconditionally defaults `Settings`, and its doc says explicitly that folding both into one function would need a parameter that is `None` on every 4a path.
*Cite: `crates/rexx-exec/src/activation.rs:83-100`; design spec §"The borrow shape", `docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md:305`.*

**I4. `Interp::trace_mode` must move onto `Activation` and be deleted from `Interp`.**
A deliberate 4a-only simplification, not an oversight: 4a has one frame, and measured, a callee's `trace off` does not survive its `return`.
The field's own doc names this as "4b's first move here".
*Cite: `crates/rexx-exec/src/lib.rs:586-602`; design spec line 305.*

**I5. `RootSet::grow_slots` panics on a non-top frame, and that invariant is false under `PROCEDURE EXPOSE`.**
Measured: `sub: procedure expose zzz` with `zzz = 5` in the callee makes the caller print 9, so a callee writes into a caller's pool while the callee's frame is on top.
4b either grows a non-top frame or binds an exposed name to a slot in the caller's frame at call time.
The panic message pins the string `4a invariant` and a `#[should_panic(expected = "4a invariant")]` test pins the wording, so 4b must remove it deliberately rather than discover a silent allowance.
*Cite: `crates/rexx-core/src/roots.rs:186-206`; `crates/rexx-core/tests/collect.rs:163-181`; design spec line 222.*

**I6. Activation depth is decided and unpaid: one Rust frame per activation plus an explicit counter, raising 11.1 at rc 245.**
Measured during 4a: unbounded `CALL` recursion gives `Error 11.1`, "Insufficient control stack space", rc 245 — a reportable condition, not a crash.
D19 chose per-frame recursion, and reopening D19 reopens the `Rc<Program>` risk it closed.
*Cite: design spec lines 284 and 522.*

**I7. `run_program_interpret_spike` and `Interp::interpret_spike` are 4b's to delete.**
Three tests consume the entry point (`tests/spike.rs:122,150,300`).
The doc comment records the trade that created the public surface and says 4b should re-make it rather than inherit it.
See decision D1.
*Cite: `crates/rexx-exec/src/lib.rs:696-704` and `1005-1036`; design spec line 317.*

**I8. Fragment plans are built, used and dropped, and whether to cache them is 4b's call.**
Revision 6's `(enclosing body, fragment id)` key was withdrawn as "sound and useless" — fragment text varies per execution so every lookup misses while every entry is retained.
Caching keyed by text is allowed only if it can show a hit rate.
*Cite: design spec line 215; `crates/rexx-exec/src/plan.rs:50-60`.*

**I9. `Fragment::body.labels` is always empty and 4b needs no label table there.**
Measured and settled in Task 1: an `INTERPRET` fragment can never contain a label, 47.1 both ways.
*Cite: `progress.md:7-8`.*

### 1.2 Conditions and the error report — 4b, seven items

**I10. `Raised::condition` has no reader.**
Carried as a field rather than hardcoded because 4b's `NOVALUE`, `NOMETHOD` and friends need to set it to something else; the first genuine reader is 4b's `SIGNAL ON` and `condition('c')`.
It is `#[expect(dead_code)]` rather than `#[allow]` on purpose, so the day 4b reads it the annotation asks to be deleted.
*Cite: `crates/rexx-exec/src/error.rs:36-55`.*

**I11. `Interp::failure_site` is never cleared mid-run.**
It matters only once a condition trap can resume execution after a raise, so clearing it in 4a would be scaffolding for a caller that does not exist.
It is set first-call-wins (`self.failure_site.is_none()` guards both callers), so a second raise after a trapped first one would report the first site.
*Cite: `progress.md:1765-1767`; `crates/rexx-exec/src/run.rs:1199,1316,1368`.*

**I12. The error report emits one clause echo; the oracle emits one per nesting level.**
Recorded as a KNOWN GAP from an `INTERPRET` measurement, with the note that closing it means giving `Raised::report` a stack of sites rather than one, and that "whoever implements 4b's real `INTERPRET` or 4b's `CALL` should decide the shape then".

Measured 2026-08-01, and this goes further than the recorded gap in two ways the exclusions file does not capture:

* `CALL` produces one echo per **activation**, innermost first, and each echo carries **that activation's own line number** — unlike `INTERPRET`, where both echoes carry the enclosing clause's line.
  A raise inside `sub:` called from line 1 prints `4 *-*   say 2 & 1` then `1 *-* call sub`.
* The echoes **accumulate two spaces of indent per activation**, on top of the lexical `static_indent` Task 11 already computes.
  A three-deep stack (`call a` → `call b` → `interpret`) prints indents 4, 4, 2, 0 for lines 5, 5, 3, 1 — the `INTERPRET` fragment shares its caller's indent, and each `CALL` adds two.

So 4b does not merely add lines to the report: it adds an **activation-base offset** to a quantity Task 11 built as a pure function of the flat instruction list, and `static_indent`'s "pure function of lexical depth" property survives only if the base is added outside it.
*Cite: `docs/superpowers/plans/phase-4-exclusions.txt:224-245`; `progress.md:1094-1097`; measured 2026-08-01.*

**I13. `Novalue::Unset` is produced by the read path and read by nothing.**
D16 required the flag from the start rather than retrofitting a raise into the hottest path; `SIGNAL ON NOVALUE` is its first reader, and the Phase 4 gate program uses `signal on novalue`.
*Cite: `crates/rexx-exec/src/lib.rs:544-556`; design spec line 226.*

**I14. Nine of the nineteen trace prefixes have no witness, and 4b/4c own six of them.**
Read from `RexxActivation.cpp:3567-3588`: `>F>` FUNCTION (4b internal routine, 4c builtin), `>A>` ARGUMENT (4b), `>I>`/`<I<` INVOCATION/EXIT (4b), `>R>` ALIAS (4b's `USE`), `>.>` DUMMY (4c's `PARSE` placeholder).
`>M>` MESSAGE and `>N>` NAMESPACE are Phase 5's; `+++` ERROR is command errors and failures, Phase 7's under D18.

Measured 2026-08-01, and worth knowing before scoping: a trapped `SIGNAL ON SYNTAX` under `trace r` emits **no** `+++` and no error report at all.
The trap label's own clause is echoed as an ordinary `*-*`.
So condition traps do not bring `+++` into 4b.
*Cite: `crates/rexx-exec/tests/trace_oracle.rs:26-35`; `RexxActivation.cpp:3567-3588`; measured 2026-08-01.*

**I15. `PUSH`, `QUEUE`, and the in-process queue are 4b's, and `QUEUED()` is a partial 4c exclusion resting on them.**
Cross-process sharing with the oracle's rxapi-backed session queue will never match, so a differential run of `QUEUED` is single-program only.
Confirmed on this host 2026-08-01: `rxapi` is running (pid 857) and `rxqueue('G')` returns `SESSION`, so the caveat is live rather than theoretical.
*Cite: `docs/superpowers/plans/phase-4-exclusions.txt:70-73`; design spec line 489.*

**I16. The temps-frame conclusion 4b must not invalidate.**
Six `push_frame` sites in `eval.rs` skip their `pop_frame` on the `?` path; `step_in_temps_frame` is the single chokepoint that heals them, and it is the *only* caller of `step` in the crate.
The investigation's conclusion that `SIGNAL ON SYNTAX` cannot accumulate leaks rests entirely on that chokepoint: a trap acts at instruction-loop level, and the wrapper has already truncated before the `Failure` reaches the loop's `Err` arm.
If 4b moves execution off that chokepoint, the whole analysis must be redone rather than assumed.
*Cite: `progress.md:1237-1287`, especially 1262-1264 and 1284-1287; `.superpowers/sdd/2026-07-30-phase-4a-executor/temps-frame-investigation.md`.*

### 1.3 Variables and stems — 4b, two items

**I17. Stem aliasing makes a currently-equivalent mutant observable.**
A mutant rerouting `stem_drop` to a slot clear survives every test today, and it is a genuine equivalent mutant rather than coverage rot: `stem_drop` is `replace_stem(name, None)`, and a freshly rebound stem is observationally identical to a cleared slot until something can hold a second reference to the old stem object.
Nothing in 4a can; `PROCEDURE EXPOSE` and argument passing can.
It becomes pinnable exactly when 4b lands, and 4b's plan should say so rather than let the mutation script quietly keep reporting it as expected-survivor.
*Cite: `progress.md:1450-1456`; `docs/superpowers/plans/phase-4a-gate.md:107`.*

**I18. `RootSet::clear_slot` exists so the read path can tell "unset" from every other value, for 4b's `NOVALUE`.**
`stem_drop` deliberately does *not* use it, and the doc comment explains why a stem's slot is not "empty or not" the way a simple variable's is.
A 4b implementer wiring `NOVALUE` needs both halves of that distinction.
*Cite: `crates/rexx-exec/src/stem.rs:288-305`; `progress.md:509`.*

### 1.4 Rooting and the collector — 4b, four items

**I19. `EXIT`'s result is under-rooted from the temps-frame pop to `exit_code_for`.**
Under-rooting, the direction that actually breaks when a collector lands, and longer than any window the crate documents.
Harmless today only because nothing between that pop and `exit_code_for` calls `alloc_with` — the conversion fills a `Number` in place or parses onto the Rust heap — so a faithful collect-on-every-allocation mode never fires inside it.
The deferral was ruled to stand because the real fix needs a root that survives a frame pop, a mechanism with no other user; the pointer was deliberately put on `Heap::collect` rather than at the leak site, because the person who turns this into a use-after-free is whoever wires a collector into the interpreter.
That instruction also says to sweep `rexx-exec` for the same shape first.
*Cite: `progress.md:1393-1397` and `1421-1433` (commit c67dd343); `docs/superpowers/plans/phase-4a-gate.md:75`.*

**I20. The collect-on-every-allocation mode has never seen a call frame.**
Criterion 4 passed on 29 programs, all of them 4a-shaped.
The gate says so plainly: "4b's body-calls-body recursion, argument passing, and everything Phase 5's object model eventually adds are all untested by this mode as it stands".
*Cite: `docs/superpowers/plans/phase-4a-gate.md:73`.*

**I21. A fifth allocation site added by 4b must go through `Interp::alloc_with`.**
`Heap::alloc_with` was renamed `alloc_with_uncollected` specifically so a new allocation site written the natural way announces at the call site that it bypasses the stress hook.
"Exactly four call sites, verified by grep" was true on the day and is exactly the fact that goes stale silently.
*Cite: `progress.md:2672-2678`; `crates/rexx-exec/src/lib.rs:946-955`.*

**I22. `pop_frame`'s truncation semantics are load-bearing and no assert may be added there without balancing the six `eval.rs` sites first.**
Recorded as a comment for exactly this reason, plus a note in `eval.rs` that the `?` skips are deliberate and later tasks should copy the pattern.
An optional debug tripwire in `step_in_temps_frame` asserting temps balance on the `Ok` path was scheduled and not built; it needs a `temps_len()` accessor.
*Cite: `progress.md:1277-1282`.*

### 1.5 Owned by 4c — eight items

**I23. The `ADDRESS` instruction's environment-name tracking is 4c's, alongside the `ADDRESS()` builtin.**
Both are load-bearing for the Phase 4 gate program: `rexxcps.rex:143` is `trace value trace(); address value address()`.
The platform-supplied default is Phase 7's — measured, `say address()` with no `ADDRESS` instruction prints `sh`.
*Cite: design spec lines 245-248 and 502; `docs/superpowers/plans/phase-4-exclusions.txt:65-68`.*

**I24. `VALUE` is split: the variable-access form is 4c's, the external-selector form (a pool such as `ENVIRONMENT`) is Phase 7's.**
*Cite: `docs/superpowers/plans/phase-4-exclusions.txt:61-64`; design spec line 487.*

**I25. `ExprKind::Call` is split 4b/4c, and the owner string says `4b`.**
A Rexx call resolves internal routine, then builtin, then external.
The owner named is the phase after which the variant stops failing loudly for *some* target.
A reader at the end of 4b who finds `f(1)` still loud for a builtin name is seeing exactly this.
Practically: **whichever sub-phase runs first has to build the `ExprKind::Call` arm in `eval`**, and the other inherits it.
*Cite: `docs/superpowers/plans/phase-4-exclusions.txt:151-160`; `progress.md:2493-2498`.*

**I26. Three `num/` corpus programs were dropped from the 4a subset for `ExprKind::List`, and the tree contradicts itself about who gets them back.**
`corpus/phase-4a.txt:18` and `corpus/README.md:108-109` both say they "stay in this directory for 4b or 4c, once `List` exists".
`phase-4-exclusions.txt:176-179` — the later Task 16 ruling — assigns `List` to **Phase 5**.
Both cannot be right, and the corpus files are the ones a 4b or 4c author will read first.
See decision D7 for the measurement that bears on it.
*Cite: `corpus/phase-4a.txt:18-21`; `corpus/README.md:108-109`; `docs/superpowers/plans/phase-4-exclusions.txt:176-179`.*

**I27. The 342 expected trace-output lines in `TRACE.testGroup` are 4b's and 4c's to satisfy, not 4a's.**
An ooTest group is not runnable as extracted.
Any recount must state which scan it used: the same file yields 239, 342, 374, 393 and 437 under five different (all defensible) anchorings, and three recounts have already gone astray.
*Cite: design spec lines 238-240; `progress.md:2112` and `1504-1510`.*

**I28. The L1 groups: `base/keyword` for 4b, `base/bif` for 4c at 66 rows.**
The split table names them as each sub-phase's L1 obligation.
Nothing extracts them today — `rexx-extract` has `extract` (test methods) and `extract_assertions` (the `base/expressions` table) and nothing else.
*Cite: design spec lines 66-67; `crates/rexx-extract/src/lib.rs:53,233,548`.*

**I29. `samples/rexxcps.rex` is the end-of-4c gate, and its dependencies are enumerated.**
`parse var`, `parse version`, `parse value`, `parse upper`, `parse source`, `trace value`, `trace off`, `signal on novalue`, one internal `call subroutine` and therefore a `Label`, the `call time 'R'` call-to-builtin form, `address value` with `ADDRESS()`, and eight builtins: `TIME`, `SUBSTR`, `FORMAT`, `WORD`, `TRACE`, `LENGTH`, `LEFT`, `ADDRESS`.
Nothing from Phase 5 or later.
See decision D8: read in full, its output is not byte-reproducible.
*Cite: design spec lines 499-502.*

**I30. `QUEUED` in scope, differential single-program only.**
Restated here as 4c's row rather than 4b's; the queue is I15.
*Cite: `docs/superpowers/plans/phase-4-exclusions.txt:70-73`.*

### 1.6 Carried with no owner — six items a 4b/4c plan will meet anyway

**I31. A Controlled (`TO`-style) loop's re-tested pass omits two `>>>` value lines.**
KNOWN GAP, measured, cause read from `DoBlock::checkControl` (`DoBlock.cpp:182`) rather than inferred, and costed honestly at about twenty lines plus re-verification of bound-before-test, `FOR` and `ITERATE` — half a day, not a rewrite.
An overstated cost in a gap row is how a cheap fix stays open; the row was corrected once for exactly that.
*Cite: `docs/superpowers/plans/phase-4-exclusions.txt:247-261`; `progress.md:2381-2385`.*

**I32. Prefix-operator chains recurse in `message_subterm` outside the shared depth budget, aborting between 1,150 and 1,200 levels on a default 2 MiB thread.**
The oracle's cliff for this construct has never been measured, so the size of the divergence is unknown.
*Cite: `docs/superpowers/plans/phase-4-exclusions.txt:206-210`.*

**I33. `Debug`, `PartialEq` and `Clone` on `Expr` are still recursive, cliffs at 2,000/2,050 and 2,100/2,200.**
No non-test path reaches them on a deep tree today; the trigger for scheduling is the first test that formats or compares a deep tree.
*Cite: `docs/superpowers/plans/phase-4-exclusions.txt:212-215`; `progress.md:333-336`.*

**I34. The depth counter protects a sized caller only.**
On a default 2 MiB thread the native abort arrives at 331 parens or 341 calls, long before any counter at 50,000 fires.
The long-term answer is a documented minimum stack or a sized entry point in `rexx-parse`.
*Cite: `docs/superpowers/plans/phase-4-exclusions.txt:217-222`.*

**I35. `Plan::by_symbol` is a `HashMap` where D16's shape wants a `Vec` index.**
`SymbolId::index()` landed at 180875a9 so the swap is now this crate's decision rather than blocked on `rexx-parse`.
Variable lookup is 8.1%/32.2% of runtime, so it is worth its own measurement rather than a side effect.
Note the constraint recorded at the time: `Option<usize>` is still required because keywords, labels and constants share the `SymbolTable`, so a dense `Vec` has holes.
*Cite: `crates/rexx-exec/src/plan.rs:78-91`; `progress.md:716-718` and `1104-1105`.*

**I36. `coverage.rs` and `loud.rs` duplicate the owner table by hand.**
An integration test cannot `mod` another test binary's directory, and no shared module was in scope.
4b and 4c will edit both on every variant they deliver; a divergence between them is not caught by anything.
*Cite: `docs/superpowers/plans/phase-4a-gate.md:91`.*

**I37. The `DO OVER` on a stem deviation carries a corpus rule that binds 4b's and 4c's corpora too.**
"NO CORPUS PROGRAM MAY CONTAIN `DO OVER` ON A STEM."
Such a program could never pass: the oracle's order is deterministic and ours is a different deterministic order.
*Cite: `docs/superpowers/plans/phase-4-exclusions.txt:79-97`.*

---

## 2. Open decisions

Fourteen.
Each is a question, the options, and a one-line recommendation with what would change my mind.

### D1. Does 4b's `INTERPRET` replace the Task 3 spike entry point, or extend it?

* **Option A, replace.** Delete `run_program_interpret_spike`, delete `Interp::interpret_spike`, and move `tests/spike.rs`'s three fragment tests onto `run_program`.
  This is what the field's own doc says ("4b's first move here is to delete this field and the branch that reads it").
* **Option B, extend.** Keep the spike as the *unit* path and let `run_program` gain the keyword independently.
  Two entry points into one machinery, which is what the doc warns against.
* **Option C, replace and re-make the underlying trade.** The doc records that a `#[cfg(test)] mod tests` inside `lib.rs` could prove the same lifetime with **no public surface at all**, and calls that a defensible-but-not-obvious trade that 4b should re-make rather than inherit.

**Recommend A.**
Changed by: if the 4b plan wants a fragment-lifetime test that runs *without* the `INTERPRET` keyword being correct, B becomes real; that is the only case where the spike is not redundant.
Note that C is free to fold into A and costs a public item.

### D2. Does the one-echo-per-nesting-level report shape change when `CALL` makes nesting ordinary?

The recorded gap (I12) describes only "one echo per nesting level, innermost first, both on the enclosing clause's line" — measured from `INTERPRET`.
Measured 2026-08-01, `CALL` behaves differently in two ways: each echo carries its **own activation's** line number, and each activation adds **two spaces of indent** to everything below it.

* **Option A, a site stack on `Raised` (or on `Interp`), pushed at each activation boundary and at each fragment entry, with an indent base per entry.**
  `Raised::report` walks it innermost-first.
* **Option B, resolve the whole stack at report time by walking `Interp::activations`.**
  Cheaper, but `run` pops the activation before `execute` sees the error — the exact reason `failure_site` exists at all — so B needs the pops to stop happening or the stack to be snapshotted, which is A with extra steps.
* **Option C, keep one site through 4b and take the stack in a later round.**
  Every raise inside any routine then differs from the oracle on stderr, which is most of what 4b's differential corpus would contain.

**Recommend A, decided before `CALL` is implemented, not after.**
Changed by: nothing I can see; C makes 4b's corpus nearly untestable and B is blocked by the teardown order the crate already documents.
Two things the plan must carry explicitly, because they are not in any durable document today: the per-activation line number, and the `+2` indent base sitting **outside** `static_indent` so Task 11's "pure function of the flat instruction list" property survives.

### D3. Is `DO OVER` on a stem still excluded once `PROCEDURE EXPOSE` exists?

The deviation is about traversal *order* — the oracle walks `CompoundVariableTable`'s balanced tree, we use a hash map, and measured, tails 1, 2, 3, 10, ZZ, B yield `1 B 3 2 ZZ 10`.
`PROCEDURE EXPOSE` changes stem *identity* (aliasing), not iteration order, so it does not touch the deviation's premise.

Worth stating because it is the thing a plan author would check: **no in-scope 4c builtin exposes stem traversal order.**
The 81-name table has no stem-iteration builtin; `~makearray`/`~allIndexes` are message sends, Phase 5's.
So `DO OVER` on a stem remains the *only* exposure through the end of 4c.

* **Option A, deviation stands unchanged, corpus rule carries into 4b and 4c.**
* **Option B, reopen it and reproduce the tree.** The entire cost the hash map exists to avoid, for a construct measured at zero occurrences in ooTest, zero in `CoreClasses.orx` and `StreamClasses.orx`, and a handful in Windows-only samples no gate runs.

**Recommend A.**
Changed by: a Phase 5 requirement (`Stem~makearray` order) forcing the tree anyway, at which point `DO OVER` becomes free — so the honest framing is "deferred to whenever Phase 5 decides", not "permanent", and if that reading is accepted the row arguably belongs in EXCLUSIONS rather than DEVIATIONS.
That reclassification is itself a plan amendment.

### D4. Which of 4c's 15 excluded builtins are genuinely blocked, and which are merely unscheduled?

Measured 2026-08-01 on this host.

| Builtin | Measured | Genuinely blocked? |
|---|---|---|
| `QUALIFY` | `qualify('foo.txt')` returns an absolute path for a file **that does not exist** | **No.** Pure path manipulation against the cwd; `std::path` plus `std::env::current_dir` covers it. Platform-specific path syntax is the only real content. |
| `USERID` | returns `moritz` | **Yes, weakly.** Rust `std` has no username API; a faithful answer needs `getpwuid` (libc, and the workspace forbids `unsafe`) or a crate. Its value is also host-dependent, so a differential run is single-machine — the same caveat `QUEUED` already carries. |
| `SETLOCAL` / `ENDLOCAL` | both return `1` | **Yes, and harder than the exclusions file implies.** They save and restore the process environment, and `std::env::set_var` is `unsafe` in edition 2024 — verified by compiling it, `error[E0133]`. The workspace has `unsafe_code = "forbid"`. Phase 7 will need either a shadow environment or a lint exception, and that is a decision, not a task. |
| `STREAM`, `CHARIN`, `CHAROUT`, `CHARS`, `LINEIN`, `LINEOUT`, `LINES` | `stream('nosuch.txt','C','QUERY EXISTS')` returns empty | **Yes.** These are the stream model. No argument. |
| `RXQUEUE`, `RXFUNCADD`, `RXFUNCDROP`, `RXFUNCQUERY` | `rxqueue('G')` returns `SESSION`; `rxapi` is running as pid 857 | **Yes.** The daemon protocol. The `SESSION` answer comes from a live rxapi, which is also why `QUEUED`'s single-program caveat is real. |

So: **11 of 15 genuinely blocked, 1 not blocked at all (`QUALIFY`), 3 blocked but for a reason nobody has written down (`USERID`, `SETLOCAL`, `ENDLOCAL` need a decision about `unsafe`/crates/host dependence, not just "the platform layer").**

* **Option A, leave all 15 excluded.** The count `66 of 81` stays, `coverage.rs`'s `EXCLUDED_BUILTINS` literal stays, no amendment.
* **Option B, move `QUALIFY` into 4c.** 67 of 81, and three assertions in `the_builtin_exclusion_set_matches_the_committed_file` change (`names.len() - (len - 3)`, the `18`, and the `66`), plus the prose in the exclusions file.

**Recommend A, with the exclusions file gaining one sentence per row saying *why* each is blocked.**
Changed by: `QUALIFY` turning out to be needed by an L1 `base/bif` group 4c must pass, which would make B forced rather than optional.
The `SETLOCAL`/`ENDLOCAL` `unsafe` finding should be recorded in the exclusions file now regardless of which option wins, because Phase 7 will otherwise meet it cold.

### D5. Sub-phase order: 4b then 4c, 4c then 4b, or interleaved?

See section 3 for the coupling argument.
Options: strict 4b→4c; strict 4c→4b; 4b with a named 4c slice (the `ExprKind::Call` arm plus the pure-string builtins) pulled forward into a second lane.

**Recommend 4b first, strictly, unless a second agent lane is genuinely available — in which case pull forward exactly the `ExprKind::Call` dispatch arm and the string builtins.**
Changed by: D9 (`PROCEDURE EXPOSE` storage) turning out to need a `RootSet` redesign, which stalls 4b's front end and makes the string builtins the obvious parallel work.

### D6. One corpus subset file per sub-phase, or one growing list?

`rust/corpus/phase-4a.txt` is read by **three** harnesses, each with its own `read_subset` copy: `tests/corpus.rs:186`, `tests/coverage.rs:586`, `tests/collect_stress.rs:96`.
All three hardcode the filename at their call sites.

* **Option A, `phase-4b.txt` and `phase-4c.txt` beside it**, each harness reading the union (or a list of lists).
  Keeps "the 4a subset ran green at commit X" as a durable, re-runnable claim.
* **Option B, grow `phase-4a.txt` and rename it.**
  Cheaper, and destroys the ability to say which sub-phase a regression belongs to.
* **Option C, one file with owner tags per line.**

**Recommend A.**
Changed by: nothing much; the cost is a `&[&Path]` in three places.
Note that under A, criterion 4's `collect_stress` should run the union, because I20 is precisely that the mode has never seen a call frame.

### D7. `ExprKind::List` — Phase 5's, or a 4c approximation?

Measured 2026-08-01: `x = (1, 2)` gives an object where `x~class` is `The Array class`, `x~items` is 2, `length(x)` is 3, and its string value is the elements joined by `'0a'x`.
So the Task 16 ruling (List → Phase 5, "the general form builds an array") is correct about the model.

But: **a newline-joined string is byte-identical to the real Array for every use that is not a message send.**
`say (1,2)`, `length((1,2))`, `substr((1,2),2,1)` all agree.
4a's own value model has no message dispatch, so the difference is unobservable until Phase 5 lands.

* **Option A, `List` stays Phase 5's; the three `num/` programs return in Phase 5; correct `corpus/phase-4a.txt:18` and `corpus/README.md:108-109`, which currently say "4b or 4c".**
* **Option B, 4c ships the newline-joined-string approximation, gets the three programs back, and records a DEVIATION that Phase 5 must delete.**
  This is a deliberate wrong model whose divergence is invisible to every test that can exist before Phase 5 — which is exactly the "measurement that cannot distinguish two hypotheses" shape this phase kept finding.
* **Option C, 4c ships nothing and the corpus comment is corrected.** Same as A minus the Phase 5 promise.

**Recommend A.**
Changed by: an L1 `base/bif` group needing `SAY`-with-comma to pass in 4c, which would make B tempting; even then B should be a recorded deviation with a Phase 5 deletion date, not a silent equivalence.
The two-file contradiction needs fixing in this round regardless of which option wins.

### D8. Does `rexxcps.rex` work as the end-of-4c gate?

Read in full 2026-08-01.
It does not work as a byte-for-byte differential, for two independent reasons:

* It prints wall-clock timings and a derived clauses-per-second figure (`say '     Performance:' format(1000/thousand,,0) 'REXX clauses per second'`).
* Worse, its loop count is **auto-adjusted from measured elapsed time** (`count=(1%total + 1) * count`, repeated until `total>1`), so the number of output lines and the control flow itself depend on how fast the host is.

* **Option A, redefine the criterion as "runs to completion at rc 0, and every line matches a committed shape template".**
  Keeps the program as the integration test it is.
* **Option B, use a modified copy with `count` and `averaging` pinned and the timing lines suppressed**, committed under `rust/corpus/`.
  Byte-comparable, but it is no longer the sample program.
* **Option C, drop it as a gate and replace it with the dependency list it was chosen for** (I29), each item witnessed separately.

**Recommend A plus C: keep `rexxcps.rex` as a run-to-completion smoke test, and make the real gate the enumerated dependency list, each with its own differential witness.**
Changed by: nothing — the auto-adjust makes B a different program and A alone tests less than the list does.
This needs deciding before the 4c plan is written, because the design spec currently states the gate as though it were a normal differential.

### D9. `PROCEDURE EXPOSE` storage: grow a non-top frame, or bind exposed names to caller slots at call time?

The design spec states both as live options and says deciding is 4b's (I5).

* **Option A, relax `grow_slots` to allow a named non-top frame.**
  Smallest change to the call path; makes `RootSet`'s LIFO story weaker, and the `#[should_panic(expected = "4a invariant")]` test must be deliberately rewritten rather than deleted.
* **Option B, resolve every exposed name to a slot in the caller's frame at call time.**
  Keeps the top-frame-only invariant intact.
  Needs the callee's plan to carry an indirection per exposed name, and has to answer what happens when the caller's body never mentions the name (the measured case: the caller's plan has no slot to write into).

**Recommend B, because it keeps the invariant that `grow_slots`'s panic and its test both defend, and because `Activation::extra` already exists for exactly "a name bound at run time that the plan never saw".**
Changed by: `EXPOSE` on a stem turning out to need object identity sharing that a slot indirection cannot express — measured in 4a that `b. = a.` shares the *same* `Body::Stem` object, so an exposed stem may want the object, not the slot.
That one measurement should be taken before this is decided.

### D10. Activation dispatch: confirm D19's per-frame recursion, or reopen it?

D19 chose one Rust frame per activation with an explicit counter.
The alternative — a flat loop over the activation stack — is explicitly noted as also producing the right 11.1/rc 245, and D19's own argument was that the dispatch loop should not be designed twice.
Reopening it reopens the `Rc<Program>` risk: "the flat-loop variant is where the local must be re-derived at every frame transition".

* **Option A, confirm per-frame.** No design work; the depth counter is new code.
* **Option B, reopen.** Costs the `Rc` shape re-proof.

**Recommend A, and say so in the 4b plan explicitly rather than inheriting it silently** — this is one of the decisions most likely to be re-litigated by an implementer who has not read D19.
Changed by: the per-activation Rust recursion interacting badly with Task 10/11's already-nested `run_bounded` (each source nesting level already costs a Rust frame), which would make the combined depth budget the thing to measure first.
That measurement does not exist: `INTERPRETER_STACK_BYTES`'s doc names four consumers and its fourth is an admitted unmeasured gap.

### D11. Determinism policy for `RANDOM`, `DATE`, `TIME` in the 4c corpus.

Measured 2026-08-01, and the result is not the obvious one: **two separate oracle processes running `say random(1,100)` twice produced the identical sequence (2 then 80).**
ooRexx's unseeded `RANDOM` is deterministic per process.
So a corpus program using `RANDOM` *is* differentiable — but only by reproducing ooRexx's exact PRNG and its default seed, not by "returns a number in range".

* **Option A, corpus rule: no `RANDOM`, `DATE` or `TIME` in any differential program**, the same shape as the `DO OVER` rule.
  These three get unit tests against pinned properties instead.
* **Option B, reproduce ooRexx's PRNG exactly** and allow seeded and unseeded `RANDOM` in the corpus; keep `DATE`/`TIME` excluded.
* **Option C, allow only the seeded form.** Note this still requires reproducing the PRNG.

**Recommend A, with the PRNG question raised as its own item if an L1 `base/bif` group turns out to assert `RANDOM` values.**
Changed by: exactly that — if `base/bif` pins `RANDOM(1,100,42) = 89`, B is forced.
Whatever is chosen, the corpus README needs the rule written next to the `DO OVER` one, because that is where the next author looks.

### D12. Where do 4b's and 4c's L1 tables come from?

`rexx-extract` has `extract` and `extract_assertions`; the latter is specific to `base/expressions`'s `assertSame` shape and already needed two modelling corrections (single-quoted method names, and `expectSyntax` markers changing what a later `assertSame` *means*).

* **Option A, a third extractor per sub-phase** (`base/keyword`, `base/bif`), each with its own conservation invariant of the `rows + dropped == calls` kind that caught the Phase 0 defect.
* **Option B, generalise `extract_assertions`.**
  Risky: the two corrections above were both about a group's *mechanics*, and the same shape will recur.
* **Option C, hand-written tables.** 66 builtins, and the 342-line `TRACE.testGroup` figure says the keyword side is bigger.

**Recommend A, and require the conservation invariant explicitly** — "a percentage cannot notice a missing population; a conservation law can" is the single most valuable thing 15a produced.
Changed by: `base/bif` turning out to use a shape `extract_assertions` already models, which is worth ten minutes of checking before the plan is written.

### D13. One plan and one gate for 4b+4c, or two of each?

Phase 4's parent row closes when 4c closes.
4a's gate has seven criteria and its assessment is 140 lines.

* **Option A, two plans, two gates.**
  Matches how 4a was run and keeps the corpus-per-sub-phase story (D6) coherent.
* **Option B, one plan with two halves and one gate at the end.**
  Fewer gate-day instruments to build twice; risks 4b shipping unassessed.
* **Option C, two plans, one gate.**

**Recommend A.**
Changed by: the gate instruments (`coverage.rs`, `loud.rs`, the mutation script) turning out to need near-identical edits twice, which argues for C.
Note that criterion 2's exempt list cannot light up at either gate (see section 4), so both gates need that stated up front rather than discovered.

### D14. Does the 4a exit gate's criterion set carry forward, and if so with what amended wording?

Two of 4a's seven were met weakly and one of the weaknesses is a wording defect that will recur verbatim.
Criterion 2's text contemplates a blocked row being unblocked by "4b or 4c" and never names Phase 5; all 35 exempt rows need Phase 5.
Criterion 3's trace table has no measure of its own coverage, and four divergences were found by probing adjacent shapes.

* **Option A, carry the seven forward with criterion 2's wording fixed to name Phase 5 and criterion 3 gaining a coverage measure of its own.**
* **Option B, carry them forward unchanged** and re-derive the same two defects at each gate.
* **Option C, rewrite the criterion set for 4b/4c from the inherited work list above.**

**Recommend A.**
Changed by: nothing; B is known-broken and C loses the continuity that makes "the 4a subset still passes" checkable.
Criterion 3 in particular needs a real answer: the honest statement today is that its five witnesses verify what they cover and the trace surface's coverage is measured by nothing, and 4b/4c add six more prefixes to an unmeasured surface (I14).

---

## 3. Sequencing, and the actual coupling

### 3.1 What blocks what, concretely

**4c depends on 4b in six named places, not vaguely.**

* `ExprKind::Call` must have an arm in `eval` before a single builtin can be called at all.
  This is the same arm 4b builds for internal-routine resolution — a Rexx call resolves internal routine, then builtin, then external, in that order, so the arm's *front* is 4b's and the builtin table hangs off its fallback (I25).
* `ARG()` and `ARG(n)` inside a routine need 4b's argument passing.
* `CONDITION()` needs 4b's condition traps.
* `QUEUED()` needs 4b's in-process queue (I15).
* `PARSE ARG` needs arguments; `PARSE PULL` needs the queue.
  `PARSE VAR`, `PARSE VALUE ... WITH`, `PARSE SOURCE`, `PARSE VERSION` and `PARSE UPPER` need nothing from 4b.
* The `CALL builtin` *instruction* form (`call time 'R'`, which `rexxcps.rex` uses) needs the `CALL` instruction, 4b's.

**4c's own gate program needs 4b.**
`rexxcps.rex` uses `signal on novalue`, an internal `call subroutine`, and `parse upper arg a1 a2 a3 ., a4`.
So even if every builtin were done first, 4c could not be gated.

**4b depends on 4c in one place, and it is soft.**
Nothing in 4b's construct list needs a builtin.
The one contact is that a 4b differential corpus written naturally will reach for builtins to make routines do something observable, and it must not — `say`, assignment and arithmetic are enough.
That is a corpus-discipline rule, not a dependency.

**4b depends on 4a's error path, which exists.**
`Raised`, `FailureSite`, `record_failure_site`, `Raised::report`, the catalogue and the 256-major exit rule are all built and byte-verified.
What 4b adds is a *reader* (`Raised::condition`, I10), a *resume* (which makes `failure_site`'s never-cleared state matter, I11), and a *stack* (I12/D2).
`RAISE` is the cheapest instruction in 4b's list because the raiser families already exist.

**4b's internal ordering is forced.**
The body selector (I1/I2) before `CALL`; `CALL` before `PROCEDURE`/`EXPOSE`; the report's site stack (D2) before any 4b differential corpus program can be byte-identical, because every raise inside a routine hits it.
`SIGNAL` to a label needs only the label table 4a has, and `INTERPRET` needs only the fragment machinery 4a built, so both can go first and neither unblocks anything.

### 3.2 The recommendation

**4b first, strictly, unless a second lane is available.**

The parallel slice, if there is a second lane, is exactly: the `ExprKind::Call` dispatch arm in `eval`, plus the ~50 builtins whose arguments are strings and numbers and whose results are strings and numbers (`ABBREV`, `ABS`, `B2X`, `BITAND`…`WORDS`, `X2B`…`XRANGE`, `LOWER`, `UPPER`).
Those need only expression evaluation, which exists, and the value model, which exists.

**Where they cannot run in parallel, and this is the real constraint:** both slices live in `rexx-exec`, and this phase's own record is unambiguous that one lane per crate is what worked.
Every scheduling collision recorded in the 4a ledger — three of them — was two agents in one file.
`eval.rs`, `run.rs` and `lib.rs` are where both 4b's dispatch and 4c's call arm land.
So the honest version of "parallel" is: *sequential in `rexx-exec`, parallel only if the builtin table is given its own module that nothing else touches.*
That is a design decision worth making for the scheduling reason alone.

**What would change the recommendation.**
If D9 (`PROCEDURE EXPOSE` storage) turns out to need a `RootSet` redesign, 4b's front half stalls behind a `rexx-core` change, and the string builtins become the obvious thing to run beside it — in `rexx-core`'s case genuinely a different crate, so genuinely parallel.

---

## 4. What the 4a corpus and harnesses give 4b and 4c for free

### 4.1 Free, and immediately useful

* **`tests/corpus.rs` runs the oracle live** and is the phase's best progress instrument — it moved 3→9→12→22→26→29 during 4a and it names which owner each remaining failure belongs to (`owner_from_stderr` parses the loud message).
  It has report mode and STRICT mode (`REXX_CORPUS_GATE=1`), a memory-limited oracle wrapper, a hard failure if the oracle binary is absent (so a machine without a build cannot report a vacuous 0 of 0), and a permanent regression test proving the report reaches a plain `cargo test` without `--nocapture`.
  Reusable as-is; needs only a subset list (D6).
* **`tests/collect_stress.rs` and `run_program_collect_every_alloc`** — the collector actually runs now.
  Free for 4b, and I20 says it is precisely where 4b's new shapes are untested, so this is the highest-value inherited instrument.
* **`tests/trace_oracle.rs`'s committed-expectation pattern**, with the regeneration command in its module doc and all five expectations verified to regenerate byte-identical from the oracle.
  Six new prefixes (I14) drop straight into the same shape.
* **`error.rs`'s catalogue, spacing, clause echo and 256-major exit rule**, byte-verified on eleven programs.
  `RAISE` inherits all of it.
* **`static_indent` and `pop_search_frame`**, oracle-verified against a fourteen-shape table that is mutation-tested and dies on the right rows.
  `TRACE`'s `*-*` indentation and the error report's indentation are the same quantity.
  D2's activation base is the only thing 4b adds.
* **`rexx-inventory::builtins::NAMES`**, generated at build time from `BuiltinFunctions.cpp`'s own table, 81 entries in table order.
  4c gets its work list from the build rather than from a copied list.
* **The oracle harness discipline**: the `ulimit -v` wrapper, unpiped exit-status reading, and `git archive` for isolated builds (with the recorded exception that `rust/corpus/` is a compile-time dependency of `rexx-parse`).

### 4.2 A compile-time and assert-time obligation, not free

`coverage.rs` and `loud.rs` enumerate with **no wildcard arm anywhere**, over `InstructionKind` (40), `ExprKind` (15), `LoopKind` (6), `PrefixOp` (3), `EndStyle` (6), `Trace` (4) and `Operator` (32).
So every variant 4b or 4c implements forces four edits that cannot be skipped:

* `coverage.rs`'s owner arm → an in-scope tag, which then requires a **witness program in the subset** or `every_in_scope_variant_is_witnessed_by_the_phase_4a_subset` fails.
* `out_of_scope_set_matches_the_committed_expectation` — the out-of-scope set is pinned against a hardcoded literal.
* `variant_counts_match_the_audited_split` — 20/9/4/6/1 for `InstructionKind` and 9/6 for `ExprKind` are asserted, so the split table's own numbers move.
* `loud.rs`'s duplicate table plus `in_scope_counts_match_the_audited_split` and `assert_witness_set_is_complete`, kept in sync **by hand** (I36).

The scale: 4b removes 9 `InstructionKind` witnesses and 2 `ExprKind` ones; 4c removes 4 `InstructionKind` witnesses.
Every one of the 13 currently asserts `exit_code == NOT_IMPLEMENTED_EXIT` and will start failing the day its variant lands, which is the mechanism working — but it means `loud.rs` is edited on essentially every task, and that is worth knowing when scoping.

### 4.3 Explicitly not free, and do not promise it

* **`tests/assertions.rs`'s 35 exempt rows will not light up.**
  4,259 rows, 4,224 passing, 35 `RUNTIME-BLOCKED`, and `grep -c 'unblocked_by: "Phase 5"'` on `EXEMPT` returns 35 of 35.
  The two whose *first-observed* blocker is a 4b construct (`test_string_range` hits `xrange()`) re-block on a message send in the same prelude one line later, so implementing `Call` moves the blocker and does not unblock the row.
  A 4b or 4c plan that promises movement here is promising something measured to be impossible.
  What 4b/4c *do* owe this harness: nothing, except that `the_exempt_set_matches_the_current_blocked_rows` fails if a listed row starts passing, so any accidental improvement shows as a red test rather than silently.
* **No 4b or 4c differential subset exists.** `phase-4a.txt` is defined by its own header to contain none of the out-of-scope constructs.
* **No L1 harness for `base/keyword` or `base/bif`** (I28, D12).
* **No mutation script for 4b or 4c.** `rust/scripts/mutate-4a.sh` is nine mutations against 4a's own code, with an exit-non-zero-on-unapplied-pattern guard that fired in four separate tasks.
  The guard mechanism is reusable; the mutations are not.
* **No activation-shaped negative control for the collector.** Criterion 4's control deletes `eval_arithmetic`'s `push_temp(left_value)`.
  A 4b control must delete a root a *call* holds — the argument list between evaluation and the callee's `USE` — or it re-tests 4a and reports a pass that means nothing.
* **`rexxcps.rex` is not a differential** (D8).

---

## 5. The traps that will recur, and where

Each of these cost real time in 4a.
Each has a concrete 4b or 4c instance already identifiable today.

### 5.1 A witness implemented out from under a test — three occurrences, and the fourth is already loaded

The three, counted as the ledger counts them: the size-contract test's fixture moved from `+` to `=` for Task 7, then from `=` to a message send for Task 8 — two occurrences in one test — and `spike.rs`'s loud probe used `do i = 1 to 3` until Task 11 implemented `DO`, which is the third.
*Cite: `progress.md:1059-1068` and `2007-2012`.*

**The fourth is sitting in the tree.**
`tests/spike.rs:187` now uses `call "sub"` and asserts both `NOT_IMPLEMENTED_EXIT` and that stderr contains `"CALL"`.
Its own doc comment says "`CALL` is 4b's".
It breaks on the first day of 4b.

**The rule for 4b/4c, stated so it cannot be got wrong:** after 4b and 4c both land, the only constructs still failing loudly are Phase 5's six `InstructionKind` variants (`Expose`, `Options`, `Message`, `Guard`, `Reply`, `Forward`), Phase 5's four `ExprKind` variants (`QualifiedCall`, `ClassResolver`, `List`, `Message`), and Phase 7's `Command`.
A test needing a not-implemented fixture must use one of those, and a **message send** is the safest because it is the one the spec assigns to Phase 5 outright rather than by ruling.

### 5.2 A criterion that cannot fail — four occurrences

The four: criterion 6's predecessor satisfied by `/bin/true`; criterion 4 with no way to fail at all until it was rewritten, then with no *subject* until it was built; `strict_comparison_never_calls_to_number`, which returned before reading either operand so no result could observe the property; the CONCATENATION rows that would have passed while testing nothing.
Plus criterion 2 vacuous once, quantifying over extracted programs that executed nothing.

**Three concrete 4b/4c shapes to write the criteria against:**

* **A builtin-coverage criterion that asserts each of the 66 names "is recognised".**
  A stub returning `''` satisfies it for all 66.
  The criterion must assert a *value* per builtin, captured from the oracle, or it is `/bin/true` with 66 rows.
* **A trap criterion that asserts "the handler ran" by checking a flag the handler sets.**
  If the raise never happens the flag is unset and renders as its own derived name — which is not the expected value, so this one does fail correctly.
  The vacuous version is the opposite: asserting the *program exited 0*, which a program that never raised also does.
* **A collector criterion for 4b.**
  Criterion 4 as written passes today with zero call frames exercised (I20).
  Carrying it forward verbatim to 4b is a criterion that cannot fail *for the thing 4b adds*.
  It needs the subset union (D6) and an activation-shaped negative control (§4.3).

### 5.3 A decision recorded where the implementer will not read it — five occurrences

The sharpest is D17: the trace-emission hook was never implemented because the decision lived in the design spec and no task body carried it, so Tasks 7, 8 and 9 each built part of the dispatch loop with no event hook and Task 13 became the retrofit D17 existed to prevent.
Also: the indentation rule lived in four documents with four owners and in none of the places an implementer reads; `Plan`'s shape was specified one way in a task's Interfaces section and another way in the tree; the `ExprKind` owner split existed only as a ruling until it was written into the exclusions file.

**The pattern that works, and it is in the tree to copy:** `grow_slots`'s panic message contains the literal string `4a invariant`, and `collect.rs`'s `#[should_panic(expected = "4a invariant")]` pins the wording, so 4b cannot relax the rule without meeting the note.
That is a decision recorded *in the failure path of the code that must change*.

**Concretely at risk in 4b/4c, today:**

* **D2's measurements exist nowhere in the tree.** The `+2` indent per activation and the per-activation line number were measured for this document.
  The exclusions KNOWN GAP records only "one echo per nesting level".
  If the 4b task body does not carry them, they get rediscovered — and rediscovering an indent rule is what cost Task 11 a whole fix round.
* **The `ExprKind::Call` split (I25) is recorded in `phase-4-exclusions.txt` and in two test-file comments.** A 4c implementer reading `eval.rs` sees none of them.
* **I19's `EXIT` under-rooting pointer is on `Heap::collect` in `rexx-core`**, deliberately, because that is the function a collector-wirer has open.
  Anyone doing 4b work in `rexx-exec` will not see it.

**The rule for both plans: every one of the 37 items in section 1 must appear in the body of the task that pays for it.**
A plan that cites this document instead is repeating the D17 mechanism exactly, because the per-task brief extraction makes anything outside a task's own section invisible.

### 5.4 A measurement whose probe values cannot distinguish two hypotheses

The two named: the `EXIT` mod-256 artifact, where every probe value was a multiple of 256 so "converted, low byte 0" and "rejected, defaulted to 0" were indistinguishable, producing a confident wrong story that propagated into a dispatch; and the absorbed-`WHEN` probe where `n` stays 0, which cannot tell "never evaluated" from "evaluated and discarded" — the defect was not that the measurement was wrong, it was that the measurement could not see the difference, and nobody asked what else it was consistent with.

**Six concrete 4b/4c probe hazards:**

* **`PROCEDURE EXPOSE`.** A probe whose exposed variable holds a value equal to its own derived name cannot distinguish exposure from non-exposure, because an unexposed unset read yields the name.
  Use a value that is neither the name nor unset.
* **`PROCEDURE` isolation.** A callee setting `x = 1` where the caller also has `x = 1` cannot distinguish isolation from sharing.
* **Argument passing.** `call sub 'X'` where the callee reads `x` cannot distinguish a received argument from an unset variable's derived name.
* **`RANDOM`.** Measured 2026-08-01, two separate oracle *processes* produce the identical unseeded sequence.
  So "the values differ between runs" is not evidence of randomness, and "our `RANDOM` returned a number in range" is not evidence of parity — parity requires reproducing ooRexx's PRNG and default seed exactly (D11).
* **`DATE`/`TIME`.** Two probes inside the same second cannot distinguish a live clock read from a cached one, which matters because `TIME('R')`'s elapsed-time semantics are stateful and `rexxcps.rex` depends on them.
* **Builtin arity.** A one-argument probe (`SUBSTR('abc',2)`) cannot distinguish a builtin that honours its optional arguments from one that ignores them.
  Every builtin with optional arguments needs at least one probe per optional position, and one with a non-default pad character.

**The general form of the countermeasure, from the phase that learned it:** before accepting a measurement, ask what *else* it is consistent with, and choose probe values that make the two hypotheses produce different bytes.
The absorbed-`WHEN` case was resolved by a third probe with a *printing* absorbed branch, which separated the models by content rather than by a variable's final value.

---

## Appendix: measurements taken for this document, 2026-08-01

All against `build/bin/rexx` under `ulimit -v 1048576`.

* Clause-echo stack under `CALL`: `call sub` from line 1, raise at line 4 → `4 *-*   say 2 & 1` / `1 *-* call sub`, rc 222.
  Two levels of `CALL` plus an `INTERPRET` → indents 4, 4, 2, 0 and lines 5, 5, 3, 1.
  Same shape for a function call (`say f(1)`).
* `signal on syntax` under `trace r`: no `+++`, no error report, trap label echoed as an ordinary `*-*`, rc from the handler's own `exit`.
* `call on error name handler` with no command: handler not invoked, rc 0.
* `parse value 'a b c' with p . q` under `trace r`: `>K> "VALUE"` then three `>>>`, no `>.>` at trace level `r`.
* `call sub 1, 2` with `use arg x, y` under `trace r`: callee clauses at indent 2, arguments as `>>>`, the return value traced twice (callee indent then caller indent).
* `(1, 2)`: `~class` is `The Array class`, `~items` is 2, `length()` is 3, byte 2 is `'0a'x`.
* `random(1,100,42)` = 89; unseeded `random(1,100)` twice = 2 then 80, **identical across two separate processes**.
* `qualify('foo.txt')` returns an absolute path for a nonexistent file; `userid()` = `moritz`; `setlocal()` = `endlocal()` = 1; `stream('nosuch.txt','C','QUERY EXISTS')` = empty; `rxqueue('G')` = `SESSION` with `rxapi` running as pid 857; `gc()` = 0.
* `std::env::set_var` under `rustc --edition 2024`: `error[E0133]: call to unsafe function`.
* `samples/rexxcps.rex` read in full: prints wall-clock timings and auto-adjusts `count` from measured elapsed time.
