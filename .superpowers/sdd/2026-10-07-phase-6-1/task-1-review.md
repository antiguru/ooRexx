# Task 1 review (spec + quality), e6af1198b..e77576cd0

### Spec Compliance

- ❌ One requirement is not met. The brief (Step 3) says "`SELECT CASE` stores its value on the activation for the construct's duration". A `SELECT CASE` typed at a debug pause overwrites the value of the construct that is still running (Important 1). Everything else in the brief checks out against the diff:
  - **Interfaces:**
    - `cached_clock` and `elapsed_anchor` are `i64` with `NO_CLOCK` (`activation.rs:511-515`, `:612`).
    - `ELAPSED_RESET` and `DEBUG_PAUSE` are bits in `ActivationFlags` (`:558-599`).
    - `current_case_text` is held inline (`:518`).
    - `ActivationCold` holds `condition`, `active_condition`, `random_seed` and `locals` (`:616-630`).
    - `random_source` is on `Activity` (`activity.rs:1217-1219`), and `top_level_activation_mut` exists (`activation.rs:924-939`).
  - **Rules:**
    - `nested` copies the anchor, the stale cached clock, the reset flag and both conditions (`:697-726`, `run/call.rs:1962-1995`).
    - `routine`, `method` and `new` start fresh.
    - The `CALL ON` handler inherits the trapped condition as its `active_condition` (`run/condition.rs:2107-2115`).
    - P89's copy lines are deleted (`ir/drive.rs:1821-1823`).
    - The P89 test is rewritten to the oracle's rule (`scheduler/tests.rs:2434`).
  - **Witnesses:** every scout B probe the brief names is present, plus RF1 (a) to (e), RF2 at depth 2000 and the unseeded-RANDOM predicate.
    - `phase-6-1.txt` is registered in all five harness lists.
    - `oracle-crashes.txt` 10b is extended.
    - The gate record is at `docs/superpowers/plans/phase-6-1-gate.md`.
  - **Size:** `size_of::<Activation>() == 480` (`activation.rs:552`), per the controller ruling.
  - Line numbers above are line numbers in the review diff file, not in the source files.
- ⚠️ Cannot verify from the diff:
  - **`refusal-sites.tsv`:** the report says it was re-derived twice and that `trace/format.rs` ends at its base content. Neither file is in the diff stat, so the net change is zero, which is consistent. The controller may want `git diff e6af1198b..e77576cd0 -- rust/corpus/refusal-sites.tsv` to show empty.
  - **Gate figures:** the perf and test numbers are taken from the report. I confirmed only that `/tmp/claude-1000/p61/t1/bin/r1c/rexx-run`'s sha256 (`b44aaf39...`) matches the gate record's `r1`.

### Strengths

- **Termination restore:** it goes through `pop_activation` alone, and every activation end passes through it on both the `Ok` and `Err` paths (`lib.rs:2477`, `run/call.rs:1060`, `install.rs:1187`; `grep -rn pop_activation` names no other site). The REPLY skip is correct because `release_method_activation` takes `replied` before the box moves (`dispatch.rs:3256`). The continuation's own end therefore restores, and the original activity's pop does not.
- **Cold box:** it is cheap on the plain call path. `ActivationCold::inherited` allocates only when the caller holds a condition, and the restore is out of line with `#[cold]` and `#[inline(never)]`.
- **RANDOM:** `next_seed` reproduces the oracle's `RexxActivation::getRandomSeed` (`RexxActivation.cpp:3453-3485`: seed flip, 13 scrambles, one advance). Lazy seeding from the activity generator matches spec section 2.
- **Stale prose:** I searched the crate and the spec for the old field paths (`Activity::elapsed_anchor`, `activity.locals`, `pending_elapsed_reset`, `Activity::debug_pause`, `activity.active_condition`, `activity.random_seed`). Nothing is left outside the spec's own D1 row and field table.

### Named-risk checks

1. **Fields built without the rule** (REPLY continuation, started activity, `CALL ON` handler):
   - The `CALL ON` handler goes through `resolve_and_run_call`, which reaches `Activation::nested` (`run/call.rs:902`) with the cold box carrying the trapped condition. A label is the only target in the oracle (`internalCallTrap`, `RexxActivation.cpp:3328-3344`).
   - A started activity uses the only `Activation::method` site (`dispatch.rs:3171`), so it starts fresh.
   - The continuation moves the box. Its new `Activity` starts with `random_source: None`, which is correct.
   - No defect found.
