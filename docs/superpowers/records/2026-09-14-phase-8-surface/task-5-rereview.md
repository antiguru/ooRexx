# Task 5 fix round 1: re-review (e1c9d32a0)

Reviewer: rereview-s5. Read-only. `$R` = `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/rereview-s5`.
Built from `git archive e1c9d32a0` (touched; 20 Compiling lines, `$R/build.txt`) into `$R/target`.
`$R/cmp.sh` runs the oracle (`ulimit -v 1048576`) and the crate, each from a fresh `mktemp -d`, and compares stdout, stderr and rc separately. Outputs are in `$R/out/`.
My probe extension is `$R/rr.cpp` plus `$R/rrc.c`, built `g++ -shared -fPIC -O1 -static-libstdc++`. Its NEEDED entries are `libgcc_s`, `libc` and `ld-linux`, and it has no undefined Rexx symbol.
The probes are `$R/p/*.rex`. Known oracle crashers (entries 16-18) were run on the crate only.

## Verdict

**Every review finding is addressed. Two new Important findings, three Minor.**

## Priority 1: the Throw* unwind

Holding:
* **Scope of `C-unwind`.** Outside `#[cfg(test)]`, `C-unwind` appears only in the five `throw_*` generics
  (`ffi.rs:938-991`), the `unwinds` arm of `entry_type!`/`entry_stub!` (`layout.rs:474`, `:515`, used by the
  15 `Throw*` members only), and `Stub<C>` (`load.rs:42`). The other 175 `extern "C" fn` in `ffi.rs` are unchanged.
* **Planted panic in a non-Throw slot.** `$R/mut/rexx-run-mut` with `RR_PANIC`: `get_context_digits` panics,
  giving rc 134 "panic in a function that cannot unwind" (`p/digits.rex`).
* **Planted foreign payload.** With `RR_PAYLOAD`, `throw_exception0` resumes a `&str` payload. It is never
  trapped: at top level it gives rc 101 (`p/rethrow.rex`). Nested under `SendMessage0` it gives rc 134 at that
  slot's `extern "C"` frame (`p/nested.rex`). Under a swallowing `catch(...)` it gives rc 134.
* **Behaviour matches the oracle.** These are identical to the oracle: `throw.rex` (destructors,
  `trapped 40 40.1`, USER via CALL ON and SIGNAL ON), the 13 `library_callback_*` witnesses (including
  `_throw`: CONTINUE stays unset, the METHOD and FUNCTION shapes match) and `thr.rex`/`thrf.rex`.
* **State after the unwind.** `p/after.rex` is identical at rc 0, unloader included. It runs three rounds of:
  a Throw rethrown through `catch(...){throw;}`; another native call on the same library; a method that uses
  `SetObjectVariable` and `SendMessage`; then a Throw on a *stashed outer* context from a nested call, then
  `gc('F')`.
  By reading: the catch sits inside `call_stub`, which is inside `Mapping::hold` (its `InFlight` guard)
  and `ThreadContext::enter`. Only extension frames and the Throw slot's own frame unwind. That slot frame holds
  no guard: the `activation_of` temporary drops before `unwind()`.
* **Other unwind paths.** A C frame with unwind tables in the middle (`p/cuw.rex`) is identical. A C frame
  without them (`p/cnouw.rex`) gives rc 134 on both sides (stderr differs). A foreign C++ `throw 42` out of an
  entry point (`p/foreign.rex`) gives rc 134 on both sides.
* **Miri.** At the archive: exit 0, 45 passed, 8 ignored (`$R/miri.txt`). It runs
  `a_throw_unwinds_the_extension_and_holds_the_condition` and `every_throw_member_unwinds_the_extension`. Miri
  handles this Rust-to-Rust `C-unwind` path, but see M5.
* **Spec note.** It is accurate on what it states. I2 lists what it omits.

## Priority 2: original findings, re-run

