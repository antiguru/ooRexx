# Task 8.8e1: Prerequisites, records and fixed rechecks

Status: DONE_WITH_CONCERNS. BASE is `dedfab2ea41aa419d993aca1d78bc1749a623620`, and final source is `1d8b03220`. The FIFO row remains open for its required dynamic EINTR/HALT witness and the controller's oracle-defect ruling. Full batch gates, performance, M6 timer proof and the original flakes remain controller-owned.

## Prerequisites and filesystem control

`rust/scripts/test_preflight.py` checks checkout-local native libraries, ooTest framework/population paths and reference XML before launching an optional command after `--`. Missing inputs produce absolute paths and explicit CMake/SVN setup commands, while present inputs permit Cargo to start. The documented entry in `rust/CLAUDE.md` requires this preflight before workspace gates and preserves actual ooTest revision validation in the full suite gate. The preflight checks input paths rather than certifying library contents, which the existing real native tests still load and validate.

The completed fresh enumeration under `t8.8/prerequisites/` is unchanged, including its default-population and feature limits. The new preflight was pointed at that frozen worktree through a private module-path control, refused before a mocked Cargo execution, and reported the observed native/source omissions. Empty, populated and missing-leaf fixture controls also pass, with a separate oracle-sibling control that cannot satisfy checkout-local prerequisites. The synthetic pool helper now supplies its library search path without canonicalizing a directory that its in-memory provider does not need.

The proposed native recipe was configured and built against unchanged C++ sources in `/tmp/codex-t88e1-native`, substituting that owned output path for `build`. CMake configure returned 0, and `cmake --build /tmp/codex-t88e1-native --target rxregexp rxmath orxmethod orxfunction rxsock -j4` returned 0. The configure used `-DCMAKE_POLICY_VERSION_MINIMUM=3.5 -DCMAKE_BUILD_TYPE=Release`, and no package installation or protected native-tree mutation was needed. `native-configure.*` and `native-build.*` retain complete commands, outputs, statuses and bounds.

The EXDEV fixture now supplies `Errno::XDEV` at the existing initial-rename result boundary and executes the production fallback on real files. It checks data, permissions, modification time, symlink preservation, source unlinking and a dangling target without needing another mount. The adjacent normal move control preserves the inode, refuses an existing regular destination and retains an ordinary rename-error control. Production behavior is unchanged apart from extracting that existing result-handling boundary, and the original fresh-enumeration failure remains historical evidence rather than being rewritten as a pass.

## Assigned dispositions

| Item | Disposition and evidence |
| --- | --- |
| POS and variable visitor pointers | Closed at `1d8b03220`. The concise POS rescan/NUL explanation is restored from pre-prune history, and expression visitors name `visit_expr`. |
| Table C literals | Closed at `1d8b03220`. The temporary Rust driver calls the existing derivation functions, regenerates 216 programs and asserts every change is the intended path literal. The normal Table C checks pass, and the README prefix is consistent. |
| 9b carry-in m1 | Closed for 9d. The new debug-profile unit witness repeats an identical miss without a write, then writes the package local and observes the value. Its repeated call reaches the debug assertion's memo-hit branch under the unchanged generation/write/name inputs. This is debug coverage, not a claim derived from release behavior. |
| Carry-in m3 | Existing correct prose retained. `objname-table.rex` uses an internal routine, whereas `objname4.rex:10,25-26` supplies the `::ROUTINE` row and its retained oracle output's eighth line is `long`. |
| Carry-in m4 | Existing reused-new-class witness retained and executed successfully. It first prints `XX`, then refuses the second subclass reuse under Deviation 32, separately from the `.Array`/`.Foo` did-not-build cases. |
| Other carry-in parts | m2, samples and SysSleep retain existing dispositions. N1 remains Phase 9.1, and the whole-process budget remains the close owner's work. |
| Gate 456 | Closed at `1d8b03220`. The comment now describes the phase-8/phase-9 corpus union and expected-output distinction, preserving its native-handle rooting limitation. |
| Gate 459 minors 5/7/9/10 | Closed at `1d8b03220`, together with the prior minor-8 evidence. Both the timeout argument and diagnostic preserve fractional seconds, cap labels derive from their constants, and plain tests read the recorded revision without executing SVN. `l3_core_suite` still calls `suite_revision`, including in both YAML runs. `Ended` already correctly documents exited, externally signaled and deadline-killed outcomes, so no documentation edit was invented. |
| Drained stdin and TRACE C | Closed as `probe agrees`. Independent durable stdout, stderr and status agree for the recorded EOF/live-pipe distinctions and original command-echo shape. |
| Named semaphores, SysFork, gate 9b1:121 | Closed as `probe agrees` for the ruled refusal labels. Event/mutex named create/open refuse with Deviation 43, and SysFork refuses with Deviation 42, all at status 120. |
| YAML and gate 488 | Closed as `probe agrees`. The reductions and actual TEST35_DIRECTIVES/TEST47_MULTILINEFLOWCOLLECTIONS pass, with the full group reporting 50/50 on each side for both crate-side class copies. |
| Class methods and gate 474 | Closed as `probe agrees`. The complete inherited/nil/shadowed name multisets preserve duplicates and agree, beyond their counts. |
| Gate 476 | Reachable REXX package copy/name/lookups/executable subpart closes as `probe agrees`. The native library-routine executable subpart remains Phase 10 Rxapi, explicitly recorded in `closed.tsv` and the inventory. |
| Gate 483 | Closed on the existing licence and S2 policy. The actual SysSleep row is Flaky, and WALL_CLOCK/`row_in_both_modes` retain their P48 once-rerun handling, separately from the hard no-auto-retry oracle deadline policy. |
| Gate 461 | Closed as no applicable production row. The durable exhaustive refusal-row derivation and current source hashes agree, and the real refusal-at-load fixture passes without fabricating a production row. |

