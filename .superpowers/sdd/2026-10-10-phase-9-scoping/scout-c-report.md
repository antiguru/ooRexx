# Scout C report: the four binaries Phase 9 ships

HEAD `a98238be5` (checked with `git rev-parse --short HEAD`). Crate side: `rust/target/release/rexx-run`,
rebuilt 21:50 at HEAD (`CARGO_INCREMENTAL=0 memcap 8G cargo build --release -p rexx-exec --bin rexx-run`,
`Finished`). Oracle: `/home/moritz/dev/repos/ooRexx/build/bin/*`, 5.3.0 r0, build date Jul 30 2026.
Probes ran from fresh directories `/tmp/claude-1000/p9scope/c/p1`, `p2` and the group copies `g_o`, `g_r`;
every oracle run under `ulimit -v 1048576` (4 GiB for a group run) and `timeout`, every crate run under `ulimit -v` and `timeout`.
`memcap` was used for the build only; the sandboxed interpreter runs used `ulimit -v` instead, which the
rules allow for the oracle and which I applied to the crate side as well. Say so if `memcap` is required there.
"Inferred" marks what was read, not run.

## Headline findings

1. **`rexxc -s` is not a syntax-check mode.** `-s` is "silent": it suppresses the version banner
   (`utilities/rexxc/platform/unix/RexxCompiler.cpp`: `silent = j`, then `if (!silent)` prints
   `RexxGetVersionInformation`). Syntax-check mode is "no output file given" (message 15:
   "Without outputfile rexxc only performs a syntax check."). `-e` is base64 encode.
   The brief's wording and any spec derived from it need correcting.
2. **The oracle `.rxo` is a raw flattened object heap, not a portable format.** Nothing in the crate can
   read it, and the oracle cannot read anything but its own. The ooTest groups that use compiled
   programs are self-consistent (rexxc writes, rexx or `.Routine~newFile`/`~new(name,data)` or the
   native `NewRoutine`/`LoadPackageFromData` read), so crate-private `.rxo` passes them. See section 2.
3. **`rxqueue` and `rxsubcom` are pure RXAPI clients.** Every operation after argument parsing is a
   `Rexx*Queue`/`Rexx*Subcom` call served by the daemon. Neither does anything a user can observe
   before D7 is executed, except usage errors and exit codes. See section 3.
4. **The embedding API does not exist in the crate and `rexx` does not need it to ship.** `rexx-api`
   is an rlib exporting no `Rexx*` symbol. The oracle `rexx` is built on four `librexx` imports plus
   the thread-context table, whose entries `rexx-api` already fills. `rexx` as a Rust binary can call
   `rexx_exec::run_program` directly; the embedding API (a cdylib named `librexx.so.4`) is a
   separate deliverable that only row 10's `INVOCATION`-style groups need. See section 4.
5. **Parse errors print unsubstituted messages at HEAD**, which `rexxc`'s whole output depends on:
   `Error 35.1:  Incorrect expression detected at "&1".` where the oracle prints `at "("`, and
   `Error 13.1:  Incorrect character in program "&1" ('&2'X).`. Both exit with the same rc as the oracle.
   `phase-4-exclusions.txt` records this as a deliberate deferral. `rexxc` and `rexx` cannot ship
   with that.

## 1. `rexx`

Source: `utilities/rexx/platform/unix/rexx.cpp` (242 lines, one `main`). Crate: `rust/crates/rexx-exec/src/bin/rexx-run.rs`.

### The oracle's command line (read, then probed)

* Options are recognised only while no program name is set (`program_name == NULL && argv[i][0]=='-'`).
  After the program name, `-v` is just an argument (probed: `rexx -e 'say 1' -v` prints `1`).
* `-e`/`-E`: next argv is the program text (`INSTORE`), run through `RexxStart` with `RXCOMMAND`.
  `-e` as the last word: falls through to the usage text, rc 255. `-e ""` is an empty program, rc 0
  (ooTest `test_optE1`).
