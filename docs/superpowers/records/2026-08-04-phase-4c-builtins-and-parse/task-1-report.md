# Phase 4c Task 1 -- boundary infrastructure and the four attribution fixes

**Status: DONE_WITH_CONCERNS.** Everything in the brief that could run, ran.
Step 4.3 is owed by Task 2 as instructed. Five things the brief or its
sources got wrong are recorded in "What the brief got wrong" below; three of
them changed what I shipped.

**Commit:** `aa7b350535a7038ff87f1e534f30e6818cf29970`
(`Record the builtin boundary by measuring it, and fix four attributions`),
read back with `git log --format="%H %s" -1`. Parent `7d8c43db`. Working
tree clean afterwards (`git status --porcelain` empty, exit 0).

One commit, not two. Step 6's "this row and Step 5's constant land in the
same commit" is satisfied trivially, and a single commit keeps every verify
result below attached to the exact tree it was measured on.

---

## 1. What was built

### 1.1 `rust/corpus/builtin-probes.txt` (new, 66 rows)

One `NAME<TAB>program` row per in-scope builtin, each a complete one-line
Rexx program that computes and prints a value. Header comments state the
format, why each program computes something rather than calling with zero
arguments, why no probe nests one builtin inside another, and the two
measured properties (determinism, self-containedness).

### 1.2 `rust/corpus/builtin-status.txt` (new, 81 data rows)

`NAME<TAB>STATUS` in `rexx_inventory::builtins::NAMES` order. Statuses
`implemented` / `loud` / `divergent` / `excluded`. Header says the file is
derived, names the command to re-derive it, and defines the four statuses.

### 1.3 `rust/crates/rexx-exec/tests/builtin_status.rs` (new)

Four tests:

| test | what it asserts |
|---|---|
| `the_status_file_matches_a_live_differential_run` | derived == committed in **both** directions with directional messages; row count == `NAMES.len()`; excluded == `wholly_excluded().len()` == 15; in-scope == 66; **oracle invocations == 66** |
| `every_loud_row_is_loud_about_its_own_builtin` | a `loud` row's stderr mentions that row's own builtin as a whole word |
| `every_divergent_row_has_a_known_gap` | a committed `divergent` row requires `KNOWN GAP: <NAME>` in `phase-4-exclusions.txt` |
| `a_name_is_only_mentioned_when_it_stands_alone` | the word-boundary rule the second test rests on, positive and negative |

On any mismatch the derived table is written to
`rust/target/tmp/builtin-status.derived.txt` and the failure names that path,
so a later task copies rather than retypes.

Each probe runs in its own freshly created directory under
`CARGO_TARGET_TMPDIR`, named for its builtin, containing only `probe.rex`.
Both interpreters are given the same canonicalised absolute path.

### 1.4 `rust/crates/rexx-exec/tests/support/oracle.rs` (new)

