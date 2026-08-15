# Task 8 report: program arguments, `ARG`, `PULL`, `PARSE PULL`, `PARSE LINEIN`

Two commits, both read back with `git rev-parse HEAD`:

* `8eb5a40234a0e314f1ae15c272f2aa0c04601766` -- the argument plumbing and the 45 mechanical call-site edits.
* `81944c76667b627f44d10d6282ceabe63a871728` -- the input model, the two `PARSE` sources, the two instruction spellings, the boundary moves, the corpus program and the new differential harness.

Tree clean at the second.

## What was built, and where

### The argument half

`crates/rexx-exec/src/invocation.rs` is new.
It holds `Invocation` (the one optional argument string plus where `.input` reads from), `ProgramInput`, and `join_command_line`, which turns a list of command-line words into an `Invocation`.
`run_program` gained a third parameter carrying it; so did `run_program_collect_every_alloc` and the private `execute`.
`bin/rexx-run.rs` now collects every word after the program path, joins them, and asks for `ProgramInput::Stdin`.

`execute` fills the top-level `call_context` before `Interp::run`: the program's path as the name, and one `Argument::Value` when the command line supplied a string.
`PARSE ARG` already read that field, so no code in `parse_template.rs` changed for arguments.

### The input half

`crates/rexx-exec/src/input.rs` is new.
It holds `Input` (the `.input` position, over one of three sources) and the two readers on `Interp`: `pull_line` (queue head first, then `.input`) and `linein_line` (`.input` only, never the queue).
`queue.rs` gained `Queue::pop`, taking the front.

`parse_template.rs` gained exactly two arms, both one line of logic each, matching that file's own module-doc prediction that a new source touches neither layer:

```rust
ParseSource::Pull => ("PULL", self.pull_line()),
ParseSource::LineIn => ("LINEIN", self.linein_line()),
```

`Cursor` was not touched.
Neither was any other part of the engine.

`run.rs`'s `step` now routes `InstructionKind::Arg` and `InstructionKind::Pull` into the existing `InstructionKind::Parse` arm, because `rexx-parse` builds an identical `Parse` body for all three.
`Loud::parse_source` is deleted.

### Boundary

* `tests/owners.rs`: `Arg` and `Pull` become `Owner::InScope`; their two `EXPECTED_OUT_OF_SCOPE` rows are gone; the audited counts move 32 -> 34 in scope and 3 -> 1 for `4c`.
* `tests/loud.rs`: both witnesses deleted; `expected_instructions.len()` 11 -> 9; the in-scope count 32 -> 34.
* `lib.rs`'s `instruction_owner`: the `Arg | Pull | Address` group did **not** empty. See "Corrections to the dispatch" below.
* `tests/coverage.rs`: `EXPECTED_SUBSET_4C` gains `lang/pull_queue.rex`.
* `corpus/keyword-exempt.txt`: five PARSE rows removed (below).

## Step 0: the survey, re-measured

The brief said to treat Step 0 as fact.
I re-measured every row anyway, varying dimensions the brief had no reason to think mattered, and two of those variations changed the design.

### (a) The argument model

Program (`argcount.rex` / `arglen.rex`), run from a fresh empty directory, all three descriptors separate:

```
say "count=["arg()"]"     /  say "len="length(arg(1))  /  say "hex="c2x(arg(1))
```

Wrapper, every run: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE ARGS )`

| argv after the path | `arg()` | `arg(1)` | length | `arg(1,'O')` / `arg(1,'E')` |
|---|---|---|---|---|
| (none) | 0 | `` | 0 | `1` / `0` |
| `a b c` | 1 | `a b c` | 5 | `0` / `1` |
| `"a b c"` | 1 | `a b c` | 5 | `0` / `1` |
| `"a,b,c"` | 1 | `a,b,c` | 5 | `0` / `1` |
| `""` | 1 | `` | 0 | `0` / `1` |
| `"" ""` | 1 | `` | **0** | `0` / `1` |
| `"" "x"` | 1 | `x` | **1** | -- |
| `"x" ""` | 1 | `x ` | 2 | -- |
| `" x"` | 1 | ` x` | 2 | -- |
| `"a  b" "c"` | 1 | `a  b c` | 6 | -- |
| `$(printf 'a\tb')` | 1 | `a<TAB>b` | 3, hex `610962` | -- |

`arg(2)` was the null string on every row, rc 0 throughout, stderr empty throughout.

Three of those rows do **not** fit "join the words with a blank": `["",""]` gives length 0 where joining gives 1, and `["","x"]` gives length 1 where joining gives 2.
The oracle's launcher explains it, `utilities/rexx/platform/unix/rexx.cpp:143`-`145`:

```c
if (arg_buffer[0] != '\0')      /* not the first one?              */
    strcat(arg_buffer, " ");    /* add an blank                    */
