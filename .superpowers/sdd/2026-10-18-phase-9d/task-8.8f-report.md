# Task 8.8f: Semantic residuals and interrupted named-stream reads

## Scope and source

The assigned eight queued items are implemented at `3979b7e31`, based on verified `48e73fe3a16c10e96937546c03b11766a31419ef` in the `plan/rust-rewrite` linked worktree. The implementation preserves the e2 required-input `Failure` APIs and actual worker-spawn 48.1 behavior. It introduces no dependency, unsafe block, descriptor ownership interface, identity side map, or scheduler change. All-program own-BASE performance, final complete corpus/suite/workspace gates, and independent review belong to the controller and are not claimed by this report.

The initial pure moves separated stream reads, assignment handling, RAISE handling, and PARSE triggers in `c85fa469c`, followed by hash removals in `0f48350f8`. Regression witnesses were committed in `a41612620` before running the red probes against the e2 pin. Implementation commits are `f3a5590b7`, `04d2aa894`, `ed68ca5dd`, `12d160b9e`, `395cf43b2`, `9e4e172ab`, `54c84b9c6`, `737ce07f3`, and `3979b7e31`. Each source revision was committed before verification, and the final source remained frozen during final checks and release construction.

## Completed boundaries

| Queued item | Responsible change | Focused evidence |
| --- | --- | --- |
| Stem index string values | `stem_tail` obtains and roots each required string, consumes it before the next conversion, and inserts separators by argument position. | `stem`, `nostring`, and registered `required_string_boundaries` cover lookup, write, hasIndex, removal, references/defaults, omitted indexes, and an empty leading index. |
| PARSE pattern conversion | String and mixed search triggers use required strings while preserving source-buffer borrowing and numeric positional triggers. | `parse-make`, `parse-raise`, `nostring`, and the registered corpus control cover search, caseless search, raised conversion, and ordinary positions. |
| MAX/MIN arguments | A borrowed ordinary path and a rooted lazy conversion path feed converted values to both whole-number quietness and numeric parsing. | `max-controls`, `max-lostdigits`, and `max-references` cover one send, missing/invalid arguments, raised conversion, whitespace, references/defaults, and Deviation 44. |
| TRACE assignment rendering | Each emitted result or assignment line obtains its own fallible trace naming result at its existing before/after-write boundary. | `trace-raise`, `trace-gates`, and the existing Deviation 81 test verify the third naming failure, completed assignment, and OFF/R/I behavior. |
| Hash collection messages | Removal consumes one located slot/predecessor, indexed Relation/Bag operations inspect the selected bucket, and observable reentry invalidates stale store locations. | `hash-messages`, `hash-controls`, `hash-collision`, `hash-raises`, `hash-reentry`, and `hash-equality-reentry` cover duplicate sends, bucket misses, collisions, raised sends, and store replacement. |
| Stdout regular-file queries | Existing `own_descriptors` and transient classification gate safe descriptor metadata/seek queries and logical pending-byte accounting. | Redirected stdout verifies SIZE/default/WRITE CHAR/SYS plus external file bytes, size, and shared offset. A buffered Invocation negative control does not inspect host stdout. |
| RAISE DESCRIPTION | Required-string conversion occurs at RAISE and stores the converted bytes. | `description`, `description-timing`, and the full ruled `raise_untraced_renders_nothing` witness pin the approved Deviation 83 behavior. |
| FIFO/terminal reads and closing LINEOUT | Named transient reads stop on EINTR and discard the incomplete operation's prefix, using a fixed small buffer. | `fifo/` proves interruption of the actual blocked thread without a writer unblock. `fifo-close/` retains raw PTY and all closing-LINEOUT controls under Deviation 82. |

The Stem adjacent-boundary oracle agrees that an omitted index contributes empty bytes and that a leading empty index retains its following period. The shared helper therefore fixes these cases together with conversion, without a new departure. Reference and stem-default differences remain governed by existing Deviations 72 and 77. TRACE continues to name objects under Deviation 81, independently of required-string conversion.

