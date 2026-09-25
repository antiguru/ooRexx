# Surface Task 3, fix round 2

The re-review of fix round 1 is APPROVED on behaviour: every finding 1-7 is
closed, and the reviewer re-measured them independently (3851 generated inputs
against real `sscanf` for the `(nil)` rule, a counting `DEFAULTNAME` across four
shapes for findings 3 and 4, `collect_stress` at head for the rooting, byte
comparisons against the oracle over sixteen receiver shapes for the guard). It
requested changes on prose only, plus one duplicated rule. Its full report is
`.superpowers/sdd/2026-09-14-phase-8-surface/task-3-fix1-rereview.md`; read the
"New findings" section for the exact text it quotes.

Nothing below changes an observable. Do not change one. If you believe a fix
below would, stop and say so instead of making it.

## 1. `rust/crates/rexx-exec/src/dispatch/library.rs:247-249`

The comment heading the refusal match says `found` is the argument's
`stringValue()` and not its string conversion, using an array as the example
that shows the difference. `Refused::NotLogical` at `:271` is the arm where the
opposite holds: a logical's `found` is the conversion, and for an array it is
`"1\n2"`. The corpus program's own header already carries that exception; the
comment does not. Correct it. Keep it minimal.

## 2. `docs/superpowers/plans/phase-4-exclusions.txt:5099`

"The three in-crate conversion-row witnesses" is wrong twice: they are corpus
programs, not in-crate, and a bare count stands in for the set the same edit
stopped naming. Name them: `lang/library_native_integer_arguments.rex`,
`lang/library_native_object_arguments.rex`,
`lang/library_native_special_arguments.rex`. The claim itself is true.

## 3. `rust/corpus/phase-8.txt:205` and `:210-211`

The comment splits the conversion rows into "rows a shipped extension declares"
(the corpus's) and "rows nothing shipped declares" (`rexx-api/tests/values.rs`'s)
and that split is false: `orxmethod` declares `TestMutableBufferLength` and
`TestVariableReferenceValue`, whose conversion is unit-test-only because their
success side aborts. Rewrite it so a reader can derive the true split, or state
the narrower true thing. This is the same class of claim as the "every
conversion row" this round already fixed.

## 4. `rust/crates/rexx-exec/src/lib.rs:5694`

"... once the queue has handed it over, and nothing else does." A negative
extent claim over holders nothing in the tree can enumerate. The walk is already
justified by its control. Drop the clause.

## 5. One rule, two copies (the only code change in this round)

`native_found` (`dispatch/library.rs:627-631`) decides "has its own
`stringValue`" as `state.buffer().is_none() && state.pointer().is_none()`.
`Interp::redirect_of` (`value.rs:1004-1007`) decides the same thing as
`state.buffer().is_some() || state.pointer().is_some()`. Neither matches
`NativeState` exhaustively, so a new variant would be classified by fallthrough
at both sites, and a reader grepping for one would not find the other.

Give `NativeState` one inherent method that answers this question by an
**exhaustive** match over its variants, so that adding a variant is a compile
error rather than a silent classification, and call it from both sites. Name it
for what it answers, not for how the callers use it. The behaviour must not
change: prove it by running the two witnesses the reviewer used
(`library_native_object_arguments.rex` and the `values.rs` unit cases) before
and after, and say in the report that they were byte-identical.

## 6. `docs/superpowers/plans/phase-4-exclusions.txt:5089-5091`

The stream-name root is recorded as witnessed by the `run/tests.rs` unit test,
with the withdrawn corpus witness explained beside it. The reviewer measured
that `collect_stress`'s L0 test witnesses it too, and more loudly: removing the
root turns exactly five L0 programs rc 120. Record that, so a reader weighing
the unit test knows it is not the only thing behind that `push_temp`. Verify the
"five" yourself before writing it, or write it without the number.

## 7. `rust/crates/rexx-exec/src/value.rs:999-1004`

Two comments now head the same match arm: "A buffer's own body holds the text,
named or not." and, directly below it, "A buffer's own body holds the text; a
stream's does not ...". The first is the superseded one. Check which of the two
the current arm actually justifies, delete the stale one, and check the diff for
any other arm that picked up a second head the same way.

## 8. One correction to your own report

Your report describes `gate_table_c`'s 82 rows as "File/Alarm/Ticker/
StreamSupplier/Stream instance rows". The reviewer counted them: File instance
50, Stream instance 24, StreamSupplier instance 8, all phase 7. Alarm (7) and
Ticker (6) are phase 6 and are not gated. Correct that sentence in the report.
Do not investigate the failure itself; that is dispatched separately.

## Constraints

Unchanged from the task: no `unsafe` outside `rexx-api/src/ffi.rs` and
`src/load.rs`; no process-global state; comments minimal, no em-dashes, and a
comment may never state the size of a set; the C++ tree, `samples/`, `build/`,
`ootest/`, `oodocs/`, `testbinaries/` and `api/` are read-only.

Commit with a message written to a file and passed with `-F`, staging explicit
paths. Before committing: `cargo fmt --all --check`, `cargo clippy -j 4
--workspace --all-targets -- -D warnings`, and the two test runs you used last
round. Check `df -h /tmp` first and delete your scratch target directories
before you report.

Append a "Fix round 2" section to
`.superpowers/sdd/2026-09-14-phase-8-surface/task-3-report.md` and return only
status, the commit, a one-line test summary, and concerns.
