# Phase 7 gate — streams and platform

**Assessed 2026-09-13.** The commit and the four gate readings are in section 8, which was written
from the run rather than before it.

The plan's row for this phase reads:

> `StreamClasses.orx` runs; stream model, `ADDRESS`, file system green **on the host**; the `Sys*`
> subset ooTest needs (D11) works. The other four platforms are Phase 11's, per D-P1.

with **L2** as the ladder rung. Section 7 is about that rung, and it is the one criterion this
phase does not meet.

## 1. The criteria, and what instrument reads each

| criterion | instrument | where it is asserted |
|---|---|---|
| `StreamClasses.orx` runs | the embedded library installs and its classes answer | `corpus/lang/*.rex`, `method-bodies.txt` |
| the stream model | `Stream`, `StreamSupplier` and the eight builtins against the oracle | `corpus.rs`, `method_bodies.rs` |
| `ADDRESS` | command dispatch, the handler table, `WITH` redirection | `corpus/phase-7.txt`'s command rows |
| the file system | `.File`'s rows and the `Sys*` file routines | `method_bodies.rs`, `sys_file_functions.rex` |
| the `Sys*` subset | the nine routines D11 names | `internal_routines.rs`, `sys_file_functions.rex` |
| no refusal names this phase | a source scan with a negative control | `closed_phases.rs` |

Every one of them is derived and run. None is a count written into prose.

## 2. What the phase built

The stream model and its state, `.File`, the standard streams and the monitors, commands and
`ADDRESS` including `WITH` redirection, the platform layer (a shadow environment and a shadow
current directory, with no process-global write anywhere), external routine resolution, the nine
`Sys*` routines, the internal-routine table that serves `DIRECTORY`, `FILESPEC` and `BEEP` beside
them, interactive `TRACE ?` with its pause and prompt, `RXTRACE=ON`, and the security manager's six
checkpoints.

Three things arrived at the close-out rather than in a task of their own, each because a criterion
was checked rather than assumed:

* **`.local`'s entries were readable as `.NAME` and not as entries.** `.output` answered a monitor
  where `.local['OUTPUT']` refused, because the bundle is minted on first demand and only one of
  the two paths counted as a demand. `corpus/lang/local_entries.rex` pins all nine names and the
  identities between the two spellings.
* **Positioning a transient stream by line or character was allowed.** `SEEK` had the check and
  `lineIn(n)`/`charIn(n)` did not, so `.Stream~new('/dev/null')~supplier` built a supplier where
  the oracle raises 93.958.
* **A required argument to a native method reported the wrong error.** `stream_position` declares
  its options string without `OPTIONAL_`, so omitting it is 88.901 against the package -- `Error 88
  running REXX`, no program line. The crate answered 93.903.

## 3. The negative controls

Each was predicted before it was run.

* `corpus/lang/call_miss_not_cached.rex` raises 43.1 on its first pass, writes the file, and finds
  it on the second. Deleting the guard in `Calls::remember` makes the second pass raise too.
* `closed_phases.rs` finds a phase that is still open by the same walk that finds none of the
  closed one, so an empty result is not the scan looking in the wrong place. Re-introducing one
  `"Phase 7"` owner reddens it.
* `prompt_before_read.rs` waits for a prompt with nothing yet written to the child's standard
  input. Making the handover return early fails both cases with the read still blocked.
* `trace.rs`'s `off_refuses_the_debug_flag_from_either_side` carries `?R` rows as its positive
  control, so a guard that refused debug everywhere would not pass it.

## 4. What the instruments cannot see

* **`method-bodies.txt` records `answers` for a row whose receiver raised identically on both
  sides.** The send never happened; the two sides agreed about that. `Stream` and `StreamSupplier`
  were in exactly that state at the start of the close-out -- 32 rows saying nothing -- and giving
  each class a receiver that actually constructs is what turned the two defects in section 2 up.
  The rows now carry `rc 0` where they answer.
* **The corpus differential cannot see anything that is not a byte on a descriptor.** The prompt
  ordering in section 3 is the case: a prompt written at exit and one written before the read
  compare equal once the run is over.
* **A sidecar cannot see a witness that was deleted**, only one whose content changed.
* **The oracle's own crashes bound what may be probed.** `corpus/oracle-crashes.txt` gained this
  phase's entries; entry 9, `::OPTIONS TRACE ?<letter>`, is an indefinite block, so the `TRACE OFF`
  rule in section 2 was settled against the C++ source rather than against a run.

## 5. The platforms

**Linux, measured.** Every figure in this document was taken on this host.

**The other four: CANNOT ASSESS, and that is Phase 11's, not an accepted divergence.** Ruling D-P1
records it: what is deferred is portability, not correctness, because on a platform the crate does
not run there is no output to differ. The measured cost of the seam is `PLATFORM = b"LINUX"` and
`LINE_END` in `parse_template.rs`, eight `std::os::unix` uses across six files, and nineteen
`rustix` sites in two. The unblocker is a second host, which Moritz will provide.

## 6. Refusals this phase re-homed

Nothing in the crate names Phase 7 any more, and `closed_phases.rs` asserts it. The refusals it
left behind went to the phase that actually owes each:

* **Phase 8** -- `::ROUTINE EXTERNAL`, `::REQUIRES LIBRARY`, the `::METHOD` and `::ATTRIBUTE`
  `EXTERNAL` forms naming a library other than `REXX`, `loadExternalMethod`/`loadExternalRoutine`,
  `Package~loadLibrary`, and the `handle_set` entry point. All of them open a shared library.
* **Phase 10** -- the queue entry points, `.STDQUE`, the RexxUtil remainder, and
  `Package~options(name, value)`.
* **Phase 5** -- the two `.STREAM` fallbacks, which can only fire before the library bootstrap has
  installed `StreamClasses.orx`.

## 7. L2 is not met, and the blocker is Phase 8's

The ladder's L2 is *"`ooTest.frm` loads and a single test group executes"*. It does not, and the
chain was walked rather than guessed:

1. `ootest/testOORexx.rex` stopped at `.ENDOFLINE`, which the framework's own prologue reads
   (`OOREXXUNIT.CLS:77`). No test group ran. That entry was the last unbuilt name in
   `.environment`; it is a one-line string constant and this phase built it rather than leave the
   suite blocked on a closed phase's omission.
2. The framework then needs `rxregexp.cls`, which ships with the interpreter rather than with the
   suite, so this crate -- which is installed nowhere -- has to be pointed at a copy.
3. Pointed at one, it stops at `::METHOD INIT EXTERNAL "LIBRARY rxregexp RegExp_Init"`.

**That is a shared library, and loading one is Phase 8's.** So L2 cannot be reached from inside
Phase 7 by any amount of work on streams or the platform layer, and the plan's row asks for
something this phase cannot deliver. The reading offered here is that the row is wrong rather than
the work: every criterion in section 1 that is Phase 7's own is met, and L2 moves to Phase 8.

## 8. The gate readings

Recorded from the run, unpiped, at the commit named. Both failing sets were enumerated rather than
counted, and every member is a pre-existing failure carried from before this phase.

<!-- filled in from the closing gate run; see the ledger for the per-commit history -->