The YAML diagnostic compiles a private copy of the current suite harness with one recorded `lay_class_files` source override. The oracle continues using its installed YAML class, while the crate is checked once with that class and once with the repository class. Both tables retain the named tests, actual suite revision and complete group population, with the override and source hashes in `yaml-harness-overlay.json` and `yaml-*/descriptor.json`. This diagnostic does not replace the controller's whole-suite gate.

The five historical inventory reconciliations preserve every reason and existing CLOSED marker. The trace rows use `fcb358fd7`, verified as the Task 6.10 closure-record commit rather than asserted as their implementation commit. The numeric rows use their existing `009fd1978` closure provenance. `inventory-file-state.json` finds no CLOSED-file/open-9d discrepancy, and `inventory-reasons-preserved.json` checks the identity, kind, probe and reason columns against BASE.

## FIFO and terminal boundary

The raw PTY control uses safe `os.openpty`, raw terminal settings and bounded children, reading `abc` and writing exact `xyz\n` with matching stdout, stderr and status. Ordinary FIFO reading and explicit stream closure retain the agreeing durable evidence. Closing LINEOUT differs without a state query and through the Stream method itself, so the old state-query hypothesis is ruled out. The oracle trace writes `hello\n`, fails a seek with ESPIPE, reads `h`, then closes with the remaining payload `ello\n`, while the crate preserves `hello\n`.

The trace and C++ `writeSetup`/`setPosition`/`checkEof`/`SysFile::hasData` path identify error-path byte consumption, not an intended LINEOUT rule. The exact syscall evidence is `fifo-oracle.trace`, with independent source, payload and descriptors under `fifo-strace.*`. The queued row remains partially open, and Deviation 46 is unchanged. Root owns the bounded read-only signal/trace probe against the frozen release pin that must demonstrate EINTR yields ERROR:4 with HALT handling before this required subpart can close.

## Verification and handoff

Commands run through `memcap 8G python3 /tmp/t88e1-check.py NAME BOUND CWD COMMAND...`, with Cargo `-j4`, `CARGO_INCREMENTAL=0` and `/tmp/codex-p9d-t88-target`. Every command has independent stdout, stderr and a JSON descriptor under `docs/superpowers/records/2026-10-18-phase-9d/t8.8/records/e1/`. Interpreter diagnostics use the recorded `memcap 2G timeout` or oracle wrapper, and all foreground checks completed within their bounds. The table includes unsuccessful controls and infrastructure corrections rather than hiding them.

| Record name | Command subject | Status and observed result |
| --- | --- | --- |
| native-configure / native-build | CMake recipe in owned output tree | 0 / 0 |
| deadline-red | New fractional deadline test before implementation | 101, fails after 1.009 seconds for a 1.7-second deadline |
| focused-unit | First debug compilation | 101, incorrect test-only `Errno::ACCES` spelling, corrected to `ACCESS` |
| preflight-controls | Python fixture controls | 0, 3 pass |
| unit-controls | Affected library population including move/miss controls | 0, 796 pass |
| additional-controls | PTY/FIFO and frozen-worktree preflight controls | 0, exact descriptors retained |
| fifo-cause | Bounded oracle syscall trace | 0, error-path consumption measured |
| suite-fixtures | Existing suite fixture population | 0, 19 pass and one existing diagnostic ignore |
| api-native | rexx-api load/invoke/context | 0, 41 pass |
| rederive | Existing Table C derivation functions | 0, intended path-only changes |
| generated-checks | Existing gate_table_c target | 0, 27 pass, report mode |
| carry-in-metaclass / synthetic-pool | Focused existing witnesses | 0 / 0, one pass each |
| yaml-both-copies | Initial private harness compile | 1, executable output collided with the linked suite directory, corrected |
| yaml-both-copies-r1 | Full YAML group with both class copies | 0, 50/50 each side and both named methods pass |
| deadline-final | Final precise deadline test | 0, one pass in 1.71 seconds |
| lint / lint-final | Affected-crate all-target clippy with `-D warnings` | 0 / 0, touched code rechecked |
| format / format-check / format-final | Formatter then checks | 0 / 0 / 0 |
| release-pin-build | Release rexx and rexxc | 0 |
| dispositions / handoff-provenance / record-audit | Descriptor consumption, inventory, artifact and current-policy checks | 0 / 0 / 0 |

The release pin is `/tmp/codex-p9d-t88-pins/8.8e1-rexx`, SHA-256 `434d6567458248fdb804eebe9edb715a54ddb9ded650fcca994eed6c4b691e06`. It was built at `7c6e12e84`, whose runtime sources are identical to final source `1d8b03220`, with the compiler pin and diagnostic executable hashes retained in `release-pin.json`. Source commits are `186cdfb3f`, `256a0bc9d`, `ae93af619`, `55f2a8234`, `7c6e12e84` and `1d8b03220`, followed by the records commit. No cache, protected oracle tree or user ZIP was removed, and no worker remains running.

Executable flags, failed worker spawn and gate 481 remain e2's work, and semantic residuals remain f's work. Gate 466 condition/routines/annotation costs and cumulative performance remain with the Phase 9 close owner. The M6 timer proof and both original flakes remain open without a licence or broader claim. Controller-owned full corpus, suite, workspace, policy and own-BASE performance gates still need to run for the joined batch.