* **C1:** `mbfill` identical. **I1:** `kept` identical, with its `gc('F')`.
* **M1:** `attach_thread` checks `home` before it reads the cell (`ffi.rs:3294-3298`). `attach` still gives a
  loud rc 134, recorded Phase 9.
* **M2:** `outer` identical. **M3:** `zero` identical.
* **Behaviour review probes:** `co2`, `v4`, `n6`, `fc`, `n2`, `dimr`, `mem2`, `s2`, `unt`, `untf`, `v2` and `v3`
  are identical.
* **N6a rebuilt** (`$R/mut/rexx-run-n6a`): `library_callback_messages` goes red on stdout only, and so does
  `n6`, as predicted.
* **Remaining differences:** each maps to a new exclusions entry or to a non-Task-5 refusal. `cond3`, `m2`,
  `c3m`, `sendthrow` (PROPAGATED/native frame); `rcond` (POSITION); `ov` (Directory override; MyStem~new is Phase 5);
  `cond2` (EXECUTABLE, Phase 5); `cd2`'s last line (oracle-crashes entry 18); `pk2` (Phase 5).
  The crate side of `refs`, `crash4` and `crashsyn` matches the text of entries 16-18.
* **CSTRING against GC.** The key is the ObjRef with its generation. `collect_now` prunes copies of freed heap
  objects (`lib.rs:2827`).
  `p/keptots.rex` (the ObjectToStringValue of an Array kept by a global reference across `gc('F')`) and
  `p/mbkey.rex` (a MutableBuffer mutated between two ObjectToStringValue calls) are identical.

## New findings

**Important I1: kept CSTRING copies of handle-carried values grow without bound.**
`dispatch/library.rs:809-815` inserts a copy for every object. The prune at `lib.rs:2827-2830` keeps every
non-heap key forever. Every `CSTRING` argument now goes through this path (`values/convert.rs:354`), so each distinct
short string or small integer an extension is ever passed stays allocated. `p/grow.rex` under `/usr/bin/time -f %M`
(max RSS, KB), with the extension called in a loop:

| Argument | Calls | Crate | Oracle |
|---|---|---|---|
| none (baseline) | 300k | 17768 | 20676 |
| `CStr(i)` | 300k | 44204 | 19880 |
| `CStr('k'i)` | 300k | 44992 | 20480 |
| `CStr('k'i)` | 1M | 123828 | 20264 |

That is about 105 B per distinct value. Wall time roughly triples against the baseline.

**Important I2: the unwind diverges in three shapes, and neither the exclusions nor the spec note records them.**
All three are loud on our side.
* **Swallowing `catch(...)`.** An extension that wraps a Throw in `catch(...)` and does not rethrow (`p/catchall.rex`):
  the oracle continues (`trapped 40 40.1 inner;caught;after-catch;`). Ours aborts, rc 134,
  "Rust panics must be rethrown".