`corpus.rs`'s oracle machinery, moved so both harnesses invoke the oracle
identically: `oracle_root`, `locate` (loud failure when the binary is
missing), `Oracle::run` (the `ulimit -v 1048576` shell wrapper),
`wrapped_exit_code`, and `descriptor_diffs` (the three-descriptor comparison
including DEVIATION 0's stderr normalisation). `Oracle` now carries an
`AtomicUsize` invocation counter incremented inside `run`; that is what makes
the "66 invocations" assertion unfakeable by a name-table classifier.

Two behaviour changes came with the move, both deliberate:

* `Oracle::run` sets the child's stdin to `Stdio::null()`. Previously the
  oracle inherited `cargo test`'s stdin. The strict corpus gate still reports
  42 of 42 matching.
* `corpus.rs`'s module doc's "The oracle" and "The memory limit" sections now
  point at `support/oracle.rs`, where the full text lives, and its DEVIATION 0
  section names `descriptor_diffs` rather than `check_case`.

### 1.5 `rexx-inventory` (modified) and `coverage.rs` (modified)

`EXCLUDED_BUILTINS` moved out of `tests/coverage.rs` into
`rexx_inventory::builtins` as `pub const EXCLUDED`, with
`PARTIALLY_EXCLUDED`, `wholly_excluded()` and `in_scope()` beside it.
`NAMES` generation is untouched; the crate's module doc now distinguishes the
generated tables from the one hand-written policy list.

`coverage.rs` reads `EXCLUDED` from there via a re-export alias so its
existing assertions work unchanged, and gained three:
`PARTIALLY_EXCLUDED ⊆ EXCLUDED`, `wholly_excluded().len() == 15`, and
`in_scope().len()` equals the arithmetic. The magic `- 3` is gone.

### 1.6 The four attributions

* `trace_oracle.rs:535` -- `("+++", Coverage::Owned("Phase 7"))`.
  `WITNESSED_PREFIX_COUNT` (13) and `OUT_OF_SCOPE_PREFIX_COUNT` (6) untouched.
  The `PREFIX_COVERAGE` doc bullet for `+++` was rewritten: it said "4c" and
  "two producers", and it called the `TRACE ?` row "deliberately
  owner-unassigned", all three of which Steps 5 and 6 make false.
* `phase-4-exclusions.txt` -- new `+++` row; "Four of the six" corrected to
  "Three of the six"; `TRACE ?` row given Phase 7 and three added
  measurements; `>I>`/`<I<` row given an ownership anchor; `QualifiedCall`
  row given the `::ROUTINE` carve-out note; the fifteen whole exclusions each
  given a reason (Step 8 / D4).
* `2026-07-30-phase-4a-executor-design.md:71` -- "every directive except
  `::ROUTINE`, which is 4c's (see the 4c plan's D-R)".

---

## 2. Oracle measurements

