# Phase 9c plan review

Plan: `docs/superpowers/plans/2026-10-15-phase-9c.md` at `7d6050d6a`. Authority: the spec at
`4ab4093b2`. Read-only review; the oracle probes below ran from fresh directories under
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/review9c/`,
each as `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
/home/moritz/dev/repos/ooRexx/build/bin/rexx ... )`.

**Verdict: revise before dispatch.** One Critical (Task 7 is planned to a budget it cannot meet, and
the plan finds that out at the close), five Important. The spec coverage is complete, every other
file:line I opened matches the tree, and the task ordering is sound.

## Critical

### C1. Task 7's design cannot fit the +0.5% budget, and the plan measures it only at the close

**Where.** Task 7 Step 2 ("Write per descriptor as the oracle does, not more often") and Step 3
("The close's callgrind judges the cost").

**Evidence.**

* What the oracle does, measured here: `say 'started X'; do forever; end` and
  `.error~say('started X'); do forever; end` under `timeout -s KILL 2`, with the descriptor on a
  file and on a pipe, keep `started X` in every one of the four shapes;
  `do i = 1 to 300; say 'line' i; end; do forever; end` keeps 300 of 300 lines on a pipe
  (`| wc -l`) and on a file. So the oracle issues a write per `say` line. Matching it means a
  syscall per `say`.
* The budget per `say`: `bench-programs/sayloop.rex` is 100,000 `say`s and its own comment measures
  304 instructions per SAY; the 9b1 close's whole-process figure for `sayloop` is 179,667,649 Ir
  (`docs/superpowers/records/2026-10-14-phase-9b1/close/callgrind.txt`). 0.5% of that is
  898,338 Ir, which is 9 Ir per `say`. A `write(2)` through glibc costs more than that in user
  space before Rust's `Stdout` lock and `LineWriter` are counted (`rexx-run.rs:49-57` does
  `write_all` then `flush` per hand-over). So no design that writes once per `say` is inside the
  budget, and 6.1's three fix rounds cannot tune it in.
* The queued item (`.superpowers/sdd/queued/2026-10-11-output-lost-on-kill.md`) makes the budget a
  precondition: "The cost of flushing per write must be measured against section 8's budget." The
  spec's 9c row (section 9) does not list this item; the plan takes it as decided.

**Why it matters.** As written, Task 7 commits, Tasks 8 and 9 stack on it, and Task 10's callgrind
is the first thing that says OVER on `sayloop`.

**Fix.** Add a "For Moritz 2" before dispatch with the arithmetic above and the choices: a budget
exception for `sayloop`; a design that is not a write per `say` (a bounded flush off the `say`
path, for example a writer thread draining a ring the `say` path fills, so a kill loses at most one
tick); or Task 7 leaves 9c and the queued item is re-owned. Whichever is chosen, Task 7 measures
`bench-programs/callgrind.sh -B 0.5 -p sayloop` against the pin at the task (well under five
minutes), not at the close. The Step 3 test shapes stay valid under a tick design because the kill
comes 2 s after the write.

## Important

### I1. The Task 5 A/B's threshold is against the wrong baseline (and the answer to For Moritz 1)

**Where.** Task 5 Step 5: `callgrind.sh -B 0.5 ... base=<rexx-run copy> head=<rexx copy>`, "An
`OVER` row is a regression".

**Evidence.** The budget is `rexx` at the close against the pin (spec section 8), and 9b1 already
spent part of it: at the 9b1 close `startup` is +0.3346%, `interpret` +0.2915%, `sayloop` +0.2416%
against the pin (`close/callgrind.txt`). A front end costing +0.3% on `startup` passes the A/B as
planned and fails the close. The A/B as the spec means it ("so that the front end's share of the
close figure is known", "It sits inside the budget") needs the pin in the table.

**Fix.** One run with the pin first, from the repo's `rust/`:
`bench-programs/callgrind.sh -B 0.5 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9c/ab pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run base=<rexx-run copy> head=<rexx copy>`.
`callgrind.sh` takes any number of `NAME=BINARY` and judges every delta against the first binary
(`callgrind.sh:5-20`), so `head`'s column is the budget question and `head` minus `base` is the
front end's share. Judge OVER on `head` against the pin.

**For Moritz 1.** The A/B at Task 5, outside the per-task budget, is sound: the two binaries exist
only in that commit, the spec requires it, and the run is from the repo's `rust/` (the 9b1 final
review's M5 reason). With the pin as the first binary it also answers the close question early.

### I2. Spec criterion 4's REGISTERED test is missing

**Where.** Task 8 Step 3: "The `REGISTERED` rows are exempt per spec criterion 4".

**Evidence.** Spec section 10 criterion 4: the rows are exempt "selected by a predicate that reads
the program, and a test asserts the refusal text carries the `Rxapi` reason." The plan has the
exemption and neither the predicate's shape nor the assertion. The rows are `parse-errors.tsv`
lines 543, 689, 920 and 963 (`grep -n REGISTERED`).

**Fix.** Task 8 Step 3 gains: the exemption predicate reads the program for `REGISTERED`, and a
test runs one exempt row through `CARGO_BIN_EXE_rexxc` and asserts its refusal text carries the
`Rxapi` reason (D3's constructor), so the exemption goes red when Phase 10 implements it.

### I3. Task 9's enumeration of the oracle's restore calls is wrong by grep, and misses a route

**Where.** Task 9 Step 2: "`parser/LanguageParser.cpp:102`, `:199`, `:242`, `:267`, `:292`;
`classes/MethodClass.cpp:430`".

**Evidence.** `grep -n 'Class::restore(' interpreter/parser/LanguageParser.cpp` in the oracle tree
and in this repo's `interpreter/` copy both give 108, 166, 205, 242, 267, 292, 324, 378 (and 4349,
4394, the `RexxStart` instore path, Phase 10's). Lines 102 and 199 are comment lines. The uncited
calls: `:166` is `createMethod(name, ctx)` from a file (`.Method~newFile`, which the prose lists);
`:324` is `createProgram(name, ArrayClass*, ctx)` with the needle check, reached from
`createPackage(name, ArrayClass*, ctx)` at `:398`, whose caller is `PackageClass.cpp:228`, so
`.Package~new(name, array)` restores an encoded image; `:378` is `createProgramFromFile`, which is
the `rexx FILE` route (`RexxStartDispatcher.cpp:132`, `:237`) and the external `CALL` route
(`RexxActivation.cpp:3138`). Measured: `.Package~new('p', <encoded file as one string>)` does
not restore (the string is one line, `sourceSize` 1, no routine), so only the array form is a
route there. `MethodClass.cpp:430` is right.

**Fix.** Replace the line list with the grep and its output, add `.Package~new(name, array)` to
Step 2 and to `tests/compiled.rs`'s per-route programs (the site is `install.rs:986`
`package_from_source`, already in the Files list), or state that it is out and why.

### I4. Review Focus 4 cannot be met through `run_program(path: &str, ...)`

**Where.** Review Focus 4 ("reported in messages byte for byte", "Task 5 pins `rexx`"), Task 5
Step 2 ("Arguments are taken as bytes (`OsStrExt`), never through a lossy conversion").

**Evidence.** `rexx_exec::run_program(path: &str, text, invocation)` (`lib.rs:3465`) and
`execute_on(path: &str, ...)` (`:3702`) carry the program name as `&str`; `rexx-run.rs:109` gets
there by `to_string_lossy()`. A name holding `'ff'x` reaches `PARSE SOURCE` and the
`Error N running <path>` line altered. Task 5's Produces list has no byte-path entry, and
`grep -rln 'run_program(' rust/crates/*/src rust/crates/*/tests | wc -l` is 54 files, so widening
the signature is not a Task 5 edit and would land as a "Shared paths" report mid-task.

**Fix.** Task 5 Produces a byte-named entry (`run_program` over `&[u8]` with the `&str` one
delegating to it, or `execute_on` taking bytes), and names `lib.rs:3465` and `:3702` in its Files.
The same applies to Task 8's `launcher::translate(path: &Path, ...)`, which is already by path.

### I5. The rendered message's type is unstated, and the existing one is lossy

**Where.** Task 2 Produces ("`ParseError::message()` renders the sub-message with its inserts"),
Task 1 Step 3 (the comparison), Task 4 Step 4 (the non-UTF-8 insert row).

**Evidence.** `ParseError::message()` returns `String` (`rexx-parse/src/error.rs:22`). The oracle's
13.1 for an `'80'x` byte prints the raw byte inside the quotes; the TSV's `message` column escapes
it as `\x80` and the reader decodes to bytes. A `String` cannot hold the oracle's bytes, so the
Task 1 test compares bytes against something that has already been converted, and the Task 4 row
either fails or is compared lossily. `errors.rs:578-630` asserts `message()` against `&str`
literals today.

**Fix.** Task 2 states that the rendering is bytes (`Vec<u8>`, or a byte-returning method beside a
`String` convenience), and that the `errors.rs` asserts move to byte literals. Task 1's test
compares bytes.

## Minor

* **M1. Task 4 Files: `src/lib.rs:96`.** The program site is `lib.rs:201`
  (`ParseError::new(99, 914, clause.span.start)`); line 96 is `parse_program`'s body.
* **M2. `derive.sh` and `printf '%b'`.** `\xNN` is bash's; dash's `printf` does not decode it, and
  `/bin/sh` here is dash. The script's first line should be `#!/usr/bin/env bash`. Anchor the
  stderr reads at line start (`^Error `): a clause echo is indented, a program line is not.
