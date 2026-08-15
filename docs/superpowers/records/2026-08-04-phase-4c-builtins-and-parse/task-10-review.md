# Task 10 review: `builtin/state.rs`

Reviewed at `f03d69f1` against `task-10-brief.md` (its own steps plus the appended shared builtin block) and `task-10-report.md`.
Every oracle run below was wrapped as `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx ABS )` from a fresh empty scratchpad subdirectory, with stdout, stderr and exit status read separately.
`git status --porcelain` is empty at the end of this review; both mutations were restored from `cp` backups, not from git.

## Verdict 1 -- spec compliance

| Requirement | Verdict |
|---|---|
| Step 0a: `ADDRESS()` corpus witness -- swap at least three deep | ✅ `address_env.rex` block `D` is four toggles deep (`D1`..`D5`) |
| Step 0a: inheritance into a callee, **both halves** | ✅ block `E`, two named environments, neither the default |
| Step 0a: decide what `None` renders as, reason at the decision point | ✅ `DEFAULT_ENVIRONMENT`'s doc; but see Important-4 for a wrong C++ citation inside it |
| Step 1: probe table across `ADDRESS ARG GC TRACE QUEUED`, top level *and* routine | ✅ report §Measurements; `GC()`=0 and `QUEUED` single-program confirmed here |
| Step 0: `CONDITION()` bare form is `'I'` not `'C'` | ✅ `builtin/mod.rs` row comment + `condition_is_empty_outside_a_handler_and_defaults_to_the_instruction` |
| Step 0: `A`'s type varies; `D` empty for SYNTAX; `C` is two words | ✅ (`A` loud rather than answered -- declared, see below) |
| Step 0: outside a handler the options are not uniform (`A`/`O` are `.NIL`) | ⚠️ partially -- `A`/`O` are loud in *both* positions, so the non-uniformity is not reproduced, only refused |
| Step 0: carry both `I` and `S` unless you can show otherwise | ✅ **refutation reproduced**, see below |
| Step 2: add the trap-kind state where the trap fires; probe from inside a live handler | ✅ `TrappedCondition`, written in `offer_to_trap` and `deliver_pending_trap` |
| Step 3: read each `<NAME>.testGroup`, **write failing tests, then implement** | ⚠️ Cannot verify from diff (ordering is not observable in one commit) |
| Step 4: all 11 `implemented`, none `divergent` | ✅ `every_state_builtin_is_implemented` + the pre-existing divergent-is-empty rule |
| Step 5: shared verify block | ✅ (controller-confirmed: 1181/0, gate 48 of 48, status 60/6/15/81) |
| Shared: reuse `not_enough_arguments`/`too_many_arguments`, no second pair | ✅ |
| Shared: probe each optional position with an interior omission, raise 40.5 where the oracle does | ✅ `arg(,'')` -> 40.5 confirmed on the oracle |
| Shared: every probe alphabet includes a byte >= 0x80, a control byte, the null string | ✅ as reported |
| Shared: never render through `from_utf8_lossy` on a compared path | ✅ |
| Shared: enumerate branches from the C++ | ✅ |
| Shared: every allocation through `Interp::alloc_with` | ✅ no `Heap::alloc*` in the new code |
| Shared: **measure whether a 40.x message substitutes the rendered value or the source spelling** | ❌ **not measured for the three new raisers, and the choice taken is wrong** -- Critical-1 |
| Shared: D15 -- at least one probe per numeric builtin where `DIGITS`/`FORM` moves between creation and rendering | ❌ absent; the missing probe is exactly the one that exposes Critical-1 |
| Shared: cross the axes, do not vary one at a time | ❌ the 60-case sweep's alphabet carries `1.0`, `99.0`, `-1`, `0`, `100` -- non-canonical renderings and out-of-range positions are both present but never crossed |

## Verdict 2 -- task quality

### Critical