Every oracle run below was
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
/home/moritz/dev/repos/ooRexx/build/bin/rexx ABSOLUTE_PATH )` from a fresh
`mktemp -d` directory under the session scratchpad's `t1probe/`
subdirectory, with stdout, stderr and exit status captured to three separate
files and the status read unpiped. `stdin` at `/dev/null` unless stated.

Scripts used, all in the scratchpad, none in the repository:
`t1probe/probe.sh` (oracle only), `t1probe/both.sh` (oracle and
`target/debug/rexx-run`), `t1probe/batch.sh` and `batch2.sh` (all probes,
twice), `t1probe/rustbatch.sh` and `rustbatch2.sh` (all probes through the
executor), `t1probe/trace_q.sh` (chosen stdin and env).

### 2.1 The 66 probes, oracle side

Run **twice**, each pass from its own freshly created empty directory.
Every one: `rc=0`, empty stderr, identical stdout/stderr/rc on both passes,
and no file left in the run directory beyond the program and the captured
descriptors.

| builtin | program | oracle stdout |
|---|---|---|
| ABBREV | `say abbrev('Print','Pri') abbrev('Print','Pro')` | `1 0` |
| ABS | `say abs(-4.5)` | `4.5` |
| ADDRESS | `address zork; say address()` | `ZORK` |
| ARG | `call f 7, 8; exit; f: say arg(2)` | `8` |
| B2X | `say b2x('11000011')` | `C3` |
| BITAND | `say bitand('cat','DOG')` | `@AD` |
| BITOR | `say bitor('cat','DOG')` | `gow` |
| BITXOR | `say bitxor('cat','   ')` | `CAT` |
| C2D | `say c2d('81'x)` | `129` |
| C2X | `say c2x('Ab')` | `4162` |
| CENTER | `say '['center('ab',6,'-')']'` | `[--ab--]` |
| CENTRE | `say '['centre('ab',6,'-')']'` | `[--ab--]` |
| CHANGESTR | `say changestr('a','banana','X')` | `bXnXnX` |
| COMPARE | `say compare('abcde','abXde')` | `3` |
| CONDITION | `signal on syntax name h; say 1/0; h: say condition('C')` | `SYNTAX` |
| COPIES | `say copies('ab',3)` | `ababab` |
| COUNTSTR | `say countstr('a','banana')` | `3` |
| D2C | `say d2c(16706)` | `AB` |
| D2X | `say d2x(129)` | `81` |
| DATATYPE | `say datatype('12.5')` | `NUM` |
| DATE | `say date('S','2026-08-04','I')` | `20260804` |
| DELSTR | `say delstr('abcdef',3,2)` | `abef` |
| DELWORD | `say delword('a b c d',2,2)` | `a d` |
| DIGITS | `numeric digits 12; say digits()` | `12` |
| ERRORTEXT | `say errortext(40)` | `Incorrect call to routine.` |
| FORM | `numeric form engineering; say form()` | `ENGINEERING` |
| FORMAT | `say '['format(3.14159,2,3)']'` | `[ 3.142]` |
| FUZZ | `numeric fuzz 3; say fuzz()` | `3` |
| INSERT | `say insert('-','abc',1)` | `a-bc` |
| LASTPOS | `say lastpos('a','banana')` | `6` |
| LEFT | `say '['left('ab',5,'.')']'` | `[ab...]` |
| LENGTH | `say length('abcdef')` | `6` |
| MAX | `say max(3,17,8)` | `17` |
| MIN | `say min(3,17,8)` | `3` |
| OVERLAY | `say overlay('XY','abcdef',3)` | `abXYef` |
| POS | `say pos('an','banana')` | `2` |
| QUEUED | `queue 'a'; queue 'b'; say queued()` | `2` |
| RANDOM | `say random(5,5)` | `5` |
| REVERSE | `say reverse('abcdef')` | `fedcba` |
| RIGHT | `say '['right('ab',5,'.')']'` | `[...ab]` |
| SIGN | `say sign(-12)` | `-1` |
| SOURCELINE | `say sourceline(1)` | `say sourceline(1)` |
| SPACE | `say '['space('a   b  c',2)']'` | `[a  b  c]` |
| STRIP | `say '['strip('  ab  ')']'` | `[ab]` |
| SUBSTR | `say substr('abcdef',2,3)` | `bcd` |
| SUBWORD | `say subword('a b c d',2,2)` | `b c` |
| SYMBOL | `zz = 4; say symbol('zz')` | `VAR` |
| TIME | `say time('N','12:34:56','N')` | `12:34:56` |
| TRACE | `trace off; say trace()` | `O` |
| TRANSLATE | `say translate('abcdef','123','abc')` | `123def` |
| TRUNC | `say trunc(12.987,2)` | `12.98` |
| VALUE | `zz = 41; say value('zz')` | `41` |
| VAR | `zz = 1; say var('zz')` | `1` |
| VERIFY | `say verify('abcde','abc')` | `4` |
| WORD | `say word('a b c d',3)` | `c` |
| WORDINDEX | `say wordindex('a b c d',3)` | `5` |
| WORDLENGTH | `say wordlength('ab cde f',2)` | `3` |
| WORDPOS | `say wordpos('c','a b c d')` | `3` |
| WORDS | `say words('a b c d')` | `4` |
| X2B | `say x2b('c3')` | `11000011` |
| X2C | `say x2c('616263')` | `abc` |
| X2D | `say x2d('81')` | `129` |
| XRANGE | `say xrange('a','e')` | `abcde` |
| LOWER | `say lower('ABCdef')` | `abcdef` |
| UPPER | `say upper('abcDEF')` | `ABCDEF` |
| GC | `say gc('Force')` | `1` |

Notes on the four the brief flagged as likely problem probes:

* **SOURCELINE** -- no problem. `sourceline(1)` returns the program's own
  first line, which is the probe itself; both interpreters read the same
  file, so it is deterministic and self-referential.
* **TRACE** -- no problem, and made stronger than `say trace()`. Plain
  `say trace()` returns `N` (the default) and would be satisfied by a
  constant; `trace off; say trace()` returns `O`, which requires the builtin
  to read the live setting.
* **GC** -- no problem. `say gc()` returns `0` ("did not run the collector");
  `say gc('Force')` returns `1` and exercises the argument check plus the
  collection path, and completed cleanly under the 1 GiB limit.
* **QUEUED** -- no problem within one program. `say queued()` alone returns
  `0`; `queue 'a'; queue 'b'; say queued()` returns `2` and is the form the
  exclusions file's own partial row permits (single-program only, because the
  oracle's queue is rxapi-backed). Run three times in three fresh
  directories: `2`, `2`, `2` -- nothing leaks between processes.

No probe was dropped and none was replaced with something weaker.

### 2.2 Determinism control

`batch2.sh` ran each of the 66 twice, comparing the two runs' stdout and
stderr with `cmp -s` and their exit statuses. Zero rows flagged
`NONDET-stdout`, `NONDET-stderr` or `NONDET-rc`. The three builtins that
would otherwise be nondeterministic are probed through deterministic forms
(`DATE`/`TIME` converting a given value between formats; `RANDOM` over a
one-element range).

### 2.3 The executor side, all 66

`rustbatch2.sh`, from fresh directories. **66 of 66** exited 120
(`NOT_IMPLEMENTED_EXIT`) with a message naming that row's own builtin, e.g.
`rexx-exec: routine "SUBSTR" is not implemented (4c)`. `ADDRESS`'s message
has the instruction shape, `rexx-exec: ADDRESS is not implemented (4c)`,
which is why the harness checks for the name as a whole word rather than for
a fixed sentence.

The draft that nested `c2x(...)` measured **5 of 66 wrong**:

```
!! BITAND  err=rexx-exec: routine "C2X" is not implemented (4c)
!! BITOR   err=rexx-exec: routine "C2X" is not implemented (4c)
!! BITXOR  err=rexx-exec: routine "C2X" is not implemented (4c)
!! D2C     err=rexx-exec: routine "C2X" is not implemented (4c)
!! XRANGE  err=rexx-exec: routine "C2X" is not implemented (4c)
```

### 2.4 `+++`, the RC line (Step 5's evidence)

```
$ probe.sh "trace e; address sh; 'exit 3'"
--- rc: 0
--- stdout:
--- stderr:
     1 *-* 'exit 3'
       >>>   "exit 3"
       +++   "RC(3)"