strcat(arg_buffer, argv[i]);    /* add this to the arg string      */
```

The blank goes in when the accumulated buffer is non-empty, not when the word is not the first.
I then **predicted three rows from that C++ and ran them**, rather than stopping at the rows that suggested it:

```
=== PREDICT x len1: "" "" "x"        -> count=1 len=1 raw=<x>    hex=78
=== PREDICT "x  y" len4: x "" y      -> count=1 len=4 raw=<x  y> hex=78202079
=== PREDICT "  x" len3: " " "x"      -> count=1 len=3 raw=<  x>  hex=202078
```

All three as predicted.
`rexx.cpp:161` is the other half: `argCount = (argCount==0) ? 0 : 1;`, counted from the words and independent of the joined text -- which is the `None` versus `Some("")` distinction, mechanised in the oracle's own source.

`join_command_line` reproduces both rules. It has **no 8192-byte ceiling**; `rexx.cpp` `strcat`s into `char arg_buffer[8192]`, and overrunning that is undefined behaviour rather than an answer to reproduce. Flagged as a deliberate divergence, unmeasured beyond that buffer.

### (a-bis) The finding the brief did not have: absent versus empty IS observable in 4c scope

The brief calls `Some("")` a defect by name but gives only `ARG()` as the discriminator, and `ARG()` is Task 10's loud builtin.
I nearly wrote in the report that the distinction was therefore unwitnessable and had to be pinned in-crate.
I ran it instead. Measured, oracle:

```
=== use arg p, NO arguments            rc=0   use arg p -> <P>
=== use arg p, ONE EMPTY argument ""   rc=0   use arg p -> <>
=== use strict arg p, no args          rc=216  Error 40.3
=== use strict arg p, ONE EMPTY ""     rc=0   use strict arg p -> <>
=== use strict arg (0 targets), no args        rc=0   "no targets"
=== use strict arg (0 targets), ONE EMPTY ""   rc=216  Error 40.4
```

`USE ARG` is already in scope, so the distinction is differentially observable three ways -- an unbound target reading as its own derived name, a 40.3 against rc 0, and a 40.4 against rc 0.
Two of the three are witnessed in `tests/input_oracle.rs` (`arg-none`/`arg-empty`, `strict-none`/`strict-empty`).

This is also why `USE ARG` at top level had to start seeing the argument, which is behaviour of an already-implemented instruction and not new surface.

### (b) The shared position

Program: `parse pull n1` / `parse linein n2` / `parse pull n3` / `parse linein n4`, then one `say`.

```
=== no queue, stdin 4 lines      1=<line-A> 2=<line-B> 3=<line-C> 4=<line-D>
=== no queue, stdin 2 lines      1=<line-A> 2=<line-B> 3=<> 4=<>
=== no queue, stdin /dev/null    1=<> 2=<> 3=<> 4=<>
=== queue 2, stdin 4 lines       1=<qentry-one> 2=<line-A> 3=<qentry-two> 4=<line-B>
=== queue 2, stdin /dev/null     1=<qentry-one> 2=<> 3=<qentry-two> 4=<>
```

Row 1 is the shared position. Row 4 is the structural finding: `PARSE LINEIN` answered from the console while the adjacent `PARSE PULL` answered from the queue, and the two `PARSE PULL`s did not advance the console at all.
rc 0 and empty stderr on all five.

**Only three of the ooTest assertion's five constructs are mine.** `runtime.objects/environmentEntries.testGroup:174-181` runs `pull`, `parse pull`, `parse linein`, `linein()` and `.input~lineIn` against one `.ArrayStream`. `LINEIN()` is an excluded builtin (`corpus/builtin-status.txt:76`, `LINEIN excluded`; D4 keeps it excluded) and `.input~lineIn` is a message send, Phase 5. That assertion is not a target this task could make pass and I did not try. `input.rs`'s module doc says so where it cites it, and `corpus/lang/pull_queue.rex` covers three constructs, not five.

### (b-bis) The line rule, byte for byte -- not in the brief

Program: three `parse linein`, each printing `length` and `c2x`.

```
" pad \n\nlast-no-newline"  ->  5 / 2070616420 ; 0 / (empty) ; 15 / 6C6173742D6E6F2D6E65776C696E65
"crlf\r\nplain\n"           ->  4 / 63726C66   ; 5 / 706C61696E ; 0
"a\x00b\nnext\n"            ->  3 / 610062     ; 4 / 6E657874   ; 0
"a\rb\nx\r\r\ny\r"          ->  3 / 610D62     ; 2 / 780D       ; 2 / 790D
```

The `\r\n` collapse is real, and it removes exactly one CR and only when a newline followed.
`common/platform/unix/SysFile.cpp:696`-`742` (`SysFile::gets`) is the site: a `\r` is rewritten to `\n` when the next byte is `\n`, and otherwise left as data with the byte pushed back.
`y\r` at end of file keeps its CR because `getChar` fails there.

### (b-ter) An unreadable console -- not in the brief, and it decided the error model

I was about to give `read_line` a `Result` and a new `Loud` for an I/O error.
I ran it first:

```
=== oracle, fd 0 CLOSED (exec 0<&-), parse linein   rc=0  1=<> 2=<> 3=<> 4=<>   stderr empty
=== oracle, stdin is a DIRECTORY                    rc=0  1=<> 2=<> 3=<> 4=<>   stderr empty
```

So an unreadable descriptor is end of input, not a condition.
`Input::read_line` reports `None` for an I/O error, which is the measured answer rather than a swallowed failure, and `tests/input_oracle.rs::an_unreadable_console_is_end_of_input` witnesses it by binding both interpreters' stdin to a directory. No `Result`, no `Loud`, and the design got simpler for being measured.

### (c) Empty queue plus `/dev/null`

Row 3 and row 5 of (b): the null string, rc 0, no condition, no hang, and it repeats -- `pull_queue.rex`'s block B reads four times past the end and gets four null strings.

### (d) `PULL` uppercases, `PARSE PULL` does not

```
=== PULL vs PARSE PULL, and bare PULL    rc=0
1=<LOWER ONE> 2=<lower two> 3=<lower four>
```

Bare `PULL` (no template) still consumed line three, which is why line four is `lower four`.
The fold is ASCII-only, measured over a byte alphabet: input `a\xe0z-\xc1` came back `len=5 hex=41E05A2DC1` -- `a`->`A`, `z`->`Z`, `\xe0` and `\xc1` unchanged.

### (e) `ARG template` is `PARSE UPPER ARG template`

With arguments `mIxEd CaSe`:

```
arg:       <MIXED><CASE>
parse arg: <mIxEd><CaSe>
upper arg: <MIXED><CASE>
bare forms survived
```

And with no arguments at all, all three print `<><>` -- the targets are **assigned the null string**, not left unset.
`arg upper t4` with the same arguments: `UPPER=<MIXED> t4=<CASE>`, so `UPPER` after the source is an ordinary target. No handling was needed; `rexx-parse` already resolves it that way.

### (f) Trace, in BOTH modes

The brief's survey was `trace i`-only and told me to probe both. I did, on two programs each.

`trace i`, program with arguments `mIxEd`, stdin four lines:

```
     2 *-* parse pull n1
       >K>   "PULL" => "LOWER one"
       >>>   "LOWER one"
       >=>   N1 <= "LOWER one"
     4 *-* pull n3
       >K>   "PULL" => "skipped three"
       >>>   "SKIPPED THREE"
       >=>   N3 <= "SKIPPED THREE"
     5 *-* arg n4
       >>>   "MIXED"
       >=>   N4 <= "MIXED"
