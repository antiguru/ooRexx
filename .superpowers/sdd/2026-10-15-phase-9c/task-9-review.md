# Task 9 review: the restore surface

Range ebb5e8a42..2056d8812. Reviewed from the review package, plus probes run on a detached worktree at 2056d8812 (removed afterwards), built into `target-rev-9` (`Compiling rexx-exec` seen, release).

## Spec compliance

Spec compliant, with two Minor deviations from process (files outside the brief, and the "inside a line" truncation case).

* Step 2 routes: `rexx FILE` through `execute_as` (`lib.rs` hunk at :3832), `parse_file` (`run.rs` :3333) for `::REQUIRES`, external `CALL` and both `newFile`, `package_from_source` (`install.rs` :977) for `.Package~new`/`~loadPackage` arrays, `parse_source_lines` (`class_protocol.rs` :339) for `.Method~new`/`.Routine~new` arrays, `Surface::compile_buffer` for `NewRoutine`/`LoadPackageFromData`. Each one matches the oracle call sites listed in the brief.
* Step 3: the no-source property is `ProgramSource::traceable` (`rexx-parse/src/source.rs`), set only by `ProgramSource::compiled`. No side map. A grep of the added lines for `HashMap|HashSet|BTreeMap|.insert(` finds no new map. The two id-keyed inserts in the touched code (`compiled_method_names`, `required_paths`) existed before this task.
* Perf rule: plain source pays `compiled::restore`'s prefix check in `program_source`. `lines_source` compares `lines.get(1)` before the `#[cold]` join. `untraceable_code` is read only where a trace line is routed (`route_trace_line_as`), at the debug banner (after the `debug` short-circuit) and at a due pause. `restored`, `joined_source`, `compile_buffer_source`, `sourceless_line` and `unrestorable_outcome` are all `#[cold] #[inline(never)]`.
* Step 4: `tests/compiled.rs` exists, runs each side's `rexxc` and `rexx`, asserts rc 0 and the 29-byte prefix, and covers the string form of `.Package~new` (`tagline.rex`/`routes.rex`), Review Focus 5 (`sub/p.rxo`, `relative.rex`) and Review Focus 2.
* Step 5: Deviation 41 is added, the row is marked DELIVERED, and the `:5391` reference is corrected (exclusions hunks).

## Probes run (oracle vs crate, fresh dirs, each side compiling its own images, 1 run per side)

Programs are in `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/rev9probe/src/`. Transcripts are `.../rev9probe/oracle.txt` and `.../rev9probe/crate.txt`. `diff` shows one hunk, the `sfname.rex` line below.

* `obs.rxo`: `PARSE SOURCE`, `SOURCELINE()` 0, `trace i` and `trace r` print nothing, `.context~package~source~items` 0, a SYNTAX in an internal routine gives `position` 13 and a two-line `traceback` (`Program in "DIR/obs.rxo" (no source available).` at 13 and 11). The two sides are identical.
* Review Focus 5, not in the shape `tests/compiled.rs` uses: `call 'sub/p.rxo'` and `.Routine~newFile('sub/p.rxo')~call` from the parent dir, where `sub/p` has a trailing `::REQUIRES 'helper.rex'`, a bare `call ext` and a public routine from the helper. Both resolve from `sub/` and the two sides are identical.
* `~source`: a restored `.Routine~new` from encoded lines gives `~source~items` 0, `sourceSize` 0 and package name `R`. `.Method~new` gives 0. `newFile` gives 0. A `.Package~new` prologue failing shows `Routine PK in package "PK" (no source available).` The two sides are identical.
* A `.rxo` given to `rexxc`, binary and encoded: 13.1 at the `@`, rc 243, no output. The two sides are identical.
* Damaged images, crate only (`mk.py`): truncated at header byte 10, at 44, at the end of the header, source minus 1, source halved, encoded body +6 (inside a quartet), +8 (inside a line, on a quartet boundary), second line +5 and +8, and one trailing byte appended. All give 3.903, rc 253, no panic. A mid-stream `QQ==` gives 3.905. CRLF line ends, and the encoded tag with nothing after it, are source (13.1). The oracle's `length > encodedHeaderLength` and its tag `memcmp` (`ProgramMetaData.cpp:333`, `:57`) read the same way. A `#!` line with no LF is source, rc 0. A trailing partial quartet decodes on the oracle too (`StringUtil.cpp:1760-1850` returns true), so 3.903 rather than 3.905 there is right.
* Mutation: `install.rs:987`, `lines_source(...)` changed to `lines_source(...).filter(|_| false)`. `each_route_runs_a_restored_program_as_the_oracles_does` FAILED on `routes.rex`, exit 101. Reversed by exact edit, and `git diff --stat` was empty.

## Controller questions

### 1. The routine name under `~call`