**C1. Three new range raisers substitute the rendered value where the oracle substitutes the converted whole number.**
`rendered()` (`state.rs`) feeds `argument_not_positive` (40.14), `sourceline_out_of_range` (40.34) and `argument_out_of_range` (40.903) with `interp.to_text(value)`.
The oracle substitutes the *integer it converted to*. Seven measured witnesses, all rc 216 on both sides, stdout identical, **stderr differs**:

```
say arg(0.0)          oracle found "0"        rexx-exec found "0.0"
say arg(-1.0)         oracle found "-1"       rexx-exec found "-1.0"
say sourceline(0.0)   oracle found "0"        rexx-exec found "0.0"
say sourceline(-1.0)  oracle found "-1"       rexx-exec found "-1.0"
say sourceline(1e1)   oracle ("10")           rexx-exec ("1E1")
say errortext(-1.0)   oracle found "-1"       rexx-exec found "-1.0"
say errortext(1e2)    oracle found "100"      rexx-exec found "1E2"
say errortext('100.0')oracle found "100"      rexx-exec found "100.0"
```

and the D15 crossing the shared block asked for:

```
numeric digits 3
say errortext(999999 + 1)
   oracle    Error 40.903: ... found "1000000".
   rexx-exec Error 40.903: ... found "1.00E+6".
```

`rendered()`'s own doc states the opposite as a settled choice -- "The *rendered value*, which is the same choice `argument_not_whole` makes" -- and that justification does not transfer: `argument_not_whole` (40.12) fires on a value that *never converted*, so rendered and converted coincide there and it is measured correct (`errortext(100.4)` -> `found "100.4"` on both sides).
The shared block explicitly assigned this measurement to "the first family task that raises a typed error", and this is that task.
The divergence is not declared in `phase-4-exclusions.txt` (searched with `/bin/grep -a` for `40.903`, `40.14`, `40.34`, `rendered value`, `source spelling` -- zero hits), so it is an undeclared wrong answer, the failure mode `divergent` exists to make expensive.
Fix is one substitution per site: pass `number.to_string()` (the already-converted `i64`) instead of `rendered(...)`. All eight measured rows above are then satisfied.

### Important

**I1. The "latent bug" the delay change is credited with fixing does not exist, and three places in the tree say it does.**
`Trap::delayed`'s doc: "it also resurrects a trap the handler turned off, which the oracle does not."
`deliver_pending_trap`'s inline comment: "Setting a flag also leaves a handler's own `CALL OFF` alone, where removing and re-inserting put the trap back."
The commit message and the report repeat it.
A `CALL ON` handler runs as a *nested activation* that inherits `traps` by copy and never writes back, so a `CALL OFF` inside the handler cannot reach the trap the caller holds -- there is nothing for a re-insertion to resurrect.
Measured on the oracle (handler turns its own trap off, caller raises the same condition again): the handler is entered **twice**, i.e. the trap survives.
Then measured by mutation: I restored `deliver_pending_trap` to remove-and-re-insert (`cp` backup, restored byte-identically) and the same program printed **identical output**; only `condition('S')` moved, `DELAY` -> `OFF`.
So the delay change is justified by `CONDITION('S')` alone, which is true and sufficient; the second justification is a sound-looking inference that was never run, the exact shape `rust/CLAUDE.md` records six prior instances of.
The neighbouring sentence -- "`trapUndelay` ... is a no-op when the handler turned its own trap off -- the C++ tests the handler for null before enabling it" -- describes what the C++ code does (fine) but attaches it to a scenario that cannot occur (not fine).

**I2. Two doc comments in `run.rs` still describe the mechanism this task deleted.**
`run.rs:2783-2784` (on `deliver_pending_trap` itself): "The trap is removed for the handler's duration and **put back afterwards**" -- it is delayed and undelayed, and a comment 60 lines below in the same function now says "**Delayed, not removed**". The two contradict each other.
`run.rs:12517-12519` (on `a_call_trap_is_put_back_after_its_handler_returns`): "A `CALL ON` trap is removed for its handler's duration and **put back** afterwards ... deleting the re-insertion left the whole suite and the corpus gate green" -- there is no re-insertion to delete any more.
The repo rule is that a false comment must be corrected or removed, not left.

