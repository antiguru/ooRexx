# Task 6 report: AddCommandEnvironment, the exit and IO-redirector contexts

Base: `362fd648c` (records only past `7eeb77846`). Scratch:
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-6/` (below, `$S`).

## Status

Code, witnesses, controls and Miri done; commit and gates below.

## Commits

* `949b15f84` Fill AddCommandEnvironment, the exit context and the I/O redirector
* `49336464d` Record Task 6's forge and the divergences it leaves

## Design

* **Registration.** `RexxInstanceInterface.AddCommandEnvironment` is filled (`ffi::add_command_environment`):
  it steps from the instance to its `Thread` as `attach_thread` does, refuses another thread
  (Phase 9), and hands the host a `load::CommandHandler` through the new `Surface::add_command_handler`.
  A handler of a type the header does not define, a null handler or a null name registers nothing
  (`InterpreterInstanceStubs.cpp:89-101` ignores an unknown type too). The call and method contexts'
  `AddCommandEnvironment` are header wrappers over the thread context's instance, so nothing more is
  needed for them (the derived reach test shows orxfunction reaching the instance member).
* **Phase 7's handler table.** `command.rs`'s built-in names are a static match (`handler_for`), so
  the registered handlers sit beside it: `Interp::command_handlers`, by upper-cased name, consulted
  first in `run_command` (a registration replaces a built-in name, as `commandHandlers->put` does;
  measured: `bash` answers the handler). `CommandHandler` holds a raw entry point tied to no mapping,
  so the table is emptied at the first library close (`run_package_unloaders`); that is the invariant
  `CommandHandler::call`'s SAFETY note rests on, witnessed by
  `a_registered_handler_answers_until_a_library_closes`.
* **The call.** `invoke::command` registers the address and command strings as locals, hands the
  handler `Contexts::exit()` (a new `Owned<RexxExitContext_, Activation>`) and, for a redirecting one,
  a `RedirectorContext` over a `redirect::Redirector`; `CommandHandler::call` (load.rs) catches the
  Throw marker as `call_stub` does. `Interp::run_command_handler` runs it in a native frame of its
  own. RC is the returned object as is (`.true`/`.false` identity measured), `.false` for none;
  status Normal unless the handler raised ERROR or FAILURE.
* **Exit context.** `EXIT_CONTEXT`: the call context's variable members and `GetCallerContext`,
  made generic over a `ContextVariables` trait (call, exit); the Throw members take the C-unwind
  marker path Task 5 built (`CallLinked` for `RexxExitContext_`). No exit-context member refuses.
* **Conditions from a handler.** A SYNTAX (raised or thrown) is the command clause's error, after the
  output reached its targets (measured `redirect_raise.rex`). Any other is raised under its own name
  with the handler's description, ADDITIONAL and RESULT; RC is the RESULT (else the answer), and an
  ERROR or FAILURE carries an RC entry even with no RESULT (`.nil`, new `Interp::pending_rc`);
  untrapped FAILURE becomes ERROR; `::OPTIONS ... SYNTAX` substitutes (description, RESULT).
* **I/O redirector.** `redirect::Redirector` (rexx-api, safe code): input lines gathered before the
  call, NUL-terminated and never moved; `ReadInputBuffer`'s buffer made once from the unread lines;
  output and error sinks porting `OutputRedirector::writeBuffer`/`flushBuffer`/`scanLine` exactly;
  one sink for both where output and error share a target. `IoContext` now keeps input as lines
  (`input_bytes` for the shell), builds the `Redirector`, and `finish_lines` writes the lines through
  Phase 7's target writers. Writes land when the handler returns (a recorded divergence).
* **Exit registration** (`REGISTERED_EXITS`, `DIRECT_EXITS`, `RexxRegisterExitExe`) is not built:
  nothing in this crate registers exits, and no table member reaches them.

## Witnesses

* STRICT corpus, through the oracle's prebuilt orxfunction: `rust/corpus/lang/library_command_environments.rex`
  (direct and redirecting `TestAddCommandEnvironment`, all rHandler flag combinations, 98.921 from a
  local and a standing `WITH`, the io predicates with `.true`/`.false` identity, replacement of a
  registered and of a built-in name) and `library_command_redirect.rex` (ioHandler over stems replaced
  and appended, arrays, an output stream object, a file read and rewritten in place, one target for
  both streams, every buffer shape of FUNCTION's `test_write_buffer`, `BUFFERINPUT`, a standing
  `WITH` and its overrides). `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --release --test corpus`:
  619 of 619, exit 0 (`$S/corpus1.txt`).
* Forge (`docs/superpowers/records/2026-09-14-phase-8-surface/task-6-forge/`, `cmd.cpp`, `build.sh`
  output NEEDED libgcc_s.so.1, libc.so.6, ld-linux-x86-64.so.2, no undefined Rexx symbol;
  `compare.sh`): `exitvars` (Set/Get/Drop/GetAll context variables, GetCallerContext,
  GetContextVariableReference, from main and a procedure), `returns` (NULL, a number, an array, the
  RC(n) trace line, an undefined type, a replaced built-in and a direct replaced by a redirecting
  one), `raises` and `raises2`/`raises2.cls` (ERROR, FAILURE, USER, with and without RESULT, CALL ON,
  SIGNAL ON, FAILURE renamed ERROR, both `::OPTIONS` escalations, a SYNTAX), `throws` (every exit
  Throw member, each with a destructor that ran), `redirect` (each stream through every redirector
  member, both buffered members, one target, no WITH, output alone, empty input), `redirect_raise`
  (write then RaiseCondition, RaiseException, ThrowException1; a standing WITH on a direct handler):
  all identical on stdout, stderr and rc.
* Recorded, not matched: `address_trace.rex` and `stream_state.rex` (both pre-existing, see
  Divergences).

## Negative controls

Each prediction written before its run; each mutated file restored from a copy and checked with `cmp`.

| # | Mutation | Prediction | Result |
|---|----------|------------|--------|
| C1 | `Redirector::sink` ignores `shared` (an error write under one target goes to the absent error sink) | STRICT red on `library_command_redirect.rex` alone (`both same` loses `l2`); `library_command_environments.rex` green (its flags read `same_target`, not the sink) | as predicted: 618 of 619, `library_command_redirect.rex` alone (`both same 3 2 l1 l3 S.3`); restored, `cmp` clean |
| C2 | `Sink::write_buffer` drops the branch joining a `\r\n` split across two buffers | `redirect::tests::buffers_split_into_the_oracles_lines` and `ffi::tests::a_redirecting_handler_reads_and_writes_each_stream` red; STRICT red on `library_command_redirect.rex` alone | as predicted: those two lib tests red (60 passed, 2 failed); STRICT 618 of 619, `library_command_redirect.rex` alone; restored |
| C3 | `run_command` consults the registered handlers only after the built-in names | STRICT red on `library_command_environments.rex` alone (`bash` runs a shell: rc 3, `.RS` 1) | as predicted: 618 of 619, `library_command_environments.rex` alone; restored |
| C4 | `run_registered_command` skips the 98.921 check | STRICT red on `library_command_environments.rex` alone (`direct with`, `upper`) | as predicted: 618 of 619, `library_command_environments.rex` alone (`direct -1 0`, then `not here`); restored |
| C5 | `run_package_unloaders` does not clear the handler table on a close | `a_registered_handler_answers_until_a_library_closes` red at its last assertion; everything else green | as predicted: `rexx-exec --lib` 855 passed, 1 failed, that test at `tests.rs:880`, the table assertion; restored |
| C6 | a handler condition's `RC` is the handler's answer, not the condition's `RESULT` | forge `raises.rex` differs (`error untrapped ret` for `res`); STRICT corpus green, a blind spot: orxfunction's handlers raise nothing | as predicted: `raises.rex` stdout differs (`error untrapped ret 1` for `res 1`); STRICT 619 of 619, the blind spot; restored |
| C7 | `EXIT_CONTEXT.ThrowException0` left as the refusing stub | `a_populated_table_refuses_exactly_the_members_it_names` red; `ffi::tests::an_exit_context_throw_leaves_the_handler` aborts the lib test binary (`abort_now`) | **partly falsified**: the lib binary aborts (SIGABRT) and `a_populated_table_refuses_exactly_the_members_it_names` is red as predicted, and `the_test_extensions_reach_only_members_that_answer` is red too: its derivation maps orxfunction's `ThrowException0` calls (made on call contexts) onto every wrapper defining that name, the exit context included, an over-approximation; restored |
| C8 | `IsRedirectionRequested` answers true with no `WITH` | STRICT red on `library_command_environments.rex` alone (`none` 10000, `not requested`) | as predicted: 618 of 619, `library_command_environments.rex` alone; restored |
| C9 | `CommandHandler::call` resumes the Throw marker instead of catching it | forge `throws.rex` differs: the unwind reaches the interpreter thread, which panics | as predicted: `throws.rex` differs, the crate at rc 101 with stdout and stderr empty (`resume_unwind` runs no panic hook); restored |

## Miri

`RUSTUP_HOME=<surface-4>/rustup-home CARGO_TARGET_DIR=$S/target-miri cargo +nightly miri test -p rexx-api --lib --offline`,
Stacked Borrows (no `MIRIFLAGS`), on the tree committed below: exit 0, 54 passed, 0 failed, 8 ignored
(`$S/miri-1.txt`). The new tests run under it: `a_command_environment_registers_a_handler_of_either_type`
(the instance slot), `a_direct_handler_binds_the_callers_variable_and_answers` (every exit-context
variable member and `GetCallerContext`), `an_exit_context_throw_leaves_the_handler` (the exit Throw
marker through a Rust `C-unwind` handler), `a_redirecting_handler_reads_and_writes_each_stream`
(every redirector member), and the `redirect::tests`.

## Gates

`$S/gates.sh` (Task 5's pattern, G3/G5 `test --no-run` outside the cap), status `$S/gates/status.txt`,
started at `49336464d`. Results below only once read from that file.

## Divergences recorded

In `docs/superpowers/plans/phase-4-exclusions.txt`, block "DIVERGENCES THE SURFACE PLAN'S TASK 6
LEAVES", none owned by Phase 8:

* a redirecting handler's lines reach their targets when it returns, and its input is read before
  it starts (read, not measured; owner none);
* a null handler registers nothing (the oracle would register it and call address 0; not run;
  owner none);
* no registered handler survives the first library close (not measured; owner none);
* `::OPTIONS ERROR|FAILURE SYNTAX` over a handler's RaiseCondition escalates when the handler
  returns, not inside the callback (error, message and RC agree; owner none);
* found, not made, by this task: a USING input stream object with no STATE method is 97.1
  (`stream_state.rex`; Phase 7's reader; owner none assigned);
* found, not made: `address env 'cmd'` echoes no clause line under TRACE C (`address_trace.rex`;
  owner none assigned).

## Concerns

* **FUNCTION's `test_input_output_stream` fails** on the stream-object reader above, a Phase 7 path
  this task did not change; the conservative option was to record it, not to rewrite that reader.
  `test_rexxqueue` still meets the RexxQueue refusal (Phase 10). Every other `AddCommandEnvironment`
  and I/O test in the group has its shape in the two corpus witnesses.
* **Two pre-existing gaps found on the way**, recorded above with no owner because the phase that
  built each has closed: the ADDRESS TRACE C echo, and the STATE reader. A third, not recorded as a
  divergence because it is a loud refusal: a condition object built by the interpreter
  (`build_condition_object`) refuses `ITEMS` and `HASINDEX` naming Phase 5 (`call on error; 'exit 3'`
  then `condition('o')~items`, measured, rc 120); the forge's `raises2.rex` reads `c~rc` instead.
* **The reach test over-approximates** (control C7): a member name orxfunction calls on one context
  counts for every wrapper defining that name. It makes the test stricter, not blind.
* The exit context's `GetCallerContext` refusal, on a host with no surface, names
  `CallContextInterface.GetCallerContext`; only the test host can have no surface.

## Controller addendum (fix after the gates, 2026-09-26)

The first gate run at `49336464d` failed two tests in G4 and G6, and passed the rest (corpus 619/619). The implementer stalled, so the controller made the fix, `b06b278ce`:
- `refusal-sites.tsv`, re-derived with `REXX_REFUSAL_SITES_REFRESH`. Existing rows change column 4 only. There are two new rows, `escalated` (`command.rs:865`) and `redirection_not_supported` (`error.rs:1581`), both `body`/`off-send-surface`.
- The sourceline expectations for `library_command_environments` and `library_command_redirect`, recorded from the oracle with the module comment's procedure. The counts equal `wc -l`.
The re-run gates are at `scratchpad/surface-6/gates/status.txt`.
