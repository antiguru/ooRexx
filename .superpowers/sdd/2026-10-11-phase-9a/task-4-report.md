# Task 4 report: D3 reasons and the relabels

BASE 052550e85. Status: DONE_WITH_CONCERNS (concerns at the end; none is in this task's code).

## What changed

* `rexx-core/src/owner.rs`: `pub enum Deferral { Rxapi, Extension, Row10NativeApi, Embedding }` and
  `pub enum Owner { Phase9, Phase10(Deferral) }`. `Owner::label()` is the only text naming Phase 10
  in the crates the structural test scans: `"Phase 9"`, `"Phase 10 Rxapi"`, ... (the controller's
  ruling, "(Phase 10 <Variant>").
* Each constructor shape takes an `Owner`: `util`, `deferred` (and `ExternalBody::Deferred`,
  `deferred_send`), `Loud::internal_routine`, `Loud::redirection`, the `gap` closure. Literal
  owners are `Owner::...label()`: `named_semaphore`, `package_option_write`, the classic-library
  site, the `.STDQUE` owed entry.
* `rexx-inventory`'s `EXCLUDED` is `(name, phase, reason)`, the reason the variant's name as a
  string; `owner_of` answers `(phase, reason)`; `rexx-exec`'s `excluded_owner` turns it into the
  label through `Owner::named`.
* `rexx-api`'s `REFUSING_MEMBERS` is `(&str, Owner)` (rexx-api depends on rexx-core, so the reason
  column is typed rather than a string); `layout::refusal(what, owner)` is the one constructor of a
  rexx-api refusal text, used by `refusal_text` and by both `ffi.rs` assertion texts.
* Test side: `suite_rows::Phase10Reason` is gone; `suite_rows` re-exports `rexx_core::Deferral`
  (one vocabulary). `refusal_carries(text, reason)` requires `"(" + Owner::Phase10(reason).label()
  + ")"` in the captured text.
* `PHASES` (`tests/internal_routines.rs`) and `OPEN` (`src/dispatch/native/tests.rs`) gain
  `"Phase 9"` and compare `owner.phase()`.