$ probe.sh "trace n; address sh; 'exit 3'"
--- rc: 0
--- stdout:
--- stderr:            (empty)
```

C++ citations, all read directly in `/home/moritz/dev/repos/ooRexx`:

| citation | line reads | verdict |
|---|---|---|
| `RexxActivation.cpp:4468` | `traceValue(rc_trace, TRACE_PREFIX_ERROR);` | VERIFIED |
| `RexxActivation.cpp:4024` | `buffer->put(PREFIX_OFFSET, trace_prefix_table[TRACE_PREFIX_ERROR], PREFIX_LENGTH);`, inside `RexxActivation::traceSourceString` (opens `:4007`) | VERIFIED |
| `RexxActivation.cpp:4305` | `if (inDebug() && !settings.wasSourceTraced())` guarding `traceSourceString()` | VERIFIED |
| `RexxActivation.cpp:4237` | `processTraceInfo(activity, Interpreter::getMessageText(Message_Translations_debug_prompt), ...)`; that message's text is `+++ Interactive trace. <q>Trace Off</q> to end debug, ENTER to continue. +++` (`rexxmsg.xml:6353`) | VERIFIED |
| `Activity.cpp:1496` | `RexxString *text = Interpreter::getMessageText(Message_Translations_debug_error);` inside `Activity::displayDebug`; that message's text is `+++ Interactive trace.  Error` (`rexxmsg.xml:6344`) | VERIFIED |
| `AddressInstruction.cpp:163` | `context->command(environment, _command, getIOConfig());` | VERIFIED |
| "one of `command()`'s two callers" | `/bin/grep -arn "\->command(" interpreter/` returns exactly two: `AddressInstruction.cpp:163` and `CommandInstruction.cpp:89` | VERIFIED |

### 2.5 `TRACE ?` (Step 6's evidence)

Program, all four runs: `say 'A'` / `pull v` / `say '<'v'>'`, with `trace ?r`
as the first line where stated. stdin two lines, `echo ONE` and `echo TWO`,
where stated.

**A. `trace ?r`, empty stdin** -- rc 0, stdout `A` then `<>`, stderr:

```
       +++ "LINUX COMMAND <abs path>/probe.rex"
     2 *-* say 'A'
       >>>   "A"
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
     3 *-* pull v
       >K>   "PULL" => ""
       >>>   ""
       >>>   ""
     4 *-* say '<'v'>'
       >>>   "<>"