* **Statically linked libgcc.** The same library built `-static-libgcc` (`$R/libstatic`): `rethrow`, `cuw` and
  `after` work on the oracle. Ours gives rc 134 with **empty stderr** (gdb: `_Unwind_SetGR.cold` abort in
  the extension's own personality). The spec note (`specs/...native-api.md:248-250`) states this as a requirement
  ("needs the process's one unwinder"). It is not recorded as a divergence, and the oracle has no such requirement.
* **Throw from inside a loader.** A Throw on an outer call context from inside a library loader that a `LoadLibrary`
  slot runs (`p/loadthrow.rex`, `librr2`): the oracle traps 40.1. Ours gives rc 134 "panic in a function that
  cannot unwind" at the extern "C" slot.

**Minor M3: a panic inside a Throw slot unwinds instead of aborting.** A bug panic in a `throw_*` body is not
caught at the slot. It runs the extension's destructors, gets past `call_stub` and ends the process at rc 101
(`RR_BUGPANIC`, `p/rethrow.rex`, `p/cuw.rex`). The spec's "a panic from a bug aborts" holds for every other slot.
A resumed non-marker payload at top level exits rc 101 with no stderr and loses the buffered SAY output
(`RR_PAYLOAD`). That payload was planted, so this is informational.

**Minor M4: the exit context's `Throw*` changed from record-and-return to `abort_now`.** This is `layout.rs:859-863`.
It is unreachable today, because `exit_context_interface()` aborts (`layout.rs:900`). The spec note calls them
`C-unwind` but does not say they refuse.

**Minor M5: Miri covers less of the Throw path than the tests suggest.** Miri runs only the `RexxMethodContext_`
instantiation of the Throw generics (`throwing_stub` is a method stub). The `CallContextInterface` instantiation is
not run under Miri. Every foreign-frame aspect (personality, cleanups, and I2) is outside Miri by nature.

## Commands

* `$R/cmp.sh <probe>.rex` (with `LIBDIR=$R/forge` for the forge probes). `BIN=`/`TAG=` select a mutant.
* Mutants: `$R/ffi.rs.orig` and `$R/callbacks.rs.orig` were restored and checked with `cmp`, then rebuilt.
* Miri: `RUSTUP_HOME=$S/surface-4/rustup-home CARGO_TARGET_DIR=$R/target cargo +nightly miri test -p rexx-api --lib --offline`.

## Predictions (written before the mutant run)
One mutant binary, env-gated, in the scratch archive tree (ffi.rs only):
- RR_PANIC: `get_context_digits` (extern "C") panics. Predict: `Again()` aborts rc 134 "panic in a function that cannot unwind"; no Rexx trap.
- RR_PAYLOAD: `throw_exception0` resumes `Box::new("other")` instead of `Thrown`. Predict: rethrow.rex -- extension dtor runs, call_stub resumes, interpreter thread reports a panic, rc != 0, no "trapped"; after.rex's nested Stash case aborts at SendMessage's extern "C" frame.
- RR_BUGPANIC: `throw_exception0` does `panic!` before recording. Predict: the panic unwinds the extension (not an abort at the slot), is resumed at call_stub, and ends the process like RR_PAYLOAD.
- N6a (callbacks.rs:1331 `name.to_ascii_uppercase()` -> `name.to_vec()`): predict `rust/corpus/lang/library_callback_messages.rex` differs from its expected output on stdout only, and review-s5a's n6.rex differs (`foo`/`fOo`, `whoami`, `"nosuchmethod"`).

## Fix round 2 (7d25fa15d)

Predictions (written before the mutant run; env-gated plants in an archive copy of 7d25fa15d's ffi.rs):
- RR_BUGPANIC: panic inside `throw_exception0`'s record closure. Predict rc 134 at the slot; the extension's destructor never runs (Dtors not reached; no "inner;").
- RR_LLPANIC: panic inside `load_library`'s body (now `extern "C-unwind"`) before the load. Predict: it unwinds into the calling extension (its `loadthrow` destructor runs), is resumed at call_stub and ends the process rc 101 -- the M3 class reopened for LoadLibrary/RegisterLibrary.

Built from `git archive 7d25fa15d` into `$R/tree2` (touched; 20 Compiling lines, `$R/build2.txt`), target `$R/target2` (deleted after). Probes as before with `BIN=$R/target2/release/rexx-run TAG=.r2`.

**Verdict: all six asks addressed. One new Minor finding (the M3 class reopened through LoadLibrary), two Minor notes.**

1. **I1 growth fixed.** `/usr/bin/time -f %M` on `p/grow.rex` at 1M calls, max RSS in KB:

   | Case | Crate | Oracle |
   |---|---|---|
   | no-call baseline | 18120 | 20668 |
   | `CStr(i)` | 18996 | 19640 |
   | `CStr('k'i)` | 18140 | 20400 |

   At e1c9d32a0 the `CStr('k'i)` case was 123828. Wall time is 0.57-0.61 s against the oracle's 0.26-0.34 s.
   These are identical to the oracle: `kept` (with `gc('F')`), `mbfill`, `throw`, `outer` and `zero`.
2. **Loader Throw fixed.** `p/loadthrow.rex` and `p/after.rex` are identical at rc 0. The forge's `loader/lthrow.rex` and `lraise.rex`, built by its `build.sh`, are identical too.
3. **M3 plant.** With `RR_BUGPANIC`, a panic inside `throw_exception0`'s record closure gives rc 134, and the extension's destructor never runs.
   The rr destructors now also write to stderr under `RR_DTOR_STDERR`. With that set, no `dtor` line appears (`p/rethrow.rex`, `p/cuw.rex`). The control, unplanted, prints `dtor inner`.
4. **Exclusions entries and spec note are accurate against my measurements.**
   * `catchall` still gives rc 134, "Rust panics must be rethrown".
   * The static-libgcc library still gives rc 134 with empty stderr.
   * The rethrow and destructor claims match.
   * Minor note: "OWNER: none: Rust cannot raise a C++ exception" is stronger than true. A C++ shim linked into the build could throw and catch one. The decision may stand, but not on that reason.
   * Minor note: the entry cites `rr.cpp`, which lives only in the scratchpad, not in the repo.
5. **Alignment.** `p/align.rex` (a method `Align` in `rr.cpp`) is identical to the oracle over three rounds with allocations in between.
   It checks `AllocateObjectMemory` of 0, 1, 3, 15, 16, 17, 100 and 4096 bytes, then realloc to `3n+5`, then realloc to 2: every address % 16 = 0. `NewBuffer` of 0, 13, 26 and 39 bytes: `BufferData` % 16 = 0.
   The alignment is by type: `Word` is `#[repr(C, align(16))]`, and `AlignedBytes` holds at least one of them (`body.rs`, d26c340f7).
6. **Miri** (default Stacked Borrows, `MIRIFLAGS` empty): exit 0, 47 passed, 8 ignored (`$R/miri2.txt`).
   It includes `every_call_context_throw_member_unwinds_the_routine`, `every_throw_member_unwinds_the_extension` and `a_throw_unwinds_the_extension_and_holds_the_condition`.
   Miri does not reach the hook-Throw path (`run_hook`'s catch): the three hook tests are ignored ("opens the running image").

**New finding, Minor: the M3 class is back through LoadLibrary and RegisterLibrary.**
`load_library` (`ffi.rs:3064`) and `register_library` are now `extern "C-unwind"` as whole functions. So any panic in their bodies unwinds into the calling extension instead of aborting at the slot. That includes `innermost_activation`'s "no native call in flight" assert, `activation.load_library`, and a non-marker payload that `run_hook` resumes.
Planted (`RR_LLPANIC`, a panic at the top of `load_library`): `p/loadthrow.rex` gives rc 101, and stderr shows `dtor loadthrow`, so the extension's destructor ran. Before this round the same panic aborted.
Fix shape: the same `throw_after` treatment. Run the body under `catch_unwind`, abort on any payload, then `unwind()` only when `threw`.

**Minor note: a short string's ObjectToStringValue copy now dies at the call's end.** This is the result of aeb7edcbf. `p/keptots2.rex`: `ObjectToStringValue` of `.array~of(1,2)` is kept, the global reference is on the *array*, and the pointer is read in the next call with no collection in between.
The oracle gives `310A32`. Ours reads freed memory: `11`. A long string value still agrees, because its heap string lives until a collection.
The oracle's pointer is only as good as an unreferenced temporary, so this is not a contract break. But at e1c9d32a0 this agreed, and it is now a read of freed memory that depends on the string's length. Record it, or keep a handle-carried copy until the next collection rather than to the call's end.

Plants: `$R/ffi2.rs.orig` restored and checked with `cmp`; the mutant is `$R/mut/rexx-run-mut2`.