2. **RANDOM and SETLOCAL from INTERPRET and typed lines.** Both run in the current activation, so the walk resolves as `isInternalLevelCall` does. Probes are in `/tmp/claude-1000/p61/t1r/`, run with `bash /tmp/claude-1000/p61/t1/orc.sh {oracle|/tmp/claude-1000/p61/t1/bin/r1c/rexx-run} /tmp/claude-1000/p61/t1r/DIR p.rex [stdin]`:
   - `interp/`: seeded stream through an `INTERPRET` inside an internal call, and `interpret 'call setlocal'` in a routine. Both engines print `main 459 / s 881 / main 444 / main sees orig`, rc 0.
   - `dbg/`: `say "typed" random(1,1000)` at a `trace ?r` pause. Stdout (`main 459 / typed 881 / main 444`) and stderr are identical, rc 0.
3. **Termination restore on every ending path.** The oracle's frames are all terminated: `popStackFrame` calls `cleanupStackFrame`, which calls `termination` (`Activity.cpp:1683-1686`), and `unwindToFrame` pops through it. Ours, read above, then probed:
   - `unwind/`: a routine leaves `SETLOCAL` open and fails with an untrapped 42.3, which main traps. Both print `main sees orig SYNTAX`, rc 0.
   - `replyend/`: oracle and ours are identical 5 of 5: `got 1 / cont method / main sees orig`.
   - `replyend2/`: a method runs `SETLOCAL`, replies, sleeps 0.1, then ends. Oracle and ours are identical 5 of 5: `got 1 / main early method / cont method / main late orig`. The restore happens at the continuation's end, not at the original activity's pop.
   - No corpus or crate witness covers these two paths (Minor 2).

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

1. **A typed debug line overwrites the running construct's `SELECT CASE` value** (`run/select.rs` `open_select_case` writes `activation_mut().current_case_text`, and the absorbed-`WhenCase` arm in `run.rs` reads it). This also falsifies report concern 2, "I found no program that can read a stale value".
   - **Probe:** `/tmp/claude-1000/p61/t1r/casedbg/` is a `trace ?r` program with `select case 'a'` and `when 'a' then when 'zz' then say 'absorbed zz'`. At the pause after `when 'a'`, the typed line is `select case "zz"; when "zz" then nop; end`.
   - **Oracle:** the absorbed `when 'zz'` traces `>>> "0"` and prints `other` and `done`, rc 0.
   - **Ours (r1):** it traces `>>> "1"` and prints `done` only, rc 0.
   - **Why:** the oracle runs the typed line in its own activation with its own `DoBlock` stack (`RexxActivation.cpp:2786-2802`; the case value is read from `topBlockInstruction()`, `WhenCaseInstruction.cpp:146-148`). Ours runs it as a fragment in the same activation, and the fragment's `open_select_case` writes the field the outer construct's absorbed WHEN reads.
   - **Regression check:** the base binary (`/tmp/claude-1000/p61/t1/bin/base/rexx-run`) prints the same wrong `done`, so this is not a regression. Spec section 2's table says "shared" for this cell (INTERPRET, typed debug line), so the spec is wrong for this one field.
   - **Fix:** save `current_case_text` before `run_fragment` in `run_debug_fragment` (`run/interpret.rs:255-276`) and restore it after. This is a cold path. Add the probe as a corpus witness with a `.stdin` file. An INTERPRET instruction cannot sit between a WHEN and its absorbed WHEN, so the typed line is the only route I found.

#### Minor (Nice to Have)

1. **Inaccurate field doc** (`activation.rs`, the doc on `Activation::cold`). "only once a handler, `RANDOM` or `SETLOCAL` has run in it" is false for an internal call made from inside a handler. That callee gets a box through `ActivationCold::inherited` with no handler having run in it. Delete the qualifier or state the rule.
2. **No witness for two restore paths:** the termination restore on a SYNTAX unwind and at a REPLY continuation's end. The `unwind/` and `replyend2/` probes above are ready-made; each restores once per process.
3. **Double walk in `next_seed`** (`builtin/numeric.rs`). It calls `top_level_activation_mut()` twice, so internal-call recursion pays two O(depth) walks per RANDOM. One `&mut` lookup is enough.
4. **`end_cold` restore idiom** (`activation.rs`). `swap_remove(0)` followed by `clear()` reads as if order matters. `std::mem::take(&mut cold.locals).into_iter().next()` states the intent.
5. **INTERPRET inside a typed debug line** (not probed). Here it inherits `DEBUG_PAUSE` from the shared activation. In the oracle the INTERPRET activation's `debugPause` is false, because only the `DEBUGPAUSE` context sets it (`RexxActivation.cpp:209-213`). This was activity-wide before the change, so it is not a regression. It may belong with Task 5 or 6.

### Assessment

**Task quality:** Needs fixes

**Reasoning:** The scope move is correct on every path I checked, including the named risks, and four oracle probes agree byte for byte. The rule "for the construct's duration" fails in one concrete shape: a `SELECT CASE` typed at a debug pause. The fix is small and on a cold path, and it needs a witness.

Note: `orc.sh` creates its run directories under `/tmp/claude-1000/p61/t1/runs/` (the implementer's scratch), so my probe runs left directories there.