* `-o`/`-O`, `-od`/`-OD`: next argv is a `::OPTIONS` string (prefixed with `::OPTIONS ` unless it
  starts with `:`), applied through `.Package~defaultOptions('D', ...)` and `('C', count)` before
  `CallProgram`. Probed: `-o "digits 9"` does not override a program's own `::options digits 5` (prints 5);
  `-o`/`-od "digits 9"` set digits 9 for a program with no `::options`. `-o` without a string: usage, rc 255.
  The crate already has `Package~defaultOptions` (`rexx-exec/src/dispatch/package.rs:112`).
* `-v`/`-V`: prints `RexxGetVersionInformation` (8 lines: `Open Object Rexx Version 5.3.0 r0`,
  `Build date: Jul 30 2026`, `Addressing mode: 64`, two Copyright lines, licence lines) to stdout, rc 0.
  It is honoured at any position before the program name (`rexx -? -v` rc 0, `rexx -v -?` rc 0).
* Any other `-x`: ignored, but `argc_base++` shifts `.SYSCARGS` (probed: `rexx -x t.rex a b` gives 2 entries).
* A bare `-` is an ignored option, not stdin. The oracle has no program-from-stdin mode: piping a program
  into `rexx` with no name prints the usage text.
* Usage (stderr, rc 255): blank line, `Syntax is "rexx [-o[d] "options"] filename [arguments]"`,
  `or        "rexx -e program_string [arguments]"`, `or        "rexx -v".`
* Arguments: every word after the program name is joined with single blanks into one string, and that
  one string is `ARG(1)` (`argCount` is 0 or 1). The words are also in `.LOCAL~SYSCARGS` as separate
  items, from `argv[argc_base..]`. Under `-e` `.SYSCARGS` is `.nil` (probed: `Error 97.1: Object "The NIL
  object" does not understand message "ITEMS"`) because that path uses `RexxStart`, not `CallProgram`.
* Program lookup (probed): the program name goes through the interpreter's resolver, not `open()`.
  `rexx t` finds `./t.rex`; `rexx viapath` finds `d2/viapath.rex` through `PATH`; through `REXX_PATH`;
  with or without the `.rex` extension. Not found is `Error 3.901: Failure during initialization:
  Program "nonexist" was not found.` on stderr, rc 253.
* Exit status: a raised condition is `-error number` (so `256 - major`, probed 214 for 42, 213 for 43, 221
  for 35, 243 for 13). Otherwise the program's `RETURN`/`EXIT` value through `ObjectToInt32`, else 0
  (probed: `return "abc"` rc 0, `return -3` rc 253, `exit 300` rc 44, `return 1.5` rc 0).
  Under `-e` the status is `rc ? rc : rexxrc`, same numbers (`-e 'exit 7'` rc 7, syntax error rc 221).
* `.rxo` loading: no special code in `rexx`. The resolver's restore path detects a compiled file by
  content (`#!` line, then `/**/@REXX`, then metadata), never by extension (probed: a `noext` copy of
  an `.rxo` runs). `PARSE SOURCE` names the `.rxo` file, not the source.

### What `rexx-run` has (probed against the oracle, same programs)

Agrees on: `ARG(1)` joining, `.SYSCARGS` count for plain runs, the exit statuses in the list above for
the six probed return shapes, error 42/43 text and rc, `PARSE SOURCE` (`LINUX COMMAND <abs path>`),
`ADDRESS()` (`sh`), `.rexxinfo` (name, languageLevel, version, platform, architecture), `ADDRESS system/command/""` and
an implicit command all running, and a child's exit status (255, 253, 129) reaching `RC`.
It also owns what the oracle's launcher does not: signal handlers, real-descriptor transient flags,
output sinks, `REXX_SWITCH_MODE`, and the wide interpreter stack (`run_program_on_stack`).

### Gaps (each probed on both sides unless marked read)