## Numeric ordering and retained rulings

Deviation 44 determines whole-number quietness by value, so a converted whole-number string must behave like its identical plain-string control. At DIGITS 5, both the plain `1234567890` and its makeString equivalent produce `1.2346E+9` here, whereas the oracle reports LOSTDIGITS. The paired earlier-wide/later-object cases in `max-controls.rex` record a later `make 2` or `bad conversion` before the final result or SYNTAX 98.900 here, while the oracle stops at the earlier wide argument. This order follows the existing whole-argument quietness prepass: the later conversion is required to decide whether all preceding whole values remain quiet.

The prepass stops once a missing, invalid, or nonwhole value makes that decision unnecessary. An earlier `bad` reports 93.904 and an earlier omitted method argument reports 93.903 without sending the later object's conversion. An earlier `1234567890.1` reports LOSTDIGITS before the later raising object is converted, and surrounding whitespace follows the existing written-whole rule. Ordinary argument slices stay borrowed, and converted arguments are rooted and reused rather than converted a second time.

Deviation 82 and Deviation 83 are separate entries in `docs/superpowers/plans/phase-4-exclusions.txt`, quoting Moritz's exact 2026-10-30 approval, "Follow your recommendation on both." Entry 82 maps that approval to preserving FIFO bytes and closing normally, with the oracle consuming the first byte after ESPIPE in the retained e1 syscall evidence. Entry 83 maps it to obtaining DESCRIPTION's required string at RAISE, so the handler sees a stored string and repeated CONDITION(D) reads do not reconvert. No other new departure is licensed by those entries.

The complete original `raise_untraced_renders_nothing.rex` remains intact apart from its corrected comment and moves from phase-9 differential registration into `corpus/unfiled.txt` under Deviation 83. The semantic-boundary test asserts that its complete source equals the corpus file and pins its complete new stdout, stderr, and status 158. The additional `raise_additional_untraced.rex` differential witness retains the unaffected ADDITIONAL and RC behavior, including 98.939 and 98.900 controls, and agrees with the oracle. Oracle-generated SOURCELINE snapshots cover both new registered witnesses and the retained full ruled witness.

## Interrupted-read proof

The reused FIFO driver identifies its own descendant process, executable, start identity, blocked TID, and descriptor-to-FIFO mapping before delivering SIGINT to that exact thread. Warmup reads `abc`, a complete READY marker precedes the signal, and the partial control additionally proves that `P` was consumed before interruption. Both successful pairs return status 0 with empty stderr, ERROR:4, empty read data, no subsequent read retry, no controlled writer unblock, and all owned tasks reaped. `fifo/verification.json` retains the independent assertions and exact per-side stdout sequences, while the associated traces retain the actual EINTR.

Both event files order NOTREADY ERROR:4 before HALT ERROR:4. The clean pair prints `read []` between those events, while the partial crate run prints HALT before the outer `read []` and the partial oracle prints it afterward. Deviation 12 says that for a nondefault stream "the halt is served when the read returns"; it does not specify the placement of that outer print against an independently posted HALT. Existing `serve_posted_halt` checks handle a request already posted by the timer, and the unchanged scheduler/clause machinery handles a request posted later, so this report asserts no stronger ordering and extends neither Deviation 12 nor P60.

The FIFO runtime pin was built from `04d2aa894`, with SHA256 `241d7bc029c9026b54beb06da88aa67c4a0ecc3f2f8b889e1e53442f67c28ef1`. Its read implementation has only rustfmt whitespace and a trailing parameter comma changed in the final source, with that exact diff retained beside the final provenance. The raw PTY pair agrees on READY/open, `read [abc]`, successful write, and READY/close. All three FIFO closing-LINEOUT shapes preserve `hello\n` and return close 0 here, against oracle `ello\n` and close 1, as approved by Deviation 82.

## Reentrant count defect disposition

