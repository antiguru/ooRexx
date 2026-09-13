# SDD ledger — plan: docs/superpowers/plans/2026-09-12-phase-7.md

Spec: `docs/superpowers/specs/2026-09-11-phase-7-streams-platform.md`.
Surveys: `.superpowers/sdd/2026-09-11-phase-7/survey/` (six reports, all cited by the plan).

## Pre-flight scan, 2026-09-12

One row per task pair that shares a file or an interface, and one row per task whose own text was
checked against itself.

| pair | shared | produced vs consumed | finding |
|---|---|---|---|
| 1 → 3, 18 | `internal_routines.rs` | Task 1 builds the table with bodiless rows; 3 fills `DIRECTORY`, 18 fills the nine `Sys*` | consistent |
| 1 → 22 | the owner strings | 1 makes every refusal name its phase; 22 asserts none names Phase 7 | consistent, and 22's assertion is 1's negative control at phase scale |
| 2 → 3, 4, 13, 17, 18 | `paths::qualify`, the shadows | 2 produces, the rest consume | consistent; **no task may join a relative path itself**, which the plan states in 2's Interfaces |
| 2 → 17 | `require.rs` | 2 moves the search onto `SearchContext`, 17 fixes its three infidelities | ordered: 2 must not change behaviour, 17 changes it deliberately. Recorded so 2's reviewer does not "fix" `.hid` early |
| 4 → 5 → 6 → 8 | `stream.rs`, the state | sequential by construction | consistent |
| 6, 7 | the EOF answers | 6 lands `NOTREADY:EOF`/`ERROR:0` as *states*; 7 raises the condition | ordered; 6's witnesses must not assert a trap |
| 9 → 10, 11 | `resolve_stream` | 9 routes names, 10 supplies the standard objects, 11 the monitors | ordered 9 → 10 → 11. Task 9 Step 1 routes `STDIN`/`STDOUT`/`STDERR` to the standard streams, and Task 11 re-points them at the monitors -- **an intentional two-step**, because the oracle routes through the monitors and Task 9 cannot until they exist |
| 10 → 21 | `input.rs` | 10 adds byte reads, 21 adds the flush-before-read | consistent |
| 11 → 15, 19 | the trace route | 11 routes trace output, 15 adds `+++`, 19 adds the pause | ordered |
| 13, 18 | `files.rs` | `.File`'s natives and the `Sys*` family share one helper | consistent, and named in both tasks |
| 14 → 15 → 16 → 20 | `command.rs` | sequential | consistent |
| 17 → 20 | the `CALL`/`REQUIRES` hooks | 17 builds the resolution sites, 20 hooks them | ordered |
| 3, 17 | the `SETLOCAL` stack | 3 pushes and pops it, 17's external routines restore at return | consistent; 17's witness must cover it |
| every task | `rust/corpus/phase-7.txt` | every task appends its witnesses | **a serialisation point**: two implementers editing it concurrently conflict. Tasks run one at a time, so this is a note, not a defect |

Task-internal check: each task's witnesses name programs that its own steps make runnable, and no
task's witness depends on a later task's behaviour, with one exception -- Task 16 Step 3
(`ADDRESS WITH STREAM`) needs the stream arm, and the plan says so in that step rather than
leaving it to be discovered.

**Rulings from the scan:** none needed; the two ordering hazards above are recorded rather than
resolved, because the order the plan already states resolves them.

## Constraint the whole plan runs under

Subagent dispatch hit the account's session limit twice on 2026-09-11/12 (resets 07:20 Europe/
Zurich). Until it resets, tasks are executed in the main session; after it, by dispatched
implementers as the skill prescribes. Either way each task ends committed with its gates recorded.

## Tasks

### Task 1: every unimplemented name refuses loudly, naming the right phase — complete

Executed in the main session (no subagent capacity until 07:20).

What landed:

- `rexx_inventory::builtins::EXCLUDED` carries `(name, owner)` pairs, with `owner_of` and
  `excluded_names` beside it; `Loud::unresolved_call` reads the owner from there instead of the
  hardcoded `"4c"` every excluded builtin used to be blamed on.
- A new `rexx-exec/src/internal_routines.rs`: the `REXX` package's three routines and the
  `REXXUTIL` package's 55, each with the phase owing it a body (Phase 6 for the semaphore wrappers,
  Phase 7 for the nine ooTest needs plus `DIRECTORY`/`FILESPEC`/`BEEP`, Phase 10 for the rest).
  The resolver consults it between the library-bootstrap lookup and `routine_not_found`, so those
  names refuse loudly where they used to answer 43.1 — an answer a program cannot tell from its
  own typo.
- `.STDQUE`'s refusal names Phase 10. That exposed a real defect in `unbuilt_collection_owner`,
  which answered `.min()` over the owners of a directory's entries: with one Phase 10 entry in
  `.local`, the *whole-directory* refusal started saying Phase 10, because `"Phase 10" < "Phase 7"`
  as strings. It now names the directory's own phase.
- Witnesses: `run/tests.rs`'s `an_internal_routine_refuses_loudly_where_an_unknown_name_still_raises`
  (seven rows across all three owning phases, plus `zorkolo` still raising 43.1 rc 213), the
  derivation test `tests/internal_routines.rs`, and `owners.rs`'s
  `every_excluded_builtin_names_a_phase_that_owes_it`.
- `corpus/refusal-sites.tsv` re-derived: 63 definitions moved, one row added.

**Negative controls, predicted before running, all three confirmed:**

| control | prediction | outcome |
|---|---|---|
| `QUALIFY` owner → `4c` | the owner assertion fails naming QUALIFY | confirmed |
| `SysSleep` owner → `Phase 4` | the phase check fails naming SysSleep | confirmed |
| the `SysCls` row deleted | the derivation fails "registered but not in the table: [SysCls]" | confirmed |

**A miss worth recording.** The first control run used plain `cargo test`, which stops at the first
failing *binary*: it reported control 1 and never ran controls 2 and 3, which I would have recorded
as confirmed had I not noticed only one `test result` line. Re-run with `--no-fail-fast`, all three
fired. This is [[mutation-runs-need-no-fail-fast]] arriving in a new costume — the controls were
not a mutation run, so the rule did not announce itself.

**Ruling on gate cadence.** The four-gate suite is ~27 minutes and the plan has 22 tasks, so
gating every task would freeze the tree for ten hours. Each task runs `fmt`, `clippy -D warnings`
and every test binary it can redden; the full four gates run at checkpoints (after the shadow
tasks, after each arm closes) and at the phase's closing commit. Cost if wrong: a cross-task
breakage sits undetected until the next checkpoint, which is bounded by the checkpoint spacing.

Task 1 gates: `cargo fmt --all --check` 0; `cargo clippy -p rexx-exec -p rexx-inventory
--all-targets -- -D warnings` 0; `cargo test -p rexx-exec -p rexx-inventory --lib --test
internal_routines --test coverage --test loud --test builtin_status --test refusal_sites --test
native_entries` — 789 + 26 + 21 + 2 + 9 + 24 + 5 passed, 0 failed.

Commit: `ca460deaa`.

### Task 2: the interpreter owns its environment and its current directory — complete

What landed:

- `Interp.env` (ordered name/value bytes) and `Interp.cwd`, initialised from the process at
  `Interp::new` unless the `Invocation` supplies them; `Invocation::with_directory` and
  `with_environment` supply them, and `into_parts` became a named `InvocationParts` rather than a
  tuple that grows an element per phase.
- `Interp::resolve_search` reads `PATH` and `REXX_PATH` through a new `Interp::shadow_var` and
  resolves against `Interp.cwd`; it read the process's own before. This is what
  `ootest/testOORexx.rex` needs, since it sets `PATH` through `VALUE` and changes directory before
  calling the program it then has to find.
- The exhaustive `Interp` destructure — the guard that makes a new field a compile error until
  someone decides whether it is a root — got both fields with a line saying why neither is one.
- `tests/no_process_state.rs`: no source file may call `env::set_var`, `env::remove_var` or
  `env::set_current_dir`, or import them.

**Plan correction.** Task 2's Step 2 said to port `SysFileSystem::normalizePathName`. The tree
already had that port — `require::normalize`, with unit tests covering `..b`, `/..`, duplicate and
trailing slashes — so the step became a *move* into the new `paths` module, with `require.rs`
calling through it. Writing a second port would have left two implementations of one rule. The plan
text is left as written; this entry is the correction.

