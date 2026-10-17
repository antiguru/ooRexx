# Task 9 re-review, fix round 1 (240508580)

## Finding verdicts

1. `held` doc orphan: ADDRESSED. `surface.rs` `fn held` has its doc again, and `compile_buffer_source` has its own two-line doc.
2. `lines_source(...)?.ok()` decode-and-discard: ADDRESSED. `surface.rs:611-618` tests `compiled::encoded_lines` directly, and `lines_source` calls the same helper (`compiled.rs`), so the test has one definition. No image is decoded and thrown away.
3. ` -- ` in `error.rs` doc: ADDRESSED (colon now).
4. Pin for an image cut inside a line: ADDRESSED. `tests/compiled.rs` adds `boundary.rxo` (`body + 8`), expected 903. `cargo test --release -p rexx-exec --test compiled`: 2 passed, run here. The implementer's mutation claim (red on `boundary.rxo`, green on the other two) was not re-run, and it reads as plausible from `decode`.
5. `CALL` naming queued: ADDRESSED. `.superpowers/sdd/queued/2026-10-17-routine-call-frame-name.md` carries the witness, the missing `METHOD CALL` frame, the restored-routine traceback, the oracle sites (`RoutineClass.cpp:209`, `:148`, `ProgramMetaData.cpp:190-191`) and the format consequence. `phase-4-exclusions.txt` has the KNOWN GAP row with `OWNER: none.` after Deviation 41.

## The behaviour shift

Old: second line is the tag, and `restore(joined)` is not source, then parse as source directly (result discarded, lines parsed). Second line is the tag, joined text is source (first line neither `#!` nor a tag), then `lines_source` is `None`, the arm returned `None` and the caller sent `.Method~new(name, array)`. New: second line is the tag always parses directly.

* Oracle: `createMethod(name, buffer)` (`LanguageParser.cpp:134`) never checks the tag and builds a `BufferProgramSource`, no context. The direct path is the oracle's shape. The old send path went through `createMethod(name, array, ctx)`, which restores only when `MethodClass::restore` succeeds and otherwise parses as source, so the same method comes out.
* One real difference, in the oracle's favour: the send path's `executable_parent` (`class_protocol.rs:361`) gives a source method the caller's package where no parent is passed, while the direct path passes `None`, as the oracle's buffer overload has no context. So for this narrow input `NewMethod` stops inheriting the caller's package. Read from the code, not measured with a native witness.
* `cargo test --release -p rexx-api`: all suites ok (69, 14, 6, 16, 24, 11, 83, 1, 4 passed). No test pins this input, so the shift is untested either way.

## New breakage in the fix diff

None. Minor, no action needed: the `compile_buffer` trait doc (`rexx-api/src/callbacks.rs:224-226`) says a `Method` "answers the method its lines parse to as source" for an encoded image, which is true only for the second-line-tag case. The implementation returns `None` for every other `Method` input.

## Out-of-scope observations

* For ordinary source, `NewMethod` still falls back to the `.Method~new` send (`compile_buffer` returns `None` when the second line is not the tag), so it takes the caller's package as parent where the oracle's `createMethod(name, buffer)` takes none. Pre-existing, not touched by this fix, and it now differs from the tagged-lines path. Worth a witness in the final review.

## Verdict

**Fix round:** All findings addressed, no new Critical/Important breakage.