```

**B. `trace ?r`, non-empty stdin** -- rc 0, stdout `A` then `<>` (identical
to A), stderr identical to A **plus** two lines:

```
/bin/sh: 1: ECHO: not found
/bin/sh: 1: ECHO: not found
```

So `trace ?r` drained both stdin lines, uppercased each, and issued it as a
shell command; the following `PULL` read `""`.

**C. no TRACE instruction, same non-empty stdin** -- rc 0, empty stderr,
stdout `A` then `<ECHO ONE>`. This is the control that makes B mean
something: without `?`, `PULL` reads the line the debug reader ate. An
implementation reproducing the banner without draining stdin would print
`<ECHO ONE>` where the oracle prints `<>` -- byte-exact at `/dev/null`,
wrong on stdout everywhere else.

**D. `RXTRACE=ON`, no TRACE instruction in the program, empty stdin** -- rc
0, stdout `A` then `<>`, stderr identical in shape to A (line numbers 1/2/3
rather than 2/3/4, since the program has no `trace` clause). The environment
variable reaches the same path.

### 2.6 D4's excluded-builtin claims, re-measured

| program | oracle stdout | note |
|---|---|---|
| `say qualify('foo.txt')` | `<run dir>/foo.txt` | absolute path for a file that does not exist -- pure path manipulation |
| `say userid()` | `moritz` | host-dependent |
| `say setlocal()` | `1` | |
| `say endlocal()` | `0` | **unpaired** -- D4 says "both return 1" |
| `s = setlocal(); say s endlocal()` | `1 1` | the pair returns 1, the call does not |
| `say stream('nosuch.txt','C','QUERY EXISTS')` | *(empty)* | |
| `say rxqueue('G')` | `SESSION` | `pgrep -a rxapi` -> `885 rxapi`, so a live daemon answered |

`std::env::set_var` unsafety, re-verified rather than quoted:

```
$ rustc --edition 2024 --crate-type bin -o /dev/null sv.rs
error[E0133]: call to unsafe function `set_var` is unsafe and requires unsafe block
$ rustc --version
rustc 1.97.1 (8bab26f4f 2026-07-14)
```

---

## 3. Step 3's expected result

Measured at this commit, exactly as the dispatch predicted:

```
$ /bin/grep -ac "	excluded$" rust/corpus/builtin-status.txt      -> 15
$ /bin/grep -ac "	loud$" rust/corpus/builtin-status.txt          -> 66
$ /bin/grep -ac "	implemented$" rust/corpus/builtin-status.txt   -> 0
$ /bin/grep -ac "	divergent$" rust/corpus/builtin-status.txt     -> 0
```

(81 data rows total. The counts above were also produced by the harness
itself, which is what the committed file was filled from -- I did not type
the rows.)

---

## 4. Step 4: falsification

All four mutations were applied to a copy-restored tree (`cp` from a
scratchpad backup, never `git checkout --`), and the tree was re-verified
green after each restore.

**4.1 -- delete a row. FAILS BY NAME.**

```
$ /bin/grep -av "^LENGTH	loud$" corpus/builtin-status.txt > … ; cargo test … builtin_status the_status_file
measured but absent from …/builtin-status.txt: ["LENGTH"]. The measured table
has been written to …/target/tmp/builtin-status.derived.txt; copy its rows
into … if the change was intended.
exit 101
```

**4.2 -- flip a row `loud` -> `implemented`. FAILS THE OTHER DIRECTION.**

```
rows whose measured status differs from …/builtin-status.txt:
  LENGTH: committed implemented, measured loud
        differing: [stdout, stderr, exit code]
        rust:   stdout="" stderr="rexx-exec: routine \"LENGTH\" is not implemented (4c)\n" exit=120
        oracle: stdout="6\n" stderr="" exit=0