| Gap | Oracle | `rexx-run` |
|---|---|---|
| No program name | usage on stderr, rc 255 | `usage: rexx-run FILE [arguments]`, rc 2 |
| `-v` / `-V` | version banner, rc 0 | tries to open a file named `-v`, rc 2 |
| `-e CODE [args]` | runs CODE as INSTORE | tries to open file `-e`, rc 2 |
| `-o` / `-od` | package default options | not parsed |
| Unknown `-x` | ignored, shifts `.SYSCARGS` | opened as a file |
| Program lookup | extension search, `PATH`, `REXX_PATH`, cwd | `fs::read` of the literal path only. `rexx t` with `t.rex` present: `rexx-run: t: No such file or directory (os error 2)`, rc 2 |
| Program not found | `Error 3.901 ...`, rc 253 | `rexx-run: nonexist: No such file or directory (os error 2)`, rc 2 |
| Compiled program | restored and run | parsed as source: `Error 13.1: Incorrect character in program "&1" ('&2'X)` rc 243 |
| Parse-error message substitutions | `at "("` | `at "&1"` (`Error 35.1`), same rc 221 |
| `.SYSCARGS` under `-e` | `.nil` | n/a |
| Exit codes of the launcher's own failures | 255 (usage), 253 (not found) | 2 |

Lookup is not missing from the crate: `rexx-exec/src/require.rs` already implements the oracle's search
(`search_entries`, `candidates`, `DEFAULT_EXTENSIONS = [".REX", ".rex"]`, `REXX_PATH` then `PATH`) for
`::REQUIRES` and external calls. It is `pub(crate)`, and `rexx-run` does not call it (read).
The ooTest group `utilities/rexx/rexx_command.testGroup` measures this: with `rexx` linked to `rexx-run`
(the `group_runner.rs::interpreter_bin` shape) 30 tests run, 22 fail
(`TEST_NOARGS`, `TEST_OPTV1`, `TEST_OPTV2`, `TEST_OPTV5`, `TEST_OPTV6`, `TEST_UNKNOWNSW`, `TEST_OPTE1`..`TEST_OPTE11`,
`TEST_OPTENOCODE`, `TEST_OPTONOSTRING1`, `TEST_OPTONOSTRING2`, `TEST_PASSINGARGS5`, `TEST_PASSINGARGS7`),
85 assertions on the oracle, 30 on the crate. The oracle passes all 30. Command, per side:
`( ulimit -v 4194304; timeout 100 REXX testOORexx.rex -f GROUPFILE -U -V 2 )` from a scratch copy of
`ootest/framework`, `ootest/ooRexx/utilities/*`, `testOORexx.rex`, `worker.rex`, `ooTest.frm` (with the
two `rxfuncquery` tests removed, as `group_runner.rs::without_rxfuncquery` does), `rxregexp.cls`.
`TEST_PASSINGARGS5` and `TEST_PASSINGARGS7` were not read; they probably turn on `-o`/`-x` or `.SYSCARGS` (inferred).

### Blast radius of renaming `rexx-run`

Counts from `git grep` at HEAD (tracked files only; `.claude/worktrees/` and untracked trees excluded):