* New `tests/phase_ten_reasons.rs` (D3's structural test); the literal scanner `closed_phases.rs`
  used moved unchanged to `tests/support/literals.rs`, which both use.

## Classification

Every `"Phase 10"` site at BASE, plus R4's `"Phase 9"` strings in `rexx-api`.

| Site (BASE line) | Owner | Reason / evidence |
|---|---|---|
| `internal_routines.rs` `util` rows `SysAddRexxMacro`, `SysClearRexxMacroSpace`, `SysDropRexxMacro`, `SysLoadRexxMacroSpace`, `SysQueryRexxMacro`, `SysReorderRexxMacro`, `SysSaveRexxMacroSpace` | Phase 10 `Rxapi` | the macro space lives in the RXAPI daemon (brief; spec D7) |
| `internal_routines.rs` every other `util` row (`SysCls` ... `SysWait`, the rows at :103-164 not named above) | Phase 9 | R2 (brief) |
| `dispatch/native.rs:301-310` `rexx_*_queue` | Phase 10 `Rxapi` | brief; oracle: `RexxQueueMethods.cpp` calls `RexxAddQueue`/`RexxPullFromQueue`, which are `rexxapi/client/QueuesAPI.cpp` and send to the daemon, the session queue included (`LocalQueueManager.cpp:445` `ADD_TO_SESSION_QUEUE`) |
| `dispatch/native.rs:264` `handle_set` | Phase 9 | fits no reason. Oracle reading: `StreamClasses.orx:176-177` calls `!handle_set` for a `HANDLE:` name, `StreamNative.cpp:2576` `handle_set` adopts the descriptor (`StreamInfo::setHandle`); no daemon, extension, row 10 group or embedding. Probe (read shape only, `oracle-crashes.txt` entry 12a forbids the write): `s = .Stream~new('HANDLE:1')` / `say s~state` / `say stream('HANDLE:1','C','query streamtype')` -- oracle rc 0, stdout `UNKNOWN` twice; crate rc 120, stderr `rexx-exec: the LIBRARY REXX entry point "handle_set" is not implemented (Phase 9)` |
| `dispatch/library.rs:838` `Refused::ClassicStyle` | Phase 9 | fits no reason. Oracle reading: `LibraryPackage.cpp:279-281` wraps a `ROUTINE_CLASSIC_STYLE` row in a `RegisteredRoutine`; the call (`NativeActivation.cpp:1527`) reads the queue name through `Interpreter::getCurrentQueue` (`Interpreter.cpp:638`), which sends `GET` to `.STDQUE`, a `RexxQueue~new('SESSION')` whose `init`/`get` (`StreamClasses.orx:445-459`) never reach the daemon. No ooTest group reaches it: `git grep -l 'REXX_CLASSIC_ROUTINE\|ROUTINE_CLASSIC_STYLE'` over `ootest`, `testbinaries`, `extensions`, `samples` finds only `extensions/platform/windows/rxwinsys/rxwinsys.cpp` |
| `redirect.rs:426`, `:521` `Loud::redirection("RexxQueue", ...)` | Phase 10 `Rxapi` | oracle reading: output, `CommandIOConfiguration.cpp:484-495` builds a `RexxQueueOutputTarget`, whose `writeLine` (`OutputRedirector.cpp:922`) sends `QUEUE`, i.e. `rexx_queue_queue`; input, the `RexxQueue` is asked `makearray` (`StreamClasses.orx` `makearray` -> `queued`/`pull`, `rexx_query_queue`/`rexx_pull_queue`); all through the daemon as above |
| `directives.rs:80` `gap(... REGISTERED ...)` | Phase 10 `Rxapi` | brief (the routine registry is the daemon's, the site's own comment) |
| `environment.rs:363` `.STDQUE` owed entry | Phase 10 `Rxapi` | brief; now `Some(Owner::Phase10(Deferral::Rxapi).label())` |
| `lib.rs:511` `named_semaphore` | Phase 9 | brief |
| `lib.rs:664` `package_option_write` | Phase 9 | brief, spec D3 |
| `rexx-inventory` `EXCLUDED` `RXQUEUE`, `RXFUNCADD`, `RXFUNCDROP`, `RXFUNCQUERY`, `QUEUED` | Phase 10 `Rxapi` | queues and the function registry are the daemon's; `QUEUED`'s excluded form is its cross-process half (the row's own comment) |
| `dispatch/native/tests.rs:150` `OPEN` | vocabulary | test module; gains `"Phase 9"` |
| `layout.rs:379-383` `REFUSING_MEMBERS` (were `"Phase 9"`) | Phase 10 `Embedding` | R4 |
| `ffi.rs:4187` `AttachThread`, `:4229` `AddCommandEnvironment from another thread` (were `"Phase 9"`) | Phase 10 `Embedding` | R4 |
| `phase-4-exclusions.txt:5516` (`TEST_REXXQUEUE`, `rexx_create_queue`) | Phase 10 `Rxapi` | the row's own text: the oracle's RexxQueue creates a named queue in rxapi's daemon. `OWNER: Phase 10.` became `OWNER: Phase 10 Rxapi.`; the quoted 2026-09-28 refusal text is a measurement and stays |

No site moved to Phase 9 had a row depending on it: the four `refusal:` rows in
`phase-9-exclusions.txt` (`LINEOUT.TEST_QUEUE`, `RexxQueue.TEST_BUG_1379`, `ROUTINE`'s two
`REGISTERED` tests) are all `Rxapi` sites. `suite-licences.txt` names no Phase 10.

### The grep (Step 2), at BASE

`git grep -n '"Phase 10"' 052550e85 -- rust/crates | grep '/src/'` (61 lines; `git grep` with the
pathspec `'rust/crates/*/src'` matched nothing, so the filter is on the output):

```
rust/crates/rexx-exec/src/directives.rs:80:            gap("::ROUTINE EXTERNAL naming REGISTERED", "Phase 10")
rust/crates/rexx-exec/src/dispatch/library.rs:838:                    message: crate::owned_message(&format!("{refused}"), Some("Phase 10")),
rust/crates/rexx-exec/src/dispatch/native.rs:264:    deferred("handle_set", Family::Stream, "Phase 10"),
rust/crates/rexx-exec/src/dispatch/native.rs:301:    deferred("rexx_create_queue", Family::Queue, "Phase 10"),
rust/crates/rexx-exec/src/dispatch/native.rs:302:    deferred("rexx_open_queue", Family::Queue, "Phase 10"),
rust/crates/rexx-exec/src/dispatch/native.rs:303:    deferred("rexx_queue_exists", Family::Queue, "Phase 10"),
rust/crates/rexx-exec/src/dispatch/native.rs:304:    deferred("rexx_delete_queue", Family::Queue, "Phase 10"),
rust/crates/rexx-exec/src/dispatch/native.rs:305:    deferred("rexx_query_queue", Family::Queue, "Phase 10"),
rust/crates/rexx-exec/src/dispatch/native.rs:306:    deferred("rexx_push_queue", Family::Queue, "Phase 10"),
rust/crates/rexx-exec/src/dispatch/native.rs:307:    deferred("rexx_queue_queue", Family::Queue, "Phase 10"),
rust/crates/rexx-exec/src/dispatch/native.rs:308:    deferred("rexx_pull_queue", Family::Queue, "Phase 10"),
rust/crates/rexx-exec/src/dispatch/native.rs:309:    deferred("rexx_linein_queue", Family::Queue, "Phase 10"),
rust/crates/rexx-exec/src/dispatch/native.rs:310:    deferred("rexx_clear_queue", Family::Queue, "Phase 10"),
rust/crates/rexx-exec/src/dispatch/native/tests.rs:150:    const OPEN: &[&str] = &["Phase 10"];
rust/crates/rexx-exec/src/environment.rs:363:        let mut owed = [(ObjRef::NIL, None), (ObjRef::NIL, Some("Phase 10"))];
rust/crates/rexx-exec/src/internal_routines.rs:99:    util("SysAddRexxMacro", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:100:    util("SysClearRexxMacroSpace", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:103:    util("SysCls", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:112:    util("SysCreatePipe", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:113:    util("SysDropFuncs", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:114:    util("SysDropRexxMacro", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:115:    util("SysDumpVariables", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:116:    util("SysFileCopy", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:119:    util("SysFileMove", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:120:    util("SysFileSearch", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:122:    util("SysFork", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:123:    util("SysFormatMessage", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:124:    util("SysGetErrorText", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:125:    util("SysGetFileDateTime", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:126:    util("SysGetKey", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:127:    util("SysGetMessage", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:128:    util("SysGetMessageX", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:130:    util("SysIsFileDirectory", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:131:    util("SysIsFileLink", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:133:    util("SysLoadFuncs", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:134:    util("SysLoadRexxMacroSpace", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:136:    util("SysOpenEventSem", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:137:    util("SysOpenMutexSem", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:139:    util("SysQueryProcess", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:140:    util("SysQueryRexxMacro", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:145:    util("SysReorderRexxMacro", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:152:    util("SysSaveRexxMacroSpace", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:153:    util("SysSearchPath", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:154:    util("SysSetFileDateTime", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:155:    util("SysSetPriority", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:157:    util("SysStemCopy", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:158:    util("SysStemDelete", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:159:    util("SysStemInsert", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:160:    util("SysStemSort", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:161:    util("SysTempFileName", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:162:    util("SysUtilVersion", "Phase 10"),
rust/crates/rexx-exec/src/internal_routines.rs:164:    util("SysWait", "Phase 10"),
rust/crates/rexx-exec/src/lib.rs:511:                Some("Phase 10"),
rust/crates/rexx-exec/src/lib.rs:664:            message: owned_message("a package settings write", Some("Phase 10")),
rust/crates/rexx-exec/src/redirect.rs:426:            return Err(Loud::redirection("RexxQueue", "Phase 10").into());
rust/crates/rexx-exec/src/redirect.rs:521:            return Err(Loud::redirection("RexxQueue", "Phase 10").into());
rust/crates/rexx-inventory/src/lib.rs:28:        ("RXQUEUE", "Phase 10"),
rust/crates/rexx-inventory/src/lib.rs:29:        ("RXFUNCADD", "Phase 10"),
rust/crates/rexx-inventory/src/lib.rs:30:        ("RXFUNCDROP", "Phase 10"),
rust/crates/rexx-inventory/src/lib.rs:31:        ("RXFUNCQUERY", "Phase 10"),
rust/crates/rexx-inventory/src/lib.rs:36:        ("QUEUED", "Phase 10"),
```

R4's `rexx-api` strings, found by the literal scan (`closed_phases`' `literals`/`names` over
`source_files("rexx-api")`, which reads literals whole), were the five `REFUSING_MEMBERS` rows and
the two `ffi.rs` texts above; after the change no literal in `rexx-api/src` names any phase.

## Structural test (Step 1)

`tests/phase_ten_reasons.rs`: `no_production_literal_names_phase_10` reads every literal of
`rexx-exec/src` and `rexx-api/src` outside inline `#[cfg(test)] mod` blocks, `tests.rs` files and
the files a `#[cfg(test)] mod NAME;` loads, and fails on any naming Phase 10. Controls in the same
file: the walk with test modules included finds Phase 10 literals, and test files are left out
while `lib.rs`, `dispatch/native.rs`, `environment.rs` are kept; a fixture for the inline-module
blanking (neighbours before and after still read, a `#[cfg(test)] fn` is not a module); module-file
resolution beside `lib.rs` and below `ir.rs`; every `EXCLUDED` reason resolves through
`Owner::named` and its label is `"{phase} {reason}"`; every `Deferral`'s label is
`"Phase 10 {name}"`.

Fails on BASE's shape: mutant `named_semaphore`'s owner back to `Some("Phase 10")` ->
`cargo test --release -p rexx-exec --test phase_ten_reasons` exit 101, `no_production_literal_names_phase_10`
FAILED naming `"rexx-exec/src/lib.rs:511: \"Phase 10\""`; reversed by the exact edit, `cmp` against
the saved copy identical.

`refusal_carries` tightening, mutant: body back to `text.contains("(Phase 10")` ->
`cargo test --release -p rexx-exec --test suite row_checks` exit 101,
`each_owner_check_passes_its_rows_and_fails_the_others` FAILED on the new FAILING fixture
`... a queue is not implemented (Phase 10)`; restored from the saved copy, `cmp` identical. The new
FAILING fixtures are a bare `(Phase 10)`, `(Phase 10 Extension)` on an `Rxapi` row, and
`(Phase 10 Rxapi2)`.

## refusal-sites.tsv (Review Focus 5)

`REXX_REFUSAL_SITES_REFRESH=1 cargo test --release -p rexx-exec --test refusal_sites -- --test-threads=1`
exit 0. `git diff --stat`: 34 insertions, 34 deletions. The file holds no refusal text and no phase
name; a field-by-field comparison of the old and new rows (same row count) found every changed
field in column 4 (`definition`), and in each the path before the last `:` unchanged: only line
numbers moved.

The refusal texts themselves, other than the label, are unchanged: every expected-text edit in
this commit (`src/dispatch/tests.rs`, `src/eval/tests.rs`, `src/run/tests/directives.rs`,
`src/scheduler/tests/native.rs`, `tests/spike.rs`, `tests/suite.rs`'s `MACRO_REFUSAL`, the four
`phase-9-exclusions.txt` refusal rows) changes only the parenthesised label (`(Phase 10)` to
`(Phase 10 Rxapi)` or `(Phase 9)`), and those tests pass against the running crate. The gated suite
over the three groups whose rows changed captured, byte for byte, BASE's committed baseline
(`docs/superpowers/records/2026-10-11-phase-9a/suite-baseline/table.tsv`) texts with ` Rxapi`
inserted:

```
base/bif/LINEOUT	TEST_QUEUE=rexx-exec: environment symbol ".STDQUE" is not implemented (Phase 10 Rxapi)
base/class/RexxQueue	TEST_BUG_1379=rexx-exec: the LIBRARY REXX entry point "rexx_query_queue" is not implemented (Phase 10 Rxapi)
base/directives/ROUTINE	TESTDOTROUTINES=... (Phase 9) | TESTEXTERNALREGISTEREDBADLIBRARYNAME=rexx-exec: ::ROUTINE EXTERNAL naming REGISTERED is not implemented (Phase 10 Rxapi) | TESTEXTERNALREGISTEREDBADNAME=... (Phase 10 Rxapi) | TESTPACKAGE=... (Phase 9)
```

Verdicts identical to the baseline's `verdicts.tsv` for all three (LINEOUT red: not passed
TEST_BINARY TEST_INVALID_FILE TEST_LINE_NONNUMERIC TEST_LINE_TOO_LONG; RexxQueue green; ROUTINE
red: not passed TESTDOTROUTINES TESTPACKAGE), with no row problem reported.