exit 101
```

**4.3 -- OWED BY TASK 2.** It requires deleting a builtin's dispatch arm and
watching that row flip `implemented` -> `loud` on its own. No builtin is
dispatched at this commit, so there is no arm to delete and no row that is
`implemented`. Not attempted, and nothing weaker was substituted.

**Extra 1 -- a spurious committed row. FAILS.** Appending `ZORKOLO<TAB>loud`:

```
committed in …/builtin-status.txt but not produced by the run: ["ZORKOLO"]
-- either the name left BuiltinFunctions.cpp's table or the row is a typo
exit 101
```

**Extra 2 -- a probe that answers for another builtin. FAILS.** Reverting
BITAND's probe to `say c2x(bitand('13'x,'11'x))`:

```
these rows exited NOT_IMPLEMENTED_EXIT without their own builtin's name in
the message, so the row is answering for something else:
  BITAND: "rexx-exec: routine \"C2X\" is not implemented (4c)\n"
exit 101
```

**Extra 3 -- the name-table classifier. CAUGHT ONLY BY THE INVOCATION
COUNT.** Replacing the per-name `measure(...)` call with a stub returning
`Status::Loud` and a synthesised message, running nothing:

```
test result: FAILED. 10 passed; 1 failed; 0 ignored
assertion `left == right` failed: the oracle was invoked 0 times for 66
in-scope builtins; every in-scope name must be measured by running its
probe, and no excluded name may run anything
  left: 0
 right: 66