The separate retained `equality-reentry/probe.rex` empties a Table from its queried index's equality method during REMOVE. Both retained oracle runs print `removed 1 items -1`, whereas e2 prints `removed 1 items 0`, all with status 0 and empty stderr. The intermediate f implementation incorrectly printed `removed 0 items 0` by consuming a stale detached item, and `3979b7e31` restores the e2 answer by refreshing store state after equality. The local contract at `oodocs/rexxref/en-US/collclasses.xml:464-477` defines ITEMS as the number of items, which is zero for the emptied collection.

This count defect is separate from the authorized collection-send fix and from Deviations 82/83. The retained ruling request includes oracle repeats, e2, the intermediate regression, source and binary provenance, and the fixed result. The controller has requested Moritz's choice between a separate Deviation 84 and preserving zero while leaving the oracle defect open. No Deviation 84 is written pending that response, and the count discrepancy is not silently licensed by closing the original collection-send queue item.

## Verification and evidence

All durable artifacts are under `docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/`. `red/`, `first/`, `second/`, and `controls/` retain separate stdout, stderr, status, commands, fresh working directories, and stage source snapshots. Oracle probes use a fresh cwd, a 1 GiB address-space limit, the recorded oracle library path, and a 20-second timeout, after consulting the crash list. Interpreter probes use `memcap 2G`, while builds and tests use `memcap 8G`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target`, and Cargo `-j4`.

| Check record | Source | Result |
| --- | --- | --- |
| `focused-final` | `737ce07f3` | 1,235 library tests plus closed-phase 8, coverage 21, input 34, numeric 28, owner 6, phase-ten 5, refusal 8, representation 3, semantic 3, stdout 5, and TRACE 46 pass. |
| `collect` | `737ce07f3` | Complete registered L0 subset passes under collect-on-every-allocation, 256.63 seconds. |
| `reentry-final` | `3979b7e31` | 1,235 library and 3 semantic-boundary tests pass, including every fixture under normal and collect-every-allocation execution. |
| `lint-complete` | `3979b7e31` | `cargo clippy -j4 -p rexx-exec --all-targets -- -D warnings` passes. |
| `format` | `3979b7e31` | `cargo fmt --all --check` passes. |
| `source-policy-final` | `3979b7e31` | Workspace unsafe-site scan, granted unsafe-site policy, and SOURCELINE coverage of every corpus program pass. |
| `refusal-refresh` | `54c84b9c6` | Refusal refresh and 8 tests pass; generated refusal table is unchanged. |
| `release-probes` | `3979b7e31` | All 17 paired release probes exit 0; `final-comparison.json` pins exact descriptor agreement or the applicable existing/approved deviation. |

The command wrapper and exact commands, status codes, elapsed times, environment, stdout, and stderr are retained under `checks/`. The first implementation build failed with a PARSE borrow conflict and was corrected by borrowing the converted rendered needle before searching. An initial formatting invocation used the repository root instead of the Rust workspace, and the initial clippy run exposed an orphaned helper doc comment, which was moved back to its definition without dropping its contract. The FIFO observer initially raced with task disappearance after interpreter exit; its failed `crate-r3` attempt remains recorded, and the observer now treats that disappearance as completion before the successful paired reruns.

## Release and handoff

The final release pin is `/tmp/codex-p9d-t88-pins/8.8f-rexx`, SHA256 `0c9fb8371ab3d6dcea780194e02c112ed7981447f7f396e606e5f418975583f0`, built successfully in 89.92 seconds from `3979b7e3135a05da4ef9372bc20e10773bde6efa`; `release-pin.json` records the compiler, build descriptor, and changed-source hashes. The own-BASE runtime remains `/tmp/codex-p9d-t88-pins/8.8e2-rexx`, SHA256 `00e29fde4456ee4845a76941263f495b827c083af35363cb54eb3ac8c6b7c733`. Both disposition columns of the eight assigned inventory rows are closed at `3979b7e31`, while their historical reason/probe fields and every unrelated row remain byte-for-byte unchanged. Parent-owned performance artifacts and the preexisting user ZIP remain unstaged for the controller's subsequent work.

## Fix round 1 after review

The scoped review identified stale removal answers and Relation snapshot store provenance defects, both addressed in `task-8.8f-fix-r1-report.md`. The round's final runtime source is `3e1aa9c879a90214bb5c680357975f58a0081abc`, with paired red/baseline witnesses and ten new normal/every-allocation fixtures under `t8.8/semantics/fix-r1/`. Exact covering commands, independent stdout/stderr/statuses, source-policy results, formatting, clippy, release provenance, and the recovered evidence-path infrastructure mistake are recorded in that report and its `checks/` records. The original f pin and evidence remain preserved, and the separate count-ruling decision remains pending without Deviation 84.

The corrected pin is `/tmp/codex-p9d-t88-pins/8.8f-r1-rexx`, SHA256 `b695b78fe24903b4be5a2874c8cd128193bbe165f92b50a4d6a8a9956d03d3be`, from the final rooted source above. Its build exits 0, and all ten final paired probes agree exactly on stdout, stderr, and status. The final semantic stress, format, and affected all-target clippy checks also exit 0, with the earlier 1,235 library and focused policy checks retained in the round report. The conservative saved-store roots are explicitly described there for the controller's scoped re-review and performance assessment.


## Performance repair round 1

The controller-authorized repair of the measured parse, dirread, and pingmsg gaps is implemented in runtime commit `4c11ee4449a1fe7b537c2a216ee7c10b1a5b12ab`, following pure-move commit `64a2e9a10` and committed boundary controls. PARSE immediate patterns skip redundant conversion/rooting, StringValue probing specializes a shared traversal without callback bookkeeping, and USE avoids temporary retention only for immediate values. F1/F2 files remain byte-identical to dispatch HEAD `b7a82272f3e111d131982854fdc2cb1ecb86acb5`, while heap/default roots, fallible TRACE line boundaries, cached hash semantics, and previous rulings remain intact. No new deviation is written, and the pending count-defect ruling remains separate.

[The repair report](task-8.8f-perf-fix-report.md) records exact commands and preserved outputs under `docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/perf-fix-r1/`. Verification passes 41 semantic/policy tests, 129 focused library controls, 48 TRACE/collection-policy tests, formatting, affected all-target clippy, and release construction, with semantic fixtures also running under collection on every allocation. Three final paired probes match oracle and baseline stdout/stderr/status independently, and the new `/tmp/codex-p9d-t88-pins/8.8f-perf-r1-rexx` has SHA256 `6ed1c08eb03d54c48d1fe3947ce376f09502aaeba34ac221c334b01b1572b86e`. Scoped re-review, matched all-program measurement, and broad final gates remain controller-owned, so this functional handoff does not claim performance-budget acceptance.


## Performance repair round 2

Runtime commit `cd418bd304e4bd1fedd339cb090ae861b1a460d8` restores separate String/Mixed required-string conversion and packed rendering, skipping only a converted-result root whose handle equals the already-rooted operand. The source audit preserves StringValue, USE, F1/F2, and scheduler production behavior exactly, while 3 semantic tests with allocation stress, 69 focused PARSE/TRACE controls, the recorded PARSE trace witness, formatting, clippy, release construction, and five three-way conversion probes all pass. [The round-2 report](task-8.8f-perf-fix-r2-report.md) retains exact commands and explains that generated ordinary-path packing is restored and duplicate rooting bypassed, but matched performance acceptance is still pending. The preserved new pin `/tmp/codex-p9d-t88-pins/8.8f-perf-r2-rexx` has SHA256 `22c317baa3ed111935a6577ac33097e428a53b64eb64b4697af2b85664e3f8eb`, with source/compiler/build provenance under `semantics/perf-fix-r2/release-pin.json`.
