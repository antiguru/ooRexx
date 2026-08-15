# Task 8 review: program arguments, `ARG`, `PULL`, `PARSE PULL`, `PARSE LINEIN`

Reviewed `review-0f427f61..81944c76.diff` (commits `8eb5a402`, `81944c76`) against `task-8-brief.md` and `task-8-report.md`, plus the tree at `81944c76`.

## Verdict 1 -- spec compliance: PASS

| Brief requirement | Verdict | Evidence |
|---|---|---|
| Step 0(a): a command line supplies at most one argument string; `ARG()` 0 vs 1; commas do not split; `rexx p.rex ""` is present-and-empty | ✅ | Re-measured independently against the oracle: `none`/`empty`/`two-empty`/`empty-x`/`x-empty`/`blank-x`/`three`/`quoted`/tab/high-byte all AGREE on stdout, stderr and rc between `rexx-run` and the oracle |
| Absent is `None`, not `Some("")` | ✅ | `Invocation.argument: Option<Vec<u8>>` (`invocation.rs:106`); `join_command_line` yields `None` only for an empty word list |
| A test distinguishes absent from empty | ✅ | `no_words_is_absent_and_one_empty_word_is_present` (unit, and deliberately separate from the byte table); `input_oracle.rs` rows `arg-none`/`arg-empty` (stdout) and `strict-none`/`strict-empty` (rc 216 vs 0) |
| Step 0(b): one shared cursor; `PARSE LINEIN` never consults the queue; only `PULL` uppercases | ✅ | One `Input` field on `Interp` (`lib.rs:1342`); `pull_line` = `queue.pop()` else `linein_line`; `linein_line` = `.input` only. Verified live: queue `qA,qB` + stdin `sA,sB,sC` gives `[qA][sA][qB][sB][sC]` on both interpreters |
| Only three of the five ooTest constructs are in scope; `LINEIN()`, `.input~lineIn` still fail | ✅ | Probed: `arg()` -> loud 4c rc 120, `linein()` -> loud 4c rc 120, `charin()`/`lines()` -> loud 4c, `.input~lineIn` -> loud Phase 5. `input.rs`'s module doc states the same and does not claim the assertion as a target |
| Step 0(c)/(f): empty queue + `/dev/null` is the null string, no hang; both trace modes probed | ✅ | Verified both modes myself against the oracle: `trace r` and `trace i`, queue-sourced `PULL`, console `PARSE PULL`/`PARSE LINEIN`, `ARG`/`PARSE ARG` -- stderr byte-identical on both. `>K>` present in both modes for PULL/LINEIN, absent for both ARG spellings, pre-upcase split reproduced |
| Step 0(e): `ARG template` is `PARSE UPPER ARG`; `UPPER` after the source is a target; bare forms legal | ✅ | One arm in `run.rs:1888`; verified `arg n4` -> `MIXED` against `parse arg n5` -> `mIxEd` on the same run |
| Step 1: `rexx-run` takes trailing words; `run_program` gains a parameter | ✅ | `bin/rexx-run.rs:53`; `run_program(path, text, invocation)`; `run_program_collect_every_alloc` and private `execute` likewise |
| Step 1: extend the oracle wrapper rather than record a gap; keep `ulimit` and the invocation counter | ✅ | `Oracle::run_with` (`support/oracle.rs:171`); `run` delegates; counter incremented at the top of `run_with`; the `ulimit -v … && exec "$0" "$@"` wrapper is unchanged and forwards `args` for free |
| Step 2: queue-empty fallback measured; first differential witness for 4b's `PUSH`/`QUEUE`; the I15 gate row updated | ✅ | `input_oracle.rs::queue-round-trip` (live under the gate) plus `corpus/lang/pull_queue.rex` block A; `docs/superpowers/plans/phase-4-exclusions.txt`'s I15 row marked CLOSED with both witnesses named |
| Step 4: `owners.rs` rows, `EXPECTED_OUT_OF_SCOPE` rows, `loud.rs` witnesses deleted; `lib.rs` arm | ✅ | All deleted. The arm shrank to `InstructionKind::Address(_) => Some("4c")` rather than disappearing -- the brief was wrong about the group's membership, and the report says so. Audited counts moved 32->34 / 3->1 / witnesses 11->9, all asserted |
| Step 5: verify block | ✅ (partly re-run) | `cargo fmt --all --check` re-run here: exit 0. `cargo test -p rexx-exec --lib` 400 passed, `--test collect_stress` 3 passed, `REXX_CORPUS_GATE=1 --test input_oracle` 9 passed. Workspace test and cold-target clippy taken from the report |
| Project rules: no `unsafe`, allocation through `alloc_with`, loud at 120 | ✅ | The one new allocation is `interp.text(&argument)` -> `text_owned` -> `alloc_with` (`value.rs:64`) |