* `git grep -l "rexx-run" -- rust | wc -l`: 26 files.
* `git grep -l 'CARGO_BIN_EXE_rexx-run' -- rust`: 12 files, all in `rexx-exec/tests/`
  (`buffer_growth_error_exit`, `datetime_zone`, `input_oracle`, `library_routine_memory`, `method_bodies`,
  `package_requires`, `program_end`, `prompt_before_read`, `signals`, `sim_processes`, `stdin_contention`,
  `support/group_runner`). Cargo sets `CARGO_BIN_EXE_<name>` only for binaries of the same package as the
  test (inferred from cargo's documented behaviour, not run): **moving the binary into a new `rexx-cli` crate
  breaks all 12**, where renaming it inside `rexx-exec` only changes the string.
* Hard-coded paths: `rexx-bench/src/bin/rexx-bench-suite.rs:581,582,635` (`release/rexx-run`, `debug/rexx-run`),
  `rexx-bench/src/bin/rexx-bench-band.rs:143`, `rexx-exec/tests/support/arity.rs:374`,
  `rexx-bench/src/bin/rexx-arms.rs:15` (doc), `rust/bench-programs/wallclock.sh:13,64,65` (stages the binary
  as `stage/rexx-run`), `rust/bench-baselines/PINNED.md` (10 mentions).
* Prose: `rust/CLAUDE.md:38` ("Run `rexx-run` from `rust/`"), `rust/corpus/README.md:295`, `rust/corpus/phase-4c.txt`,
  doc comments in `rexx-exec/src/lib.rs:2718`, `rexx-bench/src/child.rs:50,58`.
* `git grep -l "rexx-run" -- docs ':!docs/superpowers/records' | wc -l`: 41 files. `git grep -l "rexx-run" -- docs/superpowers/records .superpowers | wc -l`: 846 files
  (historical records, not rewritten).
* The ooTest harness does not depend on the name: it symlinks `rexx` to the binary
  (`group_runner.rs::interpreter_bin`), so `argv[0]` is already `rexx` there.
* `rexx-ir` (`src/bin/rexx-ir.rs`) is a second binary in `rexx-exec`; not part of this phase.

## 2. `rexxc` and `.rxo`

### What the oracle does (read, probed)

`RexxCompiler.cpp` parses argv, then calls `RexxCompileProgram(input, output, NULL, encode)`
(`interpreter/api/InterpreterAPI.cpp:185`). That runs `TranslateDispatcher::run`
(`interpreter/concurrency/TranslateDispatcher.cpp`): resolve the name, `LanguageParser::createProgram`,
and if an output name is given, `program->save(output, encode)` (`RoutineClass.cpp:334`).

* Synopsis: `rexxc inputfile [outputfile] [-s] [-e]`. Argument 2 is the output file unless it starts with `-`.
  Remaining words must each be `-s`/`-S` or `-e`/`-E`, else message 12 plus the syntax lines, rc 255.
* Banner: the same 8 lines as `rexx -v`, to stdout, unless `-s`.
* Output equals input: `Output file name must be different from input file name.`, rc 254.
* Syntax error: the clause and `Error N running FILE line L: ...` / `Error N.S: ...` on **stderr**, banner on
  stdout. Probed rc 221 for 35.1, `-s` removes only the banner.
* No arguments: `SYNTAX: ...` two lines on **stdout**, rc 255. Not found: `Error 3.901` on stderr, rc 253.
* Exit status: `RexxCompileProgram`'s negated error number, truncated by the shell (`-3` is 253, `-35` is 221).
* The rexxc ooTest group asserts `rc -1`, `-2`, `-3`, `-13` for those cases, which the harness's
  `ADDRESS` mapping presents signed (the crate and the oracle both gave 255, 253 through `ADDRESS` in my probe,
  so the mapping is in the test helper or `assertRc`, not in the interpreter; not chased).

### File format (probed on `a.rex` = 25 bytes of source, `rexxc a.rex b.rxo -s`)

Layout, confirmed by `struct.unpack_from('<16sHHHHIIQQ', d, 20)` on the file:

* bytes 0-19: `#!/usr/bin/env rexx\n` (courtesy shebang).
* bytes 20-67: `ProgramMetaData` header, 48 bytes
  (`interpreter/classes/support/ProgramMetaData.hpp`): `fileTag[16]` = `/**/@REXX`, `magicNumber` 11111,
  `imageVersion` 43 (`METAVERSION`, bumped whenever the internal form changes), `wordSize` 64,
  `bigEndian` 0, `requiredLevel` 60600, reserved, pad, `imageSize` 2048.
* bytes 68-: `imageSize` bytes of the `Envelope::pack` output, a flattened graph of C++ objects.
  A 25-byte program gives 2116 bytes; the image holds absolute paths (`/tmp/.../a.rex` and its directory),
  so it is not even relocatable by directory.
* `-e`: `#!` line, then `/**/@REXX@\n`, then base64 of header+image in 72-column chunks.
* `restore` (`ProgramMetaData::processRestoreData`, `validate`): skips a leading `#!` line, accepts the
  binary or encoded tag, and refuses (`Error 3.903: Program "..." cannot be run by this version of the REXX
  interpreter`, rc 253, probed by zeroing the version byte) on magic/version/word size/endianness/language level mismatch.
  A truncated file (first 100 bytes) **segfaults the oracle**, rc 139 (probed). Add to
  `oorexx-oracle-crashing-probes` if that list is kept.
* Same source, two runs, different bytes: `cmp -l c.rxo d.rxo | wc -l` gives 63. The differing bytes sit
  in the first object words (`90 2d 27 60 19 7f`, a `0x7f19...` user-space address), so the image holds
  process-specific pointers that unflatten rewrites (inferred from the pointer-shaped bytes and from
  `restoreSavedPrimitiveBehaviour` in `memory/RexxMemory.cpp:1534`; I did not decode the image).

### Who consumes a compiled image

Besides the `rexx` launcher: `.Routine~newFile`, `.Routine~new(name, data)`, `::REQUIRES` and external
`CALL` (all through `LanguageParser.cpp:205,242,267,292`), `Method` file loading (`MethodClass.cpp:430`),
the native `NewRoutine` and `LoadPackageFromData`, and the macro space (`RexxActivation.cpp:3184`: a macro
function's stored image goes through `RoutineClass::restore`). **Restore is therefore interpreter
surface, not only the `rexxc` binary's.** `Routine~save` is not Rexx-visible (probed: the crate says
`Object "a Routine" does not understand message "SAVE"`; the oracle's `RoutineClass::save` is C++ only, read).

### What the crate has

* `git grep -n -w -i rexxc -- rust`: no implementation. Hits are comments recording rexxc measurements
  (rexx-parse tests, `corpus/README.md`, `CLAUDE.md`), and nothing named `save`, `restore`, `ProgramMetaData`
  or `Envelope` in `rust/crates/*/src` for compiled programs.
* `.routine~newFile("b.rxo")` and `rexx-run b.rxo` both treat the file as source and fail with
  `Error 13.1 ... "&1" ('&2'X)` at line 2 (the `/**/@` after the shebang), rc 243.
* `phase-4-exclusions.txt:5534-5542`: `TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA` (in `METHOD.testGroup`,
  which row 8 otherwise closes) "OWNER: Phase 9". Both sides fail it with rc 127 where `rexxc` is not on PATH.
  With the oracle's `bin` on PATH, both sides run the oracle's `rexxc` and the crate then refuses to load
  the file.
* The oracle's `rexxc` is also the parse-error gate's instrument:
  `rust/corpus/errors/parse-errors.tsv` (`grep -vc '^#'` gives 1021 lines including the header, so 1020 rows;
  `cut -f1 | sort | uniq -c`: 444 `-` (translates, rc 0), 567 `translation`, 9 `install`) holds, per
  program, the answer `rexxc FILE` gave: major.sub, the line of the MAIN message, or `ok`. The file is
  baked, not re-derived. Its header says the programs were collected by temporarily instrumenting the
  `ok`/`err` helpers in `rexx-parse`'s unit tests, then each was run through `build/bin/rexxc` (no output
  file, so the syntax-check mode) and recorded. No committed script reproduces that run:
  `rexx-parse/tests/errors.rs` only reads the file. `phase-4-exclusions.txt` (about line 5099-5113) records
  the re-run of all 1020 rows with zero disagreement on 2026-08-20 and says "Nobody owns" the live
  re-derivation. A Rust `rexxc` would let a gated test re-derive it without the C++ oracle, which is a
  scope decision for the spec, not a requirement.

### The two compatibility questions (D2)

1. **Can the crate read an oracle `.rxo`?** Not without reimplementing the oracle's C++ object layouts.
   The payload is `Envelope::pack`'s flattened heap of the oracle's own object types
   (`RexxInstruction` subclasses, `RexxVariableBase`, `RexxClause`, expression nodes), version-stamped
   `METAVERSION 43`, which the oracle bumps on any change to the internal form, holds process-specific
   pointers (63 differing bytes between two compiles), and embeds the source path. There is no
   specification of it. The crate's own AST is a different shape. Evidence for: none. Evidence against:
   all of the above, read from `ProgramMetaData.cpp`/`.hpp` and probed. Recommendation to the spec
   (judgement, not measurement): **no**, and state it as a documented non-goal with a clean refusal naming
   `Error 3.903` (the oracle's own text for "cannot be run by this version").
2. **Can the oracle read a crate `.rxo`?** Only if the crate writes bytes the oracle's `unflatten` accepts,
   which means emitting the oracle's object graph, vtable-compatible, for each construct. Same answer: no.
   One thing is compatible for free: the outer wrapper (shebang, `/**/@REXX`, the 48-byte header with a
   different `imageVersion`) makes the oracle answer `3.903` rather than crash, if the crate picks a
   `magicNumber`/`imageVersion` the oracle rejects. A wrong-but-plausible image that passes the oracle's
   `validate` would crash it (probed: a truncated image segfaults), so the crate's version stamp must
   fail the oracle's check on purpose.
3. **What does bear on the format choice:** (a) the ooTest `rexxc` group and `METHOD`'s `rexxc` test
   only need write-then-read within one implementation, plus the first 29 bytes
   (`#!/usr/bin/env rexx\n/**/@REXX`, which the group asserts byte for byte) and "compiled file is larger
   than the source, and the encoded one larger still"; (b) restore must accept `.routine~new(name, data)`
   in three shapes (source text, encoded text, binary) and the native `NewRoutine`/`LoadPackageFromData`;
   (c) the macro space stores images as opaque bytes and the daemon never parses them, so a crate-private
   image is compatible with the bridge-to-C++-rxapi reading of D7; (d) a source-embedding or
   re-parse-on-load image would satisfy (a) to (c) trivially but would not be "compiled", and the
   corpus's parse-error gate is about translation only. Which of those the spec wants is a design call.

## 3. `rxqueue` and `rxsubcom`

### `rxqueue` (`utilities/rxqueue/platform/unix/rxqueue.cpp`, 424 lines)

Filter: reads stdin line by line (LF, CR, CRLF, `0x1a` as end), adds each line to a queue with
`RexxAddQueue(name, &line, FIFO|LIFO)`. Options `/FIFO`, `/LIFO`, `/CLEAR` (also `-` prefix, any case),
one queue name; default queue is `$RXQUEUE`, else `SESSION`. First thing it does is `RexxQueryQueue(name, &n)`;
failure there goes to `options_error`. `/CLEAR` calls `RexxClearQueue`. Line buffer 65472 bytes; longer lines truncated.
Imports from `librexx`/`librexxapi` (via `nm -D bin/rxqueue`): `RexxAddQueue`, `RexxClearQueue`,
`RexxQueryQueue`, `RexxGetErrorMessage`. The three queue calls are RXAPI's (`librexxapi.so` exports `RexxAddQueue`,
`RexxClearQueue`, `RexxQueryQueue` per `nm -D lib/librexxapi.so`). `RexxGetErrorMessage` is the
interpreter's catalog (the crate has `rexx-inventory`).
Probed quirk: usage errors exit **0** (`rxqueue /bogus`, `rxqueue a b`, `rxqueue /fifo /lifo` all print
`The syntax of the command is incorrect.` and rc 0), because `options_error(0, ...)` ends with `exit(type)`.

### `rxsubcom` (`utilities/rxsubcom/platform/unix/rxsubcom.cpp`, 123 lines)

`rxsubcom REGISTER env dll proc` / `QUERY env [dll]` / `DROP env [dll]` / `LOAD env [dll]`, case-insensitive,
mapped to `RexxRegisterSubcomDll(..., RXSUBCOM_DROPPABLE)`, `RexxQuerySubcom`, `RexxDeregisterSubcom`,
`RexxLoadSubcom`, returning RXAPI's status as the exit code. Wrong argument count: the matching message
from the catalog on stdout, rc 255 (probed: `The RXSUBCOM parameters are incorrect.`,
`The RXSUBCOM REGISTER parameters are incorrect.`, `The RXSUBCOM QUERY parameters are incorrect.`).
A registration made by one `rxsubcom` process must be visible to another process (the QUERY after a REGISTER),
so the registry has to live outside the process.

### Can either ship before D7?

* D7 is REOPENED (`2026-07-27-rust-rewrite.md:117`), executed by Phase 10. `rxapi-protocol.md` (266 lines)
  documents the wire protocol; I did not re-read it for this report.
* **Without a shared registry, neither tool has an effect.** A standalone Rust `rxqueue` could parse its
  options, print its errors with the right exit codes, and read stdin, but its queue write has no
  destination: the crate's session queue is process-local (probed: `queue`, `queued()`, `parse pull`
  work in-process on both sides; `RXQUEUE()` is refused `routine "RXQUEUE" is not implemented (Phase 10)`).
  Probed on the oracle: `"echo piped | rxqueue"` from a `rexx` program makes `queued()` answer 1, so the
  child and the parent share the SESSION queue, which is the daemon's.