**I3. `corpus/keyword-exempt.txt`'s header now makes a false claim about `CALL::test_on_name`.**
The rewritten bullet says the three `CALL` bodies "fail under the C++ oracle itself (Error 43, Routine not found) because they call ::routines the standalone program does not carry -- measured: routine "label", routine """ -- three bodies, two names.
The harness's own breakdown, run here (`cargo test -p rexx-exec --test keyword_assertions -- --nocapture`), lists `routine "CHARIN"  1` under "first construct hit", and `CALL::test_on_name`'s body is `call on notready name WORDS; call charin "/"`.
Its first blocker is the excluded `CHARIN` builtin, not a missing `::routine`.
The pre-edit sentence carried four names for four bodies (`"label"`, `""`, `"CHARIN"`, `"DIGITS"`); the correction dropped `"DIGITS"` correctly and `"CHARIN"` wrongly.
This is the recorded "a correction round introduces a new false statement" pattern, in a file whose whole purpose is to be trustworthy prose beside a derived table.

**I4. The C++ citation for `DEFAULT_ENVIRONMENT` is wrong, in three copies, and the wrongness is the load-bearing part of the argument.**
`state.rs`'s doc: "The oracle's is `SYSINITIALADDRESS`, a `#define` in `platform/unix/SystemCommands.cpp:68` returned by `SystemInterpreter::getDefaultAddressName()`; the Windows build's own file returns `GlobalNames::INITIALADDRESS` instead."
Read in the C++ tree: `interpreter/platform/unix/SystemCommands.cpp:197` and `interpreter/platform/windows/SystemCommands.cpp:209` are byte-for-byte the same function, **both** `return GlobalNames::INITIALADDRESS;`.
`interpreter/memory/GlobalNames.h:124` is `GLOBAL_NAME(INITIALADDRESS, SYSINITIALADDRESS)` with its own comment "this is defined in the platform definitions", and the platform difference lives in `PlatformDefinitions.h` -- `:66` `"sh"` on unix, `:65` `"CMD"` on windows.
The `#define` at `SystemCommands.cpp:68` is a second, local definition in that translation unit and is not the one the returned name comes from.
The same false contrast is in `phase-4-exclusions.txt`'s rewritten `ADDRESS()` row and in the report.
The *values* are right (`sh` measured, `CMD` correct at `PlatformDefinitions.h:65`), and the `CMD` arm is genuinely stronger evidence than the report's "read from `GlobalNames::INITIALADDRESS`" claims -- the literal is in the Windows platform header. So this is a citation defect, not a behaviour defect, but it is the sentence the whole "this is a `LINUX`, not a build date" decision rests on.

### Minor

**M1. `Loud::builtin_option_object`'s doc contract is false for one of its four uses.**
"A builtin's option letter whose answer is an object this crate's value model cannot make" -- `CONDITION('D')` for `NOVALUE` answers the variable's derived name, a plain string. The emitted message is fine; the contract at the top of the function is not.

**M2. `caller_trap_for` does not filter `delayed`, while the comment beside `Trap::delayed` says every such lookup does.**
"`Interp::trap_for` is what enforces that, so every lookup that decides whether to trap sees the same thing a removal used to show it" -- `caller_trap_for` (`run.rs:2943`) is a second lookup that decides whether to queue a pending trap and it has no `.filter(|trap| !trap.delayed)`.
I could not make it observable: a handler calling a routine that raises the same condition, and the same with an `ANY` trap behind the delayed one, both match the oracle byte for byte, because `deliver_pending_trap` re-checks through `trap_for` and drops the pending trap.
So this is a redundancy that currently costs nothing, but the sentence claiming the invariant is held everywhere is not true of the code.