exit 101
```

This is the demonstration the brief's Step 3.3 rationale asks for, made
rather than asserted: the stub satisfied set equality in both directions,
every count, `every_loud_row_is_loud_about_its_own_builtin`, and
`every_divergent_row_has_a_known_gap`. Only the invocation count caught it.

---

## 5. The verify block

Run from `rust/`, each status read **unpiped** (`echo "… EXIT $?"`
immediately after the command, no pipeline).

| command | exit status | result |
|---|---|---|
| `cargo test --offline --workspace --no-fail-fast` | **0** | 1031 passed, 0 failed |
| `cargo fmt --all --check` | **0** | clean |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | **0** | clean; the run re-checked all eight crates (`rexx-inventory` changed, so everything downstream was recompiled -- not a warm-cache no-op) |
| `REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus` | **0** | STRICT mode, **42 of 42 matching** |

1031 = the 1020 the brief records plus this task's 11 (4 tests of my own in
`builtin_status.rs`, plus the 7 `support::tests` that link into every test
binary that does `mod support;`).

`cargo fmt --all --check` failed once, on four hunks in the new file, and was
fixed with `cargo fmt --all`; the re-check above is the post-fix run.

---

## 6. What the brief got wrong

**6.1 The `ADDRESS` probe the brief's own example shape invites is the
excluded half.** `phase-4-exclusions.txt`'s partial row says: "The
platform-supplied default is Phase 7's -- measured, `say address()` with no
ADDRESS instruction prints `sh`". My first draft probed exactly that. Fixed
to `address zork; say address()` -> `ZORK`, which is the half that is 4c's.
`VALUE` and `QUEUED` were already on their in-scope halves. Recorded in the
probe file's header so the next reader does not repeat it.

**6.2 The `+++` text's live measurement omits the trace setting, and it is
load-bearing.** The brief says "a command's non-zero `RC`, measured live as
`+++   "RC(3)"` after `address sh` and `'exit 3'`". Measured: under `trace n`
(the default) that program emits **nothing at all**; the line appears under
`trace e`. I used the brief's text but added the trace setting and the
negative control, because a reproduction instruction that does not reproduce
is worse than none.

**6.3 The parenthetical "(`:124` is about the trace gate)" is not right.**
Pre-edit `:105` is the two-condition trace gate; `:123-124` is the heading
`THREE MEASURED WAYS A ::ROUTINE ACTIVATION IS NOT AN INTERNAL LABEL'S, which
4c will have to meet:`, which introduces the *activation model* (its own
variable pool, builtins shadowing it, trace not crossing into it). The
substantive point -- that `:124` is not the ownership sentence -- stands, and
that is what I wrote into the row. The 4c plan's D-R carries the same
mischaracterisation and is worth correcting there.

**6.4 D4's "SETLOCAL / ENDLOCAL both return 1" is true only for the pair.**
Measured: unpaired `say endlocal()` returns **0**. Recorded corrected in the
exclusions file's new SETLOCAL/ENDLOCAL row.

**6.5 Step 7's "correct the `>I>`/`<I<` row so the ownership claim cites
`:99-100` rather than `:124`" describes an edit to a file that makes no
citation.** `phase-4-exclusions.txt` does not cite itself anywhere; the
mis-citation lives in the 4c plan's D-R, and the plan already carries the
corrected `:99-100`. I read the instruction as "make the row's ownership
claim locatable by content so the next citer cannot fetch the wrong
sentence", and added a paragraph naming the ownership sentence, saying what
the two paragraphs below it are and are not about, and recording that the
claim has already been fetched from the wrong place once. Deliberately
**not** by line number: any edit to this file moves those numbers, so a
self-referential line citation is falsified by the act of committing it.

---

## 7. Line-number churn this task causes

Every line citation into `phase-4-exclusions.txt` below its line 38 has
moved. New positions of the anchors the 4c plan cites, read back after the
commit:

| anchor | was | now |
|---|---|---|
| "…six -- >.> (4c), >M> and >N> (Phase 5)…" (was "Four of the six", `:83-84`) | `:84` | `:137-138` |
| the new `+++` row header | -- | `:142` |
| `>I> / <I<` row header | `:88` | `:174` |
| the ownership sentence, "4b declines to implement it…" | `:99-100` | `:185-186` |
| "…which 4c will have to meet:" | `:124` | `:225` |
| `QualifiedCall` row | `:540` | `:641` |
| `TRACE ?` row opening | `:989` | `:1096` |
| the `TRACE ?` owner paragraph (was "Owner unassigned.") | `:1009` | `:1147` |

`2026-07-30-phase-4a-executor-design.md:71` is unchanged -- the edit was
within that line.

In `trace_oracle.rs`, `WITNESSED_PREFIX_COUNT` and
`OUT_OF_SCOPE_PREFIX_COUNT` are now at `:557` and `:561` (the brief says
`:551` and `:555`); the doc-comment rewrite above them moved them by six.
**Their values are untouched**, 13 and 6, as Step 5 requires, and the
`+++` entry itself is at `:535`.

---

## 8. Things worth knowing for the tasks that follow

* **Re-running the harness is `cargo test --offline -p rexx-exec --test
  builtin_status`.** A flipped row fails with both sides' output and the path
  of a ready-to-copy derived table. Copy the rows; do not retype them.
* **A `loud` row must be loud about its own builtin.** If a task adds a
  probe or edits one, and that probe reaches another unimplemented name
  first, `every_loud_row_is_loud_about_its_own_builtin` goes red naming both.
  That check is the reason no probe nests a builtin call.
* **`rexx_inventory::builtins::EXCLUDED` has 18 entries and the
  excluded-outright set has 15.** Use `wholly_excluded()`. Task 2's dispatch
  wants this list, which is the reason it moved out of `coverage.rs`.
* **The oracle now gets an empty stdin from both harnesses.** A 4c task
  implementing `PULL` will want to change that deliberately rather than by
  accident; `Oracle::run` is the single place.
* **`KNOWN GAP: <NAME>` is now a load-bearing string** in
  `phase-4-exclusions.txt`: it is what a committed `divergent` row requires.
* **The `+++` prefix is Phase 7's now**, so a 4c task that implements the
  `ADDRESS` instruction must not expect to produce that prefix -- issuing the
  command is the half that does, and that half is Phase 7's.