* What can ship before D7: argument parsing, usage text and exit codes of both tools (testable against the
  oracle with no daemon), as a front-end that calls a `trait` or a stub that fails like `RXAPI not available`.
  That is a decision for the spec: it ships text, not function.
* ooTest groups: `utilities/rxqueue/rxQueue.testGroup` (9 tests: `cmd | rxqueue` then `queued()`/`pull`,
  `rexx prg | rxqueue`, plus `rxqueue_environment_variable` and `rxqueue_maximum_line_length` fixtures),
  `utilities/rxsubcom/rxsubcom.testGroup` (13 tests), `utilities/rxapi.testGroup` (1 test).
  Against the oracle all four utilities groups pass (rexx_command 30/30, rexxc 11/11, rxQueue 9/9,
  rxsubcom 13/13; the oracle talks to an `rxapi` daemon already running on this machine, `pgrep -a rxapi`).
  On the crate side with only `rexx` linked: `rxsubcom` 7 failures of 13 (6 pass because they expect rc 127
  or similar from a missing program, inferred, not read per test); `rxQueue` exits 120 on `RXQUEUE`,
  `rexxc` 8 errors of 11 with rc 127 (`rexxc` not on PATH). `l1-coverage.md:452-456` lists the five
  `utilities/` groups at 0%, 0%, 100%, 66.7% and 0% L1 coverage (`rexx_command`, `rexxc`, `rxapi`, `rxQueue`, `rxsubcom`).