**M3. Admitting `RAISED` as an exempt attribution weakens the exempt gate with prose as its only control.**
`every_exempt_attribution_is_a_known_phase_or_a_declared_defect` now accepts a token that means "exited non-zero for a reason the harness cannot name".
For `NUMERIC::test_42` the cause was chased and the oracle exits 3 on the same extracted program, which is the right answer -- but nothing in the test requires that check for the *next* `RAISED` row, where the cause could be a real divergence.
Compare `divergent` in `builtin-status.txt`, which requires a `KNOWN GAP:` marker. The header bullet ("A row carrying it needs its cause written into this header") is the only enforcement, and headers are not run.

**M4. `trace('?')` -- the builtin form -- also diverges, and the rewritten `TRACE ?` row lists only instruction forms.**
Measured: `say '['trace('?')']['trace()']'` is `[N][?N]` on the oracle and `[N][O]` here.
The row records `trace ?r` and `trace l ; trace ?`; the builtin-as-setter spelling is a third witness of the same gap and belongs in the same row now that `TRACE()` reads back.

**M5. Stale row counts for `keyword-exempt.txt` outside the diff.**
`rust/CLAUDE.md:57` and `crates/rexx-exec/tests/builtin_status.rs:26` both say "796-row"; the file now holds 16 rows (`/bin/grep -av '^#' | /bin/grep -avc '^$'`).
Both were already wrong before this commit (111 rows), so this task did not introduce them -- but it is the commit that moved the number again and it is the same "one copy corrected, the others silently wrong" shape.

**M6. `corpus/README.md` gained no row for `state_builtins.rex`.**
Its "Phase 4c additions" table lists three programs; `address_env.rex` (Task 9) is also absent, so this follows an existing gap rather than creating one. Nothing polices the table, which is why it drifts.

### What I checked and found sound