### Report claims checked independently

1. **`tests/corpus.rs` does not read `phase-4c.txt`; the gate stays at 42.** Confirmed -- `corpus.rs:436` reads `phase-4a.txt` + `phase-4b.txt` only. **But "inert" is too strong**: `coverage.rs::every_in_scope_variant_is_witnessed_by_the_phase_subsets` reads the union *including* `phase-4c.txt` and parses each program, so `pull_queue.rex` is what discharges the newly in-scope `Arg`/`Pull` variants' coverage obligation today. `rexx-parse/tests/sourceline_oracle.rs` also parses it (new committed `pull_queue.txt`). Nothing *runs* it. No other check silently depends on it.
2. **The top-level `use strict arg` 40.3 fix.** Correct and in blast radius: `call_context.name` had to be filled for the argument to live in `call_context`, and filling it is what makes the substitution right. Verified byte-identical against the oracle for absolute, relative and dot-laden invocation paths, and for the matching 40.4.
3. **An unreadable stdin is end of input, rc 0, no condition.** Re-measured myself, both a directory and a closed fd, on both interpreters: `1 0` .. `4 0`, rc 0, empty stderr, all four identical. The deleted `Result`/`Loud` path is justified.
4. **The `\r\n` collapse is exactly one CR and only before a newline.** Re-measured: `crlf\r\nplain\nx\r\r\ny\r` -> `63726C66` / `706C61696E` / `580D` / `790D`, and `a\rb\nq\r\r\r\nz\n` -> `610D62` / `710D0D` / `5A`. Both interpreters agree. The rule as implemented and as documented is right.
5. **The `git checkout --` recovery.** Final state checked rather than trusted: `Loud::parse_source` is gone with no dangling references, `run.rs`'s two hunks are present and coherent, `Queue::pop`, `pull_line`/`linein_line`, `join_command_line` and the `\r\n` collapse are all present, `cargo fmt --check` is clean, and every behaviour above reproduces from the committed tree. No lost hunk found.

## Verdict 2 -- task quality

### Important

* **The trace shape for the two new sources is described, not pinned by anything that runs today.** `>K>` being a `results` prefix, and a bare `PULL`'s `>K>` carrying the pre-upcase value while the following `>>>` carries the upcased one, are stated in `parse_template.rs:496`-`502` and witnessed only by `corpus/lang/pull_queue.rex` block E, which nothing executes until Task 15. Two live homes existed in this same commit: `tests/input_oracle.rs` already compares stderr through the DEVIATION-0 normaliser, and `tests/trace_oracle.rs` runs `run_program` against committed expectations offline (a queue-sourced `PULL` needs no console). I verified the behaviour is correct in both modes, so this is a coverage gap and not a defect -- but a comment stating a measured trace split with no runnable assertion behind it is the shape this project has had rot on it before.

### Minor

