# Task 1 review -- the split, computed once (`b3e8cf4d2`)

Reviewed against `task-1-brief.md`, `task-1-report.md` and `review-cb4f27d1c..b3e8cf4d2.diff`.
Everything below was run by me, in detached worktrees at `b3e8cf4d2` and `cb4f27d1c` under private `CARGO_TARGET_DIR`s, with `ootest` and `rust/corpus-l1` symlinked in.
No file in `/home/moritz/dev/repos/ooRexx-rust-rewrite` was edited.

## Verdicts

* **Spec compliance: PASS.** All eight steps done, no behaviour change reachable by any instrument I could build, and Task 2's work is not encroached on.
* **Code quality: PASS WITH CHANGES.** Three comment claims are wrong or unmeasured, one of them falsified by a mutation I ran. No code change is required for any of them.

## Findings, most severe first

1. **The new unit test's doc comment claims a discrimination that is measurably false.**
   `plan.rs`: "A missing or **misaddressed** entry is not a wrong answer -- `Code::compound` falls back ... so a mutation that keys the table wrongly, or that never fills it for one of these positions, leaves every corpus program and every `tail_key` assertion green."
   The stated reason holds only for a *missing* entry. A misaddressed one returns a wrong split with no fallback at all.
   Measured: writing each entry under `(id.index() + 1) % len` in both fillers reddens **13** tests -- the new unit test plus twelve output-level ones (`run::tests::assignment_to_a_variable_a_stem_and_a_compound`, four `drop_*`, three `use_arg_alias_*`, `an_exposed_stem_aliases_the_callers_entry_not_the_object`, `the_exempt_set_matches_the_current_failures`, `both_engines_agree_on_every_case_file`).
   Fix: say "missing", and drop "misaddressed" and "keys the table wrongly". The test still earns its place -- M1/M3/M4 really are output-equivalent.

2. **Same doc comment names the size of a set, and the size is wrong.**
   "in each of the three syntactic positions a compound-shaped symbol reaches the pass through" violates the plan's own comment rule, and `PARSE VAR a.i` is a fourth: `note_parse`'s `ParseSource::Var` arm binds the whole dotted symbol, and `parse_template::read_parse_var`'s `NameShape::Compound` arm then reads it through `tail_key`. Verified running: `i = 3; a.i = 'p q'; parse var a.i x y; say x y` prints `p q` on the oracle and on both engines.
   `note_variable_ref` also covers `EXPOSE` / `PROCEDURE EXPOSE` / `USE LOCAL`, not only the `DROP` target the comment names. Name the positions the test exercises, without counting them.