* `rexximage` (`utilities/rexximage`) is also in the oracle build and in the plan's table; row 9 does not list it.
  `ootest/ooRexx/utilities/rexximage/` is empty. Not surveyed further.

## 4. Embedding API

* `RexxCreateInterpreter` and `RexxStart` appear nowhere in `rust/crates/*/src`
  (`grep -rn "RexxCreateInterpreter\|RexxStart" rust/crates/*/src`: the only hits are comments citing `RexxStartDispatcher.cpp` in `rexx-exec`, no definition).
  `rexx-api` has no `[lib] crate-type` (`grep -n "crate-type\|\[lib\]" rust/crates/rexx-api/Cargo.toml`: none),
  so it is an rlib, and `git grep -n "cdylib\|staticlib" -- rust '*.toml'` finds nothing. There is no `librexx.so`.
  `phase-8-surface.md` calls the SONAME question ("a `librexx.so`") Phase 9's.
* What `rexx-api` does export: the function tables handed to native code during a native call
  (`RexxThreadContext_`, `RexxCallContext_`, `RexxMethodContext_`, `RexxExitContext_`, `RexxIORedirectorContext_`,
  `RexxInstance_`), `ffi.rs`, plus the outbound loader in `load.rs`. Contexts exist only wrapping an active
  native call; `AttachThread` outside one is refused (`ffi.rs:4187`: "not implemented (Phase 9)"), and so are
  `RexxInstanceInterface.Terminate`, `.Halt`, `.SetTrace`, `RexxThreadInterface.HaltThread`, `.SetThreadTrace`
  (`layout.rs:379-383`, `REFUSING_MEMBERS`, whose owner is "Phase 9").