* **`input_oracle.rs`'s anti-vacuity guard has no floor.** `assert_eq!(oracle.invocations(), CASES.len())` compares two values derived from the same array: an empty (or shortened) `CASES` passes both that and `failures.is_empty()` having compared nothing. `tests/corpus.rs` pairs its count with `assert!(!subset.is_empty())` and `coverage.rs` pairs its subset with a committed line list; this harness does neither.
* **`ProgramInput::Bytes` is public API with no caller outside `input.rs`'s own unit tests.** `Stdin` has one caller (`bin/rexx-run.rs`), `Nothing` is the default; `Bytes` exists solely so the module's tests can feed a buffer, which they could do through the private `Source`. Speculative surface on a `pub` enum.
* **Two unasserted call-site counts in comments.** `invocation.rs:115` ("`ProgramInput::Stdin` has exactly one caller", with `:124` naming it) and `queue.rs:137` ("the one caller (`Interp::pull_line`)"). `rust/CLAUDE.md` forbids exactly this ("may not say how many call sites there are ... assert it in a test; if it is not, delete it"), and the first is load-bearing -- it is the whole of the "a test cannot block" safety argument. The neighbouring sentence ("not reachable by omission ... a caller that wants it has to name it") is the true, non-rotting form of the same claim and would stand alone.
* **`an_unreadable_console_is_end_of_input` rebuilds the oracle wrapper by hand.** It duplicates `ulimit -v`, `LD_LIBRARY_PATH` and the binary path, and bypasses the invocation counter -- the two things `run_with`'s own doc says it was written to keep in one place. `let _ = oracle;` after calling `locate()` purely for its existence assertion is the tell. It is correct today (same `oracle_root()`), but it is a second copy of the wrapper.
* **`execute` copies the argument bytes it already owns.** `interp.text(&argument)` on an owned `Vec<u8>` where `text_owned(argument)` exists precisely to avoid the copy (`value.rs:63`, whose own doc argues the point).
* **`input.rs:58`: "There is no reachable state in which a line read fails."** Read alone this is an unreachability claim about the Rust code rather than about the oracle. The following paragraph rescues it by naming the two measured cases, but the sentence is the form this project has been wrong about six times; "every state that could fail answers the null string, measured" says the same thing without the quantifier.
* **`phase-4-exclusions.txt`'s new paragraph asserts a fact about a different file** -- "No row there uses `TRACE ?` or sets RXTRACE" about `input_oracle.rs`'s `CASES`. Nothing enforces it and it goes stale the first time someone adds a trace row (which the Important finding above wants).
* **Two counts in the report's prose are wrong** (code unaffected): "18 case rows" -- `CASES` has 19; "36 trace lines" for `pull_queue.rex`'s stderr -- the hand-run produces 38 (the two `*-* trace off` clause lines). The report's "input_oracle stayed 9/9 green" is right: the binary has 9 tests, 7 of them `support::tests`.

### Things done well, recorded because they are the failure modes this project tracks

* The mutation table pairs each mutation with what did *not* catch it, and the `join_command_line` row explicitly records that the byte table and the absent/empty test are not redundant. That is "can fail" versus "adds coverage" done properly.
* `arg-one-quoted-word` is labelled in the file as the adjacent success for `arg-three-words`, and `arg-trailing-empty-word` is the asymmetric partner for `arg-leading-empty-word`. Both are the pairing rule applied without being asked.
* Measuring the unreadable-descriptor case before designing the error model deleted a `Result` and a `Loud` variant rather than adding them.
* The mechanical commit is uniform: all 45-odd sites pass `Invocation::none()`, none passes an argument, and no test's meaning changed.

## Cannot verify from the diff

* ⚠️ **The five `keyword-exempt.txt` removals** (`PARSE::test_PARSE_variable_patterns`, `Test_614`, `Test_620`, `Test_626`, `Test_632`). The two-directional policing in `keyword_assertions.rs` makes a wrong removal a test failure, and the report says the suite is green, but I did not re-derive the five bodies.
* ⚠️ **The workspace test count (1151 / 73 `ok` lines) and the cold-target clippy run.** Not re-run per the review brief; `cargo fmt --all --check` and three targeted test binaries were re-run and are clean.
* ⚠️ **"Task 15's gate should say so"** (brief Step 2). The closure was written into `docs/superpowers/plans/phase-4-exclusions.txt`, which the 4c plan names as the live record; the 4c plan's own Task 15 step was not amended. Whether that satisfies the instruction is a controller call.