**The guard test's first version was the defect, not its subject.** It matched bare substrings, so
`set_var` matched twenty `set_variable` call sites (this interpreter's Rexx-variable setter) and two
of its own doc comments — a red that named nothing real. It now matches call shapes through the
module path plus the `use` forms, and skips comment lines.

**Negative control, predicted before running, confirmed:** inserting `use std::env::set_var;` and a
`#[cfg(any())]` function calling `std::env::set_current_dir(...)` into `paths.rs` fails the guard
listing exactly those two lines; removing them is green again.

Regression checks: `--lib` 789 passed; `REXX_CORPUS_GATE=1 --test corpus` 27 passed, 1 ignored —
the differential over the committed subset, which is what a wrong search would break;
`--test refusal_sites` needed no rebuild, because this task's new code sits after every `Loud`
constructor in `lib.rs`.

Task 2 gates: `rustfmt` on the touched files, `cargo check --all-targets` 0, the tests above.

Commit: `715d7a3b4`.

### Task 3: the platform readers — in progress

**Ruling: the harness gives the crate the program's own directory wherever the oracle gets it.**
`Oracle::run` always runs from the program's directory (`tests/support/oracle.rs`, `wrapped`), while
a dozen crate-side call sites pass `Invocation::none()`, which now means the *process's* directory.
Nothing noticed because until Task 2 no crate-side construct could observe a working directory at
all. With `DIRECTORY` and `QUALIFY` landing, any of those sites can diverge for a reason that is not
the construct under test. `measure()` in `tests/builtin_status.rs` is fixed first, because Task 3's
probes go through it; the remaining sites are fixed as the tasks that need them arrive. Cost if
wrong: a probe measures the harness instead of the builtin, which is the failure this repairs.

**Plan amendment.** Task 3 inherits that harness fix; the plan had put the harness work in Task 17's
sidecar, which is too late for a `QUALIFY` probe.

Oracle behaviour measured for the rows this task commits (fresh directory, standard wrapper):
`qualify('/a/../b/c.txt')` → `/b/c.txt`; `say setlocal() endlocal()` → `1 1`; an unpaired
`endlocal()` → `0`; `filespec` `N`/`L`/`E`/`P`/`D` over `/a/b.c` → `b.c`, `/a/`, `c`, `/a/`, empty;
`value('P7_NOSUCH_VAR',,'ENVIRONMENT')` → the null string; an unknown selector → 40.914 at rc 216.

What landed:

- `builtin/platform.rs`: `QUALIFY` (through `paths::normalize` against the shadow directory, with
  `~` and `~/` expanded from the shadow `HOME` and `~user` left alone), `USERID` (`/proc/self/status`
  for the effective uid, `/etc/passwd` for the name, the environment's `USER`/`LOGNAME` as the
  fallback an NSS-only account would need), `SETLOCAL` and `ENDLOCAL` over the snapshot stack, and
  `VALUE`'s two external pools.
- `VALUE`'s empty selector reads and writes `.environment` through the security seam's new
  `set_directory_entry`; the `ENVIRONMENT` selector reads and writes the interpreter's own
  environment, with `.nil` removing, a value truncated at the first NUL, and a name that is empty
  or holds `=` silently storing nothing, as `setenv` does. Any other selector is 40.914.
- Four names left `EXCLUDED`, so the counts moved: 15 whole exclusions became 11, 66 in scope became
  70, in `coverage.rs`, in `builtin_status.rs` (three literals, one of which is an
  invocation count the derived assertion above it does not cover), and in the exclusions file's own
  prose. `builtin-status.txt`'s four rows flipped to `implemented` and `builtin-probes.txt` gained
  four rows written from the measurements above.
- `Loud::value_selector` is gone: its only caller was the refusal it replaced. The test that pinned
  its message now asserts the delivered behaviour instead — an empty selector answering `.MYVAR` for
  a name `.environment` does not hold, which is what the oracle answers, and a named-but-unknown
  selector raising 40.914 — keeping the distinction the old test existed to draw.

**Two literals the derived assertions did not cover.** `builtin_status.rs` asserts
`oracle_invocations == in_scope.len()` (which adapted from 66 to 70 on its own) and then, two lines
later, `oracle_invocations == 66` as a literal. The derived assertion passed and the literal failed,
which is the whole argument for deriving: the pair only stayed honest because one of them was
computed. The same shape sat in `coverage.rs`.

Task 3 gates: `cargo fmt --all --check` 0; `cargo clippy -p rexx-exec -p rexx-inventory
--all-targets -- -D warnings` 0; `--lib` 789 passed; `--test builtin_status` 26 passed — the run
that measures the four new builtins against the oracle on all three descriptors; `--test coverage`
21; `--test refusal_sites` 5.

**The three internal-package routines landed too.** `InternalRoutine` gained a body slot and
`Resolved` an `Internal` variant, so a row with code resolves to it and a row without still refuses
naming its phase. The variant had to be handled at five dispatch sites; four were compile errors
because `invoke_call_over`'s `match` has no wildcard, and the fifth —
`subroutine_over_pushed_args` — has a catch-all arm that would have swallowed it silently. That one
I had to find by reading, which is the argument against catch-alls in a dispatch.

- `DIRECTORY` moves and reports the shadow directory, answering the null string for a change it
  cannot make; `FILESPEC` answers the five `DELNP` pieces textually; `BEEP` checks 37–32767 and
  0–60000, writes one bell byte to the output buffer and answers the null string.
- **A guess the probe caught.** I implemented `FILESPEC`'s `E` with `.File~extension`'s rule — a
  name whose only dot opens it has no extension — and the oracle answers `hidden` for
  `filespec('E','/a/.hidden')`. Two routines, two rules; the code now says which is which.
- **A name collision worth recording.** `Raised::argument_out_of_range` already existed as 40.903,
  whose bounds are fixed in the message text; the native-routine range error is 88.907 with the
  bounds substituted. Mine is `native_argument_out_of_range`, and the doc on each points at the
  other.
- What the oracle answers for a *non-numeric* `BEEP` argument was not measured, so that path reports
  the measured range error rather than a message I would have invented.

Witness: `lang/platform_internal_routines.rex`. The corpus differential reads **471 of 471
matching** with `phase-7.txt` in its header.

`corpus/refusal-sites.tsv` was re-derived for the third time tonight — it pins every `Loud` and
`Raised` constructor by `file:line`, so any edit above one moves rows. The rebuild is a script over
the test's own diff output rather than hand editing, which is what the file's header asks for.

**My rebuild script destroyed data, and the table's other tests caught it.** Two constructors —
`missing_named_argument` and `too_many_external_arguments` — gained *body* callers from the three
new routines, so their derived surface changed from `send` to `body+send`. The script keyed rows on
(kind, name, **surface**), so a changed surface read as one row disappearing and another appearing:
it dropped the originals with their measured verdict, reached, answer and witness columns, and
re-added them with a hardcoded `off-send-surface - - -`. The main derivation test went green over
that, because it only compares the first four columns; three *other* tests in the same file caught
it — "off-send-surface is not one of the four verdicts", "reached is -, which is neither yes nor
no", and the shared-answer set changing. Rebuilt keyed on (kind, name) alone, taking surface and
definition from the derivation and preserving the four measured columns. **A mechanical rebuild may
only rewrite the derived columns**, and the check that saved me was a test that reads a column my
script thought it owned.

**The subset-file list lives in two places.** Registering `phase-7.txt` in `corpus.rs`'s
`SUBSET_FILES` left `coverage.rs`'s own copy behind, and its
`the_union_reads_every_phase_subset_file` caught it. One fact, two enumerations, one updated — the
shape that keeps recurring here. Both are policed against the directory listing, which is why this
was a red rather than a silent half-run.

The refusal test now asserts **both** sides of the boundary: `FILESPEC` and `DIRECTORY` answering
through the same resolver step that refuses `SysFileExists`, `SysStemSort`, `SysWaitEventSem`,
`LINEIN` and `RXQUEUE`. Without the positive half it would stay green over a resolver that refused
everything.

**I wrote a witness that crashed the oracle, with the crash entry already in my own hand.**
`lang/platform_qualify_setlocal.rex` ended `say 'unpaired' endlocal()` — an `ENDLOCAL` past the
outstanding `SETLOCAL`s, which is entry 10a of `corpus/oracle-crashes.txt`, committed by me hours
earlier at `91cf35f04`. The oracle died at rc 139 on the first dry run. Nothing was harmed (the
program is new, not one of the file's entries, and a corpus program that crashes the oracle is a
structural red rather than a silent pass), but the lesson is that **knowing a shape is fatal does
not stop me writing it**: the licensed-divergence list has to be read as a constraint on witnesses
at the moment a witness is written, not recalled afterwards. The line is gone and the program says
in a comment why there is no second `ENDLOCAL`.

The corpus harness got the same repair as `measure()`: `run_rust` gives the crate the program's own
directory, which is what lets a witness call `qualify('rel.txt')` at all.

**"470 of 470 matching" did not say my programs ran.** The corpus report names only mismatches, so
the two new witnesses appeared nowhere in a green run and their absence meant nothing. The control
that settles it defeats the mechanism from the crate's side — editing the *program* cannot diverge,
since both interpreters run the same text. Predicted before running: corrupting `qualify`'s answer
fails the gated run, names `lang/platform_qualify_setlocal.rex`, and drops the count. Measured:
`469 of 470 matching`, `[UNCLASSIFIED] lang/platform_qualify_setlocal.rex: stdout differ`, restored
green. So 470 is the previous 468 plus these two.

## Task 4: the Stream entry points that answer before anything is open

Thirteen rows flipped to `implemented`; `handle_set` deferred to Phase 8. State lives on the
instance as `NativeState::Stream`. Corpus `472 of 472`; fmt, clippy, `native_entries`,
`builtin_status`, `coverage`, `refusal_sites`, both lib suites and `rexx-parse` green.

**The 48.1 substitution was a guess and is now measured.** `require`'s
`Stream not initialized` came from nowhere when I wrote it -- the catalogue grep had returned
nothing. `oracle-crashes.txt` entry 12b turned out to hold it: the oracle's *guarded* entry points
raise exactly that after `~uninit`, while the unguarded half segfaults. So the wording is the
oracle's, taken from the record rather than from a probe the record forbids running.

**`query_size` was wrong and the witness caught it.** I had `metadata.len()`, which answers a
directory's own `st_size`. Prediction written before the run: one mismatch, that file, last line
only, mine `[] [4096]` against `[] [0]`. Shape confirmed exactly -- the other twelve bodies were
byte-identical -- and the magnitude falsified, because the run directory stats at 32768, not 4096.
`SysFile::getSize` then confirmed the rule: `st_size` for a regular file, `0` for anything else
that stats, false for anything that does not.

**Ruling: hand-edit two rows of `refusal-sites.tsv`, not a whole-table rebuild.** The derivation
disagreed on exactly two rows and printed both old and new values. A whole-file rewrite touches
250+ rows to fix 2, and the last one here clobbered four measured columns by re-keying on surface.
Cost if wrong: the test stays red and names the row. It went green.

`error.rs:97` moved from `body` to `body+send` because a native entry point reached by message send
now raises it, so its four columns had to stop being placeholders. Measured both sides on
`s = .Stream~new('t.txt'); s~uninit; s~flush`: rc 208, stdout `main`, stderr identical --
`Error 48.1: Failure in system service: Stream not initialized.` `stream_flush` is guarded
(`StreamNative.cpp:2236`), which is why this shape answers where `~state` crashes.

**A stale binary nearly bought that measurement.** `cargo build -p rexx-run` failed --
`rexx-run` is a *binary*, not a package -- and the run used a pre-Task-4 executable whose stderr
still said `stream_init` was unimplemented. `--bin` plus an mtime check is what settled it.

**Defect closed from earlier tasks, not from this one.** A new `corpus/lang/*.rex` needs an
expectation under `crates/rexx-parse/tests/sourceline_oracle/`. The three platform witnesses
shipped in `16c6bcbcb` and `a629183d1` without one, so
`sourceline_matches_the_interpreter_for_every_corpus_program` has been red in the committed tree
since they landed -- nothing those tasks ran touched `rexx-parse`. All four phase-7 witnesses now
have theirs. The test panics on the *first* missing file, so its output could not have named the
set; regenerating all four and letting it re-derive is what found the boundary. Generation was
scoped to those four rather than the module comment's `for f in corpus/lang/*.rex`, because
`.Package~new` executes each file's prolog.

**Ruling: no second corpus witness for the `uninit`->`flush` path in this task.** Nothing
executable pins 48.1 -- the TSV row, entry 12b and my two-sided measurement are all prose or
one-off. A corpus row would fix that, but the 48.1 message embeds the program's absolute path and
`corpus.rs`'s normalisation is indent-shaped; whether a path survives it is unresolved and reading
that properly is not Task 4's work. Cost if wrong: 48.1 stays prose-pinned until a later stream
task, which is where the guarded entry points (`~linein`, `~open`, `~chars`) land anyway -- add it
there.

Task 4: complete.

## Task 5: OPEN, CLOSE and the option tokenizer -- in progress

**Plan defect corrected in the plan.** Task 5's Files list said modify `src/stream.rs`, which does
not exist; Task 4 put the entry points at `src/dispatch/stream.rs`. Corrected in the plan text, not
just here, because briefs regenerate from the plan.

**A correction of mine was false and is reverted.** Survey A §2.2 says a failed `open read` leaves
the stream `UNKNOWN`. I measured `ERROR` on a held object and rewrote the row -- but §2.7, on
screen at the time, records the asymmetry: the *builtin* form drops the failed stream from the
table so the next `stream(f,'s')` resolves a fresh object, while the *object* form keeps `ERROR`.
Re-measured both in one program: builtin `ERROR:2`/`UNKNOWN`/`UNKNOWN:`, object
`ERROR:2`/`ERROR`/`ERROR:2 No such file or directory`. The row was right; my edit was a correction
round inventing a false statement on top of a true one. The row now names which form it describes
and carries both measurements.

**Ruling: Task 5 answers `ERROR:n` and sets state and errno; Task 7 owns the NOTREADY condition.**
`PendingTrap` has no `additional`/`result` fields and `run.rs`'s only queue site is the
routine-return raise, so there is no general "raise a named condition here" path to borrow. Cost if
wrong: a trapped NOTREADY on a failed open is unavailable until Task 7, which is the task that owns
it. The C++ site to match then is
`RaiseCondition("NOTREADY", String(stream_name), self, result)` -- description is the name as
given, additional is the stream object, result is the call's answer.

**New oracle crash, entry 13, found by probing rather than by reading.** `.Stream~new(n)~copy`
aborts at termination, rc 134, `double free or corruption (out)`, after the program has printed
everything -- so stdout alone looks clean. No open and no existing file needed. `MutableBuffer~copy`
and an ordinary instance copy are both rc 0, so it is neither `COPY` nor native state in general:
`RexxObject::copy` carries `CSELF` into the copy and both `uninit`s destruct the one `StreamInfo`.
Recorded with its four positive shapes and two negative controls; added to the spec's 4a and to
Task 5's text as a hazard.

**Ruling: `StreamState` keeps `Clone`.** The question that started this was whether an open stream
may hold a `std::fs::File`, which is not `Clone`. `COPY` (`dispatch.rs:5724`) clones the body of any
`Primitive::Instance`, and a `Stream` is one -- so the derive cannot simply go. Since the oracle's
own answer here is undefined, this crate is free: the copy gets an independent, unopened state
block, which it already does (measured rc 0, `Stream`/`f.txt`/`UNKNOWN`). The open descriptor
therefore cannot live in a plain field of a `Clone` struct, and the next step is choosing that
representation.

**Known gap, not this task's:** the spec's 4a names `rustix` and `filetime` as this phase's two new
dependencies. Neither is in any `Cargo.toml` and there is no `[workspace.dependencies]` table.
Task 5 needs neither; Tasks 13 and 18 do.

**Ruling: Task 5 implements spec A6, the per-program run directory.** It is spec §5's first
bullet, it binds every program `phase-7.txt` lists, and no task owns it -- the plan's only
harness line (`:596`) is Task 17's `<name>.env`/`<name>.d/` sidecar, a different mechanism. Task 5
is the first task whose witnesses create files, and without A6 they would leave two dozen files in
`corpus/lang/` on every gate run. Built from parts that already existed: `Oracle::run_in`,
`Invocation::with_directory`, and `licensed_divergences.rs`'s temp-directory pattern. Cost if
wrong: a harness change lands in a task that did not plan one. §5's other three items --
`REXX_CORPUS_VAR`, the `<name>.stdin` sibling, the fixture rules -- are left to the tasks that
need them.

**A6's risk was that 462 programs move house.** Prediction written before the run: 474 of 474, no
new mismatches among the seven `::REQUIRES`/relative-path programs, because each is named by
absolute path and the external search starts at the program's own directory rather than the cwd.
Confirmed exactly -- `package_requires` still resolves `'package_requires_dep'` and `package_find`
still resolves `'package_tables_lib.cls'` from a temp directory. Had they gone red, A6 would have
had to narrow to `phase-7.txt`'s own programs.

**The witnesses were written twice.** The first pair built their fixture with `lineout`, which is
Task 6's entry point: the oracle wrote the file and this crate refused, so every row would have
diverged for a reason that had nothing to do with `open`. Rewritten to need no content at all --
`open write` creates a zero-length file, which is exactly what the record-length rule keys on
(measured: `size` 0, `exists` 1, a following `binary` open raises). The rows that genuinely need
bytes in the file move to Task 6.

**Three probe errors in a row, all mine, none the oracle's.** A label inside a `DO` block (47.2);
`call s~close`, which is not an expression (35.1); and a loop whose row 13 was `replace`, which
truncated the shared fixture so rows 14-33 measured an empty file and reported `binary` as
RAISED. The third is the dangerous one: it produced a plausible, wrong table that agreed with a
survey row I nearly "corrected". A fresh fixture per row is the fix, and it is what both witnesses
now do.

**Open question for a later task, not this one.** The C++ `resetFields` clears `qualified_name`
and `streamOpen` re-resolves it against the current directory; Task 4 resolves at `~init` and its
committed witness pins that. A `DIRECTORY()` between `~new` and `~open` would part the two. Not
changed here, because the Task 4 behaviour is green and pinned.

Task 5: complete.

## Repair round: eleven red binaries, and the cadence that hid them

**The cadence ruling was wrong and is replaced.** Per-task `fmt`, `clippy` and *affected* binaries
never runs the tables that derive themselves from `corpus/`, so the workspace was red from Task 3's
`16c6bcbcb` through Task 5's `86f04b352` -- three commits, eleven binaries. **New rule: every task
ends with `cargo test --release -j 4 -p rexx-exec --no-fail-fast`, measured at 11m39s**, and the
four-gate suite stays at checkpoints. Twelve minutes a task is the price of not discovering this
again at task 20.

**Six causes, all repaired and green together** (5, 10, 24, 6, 3, 22, 23, 8, 7, 23, 41 passed):
`SUBSET_FILES` in five files not two; `CLOSED_PHASES` missing `7`; blocker attributions moving
`4c` -> `Phase 7`/`Phase 10`; `bif_assertions` having no way to rewrite its own table; the two
seam lists; and A6's run directory existing only in the differential.

**Ruling: `CLOSED_PHASES` gains `"7"` mid-flight.** Precedent adds the tag at close (`"5j"` joined
at `4954078fc`), but the property -- a phase-7 verdict cannot move silently -- is what I want for
the remaining tasks. Cost: a table refresh per implementing task, already the `method_bodies`
workflow.

**Ruling: the environment seam keeps its second `admit`, and the test pins both.** Task 3 added it
to the write path. D45 says "lookup", but a manager that can veto a read of `.local` and not a
write is a seam with a hole. A third chokepoint is still red, and both must be in the seam's own
module.

**Ruling: `ANOMALY` 3 -> 0 because the rows pass, not to quiet the assertion.** All three were
`BEEP`, which Task 3 gave a body. The file's header demands the row be ruled on before the number
moves; this is that ruling, and `phase-4-exclusions.txt`'s KNOWN GAP is marked closed with it.

**`bif_assertions` gained `REXX_BIF_EXEMPT_REFRESH`.** Both its listings are `take(40)` and the set
is 49, so no run could print what the file should hold -- the table was unrepairable by its own
output. The three sibling derived tables all had a refresh; this one did not.

**I created a defect and it reached the repository.** Registering `phase-7.txt` in three more
`SUBSET_FILES` lists pointed three directory-less harnesses at the Task 5 witnesses, which wrote 26
files into `crates/rexx-exec/`. A6 was implemented in `corpus.rs` alone; spec §5 never scoped it
there. `collect_stress` and `ir_recorded` now take a run directory, `collect_stress` emptying
between its two runs because otherwise the second reads the first's files and the divergence looks
like a GC defect. `trace_oracle` needed none -- it executes only the trace witnesses its own table
names, never the subset programs.

## Task 6: characters and lines -- in progress

**The reads and writes reproduce the oracle.** Three transcripts measured on both interpreters and
byte-identical: survey A §2.1 end to end (implicit open, write position at end, `linein` then
`charin` mid-line, `chars`/`lines`, and the two different EOF answers); §2.6's count cache
including the **`2 1` off-by-one** the port had to reproduce rather than fix; §2.5's CR rules
(`\r\n` collapses, a bare `\r` is data, an unterminated tail is a line); and §2.4's past-end write
with its OS-zero-filled hole. Corpus `477 of 477`.

**A latent Task 4 bug, found by Task 6 needing it.** `description` rendered `ERROR:0` as
`ERROR:0 Success`, because `io::Error::from_raw_os_error(0)` stringifies that way and my suffix
strip left the word. Survey §3.2 says the text comes from `strerror` only when the errno is
non-zero. Nothing witnessed it until `CHARIN` past the end became reachable.

**A duplicate registry row I shipped in Task 5.** `native.rs` carried two `stream_open` entries --
the original `deferred` one and the `implemented` one I inserted, because I assumed the name was
not already in the table. Lookup takes the first match, so everything worked and every test passed.
**No assertion checks that entry-point names are unique**, which is the class defect behind it.

**The family probe was repointed to `handle_set`.** `stream.rex` bound `stream_chars`, now
implemented. `handle_set` is the only stream entry point still deferred when the phase closes, and
the test derives the expected owner from the row, so its `Phase 8` string adapts and no later task
has to repoint it again.

**A pre-existing divergence, measured and NOT fixed: the major line of a native method's
argument-marshalling error.** Five measurements settle the rule:

    qualify(1,2,3)   88.922 marshalling, library-declared -> `running REXX`, no line
    two(1,2,3)       88.922 marshalling, user-declared    -> `running <program>`, no line
    charin(1,-1)     88.907 marshalling, library-declared -> `running REXX`, no line
    charin(0)        93.907 raised by the body            -> `running <program> line 3`
    command('bogus') 93.914 Rexx-coded library body       -> `running REXX line 283`

So it splits on **which activation raises**, not on the error number and not on the caller: an
error raised while the native activation marshals arguments is attributed to that method's own
declaring package and is lineless; an error the body raises reaches the caller's clause with its
line. `NativeActivation::reportSignatureError` and the `Error_Invalid_argument_*` raises at
`:240`-`:299` are the marshalling ones.

This crate gets the body cases right already -- Task 5's bare 93 and `charin(0)`'s 93.907 are
byte-identical uncaught -- and gets every marshalling case wrong, printing the program's path where
the oracle prints `REXX`. **`too_many_external_arguments` has had this bug since before Phase 7**;
it sets `delivery.lineless` but the name still falls back to the program, because
`blame_native_method` pushes a `FailureSite::Rendered`, which reports neither a line nor a name.

**Ruling: record it, do not fix it here.** The name is not reachable at the raise site --
`Resolution` is `{scope, method}`, `library_programs`/`compiled_method_names` are keyed by
`ProgramId`, and a native entry point has none -- so the fix means giving native entry points a
declaring-package identity, which is dispatch-seam work and not Task 6's. Nothing pins the wrong
bytes: no committed expectation contains `Error 88 running`, and the only gated program reaching a
marshalling error (`directive_method_external_arguments.rex`) is user-declared and therefore
already correct. Cost if wrong: an uncaught 88 from a library-declared native method prints the
program path instead of `REXX` until the owning task lands.

**Three hypotheses were falsified before that rule survived**: "every LIBRARY REXX method reports
REXX" (would have broken the gated user-declared witness), "the declaring package always names it"
(falsified by `charin(0)`), and "the 88s report REXX and the 93s do not" (falsified by the
user-declared 88.922). Each died to one measurement, and the gated witness caught the first before
I wrote it.

**The refusal table was rebuilt, not hand-edited.** 43 rows disagreed -- 42 of them pure line
moves from inserting one constructor into `error.rs`, one genuinely new name, zero deletions. Far
past hand-editing, and the morning's clobbering came from re-keying on `(kind, name, surface)`
when a surface changed. Keyed on `(kind, name)` alone, taking surface and definition from the
derivation and carrying columns 5-8 verbatim. Asserted afterwards against a backup taken before
the script ran: **0 rows whose measured columns changed**, 42 whose surface/definition moved, 1
added, 0 removed, 8 columns throughout, non-placeholder rows 89 -> 90. All five refusal tests
green, the three siblings that caught the earlier clobbering included.

**The new row's `answer` was wrong at first and its own test caught it.** I put the rendered bound
`18446744073709551615` there; `admits` wants an identifier the constructor itself produces, and it
builds `syntax(88, 907)`, so `88.907` is admissible and the bound is not. Corrected. `88.907` is
unique among send rows, so the two recorded `SHARED_ANSWERS` groups are untouched.

**A count I quoted was wrong and is corrected here**: I said 145 non-placeholder rows from an
`awk | wc -l` that counted the header's comment lines. The real figure is 89 before the rebuild
and 90 after.

Task 6: complete.

**The new cadence's first run earned its keep.** `cargo test --release -p rexx-exec` against
`2400563c9`: exit 101, **45 ok, 1 failed, 0 strays**, 13m37s. The failure was
`collection_arity`'s `Properties~save` row, stale because Task 6 implemented `stream_lineout` and
the method now works -- `send-differs` -> `agree/rc0`, a refresh in the harness's own words.
Refreshed and **re-run without the refresh flag**, so the green is a comparison rather than the
writer's early return. The `0 strays` is the A6 harness fix holding across the whole crate suite,
not just the two binaries I checked by hand.

**Two readings of my own I had to correct mid-check.** I called the gate finished while it was
still running -- the log's last line was `has been running for over 60 seconds` and `ps` showed the
parent at 12:48 elapsed -- and I read the failure tuple backwards, taking the committed row for the
measured one. Both were caught by looking at the committed file and the process table rather than
at the log alone. The status file also never got its completion lines on the earlier attempt
because I wrapped the command in `{ ... } &` inside an already-backgrounded call; running it
directly under the harness fixed that.

**Task 7 scoping, gathered while the tree was frozen.** `Trap { call, label, delayed }` gives the
CALL/SIGNAL split; `deliver_one_pending_trap` already sets `sigl`, assigns `RC`, builds a
`Raised::condition` and saves/restores `active_condition`; `novalue_raised` is the pattern a native
raiser follows (consult `trap_for`, escalate through `condition_raises_syntax`, else
`Raised::condition`); `ConditionSyntax::escalated[4]` is already the NOTREADY slot.
`TrappedCondition { name, code_sub, call, description }` is what `CONDITION` reads, so `ADDITIONAL`
and `RESULT` are two new fields there as well as on `PendingTrap`.

**The rooting hazard is real and named.** `Interp::object_roots` (`lib.rs:4445`) is documented as
naming every field that does **not** need handing to the collector, and each ignored field carries
a justification -- except `pending_traps`, which carries none because it holds no `ObjRef` today.
The moment Task 7 puts `additional`/`result` there, those objects are unreachable while queued: a
use-after-free that only `collect_stress` would surface, and adding a field to `PendingTrap` will
not make the exhaustive destructuring complain. Task 7 must extend the walk in the same change.

**Open scope question for Task 7:** Step 1 says "build the condition directory the oracle builds",
but `CONDITION('O')` refuses loudly (`builtin/state.rs:316`) and `refusal-sites.tsv` records that
as a `diverges` row whose witness says `CONDITION('O')` is what would build it. Whether Task 7 owes
that implementation, or builds the directory only for delivery, decides whether the task is "raise
a condition" or "raise a condition and implement condition objects".

## Task 7: NOTREADY -- measured before writing

**The raise points split by cause, not by entry point.** All six that can implicitly open --
`chars`, `lines`, `charin`, `charout`, `linein`, `lineout` -- raise when the open fails; only the
reads raise at end of file. Measured 2026-09-12:

    linein at EOF          raises, answers '',  state NOTREADY:EOF
    charin at EOF          raises, answers '',  state ERROR:0
    lines/chars at EOF     raise nothing,       answer 0
    lines/chars on nosuch  raise once EACH,     answer 0, state ERROR:2
    lineout/charout to a directory   raise, answer 1, description is the name as given
    lineout to a read-only stream    raises, answers 1, state ERROR:13 Permission denied

The raise never changes the answer: the call still returns its residual, `1`, or the null string.

**`CALL ON` delivery order measured**: the clause finishes and prints its empty value, then the
handler runs at the boundary, then the next clause. In the handler `condition('I')` is `CALL` and
`('S')` is `DELAY`, which is `Trap::delayed` -- already modelled.

**`::OPTIONS NOTREADY SYNTAX` measured, where the survey said "not probed"**:
`Error 98.974:  Stream "n.txt" is not ready.` at rc 158, substituting the stream name as given.
`Error_Execution_notready_syntax = 98974` (`messages/RexxErrorCodes.h:643`). Survey A §2.8
corrected to say so.

**The queueing gap is the substantive work.** `pending_traps.push_back` exists at exactly one site
(`run.rs:3378`), the routine-return path, which consults `caller_trap_for` because its activation
is being popped. A native stream raise is mid-clause in the *running* activation, so it needs a
second queue path keyed on `trap_for` and the running activation's own id. That is the path Task 11
and Task 15 reuse.

**Ruling: no `additional`, no `result`, no condition directory** -- recorded in the plan with the
reason. Both are observable only through `CONDITION('A')`/`('O')`, which refuse loudly here and are
a committed `diverges` row; storing two unreadable `ObjRef`s would be a hollow shell and would need
`Interp::object_roots` (`pending_traps`) and `Activation::object_roots` (`condition`) extended --
both ignore those fields today precisely because neither holds an object. The task that implements
condition objects does the fields and both root walks together.

**A survey edit silently did nothing and I nearly missed it.** My first correction anchored on an
ASCII `--` where the file has an em dash, so the script matched nothing and wrote nothing; only
printing the real sentence on a failed match caught it. The second attempt asserted the anchor
before writing.

**An explicit `~open` raises too, and its errno split vindicates Task 5.** Measured 2026-09-12
with a `CALL ON NOTREADY` handler: `open('read')` on a missing name answers `ERROR:2` and raises;
`open('read')` on a directory answers `ERROR:2` and raises; `open('write')` on a directory answers
**`ERROR:21`** and raises. Task 5 already produces that split for the right reason -- a read-only
open of a directory succeeds at the syscall and is rejected afterwards by the `is_dir` check, which
substitutes ENOENT, while a read-write open fails outright with EISDIR. The site's comment says so
and is correct.

So nine raise points, not six: the three explicit-open failures, `fail_open` (covering all six
entry points that open implicitly), `charin`'s short read, `linein`'s EOF, `charout`'s write
failure, and `lineout`'s read-only EACCES and write failure. Every one sets the state, clones the
name while the borrow is live, and raises after it ends; the `?` gives the measured semantics for
free, because a `SIGNAL ON` trap abandons the clause while a `CALL ON` or untrapped raise falls
through to the call's normal answer.

**Prediction, written before the code runs:** with NOTREADY raised but untrapped, the two committed
witnesses that read at EOF -- `stream_chars_lines.rex` and `stream_line_ends.rex` -- produce
byte-identical output and the corpus stays at `477 of 477`. Untrapped NOTREADY is silent (measured,
rc 0), so nothing should move. If either does, the untrapped path is doing more than setting state.

**Task 7 works, measured on three shapes, all byte-identical to the oracle on three descriptors.**
`SIGNAL ON` -> the clause is abandoned and the handler reads
`C= NOTREADY D= n.txt I= SIGNAL S= OFF sigl= 5`; `CALL ON` -> the clause prints its own empty value,
the handler runs at the boundary with `I= CALL S= DELAY`, then the next clause; and
`::OPTIONS NOTREADY SYNTAX` -> `Error 98.974:  Stream "n.txt" is not ready.` at rc 158, stderr line
for line. The same `SIGNAL ON` program printed `no trap fired` before the task.

**The prediction written before the code existed held**: corpus went 477 -> 479 with the two new
witnesses and the two committed EOF-reading witnesses did not move, so untrapped NOTREADY really is
silent beyond the state change.

**The refusal table was rebuilt again and asserted again**: 45 derived rows, 44 rewritten, one
added, **0 measured columns changed**, 0 removals, non-placeholder rows 90 -> 91.

**The new row's four columns were wrong, in the opposite direction from last time.** I gave
`notready_syntax` a verdict and a witness, and `every_send_surface_row_is_walked` refused it: the
derivation classifies its surface as `body`, because its only construction site is
`Interp::raise_notready` in `run.rs`, not a dispatch site -- even though a message send is what
provokes it. The `surface` column is about where the constructor is *called*, not what provokes it,
and a `body` row may carry no witness. Set to `off-send-surface - - -`, with the measured
transcript kept here instead: `s~linein` past the end under `::options notready syntax`, both sides
`Error 98.974:  Stream "n.txt" is not ready.` at rc 158.

Task 7: complete. Crate suite against `a57ed3b66`: **exit 0, 46 ok, 0 failed, 0 strays**, 9m32s.

## Task 8: SEEK, POSITION, QUERY POSITION -- measured before writing

The whole chain is read: `streamPosition` parses with the same tokenizer Task 5 ported (the four
prefixes are already single-character tokens there), then transient 93.958, then missing-offset
93.903, then `state = Ready`, then the read/write defaulting, then char or line positioning.
`setPosition(offset, style, ...)` decrements for `SEEK_SET` and increments the result back to
1-based, which is the whole of the arithmetic: `<0` on a 29-byte file is 30 and `<99` is -69,
unchecked. `seekLinePosition` computes the target line by direction and clamps **only at the
bottom** to 1; `setLinePosition` uses `reclen*(line-1)+1` for record streams and
`seekToVariableLine` otherwise, which rewinds to line 1 when the target is behind or untracked and
then `readForwardByLine`s the difference -- and that function sets `stream_line_size` to the line
it reached when it could not move the whole distance, which is why `seek 99 read line` answers 4
and caches the true count.

**Ruling: port the both-open pointer collapse unconditionally.** The C++ branches on
`last_op_was_read` with a comment saying it is "currently always true" and citing `bugs:#1739`.
Task 6 maintains that flag honestly, so branching on *my* correct flag would diverge. Measured:
after a `linein`, `seek 5` gives read 5 and write 5; after a `charout`, `query position` with
neither keyword still answers the READ position, and a following `seek 3` sets both to 3. Parity
is the contract.

**Both error shapes measured uncaught.** 93.903 is `Missing argument in method; argument SEEK is
required.` at rc 163 -- one substitution, the literal word `SEEK`, so this crate's
`missing_method_argument(position)` (which renders `argument 1`) cannot be reused. 93.958 is
`Positioning of transient streams is not valid.`, no substitutions, and no 93.958 constructor
exists here yet. A bare 93 from a rejected option prints only the major line, which sub 0 already
does.

**A survey row and the oracle disagree, and it is unresolved.** Line 242 says `/dev/null
(transient): open read READY:`; measured today, `.Stream~new('/dev/null')~open('read')` answers
`ERROR:2`, while `open('write')` answers `READY:` and is `TRANSIENT`. Survey §2.7 separately says
`query exists /dev/null` answers `/dev/null`. This crate answers `ERROR:2` for the read case
because Task 5's read-only path tests `is_file()`, which is false for a character device. Being
measured next.

**The survey row was wrong and the code was right.** `SysFileSystem::fileExists` is `S_ISREG`
only (`platform/unix/SysFileSystem.cpp:198`-`:212`), so a read-only open of a character device is
rejected with ENOENT. Measured on both forms: `open read` on `/dev/null` is `ERROR:2`, `query
exists` is empty, `open both` is `READY:` and `TRANSIENT`. Survey line 242 claimed `open read
READY:` and has been corrected. Task 5's `is_file()` test is exactly `S_ISREG`, so this crate
already agreed -- the specification was the thing that was wrong.

**A live defect of mine, found by that comparison and NOT by any witness.** `query('streamtype')`
answers a hardcoded `UNKNOWN` (Task 4), which was right while nothing could open and is wrong now
that Task 5 opens: the oracle answers `TRANSIENT` for a tty or character device, `PERSISTENT` for
an open regular file, and `UNKNOWN` only when closed. `OpenFile.transient` exists and is hardcoded
`false`, so both halves are wrong. No witness opens a device, which is why five green gates missed
it.

**Neither existing constructor fits 93.903's named form.** `missing_method_argument` is 93.903 but
takes a 0-based *position* (`max(1,,3)` -> `argument 0`); `missing_named_argument` takes a name but
is 88.901. Task 8 adds `missing_argument_named`, whose doc says why a second 93.903 constructor is
not a duplicate -- the catalogue has one substitution and the raiser decides whether it is a word
or a number.

**The streamtype defect is fixed and gated.** `query('streamtype')` now answers from the open
descriptor: UNKNOWN closed, TRANSIENT for a character device or FIFO, PERSISTENT otherwise, with
`OpenFile.transient` set at both open sites from `FileTypeExt` (the crate's first use of
`std::os::unix::fs`, safe and no new dependency). Rust output is identical to the oracle on every
cell of the witness, corpus 479 -> 480.

**The witness stops at the run directory's edge, deliberately.** The TRANSIENT cell needs a
character device, and the phase's global constraints say a witness stays inside its own directory.
So the corpus row covers closed/open/closed for a regular file plus the implicit open a read
performs, and `/dev/null` stays a ledger measurement: `open both` -> READY: and TRANSIENT,
`open read` -> ERROR:2. Cost if wrong: the TRANSIENT path is measured but not gated until Task 10,
which needs transient handling for `.STDIN` anyway.

**Three clippy findings, and I misread which site was named twice.** `suspicious_open_options`
pointed at `:706`; I assumed the primary builder and fixed that, and the lint moved to `:710` --
the write-only *fallback*. Two `manual implementation of Option::map` findings at `:898`/`:947`
were `charout` and `lineout`'s argument `match`, not the `if short { ... } else { None }` in
`charin` that I grepped for and that clippy correctly left alone. The lesson is the same one that
cost me time on the refusal table: act on the location the tool names, not the one that looks
right.

Both open-options fixes were checked against the C++ rather than silenced. `implicitOpen` uses
`RDWR_CREAT` with no `O_TRUNC`, so the implicit open says `truncate(false)` -- measured, `keep me`
plus a LINEOUT of `added` is 14 bytes. Its fallback is bare `RX_O_WRONLY`, so the write-only retry
no longer passes `create`.

**Two SEEK constructors were written and then removed again.** `missing_argument_named` (93.903
with a word, not a position) and `transient_positioning` (93.958) have no callers until SEEK lands,
and `-D warnings` fails on an unused function -- so a commit carrying them would have been red at
the gate. They return with the code that uses them.

Streamtype fix committed at `eff200a3b`. Crate suite: **exit 0, 46 ok, 0 failed, 0 strays**, 9m15s.

**The seek/cache interaction is measured, and Task 6's arithmetic already covers it.** On a
three-line file opened BOTH: `lines lines` -> `3 3`; `seek 1 read` -> 1 then `lines lines` -> **`3 2`**;
`seek <1 read line` -> 2 then `lines lines` -> `2 2`; `charin` then `lines lines` -> `2 1`. The `3 2`
is the same off-by-one as after a CHARIN and for the same reason -- a READ seek clears
`stream_line_size` and char positioning zeroes the line position, so the recount stores
`count + 0 - 1`. Task 8 has to clear the cache **before** positioning and reset the line fields on
char positioning; both orders are load-bearing, and Task 6's committed `2 1` witness would move if
either were wrong.

**`OpenFile`'s character positions become `i64`.** `seek <99` on a 29-byte file answers `-69` with
the stream still READY -- `setPosition` seeks to `size - offset` and adds one, range-checking
nothing -- and a `u64` cannot hold that. Line positions stay unsigned because `seekLinePosition`
clamps them to 1 before storing; only the character positions go negative.

**The signed-position conversion, and what it uncovered.** 28 type errors, closed by one rule
rather than 28 casts: character positions are signed (`read_position`, `write_position`,
`line_read_char`, `line_write_char`), line numbers stay unsigned (`line_read`, `line_write`,
`line_size`), and the seam is four helpers that keep `u64` file offsets with one named conversion
(`offset_of`) that clamps a negative position to zero -- because a negative offset is not a legal
read and the oracle's `lseek` fails there. `read_line_from`'s returned position went signed so the
seam stays on the parameters alone. Nothing outside `dispatch/stream.rs` and `core/src/body.rs`
reads those fields, checked.

**A latent Task 6 defect, found by the type error rather than by a test.** `lineout(data, line)`
assigns the *line number* straight to `write_position` as though it were a character position. The
oracle routes it through `setLineWritePosition`, which converts: measured in survey A §2.4,
`lineout('TWO',2)` on a three-line file leaves the write position at 22, not 2 (the survey's 19 is wrong; re-measured in Task 8). It is unwitnessed
-- `stream_write_positions.rex` never passes a line number to `lineout` -- so `480 of 480` says
nothing about it. Marked at the site and left unchanged today; Task 8's line positioning owns the
conversion, and a witness goes in with it.

## Task 8: SEEK, POSITION and QUERY POSITION

Both entry points implemented; `position` and `seek` bind the same `stream_position`, and the two
registry rows are `Arity::Fixed(1)`. Three witnesses, all byte-identical to the oracle on all three
descriptors: `stream_seek.rex`, `stream_seek_line.rex`, `stream_seek_errors.rex`.

**What the C++ tables gave that guessing did not.** `ParseAction(ME, ...)` is a mutual-exclusion
check run *before* the flag is set, so a repeated token clashes with itself. `SYS` combines with
nothing; `READ`/`WRITE` clash with each other and themselves; `CHAR`/`LINE` likewise but combine
freely with `READ`/`WRITE` -- which is what lets `StreamSupplier~init` send `"READ LINE"`. I had
first written `query position read write` as "write wins"; the oracle raises a bare 93. Measured:
the mutual-exclusion refusals print `Error 93 ... Incorrect call to method.` with **no** `93.9xx`
second line.

**Four falsified predictions, each caught by running rather than reasoning.**
1. `missing_argument_named` takes one substitution, not two. The C++ passes `"SEEK"` and `"offset"`;
   the template renders only the first -- `argument SEEK is required`. My pre-compaction note was
   right and my "correction" of it was wrong.
2. `getLineReadPosition` is not a field read. When the tracker is 0 -- what a CHARIN leaves -- it
   recomputes, so `QUERY POSITION READ LINE` after a three-byte CHARIN answers 1, not 0.
3. `queryLinePosition` counts an **inclusive** range `[0, position-1]`. Exclusive agrees at
   positions 4 and 18 and diverges only at position 1, which one witness cell happened to ask.
4. The write-line tracker starts **untracked (0)**, not 1, in both constructors; with the `+1` on
   the write side that is the difference between answering 1 and the oracle's 2.

**Ruling: the seek's implicit open takes the read-shaped path.** `implicitOpen(operation_nocreate)`
never creates, so a seek naming a missing file answers `0` and leaves state `ERROR`. Passing
`for_write` created the file. Cost if wrong: a seek would create files the oracle does not.

**Ruling: the pointer collapse is unconditional.** The C++ branches on `last_op_was_read` and its
own comment says the flag is always true (`bugs:#1739`). Measured, both pointers move together
whichever operation ran last. Cost if wrong: a BOTH-open seek leaves one pointer stale.

**One cell was dropped from a witness rather than pinned.** `QUERY POSITION SYS` answers 6 after a
line recompute and 3 before one, because the oracle's `countLines` leaves the descriptor wherever
its buffering stopped. That pins internal buffering, not behaviour, so `sys` is asked before the
line queries, where both sides leave the descriptor at 3.

**The carried `lineout(data, line)` defect is now fully measured, and the survey figure was wrong.**
Bytes land at the line's character offset and overwrite in place -- `lineout('TWO',2)` on
`alpha/beta/gamma` gives `'alpha\nTWO\n\ngamma\n'`, still 17 bytes, the stray blank line being what
the shorter text failed to cover. The write pointer ends at **end-of-file + bytes written**, which
disagrees with where the bytes went; that is upstream's inconsistency, not a misreading. Fitted on
four points spanning two file sizes (17, 25) and two pre-call pointers (18, 3), which is what broke
the collinearity -- the first three all had pointer == size + 1 and admitted two rules.

| size | pointer before | written | measured |
|------|----------------|---------|----------|
| 17   | 18             | 4       | 22       |
| 17   | 18             | 6       | 24       |
| 17   | 3              | 2       | 20       |
| 25   | 3              | 2       | 28       |

`StreamInfo::lineout` explains it: `writeSetup()` runs *before* `setLineWritePosition`, and after
the write `if (lineWritePosition > 0) { ++lineWritePosition; lineWriteCharPosition =
charWritePosition; }`. The same block gives the cache rule -- `append || charWritePosition ==
size()` bumps `stream_line_size`, anything else drops it to 0. Still unmeasured: whether "size" is
read before or after a write that *extends* the file; every case here overwrote in place.

Current code writes at `as_position(line)` -- offset 3 for line 3 -- which corrupts data. The fix
is deferred to its own commit rather than folded in here, because the per-task suite was already
running against this tree and this is file-corrupting code that deserves its own witness.

**Ruling: Task 8 and the `lineout` conversion ship as one commit, not two.** The ledger earlier
said the fix would get its own; that is no longer the cheaper option. Both touch
`dispatch/stream.rs` and `corpus/phase-7.txt`, so splitting them needs per-hunk staging --
`git add -p` is interactive and unavailable here, and the alternative is a checkout over a file
already verified byte-identical against the oracle, which this project forbids for good reason.
Cost if wrong: one commit is coarser to revert than two. Cheaper than the index surgery.

**Task 8 verification, all on the tree as committed.** `fmt` 0, `clippy -p rexx-exec
--all-targets -D warnings` 0, `refusal_sites` 5 of 5, `sourceline_oracle` 1 of 1 across four new
expectation files, corpus differential **484 of 484 in STRICT mode** (480 before, four witnesses
added). The full `-p rexx-exec` release suite before the refusal table was re-derived was 46
binaries, 1446 passed, one failure -- `the_table_holds_every_constructor_the_source_defines`,
which was the stale table and is the failure the re-derivation closes. Its printed delta named
`abstract_method -> error.rs:943` and `argument_needs_a_string_value -> error.rs:1132`,
independently matching the recomputation, and left exactly one row for the deriver to correct:
`from`, where a repeated name means the deriver's last match wins and a first-match scan does not.

**Task 8 gate, over the committed tree.** `cargo test --release -j 4 -p rexx-exec --no-fail-fast`
at `8486514a4`: **exit 0, 46 binaries, 1451 passed, no failures**, 16:33:17 to 16:42:13. Committed
before gating, tree frozen throughout, so the gated tree is the committed tree by construction.
Task 8 committed at `8486514a4` ("Position a stream, and answer where it is").

**Task 9 pre-dispatch check, done while the gate held the tree.** Three corrections went into the
plan rather than into this ledger, because the plan is what the next reader opens.

*The count was stale.* The plan said "the eight excluded stream builtins" in three places.
`EXCLUDED` holds seven wholly-excluded Phase 7 stream names -- CHARIN, CHAROUT, CHARS, LINEIN,
LINEOUT, LINES, STREAM. QUALIFY was the eighth and left the set when the shadow-cwd task
implemented it (`builtin.rs`, `run: platform::qualify`). Replaced with the named set rather than a
new number, per the rule against writing a set's size down.

*A file Task 9 must touch was missing from its Files block.* `corpus/bif-exempt.txt` has 49 rows,
**44 of them blocked on Phase 7**, and the blocked rows are exactly these builtins --
`CHARIN::test_charin_simple`, `CHAROUT::test_stdout_ret2`, `CHARS::test_stdin_wrong_synatx`,
`LINES::test_count_negativeTest1`, `STREAM::test_close_twice`, `STREAM::test_open_isUsable`. That
table is policed in both directions, so implementing the builtins turns rows green and reddens the
harness until it is refreshed. Added with the refresh command.

*And one file that looked like it needed an edit does not.* Survey A §5.8 says `tests/owners.rs`
"asserts ... the eight stream names say Phase 7". It does not: it asserts every `EXCLUDED` row
names a phase in {Phase 7, Phase 10}, that `owner_of` reaches it, and -- the negative control --
that `owner_of("LENGTH")` is `None`. The loop runs over `EXCLUDED`, so removing the rows shrinks it
and it stays green untouched. Checked rather than inferred, because the survey sentence would have
had me add a phantom file to the plan.

## Task 9: the stream table and the stream builtins

Scope and placement settled by Moritz: the whole task in one commit, and the table on `Activation`
per survey A §5.5 rather than on `Interp` keyed by `ActivationId`.

**The rooting is proved rather than argued.** `Activation::object_roots` destructures the struct
exhaustively, so a new `ObjRef`-bearing field cannot reach the collector unhandled. Verified by
running: adding a probe field and compiling gave `E0027: pattern does not mention field` at
`object_roots` **and** four `E0063: missing field` errors, one per constructor. The probe was
reverted from a file copy, never with `git checkout --`. The real field went in the same way:
`streams: NameMap<Box<[u8]>, ObjRef>` keyed by the qualified path, initialised in all four
constructors, rooted with `out.extend(streams.values().copied())`. `cargo check --all-targets`
exit 0.

**Ruling: the std-name branch routes to `.STDIN`/`.STDOUT`/`.STDERR`, not to the Monitors, and
refuses loudly until Task 10 builds them.** Survey A §5.2 already decided the first half -- the
oracle's `resolveStream` maps `""`/`STDIN`/`STDOUT`/`STDERR` to the Monitors `.INPUT`/`.OUTPUT`/
`.ERROR`, and the recorded design says to route straight to the Stream objects instead, at the cost
of one `Method UNKNOWN with scope "Monitor"` line in the traceback of an *uncaught* error on a std
stream (`p08d`), to be recorded as an accepted divergence. I had been about to rule the opposite --
resolve through `.OUTPUT` and inherit its refusal -- which the survey overrides. What the survey
does not settle is that those three objects are Task 10's and do not exist yet, so Task 9 detects
the std names for real and that branch is loud with a Phase 7 owner until Task 10 fills it in.
Cost if wrong: the std builtins stay refusing one task longer than necessary. Task 9's own three
witnesses are file-based and do not touch it.

**Before-state for the witness.** `linein('f.txt')` on a missing file answers `routine "LINEIN" is
not implemented (Phase 7)` at rc 0 today, where the oracle answers `[]` and continues. That is the
divergence this task closes.

**A quirk the plan's summary flattened.** The OPEN/CLOSE/SEEK detection that decides whether a
`STREAM(name,'C',cmd)` removes the table entry is `wordPos` over the **upper-cased whole command**
(§3.1), so `stream(n,'C','query open')` takes the OPEN branch. And `STREAM`'s arity is conditional
on its second argument -- min 1, min 3 for `C`, max 2 for `S`/`D` -- which the flat `min`/`max` of
a `Builtin` row cannot express, so those checks belong in the body.

**Three derived tables move in Task 9, not one, and one of them needs new input first.**
`refusal-sites.tsv` (the two new 40.x constructors shift `error.rs` again, as Task 8's two did),
`bif-exempt.txt` (44 of 49 rows blocked on Phase 7), and `builtin-status.txt`, which pins all seven
stream names `excluded`. The last is the awkward one: it is derived by `tests/builtin_status.rs`
running each name's program from `builtin-probes.txt` through both interpreters -- and **the seven
have no rows in that file**, because it carries one probe per name that is not a whole exclusion.
So seven probe programs have to be written before the status table can move off `excluded`. The
derived status is `implemented` only when stdout, stderr and exit status all match; a row that runs
and disagrees derives `divergent`, which the harness refuses without a `KNOWN GAP` row in the
exclusions file. All four files are now in the plan's Task 9 Files block.

`Builtin::COUNT` was checked and is unrelated: `activation.rs:983` is the ten trappable condition
names, not the builtin-function table. `IMPLEMENTED` is a flat slice indexed by a `u16` built at
run time, so seven new rows need no count adjusted anywhere.

**A fifth file, and it is the one that drives the others.**
`docs/superpowers/plans/phase-4-exclusions.txt` lists the seven stream names, and
`tests/builtin_status.rs` derives the *required* probe set from it, asserted both ways. So that
file must lose the seven rows in the same commit that adds their probes: while they are listed the
harness wants no probe, and the moment they leave it wants all seven. The form is already set --
QUALIFY, USERID, SETLOCAL and ENDLOCAL are recorded there as "DELIVERED by Phase 7, 2026-09-12,
and their rows deleted".

**The probe shape is settled by running it, not by guessing.** The rules are that a probe must
elicit a computed value and must not nest one builtin inside another, which looks impossible for a
reader that needs a file to exist. It is not: a `Stream` *method* can create the file, leaving the
builtin under test as the only builtin in the program. Measured on the oracle --
`s = .Stream~new('p.txt'); zz = s~lineout('alpha'); zz = s~close; say linein('p.txt')` prints
`alpha`. That is the template for all seven rows.

**The owner predicate is measured on all four constructs, not inferred.** `TopLevel | Method` owns
a table; `InternalCall`, `Routine` and `INTERPRET` borrow. Each probe ran in its own fresh
directory against a fixture the program seeded itself, so none carries another's drift.

```
INTERPRET   main one / interp two / main three          shares
CALL inner  main three / call four / main five          shares
::ROUTINE   main one / routine two / main three         shares
o~m(f)      main one / method one / main two / method2 one   isolated, fresh per call
```

Two premises this closes. `INTERPRET` was the one I had taken from §3.9's prose without checking:
`Activation::nested` sets `entry: Entry::InternalCall`, so a fragment borrows by construction, and
the oracle agrees. And a method's table really is fresh **per call**, not per receiver -- `method2`
read `one` again.

I nearly took the predicate by resemblance to the two `Entry` matches next door in `run.rs`, and
both are different groupings: `:1570` is `TopLevel | Routine | Method` for PROCEDURE placement and
`:1831` is `TopLevel | InternalCall | Routine` for USE LOCAL. Three neighbouring predicates, three
different partitions -- which is the argument for deriving this one rather than copying it.

**Consequence for Step 4.** Because `Routine` borrows, the end-of-program `CLOSE` sweep cannot hang
off `pop_activation`, which has five call sites and fires for every frame kind: a returning
`::ROUTINE` would close its caller's streams. The sweep fires on the same `TopLevel | Method`
predicate the table owner uses.

**The seven builtins are in and derive as `implemented`.** `resolve_stream` routes the name, the
activation table holds the stream by its qualified path, and each builtin sends the message named
after it with `Args::values_from(2)` so an omitted middle argument stays omitted -- measured,
`charin(f,,3)` is `alp` and `charin(f,2,3)` is `lph`. First end-to-end differential was identical
on all three descriptors, and the corpus stayed 484 of 484.

**Three hardcoded counts had to move, and the comment above two of them was wrong.**
`builtin_status.rs` pinned `excluded == 11`, `in scope == 70` and `oracle_invocations == 70`, under
a comment saying these "are both stable facts about the C++ table and the exclusions file, not
facts about how much is implemented, so asserting them here cannot rot as a task lands". Landing
the streams is exactly what falsified that: they are now 4, 77 and 77. Moved deliberately with the
reason recorded beside them, the way `bif_assertions`'s `DERIVED` comment records `ANOMALY 3 -> 0`.
The derived assertions next to them (`excluded == wholly_excluded().len()`,
`oracle_invocations == in_scope.len()`) needed nothing, which is the argument for deriving over
pinning.

**I destroyed a committed header and had to restore it.** `builtin-status.txt` is refreshed by
copying the harness's derived output, and that output is rows only -- so `cp` over the committed
file deleted its forty-line preamble, the one documenting the four statuses and the refresh
command. Restored from `git show HEAD:` and recombined with the derived rows. The rule I broke is
the one about looking at what you are overwriting; I had read that header earlier in this session.

**Implementing the builtins made a test harness litter the repository.** `bif_assertions` runs its
extracted rows in process with `Invocation::none()`, so they inherited the test process's working
directory -- the crate directory. While the stream builtins refused, those rows wrote nothing; once
they worked, `STREAM.testGroup`'s rows dropped `streamTest.delMe` and `streamCloseTest.delMe` into
`crates/rexx-exec/` on every run. Fixed with `Invocation::with_directory`, which already exists for
exactly this and is what `corpus.rs`, `builtin_status.rs`, `ir_recorded.rs` and `collect_stress.rs`
all use: the interpreter's own cwd moves, no process state is touched (D-P7-7).

**And my first version of that fix had the defect it was fixing.** I gave all three call sites the
same directory label while the rows run under `rayon`'s `par_iter`, so every row would have shared
one directory -- one row free to truncate another's file, with the failure moving between runs.
Now keyed by a hash of the row's own program text. That is the fourth shared-fixture mistake this
session, and the first one I made while repairing another.

**The two `STREAM::test_relative_file_exists` rows are an extraction artifact, not a defect.**
Measured: `.ooTest.dir` does not resolve outside the ooTest framework -- both interpreters render
it as the literal `.OOTEST.DIR` -- so the row's expected value is a string no implementation can
produce. The harness lifts one assertion out of a group whose `::method setup` supplies both that
symbol and the working directory via `directory(filespec('L', file))`. Crate and oracle agree
exactly on the actual value. Note the harness compares crate against crate here (expression against
expected), not against the oracle, so a MISMATCH means our answer does not satisfy ooTest's
assertion rather than that we diverge.

**Ruling: Task 9 does not build the end-of-program CLOSE sweep, because no observable was found.**
`closeStreams` exists in the oracle to flush what its buffered streams still hold; this crate
writes through with no local buffer (survey A 5.3), so the bytes are already on disk when the
program ends. Three probes, all byte-identical on both sides:

```
STATE/CHARS on an open unclosed stream      READY / 4            identical
a second program reading an unclosed file   alpha, size 6        identical
p09b's shape: unclosed streams + say 1/0    rc 214, z.txt one|two, stdout AND stderr identical
```

Building a sweep on that evidence would add machinery no witness can tell from its absence, which
is the hollow-method trap wearing the other face -- not a method that does nothing, but a
mechanism whose effect is already true. Cost if wrong: a later construct makes the close
observable and the sweep has to be written then. The likely places are Task 11's monitors and a
subclass with its own `uninit`, and whoever reaches one should look here first. Recorded rather
than silently skipped, and Step 4 stays unticked.

**The per-row directory fix, second attempt, is right and is proved by a row that broke.** Keying
the directory on each *program's* text split the two halves of a value row into different
directories, and `QUALIFY`'s `assertSame(directory(), qualify("."))` compares two readings of the
working directory -- so it failed. Keyed on `group::method` instead, both sides share one
directory and rows stay isolated. The regression was mine and was not pre-existing: there are no
QUALIFY rows in the exempt list at HEAD and `bif_assertions` was green in the 1451-passing gate.
Litter is gone (0 strays, 22 files now under `target/tmp/bif-rows`).

**A pre-existing defect the stream builtins made reachable: a `RAISE` from a Rexx method body
escapes a trap armed in a called frame.** Isolated to three cases, with no stream in the program --
a bare `::class K / ::method boom / raise syntax 93.914 array(...)`:

```
handler armed in main       oracle RAISED    crate RAISED
handler armed in a label    oracle RAISED    crate escapes, rc 163
handler armed in procedure  oracle RAISED    crate escapes, rc 163
```

The controls say it is not "raises escape a called frame": `substr('abc')` (40.x) and `1/0` (42.x)
inside a `procedure` both trap correctly on both sides. It is specifically a raise originating in a
**Rexx-coded method body**. This is why `stream_builtin_errors.rex` diverges -- its `try:` procedure
cannot catch the 93.914 the orx `command` method raises. Not a Task 9 defect and not in the seven
builtins; Task 9 only made it reachable, since before this the builtins refused before reaching any
orx body. Needs its own task; the witness stays out of the corpus until it is fixed.

**A real gap in the seven, owned by Task 10.** The readers reject an omitted name where the oracle
routes it to the default input or output. Measured: `charout( ,'x')` writes and answers `x[0]`,
`lineout( ,'x')` answers `[0]`, `chars()` and `lines()` both answer `1`; this crate raises 40.5 or
40.3 instead. The cause is the row arity -- `min: 1` rejects the call before the body runs, so
`resolve_stream`'s empty-name branch is never reached. That also explains
`CHAROUT::test_stdout_start` degrading from `Phase 7` to `RAISE-MISMATCH`: the oracle reaches
93.958 positioning a transient std stream, and this crate stops at 40.5.

**Dropping the reader arities to `min: 0` introduced a panic, and the fix needed a rule I did not
have.** `Args::values_from(2)` is `&self.values[from - 1..]`, so a call that wrote no positions at
all -- which is what an omitted name is -- sliced `[1..]` on an empty slice and panicked in
`chars()`, `linein()` and `charin()`. Guarded at the call site. A wrong-but-safe 40.3 had become a
crash, which is the worse failure, and it existed for exactly as long as it took to measure the
next thing.

**An omitted or empty name resolves by direction, and the direction is the caller's.** Measured
with stdin redirected from a three-line file: `linein()` reads `l1`, `charin()` reads `l`,
`chars()` answers 9, `lines()` answers 1, and `linein('')` behaves identically to the omitted form;
`charout( ,'W')` and `lineout( ,'W')` write to stdout. The C++ says the same in its own words at
`RexxActivation::resolveStream` -- "if length of name is 0, then it's the same as omitted. This is
a request for either the default input or output stream. The flag tells us which one." Two
independent sources, one rule. `resolve_stream` now takes `input`, and `standard_stream_name` maps
the empty name to `.STDIN` or `.STDOUT` by it; before this it returned `None` for `""` and the name
fell through to `paths::normalize("")`, which would have built a stream on the current directory.

`STREAM`'s own direction argument is unreachable and that is measured, not assumed: `stream('')`
and `stream('','S')` are both 40.27 before the name is resolved. Noted at the call site rather than
left as a silent guess.

All seven omitted-name forms now give a loud Phase 7 refusal naming `.STDIN` or `.STDOUT`, which is
the honest state until Task 10 builds them, and named streams are unchanged and byte-identical.

**Task 9 verification, all green before the commit.** corpus **486 of 486** STRICT (484 plus the
two new witnesses), `builtin_status` 26 of 26 with all seven deriving `implemented`,
`bif_assertions` 5 of 5 on the committed table, `sourceline_oracle` 1 of 1 over three newly
generated expectations, `fmt` 0, `clippy` 0, zero strays under the crate tree.

**The `sourceline_oracle` trap was caught before the commit, not after.** All three new
`corpus/lang` programs were missing their expectation files, including the unfiled one -- that test
walks every program in the directory regardless of whether a subset file runs it, and it panics on
the first one missing. It went red for three tasks running earlier in this phase; checking it as
part of the close-out rather than discovering it at the gate is the difference.

**And one last shared-directory mistake, this time purely in scaffolding.** My final hand check
reported both witnesses diverging -- `chars` 20 against 6, `lines C` 4 against 1, `main []` against
`main [one]`. All of it was one throwaway script running the *oracle* twice into the same directory
before the measured run, so its `lineout` fixture had already appended. The crate ran once. The
corpus harness, which gives each program a fresh directory per spec A6, said 486 of 486 throughout.
Fifth instance of this shape today; the tell is always a figure that is a clean multiple of the
right one, and the fix is always one run per side in a directory nothing else has touched.

**Task 9 committed at `3a06e0276`** ("Resolve a stream by name, and let the builtins say it"), 21
files, 835 insertions. Tree clean, gate started over that commit.

## Task 10 pre-dispatch check, done while the gate held the tree

Two corrections went into the plan, both cases of the plan describing a design that is not what
Phase 7 actually built.

**Survey A 5.1's `kind: Kind` never existed.** It proposes `Kind::{Closed, File, Std, Handle}` and
flat `read_only`/`write_only` bools; what Tasks 3-8 built is `open: Option<OpenFile>`,
`mode: OpenMode`, and `standard: Option<StandardStream>`. So Step 1's "whose kind is `Std`" has no
referent. The real marker is `standard`, and it has exactly two readers today -- `std_set` writes
it and `open` short-circuits on it, answering `READY:` without touching the file system. Nothing
reads or writes *through* it, so Task 10 is where it acquires meaning.

**`.SYSCARGS` cannot be built from what `Invocation` keeps.** Step 3 asks for an `Array` of the
separate command-line words, and survey C 2.2 measures `1 "2 3" 4` as `Array 3` with items `1`,
`2 3`, `4`. But `Invocation` stores a single joined `argument: Option<Vec<u8>>` and
`join_command_line` collapses the words into it, so the separate words are gone at construction and
no amount of splitting recovers them (`"2 3"` is one word containing a blank). `Invocation` has to
retain the word list, and the `rexx-run` entry point has to pass it. Recorded in the plan rather
than discovered mid-task.

Two facts worth keeping for the same task. `Interp::write_out` appends to the same `out` buffer
`SAY` uses, which is exactly what lets survey C 2.3's interleaving hold
(`S1 O1 D1 C1S2 L1 C2L3 S3 0` on stdout, `E1 E2 D2 C3T1 L2 C4L4 tail` on stderr, no final newline);
`.STDERR` writes go to the `trace` sink. And `.local` holds exactly ten names, nine of them Phase
7's -- `local_owner` already splits `STDQUE` off to Phase 10.

**Independent corroboration for the Step 4 ruling.** Survey A 5.1 says a `Drop` of `std::fs::File`
"closes the fd but never writes `wr`", so the flush has to happen in `close()`. That reasoning is
about a design carrying a pending-output buffer. This crate has no `wr` -- it writes through -- so
the sweep has nothing to flush, which is why three probes found no observable. The ruling and the
survey agree once the buffer's absence is taken into account.

**The Task 9 gate went red, and the failure was worth having.** Exit 101, 4 binaries, 6 failures at
`3a06e0276` -- every one a place still asserting the exclusion boundary Task 9 moved, none a
behaviour defect. **Three of them live in `#[cfg(test)]` modules inside the library, which a
`--test <name>` run never compiles**, so no targeted run could have found them; only
`-p rexx-exec --lib` or the full suite does. That is the same cadence hole this phase already
recorded once, in a new disguise: I ran five targeted binaries and believed the crate was covered.

Seven pinned facts across five files, from one boundary move:

```
builtin.rs      dispatch declines a whole exclusion   CHARIN -> RXQUEUE
eval.rs         a whole exclusion stays loud          CHARIN -> RXQUEUE, and its justification rewritten
run/tests.rs    the refusal-message table             the `linein()` row deleted
coverage.rs     EXCLUDED_BUILTINS.len()               14 -> 7
coverage.rs     wholly_excluded().len()               11 -> 4
coverage.rs     in_scope                              70 -> 77
keyword-exempt  CALL::test_on_name                    Phase 7 -> RAISED
```

Plus the exclusions file's own header, which carries the in-scope count a fourth time in prose and
which `coverage.rs`'s comment says must stay synced -- 70 to 77. It was accurate before this change,
and I nearly committed a message claiming it had drifted; caught by reading the line instead of
trusting the sentence I had already written about it.

**I corrupted `refusal-sites.tsv` and reverted it rather than repairing it.** Re-deriving column 4
with a scanner of my own reported 98 moved rows against the deriver's 96, and `accessor_variable`
jumped 509 -> 1228 -- my scan had matched a same-named function with a different return type,
because I had disabled the `-> Loud|Raised` check while switching from first-match to last-match.
Restored from the commit with `git checkout HEAD -- <path>` (safe here: the file was committed
clean and nothing uncommitted was in it) and reapplied from the deriver's own printed tuples, 95
rows. Third time this phase that reimplementing a harness's derivation has been worse than reading
its output; the rule is to take the tool's answer.

Repairs committed separately rather than amended, since `3a06e0276` is already gated and amending
is forbidden here.

## Task 10 reconnaissance, done while the second gate held the tree

**The four monitor crashes are already in `corpus/oracle-crashes.txt`.** Survey C 2.9 says they are
not, and says the survey cannot add them because it is read-only -- but all four are there now
(entry 11a and its neighbours), so somebody acted on it after the survey was written. Checked
rather than trusted: I was one step from "fixing" a file that needed nothing. The hazard is real
for Task 10 either way, since two of them sit in the territory Steps 1-4 explore --
`.local~remove('OUTPUT')` then `LINEOUT(,)` is rc 139, while `SAY` alone survives.

**The routing state is exactly two things** (C 2.5, and it is measured rather than argued): the
`OUTPUT`/`INPUT`/`ERROR` entries of `.local`, and each Monitor's destination stack. Redefinition
cannot reach it -- `.stream~define` is 98.985 and `.output~setmethod` is 98.991. The `.nil`
fallbacks are asymmetric and easy to get wrong: `.local~output = .nil` still sends SAY to stdout
while `LINEOUT(,)` fails 97.1 on `.nil`; a *removed* entry prints from SAY and crashes the oracle
from `LINEOUT(,)`. The 97 report through a Monitor has its own shape -- program name `REXX`, line
1457, and a `(no source available)` traceback line ahead of the program's own clause.

**Three things that shrink Task 10.** `pull_line` has exactly one caller
(`parse_template.rs:546`), so D-C4's "PULL raises NOTREADY at end of input, against the
documentation" is a one-site change. `.STDIN`'s end-of-input rule is that every *read* raises a
trappable NOTREADY with description `STDIN` and the *counts* do not. And D-C7 confirms from the
design side what I derived from the code: `Invocation` needs `c_args: Option<Vec<Vec<u8>>>`, and
`rexx-run` already holds the words -- its comment explains that it joins them deliberately, so this
is changing a documented decision rather than filling an oversight.

**A trap D-C8 names explicitly.** ooTest pushes three destinations per test and never pops them, so
the monitor stack must not be capped and a re-push of the current default must not be treated as a
no-op. That would be invisible until the suite ran.

**Task 9 gate, green over the repaired tree.** `cargo test --release -j 4 -p rexx-exec
--no-fail-fast` at `87c3a6ef9`: **exit 0, 46 binaries, 1451 passed, no failures, 0 strays**,
18:11:24 to 18:22:40. All four binaries the first gate reddened -- `coverage`,
`keyword_assertions`, `refusal_sites` and the library's own unit tests -- ran in this pass and
passed. Task 9 is complete: `3a06e0276` for the work, `87c3a6ef9` for the boundary pins it moved.

**Cadence note for the next task.** The three library failures were invisible to every targeted
run because `cargo test -p rexx-exec --test <name>` compiles integration binaries only and never
the `#[cfg(test)]` modules inside `src/`. Adding `-p rexx-exec --lib` to the per-task checks costs
1.1 seconds and would have caught half of the six.

## Task 10 Step 1: the ordering question, settled

**`build_environment` can mint `Stream` instances, so `.local`'s entries are built at construction
rather than lazily.** The question was whether the environment model is forced before the library
bootstrap installs `StreamClasses.orx`. It is not: `environment_model()` is called from eleven
sites and **all of them are inside `environment.rs`**, and the only execution route in is
`dot_variable`, reached from `eval.rs:363` when a program evaluates a `.NAME`. The bootstrap
installs classes without evaluating one, so the model is built on a *program's* first dotted
symbol, strictly after `library_bootstrap` clears. Confirmed live: `.Stream~id` is `Stream` and
`.Monitor~id` is `Monitor`, so both classes the task needs are reachable.

That removes the lazy-entry design the ordering risk would have forced. A `Stream` is
`Body::Instance` carrying `NativeState::Stream(StreamState::new(name, qualified))`, and `standard`
is a plain field, so the entries can be constructed directly instead of sending `NEW` and then
`!std_set` -- which also avoids re-entering dispatch while the environment is being built.

**An unrelated Phase 5 gap, found by a probe line and recorded rather than worked around.**
`\==` applied to one of the interpreter's own objects refuses -- "the operator `\==` applied to one
of the interpreter's own objects is not implemented (Phase 5)" -- where the oracle answers `1`
(`say (.local \== 'x')`). Reachable today, owned by Phase 5, and it makes the natural identity
probe (`.local \== '.LOCAL'`) unusable for anyone writing one.

**A directory's entries are traced, confirmed at the collector's own walk.** `Body::trace`
(`rexx-core/src/body.rs:716`) has `out.extend(native.entries.values().copied())` in its
`Body::Native` arm, and `.local` is held by `roots.add_global`. So three `Stream` instances stored
as `.local` entries are reachable and cannot be swept. Read rather than inferred, because an
unrooted entry fails only under collection pressure -- and the empirical probe I reached for could
not have answered it anyway: `.environment~setentry(...)` is itself unimplemented
(`method "SETENTRY" of class "Directory" is not implemented (Phase 5)`), which is a fourth
incidental Phase 5 gap this session, reachable today and owned elsewhere.

**Step 1's mint, fully specified from measurement.** `new_instance(class)` does the abstract check,
takes the instance behaviour handle, allocates `Body::Instance` with fresh `ScopePools`, roots with
`push_temp` and registers `UNINIT` -- and sends nothing, so calling it from `build_environment`
cannot re-enter dispatch. On top of that: `native = NativeState::Stream(StreamState::new(name,
name))` with the **bare** name as the qualified one (measured, `.stdout~qualify` is `STDOUT`, not a
path -- the one stream in the system that is not normalised against the cwd), `status: Ready`
(measured, `~state` `READY` and `~description` `READY:`), and `standard: Some(...)`, which is what
parts it from the ordinary `Stream` a program builds for the same name.

Placement: the helper lives in `dispatch/stream.rs` rather than `environment.rs`, because
`new_instance` is private to `dispatch.rs` and only its *child* modules can import it --
`environment.rs` is a sibling. That also keeps stream construction beside the rest of the stream
code.

**My ordering proof was wrong, and the standard streams must be minted lazily.** I argued that
`build_environment` runs after the library bootstrap, because `environment_model()` is reached only
through `dot_variable` and only a *program* evaluates a `.NAME`. The second half is false: the
embedded bootstrap is Rexx, and it evaluates `.NAME` itself -- `CoreClasses.orx:55`
`.environment~objectname = ...` and `:65` `.environment~put(class, name)` -- and `CoreClasses.orx`
installs **before** `StreamClasses.orx`. So the environment model is built while `Stream` does not
yet exist, `classes().lookup("Stream")` answers `None`, and the `expect` I wrote fired on every
program: **202 library tests failed** plus `unreachable_classes` and two corpus tests. Reverted
from a file copy.

The probe I had taken as confirmation could not discriminate. `.environment~at('STREAM')` answering
`Stream` is consistent with the class being registered at build time *and* with
`CoreClasses.orx:65` putting it there afterwards; and `.Stream` resolving at run time comes from
`rexx_package_class`, which searches the library packages' public classes **before** it ever reaches
`classes().lookup_upper`. Two different tables, one probe that both explain. Third time this
session: a confident claim, a true premise, a missing second premise.

**Ruling: mint `.STDIN`/`.STDOUT`/`.STDERR` on first lookup rather than at construction.** The
`directory_lookup` miss in `dot_variable`, ahead of the `unbuilt_owner` refusal, is the point;
`directory_entry` already holds the directory handle, so mint-and-store there adds no `admit(` call
site and leaves the chokepoint count at the two `environment_seam.rs` pins. Cost if wrong: the
entries appear on first use rather than at startup, which is unobservable to a program -- the
oracle builds them in `LocalServer~initInstance`, and nothing can look at `.local` before its own
first clause. And an `expect` never goes in `build_environment` again: a panic there is every
program, where a refusal is one name.

**The mint was unreached because I looked in the wrong class table -- twice.** `classes().lookup`
searches `by_name`, and a `::CLASS` directive calls `define_unregistered_class`, whose own contract
is that it records the class **"without registering the name"** (`registry.rs:106`, and
`lib.rs:3401` is the orx install path that calls it). So `Stream`, declared by the embedded
`StreamClasses.orx`, is never in `by_name` and `lookup("Stream")` cannot find it. The `?` in the
fold returned `None` silently and the Phase 7 refusal stood. `rexx_package_class` -- which searches
the library packages' public classes and only then the registry -- is the route `.Stream` itself
resolves through and the one Task 9's `resolve_stream` already used successfully.

**And the probe I wrote to settle it could not see its own subject.** It built a fresh
`Interp::new()`, ran the program through `crate::run_program` (which uses a *different*
interpreter), then queried the un-bootstrapped one -- so both tables answered `false` and the
result said nothing about either. Fourth check this session that was blind to what it was
measuring. The honest version is a program: `.Stream~id`, `.environment~at('STREAM')~id` and their
identity comparison all agree inside a bootstrapped interpreter.

**What the lazy fold got right, and it is worth keeping.** No panic (789 of 789 library tests pass
where the eager attempt failed 202), and the seam counts stay 2/2 with `environment_seam` green,
because the mint reuses the clearance and the handle `directory_entry` already holds rather than
spending a second one. `Admitted` is deliberately not `Copy`, and that is what forced the mint into
the right place.

### Task 10 Step 1 -- the standard stream objects, measured

**`~string` is fixed by assigning the object variable, not by sending `INIT`.** `::METHOD string`
is `expose stream_name`, and a hand-minted instance runs no `INIT`, so the unset exposed variable
rendered as its own derived name (`STREAM_NAME`). `set_pool_variable(object, class, b"STREAM_NAME",
named)` in `standard_stream` fixes it; verified against the oracle on one program over three
descriptors, byte-identical.

**Ruling: do not send `INIT` to the minted stream.** `stream_init` normalises a non-empty name
against the cwd and replaces `native` wholesale, so an `INIT` send would answer `<cwd>/STDOUT` for
`~qualify` and discard the hand-set `status`/`standard`. Measured, the bootstrap object and
`.Stream~new('STDOUT')` are genuinely different: `~qualify` is the bare `STDOUT` against a full
path, `~state` is `READY` against `UNKNOWN`, and `==` is `0`. Sending `INIT` would have produced
the wrong one of the two. Cost if wrong: a minted stream diverges from a program-built one in some
field not yet probed -- checked by running the whole query set, not by reasoning.

**Ruling: answer `query streamtype` faithfully instead of recording a divergence** -- but the
first instrument for it was wrong and is withdrawn. `std::fs::metadata("/proc/self/fd/<n>")`
reproduces the oracle exactly under `rexx-run` and makes the same program answer differently under
`cargo test` or a library embedding, because the interpreter never writes to fd 1: it appends to
`Interp.out` and the embedder decides what becomes of it. Deriving a Rexx-visible answer from
process-global state is what the phase constraints forbid. The transience of a standard descriptor
travels on `Invocation` beside `input`, `directory` and `environment`, defaulting to transient --
the terminal case, and what the plan was already willing to ship everywhere -- with `rexx-run`
setting it from the descriptors it legitimately owns. Cost if wrong: an embedder that does not set
it sees `TRANSIENT` and 93.958 where its real descriptor is a file. No dependency; `rustix` stays
unadded.

**The diversion's specification, measured on the oracle.** One position, not three: `PULL` (queue
empty), `.stdin~linein` and `PARSE LINEIN` all advance the same cursor, so `.stdin` must read
through `Interp::linein_line`'s own `Input` rather than a second reader. One buffer: `charout('A')`,
`SAY 'B'`, `charout('C')`, `lineout('D')`, `SAY 'E'` gives `AB\nCD\nE\n`, which routing through
`write_out` produces for free. `.stderr` lands on its own descriptor. `.stdin~lineout` raises
NOTREADY and answers `1`; `.stdout~linein` answers `''` and `.stdout~chars` answers `0`, neither
raising. State is per stream: exhausting `.stdin` leaves it `NOTREADY`/`NOTREADY:EOF` while
`.stdout` and `.stderr` stay `READY`.

**`lines`/`chars` need no buffering, but the first version of this entry got the shape wrong.**
They are seekability-dependent -- a regular file answers real counts (`3`/`17`, falling as lines
are read), a pipe and `/dev/null` answer `1`/`1`. **`ProgramInput::Nothing` is a live-but-empty
descriptor, not a pre-exhausted one**, and this entry first said the opposite; modelling it as
exhausted is what failed the corpus gate at 486 of 487. Measured, the counters differ from each
other and from themselves: `chars` answers `1` on every ask until a read finds the end and `0`
after, while `lines` answers `1` on the *first* ask and `0` on every ask after with no read in
between -- and `LINES('N')` is that first ask. `Input` carries `exhausted` and `lines_asked` for
exactly those two rules. `Bytes` has its remaining count in the `Cursor`; only `rexx-run` uses
`ProgramInput::Stdin`, which is why every differential run through it was green while the gate
was red -- the wrong `ProgramInput`, twice in one task.

**`state`/`description` need no new state.** Both read `StreamState.status` alone and never `open`,
so a standard stream carrying `NotReady` renders `NOTREADY:EOF` without an `OpenFile`.

**Two probes this task could not see their own subject, the same shape as the two already recorded
above.** The `streamtype` reading of `TRANSIENT` was my own harness piping stdout into `head`, not
a property of the object -- a regular file answers `PERSISTENT`, and the A/B settled it. The close
probe reported on stdout *after* closing stdout, so every line it was written to print was
destroyed; re-run reporting into a file, it showed the plan's claim was right and my first reading
of it was wrong. Both were caught by re-running, neither by rereading.

**A program-built `.Stream~new('STDOUT')` is not the bootstrap object, and does not share its
buffer.** Measured, one program, `SAY 'AAAA'` then a built stream's `LINEOUT`s interleaved with
more `SAY`s: to a pipe the oracle's descriptor carries `AAAA BBBB CCCC BUILT-1 BUILT-2` -- the
built stream's writes buffered and flushed at exit -- and to a regular file `BBBB CCCC BUILT-1
BUILT-2`, `AAAA` overwritten because the built stream writes from its own position at offset 0.
It creates no file named `STDOUT`; it holds the real descriptor. The plan's Step 1 said such a
stream "still writes to the same buffer", which is false, and its text is corrected.

**Ruling: route every `standard`-marked stream to the sink regardless, and record the ordering
divergence.** `StreamState::standard` is set by `!std_set` from the name, so it cannot tell a
bootstrap object from a built one, and adding a mark that could would send built ones down
`ensure_open` -- creating a real file named `STDOUT` in the program's working directory. A stray
file is a worse failure than an ordering difference, especially under A6 where corpus programs own
their directories. Cost if wrong: writes through a built standard-named stream interleave with
`SAY` instead of being flushed at exit.

**`bif_assertions` went red on the exempt-set assertion, and it is the boundary moving rather than
a defect.** `CHAROUT::test_stdout_ret2#1` now passes while still listed, and
`CHAROUT::test_stdout_start#1` was attributed `Phase 7` and now measures `RAISE-MISMATCH`. The
second is not a row to regenerate away: the file's own header makes an absorbed improvement as red
as a regression, and `RAISE-MISMATCH` means both sides raise with different conditions, which is a
behaviour difference this task introduced. It is read before the table is re-derived.

**93.958 for a positioned standard stream, and the instrument it needed.** Measured on the oracle:
positioning a standard stream raises `93.958 Positioning of transient streams is not valid` only
when the descriptor is transient -- the same `CHAROUT( , 'xxxxx', 15)` writes and answers `0` with
fd 1 on a regular file. It applies to all three streams and to reads as well as writes: measured,
`.stderr~charout('x',15)`, `.stderr~lineout('x',2)`, `.stderr~charin(5)`, `.stdout~charin(5)` and
`.stdout~linein(2)` all raise on pipes and all answer normally on files. An earlier note here
saying `.stderr` was exempt was wrong -- that probe had fd 2 on a file.

**Ruling: the transience of a standard descriptor travels on `Invocation`.** A first version read
`/proc/self/fd/<n>` from inside the interpreter. It matched the oracle under `rexx-run` and is
wrong in principle: this interpreter writes to `Interp.out`, not to a descriptor, so the host
process's fds are not the embedding's, and a Rexx-visible answer must not depend on them.
`with_standard_transient([bool; 3])` defaults to all transient (the terminal case); `rexx-run`
fills it from `/proc/self/fd`, which is legitimate there because that process owns them. The
`Interp` field was classified in the exhaustive destructure rather than waved past it -- `E0027`
caught it, as it caught the `Activation` field in Task 9.

**An unexplained disagreement, recorded because it is not solved.** With the direct read in place,
`bif_assertions.rs` classified `CHAROUT::test_stdout_start#1` as `RAISE-MISMATCH` on three
consecutive runs while a byte-identical copy of the same file, same lib, same tree, classified it
`Pass`. Six explanations were tested and falsified: a stale test binary (forced relink, unchanged),
the 60s deadline, the per-row working directory, a rayon race (64 parallel runs, 64 identical),
the wrong row being selected (the harness's own selection path was driven, and it picks the row
that passes), and nondeterminism (three runs, identical). Direct reproduction through
`run_program`, through `watchdog::run_bounded`, and with the harness's own directory all gave
exit 163 with the correct condition. The disagreement went away with the process-global read.
**The cause was never isolated**, so the read is not to be reintroduced to investigate it.

### Task 10 Step 1 -- committed at `5dcc4513a`, and the gate

`Give .STDIN, .STDOUT and .STDERR objects that answer`, 12 files, 707 insertions. Fast checks
before committing, each status read unpiped: `fmt` 0, `clippy --workspace --all-targets -D
warnings` 0, `--lib` 793 passed, `bif_assertions` 5 passed, `REXX_CORPUS_GATE=1 --test corpus` 27
passed. Committed first, then the five gates over the frozen tree; the status file's first line is
the commit sha and HEAD still matches it, so the gated tree is the committed tree by construction.

```
G1 fmt                exit=0
G2 clippy             exit=0
G3 release-workspace  exit=0     (120 binaries reporting)
G4 debug-gated        exit=101
G5 corpus-release     exit=0     (27 passed, 1 ignored)
```

**G4's two failures are inherited, and that is measured rather than argued.** `gate_table_c`
reports 82 rows of `gate-tables/methods/file__instance.rex` and `gate_table_d` reports
`requires__library__subkeyword.rex` and `routine__external__subkeyword.rex`. A detached worktree at
`HEAD~1` (`87c3a6ef9`), built in its own `CARGO_TARGET_DIR` so the two revisions could not share a
binary, fails with the **identical** rows and counts: 82 and 2. This commit touches neither table.

**Why they are red at all, and when they go green.** `gate_tables::CLOSED_PHASES` contains `7`, so
Phase 7's rows are gated *while the phase lands tasks* -- its own comment says a verdict of its own
moving is a regression rather than progress. The failing rows are owned by tasks not yet built:
`File` instance methods are Task 13, and `::REQUIRES LIBRARY`/`::ROUTINE EXTERNAL` are Task 17.
Task 22 Step 2 already schedules re-deriving the gate tables so the `File` rows read `answers`. So
G4 stays red for the middle of this phase by construction, and closing it is those tasks' work.

**Two process notes.** The first `HEAD~1` run passed both tables to one `cargo test` without
`--no-fail-fast`, so it stopped at table C and never reached table D -- the same trap CLAUDE.md
names, hit while checking something else. Table D was re-run alone. And earlier Phase 7 tasks
recorded their gate as `cargo test --release -j 4 -p rexx-exec --no-fail-fast`, one package in
release, which never runs G4 at all; that is why nothing noticed these rows before.

**`rust/CLAUDE.md:59` is stale for the duration of this phase.** It says the debug gated run is
"Expected against the tree as committed: exit 0, no failures". With `7` in `CLOSED_PHASES` that
cannot hold until Tasks 13 and 17 land. Left unedited deliberately -- it is Moritz's standing-rules
file and the caveat is his to make -- and raised in the report instead.

### Task 10 Steps 2 and 3 -- the monitors and `.SYSCARGS`

**Step 2, the five monitors, matched the oracle on the first build.** All seventeen compared lines
byte-identical on both descriptors: class `Monitor` for each, the object names
`The INPUT monitor` / `The OUTPUT monitor` / `The ERROR monitor` / `The DEBUG INPUT monitor` /
`The TRACE OUTPUT monitor`, and all five stacking identities `1`
(`.input~current == .stdin`, `.debuginput~current == .input`, `.output~current == .stdout`,
`.error~current == .stderr`, `.traceoutput~current == .error`). `.local~objectname` already
answered `The Local Directory` before this task.

**Ruling: build the whole `.local` bundle on the first ask for any of its names.** The monitors
stack, so minting one alone would either mint its target twice or leave
`.debuginput~current == .input` false. Storing each as it is built also keeps it rooted through the
directory across the sends that follow, and the early-return guard makes the bundle re-entrant-safe.
`Monitor` needed no native code: it is pure Rexx in the embedded `CoreClasses.orx`, and the crate
already answered `.monitor~new(.stdout)~current == .stdout` before this task.

**`~destination` is a setter, and a probe that reads it destroys its subject.** Measured: `Monitor`
keeps a queue of destinations, `~destination` with an argument pushes and **without one pops**, so
`say .output~destination == .stdout` left the monitor empty and the next `SAY` died with
`Object "The NIL object" does not understand message "SAY"`. `~current` is the reader.

**Step 3: `.SYSCARGS` is never absent, and it is not derived from `ARG(1)`.** Measured both ways,
byte-identical: with no words an `Array` of `0` items at `~dimension 0`; with `one "two three"
four`, three items at `~dimension 1`. `~append` through one ask is visible through the next, and
leaves `ARG(1)` unchanged -- two independent derivations of the same words, which is why
`Invocation` now keeps `words` beside the joined `argument`. **`Body::array`, not `.Array~of`'s
shape**: an empty `.SYSCARGS` answers `~dimension` `0` like `.array~new`, where `.array~of()`
answers `1`.

**A Phase 5 gap recorded because nothing else records it: `==` applied to an `Array` refuses.**
`rexx-exec: the operator `==` applied to an array is not implemented (Phase 5)`, rc 120. It is in
no exclusions file, no `unfiled.txt` row and no ledger entry. It cost a probe: comparing
`.syscargs` with itself produced an empty stdout and rc 120, and the first diagnosis blamed the
probe's shell quoting -- the broken form ran fine, and the cause was in the stderr that went
unread. Identity is shown by mutation visibility instead.

**Every `Directory` accessor a `.local` witness would use also refuses as Phase 5** -- `ALLINDEXES`,
`HASENTRY`, `ENTRY`, `ITEMS` -- so survey C 2.2's `.local~allindexes` half cannot be witnessed
today. `==` on *instances* works, which is how the monitor identities above were measured.

### Task 10 Steps 4 and 5 -- positioning, the descriptor queries, and the witness

**`QUERY HANDLE` and `QUERY STREAMTYPE` now answer from the standard stream itself.** `handle` is
`0`, `1`, `2` by `StandardStream`, stable under redirection; `streamtype` reads the `Invocation`'s
transience triple ahead of the `open` logic, so it answers without the stream being opened.
Measured byte-identical both ways: fd 1 on a regular file gives `TRANSIENT PERSISTENT TRANSIENT`
across the three, and fd 1 on a pipe flips the middle one. The two existing witnesses that pin
these for *ordinary* streams -- `stream_unopened_queries.rex` (`''` and `UNKNOWN` unopened) and
`stream_streamtype.rex` (the open/close transitions) -- both stay green, because the new branch
sits ahead of the old logic rather than replacing it.

**One divergence found by probing rather than by any test: the first `~close`.** Measured on the
oracle, closing a standard stream answers `READY:` and leaves `UNKNOWN`; closing it again answers
the null string; `~flush` answers `READY:` on all three, before or after. This crate answered the
null string for the *first* close, because `close()` keys on `open.take().is_some()` and a standard
stream never has an `OpenFile` -- the Step 1 design that keeps every reader and writer unchanged.
The state to distinguish the two closes is already carried in `status`.

**`local_std_objects.rex` is the Step 5 witness, and it passed on its first run** -- 29 lines
byte-identical on both descriptors, including five identity assertions that had never been measured
when it was written and were confirmed separately afterwards. It asserts names and identities
through the objects themselves rather than through `.local`, because every `Directory` accessor a
directory-shaped witness would need refuses as Phase 5.

**Gates at this point**: corpus gate exit 0 with the new witness in `phase-7.txt` (27 binaries),
`bif_assertions` 5 passed with `186 of 186` raise rows, `--lib` 793, `fmt` 0, `clippy` 0,
`sourceline_oracle` 1, corpus coverage 1.

### Task 10 closed -- committed at `f1acca259`, gated

`Stack the monitors over the standard streams, and answer .SYSCARGS`, 8 files, 314 insertions.
Committed before the gate; the status file's first line is the commit sha and HEAD still matched it
when the run finished, so the gated tree is the committed tree by construction.

```
G1 fmt                exit=0
G2 clippy             exit=0
G3 release-workspace  exit=0     (120 binaries reporting)
G4 debug-gated        exit=101   (predicted)
G5 corpus-release     exit=0     (488 of 488 matching)
```

**The G4 prediction was written into the status file before the run and held exactly**: 82 rows of
gate table C (`file__instance`, Task 13's `.File`) and 2 of table D
(`requires__library__subkeyword`, `routine__external__subkeyword`, Task 17's), the same two test
names and the same counts proven inherited at `HEAD~1` for the previous commit. Nothing else in G4
failed, which is what makes the run informative rather than merely red.

Task 10 is complete: Steps 1-5 implemented and witnessed, Step 6 committed. Two witnesses --
`std_stream_writes.rex` and `local_std_objects.rex` -- are inside the 488, each with a
`sourceline_oracle` companion. `bif-exempt.txt` is at seven rows and raise rows are `186 of 186`.

### Task 11 -- the instruction baseline, taken before any edit

Step 4 gates on `rexxcps` and `emptyloop` instruction counts with a 1% regression floor, so the
pre-change reading is taken at `f1acca259` with the tree untouched. Release build,
`perf stat -e instructions:u`, three runs each:

```
emptyloop   9,471,649,898   9,471,645,296   9,471,648,028     spread 4.6e3   (~0.00005%)
rexxcps    20,573,788,370  20,573,688,780  20,573,575,224     spread 2.1e5   (~0.001%)
```

The spread is three to four orders of magnitude under the 1% floor, so that floor measures the
change rather than the instrument. `perf`'s `instructions:u` counter is available in this session
(146,634 for `true`) -- checked deliberately, because an earlier truncated reading showed no counter
line and a wrong "counter unavailable" call would have retired the measurement permanently.

**Hazards written down before the code, because three are oracle SIGSEGVs rather than divergences.**
`corpus/oracle-crashes.txt` entry 11: 11a `.local~remove('OUTPUT')` then `LINEOUT` with the name
omitted (SAY survives, the builtin dies); 11b the same for `INPUT` then `linein()`; 11c
`.traceoutput~destination(.nil)` then a traced clause -- the trace lines land on stdout and the
process dies after, and restoring the destination first does not save it. Step 5 asserts the
crate's own answer for those as a self-witness and records it as licensed; they are never run
against the oracle.

**What survey C measured that the implementation would otherwise get wrong**: `~destination(d)`
returns the **new** top, not the previous one; the pop returns the new top after popping; a bare
`say` sends `SAY` with one *null-string* argument rather than none; `LINEOUT('STDERR', ...)` goes to
the `.ERROR` monitor rather than to `.stderr`; every trace line is a `LINEOUT` of a `TraceObject`
(which this crate already has); and the 97 report *through* a Monitor names the program `REXX` at
line 1457 with a `(no source available)` traceback line, where the same failure without a Monitor
in the way names the program.

### Task 11 -- what is reachable, and one live divergence found before writing code

**The oracle re-reads the route per `SAY`, measured.** `say 'one'`, `.local~output = .stderr`,
`say 'two'`, `.local~output = .stdout`, `say 'three'` puts `one` and `three` on stdout and `two` on
stderr. No caching, so Step 4's fast path has to be invalidated by an assignment to `.local` -- which
is what its generation counter is for, and the reason to build one even though two of the three
mutators refuse today.

**A live divergence, and the most misleading possible intermediate state.** With
`.local~output = .stderr`, the oracle puts every later `SAY` on stderr; this crate stores the
assignment -- `.output~string` answers `STDERR` on both sides -- and `SAY` keeps writing to stdout.
So the crate *reports* the redirect took effect while the bytes say otherwise. That is Task 11's
whole subject rather than a regression, but it is worth naming: until Step 1 lands, a program can
observe the route being set and not honoured.

**What of survey C is reachable in this crate today**, measured one construct at a time:
`~destination` (push and pop), `~current`, `.local~output =`, `.monitor~new(...)~lineout` and
`.output~say` all run; `.local~setentry`, `.local~remove` and `.local~entry` all refuse as Phase 5.
Survey C §7 already lists `Directory~remove` as a blocker "hit repeatedly here", so this is
pre-recorded rather than new. The consequence for Step 5: survey §2.5's removal and replacement
witnesses (p03g, p03g2, p03e, p03f, p14) cannot be written until those land, and p03g is an oracle
SIGSEGV anyway.

**Survey §6's T2 slice already ruled the crasher case**: for `LINEOUT(,)` with no `OUTPUT` entry the
crate's own answer is a decision, recommended as 97.1 on `.nil` as for p03e and recorded as
licensed. Not measured against the oracle, ever -- it is `oracle-crashes.txt` entry 11a.

**No op consults `.local` per execution today**, so a per-`SAY` route check is new cost on the hot
loop the 1% floor measures. The baseline above is what it is measured against.

**The gap is three routes, and it is witnessable today.** Measured with `~destination(.stderr)`
pushed onto `.output` -- the mutator that *does* work in this crate -- oracle against crate:

```
A-say                 oracle stderr   crate stdout    <- SAY ignores the redirect
B-output-lineout      oracle stderr   crate stderr    <- agrees (sent to the monitor directly)
C-lineout-omitted     oracle stderr   crate stdout    <- LINEOUT(,) ignores it
D-charout-omitted     oracle stderr   crate stdout    <- CHAROUT(,) ignores it
E-stdout-direct       oracle stdout   crate stdout    <- agrees (bypasses, correctly)
```

So exactly `SAY`, `LINEOUT(,)` and `CHAROUT(,)` are unrouted; the sends that already go to a monitor
or deliberately bypass one are right. **An earlier note here overstated the blockage**: §2.5's
*removal* witnesses need Phase 5, but §2.4's *redirection* witnesses need only `~destination`, which
runs. Step 5 has plenty to assert.

**Two things that need no work.** Survey C §2.3's whole byte-order program (nineteen clauses over
every route, unredirected) is already byte-identical on both descriptors, and so is a `trace r`
transcript. Step 1 and Step 2 are therefore routing changes, not rendering changes -- and a witness
that exercises only the default route would be green over an unbuilt feature, so every witness has
to redirect first.

**Survey C 3.3 is the rule table Step 1 implements, and the `.nil` asymmetry is measured.**
`sayOutput`: resolve `.OUTPUT`; present **and not `.nil`** -> send `SAY` with the line and drop the
reply; otherwise write to stdout directly. The omitted-name builtins go through `resolveStream`
instead, which has **no `.nil` check** -- empty name to `.OUTPUT`/`.INPUT`, `STDOUT` to `.OUTPUT`,
`STDERR` to **`.ERROR`**, `STDIN` to `.INPUT` -- and send the message named after the builtin with
exactly the arguments given. That asymmetry is why `.local~output = .nil` prints through `SAY` and
then dies 97.1 on `LINEOUT(,)`, and why a *missing* entry crashes the oracle instead.

Measured against this crate: `.local~output = .nil` then `say`/`lineout(,)`/`say` gives the oracle
`H-via-say` on stdout and rc 159 with the 97.1 report, where the crate prints all three lines and
exits 0.

**The routing is not "send if it is a Monitor".** With the entry replaced by a plain `Stream`
(`.local~output = .stderr`, which leaves `.output~class~id` as `Stream` on both sides), the oracle
still sends all three routes to it. Always resolve and send; the object's class does not enter into
it.

**Only three sites write `Interp.out` directly**: `Interp::write_out` (`lib.rs:3068`) and
`say_evaluated`'s two lines (`run.rs:2084-2085`). `write_out`'s other caller is `BEEP`
(`platform.rs:262`), which must stay unrouted.

**Survey C §7's open question is now read, not counted, and it invalidates Step 4's criterion.**
It asked "whether SAY sits inside a timed section of `rexxcps` or any bench axis". Measured across
every axis: `emptyloop` is `do i = 1 to 25000000; nop; end` with a single `SAY` after the loop;
`rexxcps` has twelve `say`s lexically inside `do` blocks and every one is an `if ... then say
'FailedN'` guard -- a real run prints five banner lines, no `Failed` line, `10190112` clauses per
second. `decrender` (2) and `heapshape` (1) are the only other in-loop `say`s and neither is an
output axis. **No axis executes a `SAY` inside a timed section.**

So Step 4's "regression above the 1% noise floor" would read ~0% however expensive a routed `SAY`
became: the criterion is satisfiable and measures nothing, which is the gate-vacuity shape rather
than a threshold. **Ruling: implement Steps 1-3 without the fast path, measure on an axis that can
see it, and let that decide whether the counter is built.** A `sayloop` axis goes in beside the
others, with `rexxcps` kept in the comparison per the standing rule. Cost if wrong: a real `SAY`
regression ships unnoticed on output-heavy programs -- which is precisely what the new axis exists
to catch, and why the ruling adds an instrument rather than dropping the check.

Recorded in the plan's Step 4 with the measurements, since a brief regenerates from the plan.

### Task 11 -- the `sayloop` axis, and what registering it taught

**The 10x gap is `SAY` itself, not buffering -- the first explanation was wrong and was falsified
by running it.** A million-`SAY` loop costs the oracle 8,014,017,855 instructions to `/dev/null`
and 8,048,021,204 to a regular file (+0.4%); this crate 799,724,026 and 799,727,983 (+0.0005%).
Both emit the full 2,000,000 bytes, so destination barely matters on either side and the gap is the
operation. Decomposed against the same loop with `nop`: **304 instructions per `SAY` here against
7,499 on the oracle**, the two bare loops within 4% of each other. That is real headroom -- a routed
`SAY` can cost several hundred instructions more and still leave this crate far ahead.

**Sized by measurement, not by guess.** Per-`SAY` cost is 304 at 100k, 250k and 1M iterations
alike, so the axis uses 100,000: the same signal, a hundredth of the stdout the suite holds in
memory (`PAIRS = 9` plus a warm-up keeps every sampled run's bytes to compare them), and a run in
milliseconds.

**Registering it needed three lists, and a fourth assertion caught a guess.** `AXES`
(`rexx-bench-suite.rs`, pinned against the directory) and `PROGRAMS`/`NOT_BENCHMARKED`
(`rexx-bench/src/lib.rs`, pinned against the same directory) are independent. I first put `sayloop`
in `NOT_BENCHMARKED` with the reason "runs in milliseconds" -- and
`the_exemptions_are_true_of_the_programs_they_name` rejected it: that list recognises **one**
reason, that the program reports its own timing via `TIME(`, which is why `heapshape` is there.
**Ruling: benchmark it.** Changing the test to admit my reason would be fitting the instrument to
the program, which the assertion's own message warns against. `rexx-bench` is 29/29 with the axis in
`PROGRAMS`, and `AXES` matches the directory (checked by diffing the two lists as well as by the
test).

`bench-control/` holds one program and its test is a hardcoded `alloc4c` pair, not a per-axis
requirement; the "absent row" sentence in the suite is about the CPU-affinity header line, not a
missing baseline row. Neither needs anything for a new axis.

**`sayloop` baseline at `f1acca259`, on the registered program, before any Task 11 code.** Release
build, `perf stat -e instructions:u`, three runs interleaved by hand:

```
crate    189,498,434   189,498,571   189,488,577    spread 1.0e4  (~0.005%)
oracle   809,650,007   809,651,830   809,667,325    spread 1.7e4  (~0.002%)
```

Crate is 4.27x cheaper than the oracle on this axis today. Both spreads are two orders of magnitude
under a 1% floor, so unlike `rexxcps` and `emptyloop` this axis can resolve a change to how `SAY`
reaches its destination -- which is the reason it exists.

**Two hazards checked off before writing the routing.** The embedded library contains no `SAY` at
all -- zero occurrences in `CoreClasses.orx` and `StreamClasses.orx` -- so a routed `SAY` cannot
fire during bootstrap before `.local` is built, and the ordering problem that forced Task 10's lazy
mint does not arise here. And every receiver the routing will send to is an ordinary object
(`.output` a `Monitor`, `.stderr` a `Stream`, `.nil` an `Object`), so `send_message` has a valid
receiver in each case rather than a primitive that would refuse.

**Pre-change answers the routing must preserve, measured before the edit.** Byte-identical on both
sides today, so any of these moving after Step 1 is a regression rather than progress:

```
stream('STDOUT','S')   READY        stream('STDOUT','D')   READY:
lines('STDIN')         1            chars('STDIN')         1
stream('')             rc 216, Error 40.27      stream('','S')   rc 216, Error 40.27
```

The 40.27 pair confirms the reachability claim at `builtin/stream.rs:186` -- `STREAM` never reaches
a default stream, so changing `standard_stream_name`'s mapping cannot alter it through that path.
That claim was an argument about unreachability in a comment, which is the shape this project keeps
getting wrong, so it was run rather than trusted.

**Blast radius of the mapping change, derived rather than estimated.** `standard_stream_name` is
called only from `resolve_stream`, and `resolve_stream` has three callers: `send_to_stream`
(`builtin/stream.rs:37`, all seven builtins), `LINES` (`:131`) and `STREAM` (`:186`). Across the
whole corpus only `std_stream_writes.rex` names a standard stream as a string, and no witness pins
`LINES`/`CHARS` on one -- so those two callers are currently unwitnessed for standard names, which
is a reason to add a witness in Step 5 rather than a reason to assume the change is safe.

### Task 11 -- a latent uninit defect the routing exposed, and eight wrong mechanisms

**The finalizer loss is not caused by routing `SAY`.** Measured with routing on: a program whose
only output before `call gc 'force'` is `.stdout~lineout('one')` -- which never touches the routed
`SAY` path -- loses the retained finalizer exactly as one using `say 'one'` does, while a program
with **no output at all** before the `gc` keeps it. So the trigger is *any allocation before the
forced collection*, not `SAY` and not the routing. What the routing changed is that `SAY` now
allocates where it did not, which makes the latent defect reachable from programs that were green.

The shape, from the code: a collection resurrects a flagged-but-unreachable object and pushes it to
`pending_uninit` only when `!ready_for_uninit`, setting that flag (`heap.rs:171-192`);
`lib.rs:4671` moves those into `uninit_ready`; `GC('force')` is the only caller of
`run_ready_uninits`, which takes that list and calls `clear_uninit_all`, clearing `has_uninit`. An
object readied by an *incidental* collection and then cleared that way is no longer in the registry
`take_uninit_flagged` walks at termination.

**Eight mechanisms proposed today, eight falsified.** In order: `/proc` reads racing under rayon;
a stale test binary; the 60s deadline; the per-row working directory; the wrong row being selected;
nondeterminism; the output buffer being taken before the sweep; the trailing `SAY` consuming the
flag. Each was plausible, each was killed by one command. Two things I called divergences were not:
`GC('force')` running finalizers inline is correct and matches the oracle (`a|b|uninit|c` on both),
and the driven-shape ordering reproduces with the hunk reverted. **The pattern is proposing a
mechanism before running the control that separates it from its neighbours** -- the isolation test
that finally settled this (`say` versus `.stdout~lineout` versus no output) took one command and
would have been just as cheap eight rounds earlier.

**Proved at HEAD: the uninit loss is pre-existing and reachable with no Task 11 code.** A detached
worktree at `f1acca259`, built in its own `CARGO_TARGET_DIR`, runs two programs that differ only in
whether anything is written before `call gc 'force'`:

```
stream write, then gc    stdout=[one]        stderr=[]        <- finalizer lost
no output, then gc       stdout=[]           stderr=[FIN]     <- finalizer runs
```

The oracle runs the finalizer in **both** shapes (and in the `say` variant), so this is a crate
defect rather than a licensed divergence. Neither program contains a `SAY`, so no routing is
involved on either side.

**Why Task 11 appeared to cause it.** Six of the eight failing corpus programs never call `gc` at
all -- they rely on the termination sweep -- and every one of the eight writes with `SAY` before its
collection. Routing made `SAY` allocate where it previously appended to a buffer, so a latent bug
that no committed program could reach became reachable by all eight at once. `479 of 488` is real;
its cause predates `f1acca259`.

**Ruling: Task 11 does not absorb this.** The uninit defect is fixed as its own change with its own
witness, and the routing lands afterwards on a tree whose corpus can judge it. Folding a
pre-existing GC bug into a routing commit would hide both, and the routing is measured correct --
all three redirect shapes byte-identical, including the `.nil` case at rc 159. Cost if wrong: one
extra commit and a delay to Task 11's remaining steps.

### The finalizer defect is Task 10's, and the mechanism is measured end to end

**It is not the routing, and it is not pre-existing.** Merely evaluating `.stdout~string` -- no
write anywhere -- loses a later finalizer exactly as a write does; a program that never touches
`.local` keeps it. So the trigger is **minting the standard-stream bundle**, which `5dcc4513a`
introduced. An earlier note here said the defect predates Task 11 and is reachable at HEAD: both
true, but I read that as predating my work entirely. It does not -- it arrived with Task 10, and
the HEAD proof used a stream write, which mints the bundle too.

**The mechanism, from instrumentation rather than inference:**

1. Touching `.local` mints three `Stream` objects. Each answers `UNINIT` (`native.rs:265`,
   `stream_uninit`), so each is flagged for finalization at construction (`dispatch.rs:4112`).
2. At termination the sweep finds **four** flagged objects where an untouched program finds one,
   and delivers to the bundle Streams as well as the program's own object.
3. `stream_uninit` sets `native = None`. The sweep's `SWEEPS` loop runs a second pass, re-delivers
   to the same Stream, and `require` finds no state and raises **48.1** (`Stream not initialized`).
4. `run_one_uninit` discards `Err(Raised)` by design (`dispatch.rs:2542`), and that discard
   abandons the rest of the bounded sweep -- so the program's own finalizer, later in the batch,
   never runs at all.

Instrumented outcomes, passing shape against failing: `Ok Ok Ok Ok` -> `FIN` printed, against
`Ok Ok Ok Raised 48.1` -> nothing printed. Every destination behaves the same (stdout, stderr and
`SAY` all lost), where the oracle prints all three.

**Eleven mechanisms proposed today, ten falsified.** The one that survived came from instrumenting
the sweep -- two `eprintln!`s -- rather than from reasoning about it. Three separate readings were
wrong in ways that each looked settled at the time: "the finalizer never runs" (it runs, and its
output is discarded), "an allocation before construction is the trigger" (over-fitted to one data
point; the class-side case splits differently), and "pre-existing, not mine" (Task 10's).

#### Correction: it is not a second delivery, and the controls are predicted before running

Probing `s~uninit` twice directly raises **91.999 `Message "UNINIT" did not return a result`** on
both engines -- not 48.1. `stream_uninit` never calls `require`; it returns `Ok(None)`
unconditionally. So the "second sweep re-delivers to a drained Stream and `require` raises"
mechanism in the entry above is **falsified**. That makes twelve mechanisms proposed and eleven
falsified, and this one I had already written into the ledger as measured.

The reading that fits every observation instead: the sweep drains the three bundle Streams first
(flagged at mint, before the program's own object), and then the user finalizer's own
`.stderr~lineout('FIN')` raises 48.1 because its destination is already dead. The finalizer runs;
its output has nowhere to go. `run_one_uninit`'s discard is then a red herring for this defect --
the raiser is last in the batch, so no finalizer is abandoned after it. The oracle keeps all three
because its bundle streams are reachable from `.local` and so are not dead objects; we flag
reachable ones.

**Predictions, recorded before the controls ran:**

- **B** -- bundle minted, finalizer writes to a `.Stream~new('fin.txt')` it makes itself:
  `fin.txt` holds `FIN`. Confirms the finalizer runs and only the destination died. An empty or
  absent `fin.txt` puts sweep-abort back on the table.
- **C** -- `.K~new` before the bundle is touched: `FIN` printed, because the K object is flagged
  first and its UNINIT precedes the streams'.
- **D** -- the known failing shape, as positive control: `FIN` lost.

#### Results: all three predictions confirmed, on both engines

```
B  bundle minted, finalizer writes its own Stream   crate fin.txt=[FIN]   oracle fin.txt=[FIN]
C  .K~new BEFORE touching the bundle                crate err=[FIN]       oracle err=[FIN]
D  the failing shape                                crate err=[]          oracle err=[FIN]
```

B is decisive: the finalizer runs to completion, and inside the sweep it allocates a *new* Stream
and writes through it successfully. Nothing is abandoned, so `run_one_uninit`'s discard is not
implicated in this defect -- it only hides the raise. C is decisive on ordering: construct the
object before the bundle is minted and its output survives.

**The defect, stated as the observable:** the bundle's standard Streams are finalized before the
program's own objects, and a later finalizer writing to `.stderr`, `.stdout` or via `SAY` raises
48.1 into a destination whose `native` the sweep already dropped.

**Why Task 11 exposed it.** At `f1acca259` the bundle is minted only when a program explicitly
touches `.stdout`; Task 11's `local_route(b"OUTPUT")` in `say_evaluated` mints it on every `SAY`,
so the eight `uninit_*` programs -- which say from their finalizers -- go from latent to failing.
The corpus stood at 488/488 there because nothing in it touched the bundle.

**Ruling: a bootstrap standard stream does not participate in finalization.** In the oracle these
are reachable from `.local` for the interpreter's lifetime, so they are never dead objects and are
never finalized out from under a running program. This crate flags at construction
(`new_instance`, `dispatch.rs`) with no reachability notion, so the fix is to not flag the three
`standard_stream` objects: their `native` wraps descriptors 0/1/2, which this crate does not own
and must never close. Cost if wrong: a standard stream's buffer is not flushed by a finalizer --
which is already true, since termination flushes through `Outcome`, not through `UNINIT`.

#### The fix, and why it cannot make things worse

One call in `standard_stream` (`dispatch/stream.rs`), after `new_instance` flags the object:
`interp.heap.clear_uninit_all(&[object])`. `check_uninit` (`dispatch.rs`) already uses that call
as a single-object clear when a `SETMETHOD` removes an `UNINIT`, so this is an existing idiom, not
a new heap API. `new_instance`'s flagging rule is untouched, so every program-created Stream still
finalizes and still flushes and closes.

**The one way this fix could have been worse than the defect, checked before writing it.**
`set_uninit`'s own doc says the flag makes the collector "resurrect and report" the object rather
than sweeping it -- so clearing it could in principle make the three bundle Streams sweepable, and
a stream collected *mid-run* would raise 48.1 during the program instead of after it. It cannot:
`Interp::object_roots` classifies `environment` with the comment "`.environment` and `.local` are
globals", registered through `RootSet::add_global`. The bundle is reachable from a global root for
the life of the run, and reachability, not the uninit flag, is what keeps it alive.

**Witness:** `corpus/lang/uninit_after_std_streams.rex`, named in `corpus/phase-7.txt`, with its
`sourceline_oracle` companion. One object, one finalizer, one writer per descriptor -- two
finalizers writing to the same descriptor would make the byte order depend on sweep order, which
is licensed to diverge and so is not something the corpus can judge. It keeps the explicit
`.stdout~string`, so it is red without the fix even on a tree without Task 11's `SAY` routing.
The adjacent successes are the eight existing `uninit_*` programs in `phase-5b.txt`, which must
stay green.

Oracle: stdout `built` / `say reaches` / `stdout reaches`, stderr `stderr reaches`, rc 0.

#### Landed: `d5f49c184` "Keep the standard streams out of the finalization sweep"

Measured on the committed tree (fix plus Task 11's partial routing, which was in the working tree
for the run): `REXX_CORPUS_GATE=1 cargo test -j 4 -p rexx-exec --test corpus` reports
**489 of 489** matching, 27 passed, 0 failed, 1 ignored. That covers the new witness, the eight
`uninit_*` programs in `phase-5b.txt` that are its adjacent successes, and Task 11's `SAY`
routing. `cargo fmt --all --check` and `cargo test -p rexx-parse --test sourceline_oracle` are
both green.

The background run's own exit code was 0 for the wrapper block, which ends in `echo finished` and
would be 0 whatever the test did -- the figure above is the `corpus status=` line inside the
output file, not the task notification.

Staged by exact path: `dispatch/stream.rs`, `corpus/phase-7.txt`, the witness, and its
`sourceline_oracle` companion. Task 11's `dispatch.rs`, `environment.rs`, `run.rs`, the bench
files and the plan edit stayed out of the commit and remain in the working tree.

**Still owed by Task 11**, and now unblocked: Steps 2-6, plus re-pinning the three inflated
counter tests. Those counts are Task 11's doing rather than this fix's -- routing `SAY` through
`.OUTPUT` mints the bundle and allocates -- so they are re-pinned inside Task 11 with that reason,
not as a change of their own.

### Task 11 Step 2: measured before writing anything

**A trace line and an error-report line both arrive as one `LINEOUT` of a `TraceObject`.** Measured
against the oracle with a sink installed as `.local`'s `TRACEOUTPUT` entry:

- `trace i` over `aa = 1 + 1` sends **exactly 7** `LINEOUT`s -- the `*-*` clause line, two `>L>`,
  `>O>`, `>>>`, `>=>`, and the following `trace off` clause. Each argument is a `TraceObject`
  carrying one already-formatted line, indent included, with no trailing newline.
  **The sink's own body is not traced**, so the re-entry I expected to have to guard against does
  not happen on the oracle. That was measured, not assumed.
- An uncaught `1/0` sends three: the `*-*` clause echo, `Error 42 running <file> line 3: ...`, and
  `Error 42.3: ...`. Nothing else reaches stderr, so the report does not additionally go there.
- A `TraceObject` is a `StringTable` of eight entries -- `TIMESTAMP` (a `DateTime`), `THREAD` 1,
  `INVOCATION` 0, `NUMBER` (increments per line), `INTERPRETER` 1, `TRACELINE` (the text),
  `OPTION` `N`, `STACKFRAME` `.nil` -- and `makeString` answers `TRACELINE`. `~string` answers
  `a TraceObject`, so the text comes through `MAKESTRING`, not `STRING`.

**Two things this crate already gets right, so Step 2 does not touch them.** A bare
`.TraceObject~new` is byte-identical on both engines (3 items: `TIMESTAMP`, `NUMBER`, `OPTION`;
`makeString` `?`), so the interpreter is what fills the other five entries. And monitor forwarding
already matches: `.traceoutput~lineout` and `.error~lineout` reach stderr, `.output~lineout`
reaches stdout, identically on both engines. Step 2 is therefore only about routing lines *into*
`.TRACEOUTPUT`.

**`oracle-crashes.txt` entry 11c is a rule this step implements, not a hazard to dodge.**
`.traceoutput~destination(.nil)` plus a traced clause SIGSEGVs the oracle; for this crate it is a
licensed divergence -- the line stays on stdout and the program exits normally.

#### The re-entrancy guard: my prediction was falsified, and the reason for the guard changed

**Prediction, recorded before the run:** with `::options trace i` making method bodies traced too,
the oracle would still complete with a bounded line count, because it suppresses tracing while
delivering to the trace route. Outcome: **neither of the two outcomes I named.** The oracle
**SIGSEGVs, rc 139** -- stdout empty, stderr 762 bytes ending mid-trace at `>A>   "a SINK"`, at the
`~destination(s)` clause. Filed as a new `oracle-crashes.txt` entry.

**The innocent explanation was the right one, and a separate probe settles it.** Under `trace i`,
a method send traces as `>M>   "M" => "4"` and the method's own `qq = 2 + 2` is never traced --
measured, and this crate is already byte-identical there. So the sink's body went untraced in the
earlier probe because **trace does not propagate into a method body**, not because the interpreter
suppresses during delivery. There is no suppression behaviour to match.

**Ruling: keep a re-entrancy guard, but for a different and checkable reason.** It is not there to
match the oracle -- the oracle crashes in the only shape that reaches it. It is there because
`::options trace i` plus a routed trace line would otherwise re-enter and hang this crate, and a
licensed divergence still has to *terminate*. That is a property a witness can check; "matches the
oracle's suppression" would have been a claim about behaviour that does not exist. Without this
correction the guard would have shipped with a false justification riding a correct decision.

#### Step 2 written: where the hook goes, and one design correction caught before it shipped

**The hook is at the `Interp`-method level, not inside the `push_*` helpers.** Those are free
functions whose `out` is sometimes *not* `self.trace` -- `error.rs` builds a report into a local
buffer and `dispatch/context.rs` does the same -- so hooking them would have routed text that is
not a trace line at all.

**Correction made before writing it:** my first design scanned back from the end of `self.trace`
to the previous newline to find the line just pushed. That is wrong, because `self.trace` also
carries `.STDERR` writes and a `CHAROUT` leaves an unterminated fragment, which the scan would
sweep into the trace line. Each call site now captures `self.trace.len()` before the push and
passes it in.

The routing cannot cycle: a routed line reaches the monitor, then `.STDERR`, then returns through
`write_err` into the same buffer -- bytes and order preserved -- and `write_err` never routes.
`routing_trace` exists only for entry 14's shape, where the oracle SIGSEGVs and this crate must
merely terminate.

`TraceObject` needs no new class: it is already an orx class here and a bare `.TraceObject~new` is
byte-identical to the oracle's. **Every entry is a `String`** -- measured, `THREAD`, `INVOCATION`,
`NUMBER`, `INTERPRETER`, `OPTION` and `TRACELINE` all answer `String`, so there is no
Integer-versus-String divergence to reproduce.

Only the two `raised.report(&site)` sites route. `DEADLINE_REPORT` and the `rexx-exec: <loud>`
lines are this crate's own diagnostics with no oracle counterpart, and routing them would invent
trace traffic the oracle never produces. The report routes **one `LINEOUT` per line** -- measured,
three for an uncaught `1/0` -- so it has its own loop rather than reusing `route_trace_line`,
which takes everything from its offset to the end and would have sent the whole report as one.

**Witness answer captured from the oracle** (`trace_redirection.rex`, using `~destination` because
`Directory~setentry` refuses as Phase 5): rc 0, `delivered 7`, then the seven lines with their
`TraceObject` class and text, stderr empty. Not yet added to `corpus/lang/` -- it goes in once the
crate is measured against it, not before.

#### Step 2 measured against the oracle: the report half is right, the trace half is half right

**The error report matches byte for byte.** A program whose `.traceoutput~destination` is a Rexx
sink and which then divides by zero prints `REPORT TraceObject` three times on both engines, rc
214, stdout and stderr identical. So `write_trace_report`, the per-line loop, and the decision to
route only the two `raised.report(&site)` sites are all correct as written.

**The trace half delivers correctly and carries nothing.** The crate sends exactly 7
`TraceObject`s in the right order with the right class -- the count, the ordering and the class
all match -- but `~request('STRING')` answers `?` where the oracle answers the line, and `?` is
what a bare `.TraceObject~new` answers on both engines. Measured directly: the delivered object
holds `items= 3 idx=[TIMESTAMP,NUMBER,OPTION]` against the oracle's eight.

**Cause: a silent no-op write.** `if let Some(target) = self.heap.get_mut(object) && let
Body::Native(native) = &mut target.body { native.set_entry(...) }` compiles -- `set_entry` resolves
on the type -- and simply does not match at run time, so every entry write vanished with no
diagnostic. This is the shape my own note warns about: the send returned an object, the object
arrived, and the write it was supposed to carry was absent. A conditional write whose failure
looks exactly like success.

**Direction: send the object the message Rexx uses**, rather than reaching into its body. The orx
`INIT` fills `OPTION`, `NUMBER` and `TIMESTAMP` through the ordinary collection path, so that path
demonstrably reaches the real storage; my Rust-side poke was aimed at a body these instances do
not have.

#### Step 2 complete, and Step 3's message table measured

**The silent no-op is fixed by sending the message instead of poking the body.** A `StringTable`
keeps its entries in a bucket table built from heap slots (`dispatch/hash.rs`), not in the
`Body::Native` map that backs `.local` -- so `native.set_entry` was aimed at a representation these
instances never had, which is why it compiled and did nothing. The put now goes through
`("StringTable", "PUT", Arity::Fixed(2), native_hash_put)`, and the argument order is the one
`dispatch.rs` states rather than one I guessed: `t[i] = v` sends `t~"[]="(v, i)`, value first.

**Measured after the fix, all byte-identical to the oracle:**

- the delivered object holds `items= 8` with
  `idx=[TIMESTAMP,THREAD,INVOCATION,NUMBER,INTERPRETER,TRACELINE,OPTION,STACKFRAME]` -- **index
  order included**. Inserting in the oracle's own display order was a guess that paid: bucket
  chains resolve collisions in insertion order, and the orders now agree, so `~allindexes` is
  safe to put in a witness.
- `trace_redirection.rex`: stdout and stderr identical, rc 0, 7 deliveries.
- the error-report probe: identical on both descriptors, rc 214, 3 deliveries.

**Step 3's message table, measured in one run** by giving `.input` a destination whose `UNKNOWN`
reports every message. It confirms the plan's text and settles the one item the plan states
without a citation:

```
linein()  -> MSG LINEIN n= 0
lines()   -> MSG LINES  n= 1 [NORMAL]
charin()  -> MSG CHARIN n= 0
pull d    -> MSG LINEIN n= 0
```

`PULL` sends **`LINEIN`**, not a `PULL` message of its own.

#### A stream write does not convert its argument the way the oracle does

Task 11's routing exposed this: every trace line arrived on stderr as the literal text
`a TraceObject`, so `trace_oracle` went 18 red, `trace_indent` 1, `ir_recorded` 3. The routing
itself is right -- the count, order, class and index set all match -- and what is wrong is how a
stream renders a non-string argument.

**Measured, the oracle's rule for `LINEOUT`:**

```
.stdout~lineout(.M~new)          MADE          -- M defines ::method makestring
.stdout~lineout(.array~of(p,q))  p / q         -- a built-in MAKESTRING participates
.stdout~lineout(.K~new)          Error 88.909  -- "Argument 1 must have a string value", rc 168
```

So a write requires a string value through `REQUEST('STRING')`, and an object with no
`makeString` is **not** rendered as `a K` -- it raises. This crate prints `a M`, `p`/`q`, `a K`
and exits 0, so it is wrong on the conversion *and* on the refusal. The conversion site is
`dispatch/stream.rs`'s `interp.to_text(value)`.

**My own verification missed it, and the shape is worth naming.** Every probe I ran for Step 2
used a custom Rexx sink that called `v~request('STRING')` itself, and the report probe printed
only the class -- so not one witness exercised the default route through a real stream, which is
the path every existing trace transcript takes. Three green probes, whole suite red.

**Unexplained and not to be theorised past:** on this crate `say .M~new` answers `MADE` while
`.stdout~lineout(.M~new)` answers `a M`, though both are supposed to go through `to_text`. Until
that is located, any edit risks being aimed at the wrong site.

**Ruling: this lands as its own change with its own witness**, like the uninit defect, and the
routing lands on top of it. Folding an oracle-conformance fix for stream writes into a routing
commit would hide both.

#### The stream-write conversion defect, fully measured

`CHAROUT` behaves exactly as `LINEOUT` does, so both write sites take one fix:

```
                                  oracle              crate
.stdout~charout(.M~new)           MADE                a M
.stdout~charout(.array~of(p,q))   p\nq                p\nq      (already right)
.stdout~charout(.K~new)           88.909, rc 168      a K, rc 0
```

The array case already matches because `to_text`'s `Redirect::Array` arm joins with a newline;
only the instance cases are wrong, and they are wrong twice over -- the conversion and the
refusal.

**Scope: two lines.** Of the nine `to_text` calls in `dispatch/stream.rs`, only `charout`
(`:1100`) and `lineout` (`:1156`) convert a *write argument*. The others take a stream name
(`:102`), option strings (`:528`, `:1470`, `:1725`), a `QUERY` letter (`:1293`) and two numeric
positions (`:1343`, `:1352`), all of which genuinely want `stringValue()` rather than a
`makeString` conversion. Changing those would be the wide, wrong fix.

**The helper already exists and covers both halves**: `blamed_string_conversion` answers the
converted string or `None`, and `required_string_argument` turns the `None` into the oracle's
`Argument 1 must have a string value`. Private to `dispatch`, reachable from `dispatch::stream`
through `super::`.

**Why `to_text` itself is not the fix.** `~string` answers `a M` on *both* engines, so `STRING`
is not the conversion; `to_text` is right for what it is, and its own doc already separates
`stringValue()` from the string conversion. `SAY` on this crate already reaches `makeString`
correctly -- only the stream write does not.

#### A probe masked by the defect it was probing

The blame probe -- does an existing `Stream` refusal already carry
`*-* Compiled method "LINEOUT" with scope "Stream".` -- could not answer, because this crate's
stderr for it was `a TraceObject` four times over: the error report now routes through
`.TRACEOUTPUT` and is rendered by the very conversion under repair. Re-running it after the fix
is what answers it.

It did settle two things by accident. rc 163 matches the oracle, and the report split into
**four** lines against the oracle's four-line transcript (blame line, clause echo, and the two
`Error` lines) -- independent evidence that `write_trace_report`'s per-line loop is right, from a
program written to ask something else.

It also showed the oracle emits that blame line for an ordinary 93.907 from the position check,
so stream refusals carry it generally and this crate must already produce it by some route. The
explicit `blame_native_method` in `write_text` is therefore safe either way: it is a no-op when a
failure site is already set, so whichever side records the line, the same one line appears.

#### The conversion fix measured, and three residuals separated

`write_text` -- `required_string_argument` then `to_text` -- at the two write sites only.
Measured after it:

- `t11saylineout`: byte-identical on both descriptors.
- `t11conv` and `t11charout`: **stdout identical** (`MADE`, and the array joined by newline) and
  rc 168 matching. The conversion and the refusal are both right.

Three residuals, and they are not one thing:

1. **The explicit blame was redundant, measured.** The crate carried
   `*-* Compiled method "LINEOUT" with scope "Stream".` **twice** where the oracle has it once, so
   the dispatch layer already records that line. Removed from `write_text`. This is the answer the
   masked probe could not give, and it came from the diff rather than from reading the dispatch
   path.
2. `Error 88 running REXX:` on the oracle against `running <path> line 4:` here -- the oracle
   attributes the refusal to the REXX package with **no line at all**. Re-measured after removing
   the duplicate blame, since a doubled blame seals an extra level and could be the whole cause.
3. **A pre-existing numbering defect, not this fix's.** `.stdout~lineout('keep', 0)` answers
   `Method argument 2 must be a positive whole number` where the oracle says `argument 1`. That is
   `optional_position(..., 2)`, untouched by this change and invisible until the transcript stopped
   being masked. Recorded here rather than fixed inside a conversion commit.

#### After the conversion fix: 22 red became 4

```
trace_indent   23 passed, 0 failed   (was 1 failed)   -- fully green
trace_oracle   39 passed, 2 failed   (was 18 failed)
ir_recorded     6 passed, 2 failed   (was 3 failed)
```

The duplicate blame line is gone, so removing the explicit `blame_native_method` was right. But
residual 2 **persists**, which falsifies the guess that the doubled blame caused it: the oracle
reports `Error 88 running REXX:` with no line where this crate reports
`running <path> line 4:`. It is a report-attribution difference for a refusal raised inside a
native method's own activation, and because it is the same refusal's transcript, the conversion
fix's witness cannot be byte-identical until it is settled -- so it is in scope, unlike the
argument-numbering defect.

**End of input already matches, byte for byte**, on empty stdin with `signal on notready`:
`lines=[1]` -- `LINES` answers 1 at EOF, not 0 -- then `LINEIN` raises `NOTREADY`, handler at
line 4, rc 9, both engines. So Step 3's observable behaviour is largely right already, and the
routing there will be visible mainly through a custom `.input` destination rather than through a
change in what a program sees.

#### The attribution difference is narrow, and a worse defect surfaced behind it

**The general native path is already right.** `'abc'~pos(.K~new)` -- a native
`required_string_argument` site with no stream anywhere -- is **byte-identical** on both engines,
`Error 88 running <path> line 2:` included. And `t11blame`'s Stream 93.907 reports the program
path on the oracle too. So `running REXX` is not "Stream methods report REXX": two measurements on
the same class point opposite ways, and the form has to be mapped before anything is changed.

**The remaining test failures are not the conversion.** Both `trace_oracle` residuals are *stdout
empty* where output was expected, and `ir_recorded` reports rc 120 with
`rexx-exec: a compiled call op does not name a call of its own body` on a program that does
`call trace 'i'` ... `call trace 'o'`. That is a loud refusal, not a transcript mismatch.

Reading, to be confirmed by the repro rather than asserted: sending a message from inside the
trace emitter re-enters the interpreter at a point the IR engine does not expect, and perturbs
call state mid-clause. `routing_trace` prevents *recursion* and does nothing about perturbation,
so the guard as designed does not cover this. If that holds, always-routing at every emitter is
not implementable as written and the routing needs a different insertion point.

#### The `running REXX` form, mapped

```
.stdout~lineout(.K~new)      Error 88 running REXX:            no line
.stdout~charout(.K~new)      Error 88 running REXX:            no line
.Stream~new(.K~new)          Error 93 running REXX line 164:   93.938, a real package line
'abc'~pos(.K~new)            running <path> line 1:
.stdout~lineout('keep', 0)   running <path> line 1:            93.907
```

Coherent reading: `Stream`'s string-value refusals arise inside the **REXX package** -- `.Stream~new`
names package line 164 -- while `pos` and the position check are native code entered straight from
the program's clause, which is why those two report the program. Matching it means attributing the
two write conversions to a `REXX` site with no line, which is report machinery, not conversion.
Not folded into the conversion fix.

**The IR refusal outranks it.** `say 'one'; call trace 'i'; say 'two'; call trace 'o'; say 'three'`
is rc 120 printing only `one` and `two`, so it is not the `if ... then call` shape from the
recorded case. What is **not** established is whether the routing causes it: that probe ran on a
binary that already carries the routing, so it cannot separate a regression of mine from something
pre-existing. The recorded expectation says rc 0, which is evidence about a recording rather than
about a run.

#### The IR corruption is the routing's, isolated by A/B

With `route_trace_line` patched to an immediate return, rebuilt, and the five shapes re-run:

```
plaincall     ROUTING-OFF rc 0, no loud, stdout matches      (routing on: rc 0)
callbuiltin   ROUTING-OFF rc 0, no loud, stdout matches      (routing on: rc 120 LOUD)
calltraceon   ROUTING-OFF rc 0, no loud, stdout matches      (routing on: rc 0)
calltraceoff  ROUTING-OFF rc 0, no loud, stdout matches      (routing on: rc 120 LOUD)
tracestmt     ROUTING-OFF rc 0, no loud, stdout matches      (routing on: rc 0)
```

So the routing causes it, established by running rather than by trusting the recorded expectation.
It is a regression of mine, not something pre-existing. The failing shapes are the two that emit
`>A>`/`>F>` from inside an `Op::Call` region -- a builtin call under trace -- and the refusal is
`Loud::call_op_off_its_node`, "a compiled op that runs or traces a call does not describe the call
it was emitted for". A re-entrant send issued from a trace emitter inside a call op leaves that op
unable to resolve its own node.

The source was restored by the A/B's own trap, which means the binary left in `target/debug` was
the **no-op** build -- rebuilt before measuring anything further, since a stale binary outliving
its revert has cost this project time before.

**Ruling: route only when the chain has actually been redirected.** When `.TRACEOUTPUT` is still
the bootstrap monitor and its destination is still the bootstrap `.STDERR`, routing and writing
directly produce *identical bytes* -- the monitor forwards into the same buffer -- so the direct
path is exactly equivalent rather than an approximation, and every existing transcript stays on
the path it has always taken. The plan's Step 4 already anticipates this bootstrap-identity check
for `SAY`; this applies the same shape to trace. Cost if wrong: a program that redirects trace
inside a builtin call under `trace i` can still reach the IR fragility, which is a narrow, newly
supported case and is recorded rather than hidden.

#### The real cause: the routed send skipped the nested-send protocol

```rust
take_value_buffer  -> let buffer = std::mem::take(&mut self.value_buffer); let mark = buffer.len();
give_value_buffer  -> buffer.truncate(mark); self.value_buffer = buffer;
```

Every other nested send in the tree carries the shared argument stack out and back this way --
`FORWARD` at `run.rs:2219`, the dispatch site at `dispatch.rs:2825` -- leaving `self.value_buffer`
empty for the nested run. **My routed send did a bare `send_message`.** `clause.rs` clears that
buffer at a clause boundary, so running the monitor's Rexx body on the caller's live stack wipes
the arguments an enclosing call has already pushed, and `run_call_args`' `len - argc` underflows.
That is why only calls *with arguments* broke: `call sub` pushes nothing, `length('abcd')` pushes
one, and `>A>` is emitted between the push and the consume.

**This retires the bootstrap-identity fast path.** The fix is the existing convention rather than
a way around the problem, it also fixes the redirected case, and it needs neither the monitor's
destination variable nor an identity check -- fortunate, since `Monitor` has no native rows and no
`DESTINATION` anywhere in the tree, so there was nothing to read.

The wrap goes around the **whole** delivery in `route_trace_line`, not just the `LINEOUT`: `NEW`
runs the `TraceObject`'s `INIT` and each `PUT` runs Rexx, so all of them cross clause boundaries.

Four mechanisms were proposed for this failure before the right one: stale node reference, general
call-state perturbation, IR fragility needing a re-resolve, and a bootstrap fast path to avoid it.
The first three were killed by reading the two functions that actually own the invariant, the
fourth by the same reading making it unnecessary.

#### The wrap works: 22 red, then 4, now 2

```
trace_oracle   40 passed, 1 failed
trace_indent   23 passed, 0 failed
ir_recorded     7 passed, 1 failed
```

All five call shapes are rc 0 with matching stdout and no loud message, so the nested-send
protocol was the whole of that defect. `t11saylineout`, `t11report` and `trace_redirection` are
byte-identical. `t11conv` and `t11charout` still differ only in the deferred attribution residual
(`Error 88 running REXX:` against `running <path> line N`).

**My witness could not have caught the bug it was written alongside.** The landed
`trace_redirection.rex` traces `aa = 1 + 1` and contains **no builtin call with arguments**, so it
emits no `>A>`/`>F>` -- exactly the argument-window lines that broke the engine. The extended
version adds `zz = length('abcd')` and the oracle delivers 13 lines including `>A>   "abcd"` and
`>F>   LENGTH => "4"`.

That is the second time in this task that a witness missed the shape that actually failed: first
every Step 2 probe used a custom sink and so bypassed the default stream conversion, now the
witness omitted the call shape. Both times the gap was the same one -- the witness exercised the
path I was thinking about rather than the path the code takes by default. Worth carrying as a
pattern: after writing a witness, ask which ordinary shape it still never executes.

#### The nested-send protocol fixed one family, not the class

`use_arg_alias` is not a transcript mismatch: the crate exits **239** with
`Error 17.1: PROCEDURE is valid only when it is the first instruction executed after an internal
CALL`. The program is `call sub >orig` ... `sub:` / `procedure`, and the routed delivery runs Rexx
clauses **between** the CALL and the callee's first instruction, so `PROCEDURE` is no longer first.
That is a second per-clause state family -- `procedure_permitted` / `region_procedure_permitted` --
perturbed exactly as `value_buffer` was. The remaining `ir_recorded` case is a `SELECT` that should
raise 7.3 and now prints `after`, so control flow changed too: probably a third.

**This retires my own conclusion from a few steps ago.** I wrote that the nested-send protocol was
"the whole of that defect". It was the whole of *one family*. Two, probably three, have now been
found one at a time, each by a different failing test, and each after I had declared the mechanism
understood. Continuing to save and restore fields until the suite goes green is the whack-a-mole
this project's notes warn about, and a green suite would not show that the perturbation was gone --
only that no current test sees it.

**Ruling revised: route only when the chain has actually been redirected**, which is what the
plan's Step 4 anticipated for `SAY`. Default runs then never send from a trace emitter, so every
existing transcript keeps the exact path it has always had, and the perturbation question is
confined to the newly supported redirected case. Cost if wrong: a program that redirects trace can
still meet these state families, which is recorded rather than hidden.

The blocker is reading a monitor's destination without a send. The REXX package is parsed from
embedded `.orx` sources at bootstrap -- `StreamClasses.orx` and `PlatformObjects.orx` are named in
the tree -- so `Monitor` has real Rexx source somewhere, and its destination variable has a name
this crate can read through `pool_variable`.

#### The third family is a control-flow divergence, measured directly

```
trace r
select case 2
  when 2 then
  when 3 then nop
end
say 'after'
```

```
oracle  rc 249, stdout empty, Error 7.3 "All WHEN expressions of SELECT are false"
crate   rc 0,   stdout "after"
```

Not a transcript difference: the routed delivery inside the `WHEN` search changes which branch the
program takes. A program that should fail runs to completion. That is the most serious of the
three and it settles the approach -- a default run must not send from a trace emitter at all.

Under "route only when redirected" every one of these failures takes the direct path and goes
green by construction, and the redirected case demonstrably works: the extended witness, which
does emit `>A>` and `>F>` from inside an argument window, is byte-identical to the oracle.

**Identity of the `.TRACEOUTPUT` entry is not a sufficient discriminator.** The witness redirects
with `~destination(s)`, which leaves the *same* monitor object in `.local` -- so the check has to
read the monitor's current destination, and that needs `Monitor`'s Rexx source. `rexx-lib` holds
only `build.rs`, `Cargo.toml` and `lib.rs`, with no `.orx` text and no `Monitor` in it, so the
sources are produced at build time and that is where to look next.

#### `Monitor`, read from the interpreter's own source

The `.orx` sources are the C++ tree's, embedded at build time with pinned sha256s
(`rexx-lib/build.rs`): `CoreClasses.orx`, `StreamClasses.orx`, `PlatformObjects.orx`. My earlier
greps found nothing because they only ever searched the Rust worktree. `Monitor` is
`CoreClasses.orx:1391`:

```rexx
::METHOD init         expose destination; destination = .queue~new; destination~push(dest)
::METHOD destination  expose destination; push with an arg, pull without; return destination~peek
::METHOD current      return destination~peek
::METHOD unknown      forward to (destination~peek) message (msgname) arguments (arglist)
```

So the object variable is `destination`, holding a **Queue** whose head is the live destination,
and `LINEOUT` arrives through `UNKNOWN`/`FORWARD` rather than a method of its own.

**This makes the plan's Step 4 wording exact rather than loose.** "The bootstrap monitor by
identity, its destination queue's head against the bootstrap `.STDOUT`" describes this structure
precisely, so the design arrived at here by measurement is the one the plan already intended.

For trace the untouched chain is two monitors deep, because the mint stacks `TRACEOUTPUT` over
`ERROR` over `STDERR`. "Not redirected" therefore means: the `.local` entry is the minted trace
monitor, its queue head is the minted `.ERROR` monitor, and that monitor's queue head is the
bootstrap `.STDERR` stream. The three ObjRefs are recorded at mint time as new `Interp` fields, so
the exhaustive destructure forces them to be classified.

#### The discriminator works, verified from both sides

```
SIDE B  redirected witness (~destination(s), with >A> and >F>)   identical = yes
SIDE A  plaincall / callbuiltin / calltraceon / calltraceoff / tracestmt   rc 0, no loud, matching
        select-7.3                       rc 249, stdout empty     (oracle 249)
        trace_oracle 41/0    trace_indent 23/0    ir_recorded 8/0
        corpus                           490 of 490 matching
```

Both directions matter and both were checked: a check that wrongly said "redirected" would leave
the trace targets red, and one that wrongly said "not redirected" would break the witness. Neither
happened, so the check is not merely permissive. The corpus is 490 with the new witness in it.

Two constants would have failed *silently* had I guessed them, and reading the code caught both:
`QUEUE_ITEMS` is `b"ITEMS"`, not `b"QUEUE_ITEMS"`, and the store hangs under the **Array** class
scope, not Queue's. Either mistake returns `None`, which reads as "not redirected" -- i.e. it would
have looked exactly like success on every currently-red test while quietly disabling the feature.

**Step 3's target is measured**, and one rule is not what pass-through would predict:

```
linein=[a line]   lines=[1]   charin=[c]   pull=[A LINE]
LINEIN n= 0 / LINES n= 1 opt= NORMAL / CHARIN n= 0 / LINEIN n= 0
```

`lines()` answers **1** although the sink returned **7**, so the builtin normalises the monitor's
answer rather than passing it through.

**The conversion fix's witness covers conversion only.** The draft ended with the `.K~new` refusal,
whose `running REXX` attribution still differs, so landing it would have encoded a known divergence
inside a witness. The refusal stays recorded as an open defect instead.

#### Landed: `3809be075` and `2141620a7`

`3809be075` "Write a stream's argument through its string value" -- the conversion fix, its own
witness, conversion only.
`2141620a7` "Route output and trace through the monitors that own them" -- Task 11 Steps 1-2, the
trace witness, and `oracle-crashes.txt` entry 14.

Split so each commit stands on its own: `phase-7.txt`'s trace row was held back from the first
commit and restored for the second, because a corpus run at the earlier commit would otherwise
name a witness file that does not exist there, and this project does run the corpus at earlier
commits in worktrees.

Measured at the second commit: `cargo fmt --all --check` 0, `cargo clippy --workspace
--all-targets -- -D warnings` 0, corpus **491 of 491 matching**, `trace_oracle` 41/0,
`trace_indent` 23/0, `ir_recorded` 8/0, `sourceline_oracle` green.

Left uncommitted on purpose: the `sayloop` bench axis and `bench-programs/sayloop.rex`, which are
Step 4's, and the plan document. Sweeping the bench files into a routing commit would have put an
unmeasured benchmark axis inside it.

**Step 4 is now a clean A/B**, because both sides are committed: `3809be075` has no `SAY` routing
and `2141620a7` has it. The figures already in `sayloop.rex`'s own comment predate the routing and
so cannot answer Step 4's question. The trace fast path already built is a *correctness*
requirement -- the three per-clause state families -- and does not pre-empt the `SAY` decision the
plan reserves for measurement.

#### Step 4's instrument: instructions yes, cycles no

`perf stat -e instructions:u,cycles:u /bin/true` counts **instructions** (146,636) and reports
`cycles:u` as `<not counted>` at 0.00%, suggesting the NMI watchdog. `perf_event_paranoid` is -1,
so counting is permitted; there is simply one usable slot, which matches the environmental limit
already recorded for this machine. **Step 4's figures are therefore in instructions and say so.**
For a crate-against-crate A/B that is the right instrument anyway -- deterministic here, where
cycles drift a few percent between runs.

**A near-miss worth recording.** My first check piped `perf stat` through `tail -6`, which prints
the timing block and cuts the counter lines above it, so it read as "no counter available". One
untruncated re-run showed the count was there all along. Trusting the first read would have
retired the instrument on a machine property that does not exist -- the same shape as this
project's earlier truncation and environmental-absence failures, and the second time today that a
truncated or partial read nearly became a conclusion.

The A/B driver interleaves the two arms within every repetition rather than running them in
blocks, because blocked runs once invented a 5% regression on this project that was really a 5%
improvement. `sayloop.rex` is untracked and so absent from the baseline worktree; both binaries
are pointed at the same absolute file in the main tree.

#### The measurement harness, null-controlled

Run with the **same binary in both arms**, over a 2,000-iteration `SAY` loop:

```
rep 1   base=185,703,888   head=185,695,279
rep 2   base=185,689,949   head=185,741,362
rep 3   base=185,702,872   head=185,694,514
ratio head/base = 1.0001  (0.01%)
```

The harness is sound -- it parses, compares, and reports 0.01% where the answer is zero. **But my
prediction was wrong in a way worth keeping**: I predicted identical counts, and the counts vary
by roughly +/-0.03% run to run. The "deterministic to eight significant figures" figure this
project has recorded belongs to a different setup and does not hold here.

So Step 4 has a **noise floor of about 0.1%**, and a "no regression" verdict has to be stated
against that floor rather than as an exact match. Reps raised from 3 to 7 for the real runs, and
`rexxcps` is measured alongside `sayloop` because the standing rule is that every benchmark
includes it.

Had I skipped this control and read a real 0.05% difference as signal, the fast-path decision
would have rested on noise -- which is the failure the control exists to prevent, and the reason
it was run before the measurement rather than after a surprising result.

### Task 11 Step 4: the measurement, and what it decided

Interleaved, 7 reps, `instructions:u`, release builds of `3809be075` (no `SAY` routing) against
`2141620a7` (routing, no fast path):

```
sayloop   base    189,494,671   head  1,825,464,942   ratio 9.6333   (+863%)
rexxcps   base 20,573,761,193   head 20,793,621,354   ratio 1.0107   (+1.07%)
```

**~16,360 extra instructions per `SAY`**: each one sends a Rexx message to a `Monitor`, whose
`UNKNOWN` forwards through Rexx-level dispatch to a `Stream`. Both figures are far above the 0.1%
noise floor the null control established.

**The plan's own worry is settled by this, in the plan's favour.** Step 4 anticipated that
`sayloop` might read ~0% and leave the 1% criterion satisfiable and worthless. It did not: without
the axis the suite would have seen only `rexxcps`' 1.07%, and the 9.63x -- the thing actually worth
fixing -- would have been invisible. The axis earned its place.

`rexxcps` regressing 1.07% while barely executing `SAY` in its timed section is **not yet
explained**, and no explanation is offered here. It is recorded as open.

So the fast path is required rather than optional, and it is the same shape as the trace one:
`route_ends_at(route, terminal)` generalised over the terminal stream, with `bootstrap_stdout`
recorded at mint beside `bootstrap_stderr`.

**`SAY` needed its own side-B witness and did not have one.** The fast path skips delivery whenever
the route looks unredirected; if that check were wrong in the permissive direction, `SAY`
redirection would silently stop working and every corpus program would stay green, because none of
them redirect `.OUTPUT`. That is the one-sided-verification trap twice already hit today with
trace. Oracle answer for `output_redirection.rex`: `delivered 2`, `SAY[one]`, `SAY[two]`, rc 0.

#### The fast path works and is still too slow -- prediction wrong again

```
sayloop   base    189,493,486   head   265,897,502   ratio 1.4032  (+40.32%)
rexxcps   base 20,573,838,739   head 20,795,543,702  ratio 1.0108  (+1.08%)
```

I predicted ~1.00-1.05x. The delivery is gone -- 9.63x to 1.40x -- but the **check itself costs
764 instructions per `SAY`**, so re-deriving the route per line is not viable. That is exactly why
the plan specified a generation counter instead of a bare re-derivation; the plan was right and my
"measure first, the counter may be unnecessary machinery" reading of it was half wrong. The
counter is needed; what the measurement settled is that it is needed for *cost*, not for
correctness.

Correctness is green throughout: `SAY` redirection identical, trace redirection identical, corpus
**491 of 491**, `trace_oracle` 41/0, `trace_indent` 23/0, `ir_recorded` 8/0.

**`rexxcps` is unchanged by the fast path** -- 1.0107 before, 1.0108 after. So its regression is
neither the delivery nor the check: it is a fixed cost from the routing commit that tracks clauses
rather than `SAY`s, about 11 instructions per clause. `Interp` gaining three fields and shifting
layout is a candidate, but this project has measured dead code alone moving an axis up to 4%, so
that stays a hypothesis needing its own control rather than a recorded cause.

#### The 764 instructions split 58/42, and the prediction missed again

Short-circuiting `route_ends_at` to skip the queue walk while still paying `local_route`:

```
walk skipped    head 221,394,870   ratio 1.1684   ->  319 instr/SAY  (local_route)
walk included   head 265,897,502   ratio 1.4032   ->  764 instr/SAY  (total)
                                                  ->  445 instr/SAY  (the walk)
```

My prediction named two outcomes -- "the walk dominates, near 1.00" or "the lookup dominates,
above 1.2" -- and it is **neither**: 58% walk, 42% lookup. That is the third prediction in this
area to miss, and all three missed the same way: I keep expecting one dominant cause where the
cost is split. The practical consequence here is concrete -- caching only one half would have left
~17% or ~23% and still looked like a fix.

So the cache has to hold the **whole verdict**, not a part of it. Invalidation must be
conservative: over-invalidating only recomputes, while under-invalidating silently stops
redirection working -- the dangerous direction, and the one no corpus program would catch, since
none of them redirect. Three ways the answer can change: `store_local`; an ordinary directory put,
because `.local['OUTPUT'] = x` never goes through `store_local`; and a destination queue being
pushed or pulled, which is how `~destination` works.

#### The invalidation set, and how to keep it honest

Enumerated: six queue mutators (`init`, `queue`, `push`, `pull`, `delete`, `put`; `peek`, `at`,
`items` and `size` are read-only), the hash writers (`insert_in` plus `native_hash_remove`,
`native_hash_remove_item`, `native_hash_remove_entry`), and `store_local`, which is the only
`.local` writer and has three call sites, all in the mint.

**This is the same smell I objected to earlier**, when I rejected saving per-clause interpreter
state field by field: a hand-enumerated list that is silently wrong if it misses one entry. The
asymmetry is what makes it dangerous -- over-invalidating only recomputes, while
under-invalidating stops `SAY` redirection working, and **no corpus program would notice, because
none of them redirect `.OUTPUT`**.

Two things make it acceptable rather than a hope. If the queue mutators share one write helper the
set collapses to a few choke points rather than ten sites. And this project's own idiom applies
directly: assert in the accessor that the cached verdict equals a freshly computed one under
`debug_assert`, then invert the assertion once to prove it can fail. That turns a missed writer
from a silent wrong answer into a red debug gate -- which is the gate this repository already runs
precisely because `--release` compiles such assertions out.

#### No single queue writer, and over-invalidation is not free

The six queue mutators reach different generic *Array* helpers: `push` -> `array_splice_slot`,
`pull` -> `array_splice`, `queue` -> `append_slot`, `put` -> `write_slot`, `delete` ->
`native_array_delete`. There is no one choke point to bump.

**That kills the obvious design, and corrects something I wrote two entries ago.** I said
over-invalidating "only recomputes", which is too glib: recomputation costs 764 instructions, so
bumping a generation on every array write would let any program that mutates an array in a loop
invalidate the cache continuously and land straight back at 1.40x. Over-invalidation is cheap only
while invalidation stays rare, and hooking generic array writes makes it common.

So invalidation has to be scoped to what can actually change the answer: a write to `.local`'s
`OUTPUT` entry -- `insert_in` and `take_in` are single choke points, and `store_local` is the only
`.local` writer, all precise and rare -- and a push or pull on a monitor's own destination queue,
which is the awkward half because those go through the generic array helpers.

#### The seam is free, so the cost is the lookups themselves

`env_seam::admit`'s entire body is `let _ = (interp, scope, name); Ok(Admitted(()))` -- a no-op.
So my hypothesis that the clearance dominated `local_route`'s 319 instructions is **wrong**: that
cost is the hash probe of `.local['OUTPUT']`. Both halves of the 764 are hash lookups -- one
directory probe, then two pool lookups and two class lookups in the walk -- and only a cached
verdict removes both.

That is the fourth wrong guess in a row on this axis, and they share a shape: each time I
attributed cost to the machinery that *looked* expensive (clearance, a single dominant half) rather
than measuring where it actually sat. The isolation runs, not the reasoning, have been right every
time.

**Design settled.** `Directories` holds the `local` handle, so directory-write invalidation is
precise: `store_local`, plus `insert_in`/`take_in` when the receiver is that handle. For a
monitor's destination queue -- whose writes go through generic array helpers -- the generation is
bumped unconditionally in those helpers. A bump is an increment; what I warned about is
invalidation *frequency*, and that only bites a program that both says heavily and mutates arrays
heavily. `sayloop` mutates no arrays, so it stays cached, which is the axis the regression appears
on. Documented trade, not a hidden one, and a `debug_assert` that the cached verdict equals a fresh
computation keeps a missed writer from becoming a silent wrong answer.

#### The bump set, reduced by reading the writers

`append_slot` calls `write_slot`; `array_splice` delegates inserts to `array_splice_slot` and
removes inline; `native_array_delete` mutates through those. So the leaf mutators are **three** --
`write_slot`, `array_splice_slot`, `array_splice` -- and bumping them covers all six queue
operations. Bumping both `array_splice` and `array_splice_slot` double-counts an insert, which is
harmless because the generation only has to change. With `store_local`, `insert_in` and `take_in`,
the whole invalidation set is six sites, all leaves, none of them guessed.

**The cost being accepted, stated rather than hidden:** `write_slot` is every array element write
in the system, so a program that both builds arrays in a loop and says heavily re-decides its
route each time. `sayloop` builds no arrays, which is why it is the axis that shows the win.

**Still owed: inverting the `debug_assert` to prove it can fail.** It is the justification for a
hand-enumerated bump set, and an assertion that never fires is indistinguishable from one that
cannot. Removing one bump and confirming something goes red is what turns that justification from
decoration into evidence. Eight tests have shipped in this project that could not fail, two of
them fixes for earlier such findings, so this is not a formality.

#### Step 4 done: 9.63x, then 1.40x, now 1.0111

```
                 no routing -> routing -> +fast path -> +cache
sayloop            1.0000      9.6333      1.4032       1.0111   (+1.11%)
rexxcps            1.0000      1.0107      1.0108       1.0107   (+1.07%)
```

Per `SAY`: 16,360 instructions delivered, 764 re-decided per line, **21** on a cache hit.
Correctness green at every step -- both redirection witnesses byte-identical, corpus **491 of 491**
under the *debug* gate with the `debug_assert` live across every program, `trace_oracle` 41/0,
`trace_indent` 23/0, `ir_recorded` 8/0, `fmt` 0, `clippy` 0.

**The residual ~1% is one phenomenon, not two.** `sayloop`'s 2.1M over roughly 200k clauses is
about 10 instructions per clause; `rexxcps`' 220M over roughly 20M clauses is about 11. They agree,
so the leftover is a uniform per-clause cost from the routing commit and is definitively not
`SAY`-related -- the fast path and then the cache each left it untouched. That is a claim about
shape; naming a cause needs its own control, since this project has measured dead code alone
moving an axis by several percent.

#### The first inversion was inconclusive, and saying so matters

Removing the `array_splice_slot` bump -- the one `Queue~push` uses, and so the one
`~destination` redirects through -- left the witness **byte-identical with no assertion fired**.
My prediction said it would diverge or trip.

The reason is in the witness: its sink does `seen~append(...)` on every `SAY`, and `append_slot`
reaches `write_slot`, which bumps anyway. A different bump masked the one removed. So the control
**did not demonstrate that the assertion can fail** -- it demonstrated that the bump set is
redundant for this witness. Recording that as a pass would have been exactly the
"control that could not have run" failure: a load-bearing check that looks green because it never
exercised its subject.

It does yield one real fact about cost. `write_slot` fires on *any* array append, so the cache is
invalidated far more often than "somebody redirected output". That is safe, and it means
`sayloop`'s 1.11% holds because that program touches no arrays; a say-heavy loop that also appends
to an array would re-decide its route each time and sit nearer 1.40x.

The sharper control -- disabling `bump_route_generation` outright -- follows, with the witness's
own restore-then-say structure as the discriminator.

#### The second inversion fired: the assertion is live

Disabling `bump_route_generation` outright:

```
rc=101   assertion fired
environment.rs:554: the cached SAY route is stale, so some writer does not bump
Interp::route_generation
  left: Some(ObjRef(852))    right: None
```

`left` is the cached "deliver", `right` the fresh "direct" -- the stale-after-restore case, at the
exact `SAY` predicted. So the `debug_assert` can fail, names the reason, and is real evidence for
the hand-enumerated bump set. The first inversion's inconclusiveness is what made this one
necessary, and the pair together are the argument: one bump removed is masked by the others, all
invalidation removed is caught.

**The stale-binary hazard bit again in the same shape.** Both inversions restore the source through
a trap that runs *after* the build, so `target/debug` is left holding the patched binary. That is
the third time this session; the rebuild is now part of the same command rather than a thing to
remember afterwards.

#### Landed: `508bc63fe` "Decide SAY's route once and remember it"

Step 4 complete. Verified on the restored build before committing: both redirection witnesses
byte-identical, corpus **492 of 492**, `sourceline_oracle` green, `trace_oracle` 41/0,
`trace_indent` 23/0, `ir_recorded` 8/0, `fmt` 0, `clippy` 0. Only the plan document is left
uncommitted.

Three commits now stand for this work: `3809be075` the stream-write conversion, `2141620a7` the
routing, `508bc63fe` the fast path and its cache.

**Step 3 inherits two lessons rather than starting fresh.** The fast path is required there for
*correctness*, not only cost: `linein()` is a builtin function and can be evaluated inside an
argument window (`zz = length(linein())`), which is exactly where a re-entrant delivery destroyed
the argument stack. So reads take the same `route_ends_at` gate with `bootstrap_stdin` as the
terminal, on the `route_generation` that already exists.

And `lines()` must not pass a monitor's reply through: measured, a sink answering **7** yields
**1** from the oracle's builtin. That needs care against the `C` option, which is a real count --
assuming pass-through would have been wrong in a way only a redirected witness could catch.

#### Correction: Step 3 does not inherit the argument-window hazard

One entry ago I wrote that Step 3's fast path is required for *correctness*, because `linein()` is
a builtin function and could be evaluated inside an argument window. **Reading the code says
otherwise.** `send_to_stream` already resolves the name and *sends a message to a Stream object*
today, and the corpus passes with builtin reads inside expressions. The difference from the trace
case is where the send sits: a builtin's body runs **after** `CallArgs` has consumed the pushed
arguments, while the trace emitters fired **between** `PushArg` and `CallArgs`. Only the second is
inside the window.

So that note was written from the lesson rather than from the code -- the right instinct applied
to the wrong site. Step 3's fast path is a cost question, not a correctness one, and it should be
justified by measurement rather than by analogy.

**What Step 3 actually is, then:**

- builtins (`linein`, `charin`, `lines`) already send; only the receiver is wrong -- ours resolves
  to the `.STDIN` stream where the oracle sends to `.local`'s `INPUT` monitor;
- `PARSE PULL` and `PARSE LINEIN` read the descriptor with no send at all, and the oracle sends
  `LINEIN` for `PULL` too (measured);
- `LINES` clamps `N` to 0/1 inside the standard stream today, so a reply arriving from a
  redirected monitor has to be clamped where the builtin sees it -- measured, a sink answering 7
  yields 1.

### Task 11 Step 3: measured, and smaller than expected

**The builtin half already routes.** `standard_stream_name` answers `.INPUT`/`.OUTPUT`/`.ERROR`
(Task 10), so `linein()`, `charin()` and `lines()` reach the monitor today. Measured against a
route whose destination answers the messages itself:

```
              oracle   crate
linein=       a line   a line   OK
LINEIN n=     0        0        OK
CHARS n=      0        0        OK, and 9 passes through
LINES opt=    NORMAL   N        the spelling
linesN=       1        7        the clamp
LINES opt=    C        C        OK
linesC=       7        7        OK
```

**The spellings are asymmetric**: `N` goes out as `NORMAL` while `C` stays `C`. I was about to
write `COUNT` for the counterpart. And the clamp is only for the quick option -- the oracle
answered 1 for `NORMAL` and passed 7 through for `C`, so clamping both would have destroyed the
real count.

**The instruction half does not route at all.** With `.input~destination(s)`:

```
oracle   pull=[from the route]  linein=[from the route]  seen 2   (LINEIN n=0 twice)
crate    pull=[]                linein=[]                seen 0
```

That is Step 3's actual gap: `PARSE PULL` and `PARSE LINEIN` read the descriptor directly.

**A hazard to measure before changing them.** `input.rs` records that `.STDIN~LINEIN` raises
`NOTREADY` at end of input where `PARSE LINEIN` cannot tell EOF from an empty line. Routing the
instruction through the monitor to the stream could therefore introduce a `NOTREADY` the
instruction never had -- a behaviour change no corpus of non-redirecting programs would surface.

Also removed a comment in `resolve_stream` that had become false: it said the standard names route
"straight to the Stream objects rather than through the Monitors... until they exist", which Task
10 settled, and it contradicted the same function's doc comment twelve lines below.

#### The `LINES` fix lands, and both instruction reads diverge at EOF

`lines()` is now byte-identical to the oracle: `NORMAL` on the wire, `1` for the quick option, `7`
passed through for `C`, `CHARS` unchanged.

**`PARSE PULL` alone at end of input:**

```
oracle   rc 9, NOTREADY at line 2
crate    rc 0, pull=[] then done
```

So `PARSE PULL` diverges exactly as `PARSE LINEIN` does. Measuring it on its own mattered: in the
earlier probe the run exited at the `linein` clause, so the `pull` clause never executed, and any
reading of its behaviour from that run would have been a claim about a line that did not run --
the "control that could not have run" shape, one step from being repeated.

**A false comment sitting on top of the code being changed.** `input.rs` says `.STDIN~LINEIN`
raises `NOTREADY` at end of input "where `PARSE LINEIN` cannot tell them apart". The oracle *does*
tell them apart -- it raises. The sentence states our own behaviour as though it were a measured
oracle property, and it sits directly above the readers Step 3 must change, where it reads as
justification for leaving the divergence alone.

Remaining for Step 3: route `pull_line`/`linein_line` through `.INPUT` with a `bootstrap_stdin`
terminal mirroring `output_route`; raise `NOTREADY` at EOF on the unredirected path through the
existing `raise_notready`; correct that comment; land `input_redirection.rex`. Both readers answer
`Vec<u8>` today and must answer a `Result`, so their `parse_template.rs` callers change with them.

#### Step 3 written: the instruction reads route too

`pull_line` and `linein_line` now answer `Result<Vec<u8>, Failure>` and send `LINEIN` with no
arguments to whatever `.local` holds under `INPUT`. Two call sites, both in `parse_template.rs`,
inside a function that already returns `Result`.

**No fast path here, deliberately, and for a different reason than `SAY`.** `.STDIN~LINEIN`
already sets `NotReady` and calls `raise_notready` at end of input, so routing inherits the correct
`NOTREADY` for free; a fast path would mean re-implementing that raise on the direct path and
inviting it to drift. `SAY` needed one because delivery cost 16,360 instructions a line on an axis
that says in a loop -- reads are bounded by input and are not that axis. If a measurement later
says otherwise, `route_ends_at` and `route_generation` are already there to key it on.

A missing or `.nil` entry still reads the descriptor: that is entry 11b's licensed answer, where
the oracle SIGSEGVs.

Corrected the `input_line` doc comment, which claimed `PARSE LINEIN` "cannot tell [end of input and
an empty line] apart". The oracle tells them apart by raising, measured twice -- rc 9 at the
`PARSE LINEIN` clause and rc 9 at the `PARSE PULL` clause. The sentence described this crate's
defect as though it were an oracle property, directly above the readers that had to change.

#### A guard that ran, passed, and could not see its subject

The Step 3 build failed on two missing imports, but the four verification probes in the same
command ran anyway and all reported `match = NO`. Those results are **void**: they exercised the
previous binary, because the guard was

```sh
[ -x target/debug/rexx-run ] || exit 1
```

which tests that the binary *exists*, not that it was just built. Left over from the prior build,
it existed. Read at a glance, four "NO" lines look like the routing failing wholesale; they are
four measurements of code that does not contain it.

That is the fourth stale-binary incident this session -- twice from a trap restoring source after
a build, once from the A/B worktree, now from an existence check standing in for a freshness
check. The pattern is the same each time: something makes the binary and the source disagree, and
nothing in the command notices. The fix here is to key the guard on the build's exit status and
abort on it, rather than on the file being present.

### Task 11 complete: `21014c48c`

```
21014c48c  Read through `.INPUT` rather than the descriptor
508bc63fe  Decide SAY's route once and remember it
2141620a7  Route output and trace through the monitors that own them
3809be075  Write a stream's argument through its string value
d5f49c184  Keep the standard streams out of the finalization sweep
```

Step 3 verified on a freshly built binary behind a status gate: all four probes match, corpus
**493 of 493**, `input_oracle` 22/0 -- which matters, since widening the readers to raise
`NOTREADY` disturbed none of the 22 tests pinning input behaviour -- plus `trace_oracle` 41/0,
`trace_indent` 23/0, `ir_recorded` 8/0, `fmt` 0, `clippy` 0.

Five witnesses landed across the session: `uninit_after_std_streams`, `stream_write_conversion`,
`trace_redirection`, `output_redirection`, `input_redirection`. Step 5's remaining witnesses stay
unwritten and named rather than faked, because `Directory~setentry`, `~remove` and `~entry` all
refuse as Phase 5 -- which the plan anticipated.

**Tasks 12-22 remain**, so the phase is not close to finished. Task 12 is `arrayIn`, `arrayOut`,
`makeArray`, `supplier` and `say`.

Two constraints in Task 12's own text to respect rather than rediscover: it names
`dispatch/native.rs` for `stream_arrayin`, where every stream method touched this session lives in
`dispatch/stream.rs`, so the binding site is checked before editing a file named from memory; and
**"the Rexx halves already exist and must not be rewritten"** -- these are `StreamClasses.orx`
methods, so the work is the native piece they call.

### Task 12: measured first, and smaller than its text suggests

Oracle targets, on a three-line file:

```
arrayin    items 3, alpha/beta/gamma
makearray  items 3, class Array
supplier   class StreamSupplier, item/index pairs 1..3, ~next past the end raises SYNTAX 93
~arrayin('q')   93 on BOTH engines already
arrayOut + say round-trip   one/two/three
```

The crate answers all of them with one refusal: `the LIBRARY REXX entry point "stream_arrayin" is
not implemented (Phase 7)`, rc 120. So a single deferred entry blocks the whole group.

**Three corrections to what I assumed from the plan's text.** `dispatch/native.rs` does exist, so
the plan's file reference was right and my doubt was wrong. `stream_arrayin` binds to
`::METHOD line_arrayin PRIVATE`, not to `arrayIn` -- the `arrayIn` at orx line 102 is a mixin's
pure-Rexx loop that never calls it, and the real caller is line 229, `self~line_arrayin(array)`.
And the `say`/`makearray` bodies at orx 479/482 are **Queue**'s, not Stream's: they use
`self~queued` and `self~pull`. Stream's `say` is line 358, forwarding to `self~lineout`.

**`StreamSupplier` needs no native code.** It is an orx class over `stream~linein`, `~state`,
`~!query_streamtype` and `~!query_position`, and its `93.937` past the end is its own
`raise syntax 93.937`. So Step 2 is a thing to *verify* rather than implement -- the crate reached
none of it only because `arrayIn` died first. Implementing it in Rust would have duplicated
working Rexx, which is exactly what Step 1's "must not be rewritten" warns against.

#### `Cleared` rules out reusing `linein` directly

`Cleared` is `pub(super) struct Cleared(())` -- a token with a private field that only
`seam::clear` can mint, the dispatch security seam's evidence that a send passed the chokepoint.
So `stream_arrayin` **cannot** synthesise one and call `stream::linein` in a loop, which was the
strategy I proposed a step earlier. The token exists precisely to prevent that, and the type
system enforced it before I wrote the code rather than after.

Two routes remain: factor `linein`'s reading body into a helper both entry points call, or send
`LINEIN` per line. The helper is the right one -- a send per line re-enters dispatch for every
line of the file, and this entry point exists to avoid exactly that cost.

`append_slot` is likewise private to `collection.rs`, so filling the array needs either an exposed
helper or the `("Array", "APPEND", ...)` row. And since the orx pre-sizes the array with
`.array~new(self~lines)`, whether `append` lands on slot 1 or past the extent decides whether the
result has the right items -- extent is not item count, and that is measured rather than reasoned.

`linein`'s EOF path is confirmed as the propagation this needs: it sets `NotReady`, calls
`raise_notready(&name)?`, and only then answers the null string, so a `signal on notready` in the
orx sees the `Err`.

#### `.stdin~arrayin` is legitimate, which decides the helper's shape

Measured with stdin redirected from a two-line file: the oracle answers `items= 2`, `one`, `two`.
The crate refuses on `stream_arrayin` like every other member of the group.

That closes a gap that would otherwise have shipped silently. `linein` has two branches -- the
standard-stream one reading `interp.input_line()`, and the persistent-file one reading
`read_line_from` -- and had the extraction taken only the file tail, `.stdin~arrayin` would have
answered nothing while every file-based witness passed. The plan's Step 3 witness is a file, so
nothing in the task's own text would have caught it.

Settled for the implementation:

- `native_array_append` is `item_argument` + `single_dimension_only` + `append_slot`; the array is
  one the orx just built with `.array~new(n)`, so the dimension check has nothing to catch, and
  widening `append_slot` to `pub(super)` reaches `stream.rs` -- both are children of `dispatch` --
  without re-entering dispatch once per line.
- Appending to a pre-sized array lands on **slot 1**: measured on both engines, `.array~new(3)` is
  `items 0, size 3`, and one append gives `items 1, size 3, [1]=first, [4]=.nil`. Extent is not
  item count, so no index arithmetic is needed.
- `linein`'s persistent path (`ensure_open`, `seek_to_line`, the `count == 0` guard, `require_mut`,
  `read_line_from`/`offset_of`, then `NotReady` + `raise_notready(&name)?`) needs no `Cleared`, so
  the move is faithful rather than a paraphrase.

#### Task 12 Step 1 written

`stream_arrayin` is implemented and registered -- `implemented("stream_arrayin", Family::Stream,
Arity::Fixed(1), super::stream::arrayin)` replacing the `deferred` row.

The reading half of `stream_linein` moved into a shared `next_line`, so `LINEIN` and `ARRAYIN`
cannot grow separate answers at end of input. `linein` keeps its own guards -- the transient
position refusal, the non-input standard answer, `ensure_open`, `seek_to_line`, the `count == 0`
answer -- and the `count == 0` check hoisted after the branch preserves the original order on both
paths. `append_slot` widened to `pub(super)`, which `stream.rs` reaches as a sibling under
`dispatch`, so filling the array does not re-enter dispatch once per line.

Each item is `push_temp`'d before the append, because `text_built` allocates and the append grows
the array -- the unrooted-value trap that cost two GC panics earlier in this phase.

**The risk this carries is a refactor, not an addition**: every stream read now goes through
`next_line`, so the `stream_*` corpus programs and `input_oracle`'s 22 tests are the regression
surface, not the new witness. Verified against the captured oracle answers for the group probe,
`.stdin~arrayin` (the branch a file-only witness never touches), the `arrayOut`/`say` round-trip,
and `~arrayin('q')`, which matched before this change and must still.

#### Task 12 Step 1 verifies, and the witness finds an `arrayOut` divergence

Every captured target matches after the build: the group probe (`arrayin` 3, `makearray` 3 `Array`,
`supplier` `StreamSupplier` 1..3, past the end SYNTAX 93), `.stdin~arrayin` from redirected input,
the `arrayOut`/`say` round-trip, and `~arrayin('q')` at 93. Corpus **493 of 493**, `input_oracle`
22/0, `builtin_status` 26/0, `method_bodies` 23/0, `fmt` 0, `clippy` 0 -- so refactoring `linein`
through `next_line`, which every stream read now crosses, regressed nothing.

`native_entries` fails one test: the pinned set of implemented `LIBRARY REXX` entry points does not
contain `stream_arrayin`. That is a pin to move, and the plan names this entry point as Task 12's
work -- but the list's own comment says "The I/O entry points beside them are still deferred" while
already listing `stream_linein`, `stream_charin` and `stream_open`, so the sentence needs checking
before a row is added beneath it.

**The witness caught a real divergence my earlier probe could not.**

```
oracle   arrayin 1   [gamma]
crate    arrayin 3   [alpha] [beta] [gamma]
```

The earlier `arrayOut` probe matched because it opened the stream `write replace` first; the
witness calls `arrayOut` on a **fresh unopened** stream, and there the crate writes where the
oracle does not. Task 12 names `arrayOut`, so this is plausibly its work -- characterised in
isolation before deciding, since the witness also opens `write append` and calls `say`, and a
three-clause program is no place to attribute a cause.

#### Correction: `arrayOut` is not divergent -- the witness was

The isolated probe is byte-identical on both engines:

```
state before [UNKNOWN]   arrayout answered [0]   state after [READY]
file holds 2 lines: one, two
```

So the entry I wrote one step earlier -- blaming "`arrayOut` on an unopened stream", and even
noting Task 12 names `arrayOut` so it was "plausibly its work" -- is **wrong**. Writing to a fresh
unopened stream matches.

What the witness actually exercised is a **second** `.Stream~new('lines.txt')` on the same file
while the first, an anonymous temporary, was never closed: two live Stream objects over one file,
where the oracle loses the unflushed first write and this crate writes through. That is buffering
and object sharing -- adjacent to a divergence Task 10 already recorded about a program-built
`.Stream~new('STDOUT')` not sharing `SAY`'s buffer -- and it is not Task 12's subject at all.

**The fault was in the witness, not the code**: the program did something incidental to what it
meant to demonstrate, and I read the resulting difference as a defect in the feature under test.
The rewrite uses one stream object for writing and closes it before reading, so the witness tests
the array group and nothing else. That two-objects-on-one-file question is left recorded here
rather than folded into this task.

The `native_entries` comment is confirmed false and corrected in the same change: "The I/O entry
points beside them are still deferred" stood above a list already naming `stream_linein`,
`stream_charin`, `stream_lineout` and `stream_open`.

### Task 12 complete: `0c3451ad3`

"Fill an array from a stream, and share one reader with `LINEIN`". Verified before commit:
the group probe, `.stdin~arrayin`, the `arrayOut`/`say` round-trip and `~arrayin('q')` all match;
`native_entries` 24/0 once the pin moved; corpus **494 of 494**; `input_oracle` 22/0,
`builtin_status` 26/0, `method_bodies` 23/0; `fmt` and `clippy` clean.

Seven commits stand for this session: `d5f49c184`, `3809be075`, `2141620a7`, `508bc63fe`,
`21014c48c`, `0c3451ad3`.

**Recorded, not fixed:** two live `Stream` objects over one file diverge -- the oracle loses an
unflushed write where this crate writes through. Found by a witness draft, isolated away from
`arrayOut` (which matches), and left for whoever owns stream buffering.

**Task 13 is `.File`** and is much larger: 24 `file_*` entry points, a new `files.rs`, and nine
steps over predicates, listing, timestamps, mutation and class methods. Its global constraints
bind hard here -- no dependency beyond `rustix` and `filetime`, no process-global state, and
file-system probes confined to their own directory.

### Task 13 `.File`: the oracle's answers, measured before any code

**24 deferred `File`-family rows**, derived from the table rather than taken from the plan's
prose -- and my own `grep -c` of "31" was the wrong instrument, counting `file_separator`,
`file_path_separator` and doc-comment text.

`.File~new('.')` is rc 120 on `file_qualify` where the oracle answers `exists 1`, so Step 1's claim
that one entry point gates the class is measured, not assumed: `init` calls `normalizePathSyntax`
then `qualifiedPath`, which calls `qualifyImpl`. `.File~new` with no argument already raises 93 on
both engines.

**`paths::normalize` already answers what `file_qualify` needs.** Every shape matches its purely
lexical rule -- `./x` and `sub/../x` collapse, `sub/` loses the separator, `..` climbs, an absolute
path passes through -- and the oracle avoids `realpath` here for the same reason.

```
predicates   name       exists isFile isDir isHidden canRead canWrite length
             plain.txt    1      1      0      0        1       1        4
             sub          1      0      1      0        1       1       40   <- a directory HAS a length
             .hidden      1      1      0      1        1       1        2
             dangling     0      0      0      0        0       0        0   <- symlink followed
listing      list -> Array in read_dir order, genuinely unsorted; .nil for a non-directory
timestamps   DateTime, local wall clock; .nil when missing; the setter preserves the other stamp
class        isCaseSensitive 1 · listRoots [/] · temporaryPath /tmp
mutation     makeDir 1 then 0 · makeDirs 1 · delete 1/0 · empty dir 1 · non-empty 0
             renameTo 1 · same path 0 · existing target 0
             setReadOnly raises 91.999 at rc 165 -- Step 8's named adjacent case
```

The directory `length` of 40 is not in the plan, which mentions only that a *missing* file answers
0. And `isHidden` is true for a bare relative `.hidden`, which only works because the rule tests
the **qualified** path for `/.`.

`rustix 1.1.4` and `filetime 0.2.29` resolve offline; a `--dry-run` left `Cargo.lock` untouched.
Its pre-change state is recorded (867 lines, sha `4423be5d…`) because a cargo-driven lock rewrite
has silently shipped ~30 unreviewed dependency bumps on this project before.

#### The `File` contract, read off the Rexx halves

**Every native entry takes the qualified path as an argument** -- `existsImpl(self~qualifiedPath)`,
`canReadImpl(...)`, `lengthImpl(...)`, `listImpl(...)`, `deleteDir(self~qualifiedPath)`,
`renameToImpl(self~qualifiedPath, dest~qualifiedPath)`. None reads an object variable, so `files.rs`
is a set of stateless path-in helpers.

**The timestamp contract is the Rexx half's, not mine.** `getLastModifiedImpl` answers
**microseconds**, the missing sentinel is the literal `-999999999999999999` (the orx turns it into
`.nil`), and the setters take `date~fullDate`. So no `DateTime` is ever constructed in Rust.

Measured arithmetic: unix mtime `1583294767` becomes `63718895167000000` us. The difference is
`62135600400` seconds -- the year-1 epoch offset `62135596800` **plus 3600**, the timestamp's own
local offset (CET, +1 in March), not the current one. So the encoding is local civil time since
year 1, at the offset in force *at that timestamp*. A getter that applied today's offset would be
an hour out for half the year, and no single-timestamp witness taken in summer would show it.

`DateTime` is already byte-identical on both engines for `new`, `isoDate`, `fullDate` and
`baseDate`, and `fromFullDate` raises 97 on **both** -- so the plan's instruction to check
`fullDate` before relying on it is discharged: it works, and the method it warned about does not
exist anywhere.

`setReadOnly` raises 91.999 and still acts: mode `666`, `canWrite` 0, `canRead` still 1, and
`setWritable` restores it.

**Dependencies wired**: `rustix = "1.1.4"`, `filetime = "0.2.29"`. The lock gained six packages,
four of them transitive behind `rustix` (`bitflags`, `errno`, `linux-raw-sys`, `windows-sys`) --
worth stating, since the phase permits two by name and a `windows-sys` row in a Linux-only tree
invites the wrong conclusion.

#### The dependency audit is clean, and the last unknown is closed

`diff` of `Cargo.lock` before and after: **zero** removed or changed lines. The six new packages
are pure additions and no existing version moved, which is the check that would have caught this
project's earlier ~30 silent dependency bumps. The workspace still builds `--offline`.

`.DateTime~new(63718895167000000)` answers `2020-03-04T05:06:07.000000` and round-trips its
`fullDate` **identically on both engines**, so the getters need only return the right integer.

**The local-time conversion already exists**: `builtin/datetime.rs:792`,
`local_offset_micros(utc_micros)`. It takes the instant as its argument, which is exactly the
"offset in force at that timestamp" semantics the measured arithmetic requires -- reusing it is
also the only route available, since the phase permits no date library beyond `rustix` and
`filetime`.

So `files.rs` needs no new time logic and no new path logic: `paths::normalize` for `file_qualify`,
`local_offset_micros` plus the module's own basetime constants for the timestamps, and `rustix`
only for `access(2)` and the mode bits.

#### Correction: `chrono` is already here

`local_offset_micros` is implemented with **`chrono::Local`**, so this crate already depends on
`chrono` and my note one entry earlier -- that the phase "permits no date library beyond `rustix`
and `filetime`" -- was wrong about what is present. The constraint governs *new* dependencies;
`chrono` is already the thing `DATE` and `TIME` convert through. Reusing it needs no Cargo change,
and writing a hand-rolled zone conversion beside it would have been duplication of exactly the kind
that let two EOF paths drift in Task 12.

`files.rs` therefore needs no new machinery at all:

```
Step 1  paths::normalize + Interp::cwd_text        -- the pairing stream.rs:122 already uses
Step 2  std::fs::metadata + rustix access(2)
Step 3  std::fs::read_dir, unsorted
Step 4  UNIX_BASE_TIME + MICROSECONDS_IN_DAY + local_offset_micros, and filetime for the setters
Step 5  std::fs
```

`real_clock_base_time()` shows the exact shape for turning a `SystemTime` into a basetime, which is
what a file's mtime needs, and `UNIX_BASE_TIME` is the module's own constant rather than the
`62135596800` I derived by hand from one measurement.

#### Step 3 and Step 6 corners, and the plan for landing Task 13

```
list of a missing path        .nil
list of a non-directory       .nil
list of an empty directory    Array, 0 items
searchPath hit                a File -- /usr/bin/sh
searchPath miss               .nil
listRoots                     Array
isCaseSensitive (instance)    1
```

`chrono 0.4.45` is a direct dependency carrying a manifest comment about its vetting, so reusing
its `Local` conversion is sanctioned rather than smuggled. The three items `files.rs` needs --
`MICROSECONDS_IN_DAY`, `UNIX_BASE_TIME`, `local_offset_micros` -- are private to
`builtin/datetime.rs`, and only `date` and `time` are `pub(crate)`, so there is no existing
conversion helper to call instead and widening those three is the move. `lib.rs` has no `files`
module; `pub(crate) mod paths;` is the nearest precedent.

**Landing Step 1 alone first, deliberately.** `file_qualify` is the measured gate on the whole
class -- `.File~new('.')` is rc 120 without it and every other method is unreachable, so nothing
else can even be *measured* until it exists. It is also the earliest test of whether my reading of
the argument convention (`self~qualifyImpl(path)`, a stateless path-in call) is right, and getting
that wrong would invalidate the shape of all 24 entries rather than one.

#### Task 13 Step 1 written: `file_qualify`

`paths::normalize` against `Interp::cwd_text`, which is the pairing `stream.rs` already uses. The
contract was pinned shape by shape first: `a/b/../c.txt` collapses lexically to `<cwd>/a/c.txt`
although no such directory exists -- so no `realpath`, no symlink resolution, nothing asked of the
file system -- while `.` and `..` resolve against the cwd, `/` stays `/`, and an absolute path
passes through. `~string` keeps the original spelling, which the orx holds in `path`, so the native
side only ever answers the qualified form.

Widened to `pub(crate)` for the timestamp steps to come: `MICROSECONDS_IN_DAY`, `UNIX_BASE_TIME`
and `local_offset_micros`, the last with a line saying `.File` reads through it, since its
"offset at that instant" behaviour is the whole reason the getters can be correct across DST.

**A location I am following rather than endorsing.** The plan names `rexx-exec/src/files.rs`, but
every other native family lives under `dispatch/` -- `stream.rs`, `collection.rs`, `hash.rs` -- and
is registered as `super::stream::…`. Putting this one outside makes it the only family reached by a
different path, and it has to see `Cleared`, which is `pub(super)` inside `dispatch`'s own `seam`
module. If the build rejects that, the answer is to move the file under `dispatch/` rather than to
widen a security-seam token's visibility to suit a filename.

**The evidence Step 1 worked is a *changed* refusal, not a green run**: `.File~new('.')` should now
construct and `~exists` should refuse naming `file_exists` instead, showing the gate has moved to
the next entry point rather than that the class works.

#### The compiler settled the file's location, against the plan

`use crate::dispatch::Cleared;` is `E0603`: `Cleared` is `pub(super)` inside `dispatch`'s `seam`
module and re-imported privately into `dispatch`, so a module *outside* `dispatch` cannot name it.
The security seam is scoped that way on purpose -- the same scoping that stopped me synthesising a
token in Task 12.

**Ruling: move the file to `dispatch/files.rs`; do not widen the seam token.** The two options were
to widen `Cleared`'s visibility so `src/files.rs` could see it, or to put the file where every
other native family already lives (`stream.rs`, `collection.rs`, `hash.rs`) and register it as
`super::files::qualify` like its siblings. Weakening a dispatch chokepoint's scoping to satisfy a
filename the plan chose in passing would be the wrong trade, and the plan was accurate about
`dispatch/native.rs` while being loose about `src/files.rs`. Cost if wrong: a path that differs
from the plan's text, noted here.

I had flagged that inconsistency before writing the file and followed the plan anyway; the
compiler then made the case. Worth remembering that "the plan names this file" is weaker evidence
than the module structure every sibling already follows.

#### Step 1 lands: the gate moved

```
.File~new('.')   built [.]        <- construction works
~exists          rc 120, "the LIBRARY REXX entry point \"file_exists\" is not implemented"
file_qualify     identical to the oracle on all six path shapes
corpus           494 of 494
```

The evidence is a **changed refusal**, not a green run: same rc, next entry point, gate advanced by
exactly one. That was the prediction written before the build, and it is what distinguishes "the
entry point works" from "the class works" -- which for a hollow implementation would look the same
from the outside.

`native_entries` is red again on the same pin as Task 12, now for `file_qualify`. Expected
bookkeeping, but the failure text gets read rather than assumed: last time that same test also
surfaced a false comment above the list.

Next is Step 2, the seven predicates, because they share one `stat` plus a `rustix` access check
and their oracle table is already pinned -- including the two details I would have written wrong:
a directory answers **40** for `length`, and `isHidden` tests the **qualified** path for `/.`,
which is the only reason a bare relative `.hidden` reads as hidden.

#### Step 2 written: the seven predicates

`exists`, `isFile`, `isDirectory` from `std::fs::metadata` (following symlinks, so a dangling link
is false everywhere); `canRead`/`canWrite` through `rustix::fs::access` with `READ_OK`/`WRITE_OK`;
`isHidden` as "exists and the qualified path contains `/.`"; `length` from `metadata().len()`.

Three details measured rather than assumed, each of which I would have written wrong:

- **`length` is not `Stream`'s `query_size`.** A directory answers its own size -- 40 and 60 for
  two of them -- where `query_size`'s own doc records that a directory answers `0` there. They must
  not share a helper.
- **`length` needs no read permission**: a mode-`000` file still answers 2, because `stat` does not
  read the file.
- **`isHidden` tests the *qualified* path.** A bare relative `.hidden` has no separator of its own,
  so testing the given name would answer 0; the Rexx half hands over the qualified spelling, which
  carries `/.`.

`access(2)` alone suffices for `canRead`/`canWrite` at uid 1000 -- measured `none.txt` 0/0,
`ronly.txt` 1/0, `wonly.txt` 0/1 -- so the plan's "*and* an existence check" is belt-and-braces
here rather than load-bearing. **As root every one of those answers 1**, so any witness over them
is uid-dependent, which is worth knowing before one is written into the corpus.

**A comparison hazard avoided.** The stored `canRead`/`canWrite` capture was taken while a fixture
directory was mode `000`, and that same command chmod'd it back to 755 afterwards -- so diffing a
fresh crate run against the capture would have compared two different filesystem states and
invented a mismatch. Both engines now run against one state rebuilt inside the same command. It is
the file-system form of the stale-binary trap that has already cost four incidents today.

#### Step 2 matches, and the red test was a different one

All seven predicates are byte-identical to the oracle on both fixtures -- file, directory, dotfile,
dangling symlink, missing name, and the three mode-bit cases -- with both engines run against one
state built inside the same command. Corpus 494 of 494. Enabling `rustix`'s `fs` feature left
`Cargo.lock` **byte-identical**, and across the whole session's dependency work the lock has zero
removed-or-changed lines.

**The `native_entries` failure was not the pin.** It is
`invoking_an_unimplemented_entry_point_is_loud_and_names_its_phase`:
`corpus/gate-tables/native-entries/file.rex` binds `file_exists` to demonstrate the loud refusal,
and implementing `file_exists` made that program answer instead -- so it now pins nothing, and the
test says so in as many words. I had assumed it was the same set-drift assertion as before and
would have re-edited an already-correct list; reading the text is what caught it. The gate program
moves to an entry point still deferred.

**Step 3 measured**: `list` answers names in raw `readdir` order -- `.dot`, `two.txt`, `one.txt`,
`nested`, matching `os.listdir` exactly -- including dotfiles and subdirectories, excluding `.`
and `..`; `listFiles` wraps the same four as `File` objects with qualified paths; both answer
`.nil` for a file and for a missing path.

Since the order is raw `readdir`, a corpus witness that printed the listing verbatim would be
non-deterministic across filesystems. **The witness must sort before printing even though the
implementation must not sort** -- the corpus compares two engines on one machine, but a witness
that depends on directory layout is a flake waiting to happen.

#### Step 4 and Step 5 pinned, and a probe of mine that lied

**The timestamp setters are attributes, not methods**: `f~setLastModified` is 97.1 "does not
understand", while `f~lastModified = .DateTime~new(...)` works. And each preserves the other stamp
**in both directions** -- setting `lastModified` keeps `lastAccessed`, and setting `lastAccessed`
keeps `lastModified`. Checking both directions is what makes `filetime`'s `set_file_mtime` /
`set_file_atime` the right primitives; `set_file_times` sets both at once and would have passed a
one-directional test while quietly destroying the other stamp.

```
makeDir, missing parent      0
makeDirs, same path          1
makeDir over an existing file 0
delete of a non-empty tree   0
renameTo into a missing dir  0
```

**One row of my own probe was a fabrication and I nearly recorded it.** It printed
`makeDir over a file  1` -- but only because `afile.txt` did not exist yet; the program created it
on the *next* line. The honest answer is the second attempt, `0`. That is a probe-ordering mistake
producing a confident wrong number about the oracle, the same shape as the witness that diverged in
Task 12 for reasons incidental to what it was testing.

**The gate-table program gets repointed last.** `corpus/gate-tables/native-entries/file.rex` binds
`file_exists`, which is now implemented, so it pins nothing; it must name a still-deferred entry of
the same family. Sixteen remain -- and Steps 3 to 6 consume all sixteen. Choosing one now would
mean breaking it again within the hour, so it moves once the task's real deferred set is known.

#### Step 6 measured, and the return types are not uniform

```
isCaseSensitive     1
listRoots           Array, 1 item, and that item is a STRING "/"
temporaryPath       /tmp, and it is a FILE
searchPath hit      a File, /usr/bin/sh
searchPath miss     .nil          searchPath('sh', '.')  .nil
```

So some of these natives answer a string the Rexx half wraps in a `File` and others answer the
string itself. `searchPath`'s orx visibly wraps (`file = .file~new(file)`); `temporaryPath` never
appeared in my earlier method-list grep, which means that listing was truncated rather than
complete -- so its definition is read before its native half is written, not inferred from the
class it happens to answer.

**The timestamp arithmetic checks out across two independent stamps.** mtime `1583294767` gives
`63718895167000000` us and atime `1573560855` gives `63709161255000000` us; both imply exactly
`62135600400` seconds, which is `UNIX_BASE_TIME` plus 3600 -- CET in March and in November alike.
Two stamps agreeing rules out a coincidence that one would not: the getter is
`utc_micros + local_offset_micros(utc_micros)` on the existing helper, and the setter inverts it.
TMPDIR is unset here, so `/tmp` is a default rather than an environment reading -- worth knowing
before a witness depends on it.

#### Steps 3-6 match, and two "defects" that were my harness

With the fixture rebuilt per engine, every one of the sixteen entry points is byte-identical to
the oracle: listing (raw `readdir`, `.nil` for a file and a missing path), both timestamp
directions, all fourteen mutation answers, and the four class methods. `temporaryPath` matches both
with `TMPDIR` unset and with it set -- the second case I had never measured and would otherwise have
shipped as a hard-coded `/tmp`. `delete read-only 0` on both sides confirms the `access(W_OK)`
guard the plan asserted from the survey, which was the one behaviour here I implemented without
measuring first.

**Both apparent defects were the differential harness, and I misdiagnosed the second while
correctly diagnosing the first.** `run_both` copies the fixture with `cp -a` *after* the oracle has
run, so the crate inherited the oracle's side effects. For timestamps I spotted it. For the
mutation fixture I wrote that copying "cannot explain" it because "the crate ran against a fresh
tree" -- which was simply false: `made/`, `no/such/deep/`, `to.txt` and `other.txt` were all
already there, so `makeDirs` and `renameTo` correctly answered 0 for a directory and a target that
existed. I asserted a property of my own harness without checking it, one paragraph after
describing the very mechanism that made it untrue.

**The rule this earns:** a differential over a fixture the program *mutates* must rebuild that
fixture for each engine. Copying after one run carries the first engine's effects into the second.
Read-only fixtures copy safely; mutating ones never do. That is the third false failure from a
comparison harness today, after the stale binary and the chmod'd capture, and all three share one
shape -- the thing being compared was not the thing I thought I had set up.

`file.rex`'s comment is corrected: the file family now has no deferred row, so the probe test skips
a family whose rows are all implemented, conditioned on the registry rather than on a family name,
and asserts consistency instead of skipping silently. Holding a `file_*` row back to keep a refusal
alive would be a shell kept for a test's benefit.

#### The witness draft diverges on `length`, and the shape is familiar

```
oracle  length 0
crate   length 11
```

Both engines ran in their own fresh directory with the fixture rebuilt per engine, so the
copy-after-run trap is genuinely unavailable this time. But the program writes through
`.Stream~new('tree/one.txt')~arrayout(...)` and then closes a **second** `.Stream~new` object on the
same name -- the two-live-streams-on-one-file shape recorded during Task 12 as an unowned buffering
divergence, where the oracle loses the unflushed write and this crate writes through. `length`
itself matched the oracle in three earlier probes, so the entry point is very likely right and the
witness wrong for the third time in this phase.

That is being confirmed by isolating `length` over a file written and closed through **one** stream
object, rather than assumed from the resemblance -- the last time I reasoned from a resemblance
here ("the crate ran against a fresh tree") the reasoning was simply false.

The corpus harness gives each program an emptied directory under
`CARGO_TARGET_TMPDIR/corpus-run/<name>/`, and existing `stream_*` witnesses create files freely, so
a `.File` witness needs no cleanup of its own.

#### `file_length` is right; the witness was wrong, again

```
oracle   one object 11   two objects 0
crate    one object 11   two objects 11
```

So `file_length` matches for a file written and closed through a single stream object, and the
witness's `length 0` was the two-live-streams-on-one-file divergence recorded in Task 12 -- the
oracle loses the unflushed write where this crate writes through. Isolating it settled the question
rather than the resemblance did, which matters because the last time I argued from resemblance here
the argument was false.

**Third witness-design fault of this phase, one root cause.** `trace_redirection` omitted the call
shape that actually broke; `stream_array`'s draft kept two live streams over one file; this one did
the same again through `arrayout`. Each time the program did something incidental to what it was
meant to demonstrate, and the incidental part is what diverged. The corrected witness writes and
closes through one stream object per file, sorts the listing before printing, and keeps the
uid-dependent permission predicates out entirely.

#### Task 13 witness lands byte-identical

The corrected `file_queries.rex` matches on both engines and both runs leave nothing behind but the
program: predicates, listing, the timestamp round trip, every mutation answer, and the class
methods. Landed with a 42-line `sourceline_oracle` companion and a `phase-7.txt` row.

Three deliberate choices in it, each guarding against a way it would otherwise pass here and fail
elsewhere. It sorts the listing before printing, because `list` answers raw `readdir` order, which
belongs to the file system rather than the interpreter. It uses no permission predicate, because
`canRead` and `canWrite` answer 1 for everything when the run is root, so such a witness would pin
the user rather than the code. And it writes each file through one stream object, avoiding the
two-live-streams divergence that broke two earlier drafts.

`class-set.txt`'s `File` row gains `.File~new('.')`, with `status` and `reason` moved together so
the row does not contradict itself. `Stream`'s row carries the identical stale reason -- "no
construction program is committed" -- and is deliberately left alone: it is another task's row, and
widening this commit silently is how unrelated changes ride along.

#### Step 7 reverted: `class-set.txt` is generated, not editable

`gate_table_c` went red on `class-methods.txt and class-set.txt disagree on File's status`, and
chasing it showed the edit was wrong in kind rather than in detail:

- `corpus/docs/` is what `rexx-extract-docs` writes, and `rexx-extract/tests/extract_docs.rs`
  re-derives it from the `oodocs/` SVN checkout and `interpreter/`, comparing in both directions;
- a class's `status` and construction come from `CONSTRUCTION` and `UNCONSTRUCTIBLE` tables inside
  `rexx-extract/src/docs/classes.rs`, with assertions tying a status to the documentation sentence
  that grounds it;
- `gate_table_c` only ever *compares* its derived probes -- there is no writer and no refresh
  switch, so `gate-tables/methods/file__instance.rex` would have to be hand-matched to generated
  text;
- the not-covered sentence appears **7** times in `class-set.txt`, **118** in `class-methods.txt`
  and once in the derived probe.

**Ruling: revert the edit and leave Step 7 owed.** Setting `covered` honestly means teaching the
extractor and regenerating the row sets, which touches a generator and three generated artefacts
and needs the `oodocs/` checkout. Folding that into a commit about implementing `.File`'s sixteen
entry points would hide both changes. Cost if wrong: Task 13 lands without its `class-set.txt` row,
recorded here and in the commit message rather than quietly skipped.

The `RECEIVER_OVERRIDES` entry stays regardless -- it pins the method-body probe to an absolute
path, which is right whether or not the documented construction is ever committed, and the eleven
`unanswered -> diverge` rows it fixed were caused by the *relative* receiver, not by the row.

### Task 13 complete: `0f9023ba0`

"Build `.File` on the paths its Rexx half hands over". All 24 `LIBRARY REXX` entry points of the
`File` family implemented in `dispatch/files.rs`, each a function of the qualified path the Rexx
half passes in.

Verified on the committed tree: corpus **495 of 495**, `native_entries` 24/0, `gate_table_c` 22/0,
`method_bodies` 23/0 **without** the refresh switch -- so the refreshed table stands on its own
rather than being green only in the run that wrote it -- `sourceline_oracle` green, `fmt` 0,
`clippy` 0.

Nine commits now stand for this session: `d5f49c184`, `3809be075`, `2141620a7`, `508bc63fe`,
`21014c48c`, `0c3451ad3`, `0f9023ba0`.

**Owed, and recorded in the commit message rather than skipped quietly:** Step 7's `class-set.txt`
row. It is generated by `rexx-extract-docs`, so committing `covered` means teaching the extractor
and regenerating the row sets against the `oodocs/` checkout -- a documentation-pipeline change
touching a generator and three generated artefacts.

**A rule this task earned, after it cost three false failures:** a differential over a fixture the
program *mutates* must rebuild that fixture for each engine. Copying it after one engine has run
carries that engine's side effects into the other, and the resulting "divergence" looks exactly
like a defect. Twice I diagnosed it correctly and once I asserted my harness was innocent one
paragraph after describing the mechanism that made it guilty.

### Task 14 complete: `b3cdbdafb`

"Issue a command clause to the environment ADDRESS names". A command clause was the runner's
catch-all refusal; `command.rs` now runs it. The handler table, the rc-to-condition mapping,
`RC`/`.RS`, the in-interpreter `cd`/`set`/`unset`/`export`, and the trace the setting asks for.

Verified on the committed tree: corpus **498 of 498** matching (495 before, plus three new
witnesses), `fmt` 0, `clippy` 0, `trace_oracle` 41/0, `keyword_assertions` 7/0, `native_entries`
24/0, `sourceline_oracle` 1/0. A hand-built differential over every case measured for this task ran
23 of 23 matching on three descriptors before any of that.

**Rulings.**
- `.RS` is its own `Activation` field carried in `Inherited`, not on `rexx-num`'s `Settings` --
  that struct is NUMERIC digits/fuzz/form and is passed to `start_from_package` only for
  `::OPTIONS NUMERIC INHERIT`. Inheriting at `Entered::Label` and nowhere else is the measured
  split. Cost if wrong: one field moves.
- `ChunkTrace` gains a commands bit. I first ruled it would not need one, reasoning the failure
  retrace is a run-time decision -- true for the retrace, false for `TRACE C`, where whether a
  command clause echoes decides what ops the chunk carries. `clause_echoes` masks it in, or a
  chunk compiled under `TRACE C` compares equal to one under `TRACE O`.
- `+++` moves off `Coverage::Owned` even though the plan gives that row to Task 15. The row claims
  the prefix is unreachable and `command_dispatch.rex` makes that false; "a later task owns it" is
  a scheduling reason, not a truth reason. It was the last owned row, so the owner mechanism went
  with it rather than being kept alive to have something to describe.
- `::OPTIONS ERROR/FAILURE SYNTAX` is left to Task 15 as the plan assigns, though the C++ checks it
  in the same function. There is no `Raised` constructor for its 98.9xx error yet, and half-building
  one here would be the shell this project forbids.

**Two defects this task's own witnesses caught, both mine.**
- `$NAME` expansion: I implemented the shell's identifier rule. `sys_process_export` ends a name
  only at `/`, `:`, `$` or the end, so `$V-tail` looks up `V-tail` and stores nothing.
  `command_internal.rex` diverged on exactly that line and nothing else would have found it.
- I wrote a measurement into a comment before running it -- that an internal call inherits `.RS`
  and a routine does not. It happened to be true when measured afterwards, which is luck, not
  process.

**Owed by Task 15**, recorded here rather than in a commit message: the four interactive-debug
producers of `+++` (`phase-4-exclusions.txt` keeps that row for them), and the `::OPTIONS ...
SYNTAX` overrides.

### Task 14 gate run, and what the baseline showed

Gates on `b3cdbdafb`: G1 fmt 0, G2 clippy 0, **G5 release corpus gate 0 at 498 of 498**, G3 101
(5 targets) and G4 101 (7 targets).

**A baseline at the parent `79d8fd2cc`, in a throwaway worktree with its own `CARGO_TARGET_DIR`,
settles the attribution.** Ten of the thirteen failures were already red before this commit:

    --lib            4  every_family_still_defers_something, and three ir::drive counter tests
    collect_stress   2  both
    dispatch_seam    1
    gate_table_c     1
    gate_table_d     1
    refusal_sites    1
    loud             0  -- PASSED at the parent

**Task 14 introduced exactly three**, every one a test asserting that a command clause stays
unimplemented: `run::tests::the_command_and_with_forms_stay_loud_and_name_phase_7`,
`dispatch::tests::a_reqstr_context_whose_instruction_is_another_phases_is_still_loud` (its probe
`.K` alone on a line IS a command clause), and `tests/loud.rs` via `owners.rs`'s `Command` and
`Address::Command` rows.

**Two of my own attributions were wrong before I ran the baseline**, and both were plausible
stories fitted to a symptom. I read `refusal_sites` as line drift from inserting `mod command;`
into `lib.rs` -- it was red at the parent. I suspected the `ChunkTrace` commands bit for the three
`ir::drive` counter tests and the two `collect_stress` ones -- also red at the parent. Without the
baseline I would have hunted a regression that does not exist and "fixed" a table whose redness
has another cause.

**Of the ten, three are Task 13's** and are red for a reason its own verification could not see:
`every_family_still_defers_something` is a **lib unit test** where Task 13 ran the `native_entries`
*integration* test, and `gate_table_c`'s 82 `file__instance.rex` rows only run under
`REXX_CORPUS_GATE=1`, which Task 13's check did not set. `dispatch_seam` needed `files.rs` in its
allowlist. The remaining seven pre-date Task 13 and are not attributed further here.

**Process gap, mine.** `docs/superpowers/plans/2026-09-04-phase-5c-followup.md:192` says a commit
touching `src/*.rs` must include `--test refusal_sites` in its fast checks. My pre-commit set had
neither that nor `--lib`, which is why three command-related unit tests reached a 27-minute gate
run instead of a ten-second one. The fast-check set for the rest of this phase is fmt, clippy,
**`--lib`**, **`refusal_sites`**, and whatever targets the change names.

### Task 14 test fixes: `7b14ca1d3`

"Retire the assertions that commands stay unimplemented". The three failures `b3cdbdafb`
introduced, all tests asserting a command clause refuses: `owners.rs` moves `Command` to
`Owner::InScope` and narrows the `ADDRESS` split to `a.io.is_some()` (tag `Address::With`),
`loud.rs` drops its `Command` witness and repoints the other at a `WITH` source, four committed
counts move, and two `run.rs`/`dispatch.rs` unit tests lose the rows that now run.

Verified: `loud` 9/0, `--lib` 793 passed / **4 failed -- exactly the pre-existing baseline set**,
`fmt` 0, `clippy` 0. `dispatch_seam` 6/0 where the parent was red, so the suite is one better than
it was found.

**`address with output stem o.` is not a `WITH` form**, and mistaking it for one nearly cost a
wrong fix. It left the loud test's list and now runs; that looked like a redirection silently
executing as a command. `WITH` is a keyword only after an environment or a `VALUE` expression
(`addressNew`), so the clause names an environment called `WITH` and runs `OUTPUT STEM O.` against
it -- the oracle answers `+++ "RC(30)"` at exit 0 and this crate is byte-identical. The old test
passed at the parent for the wrong reason: the refusal keyed on `command.is_some()`, which happened
to cover a clause that was never a `WITH` form.

**Owed cleanup:** the baseline worktree at `/tmp/claude-1000/baseline-79d8fd2cc` is still
registered (`git worktree list`). Kept in case Task 13's two remaining reds -- `gate_table_c`'s 82
`file__instance.rex` rows and `every_family_still_defers_something` -- get taken up, since
re-baselining costs a rebuild. The other seven worktrees in that listing predate this session.

**Still red, all pre-existing at `79d8fd2cc` and none of them this phase's doing so far:** three
`ir::drive` counter tests, both `collect_stress` tests, `gate_table_d`, `refusal_sites`. Plus Task
13's `gate_table_c` and `every_family_still_defers_something`, which are this phase's and remain
open.

### Task 15 Step 2 in progress: where `::OPTIONS ... SYNTAX` escalates

**The escalation happens where the condition is RAISED, not where it is delivered.**
`Activity::raiseCondition` (`concurrency/Activity.cpp:596`) reports the syntax error from inside
the handler's own `RaiseCondition`, so `RexxActivation::command` never regains control: no `RC`,
no `.RS`, and no failure retrace. The two catalogue entries already exist in `rexx-inventory`
(generated from `rexxmsg.xml`), quoting `&1` themselves -- 98.970 "ended with", 98.971 "failed
with", substitutions `(command, rc)`.

Measured, six probes plus a control:

    ::OPTIONS FAILURE SYNTAX, untrapped 127   no retrace; the only `*-*` is the error report's
    the same with `trace f`                   no retrace either
    ::OPTIONS ERROR SYNTAX, `trace e`, exit 3 no retrace
    CONTROL: `trace e`, exit 3, no directive  retrace present, exit 0
    ::OPTIONS FAILURE SYNTAX, TRAPPED 127     retrace present -- arming a trap clears the
                                              escalation, so the raise-time test is false
    ::OPTIONS ERROR SYNTAX, untrapped 127     retrace present, then 98.970: the raise-time
                                              condition is FAILURE and FAILURE SYNTAX is off, so
                                              it traces, is renamed, and the SECOND check catches
                                              the renamed ERROR
    `say trace()` under the directive         "N" -- the directive does not touch tracing

`RC` inside a `SIGNAL ON SYNTAX` handler is **98**, the raise's own major, not the command's code
and not a stale earlier value (checked against a program whose previous command set `RC` to 5).
`Raised::syntax` already carries that, so returning before assigning `RC`/`.RS` is right.

**My first attempt put the check in `raise_command_condition`**, which runs after the tracing, so
it emitted a `>>>`/`+++` pair the oracle never produces. `corpus/lang/command_options_failure_syntax.rex`
caught it -- and it caught it only because the witness carries a leading `SAY` and a `TRACE N`
default, which is the combination the earlier hand probes lacked. The check now sits immediately
after the command runs, keyed on the **original** status's condition; the one in
`raise_command_condition` stays and is what catches the renamed ERROR.

Two hypotheses were falsified on the way: that `::OPTIONS ... SYNTAX` changes the trace setting
(it does not, `trace()` answers N), and that `command()`'s own ordering explains the missing
retrace (it does not; the throw happens a level below it).

### Task 15 Steps 2 and 3 complete: `cb1b1296f`

"Escalate a command's condition where it is raised". `::OPTIONS ERROR|FAILURE SYNTAX` now gives
98.970 and 98.971, checked on the **original** condition immediately after the command runs --
before `RC`, the trace and `.RS` -- because `Activity::raiseCondition` escalates from inside the
handler's own `RaiseCondition` and `RexxActivation::command` never regains control. The check in
`raise_command_condition` stays for the renamed ERROR, which is the C++'s second test.

Verified: differential **10 of 10** on three descriptors (including the control that keeps its
retrace when the directive is absent, which is what rules out an over-broad fix), corpus gate
**500 of 500**, `sourceline_oracle` 0, `fmt` 0, `clippy` 0. `--lib` 793/4 and `refusal_sites` 4/1,
both exactly the pre-existing sets and unchanged by this work -- run this time, unlike before the
Task 14 commit.

Step 3 landed with the command dispatch in `b3cdbdafb`.

**Step 1 is deliberately not here.** `CONDITION('O')` is a loud refusal for every condition, not
just a command's, so the condition directory is shared machinery. Building it far enough to serve
commands would leave the same refusal standing everywhere else, which is the hollow-shell trap.
What exists to build on: `object_model.directory` and `package_object` are resolved handles, the
`List` class is complete with `NEW`, and `.context~stackFrames[1]~class~id` is already `StackFrame`.
What does not: **`TRACEBACK` has no machinery here at all** -- every occurrence of the word in this
crate is a traceback *line in an error report*, never a List object -- and that was checked with a
wide search rather than one guessed pattern.

### Task 15 Step 1, measured before writing any of it

`CONDITION('O')` is a loud refusal here for **every** condition, confirmed live at exit 120, and
`CONDITION('A')` one line above it in the same match is too. So Step 1 is shared condition
machinery, not command work; the plan's one line understates it.

The eleven indexes, measured and cross-checked against survey D 2.3 (they agree):
CONDITION, DESCRIPTION, INSTRUCTION, PACKAGE, POSITION, PROGRAM, PROPAGATED, RC, RESULT,
STACKFRAMES, TRACEBACK. `RESULT` duplicates `RC`; `POSITION` is the command clause's line;
`PROPAGATED` is 0; `INSTRUCTION` names the **trapping** instruction, so `CALL` under `CALL ON` and
`SIGNAL` under `SIGNAL ON` -- not the raising clause, which is what I first predicted.

**`CONDITION('A')`, never measured before and not named by the plan or by survey D:** `.NIL` inside
a command's ERROR handler and `.NIL` with no condition active, but an empty **Array** inside a
`SYNTAX` handler for `1/0`. Not uniformly `.NIL`. `ADDITIONAL` is genuinely not one of the eleven --
`~at('ADDITIONAL')` answering `The NIL object` is `Directory~at` on an absent key.

`STACKFRAMES` and `TRACEBACK` are **both Lists carrying the same trace lines**, innermost first,
and at depth two both include the caller's own clause. But the element types differ: `StackFrame`
objects in the first, plain `String`s in the second. Two probes were needed to see that -- the
first concatenated each element, which asks for its string value and cannot tell them apart, and
the second indexed with `[1]`, which on a `.List` is not positional and answered `.NIL` three times
while looking like data. A `StackFrame` answers `~name` (the program path), `~line` and
`~traceLine`.

What exists to build on, all confirmed rather than assumed: `object_model.directory` and
`package_object`; `build_frame(interp, depth)` already constructs a `StackFrame` and
`.context~stackFrames` works here (answering an **Array**, where the condition directory wants a
**List**); `dispatch/context.rs` implements the whole `StackFrame` surface. An earlier note here
guessed `StackFrame` was a shell -- that was wrong, from grepping `introspection.rs` when the
methods live in `context.rs`.

Construction discipline to copy from `context_stack_frames`: root the container before allocating
anything it will hold, because `alloc_with` collects before it allocates.

### Task 15 Step 1 in progress: the condition directory

Built in `src/condition.rs`, replacing the `CONDITION('O')` and `CONDITION('A')` loud refusals.
`7 of 7` condition shapes match the oracle on the full index **set** and on every value: command
ERROR under `SIGNAL ON` and `CALL ON`, a `RAISE`d ERROR, NOVALUE, SYNTAX, USER, command FAILURE.
The collector test passes for both trap kinds with `collections > 0` asserted.

**Four wrong diagnoses on the way, each caught by a measurement rather than by review.**

1. `Body::Native` for the two Lists. A `List` is a `Body::Instance` -- `is_list` asks
   `class_of_value`/`is_a` -- so a native-bodied one matched no `Primitive` arm and every send was
   refused before the program touched the object. `NEW` through the class is the route.
2. `POSITION` from `clause_state.line()`. Wrong once a routine has unwound.
3. `POSITION` from frame 0. **Also wrong, and for the same reason** -- after a `RAISE ... RETURN`
   the innermost surviving frame is the *caller's*, so both sources gave the caller's line. The
   line has to be captured at the raise, which is what `Raised.position` now does.
4. `ADDITIONAL` under USER called structural, then retracted, then correct on the third look: the
   C++ does not array-convert the *substitution list* for non-SYNTAX (that comment is right), but
   the condition object carries the **original object**, so the substitution list was simply the
   wrong source.

**A blind instrument.** The first differential enumerated with `o~allindexes~sort` and every case
"failed" with `method "ALLINDEXES" of class "Directory" is not implemented (Phase 5)` -- a
pre-existing gap, not a defect of this work. Rewriting the probes to read named keys fixed the
runs but made them blind to the index *set*, which is this task's central finding; the third
version prints present/ABSENT for all fifteen keys so the set is compared in both directions.

**Two rooting repairs, both where `E0027` cannot reach.** `TrappedCondition` gained an `ObjRef` and
`Activation::object_roots` destructured `condition: _`; `PendingTrap` gained one and
`Interp::object_roots` destructured `pending_traps: _`. Both now destructure the inner struct
exhaustively so the next field added to either is a compile error.

**Fixed in passing, both pre-existing:** NOVALUE's condition carried no `DESCRIPTION` although
`novalue_raised` already computes `derived_name_text` for its `::OPTIONS NOVALUE SYNTAX` branch;
and `exec_raise`'s `SIGNAL` arm dropped the raise's `ADDITIONAL` list entirely.

**`Raised` grew twice and `clippy::result_large_err` caught the first**: `result: Option<Vec<u8>>`
became `result_is_rc: bool` once measurement showed `RESULT` is always a copy of `RC`.

### Task 15 Step 1: `ADDITIONAL` took three attempts

The condition object's `ADDITIONAL` is **the object the raise named**, not a rebuild of it.

    additional 'JUSTONE'          oracle: a String
    additional (.array~new)       oracle: an Array of no items
    additional (.array~of('x','y'))  oracle: an Array of two

Attempt 1 emitted the substitution list only when non-empty, which gave one item holding `x\ny`.
Attempt 2 split the array for every condition, which fixed the two-element case and broke the
other two -- a `String` came back as a one-item `Array` and an empty `Array` vanished entirely.
Only the third carries the object, and it lives on **`Interp`**, not on `Raised`: a `Raised`
travels inside a `Failure` on the Rust stack, where no destructure can root it, while an `Interp`
field is covered by the exhaustive match in `object_roots` and so cannot be added without being
classified. `take()` at the build clears it, and a program raising twice was measured to confirm
the second raise does not inherit the first's value.

The substitution list is a different thing and stays as it was: it is what a catalogue message
renders from, and only a `SYNTAX` condition has one. That is why the C++ calls `requestArray`
inside its `SYNTAX` branch alone -- a comment in `exec_raise` I twice read as governing the
condition object too.

**Task 15 Step 1 is complete on every measured axis:** seven condition shapes agreeing on the full
index set and on every value, four `ADDITIONAL` shapes, both witnesses, the collector test with
`collections > 0` asserted, corpus **502 of 502**, `--lib` back to its pre-existing four failures.

Two pre-existing gaps closed on the way: `novalue_raised` built its condition with no
`DESCRIPTION` although it already computes `derived_name_text` for its `::OPTIONS NOVALUE SYNTAX`
branch, and `exec_raise`'s `SIGNAL` arm dropped the raise's `ADDITIONAL` list. Two `builtin::state`
tests pinning the old refusals are retired to pin the answers instead.

### Task 15 complete: `647be7ea1`

"Build the condition object a handler reads back". Step 1 lands here, Step 2 landed at
`cb1b1296f`, and Step 3 came free with the command dispatch in `b3cdbdafb`.

Verified on the committed tree: corpus **502 of 502**, `fmt` 0, `clippy` 0 workspace-wide,
`sourceline_oracle` 1/0, `--lib` 793 passed / 4 failed -- exactly the pre-existing set a baseline
at `79d8fd2cc` proved was not this work's. `refusal_sites` stays 4/1, also pre-existing at that
baseline; re-deriving that table would fold someone else's redness into this commit.

Four commits stand for this session: `b3cdbdafb`, `7b14ca1d3`, `cb1b1296f`, `647be7ea1` -- Tasks 14
and 15 of 22.

**Owed cleanup:** the baseline worktree at `/tmp/claude-1000/baseline-79d8fd2cc` is removed and
`git worktree list` no longer names it.

## Task 16: `ADDRESS ... WITH`

Steps 1, 2 and the half of 4 they own. `STREAM` and the stream-shaped `USING` objects still refuse
loudly and name Phase 7, which is Step 3.

`src/redirect.rs` holds all of it, split as the oracle splits it: a configuration is what the
parser built and what `ADDRESS env WITH ...` stores under an environment name; a context is built
at issue time by evaluating that configuration's sources and targets in the variable context then
in force. The permanent table sits on the activation behind an `Rc`, inherited by refcount, cloned
on write -- `checkIOConfigTable`'s lazy copy, which is what makes the inheritance one-way without
a per-call clone of a map almost every program leaves empty.

**Measured against the oracle before writing, each prediction recorded first:** `>K>` follows
`>>>` and sits on the *command* clause, not the `ADDRESS` one; targets evaluate after the command
string; the table is keyed upcased; a one-off configuration is stored nowhere; the merge is per
stream; `NORMAL` resets one; one object taking both streams interleaves them in write order; every
input line carries a terminator including the last; a collection target is emptied then appended
to; and p55's inheritance -- a `PROCEDURE` callee fills its **own** stem while the caller's keeps
what it had, byte-identical.

**Two predictions of mine were falsified.** "No `>>>` under `trace r`" -- a command clause echoes
`>>>` under `results` while its `>L>` stays hidden. And `address with output stem o.` is not a
`WITH` form at all: `WITH` is a keyword only after an environment or a `VALUE` expression, so that
clause names an environment called `WITH` and runs `OUTPUT STEM O.` as a command against it,
`+++ "RC(30)"` at exit 0. That is the second time this clause has caught me, and it means an
`ADDRESS` carrying neither a name nor an expression can never carry a configuration -- the
toggle-plus-configuration case I had written code for cannot occur.

**Three defects, every one found by running and none by reading.** A deadlock: `Command`
duplicates a `Stdio` rather than consuming it, so after the spawn three copies of the merged
pipe's writing half were open and the read waited for an end-of-file that could not arrive -- the
witness hung, and dropping the builder is the fix. A `SmallInt` not counted as a `String`, so
`with input using 42` would have asked a number for an array. And a stem *object*'s unresolved
tail falling back to the derived `IN.2` where the oracle answers the stem's own name `IN.`.

**The boundary is enforced, not described.** `owners.rs`'s `Address` split is three rows now, the
owed one picked out by whether any redirection names a `STREAM`; the two tests that asserted the
whole `WITH` form refuses are re-aimed at that row rather than deleted. Both of their sources have
to carry a command, because a target is evaluated where the command is issued -- an `ADDRESS env
WITH` alone stores a configuration and refuses nothing.

`keyword-exempt.txt` reddened, which is the derived table working: `ADDRESS::test_environment_
value_with` and `..._expression_with` now pass and came off it, and the Phase 7 bullet describing
them went too, leaving that file with no Phase 7 row.

**Process error, mine:** I started the full gate before committing. The rule is commit-first so the
gated tree is the committed tree by construction. The run was minutes rather than the 27-minute
suite and I held every write until it landed, but the ordering was mine to get right.

### Task 16 Steps 1-2 complete: `97d84e981`

"Redirect a command's three streams with ADDRESS ... WITH". Gate run over the committed tree, in
the right order this time -- commit, then gate, tree frozen until the status file said finished:

| gate | status | figures |
|---|---|---|
| G1 `fmt --all --check` | 0 | |
| G2 `clippy --workspace --all-targets -D warnings` | 0 | |
| G3 `test --release --workspace` | 101 | 2337 passed / 7 failed |
| G4 `REXX_CORPUS_GATE=1 memcap 8G test --workspace` | 101 | 2336 passed / 9 failed, **504 of 504** |
| G5 `REXX_CORPUS_GATE=1 test --release -p rexx-exec --test corpus` | 0 | **504 of 504** |

**Not one of those failures is this work's.** G3's seven are the set an ungated run at `7c72ce986`
already showed before any of it was written; G4 adds `gate_table_c` (82 `file__instance.rex` rows)
and `gate_table_d` (two named rows), which run only under the gate flag and match the figures
recorded for them. `keyword_assertions` appears in neither, which is the check that the exempt-set
fix holds. The corpus differential is 504 where it was 502 -- the two new witnesses, both
byte-identical on three descriptors.

**Two checks I had deferred, both answered by running, and one found a defect.** `is_stream_like`
does fire: a `.stream` object as a `USING` target refuses loudly at rc 120. But a `.rexxqueue`
target refuses with the *stream object* wording and names **Phase 7**, where D10 says a RexxQueue
target is Phase 10's -- the right refusal under the wrong owner, and Step 3 fixes it. `OPEN`,
`LINEOUT` and `CLOSE` resolve by name through the class dictionary (`READY:`, `0`, `READY:`), so
Step 3 sends them rather than reaching past the class.

**An instrument of mine was wrong and nearly became a finding.** A grep for `::class <name>` in
`crates/rexx-lib/` answered zero for every class including `Stream` and `File`, which plainly
exist; I had been about to record that the stream-shaped classes do not resolve here. The empty
result meant the pattern was wrong, not the world -- the same shape as every other too-narrow grep
this project has logged. Running settled it in one line.

### Task 16 complete: `4d136e8e2`

"Redirect a command's streams to and from a stream". Step 3 and the half of Step 4 it owns, so
Task 16 is done and **no instruction is owed to Phase 7** -- `owners.rs`'s `Address` split by
target kind existed only while `STREAM` did, and the count of Phase 7 instruction rows is asserted
at zero rather than deleted, so a later task cannot re-owe one quietly.

| gate | status | figures |
|---|---|---|
| G1 `fmt --all --check` | 0 | |
| G2 `clippy --workspace --all-targets -D warnings` | 0 | |
| G3 `test --release --workspace` | 101 | 2337 passed / 7 failed |
| G4 `REXX_CORPUS_GATE=1 memcap 8G test --workspace` | 101 | 2336 passed / 9 failed, **505 of 505** |
| G5 `REXX_CORPUS_GATE=1 test --release -p rexx-exec --test corpus` | 0 | **505 of 505** |

**Not one of those failures is this work's**: G3's seven are the pre-existing set, and G4 adds only
`gate_table_c` and `gate_table_d`, which run under the gate flag alone and match their recorded
figures. The corpus differential is 505 where the previous commit left it at 504 -- the new
witness, byte-identical on three descriptors.

**Everything was measured before it was written**, each prediction recorded first: a named stream
is opened and closed by the redirection while a `USING` stream object is written and left open
(`~state` still `READY`, bytes on disk); the open happens **before** the command runs, witnessed by
a side-effect file an unopenable-output command would have written being absent, so 98.920 and
98.999 are raised from `init` carrying the cwd-qualified name and the open's `ERROR:n`; two names
for one file collapse to one target and interleave (`o1` `e1` `o2`); the same file may be both
sides and round-trips intact; and `LINEIN` stops on the stream's **state**, because `''` is both a
blank line and the read past the end -- four `READY` reads over `l1`, `l2`, an empty line and `l4`,
the fifth `NOTREADY`.

**No buffering layer, and the divergence is mechanism only.** The oracle inserts a
`BufferingOutputTarget` when input and output name one file, because `readInputBuffered` gathers
input lazily while the command runs. Here the whole input is read at context-build time, before the
output target exists, so a truncating open cannot reach it. Witnessed, not argued.

**A defect the previous commit shipped**, found by the deferred check: a `RexxQueue` target refused
correctly under the wrong owner, Phase 7 where D10 says Phase 10, because `Loud::redirection`
assumed the phase. It takes the owner as an argument now.

**A test retired for a reason worth keeping.** `.REXXQUEUE` is not a resolvable environment symbol
in a bare `Interp`, so the unit test asserting the queue refusal could not construct its own
subject; `dispatch.rs`'s reqstr row pins it instead, library bootstrapped, exact message asserted.
In its place is the positive half of a boundary that has caught me twice: `address with output stem
o.` names an environment called `WITH`, and `RC` of 30 parts that reading from a `WITH` form, which
would have issued no command at all.

**Two more empty greps that meant nothing.** `::class` in `crates/rexx-lib/`, and `fn
resolve_search` in `require.rs` -- the function lives in `lib.rs`. Both described the pattern
rather than the world.

**Owed, found while preparing Task 17:** 24 sites still name Phase 7 as an owner -- the nine
`Sys*` routines, the external-routine and package-API gaps, `.local['STDOUT']`, `EnvScope::Local`,
and one this task introduced, `open_named_stream`'s `.STREAM` fallback. Task 22 Step 4 wants all of
them gone, and that is what its real size looks like.

**Plan corrected rather than the message carrying the work.** Task 17 said "consumes Task 2's
`SearchContext`"; no such type exists anywhere in `rust/crates/`. Task 2 delivered the capability --
`Interp::resolve_search`, `Interp::shadow_var`, the shadow cwd -- and never built the struct survey
E Q4 recommended, so the interface line now says what is actually there and that a pure, testable
search has to be extracted first.

### Task 17: external routines and package loading -- starting

Step 1's three `require.rs` infidelities are confirmed **in the code as written**, against survey E
§3.2's reading of `SysFileSystem` rather than against their effects: `program_extension` uses a
bare `rfind('.')` where `hasExtension` never examines the first byte, so `.hid` wrongly carries one
(`a.b` correctly does); `candidates` returns a flat `Vec` with no spelling+extension group
boundaries, which is exactly what `searchPath`'s "any other successful stat abandons the remaining
entries for that spelling+extension" needs; and it lowercases through Unicode where `strlower` is
byte-wise ASCII. The group abort belongs in `lib.rs`'s `resolve_search` loop, which today falls
through to the next candidate.

### Task 17 Step 1 complete: `cd2c1a6c4`

"Search for a program file the way the oracle searches". All three infidelities, each observable
today through `::REQUIRES` because that already resolves through this path.

| gate | status | figures |
|---|---|---|
| G1 `fmt --all --check` | 0 | |
| G2 `clippy --workspace --all-targets -D warnings` | 0 | |
| G3 `test --release --workspace` | 101 | 2340 passed / 7 failed |
| G4 `REXX_CORPUS_GATE=1 memcap 8G test --workspace` | 101 | 2339 passed / 9 failed, **505 of 505** |
| G5 `REXX_CORPUS_GATE=1 test --release -p rexx-exec --test corpus` | 0 | **505 of 505** |

Same pre-existing set as every gate this session; the passed counts rose by three, which is the
three unit tests this step added.

**Each fix was differentially verified, not merely unit-tested**, one probe per fix and each one
discriminating: a directory named `zdir.rex` beside the program with a real `zdir.rex` in the
current directory now fails 43.901 on both sides where the fall-through would have loaded the later
file; `::requires '.hid'` with both `.hid.cls` and `.hid` present loads the `.cls` on both sides;
`::requires 'LIBÉ'` with only `libé.rex` present fails 43.901 on both sides. Byte-identical on
three descriptors each.

**One control was reasoned rather than run**, and it is worth naming: that the *previous* code
loaded the later file in the abort case follows from reading it and is proven at unit level, where
`a_non_regular_hit_abandons_the_rest_of_its_group` carries its own control. Running it end to end
would mean building the parent commit in its own worktree. Step 6's corpus witnesses are what will
pin it.

**`first_regular` takes the stat as a closure**, which is what lets the abort rule be tested with
no file system at all -- `CARGO_TARGET_TMPDIR` exists only for integration tests, and
`resolve_search` is `pub(crate)` and so unreachable from one. The pure-function shape was the way
out of that, not a workaround for it.

**The enumeration lesson repeated, and cost a compile error.** I converted the `candidates` call
sites I had read and assumed that was all of them; one I had never read bound the result to a
`tried` variable. Grepping `candidates(` first would have listed the set. It is the same shape as
the empty greps above, one step over: acting on a sample as though it were the population.

### Task 17 Step 2: the external file call -- designed, not yet written

Three rulings, each made against the types rather than survey E's wording.

**Q1's wording has to bend.** It recommends dispatching the file through `Resolved::Routine`.
`InstalledRoutine` is `{ program, directive }` and `Entered::Routine` goes through
`body_of(program, Some(directive))`; a called file's main body is `body_of(program, None)`. Reusing
that variant means inventing a directive index other code trusts to be real. Step 2 gets its own
resolved kind walking the *same* sequence -- which is what Q1 actually asks for, one
activation-building path, and `run_loaded` already is that sequence for a main body.

**`run_loaded` needs two new parameters, not one.** It builds with `Activation::new`, so the
address starts at `AddressState::default()` and there is no seam for the ADDRESS the file route
inherits and the `::ROUTINE` route must not (t01). And it passes `None` for inherited settings
where `Entered::Routine` passes `Some(&caller.settings)`; NUMERIC starts at defaults for a called
file, but `::OPTIONS NUMERIC INHERIT` needs the caller's settings to inherit from.

**The file is re-read every call and 3.1 carries the resolved path.** No `required_packages` entry
(t05 rewrote a file between two calls and both versions ran), `merge_required` after a normal
return (t06, transitive by t16), and a resolved-but-unreadable file is 3.1 with the absolute path
(t12). `Raised::executable_file_unreadable` is genuinely `syntax(3, 1, ...)` -- checked, not
inferred from its name -- but its doc says the substitution is "the name as the program wrote it",
true of its one caller today and false of this one, so that comment gets corrected rather than left
to rot.

Q2's callee parse error is deliberately left to a later step: landing the call first and the error
conversion second keeps each witnessable on its own.

### Task 17 Step 2 complete: `a161e3860`

"Call an external Rexx file". A call matching no label, builtin, `::ROUTINE` or internal routine
now searches for a file named for it, as the oracle does before answering 43.1.

| gate | status | figures |
|---|---|---|
| G1 `fmt --all --check` | 0 | |
| G2 `clippy --workspace --all-targets -D warnings` | 0 | |
| G3 `test --release --workspace` | 101 | 2340 passed / 7 failed |
| G4 `REXX_CORPUS_GATE=1 memcap 8G test --workspace` | 101 | 2339 passed / 9 failed, **505 of 505** |
| G5 `REXX_CORPUS_GATE=1 test --release -p rexx-exec --test corpus` | 0 | **505 of 505** |

The pre-existing set, unchanged, and the passed counts are identical to Step 1's gate -- this step
added no tests, which is honest rather than an omission: its evidence is seven differentials, and
corpus witnesses need the harness sidecar of a later step.

**Seven cases, byte-identical on three descriptors:** the callee's whole view (`PARSE SOURCE` =
`LINUX SUBROUTINE <resolved absolute path>`, arguments, `digits` 9 under a caller at 5, **ADDRESS
inherited** and TRACE not, `.local` shared, caller's settings untouched after); `RETURN`/`EXIT`
with and without a value and what each leaves in `RESULT`; the public/private merge; **the file
re-read on every call**; 3.1 naming the resolved path for a resolved-but-unreadable file;
`FUNCTION` against `SUBROUTINE` including through `INTERPRET`; and a nested callee finding its own
neighbour. Only the re-read case discriminates the no-caching ruling -- the other six pass either
way.

**Three rulings, each made against the types and two against the survey's wording.** Q1 said reuse
`Resolved::Routine`; `InstalledRoutine` names a *directive* and a file's main body is not one, so
the file follows the `Resolved::Library` precedent instead and returns early before the `Entered`
match -- whose own comment is the argument for it word for word. The resolved kind carries **no
path**, forced by `CallSite(Cell<Option<Resolved>>)` requiring `Copy` and by `resolve_call` being
`&self`; widening it to `&mut self` does not compile, because all four `ir/drive.rs` sites hold a
`self`-derived `chunk` borrow across the call -- and the three sites that *would* have permitted it
are exactly the ones I would have checked had I sampled. And `run_loaded` gained two parameters,
not one: there was no seam for the inherited ADDRESS, and it passed `None` for caller settings
where `::OPTIONS NUMERIC INHERIT` needs `Some`.

### Two pre-existing divergences on the parse-error path, found on the way to Step 3

Neither is this task's doing; both were found by reading and then **measured**.

`interpret "do forever then"` prints `found "&1"` where the oracle prints `found "THEN"` --
`From<&ParseError>` passes `Vec::new()`, `ParseError` carries no substitution, and `substitute`
renders an unsupplied placeholder as the literal characters. The same run shows a second gap: the
oracle echoes the fragment's own failing clause above the `INTERPRET` line and this crate does not.

**The cause of the second was going to be an explanation and is instead a measurement.** A
fragment that fails at **runtime** (`interpret "say 1/0"`) echoes both clauses byte-identically,
rc 214, stderr identical -- so the echo path works wherever a clause exists and the gap is
parse-time only. That is also why `interpret_error_echo.rex` passes: its fragment fails at runtime.

A top-level parse failure is separately `rexx-exec: 35.1: Invalid expression.` at rc 120 against
the oracle's full two-line report at rc 221 -- wider than a missing substitution, and driven
through `rexxc` by `corpus/errors/parse-errors.tsv`, which is presumably why it has not surfaced.

**A scope figure I had to walk back before using it.** A scan found 121 distinct `(code, sub)`
pairs in `rexx-parse`, 56 needing substitutions -- but 27.901, the error I had just measured, is
absent from it, because it is raised as `loop_conditional((27, 901))` and my pattern only saw
literal pairs. Two further shapes evade it (`expr(Terminators::EOC, 1)`, `block_error(kind)`). The
honest figure is **at least 56 pairs, over at least 121, across at most ~230 sites**, exact count
from the compiler. Recorded in the plan that way, and Step 3 is split into 3a (the substitution)
and 3b (the attribution) because of it.

### Task 17 Step 4, first two pieces: `05fb858cb`

"Load a package from source, and search Package~new globally". `loadPackage(name, array)` builds
the package, runs its prologue and merges; `Package~new(name)` searches the global context and so
stops finding files beside its caller that the oracle does not. Both differentially verified:
t13 §2.3 and t08 §2.3 byte-identical on both descriptors, where before the commit one was a loud
refusal and the other answered rc 0 against the oracle's 43.901.

| gate | status | figures |
|---|---|---|
| G1 `fmt --all --check` | 0 | |
| G2 `clippy --workspace --all-targets -D warnings` | 0 | |
| G3 `test --release --workspace` | 101 | 2339 passed / **8** failed |
| G4 `REXX_CORPUS_GATE=1 memcap 8G test --workspace` | 101 | 2338 passed / **10** failed, **505 of 505** |
| G5 `REXX_CORPUS_GATE=1 test --release -p rexx-exec --test corpus` | 0 | **505 of 505** |

**One of those failures was new and mine**, and I have to name how it got in.
`a_reached_row_carries_its_own_site_identifier_and_an_unreached_one_does_not` panicked with
`package_from_source: no constructor to derive identifiers from`: the commit deleted
`Loud::package_from_source` when its last caller went away and left its row in
`corpus/refusal-sites.tsv`. Removing the row fixes it -- `refusal_sites` goes from two failures to
one -- and the row was correct to remove on its own terms, since it named a constructor that no
longer exists.

**The process lapse, stated plainly.** Before committing I ran `--lib`, `owners` and `loud`. Earlier
in *this same session* I wrote, after this happened twice: "the instrument is a whole-workspace run
before committing, not a longer hand-picked list." I diagnosed it, recorded it, and then hand-picked
again. Third occurrence.

**And a hazard that made it easier to miss.** `refusal_sites` was already red -- its sibling
`the_table_holds_every_constructor_the_source_defines` has failed all session -- so the binary sat
in my "known pre-existing" set and I skipped it. A permanently-red binary hides new failures inside
it: the count went 1 -> 2 and nothing in my routine looked at the count.

### `corpus/refusal-sites.tsv` is wholly stale, and it is not Task 17's to fix

Measured while fixing the above. The table is a `file:line` census of every `Loud` and `Raised`
constructor, and the disagreement is total: **~230 rows in the source and not the table, ~218 in
the table and not the source**, against 219 rows. That is every row, and the cause is the
`definition` field carrying line numbers that shift whenever anything is inserted above them.

- It predates this session by a long way: the TSV was last touched by the stream-builtin commits
  (`87c3a6ef9`, `8486514a4`), and `refusal_sites.rs` last changed at `8486514a4` -- many commits
  before Task 14.
- **There is no re-derivation writer.** The test says "re-derive it rather than editing the
  disagreeing row", and the only occurrences of write/env/regenerate in the file are that sentence
  and the module doc. The procedure it names does not exist as a command.
- My own contribution is real but small: `Loud::redirection`, added in Task 16, has no row. Adding
  it by hand would both violate the test's instruction and change nothing, since the table is stale
  in ~219 other places.

**Ruling: defer.** Regenerating means writing the generator the file assumes, rewriting the whole
table, and taking ownership of a red that long predates Phase 7 -- orthogonal to external routines
and exactly the "someone else's redness in my commit" trap. Recorded here so the next reader finds
a decision rather than a gap, and so the `redirection` omission is attributed rather than absorbed.

### Task 17 Step 4, third piece: `6bf69753b`

"Resolve a newFile executable's routines through the package that built it". `package_parents`
records the creator, `new_file_executable` writes it **before** `install_directives` (a
`::REQUIRES` prologue running there resolves routines and must already reach the parent, the same
ordering `load_requires` documents for its cache), and `installed_routine` walks it after the
package's own table and its imports.

| gate | status | figures |
|---|---|---|
| G1 `fmt --all --check` | 0 | |
| G2 `clippy --workspace --all-targets -D warnings` | 0 | |
| G3 `test --release --workspace` | 101 | 2340 passed / 7 failed |
| G4 `REXX_CORPUS_GATE=1 memcap 8G test --workspace` | 101 | 2339 passed / 9 failed, **505 of 505** |
| G5 `REXX_CORPUS_GATE=1 test --release -p rexx-exec --test corpus` | 0 | **505 of 505** |

The pre-existing set exactly, and `a_reached_row_carries_its_own_site_identifier...` is gone --
`0c9c22384` holds.

**Measured with a control that could have failed:** a `newFile` executable calling its creator's
`::ROUTINE` answers `from main` on both sides where this crate raised 43.1; and the same shape
calling a routine defined nowhere is still `43.1 Could not find routine "NOHELPER"` on both sides,
so the chain did not turn a genuine miss into a hit.

**The pre-commit check finally ran before the commit rather than after.** Whole workspace, 2341
passed / 7 failed, the pre-existing set. Three earlier commits this session were hand-picked and
the gate caught two of them; this is the correction actually applied.

### The in-memory package's directory: diagnosed, and the first fix I wrote down was wrong

`findProgram` on an in-memory package answers a path where the oracle answers `.NIL` (§2.4 t13),
and the parent-chain commit left it exactly as it was.

**Mechanism, read end to end:** `find_program` -> `resolve_program_name`, which is
`package.map(|p| self.package_path(p))` fed to `resolve_search`; `package_path` falls back to
`&self.program_path` when `required_paths` has no entry; and `package_from_source` never inserts
one. So an in-memory package silently borrows the **main program's** directory.

**A discriminating probe, because the measured case could not tell two readings apart.** With
`pk2.rex` present in *both* the program's directory and the current one, the oracle answers the
**cwd** copy and this crate answers the **main** copy. So an in-memory package *does* search, it
searches **globally**, and t13's `.NIL` was a global search finding nothing rather than an absence
of searching. Passing `None` is right; "search nothing" would have been wrong, and the earlier
evidence was consistent with both.

**And the discriminator I first wrote down was wrong.** The note said "pass `None` where
`required_paths` has no entry" -- but the *main* program's package has no entry either and
legitimately resolves through `program_path`; t13's own first line shows both interpreters
agreeing there. The test has to be "built in memory", which `package_from_source` marks with
`compiled_method_names`. Fixing one divergence by that first rule would have created another.

**Where the fix must not go:** `package_path` has eight callers -- `PARSE SOURCE`'s third word,
`~package~name`, the traceback, the circular-`::REQUIRES` check, `resolve_requires` -- all relying
on the fallback naming the running program.

### The fix landed: `a570da543`

"Give a package compiled from source no directory of its own". `resolve_program_name` skips the
package path for a source-compiled package, so it searches the global context.
`compiled_method_names` is the test, and its three writers are exactly the source-text
compilations -- `package_from_source`, `compile_method_source`, `compile_routine_source` -- while
`new_file_executable` records a path instead because it came from a file.

| gate | status | figures |
|---|---|---|
| G1 `fmt --all --check` | 0 | |
| G2 `clippy --workspace --all-targets -D warnings` | 0 | |
| G3 `test --release --workspace` | 101 | 2340 passed / 7 failed |
| G4 `REXX_CORPUS_GATE=1 memcap 8G test --workspace` | 101 | 2339 passed / 9 failed, **505 of 505** |
| G5 `REXX_CORPUS_GATE=1 test --release -p rexx-exec --test corpus` | 0 | **505 of 505** |

The pre-existing set exactly. Whole workspace **before** committing: 2341 / 7 -- the correction
applied for the second commit running.

**Three measured cases, one of them the control that made the fix meaningful:** the same file name
in both directories now resolves to the current one on both sides; the file only beside the
program resolves to `The NIL object` on both sides; and `.context~package~findProgram` from the
**main program's** own package still answers its neighbour -- the case my first discriminator
would have broken.

### Task 17 Step 4, explicit context: three probes, two of which corrected me

- **The target is confirmed.** `.Routine~newFile(path, ctx)~call` with an in-memory package as the
  context answers `from c2` at rc 0, where this crate refuses at rc 120.
- **The accepted set is wider than the code believes, and the error is a different number.** The
  oracle rejects a bad context with `40.904: NEWFILE argument 2 must be one of "PROGRAMSCOPE",
  Method, Routine, or Package object`. So a **string `"PROGRAMSCOPE"`** is accepted, which nothing
  here knows about, and `newFile` rejects with **40.904** where `package_new`'s own third-argument
  check raises 93.953. Two different errors for what I had been treating as one concept.
- **A fourth argument to `Package~new` is an arity error**, `93.902: Too many arguments in
  invocation of method; 0 expected.` at rc 163 -- not the `executable_context` refusal sitting
  there now. Carrying that refusal forward would have shipped the wrong message.
- **My `Native`-as-context probe was malformed** and that question is still open:
  `.Array~instanceMethod('ITEMS')` answered `.NIL`, so no native-sourced Method was ever
  constructed. Re-probing with the candidate printed before it is used.

### The context argument, measured rather than assumed

The second probe was malformed too -- `.Array~method('OF')` is 97.1, because `OF` is a class
method and `~method` searches the instance side, and one bad line killed the whole file. Re-run one
probe per file with `APPEND`:

- **A native method's package is the REXX package.** `.Array~method('APPEND')~package` says so, and
  as a `newFile` context it answers 43.1 for a user routine. That is `Package::Rexx`, which the
  enum already has -- the case needed no new concept.
- **`"PROGRAMSCOPE"` is caseless and means the caller's own package**, the same answer an absent
  argument gives. A Routine object and a Package object each redirect to their own package.
- **40.904's `found` is the string *conversion*, not the string value**: `.array~of(1)` renders
  `1` (an Array's `makeString` joins its lines), `.NIL` renders `The NIL object`, `.Array`
  renders `The Array class`.
- **The two surfaces disagree on both the accepted set and the error.** `Package~new` refuses
  `"PROGRAMSCOPE"` with 93.953 where `newFile` takes it, and its context redirects resolution for
  the **source** form only: with the package file actually found, a file-form package with a
  context that defines the routine its prologue calls still raises 43.1, exactly as without one.
- **The defaults are opposite.** `newFile` with no context resolves through the caller; a source
  package with no context has no parent at all and raises 43.1 for the caller's own routine.
- **A fourth argument is `Object~init`'s, not this function's.** The package is built and its
  prologue *runs* -- oracle stdout carries the prologue's line -- and only then is it
  `93.902 ... 0 expected.` under a `Compiled method "INIT" with scope "Object".` frame. The first
  implementation raised it early and lost that stdout line; the fix is to send `INIT` after
  building, which is what every other constructor here already does.
- Context validation precedes the arity error: an invalid third argument *and* a fourth is the
  93.953.

**Ruling: the package-prologue traceback gap is pre-existing and is not this task's to close.**
An error escaping a package prologue loses the `Compiled method "NEW"` frame and the calling
clause. It is not general to nested runs -- a failing `::REQUIRES` prologue is byte-identical --
so I checked it against the parent commit's own binary, built from `git archive` into its own
target directory rather than reasoned about: `a570da543` produces the same short traceback for the
no-context cases the oracle disagrees with. My change moves the file-form-with-context case out of
a loud refusal and into that same pre-existing gap. Recorded, not claimed fixed.

**Ruling: `dispatch.rs`'s `native_package_new` is unreachable** for `.Package~new` -- the class
loop registers `package::NATIVE_CLASS_METHODS` after the instance table, so `package_new` wins,
which the 93.953 both binaries answer confirms. Pre-existing, left alone, noted so the next reader
does not read it as the live implementation.


### Task 17 Step 4 explicit context landed: `1a6a37262`

"Take the context argument newFile and Package~new are given". The three
`executable_context` refusals are gone and `Loud::executable_context` with them,
so no refusal in this area names Phase 7.

**What the single refusal had been hiding, each measured on three descriptors:**

| question | `newFile` | `Package~new` |
|---|---|---|
| `"PROGRAMSCOPE"` | accepted, caseless | 93.953 |
| bad value | 40.904, `found` is the string *conversion* | 93.953 |
| no context | the caller's package | no parent at all |
| file form | n/a | never consumes the argument |
| surplus argument | 93.902 `2 expected` from `Arity::Fixed(2)` | 93.902 `0 expected` from `Object~init` |

**Two findings that only running produced.**

The fourth argument is `Object~init`'s, and the package is *built and its
prologue run* before it is refused -- the oracle prints the prologue's line
first. Raising it early passed every check I had until a probe compared stdout.
Sending `INIT` the unconsumed arguments gives the line, the frame and the count
at once.

The file form does not ignore a context, it never consumes it, so the value
lands in `INIT`. My first reading was "ignored", taken from a pair where the
load raised first and the context could not have shown. A file that loads
cleanly separates them, and the answer is order-independent.

**A defect the witness found that no probe would have.** `new_file_executable`
read relative names against the *process* directory, not
`Invocation::with_directory`. Only a caller that sets one can see it, and the
corpus harness is the only such caller here, so it appeared as the witness
failing inside the harness while the identical program passed by hand. Fixing
it by normalising the recorded path then broke `~package~name`, which answers
the spelling as given -- resolved for the read, recorded as asked.

**Witness:** `corpus/lang/executable_context.rex`, trapping every error so the
pre-existing prologue traceback gap stays out of it, and printing no absolute
path. Corpus **506 of 506**.

**Owed, unchanged:** `corpus/refusal-sites.tsv`'s wholesale re-derivation. Its
test is red at the parent commit with the same failure, and the row this commit
removes appears nowhere in that failure.

**Gate for `1a6a37262`, tree frozen from commit to finish:**

| gate | exit | figures |
|---|---|---|
| G1 `fmt --all --check` | 0 | |
| G2 `clippy --workspace --all-targets -D warnings` | 0 | |
| G3 `test --release --workspace` | 101 | 2340 passed / 7 failed |
| G4 `REXX_CORPUS_GATE=1 memcap 8G test --workspace` | 101 | 2339 / 9, **506 of 506** |
| G5 `REXX_CORPUS_GATE=1 test --release -p rexx-exec --test corpus` | 0 | **506 of 506** |

Both failing *sets* enumerated rather than counted, and both are the
pre-existing ones exactly: G3's seven, and G4's those plus
`concept_and_class_gate_table` and `directive_option_gate_table`.


### Task 17 Steps 5-7 landed: `d8f8d539a`

"Give a corpus program a fixture directory and a search environment". `<name>.d/` is copied into
the run directory before each side runs and `<name>.env` gives `PATH=`, `REXX_PATH=` and a `CWD=`
subdirectory, `{run}` spelled as the run directory. The environment is laid over the inherited one,
because the oracle's overrides sit on a process that inherits everything else.

**The witness could not fail, and the control is what found that.** `external_search_order.rex`
reads the order off the answers -- current directory, then `REXX_PATH`, then `PATH`, with 43.1 for
a name none of them has. But the differential compares the two interpreters, so deleting the
fixtures leaves both failing alike: the gate stayed at 507 of 507 with the `.env` removed and again
with the `.d/` removed. `a_sidecar_changes_what_the_oracle_answers` runs the oracle with and
without the sidecar and requires the two to differ -- verified by defeating it in both directions
and restoring.

`phase-4c.txt`'s rule kept its scope and lost a false reason ("this crate raises 43.1 instead").

| gate | exit | figures |
|---|---|---|
| G1 / G2 | 0 / 0 | |
| G3 `test --release --workspace` | 101 | 2342 passed / 7 failed |
| G4 `REXX_CORPUS_GATE=1 memcap 8G test --workspace` | 101 | 2341 / 9, **507 of 507** |
| G5 corpus | 0 | **507 of 507** |

Both failing sets enumerated, both the pre-existing ones.

### Task 18 measured before writing anything

Every figure re-measured against the oracle rather than read from survey F, and every prediction
confirmed: `SysFileExists` counts a directory and `SysIsFile` does not; `SysFileDelete` answers
**13** for a missing file, because the `access(W_OK)` pre-check collapses every failure to
`EACCES`, where `SysRmDir` answers the raw `remove(2)` errno (`2` missing, `39` non-empty) and
`SysMkDir` the raw `mkdir(2)` one (`17`); `SysSleep` is 88.907 below zero and 88.902 for a
non-number; `SysVersion` and `SysLinVer` answer the same two words.

Arity errors are 88.901 and 88.922 at rc 168 under a `Compiled routine "NAME".` frame, and their
major line reads `running REXX:` where `SysSleep`'s own validation reads
`running <program> line N:`.

**Two `SysFileTree` findings that decide the port's shape**, neither in the survey: `*` does not
cross `/`, and the directory part of a spec is **literal** -- `su*/deep.txt`, `*/deep.txt` and
`su?/deep.txt` all match nothing where `sub/deep.txt` matches. So the spec splits at its last `/`
into a directory to walk and one pattern for the final component. Bracket grammar confirmed for
`[abc]`, `[!abc]`, `[a-z]`, `[A-Z]` and `?`; `S` descends after finishing a level; readdir order is
not alphabetical, so a witness sorts and the dated `F`/`FL`/`FT` formats cannot appear in one.


### Task 18 landed: `e0a46cc36` and `cd399f502`

**`e0a46cc36`, a prerequisite found by probing rather than planning.** `DIRECTORY`, `FILESPEC` and
`BEEP` were implemented but their argument errors carried no `Compiled routine "NAME".` line and
named the running program where the oracle names `REXX`. Pre-existing, checked against the parent
commit's binary. The frame and the attribution are two rules: every error escaping such a routine
gets the frame, and only the argument errors also name the package and drop the line -- a body's
own 88.902 keeps the program and its line. `Delivery` took a flag, not the name, because an
`Option<&'static str>` grew `Raised` to 136 bytes and tripped `clippy::result_large_err`.

**`cd399f502`, the nine routines.** The three deletions follow three rules: `SysFileDelete`'s
`access(W_OK)` pre-check reports its own failure so a missing name is `13`; `SysRmDir` is
`remove(2)` and deletes a plain file, answering `0` where `fs::remove_dir` would answer `ENOTDIR`;
`SysMkDir` is `mkdir(2)` and does not range-check its mode. `SysFileTree` globs its last component
only, matcher hand-written rather than a dependency, and its date layouts match to the byte.

**Stated, not hidden:** `SysFileTree`'s two attribute-mask arguments are accepted and not applied.

| gate for `cd399f502` | exit | figures |
|---|---|---|
| G1 / G2 | 0 / 0 | |
| G3 | 101 | 2342 passed / 7 failed |
| G4 | 101 | 2341 / 9, **509 of 509** |
| G5 | 0 | **509 of 509** |

Both failing sets enumerated; both pre-existing.

### Task 19 measured before writing

Every case in survey C §2.8 re-run against the oracle and reproduced. The banner is seven blanks
then `+++ "LINUX COMMAND <path>"`, once, before the first traced clause; the prompt is unindented
and once per activation. `""` continues, `=` re-executes the clause so it traces twice, anything
else runs untraced.

**The instruction and the builtin disagree, and the prompt disagrees with both.** In debug every
`TRACE` *instruction* is traced and ignored, so `trace()` keeps answering `?R`; the `TRACE`
*builtin* has no such guard and `call trace 'O'` ends debug from the program; and from the prompt
`trace ?` ends debug keeping `R` while a bare `trace` ends it at Normal.

`trace 2` from the prompt traces the skipped clauses without pausing where `trace -2` does not
trace them at all. A fragment's syntax error prints two `+++ Interactive trace.` lines, is
swallowed, and the pause continues at rc 0.

**Two constraints on the work:** the harness needs a `.stdin` sidecar, and the `.DEBUGINPUT`
redirection cases rest on monitor support the crate still refuses, so they stay recorded rather
than witnessed. `::OPTIONS TRACE ?<letter>` stays out of every probe -- `oracle-crashes.txt` entry
9 is an indefinite block, which gives no signal at all.

### Gate for `fe40a7977`, covering four commits

| gate | exit | figures |
|---|---|---|
| G1 / G2 | 0 / 0 | |
| G3 | 101 | 2341 passed / 8 failed |
| G4 | 101 | 2340 / 10, **511 of 511** |
| G5 | 0 | **511 of 511** |

Both failing sets enumerated. Every member is pre-existing except one, and that one is a
regression this gate caught: `the_exempt_set_matches_the_current_failures`, because
`base/keyword`'s `TRACE::test_trace_?o` started failing under Task 19.

**Ruling D-T1: `OFF` refuses the debug flag, and the exempt file stays at four rows.** Measured
against the C++ source rather than the oracle, which entry 9 forbids running for this: `TRACE
OFF` clears every flag including debug (`parseTraceSetting`, `TraceSetting.cpp:225`-`238`), and a
bare `?` toggle applies only while the setting in force is not `OFF`
(`RexxActivation::setTrace`, `RexxActivation.cpp:1010`-`1017`). `applied` honoured neither, so
`trace o` followed by `trace ?` entered debug -- and once there, every later `TRACE` instruction
was correctly ignored, so `trace n` did not take either. `trace.rs`'s `off_refuses_the_debug_flag_
from_either_side` asserts both halves with `?R` rows as the positive control, and inverting each
guard fails it. The body now passes: 892 of 896 where the gate measured 891.

`mode_from_setting`'s all-`?` fallback is deliberately outside the rule. What it stands in for is
`setDebugToggle`, not `setTraceOff`; its `OFF` letter is a stand-in this crate cannot measure,
because doing so means running `::OPTIONS TRACE ?` against the oracle.

## Task 20: the security manager's Phase 7 hooks

**Where a manager lives.** On the package, not on the executable: `MethodClass::setSecurityManager`
and `RoutineClass::setSecurityManager` are both `code->setSecurityManager`, and `RexxCode`'s
override writes `package->setSecurityManager` (`execution/RexxCode.cpp:245`). So
`Interp.security_managers` is keyed by `ProgramId` and all three setters write it; the effective
manager is the running activation's own package's, which is `getEffectiveSecurityManager`'s first
arm with no instance-level manager behind it.

**One send, two refusals.** `Interp::security_check` builds the info directory, sends the
checkpoint's message, and requires a result (91.999) that is exactly `0` or `1` (34.903). Measured
against the oracle on three descriptors: `code 91.999 / Message "CALL" did not return a result.`
and `code 34.903 / Authorization return value must be exactly "0" or "1"; found "yes".`, and the
same exit statuses.

**The checkpoints and where each sits.**

| message | site | measured |
|---|---|---|
| `COMMAND` | `Interp::checked_command`, after the redirection context and before the handler | `event COMMAND ADDRESS = sh COMMAND = echo command` |
| `STREAM` | `resolve_stream`, on a table miss, with the qualified name | replacement taken from `STREAM`, not `RESULT` |
| `CALL` | `Interp::call_checkpoint`, past the package's own `::ROUTINE`s | fires for `FILESPEC`, an internal-package routine |
| `REQUIRES` | `check_requires_access`, on the short name and again on the resolved one | two events per load, both before the cache |
| `LOCAL` / `ENVIRONMENT` | `env_seam::admit` under `Access::Resolve` | `.rs` and `.context` each ask both |
| `METHOD` | the dispatch seam, for a `PROTECTED` method | `NAME`, `OBJECT` and an `ARGUMENTS` array |

**Ruling D-S1: an unresolvable call name is `Resolved::Unresolved`, not an error from the
resolver.** `externalCall` asks the manager before it reports 43.1, and the checkpoint needs the
arguments, which are evaluated after resolution. `resolve_call` is `&self` and cannot run Rexx
code, so the raise moved to the consumer. Cost if wrong: 43.1's own path now has one more arm to
keep right, and `arguments_before_failure` no longer carries the not-found case.

**Ruling D-S2: only a name resolution passes the manager, not every directory read.** C++ puts the
`LOCAL` and `ENVIRONMENT` checks in `PackageClass::findClass` alone
(`classes/PackageClass.cpp:1134`-`:1158`); `ActivityManager::getLocalEnvironment` -- the output
routes -- and `ClassDirective`'s own search read the directories directly. `env_seam::admit` takes
an `Access` saying which, and a write never asks. Cost if wrong: a manager would see `SAY`'s route
lookup, which the oracle's does not.

**Both seam guards moved and neither was weakened.** `dispatch_seam.rs` and `environment_seam.rs`
bound their seam module to a listed set of items; each grew the enum the seam now answers with, and
the derive ban stands, so `Access` is not `Copy` and is passed by reference.

**Witnesses.** `corpus/lang/security_manager.rex` runs every checkpoint twice -- against a manager
that declines and one that answers -- with a `.d/` fixture tree, and
`corpus/lang/security_manager_result.rex` carries the two refusals. Qualified names are cut to
their last component in the witness, because the qualified form is the run directory's.

**Step 4's target is not met, and the blocker is not this task's.** `ootest/ooRexx/base/security.
manager/SecurityManager.testGroup` reaches `ooTest.frm`'s own framework load and stops at
`rexx-exec: environment symbol ".ENDOFLINE" is not implemented (Phase 5)`, an unbuilt
`.environment` entry. Measured with `REXX_PATH` naming `ootest` and `ootest/framework`. Task 22
Step 5 is where how far the suite gets is recorded.

**D-S1 came with a defect of its own, and the control is committed.** Making the resolver answer
`Resolved::Unresolved` put a miss into the compiled call-site table, which the raising version
could never reach: the same site would then never search again.
`corpus/lang/call_miss_not_cached.rex` raises 43.1 on its first pass, writes the file, and finds it
on the second; deleting the guard in `Calls::remember` makes the second pass raise as well. **It is
a control, not coverage**: run against the tree before this task it passes, because there the miss
was an `Err` nothing cached.

| gate for `bacd0bae5` | exit | figures |
|---|---|---|
| G1 / G2 | 0 / 0 | |
| G3 | 101 | 2343 passed / 7 failed |
| G4 | 101 | 2342 / 9, **514 of 514** |
| G5 | 0 | **514 of 514** |

Both failing sets enumerated and both are the pre-existing ones;
`the_exempt_set_matches_the_current_failures` is gone from them, which is the `64ca47832` fix
holding.

## Task 21: a prompt before the read

**The seam is an `Invocation` field, so no interpreter code writes to a descriptor.**
`Invocation::with_sinks` takes two `Box<dyn FnMut(&[u8]) + Send>` -- `Send`, because the
interpreter runs on a thread of its own for its stack -- and `rexx-run` is the only caller that
installs them. `Interp::hand_over_before_read` hands both buffers over and empties them, and does
nothing at all for an embedding that installed no sinks or for a source that cannot wait, so no
differential run's output moves: corpus stayed at 514 of 514.

**Three read entry points, one of which is the interesting one.** `input_line`, `input_bytes` and
`linein_line`'s direct arm. `PARSE PULL` reaches `input_line` through `.INPUT`'s own `LINEIN`, and
so does the interactive `TRACE` pause -- its prompt and the read that waits for the answer are the
same clause, which is the case the task exists for.

**`tests/prompt_before_read.rs` spawns the binary, because nothing else can see this.** The
differential harness supplies stdin as a buffer and compares at the end, where a prompt written at
exit and one written before the read compare equal. Each case reads until the prompt arrives with
nothing yet written to the child's stdin, then answers. Defeated by making the handover return
early: both cases then fail with `nothing arrived within 20s ... blocked on a read whose prompt has
not been written`, one per case, which is the whole mechanism.

## Task 22: close-out

**Step 4 was the substantive step, and it found work rather than labels.** Re-homing every refusal
that named Phase 7 turned up two defects and one stale claim: `.local['OUTPUT']` refused where
`.output` answered a monitor, because only the dot-variable path counted as a demand for the
minted bundle; and `instruction_owner` still owed `ADDRESS ... WITH` to Phase 7, which Task 14
delivered. `crates/rexx-exec/tests/closed_phases.rs` is the standing assertion, with a negative
control that finds an open phase by the same walk.

**Ruling D-C1: the scan covers Phase 7 alone.** Phase 5 closed leaving refusals that name itself --
an unimplemented `Directory~items` is one -- and widening the list would redden the tree for a debt
this phase never took on. The constant says so at its definition. Cost if wrong: a later phase
inherits a list it has to widen rather than one that was already right.

**Ruling D-C2: Phase 7 built `.ENDOFLINE`, which is Phase 5's row.** The ooTest framework's
prologue reads it and the suite stopped there, so measuring this phase's own exit needed it. It is
a one-line string constant, measured `0A`. Cost if wrong: a Phase 5 row was closed by another
phase and its record sits here rather than in Phase 5's.

**Step 2's criterion could not be met as written, and the instrument was the problem.**
`method-bodies.txt` sends to a receiver built by a committed expression; `Stream` and
`StreamSupplier` had none, a bare `~new` raises on both sides, and all 32 instance rows recorded
that the method was never sent. Giving each class a receiver that constructs -- `/dev/null` for
`Stream`, because it is writable and idempotent, and `/etc/hostname` for `StreamSupplier`, because
a supplier over a transient stream raises -- turned 11 rows red, and the two defects behind them
are in the gate document. Every `File`, `Stream` and `StreamSupplier` row now reads `answers` with
real evidence.

**Step 5: L2 is not reachable from inside this phase.** The chain is in the gate document, section
7: `.ENDOFLINE`, then `rxregexp.cls`, then `::METHOD INIT EXTERNAL "LIBRARY rxregexp RegExp_Init"`
-- a shared library, which is Phase 8's. The plan's row asks Phase 7 for a rung that Phase 8 owns.

**`RXTRACE=ON` was delivered rather than re-homed.** It was unimplemented *and* unrefused, so a
program under it got a wrong answer at rc 0, and interactive debug had just made it cheap. Three
corrections, each from a diff: the setting has to go through `setTrace`'s own route or the banner
never prints; the banner was owed by the IR's clause echo as well as the stepped one; and the
library bootstrap runs through the same entry point and traced itself at the user. The remainder --
a called external program does not print `>I>`/`<I<` -- is measured and recorded in the exclusions
file's TRACE ? row.