* Oracle `rexx` main uses (`nm -D bin/rexx`): `RexxCreateInterpreter`, `RexxStart` (for `-e` only),
  `RexxGetVersionInformation`, `RexxFreeMemory`; and through the thread context `NewArray`, `ArrayPut`,
  `NewStringFromAsciiz`, `GetLocalEnvironment`, `DirectoryPut`, `FindClass`, `SendMessage2`, `CString`,
  `Int64ToObject`, `CheckCondition`, `CallProgram`, `DisplayCondition`, `ObjectToInt32`, and the instance's
  `Terminate`. `grep -c` finds each of `CallProgram`, `DisplayCondition`, `CheckCondition`, `GetLocalEnvironment`,
  `FindClass`, `SendMessage2`, `NewStringFromAsciiz`, `Int64ToObject`, `ObjectToInt32`, `DirectoryPut`, `ArrayPut`,
  `NewArray`, `CString` in `rexx-api/src/ffi.rs`; only `Terminate` is a refusal.
* `rexxc` uses `RexxCompileProgram`, `RexxGetErrorMessage`, `RexxGetVersionInformation`, `RexxFreeMemory`;
  `rxqueue`/`rxsubcom` use the RXAPI calls above. `librexx.so.4` exports 16 `Rexx*` functions
  (`nm -D lib/librexx.so.4`: `RexxCompileProgram`, `RexxCreateInterpreter`, `RexxCreateInterpreterImage`,
  `RexxDidRexxTerminate`, `RexxGetErrorMessage`, `RexxGetErrorMessageByNumber`, `RexxGetVersionInformation`,
  `RexxResetTrace`, `RexxSetHalt`, `RexxSetTrace`, `RexxStart`, `RexxStemSort`, `RexxTranslateInstoreProgram`,
  `RexxTranslateProgram`, `RexxVariablePool`, `RexxWaitForTermination`).