3. **The case file's header claims unique coverage for rows where it was not measured.**
   "The rows a precomputed split can get wrong in a way nothing else notices" heads three bullets, but only the fragment bullet was measured that way (M9 against M9', which I reproduced). The `DO`-control bullet has a same-shape test that pre-dates this commit (`run.rs:10737` at `cb4f27d1c`, `a_compound_control_variables_tail_re_resolves_every_pass`), and no mutation was run for the changing-tail bullet at all. The plan's standing rule is: a measured witness, or a labelled transcript claiming nothing.

4. **The `INTERPRET` path got slower, and I measured what the report left unmeasured.**
   `fragment_plan` builds and discards a `compounds` table per execution, and the fragment then re-splits its own spelling at every reference -- now allocating a whole `CompoundName` where the old code borrowed slices out of the interned name.
   Measured, three interleaved runs per arm at one fixed binary path, `do i = 1 to 200000; interpret "x = i + 1"; end`: base 48.48/48.64/48.66 G, head 49.02/49.13/49.16 G, arms non-overlapping, **+1.0%**. An `INTERPRET`-free loop of the same shape moved +0.45% (codegen drift), so roughly half a percent is attributable, on a program that does nothing but `INTERPRET`. Nothing corpus-shaped sees it. Not a blocker; worth a line in the plan rather than a fix.

5. **`bind` records a split for stem-shaped spellings that nothing ever reads** (`A.` gets stem `A.` and one empty constant piece). One small allocation per stem symbol at plan-build time. Disclosed in the report; the uniformity is what makes "recording a split is a property of binding" hold, so I would leave it.

6. **The report's own `rexxcps` base figure is its high outlier.** It reports base 28,602,816,478 while its three arm-internal base runs were 28,588-28,592 M, and my two base runs are 28,588,541,742 and 28,588,546,264. The effect is -7.12%, not -7.19%. Immaterial to the conclusion.

## The five checks

**1. Behaviour preservation -- PASS.**
`tail_key` still joins with `.`, takes a constant piece verbatim, reads a variable piece's current value through `read_by_name` and renders with `to_text`; only the source of the pieces moved, and `join_tails` is entered identically by the table path and the fallback.
The "cached split equals `compound_parts`" requirement I checked directly rather than by argument: an `assert_eq!(*entry, CompoundName::split(self.symbols.name(id)))` inside `Code::compound`, whole `rexx-exec` crate, `--no-fail-fast` -- **757 passed, 0 failed, zero firings** across the corpus sweep, the `ootest` population sweep and every unit test. Negative control: inverting it to `assert_ne!` fires immediately, on `ZA.ZI` inside `both_engines_agree_on_every_case_file`, so the probe is live and not vacuous.
Gates in my worktree: `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0 after wiping `.fingerprint/rexx-exec-*` so the crate was genuinely re-linted; `memcap 8G cargo test --workspace --no-fail-fast` **1477 passed, 0 failed, 4 ignored** (my first run showed one failure, `rexx-bench`'s `every_blocked_axis_still_fails_on_this_crate`, which wants `target/{debug,release}/rexx-run` at a path relative to `rust/` -- an artifact of `CARGO_TARGET_DIR`, green once symlinked).
`const _: () = assert!(size_of::<Op>() == 16)` is untouched at `ir/mod.rs:83`.

**2. The `INTERPRET` fragment stanza -- PASS, and the stanza does exactly what it claims.**
I built the mutation myself: the fragment's `Code` carries `Some(enclosing plan)`, with `printed_indent` pinned to its fallback so only the compound half moves. Result over the whole crate: **exactly one** test red, `both_engines_agree_on_every_case_file`, failing at `tests/ir_dual_cases/compound-names:170` -- stanza 9 -- with actual
`ZB.5 / in / ZC.1 / ZC.2 / 2` at **rc 0**, no panic.
That is the in-range, quiet case, not an out-of-range one: every line wrong and the program still exits 0. Stanza 8 (the `INTERPRET` in a loop, whose enclosing first symbol `ZI` is not compound) stays green under the same mutation, which is precisely why stanza 9 had to be arranged. The claim in the report is reproduced byte for byte.

**3. The `DO`-control caller and the no-slots half -- PASS, both halves verified.**
`bind` now records `CompoundName::split(name)` for any bound spelling containing `.`, and assigns **no** slot to the stem or to any piece: the only `slot_for` call is the one that was always there for the whole dotted name. So an entry does **not** imply its pieces have slots, and Task 2 must not assume it does -- the plan file already carries that amendment.
The reachability claim is sound: `note_loop` binds a controlled control, an `OVER` control and a `WITH` index/item, and `note_parse` binds a `PARSE VAR` source, all of which can be compound-shaped and all of which reach `tail_key`. Fixing at `bind` covers the `PARSE VAR` case the report never names.

**4. Scope -- PASS.**
Six sites now take `Code::stem_name`: `eval.rs` read and echo (the brief's two), `run.rs` `assign_expr_target`, the controlled-loop step and `drop_variable`, and `parse_template.rs` `read_parse_var`. Each is the same one-line substitution of the stem's source; the surrounding `stem_get`/`stem_set`/`trace_compound_name` calls and their ordering are unchanged.
The write path is equivalent: `assign_expr_target` still takes an owned stem name (it is extended into `resolved`), still calls `stem_set(&stem_name, &key, value)` before the trace lines, in the same order. It is exercised by every stanza and by the corpus.
After this commit `compound_parts` survives in `rexx-exec` only as `CompoundName::split`'s body and `Code::stem_name`'s fallback, so no hot caller was left behind and none was added.

**5. The `Vec` choice -- PASS, and the waste bound is honest.**
Density verified in `rexx-parse/src/token.rs` myself: `intern` assigns `SymbolId(names.len())` before pushing and returns an existing id for a re-spelling, and `name` indexes `names[id.0]` directly, so ids are dense and zero-based per table. `SymbolId::index`'s doc comment already stated the guarantee and already warned about the quiet cross-table case, which is the hazard stanza 9 pins.
The waste: sized `symbols.len()` per body, and a table is per program, so the bound is symbols times bodies. The worst-file arithmetic checks out -- `samples/windows/ole/apps/MSAccessDemo_32bit_only.rex` has ten `::ROUTINE`s plus main, and a crude upper bound on its distinct symbols is 242 against the reported 204, so 204 x 11 = 2,244 entries is the right order, and 2,244 x 32 B = 71,808 B is the claimed 72 KB. The report says plainly that nothing in the code bounds a larger program, which is the honest way to leave it.

## The three concerns, adjudicated

* **"No test catches that the table is never consulted" -- correct, and I confirmed it.** With `Code::compound` returning `None` unconditionally, the whole `rexx-exec` crate is **757 passed, 0 failed**. That is the right outcome rather than a gap: the fallback is required to agree, so no output comparison can see the difference. The instruction counter is the instrument, and it does see it -- see below. The report is right not to claim a test here.
* **The killed M7 run -- accepted as a lower bound, and it costs nothing.** M7 (`Code::stem_name` returns the whole spelling) is a mutation whose catchers were already 21 before the run hung; no "only" claim rests on it, and the four claims that do carry "only" were re-run at the commit's final state. Recording it as aborted is the right disposition. Worth knowing for future runs: a wrong stem name can send a corpus program non-terminating, so that mutation wants a per-test timeout rather than a whole-suite one.
* **The per-`INTERPRET` discarded table -- real, small, and now measured** (finding 4): +1.0% on an `INTERPRET`-saturated loop against +0.45% drift on the same loop without `INTERPRET`. It does not touch anything corpus-shaped. If it is ever worth removing, the cheap form is a build mode that skips `compounds` for a fragment plan, since `fragment_plan` keeps only the id-to-slot translation anyway.

## Measurement, reproduced

`perf stat -e instructions:u`, `REXX_ENGINE=ir`, both arms staged at one fixed path (`.../bench/armbinary`), run from a fresh empty directory, interleaved.

| axis | base `cb4f27d1c` | head `b3e8cf4d2` | difference | report |
|---|---:|---:|---:|---:|
| `rexxcps` | 28,588,541,742 / 28,588,546,264 | 26,552,512,331 / 26,552,501,431 / 26,550,130,679 | **-7.12%** | -7.19% |
| `compound` | 37,163,093,394 | 30,589,529,208 | **-17.69%** | -17.70% |

Both `rexxcps` arms self-calibrated to `100 x 100`, so both did the same work. `compound`'s stdout is byte-identical between the arms. The direction, the magnitude and the two axes that move are all as reported.

## Runs behind this review

* Oracle, fresh empty directory, absolute paths, `ulimit -v 1048576`: all ten stanzas of `tests/ir_dual_cases/compound-names` re-captured. **Every expected block matches byte for byte, rc 0, empty stderr.**
* Whole workspace at head: 1477 passed, 0 failed, 4 ignored. `fmt` and `clippy` exit 0.
* Four mutations, whole `rexx-exec` crate, `--no-fail-fast`: fragment-carries-enclosing-plan with the indent half pinned (1 red, stanza 9); entry misaddressed by one (13 red); `Code::compound` always `None` (0 red); cached-split-equals-fresh-split probe (0 red, live under a negative control).