* **M3. `UNREACHED_BY_FILE` reasons.** Only the INTERPRET reason is checked (Task 4 Step 3). Any
  other reason is a string nothing re-runs (`gate-criteria-failure-modes`). Make the reason a
  closed enum, each variant with its check: INTERPRET-only through
  `the_interpret_only_errors_match_the_condition_objects_own_code`; anything else carries the
  oracle raise site and the probes tried, as the GUARD rows of `refusal-dispositions.tsv` do.
* **M4. Task 8 Files omit the test it inverts.** `suite.rs:1883`
  (`the_crate_side_finds_its_own_rexx_and_no_rexxc`) asserts `resolve("rexxc") == None` and, at
  `:1913-1916`, that a copied `rexxc` makes `check_path` fail. Task 8 turns both around; name the
  test. (Task 5's "update the test at `suite.rs:1883`" covers `:1902`'s
  `fs::read(env!("CARGO_BIN_EXE_rexx-run"))`, which is inside the same function.)
* **M5. `token.rs:40-44`.** "A `u32` keeps the error two words wide" is false once the inserts
  field lands (Task 2 pins the new size). Delete the sentence with the size pin.
* **M6. Task 5 Step 3's `-v` comparison.** Byte-identical only while `build/bin/rexx` carries
  `Jul 30 2026`, the day `version.rs:23` fixes; `rust/CLAUDE.md:36` already normalises the `rexxc`
  banner's date for exactly this. Say so in `tests/rexx_command.rs`, or normalise the date line.