* Consequence for the spec: **`rexx` can be built on `rexx_exec::run_program` plus a front-end and need not
  go through an embedding API.** If the spec wants the oracle's shape (`rexx` as a client of the embedding
  API, so the API is exercised by the binary), then `rexx-cli` needs a `RexxCreateInterpreter`-equivalent that
  returns an instance and a first thread context outside a native call, which `rexx-api` does not have.
  Row 10's groups (`INVOCATION`, `RexxStart`, `ProcessInvocation`, and the `orxexits` import chain) are what
  require the exported C symbols; row 9 does not.
* The Phase 9 refusals that remain, by command: `grep -rn '"Phase 9"' rust/crates/*/src` gives the five
  `REFUSING_MEMBERS` rows and `rexx-exec/src/lib.rs:612`; `grep -n "OWNER: Phase 9"
  docs/superpowers/plans/phase-4-exclusions.txt` gives lines 5284 (a second interpreter instance: the unloader and
  termination UNINIT run on a separate instance in the oracle), 5542 (`rexxc`), 6114 (conditions and stack frames under
  INTERPRET), 6179 (benchmark and resource parity).

## 5. Rexx-level surface tied to the binaries (probed on both sides)

| Item | Oracle | `rexx-run` |
|---|---|---|
| `PARSE SOURCE` | `LINUX COMMAND /abs/path.rex` | same |
| `PARSE SOURCE` under `-e` | `LINUX COMMAND INSTORE` | n/a |
| `PARSE SOURCE` of a compiled file | names the `.rxo` | n/a |
| `ADDRESS()` default | `sh` | `sh` |
| `.rexxinfo~name` etc. | `REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026`, `6.06`, `5.3.0`, `LINUX`, `64` | same |
| `ADDRESS system` / `command` / `""` / implicit | all run through `sh` | same |
| `.SYSCARGS` | argv words after the name (`argv[argc_base..]`), `.nil` under `-e` | 3 items for `t.rex a b "c d"` (agrees for plain runs) |
| `#!` first line | ignored | ignored (agrees) |
| `rexx -e` PARSE SOURCE / `.SYSCARGS` | `INSTORE` / `.nil` | none |
| `RXQUEUE` BIF | works | refused, Phase 10 |
| rexxc-compiled input | restored | `Error 13.1` |

The version banner (`rexx -v`, `rexxc` banner) has no source in the crate: the 5.3.0 / `r0` / `Jul 30 2026`
build date / `Addressing mode: 64` lines must come from somewhere the crate controls. `PARSE VERSION` already
answers `REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026` (same on both sides in the probe), so the date is
already fixed in the crate; the banner's `r0` revision and the copyright years are not (inferred, not grepped).

## Cheap probes the spec can reuse

All under `/tmp/claude-1000/p9scope/c/` (not in the tree): `p2/` holds `t.rex` (ARG, `.SYSCARGS`), `q.rex`
(queue), `rc.rex` (child exit status), `opt.rex`/`opt2.rex` (`-o`/`-od`), `sh.rex` (shebang). The group
copies `g_o`, `g_r` and the `bin_r/rexx` symlink reproduce the ooTest runs above.