```

`trace r`, identical program:

```
     2 *-* parse pull n1
       >K>   "PULL" => "LOWER one"
       >>>   "LOWER one"
       >>>   "LOWER one"
     4 *-* pull n3
       >K>   "PULL" => "skipped three"
       >>>   "SKIPPED THREE"
       >>>   "SKIPPED THREE"
     5 *-* arg n4
       >>>   "MIXED"
       >>>   "MIXED"
```

Three facts, both modes:

* `>K>` is emitted for `PULL` and `LINEIN` in **both** modes, so it is a `results` prefix and not an `intermediates` one.
* `ARG` and `PARSE ARG` emit **no** `>K>` in either mode.
* For a bare `PULL`, `>K>` carries the value **before** the upcase and the `>>>` after it carries the value after -- the two lines disagree on one instruction.

The target's own value line is the choice of prefix Task 7 recorded (`>=>` at `i`, `>>>` at `r`).
This needed no code: `parse_strings` traces `>K>` with the raw line and `next_template` applies `UPPER` and traces `>>>`, in that order, which is already how the engine is written.

A second pair of programs covered a queue-sourced `PULL`, bare templates and reads past end of input, under both modes; all four ran byte-identical against this crate afterwards.

### (g) What the suite checks

Not re-derived; taken as given. What I did check is the five ooTest bodies that changed state -- see "Corrections" below.

### (h) Probe hygiene

`n1`..`n23` throughout. No probe used `a`, `b` or `c` as a target name.
The corpus program's targets are `n1`..`n23` for the same reason.

## Decisions, each with its reason

### The stdin model

`ProgramInput` has three arms: `Nothing` (default), `Stdin`, `Bytes(Vec<u8>)`.
`Invocation::none()` gives `Nothing`, which behaves exactly as `/dev/null` -- the state the differential harnesses already put the oracle in.

**A test cannot block, and the reason is structural rather than a convention.** Reaching the process's real descriptor requires writing `ProgramInput::Stdin`, and the only place in the tree that writes it is `bin/rexx-run.rs`, which is a separate process with its own stdin. Every in-process caller of `run_program` -- 43 of them, all in `src/` unit tests and `tests/` harnesses -- passes `Invocation::none()`. There is no path by which omitting a parameter, or forgetting a builder call, reaches the harness's stdin.

Reading is incremental (`BufRead::read_until`), not read-it-all-then-split, because the oracle reads a line at a time: a program that reads one line and exits must not first require the descriptor to reach end of file.

### The harness extension -- extended, as ruled

`Oracle::run_with(path, args, stdin)` sits beside `Oracle::run`, which now delegates to it, so the `ulimit`, the working directory, the `LD_LIBRARY_PATH` and the invocation counter stay in one place.
The `sh -c '... exec "$0" "$@"'` wrapper already forwards `"$@"`, so arguments needed no escaping.
`stdin: None` keeps a literal `Stdio::null()` rather than an immediately-closed pipe, so `run`'s existing behaviour does not change shape; `/dev/null` and a closed pipe are different descriptors and `input-closed-pipe` is the row that checks they answer the same.

**`tests/input_oracle.rs` drives the `rexx-run` binary, not `run_program` in process, and that is a deliberate departure from every other harness here.** The two things under test both live in the binary: turning `argv` into one string, and choosing the descriptor. An in-process comparison would supply the joined string and the input bytes itself, so it would test neither -- it would compare this harness's idea of a command line against the oracle's, which is a check on the harness. Running the binary compares one whole command line against the other. `CARGO_BIN_EXE_rexx-run` is Cargo's path to the freshly built binary, so no stale copy from `PATH` can answer.

19 case rows plus the unreadable-console test. No `KNOWN GAP` row was needed. (This sentence read "18" until fix round 1; the corrected figure is counted by script, not by eye.)

### `run_program_collect_every_alloc`

It takes the parameter.
It is a mode of the same `execute`, not a narrower door, and a mode that could not run an argument-carrying program would be unable to stress the one value in this change whose reachability is least obvious -- the argument `ObjRef`, created before the first clause and read possibly by the last, held in a field the collector does not walk.
`tests/collect_stress.rs::a_command_line_argument_survives_collect_on_every_allocation` is that stress, and mutation showed it is the only thing in the tree that sees the root (below).

### The argument representation

`Option<Vec<u8>>` in `Invocation`, becoming `Vec<Option<Argument>>` in `Interp::call_context.arguments` -- empty for `None`, one `Some(Argument::Value(..))` for `Some(..)`.

`call_context` rather than a field of its own, for three reasons, in order of weight:

1. **It is what `USE ARG` already reads**, and `USE ARG` at top level is measured as seeing the program argument. A separate field would have left an in-scope instruction wrong.
2. **It is the interface Task 10's `ARG()` needs.** `ARG()` inside a subroutine must read the call's arguments and at top level the command line's; `run.rs:2129` already reads `self.call_context.arguments.len()` for `USE STRICT ARG`'s arity. One field, one reader, no special case for depth 0.
3. `PARSE ARG` needed no new code at all.

Rooting is a `push_temp` taken before `Interp::run`, which outlives every clause because `step_in_temps_frame` truncates to a watermark it takes on entry and every such watermark sits above this push. That is the same mechanism `resolve_and_run_call` uses for a call's own arguments.

## Corrections to what the dispatch said

**`lib.rs`'s `instruction_owner` arm did not empty and the arm did not go away.** The controller's note said the group holds exactly `Arg` and `Pull` after Task 7. It holds three: `InstructionKind::Arg(_) | InstructionKind::Pull(_) | InstructionKind::Address(_) => Some("4c")`. `Address` is a later 4c task's, so the arm shrinks to `InstructionKind::Address(_) => Some("4c")` rather than disappearing, and `tests/owners.rs:173` / `EXPECTED_OUT_OF_SCOPE`'s `("InstructionKind", "Address", "4c")` / `tests/loud.rs`'s `Address` witness all stay.

**The brief's Step 4 said "both `owners.rs` rows, both `EXPECTED_OUT_OF_SCOPE` rows, `lib.rs:758`'s arm, and both `loud.rs` witnesses deleted."** All of that happened except the arm, for the reason above. Three further edits were needed that no dispatch mentioned, each found by a failing test rather than by reading: the audited counts in `owners.rs` (`variant_counts_match_the_audited_split`), the two in `loud.rs` (`assert_witness_set_is_complete`, `in_scope_counts_match_the_audited_split`), and `corpus/keyword-exempt.txt`.

**Five ooTest bodies now pass and had to leave `corpus/keyword-exempt.txt`**: `PARSE::test_PARSE_variable_patterns`, `PARSE::Test_614`, `Test_620`, `Test_626`, `Test_632`. The file's own harness names them and demands their removal in exactly those words ("now PASSES but is still on the committed exempt list -- remove it"), which is the two-directional policing its record describes. `PARSE::Test_638` still fails and stays.

**A pre-existing divergence was fixed because this change made it reachable.** `use strict arg p` as a program's first clause reports 40.3 substituting the program's own path; this crate reported an empty name, because `call_context.name` was never filled at top level:

```
oracle: Error 40.3:  Not enough arguments in invocation of /abs/path/usestrict.rex; minimum expected is 1.
rust:   Error 40.3:  Not enough arguments in invocation of ; minimum expected is 1.
```

Supplying an argument makes the matching 40.4 reachable too (`use strict arg` with no targets and one argument). Both now match byte for byte and both are rows in `input_oracle.rs`.

**`phase-4c.txt` is not read by `tests/corpus.rs`.** I found this while checking that `pull_queue.rex` was running, and it is not a defect: the plan states it explicitly at Task 7 -- "`tests/corpus.rs` does not read it until Task 15 Step 4 -- so this task's witness is committed but inert, and that is expected rather than a defect." So `pull_queue.rex` is committed and inert exactly as Task 7's three are, and the gate stays at 42 of 42 rather than moving to 46. **I ran it against the oracle by hand instead** (below), and added a `queue-round-trip` row to `input_oracle.rs` so the queue's round trip has a witness that runs *today* rather than at Task 15. Wherever I first wrote "the queue's first differential witness" in `pull_queue.rex`'s header and in `phase-4c.txt`, I corrected it before committing, because with the live row present the claim was false.

**The `SOURCELINE` driver now needs `</dev/null`.** `crates/rexx-parse/tests/sourceline_oracle.rs` regenerates its expectations by calling `.Package~new(file)`, which runs the file's prolog. `pull_queue.rex`'s prolog reads the console, so the documented recipe hung -- measured, a 2-minute timeout with no output. The recipe in that file's module doc now carries the redirect and says why.

## The corpus program

`corpus/lang/pull_queue.rex`, five blocks, each stating in the header which wrong answer a defective engine prints. Run by hand against the oracle with stdin at `/dev/null` and no arguments, which is exactly what the harness will give it at Task 15:

```
oracle rc=0 / rust rc=0
--- stdout diff  SAME
--- stderr diff  SAME
A [A][c][b]
B [][][][]
C [][queued-not-for-linein]
D [][][][]
D bare forms are legal
E done
```

stderr is 38 trace lines across the `TRACE R` and `TRACE I` sections, byte-identical, including the `>K>` pre-upcase split on `pull n14`/`pull n19`. (This read "36" until fix round 1.)

## The hand-run differential set

17 programs, each run through both interpreters with the same argv and the same stdin, comparing stdout, stderr and exit status as three separate files. All 17 `AGREE`:

```
shared cursor, 4 lines / 2 lines / devnull; queue + 4 lines; queue + devnull;
pull uppercase; pull high bytes; byte-level lines; CRLF; CR variants; NUL;
trace i; trace r; trace i (queue+bare); trace r (queue+bare);
unreadable stdin (dir); arg template family
```

Before `rexx-run` was wired to `ProgramInput::Stdin` the seven stdin-reading rows diverged with this crate answering the null string throughout -- which is what caught the missing `.with_input` call, and is worth recording because a `Nothing` default that is *correct* for the harness is *silently wrong* for the binary.

## Mutation results

Every new test was checked against a mutation, and against the suite *without* it, so that "can fail" is not being reported as "adds coverage".

| Mutation | Caught by | NOT caught by |
|---|---|---|
| `Queue::pop` takes the back | `interleaved_push_and_queue_survive_a_round_trip`; `input_oracle` `queue-round-trip` | both pre-existing `queue.rs` order tests, which stayed green |
| `join_command_line` returns `Some(Vec::new())` for no words | both `invocation.rs` unit tests; `input_oracle` `arg-none` (stdout) and `strict-none` (stdout, stderr, exit code) | -- |
| Drop the `\r\n` collapse | `a_line_is_the_bytes_before_the_terminator`; `input_oracle` `input-crlf` | -- |
| `pull_line` skips the queue | `input_oracle` `queue-round-trip`; `keyword_assertions` | -- |
| Join with a blank between every pair | `command_line_words_join_the_way_the_oracle_joins_them`; `input_oracle` `arg-leading-empty-word` | `no_words_is_absent_and_one_empty_word_is_present`, which stayed green -- the two tests are not redundant |
| Unroute `Arg`/`Pull` from the `PARSE` arm | `input_oracle` (both tests); `keyword_assertions` | -- |
| Drop `execute`'s `push_temp` on the argument | `a_command_line_argument_survives_collect_on_every_allocation` **only** | `input_oracle` stayed 9/9 green, and the whole plain suite stayed green |

The last row is the one that justifies the stress test existing: nothing else in the tree sees a dropped argument root.

## Verification

All from `rust/`, at `81944c76`.

```
$ cargo test --workspace
73 lines of "test result: ok", 0 lines containing FAILED, 1151 tests passed
```

Baseline was 72 `test result: ok` lines; the 73rd is the new `tests/input_oracle.rs` binary.

```
$ cargo fmt --all --check
fmt=0
```

```
$ rm -rf <fresh dir>; CARGO_TARGET_DIR=<fresh dir> cargo clippy --workspace --all-targets -- -D warnings
clippy exit=0
0 lines beginning "warning" or "error"
```

Run from a cold target directory, twice, per the standing rule that a same-session green is provisional.

```
$ REXX_CORPUS_GATE=1 cargo test --workspace
42 of 42 matching
0 lines containing FAILED
```

The gated run also executes `tests/input_oracle.rs`'s two tests, which skip without the variable.

## Things inferred rather than measured, flagged

* **The 8192-byte argument ceiling.** `rexx.cpp` `strcat`s into a fixed buffer with no bound check. I did not run a command line long enough to overrun it and would not; the divergence above that length is unmeasured and `join_command_line` deliberately has no ceiling. Recorded in that module's doc.
* **`ProgramInput::Stdin` on a terminal.** Every measurement fed a file, a pipe, `/dev/null` or a directory. Line-at-a-time reading is what the oracle does and is what avoids requiring EOF before the first line, but no probe used an actual tty.
* **Which platform word `PARSE SOURCE` uses elsewhere** -- unchanged from Task 7, not this task's, noted only because `pull_queue.rex` avoids `PARSE SOURCE` for the determinism reason.
* **`Loud::instruction`'s arms for `Arg`/`Pull`.** They remain in that exhaustive match (they must, it has no `_` arm) but are no longer reachable, because `step` routes both before the fallthrough. I did not construct a proof of unreachability; the observation is that no test exercises them and the loud message for both is gone from `input_oracle` and the corpus.

## One process failure worth recording

While restoring a mutated file I ran `git checkout -- rust/crates/rexx-exec/src/run.rs`, which restored it from HEAD and **silently discarded that file's uncommitted Task-8 edits** -- exactly the hazard `rust/CLAUDE.md`'s gate section records ("restore from a copy rather than from git"). I caught it immediately by grepping for the edit, re-applied both hunks by hand, and confirmed with `git diff` that the file matched what had been there. Every other mutation in the table above was restored from a `cp` backup taken first. No other file was affected, and the committed `run.rs` is the intended one.

---

# Fix round 1

Base moved before this round: the coordinator's plan commit `87b762b0` sits on top of `81944c76`, and this round builds on it.

## The trace shape is now pinned, in the ungated suite

The gap was real and the coordinator's framing of it was exact: `parse_template.rs` *described* the trace behaviour of the two new sources, and the only thing checking it was `corpus/lang/pull_queue.rex`, which `tests/corpus.rs` does not read until Task 15.

**Pinned in `tests/trace_oracle.rs`**, not `input_oracle.rs`, for two reasons.
It runs in the plain ungated `cargo test --workspace`, where `input_oracle.rs` runs only under `REXX_CORPUS_GATE`; and it compares against a committed expectation, so it needs no oracle at check time -- which is what "cannot drift silently" means for a fact I measured once.

**No new program was written.** `trace_oracle.rs` already reads `trace_output.rex` out of `rust/corpus/lang/` by relative path, so the new witness points at `corpus/lang/pull_queue.rex` itself and only `tests/trace_oracle/pull_queue.expected` is new.
That keeps one program, checked two ways: byte-exactly and offline today, and live against the oracle once Task 15 wires `phase-4c.txt` into `corpus.rs`.
`WITNESS_PREFIXES` gains `("pull_queue", &["*-*", ">>>", ">=>", ">K>"])`; `CLAIMED_PREFIXES` needed no change, all four being claimed already.

The expectation was captured with the file's own documented recipe **plus `</dev/null`**, and the recipe now carries that redirect and says why: `pull_queue.rex` reads the console, so without it the capture blocks on a terminal or eats whatever a pipe holds, and `run_program`'s own default is the empty console, so any other input produces an expectation this crate can never reproduce.
This is the second harness whose regeneration recipe needed the redirect; `sourceline_oracle.rs`'s needed it in the first round, and both now say so.

### Mutation, and the coverage question separately

| Mutation | `trace_oracle::pull_queue` | everything else in the workspace | `input_oracle` (gated) |
|---|---|---|---|
| `>K>` carries the value **after** the upcase (`make_ascii_uppercase` before the trace, so stdout is unchanged) | FAILED | no other FAILED line | -- |
| `ARG` emits a `>K>` line like every other source | FAILED | no other FAILED line | 9 passed, green |

Both are caught by the new witness and by **nothing else**, including `parse_placeholder`, which is the existing `PARSE` trace witness and stayed green through both.
So this is added coverage, not merely a test that can fail -- which is the distinction the second row was worth running to establish, since a `>K>` on `ARG` is the kind of extra line a byte-exact comparison somewhere else might plausibly have caught.

## Minor fixes

1. **`input_oracle.rs`'s vacuity floor.** `assert_eq!(oracle.invocations(), CASES.len())` derived both sides from the same array. Added `assert!(oracle.invocations() >= 15, ...)` above it -- a literal, not derived from `CASES`, and `>=` so that adding a row is not an edit here. The equality stays beside it, because it is what catches a row that failed to start the oracle. Checked by gutting `CASES` to one row: `only 1 oracle runs, which is fewer than this file has ever had`, FAILED. Without the floor that mutation passes.
2. **`an_unreadable_console_is_end_of_input` now goes through the harness.** `Oracle::run_with_stdin(path, args, Stdio)` is new, taking a descriptor where `run_with` takes bytes -- separate rather than a widened `Option`, because `Stdio` cannot express "feed these bytes" and a bytes caller should not have to build a pipe. The `sh -c 'ulimit … && exec "$0" "$@"'` wrapper, the library path and the working directory are factored into one private `Oracle::wrapped`, which deliberately does not touch the counter so each public method increments exactly once. The test now asserts `oracle.invocations() == 1`, which the hand-rolled version could not.
3. **Two unasserted call-site counts deleted.** `invocation.rs`'s "`ProgramInput::Stdin` has exactly one caller" and `queue.rs`'s "the one caller". The reviewer's reading was right that the first looked load-bearing and is not: the argument is carried by "not reachable by omission -- reaching it requires writing `ProgramInput::Stdin`", which is a property of the type rather than a census, and the rewritten paragraph leads with it.
4. **`phase-4-exclusions.txt`'s `TRACE ?` paragraph** no longer asserts "No row there uses `TRACE ?`". It states the consequence instead -- a row that did would fail for this reason rather than for a defect in what it was checking, so read this row first if one ever does. That is a directive to a future reader rather than a claim about a file's current contents, and it cannot be falsified by a later row. It would in fact have survived my own fix, since the trace coverage landed in `trace_oracle.rs`, but that is luck rather than a reason to keep it.
5. **Report counts corrected**: 18 -> 19 case rows, 36 -> 38 trace lines. Both are counted by script now, and the original sentences carry a note saying what they read before. Worth naming the mechanism rather than just the patch: both were counts of text I had written minutes earlier in the same session, which is exactly the footing of Task 7's "five of seven" miscount -- the material feels known, so it gets counted from memory instead of from the file. The habit that fixes it is not more care; it is that a number in prose gets a command run against the artefact, in the same way a measurement does.

## Corrections accepted, not acted on

* **"Committed but inert" was too strong for `pull_queue.rex`**, and the reviewer is right. `tests/corpus.rs` does not run it, so the gate legitimately stays at 42 -- but `coverage.rs::every_in_scope_variant_is_witnessed_by_the_phase_subsets` reads the union *including* `phase-4c.txt`, so that program is what discharges the newly in-scope `Arg`/`Pull` coverage obligation today. Parsed, not run. As of this round it is also byte-compared offline by `trace_oracle.rs`, so "inert" is now wrong twice over.
* The `git checkout --` recovery was independently checked against the tree: no lost hunk. Recorded.
* Task 15's criteria carry the 4b queue closure at `87b762b0`, cited to `input_oracle.rs`'s live row rather than to the corpus program.

## Deferred, untouched, as instructed

`ProgramInput::Bytes` being public with no caller outside its own unit tests; `execute` calling `interp.text(&argument)` where `text_owned` would avoid a copy; `input.rs`'s "There is no reachable state in which a line read fails", which reads as an unreachability claim until the next paragraph rescues it.

## Verification, all from `rust/`

```
$ cargo test --workspace
73 lines of "test result: ok", 0 lines containing FAILED or "panicked", 1152 tests passed
```

1151 -> 1152: the one new test is `pull_queue_covers_the_line_reading_sources_in_both_modes`. Still 73 test binaries.

```
$ cargo fmt --all --check
fmt exit=0

$ rm -rf <fresh dir>; CARGO_TARGET_DIR=<fresh dir> cargo clippy --workspace --all-targets -- -D warnings
clippy exit=0
0 lines beginning "warning" or "error"

$ REXX_CORPUS_GATE=1 cargo test --workspace
42 of 42 matching
0 lines containing FAILED
```

Every exit status above was read from an unpiped command, the grep counts from separate runs.