* **M7. Line length.** Lines 323 (212 columns), 380 (177), 438 (157), 299 (130) and 265 (124)
  against the plan's own wrap.
* **M8. The TSV header counts.** It says "four tab-separated fields" and "Nine rows"; Task 1
  rewrites the header, so the counts go with it (`rust/CLAUDE.md`, no set sizes).

## Checks the brief asked for

**1. Spec coverage.** Every 9c deliverable of section 9's row has a task: D6 inserts, script,
column and reader (Tasks 1 to 4); `rexx` (5), `rexxc` (8), the restore surface (9), the switch and
its A/B (5, 6), section 7's 9c edit (9 Step 5). Criterion 4's pieces map to Tasks 4, 5, 8, 9 and
10, except the REGISTERED assertion (I2). Section 5's `rexx` and `rexxc` bullets and the switch's
enumerated sites are all in Tasks 5, 6 and 8. Out of scope: Task 7 is not in the spec's 9c row; it
is licensed only by the queued item's owner line, under a budget condition the plan does not meet
(C1).

**2. Prerequisites against the tree.** Opened and matching: `errors.rs:33`, `:86`, `:436`, `:578`,
`:615`; `build.rs:49`; `token.rs:28-73`; `error.rs:22`; `expr.rs:338`; `rexx-exec/src/error.rs:1982-2005`;
`instruction.rs:239`; `directive.rs:140`; `block.rs:356`, `:384`, `:395`, `:442`, `:650-670`;
`invocation.rs:41`; `lib.rs:3702`, `:3712`, `:2365`, `:2859-2860`; `require.rs:30`, `:57`, `:129`;
`group_runner.rs:582`, `:706-740`; `suite.rs:298-328`, `:1883`; `package.rs:374-419`;
`rexx-run.rs:18-149`; `sim_processes.rs:124`; `version.rs:23`; `arity.rs:399`; the bench, script
and prose sites of Task 6; `install.rs:715`, `:986`, `:1928`; `options.rs:584`; `run.rs:3304`;
`class_protocol.rs:339`; `callbacks.rs:1697`, `:1759`; `input.rs` `hand_over_output`; `clause.rs`
`idle_until`; `phase-4-exclusions.txt:5205-5262`, `:5391`, `:5679-5687`; `suite-9d-set.txt:208-210`;
the queued items; `rexx.cpp:80-242` (`argc_base` starts at 2, `:60`); `RexxCompiler.cpp:89-200`;
`InterpreterAPI.cpp:250-278`; `ProgramMetaData::validate` and `processRestoreData`;
`Routine.testGroup:238`; `ChildPath.testGroup:16`; `REXX_SUITE_GROUPS` and `REXX_SUITE_TABLE`
(`suite.rs:53`, `support/suite_rows.rs:54`); the pin at `/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run`;
the 9b1 close records; `rexx_inventory`'s generated `lookup` (`build.rs:172`); `Raised::syntax(98, 903, ..)`
for the `install` rows (`error.rs:308`). Wrong: I3's `LanguageParser.cpp` lines, M1.
Measured and matching the plan's "measured at plan time" claims: not found prints `Error 3:  Failure
during initialization.` then `Error 3.901:  Failure during initialization: Program "nonexist" was
not found.`, rc 253, stdout empty, no `running` line; `-e` names `INSTORE` in the error line and
`.local~syscargs` is `The NIL object`. The oracle's `validate` runs before `Envelope::puff`
(`ProgramMetaData.cpp:176-183`), so a header whose magic is not 11111 is 3.903 by reading, which
supports Task 8 Step 3's assertion. No committed recording outside `parse-errors.tsv` pins an
unfilled `&1` (`grep -rn '"&1"' crates/rexx-exec/tests crates/rexx-parse/tests corpus`), so Tasks 2
to 4 redden nothing unnamed.

**3. Consistency and ordering.** Task 5 leaves `run_crate_process` and `arity.rs` on `rexx-run`
until Task 6, and `interpreter_bin` moves first so `rexx_command` witnesses the new binary. Task 6
deletes `rexx-run` after the A/B's copies are in the pin directory, and Task 10's `pin=` is that
file. Task 7's test uses `CARGO_BIN_EXE_rexx`. Task 8 says which `rexxc` tests wait for Task 9.
`check_path`'s "no ELF but copies of this crate's `rexx`" needs a second identity at Task 8 (M4).
Interfaces that are unstated: I4, I5.

**4. Gate criteria.** Each test the plan adds has a stated red (Task 1's converse check, Task 4's
removed row, Task 7's reverted change) or compares against the oracle's own output. The
`.rxo`-rejected-by-the-oracle test and the TSV agreement test can fail. Weak: M3's unchecked
reasons; Task 2 Step 1's "sample of constants" test is a smoke test, the real check being the
compiler, which is fine.

**5. Budget.** Per-task checks fit: `derive.sh` over the oracle (about 1100 `rexxc` runs) and the
Task 8 TSV agreement (the same count, in parallel) are well under five minutes; the harness
groups named per task are small. Callgrind only at Task 5 (I1) and the close, except that Task 7
needs `-p sayloop` at the task (C1).

**6. Prose.** No em-dashes; ` -- ` appears only inside quoted commands. No set sizes in prose
beyond specified or measured figures (29-byte prefix, 48-byte header, 72-column lines, three fix
rounds). The Produces sections describe deliverables, not mechanisms in the tree. Long lines: M7.