* Does the oracle's image store the name? Yes, implicitly. `RoutineClass::callRexx` passes `executableName` (`classes/RoutineClass.cpp:209`, also `:227` and `:248`), and `executableName` is part of the flattened routine (`flattenRef(executableName)`, `:148`). `ProgramMetaData::restore` resets only the package's name (`ProgramMetaData.cpp:190-191`, `setProgramName`). The routine keeps the name `rexxc` gave it, which is the full source path (`routine-object/oracle.txt`: `Routine DIR/fail.rex`).
* Would a name field in our format fix it? Only the image half. The crate does not name a `~call` activation by the executable's name at all, so a stored name would be read by nothing.
* Is `CALL` pre-existing for source routines? Yes, measured (`sfname.rex`). The oracle gives `ROUTINE MYNAME` for `.Routine~new('MYNAME', ...)~call` and `ROUTINE sf.rex` for `.Routine~newFile('sf.rex')~call`. The crate gives `ROUTINE CALL` for both. For source routines it is invisible in a traceback, because the clause is shown, but `StackFrame~name` exposes it.
* Recommendation: **(c), queue both, as one item**, with the `CALL` naming first. The image half alone changes no observable, so (b) would ship a format bump with no reader. Fixing the naming is a change to routine-object activation naming that reaches source routines, which is outside a restore task. Record the restored-routine traceback line as a known divergence until then. It is not in `tests/compiled.rs` now, which is correct, since the test asserts identity.

### 2. A restored executable takes no parent package, even with a context

I re-measured with two cases. `.Routine~new('X', <encoded lines>, .context~package)~call` and `.Routine~newFile('re.rxo', .context~package)~call`, where the routine calls the caller's `::ROUTINE mine`, give 43.1 on both sides. The control is the same routine from source with the context (`ctx.rex`), which gives `mine reached` on both. This matches `LanguageParser::createRoutine(name, array, ctx)`/`(name, ctx)` (`LanguageParser.cpp:195-245`), where `restore` never sees `sourceContext`. The ruling is confirmed.

### 3. `NewMethod` never restores

Confirmed against `createMethod(name, buffer)` per the brief. The implementation is `surface.rs` `compile_buffer`'s `BufferKind::Method` arm. It has one clarity issue (Minor 3 below).

### 4. Files outside the brief

* The trace and traceback edits (`trace.rs`, `environment/route.rs`, `run/interpret.rs`, `dispatch/context.rs`, `run.rs` `sourceless_site`/`sourceless_line`, `install.rs` `directive_text`, `error.rs`) are needed. Step 3 requires the Step 1 observables, which include `trace r`/`trace i` silence, the `TRACE ?R` pause gate and the sourceless traceback lines, and the brief's file list could not deliver them. The Global Constraint says to report and let the controller move such an item, not widen in place. The implementer widened, then reported. That is a process Minor, not a code defect.
* The `RECORDED` removal in `api_group_tests.rs` is correct. The file's contract (`api_group_tests.rs:16-17`) is that each run-entry of `RECORDED` must not pass on this crate, so a delivered test has to leave it.

## Strengths

* One property, on the program's own source object, read at each observable. No side map, and no new id key.
* The restore detection reads exactly like `processRestoreData`, including the equal-length and CRLF edges, which I probed.
* `tests/compiled.rs` compares against the live oracle on each side's own images and goes red on a route mutation. Deviation 41 is measured and recorded with its oracle transcript.

## Issues

### Critical

None.

### Important

None.

### Minor

1. `rust/crates/rexx-exec/src/dispatch/library/surface.rs:808-811`: the insertion orphaned `held`'s doc comment. "The value `result` carries, or `None` with its condition held..." now heads `compile_buffer_source`'s doc, and `fn held` (:839) has none. Move the two lines back above `held`.
2. `rust/crates/rexx-exec/tests/compiled.rs`, `a_foreign_or_damaged_image_is_refused`: `line.rxo` (`body + 73`) cuts at a line end ("a whole line cut away"), not inside a line. `quartet.rxo` is inside both. The case of inside a line on a quartet boundary (`body + 8`) is unpinned. Probed: 3.903, so it is coverage only.
3. `surface.rs:66-71`: `crate::compiled::lines_source(&borrowed, name)?.ok();` is a statement kept only for its `?`, and it decodes and discards a whole image. A plain `if lines.get(1) != ENCODED_LINE { return None }`-style test, or a named helper, would say what it does. Clarity only.
4. `rust/crates/rexx-exec/src/error.rs` (new `sourceless_routine_line` doc): "The same for a routine activation -- `formatSourcelessTraceLine`'s ..." is new prose with ` -- `, which the Global Constraints forbid.
5. Process: the task widened into files outside its list (question 4) instead of reporting first, and two builds wrote to `rust/target` (reported). Neither is a code defect.
6. Known divergence, queued per the recommendation above. A restored routine run by `~call` shows `Routine CALL` where the oracle shows the `rexxc`-time path. It is pre-existing via `StackFrame~name` for source routines. Record it in the queue and in the exclusions file.

## Assessment

**Task quality:** Approved.

**Reasoning:** every route restores where the oracle does, and the observables match the oracle on the shapes I probed beyond the test. The remaining items are a doc orphan, a prose dash, one unpinned truncation offset and a queued pre-existing naming divergence.