* **The `I`/`S` refutation is real.** `c6.rex` inside a `SIGNAL ON SYNTAX` handler: `I SIGNAL / S OFF`, then `signal on syntax name h` -> `I SIGNAL / S ON`, then `signal off syntax` -> `I SIGNAL / S OFF`. `c7.rex` inside a `CALL ON USER UC` handler: `S DELAY / I CALL`, re-arm -> `S ON / I CALL`, `call off` -> `S OFF / I CALL`, `signal on` -> `S ON / I CALL`. The crate reproduces all eight rows byte for byte. Carrying both fields, one stored and one looked up live, is correct.
* **M4 reproduced exactly.** Mutating `resolve_and_run_call` to `AddressState { current: caller.address.current.clone(), alternate: None }` and running `REXX_CORPUS_GATE=1 cargo test --offline --workspace --no-fail-fast`: **72 `Running`/`Doc-tests` process headers** (the documented baseline, so not truncated), exit 101, exactly one failing test in the whole workspace -- `corpus_differential`, 47 of 48, `lang/address_env.rex` stdout `E3 sub sh` against the oracle's `E3 sub ENVE1`. The debt Step 0a names is discharged and the "caught by nothing else" claim holds.
* **Both divergences found by probing check out on the oracle**, and both are pinned. `arg('x','q')` 40.12, `arg(,'')` 40.5, `arg(0,'')` 40.14, `arg(1,'')` 40.904 -- all four rc 216, and `args_three_checks_run_in_the_oracles_order` asserts each, including the `arg(0,'')`/`arg(1,'')` pair that places the emptiness check on the switch. `say trace('5')` is 24.1 rc 232 and `trace value 5` is 24.901 rc 232, `trace('-3')` reports `found "-"`, and `a_digit_string_is_a_bad_letter_to_the_builtin_and_a_skip_count_to_the_instruction` pins all three.
* **The two rewritten `phase-4-exclusions.txt` claims about `TRACE()` are true.** `trace ?r ; say trace()` is `?R` on the oracle and `R` here; `trace l ; trace ? ; say trace()` is `?L` against `O`. The deviations note is right that `TRACE()` is readable again.
* **Leaving the `?` prefix uncarried is a defensible call.** Carrying it in the rendering would fix `trace ?r` outright and would leave `trace ?` as `?O` against `?L` -- so nothing is *hidden* in the corpus sense. What the row's argument does buy is that `trace()` never reports an interactive mode whose interactive behaviour (the banner, the stdin drain) is absent, and that half of the gap has an owner already. I would not ask for the change.
* **`sh` survives the corpus determinism rule.** `corpus/README.md`'s rule is byte-identical output "on every run of the *same* interpreter"; a compile-time platform constant satisfies it, and the host-dependence is flagged in `address_env.rex`'s header and in `phase-4c.txt` exactly as `LINUX COMMAND` is. The reason is written at the decision point (`DEFAULT_ENVIRONMENT`'s doc), modulo I4.
* **The four loud sub-cases are consistent with the status row, and they are asserted, not merely described.** `builtin_status.rs`'s module doc defines `implemented` as "stdout, stderr and exit status all matched the oracle on **this name's probe**", and the `ARG`/`CONDITION` probes (`call f 7,8; ... say arg(2)` and `signal on syntax ... say condition('C')`) do not reach the loud shapes. A regression in them is caught by `the_object_valued_options_are_loud` (three cases, exit 120 plus message text) and `only_novalues_description_is_loud` (with its adjacent success). Deleting either subject turns those red.
* **The "value model cannot construct it" claims are true.** `/bin/grep -a` for `Body::Array` finds only `rexx-core`'s own tests and benches plus `stem.rs`'s name-for-diagnostics arm -- nothing in the interpreter constructs one -- and `value.rs:178` is `other => unreachable!(...)`. No `Directory` type exists anywhere in `rexx-core` or `rexx-exec`.
* **`code_sub: None` for every `CALL ON`-reachable condition is true, and I ran the premise the comment leaves out.** The comment argues from `call on syntax` being a translation-time 25.1, but `trap_for` falls back to an `ANY` trap, so `call on any` was the missing route. Measured: `call on any name h` with `say 1/0`, and again with `raise syntax 40.4 return 1`, does **not** enter the handler on the oracle, and the crate agrees byte for byte on both. The inference holds; it is stated in the code as a conclusion rather than flagged as one, which is the only thing I would change.
* **`GC('Force')` collecting for real is safe here.** Builtin arguments are rooted by `resolve_and_run_call`'s `push_temp(argument.value())` before dispatch, `optional_string` has already copied the option to an owned `Vec<u8>`, and the one documented under-rooted window in `heap.rs` (`EXIT`'s result between `pop_frame` and `exit_code_for`) is not reachable from a builtin. The test compares `collections_performed()` across the call rather than the return value against itself.
* **Counts spot-checked with `/bin/grep -a`**: 21 `#[test]` in `state.rs`, 11 `run: state::` rows in `builtin/mod.rs`, 11 `STATE_FAMILY` entries, 16 data rows in `keyword-exempt.txt`, and the harness's own `880 of 896 bodies ... 1710 of 1773 assertSame calls`. All match the report.
* **A 28-case differential sweep of my own** over the eleven names (option-letter case and word forms, the null option, non-canonical numeric renderings, boundary positions, `1e2`/`1.0`/`2000000000000`, `sourceline` past the end, a file with no trailing newline, `TRACE()` inheritance into a callee, `NOVALUE`/`HALT`/`NOTREADY` handlers, queue interleaving) found exactly the C1 family and the declared `?` gap. Everything else matched on all three descriptors.

### Cannot verify from diff

* ⚠️ Step 3's "write failing tests, then implement" ordering -- one commit carries no evidence of it.
* ⚠️ The report's 60-program differential sweep (57 identical / 3 differ) -- the sweep is not committed, so only its conclusion is checkable, and my own sweep contradicts it on the C1 family (their stated alphabet contains `1.0` and `99.0` but crosses neither with an out-of-range position).
* ⚠️ M1, M2, M3, M5, M6, M6b, M7, M8 process counts -- I reproduced M4 only. M6b's corpus claim is however deducible from `state_builtins.rex`'s `F1` line, which prints `condition('S')` inside a `CALL ON` handler and would read `OFF` under the removal mutation.