Gate groups reaching a Phase 10 site after the relabel, from the baseline `tests.tsv`: only
`rexx_clear_queue`, `rexx_query_queue`, `.STDQUE` and `REGISTERED`, all `Rxapi`, all on rowed tests.

## Checks (each run unpiped, output to a file, status echoed)

All cargo commands with `CARGO_TARGET_DIR=/tmp/claude-1000/p9/t4-target CARGO_INCREMENTAL=0 memcap 8G`, `-j 4`.

| Command | Exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy -p rexx-core -p rexx-inventory -p rexx-api -p rexx-exec --all-targets -- -D warnings` | 0 |
| `cargo test --release -p rexx-exec --test owners --test closed_phases --test refusal_dispositions --test refusal_sites --test internal_routines --test phase_ten_reasons --test spike --test native_entries --test suite --no-fail-fast` | 0 (6, 8, 3, 5, 2, 6, 9, 24, 50 passed) |
| `cargo test --release -p rexx-api -p rexx-core -p rexx-inventory --no-fail-fast` | 0 |
| `cargo test --release -p rexx-exec --lib` | 101: 1082 passed, 1 failed, pre-existing (concern 1) |
| `cargo test --release -p rexx-exec --test coverage` (in an earlier combined run) | 101: 20 passed, 1 failed, pre-existing (concern 1) |
| `REXX_BIF_GATE=1 cargo test --release -p rexx-exec --test bif_assertions` | 0 (5 passed) |
| `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/bif/LINEOUT,base/class/RexxQueue,base/directives/ROUTINE cargo test --release -p rexx-exec --test suite` | 101: `l3_core_suite` 2 of 3 groups red, the same verdicts as BASE's baseline (above) |

Per-task wall time: about 2 minutes for the final fmt, clippy and test runs over a warm target (the test run 1m54s).

## Concerns

1. **Two failures predate this task**, both from Task 3's `rust/corpus/phase-9d-set.txt`
   (c0e99bd1d), which matches the `phase-*.txt` glob harnesses use for corpus program lists:
   `coverage::the_union_reads_every_phase_subset_file` (`SUBSET_FILES` lacks it) and
   `ir::corpus_shape_tests::every_corpus_body_compiles_the_minimum_promotion_set_to_its_own_ops`
   (reads its rows as program paths: `cannot read .../corpus/base/bif/ARG	TEST_EXTERNAL_CALLS`).
   This task changed neither file's logic (`coverage.rs` only destructures `EXCLUDED`'s new column).
   Other `phase-*.txt` globs in gated harnesses may trip the same way.
2. **`handle_set` is now Phase 9's, and its body adopts a raw descriptor**, which needs `unsafe`
   (`from_raw_fd`) and so a per-site grant from Moritz; the `HANDLE:` write shape kills the oracle
   (`oracle-crashes.txt` 12a), so its witness can only be a read.
3. **The classic-style library routine is now Phase 9's**, but its oracle call reads the queue name
   from `.STDQUE`, which this crate refuses as Phase 10 `Rxapi`; no Linux ooTest group reaches it.
4. Files the brief did not name, changed because their content derives from the refusal text:
   `rust/corpus/bif-exempt.txt` (RXQUEUE rows `Phase 10` -> `Phase 10 Rxapi`, and the header line
   saying so) and `tests/bif_assertions.rs`'s `PHASES`; `tests/coverage.rs`, `tests/owners.rs` for
   the `EXCLUDED` column; `tests/refusal_dispositions.rs` reads `Owner` forms (an `Owner` parameter's
   label is the caller's, `Owner::...label()` is that label) and `COMPUTED_OWNERS` names
   `excluded_owner`; `tests/closed_phases.rs`'s two negative controls were repointed, since
   `rexx-api` no longer holds a phase literal and `rexx-exec`'s production code no longer holds
   `"Phase 10"`.
5. Not touched: `phase-4-exclusions.txt:5284` (the second instance, R4) still reads `OWNER: Phase 10`
   with no reason; `refusal-dispositions.tsv` records quoting probes that "only STDQUE (Phase 10)
   refused" are history and stay.
