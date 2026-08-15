# Task 13 report: `::routine` dispatch, `>I>`/`<I<`, and the unguarded copy

Status: DONE_WITH_CONCERNS.
Four commits, in order:

| hash | subject |
|---|---|
| `e00528df2bf57f9a8571c722eb55569ada135488` | Dispatch a call to a same-file ::routine, third in the resolution order |
| `3aa457e10db33b79366737615f04ac37b23b1960` | Refuse a directive this crate cannot install, and only those |
| `622f7198e9c63f95ed7e78cb5926a19f0a76c38d` | Announce a ::routine's entry and exit, and end it on its own EXIT |
| `88b3701fcf963d57ba97ecd79d8ee4bfa0a98cdd` | Stop rendering a value no trace line will print |

Read back with `git log -4 --format='%H %s'` after committing, not written from memory.

---

## 1. Where the brief was wrong, and where it was right

Four findings, in descending order of how much they change the delivered behaviour.

### 1.1 `::REQUIRES` runs the required file's prolog, so presence IS use

Step 4 records `::requires 'helper.rex'` (file present) as `rc 0, "main ran"` and states the rule
"a program that *contains* a well-formed directive it never uses must run identically on both
interpreters". Both halves of that transcript are correct and I reproduced them. The rule drawn
from them is not safe, and the probe that shows it was not taken.

Measured 2026-08-07, clean directory `pQ`, `loud_helper.rex` whose first clause is
`say 'PROLOG RAN'` and which then carries `::routine hfn public`:

```
q1.rex:  say 'main ran'
         ::requires 'loud_helper.rex'

oracle:  rc 0, stdout "PROLOG RAN" then "main ran"
```

`::REQUIRES` loads the file and runs its prolog before `main`. So ignoring the directive can change
**stdout**, not only which routine a call finds, and there is no way to tell an empty prolog from a
printing one without loading the file. `::REQUIRES` is therefore refused here, naming Phase 5, and
that is a deliberate departure from the brief's o4 row. It is recorded in
`phase-4-exclusions.txt`'s new directive section as one of two named over-refusals.

### 1.2 `EXIT` inside a `::ROUTINE` ends the routine, not the program

Not in the brief at all. Found by the `>I>` probe sweep: `pK/k2.rex` is a routine whose body is
`trace l` then `exit 0`, and the oracle printed the caller's `say 'never'` line afterwards.

Measured, all rc 0, `pW`/`pX`, one row per route the exit takes to the routine boundary:

| program | oracle stdout |
|---|---|
| `call rtn / say result` with `::routine rtn ; exit 5` | `after, result=[5]` then `done` |
| `n1 = rtn() / say n1` with `::routine rtn ; exit 7` | `r= 7` |
| `call rtn / say result` with `::routine rtn ; exit` | `after, result=[RESULT]` |
| a label **inside** the routine doing `exit 9` | the caller runs on |
| an expression call inside the routine reaching `exit 9` | `r= 9` |
| `interpret "exit 4"` inside the routine | the caller runs on |
| falling off the routine's own end | `RESULT` unset, caller runs on |
| the same `exit 5` in a **CALLed label** | rc 5, program ends |

`RexxActivation::implicitExit` (`RexxActivation.cpp:1455`-`1469`) sets `RETURNED` outright for an
`isProgramLevelCall` activation and only otherwise walks up through `exitFrom`. Two of the six
routes arrive as `Failure::Exited` (an `Err`) rather than as `Ended::Exited`, which is why the
conversion sits where both meet.

### 1.3 Step 6's "non-dynamic" is `earlyTraceEntry`'s condition, not the gate

The brief says the routine's **non-dynamic** trace instruction fires the pair. Measured `pJ/j3`:
`trace value 'l'` as a routine's first clause fires both lines. The C++ has two routes and the
brief cites the narrower one: `earlyTraceEntry` (`:3630`-`:3641`) does demand
`nonDynamicTracingLabels()`, but `setTrace` calls `traceEntry` unconditionally (`:1024`) with the
flags already installed and `traceEntryAllowed` still true. Only the `setTrace` route is
implemented here; no probe separates the two, because `earlyTraceEntry` exists to announce the
entry before a guarded **method** takes its object lock and nothing can run between the two points
without methods.

What "first instruction" actually excludes is narrower still, and is measured (`pL`):

| routine body | oracle stderr |
|---|---|
| `trace l` / `return` | both lines |
| `n0 = 0` / `trace l` / `return` | nothing |
| `lbl:` / `trace l` / `return` | **nothing** -- a label spends the permission |
| blank line, `/* comment */`, `trace l` | both lines |
| `trace l` and nothing else (implicit exit) | both lines |

The label row is the one that matters for the implementation: `traceEntryAllowed` and
`first_instruction_pending` differ exactly there, since `PROCEDURE` after two labels is legal.

### 1.4 The Step 9 gap row's four neighbours are two different defects

The brief lists four programs that still abort after the argument-render guard and calls them all
instances of "an unguarded owned copy on a path that may discard it". One of the four
(`x = copies('a',N)`) is that shape and is closed. The other three are not discarded at all -- the
copy is read -- and no guard closes any of them. Section 6.3 has the inventory.

Everything else in the brief that I re-measured held, including all four corrected addresses:
`lib.rs:1690`-`1697` is the `BodyKey` production site with `directive: None` at `:1693`;
`Activation::nested` is at `activation.rs:586`-`619`; `lib.rs:501` / `eval.rs:484` are the
resolution-order contradiction; `WITNESSED_PREFIX_COUNT`/`OUT_OF_SCOPE_PREFIX_COUNT` are
`tests/trace_oracle.rs:621`/`:625` holding 14 and 5.

---

## 2. Steps 1-3: resolution order

**Alphabet.** Call form x target spelling x what the name matches. Call forms: `CALL name`,
`CALL 'NAME'`, `CALL (expr)`, `name(...)`, `'NAME'(...)`. Target spellings: bare symbol
(upcased by the scanner), quoted upper, quoted lower, quoted mixed. Matches: internal label,
builtin (in scope), builtin (wholly excluded), `::ROUTINE`, nothing. **34 programs** run under both
interpreters, byte for byte on all three channels, all from clean directories created for the
purpose.

Every one matches after the change. The load-bearing rows:

| program | oracle | this crate |
|---|---|---|
| `call max 1, 9` with `::routine max` | `9` | `9` |
| `call zorkolo` with both `zorkolo:` and `::routine zorkolo` | `LABEL` | `LABEL` |
| `call 'ZORKOLO'` with both | `ROUTINE` | `ROUTINE` |
| `call 'MAX' 1, 9` with `::routine max` | `9` | `9` |
| `call 'max' 1, 9`, nothing named max | 43.1 rc 213, `"max"` | same |
| `call 'max' 1, 9` with `::routine 'max'` | `ROUTINE-lower-max` | same |
| `call 'ZORK'` / `call zork` / `call 'zork'` with `::routine 'zork'` | `HIT` x3 | same |
| `call 'mixed'` with `::routine MiXeD` | `HIT` | same |
| `nm='zork'; call (nm)` with `::routine zork` | `HIT` | same |
| `nm='max'; call (nm) 1,9` | 43.1 rc 213, `"max"` | same |
| `call mainlabel` from inside a routine, label in the main body | 43.1 rc 213 | same |

**The asymmetry the brief did not name, and it is what makes a `::routine 'max'` reachable at
all.** The builtin table is matched **case-sensitively** and the routine table upcases both sides.
`call 'max'` misses the builtin and finds the routine; `call 'MAX'` reaches the builtin. This crate
already had the case-sensitive builtin lookup (`builtin::is_builtin`'s own doc), so no change was
needed there -- but the fact is load-bearing for the routine step in front of it and is now in
`Activation::body`'s doc.

**A defect the resolution change would have introduced, caught before landing.** `is_builtin` reads
`rexx_inventory::builtins::in_scope()`, the 66 names Phase 4 dispatches -- not the full 81. Adding
the routine step behind it sent the 15 wholly-excluded builtins (`CHARIN`, `LINEIN`, ...) through
to 43.1, a condition the oracle never raises for them, which is exactly the shape that lets a
program *expecting* that condition pass against a gap. `builtin::is_excluded_builtin` is its own
resolution step, in front of the routine lookup because a builtin always beats a `::ROUTINE`. It
was found by `keyword_assertions.rs` going red, not by review.

**`lib.rs:501` against `eval.rs:484`** is settled: `Loud::unresolved_call`'s doc no longer claims
external resolution is 4c's, and the resolution order is stated once, in
`resolve_and_run_call`'s own comment at the decision, with the measurement for each step.

---

## 3. Step 2: the six non-inheritances

`Activation::routine` is its own constructor and takes no `Inherited` at all. One probe per field,
each with a caller that set the value and a routine that reads it back, plus the neighbouring
`CALL`ed label that does inherit it. All rc 0.

| field | caller sets | routine reports | label reports |
|---|---|---|---|
| `settings` | `digits 7`, `form engineering` | `9 SCIENTIFIC` | `7 ENGINEERING` |
| `address` | `address system` | `sh` | `SYSTEM` |
| `trace_mode` | `trace r` | `trace()` = `N`, no clauses echoed | clauses echoed |
| `traps` | `signal on syntax name mytrap` | the routine's own `mytrap:` never runs | n/a |
| `condition` | inside a live `SIGNAL ON SYNTAX` handler | `condition('C')` = `''` | `SYNTAX` |
| the pool | `vv = 'CALLER'` | prints `VV`, its write does not survive | prints `CALLER`, write survives |

`form engineering` rather than `digits` alone is deliberate: the default `FORM` is `SCIENTIFIC`, so
a caller that set `SCIENTIFIC` would read identically whether inherited or defaulted.

The trap probe separates "not inherited" from "the caller caught it after the routine unwound",
which every two-level program answers the same way unless the routine has a label of the trap's own
name. It does, and it never runs.

Two more, neither of which falls out of the six:

* **No `SIGL` on either side.** `signal there` / `there:` / `call rtn` leaves the caller's `SIGL`
  at 1, and `sigl` inside the routine is the derived name `SIGL`. The one `set_sigl` line writes
  the *caller's* pool for a label and would write the *routine's* for a routine, so both probes
  would go wrong if it ran on both paths.
* **The callee's clause echoes start at indent 0**, not the caller's plus two. Section 5.3.

---

## 4. Step 4: directive resolution

**Alphabet.** All nine `DirectiveKind` variants, crossed with whether the directive names something
outside itself: bare, naming a class, naming a file, naming a library, naming a package member.
**21 programs**, each `say 'main ran'` followed by the directive, all from clean directories.

Runs on the oracle at rc 0 printing `main ran`, and runs here identically (7 programs):

```
::class foo
::class foo + ::method bar
::class foo + ::attribute baz
a loose ::method with no ::class
::constant kk 5
::resource foo ... ::END
::annotate package author 'me'
```

Refused by the oracle before `main` with **empty stdout**, and refused here (7 programs):

| program | oracle | this crate |
|---|---|---|
| `::class foo subclass zzznotaclass` | 98.909 rc 158 | loud, `::CLASS naming another class (Phase 5)`, rc 120 |
| `::class foo metaclass zzznotaclass` | 98.908 rc 158 | same message |
| `::class bar inherit zzznotaclass` | 98.909 rc 158 | same message |
| `::annotate routine nosuchrtn` | 99.945 rc 157 | `::ANNOTATE naming a target (Phase 5)` |
| `::requires 'no_such_file_zz.rex'` | 43.901 rc 213 | `::REQUIRES (Phase 5)` |
| `::routine z external "LIBRARY nosuchlib nosuchfn"` | 98.903 rc 158 | `::ROUTINE EXTERNAL (Phase 7)` |
| `::method m external "LIBRARY nosuchlib nosuchfn"` | 98.903 rc 158 | `::METHOD EXTERNAL (Phase 7)` |

The oracle's own number/sub/rc are **not** reproduced -- only "refuses before `main` with empty
stdout" is shared. That is the standing convention for a construct another phase owns.

Two deliberate over-refusals, both of which reject a program the oracle runs:

* `::class foo subclass object` -- rc 0 `main ran` on the oracle, loud here, because 4c has no
  class table and cannot tell a resolvable superclass from an unresolvable one. The alternative is
  running the `zzznotaclass` program at rc 0 where the oracle produces nothing at rc 158.
* `::requires` of a file that exists -- section 1.1.

`::OPTIONS` is refused without that trade: its whole effect is to change package settings,
unconditionally. Measured, `::options digits 12` with a body of `say digits() form() fuzz()` prints
`12 SCIENTIFIC 0`; `::options trace labels` makes every `::ROUTINE` in the file emit `>I>`/`<I<`.
There is no unused `::OPTIONS`.

**Duplicate `::ROUTINE` is not in that table** because it is reproduced exactly. Measured, two
`::routine zork` directives: rc 157, stdout empty, and stderr

```
     5 *-* ::routine zork
Error 99 running <path> line 5:  Translation error.
Error 99.903:  Duplicate ::ROUTINE directive instruction.
```

byte for byte on both interpreters. `rexx-parse` does not detect it, so it is detected at install
time where the accumulated table is what answers, and `Interp::blame_directive` supplies the
directive clause as the failure site so the echo line is the oracle's own. A translation error runs
in a first pass over every `::ROUTINE` before any install-time check, which is the order the oracle
uses.

---

## 5. Steps 5-8: `>I>` and `<I<`

### 5.1 The gate

Measured under all **nine** accepted trace letters, same routine each time
(`pI/own_<letter>.rex`): `a`, `i`, `l`, `r` announce both lines; `n`, `c`, `e`, `f`, `o` produce
zero stderr. Plus the five non-letter conditions in section 1.3, `trace off` after `trace l`
(entry only, no exit), the caller's `trace l` (nothing), a main body's own `trace l` (nothing), and
two calls into the same routine (two pairs).

**The bytes**, `cat -A`'d:

```
       >I> Routine "RTN" in package "/abs/path/own_l.rex".$
       <I< Routine "RTN" in package "/abs/path/own_l.rex".$
```

Seven blanks, the prefix, one blank, message 101018 rendered
(`interpreter/messages/rexxmsg.xml:6470`-`6471`), the trailing period outside the closing quote, no
trailing whitespace. The name is the directive's own spelling: `::routine 'zork'` announces
`"zork"` and `::routine MiXeD` announces `"MIXED"`, the second because the scanner upcases a bare
symbol before the directive parser sees it.

Endings, all measured, all announce `<I<`: `return`, `exit`, falling off the end, and an untrapped
condition -- where the line lands **before** the error report's clause echoes, which this crate
produces anyway because the report is written last.

### 5.2 The witness lives in the corpus, and `Coverage` grew a variant to say so

`corpus/lang/routine_dispatch.rex`, five blocks, its own header stating per block which wrong
answer each would print. It is in the live corpus because the `>I>` line names the package by its
**absolute path**: a committed `.expected` would be true on the machine that captured it and false
on the next, where `tests/corpus.rs` gives both interpreters the same canonicalised path in one
run.

Step 8 says to flip both prefixes to `Witnessed` and move both counts by two. `Witnessed`'s
contract is "a committed `.expected` under `tests/trace_oracle/` contains it", asserted by
`every_witness_still_emits_every_prefix_it_is_named_for`, and no such file can exist here. So
`Coverage` gained a third variant, `WitnessedLive(&'static str)`, and both counts moved by two as
specified: `WITNESSED_PREFIX_COUNT` 14 -> 16 (counting `Witnessed` and `WitnessedLive` together,
both compared against the oracle byte for byte), `OUT_OF_SCOPE_PREFIX_COUNT` 5 -> 3, and `"4c"`
left `OWNER_PHASES`.

`every_live_witness_emits_its_prefix_and_is_run_by_the_corpus` is the new variant's own chain,
three links: the program emits the prefix (run in process, so it goes red with no oracle on the
machine), the path appears in a phase subset file (so `corpus.rs` actually compares it), and the
file exists. It was verified to fail before `phase-4c.txt` listed the program.

`tests/coverage.rs`'s `assert_program_has_no_directives` became
`assert_program_has_only_routine_directives`, and `each_instruction` now descends into a
`::ROUTINE` body. Admitting a directive the walker did not follow would have opened exactly the
hole the guard names.

### 5.3 The one thing no differential harness can see

`support::normalize_stderr` (DEVIATION 0) collapses the space run between a trace line's marker and
its content, so both `corpus.rs` and `trace_oracle.rs` compare a clause echoed at indent 2 equal to
one echoed at indent 0. Measured: applying the caller's own `value_indent() + 2` to the routine
path leaves the **whole workspace** green.
`a_routines_own_clauses_echo_at_indent_zero_however_deep_the_call_site_is` asserts the exact
stderr and is the only thing that catches it.

---

## 6. Step 9

### 6.1 What was measured before

Reproduced all five of the brief's programs at `ulimit -v 1048576`, oracle rc 0 in every row and
`SIGABRT rc 134` here, with `memory allocation of 400000000 bytes failed` on stderr.

The instrument for locating each was `RUST_BACKTRACE=1` under the limit: the allocation failure is
a panic, so the backtrace names the exact file and line. Every site below was found that way, not
by reading.

### 6.2 The seven sites changed, and the witness for each

The guard is paired with the render rather than written beside it, which is the whole reason
`Interp::intermediate_text` and `Interp::result_text` are functions: a guard at a call site can
name the wrong `TraceMode` field and the failure mode is a trace line that silently stops printing.

For each, the mutation is "this site never renders, so its line never prints", applied one at a
time to the tree **with** the fix, full workspace, `--no-fail-fast`, 73 process headers in every
run. The witness is the existing test that goes red.

| site | gate | mutation result | catchers |
|---|---|---|---|
| the argument render, `>A>` | `intermediates` | 1259 passed / 3 failed | `call_arguments`, `function_call`, `task_9s_two_new_indents_...` |
| Assignment's `>>>`/`>=>` | `results` | 1246 / 16 | `trace_output`, `corpus_differential`, 14 others |
| `EXIT`'s and `RETURN`'s own `>>>` (one text, two sites) | `results` | 1255 / 7 | `exit_value`, `a_returned_value_traces_in_the_callee_and_again_in_the_caller`, `corpus_differential`, 4 others |
| `USE ARG`'s `>>>`/`>=>` | `results` | 1259 / 3 | `call_arguments`, `function_call`, `task_9s_two_new_indents_...` |
| `RAISE ... RESULT`'s `>K>` | `results` | 1261 / 1 | `corpus_differential` |
| the caller's own `>>>` after a `CALL` | `results` | 1260 / 2 | `corpus_differential`, `a_returned_value_traces_in_the_...` |

Where a site feeds both a `results`-gated and an `intermediates`-gated line, the gate used is
`results`: it is true wherever `intermediates` is, so it can never drop a line, and the cost of
choosing it is one redundant copy under `TRACE R`. **The opposite choice is caught**: swapping all
six `result_text` calls for `intermediate_text` gives 1248 passed / 14 failed.

`assign_expr_target`'s `rendered` parameter became `Option<&[u8]>` so the guard is visible at the
callee. `parse_template.rs`'s caller passes `Some` unconditionally and correctly -- it hands over a
borrow of the parse source, not a fresh copy, so there is nothing there for the `Option` to guard.

`eval.rs`'s seven intermediate renders needed nothing: `trace_intermediate` already returns at the
top when `!tracing_intermediates()`, which is the same guard one level up. I checked rather than
assumed.

**Sites deliberately not changed**, each because the rendered bytes are read rather than discarded:
`SAY` (goes to `Interp::out`), `PUSH`/`QUEUE`, `INTERPRET`'s text, `SELECT CASE`'s comparand,
`CALL (expr)`'s target, `SIGNAL VALUE`'s target, `RAISE`'s `DESCRIPTION`/`ADDITIONAL`/`ARRAY`
(stored on the condition), the `DO` header's `TO`/`BY`/`FOR`/`OVER` renders (fed to arithmetic and
to error substitutions), `eval_condition`, `NUMERIC VALUE`, `TRACE VALUE`, `ADDRESS VALUE`, and
PARSE's source and trigger renders. **The `DO` *control variable* renders were wrongly in that list
and are fixed in round 1** -- see M4 below; the arithmetic after them takes the `ObjRef`, not the
bytes, so they were the discard shape all along. Changing any of them is not a guard, and I owe no witness for a change
I did not make.

### 6.3 The third cause, measured and not closed

The brief's four neighbours are two shapes, and only one of them is a guard's job. Measured after
the fix, standard `ulimit -v 1048576`, oracle rc 0 in every row:

| program | this crate |
|---|---|
| `say length(strip(copies('a',400000000)))` | SIGABRT rc 134 |
| `say length(reverse(copies('a',400000000)))` | SIGABRT rc 134 |
| `say length(substr(copies('a',400000000),1,5))` | SIGABRT rc 134 |
| `say length(copies('a',400000000) || 'x')` | SIGABRT rc 134 |
| `say copies('a',400000000)` | SIGABRT rc 134 |
| `if copies('a',400000000) == '' then nop` | SIGABRT rc 134 |
| `parse value copies('a',400000000) with y` | SIGABRT rc 134 |

Producers, from each abort's own backtrace: `builtin::required_string`'s `into_owned()`,
`Interp::concat`, `Interp::eval_compare`'s `into_owned()` of both operands, `SAY`'s render into a
`Vec` before copying into `Interp::out`, and `parse_template`'s source render. In each the copy is
**read**, not discarded.

An earlier version of this paragraph said "every one holds THREE copies of the value where the
oracle holds two". The abort reproduces and the producers are named by the backtraces, but the copy
count is a summary I cannot check from outside the process -- the backtrace names the frame that
failed to allocate, not how many live buffers existed at that moment. The number is dropped and the
mechanism kept. No guard closes any of them: each exists to end a borrow of the
interpreter, because `to_text` borrows it and every consumer needs it mutably again. Closing it
needs `Interp` methods that read and allocate in one call. Recorded in `phase-4-exclusions.txt` as
the THIRD CAUSE, **unassigned** -- it is not a Phase 5 or Phase 7 feature, it is this crate's own
allocation discipline, and no phase has been given it.

### 6.4 The RSS re-measurement

`say length(copies('a',500000000))` at `ulimit -v 4194304`, where both sides succeed,
`/usr/bin/time -v`:

| | before | after |
|---|---|---|
| rexx-exec | 978,460 kB | **490,692 kB** |
| oracle | 495,860 kB | **496,364 kB** |

The row predicted parity from arithmetic and got it, landing 5.7 MB **under** rather than exactly
on. The oracle's own two numbers differ by 504 kB between runs, which is this instrument's noise
floor, so the 5.7 MB is about ten times that and is not noise; I have not chased it, and the
honest statement is that the redundant 482 MB copy is gone and the residue is unexplained.

**This measurement survives the `INTERPRETER_STACK_BYTES` reservation being free**, unlike the rc
table. `/usr/bin/time -v` reports resident pages; the 512 MiB reservation is address space and is
never resident. The 4 GiB `ulimit -v` here is a runaway guard only -- neither side comes near it.

The abort/no-abort rows in section 6.1 do **not** survive it: they are taken under `ulimit -v`, so
they charge this crate for a reservation the oracle does not make. What they show is that the abort
is gone at the limit this project uses, not that the two processes have equal room. The SECOND
cause is the remaining half of that and is D19's, untouched here.

### 6.5 The new test, and whether it adds coverage

`nothing_is_rendered_for_a_trace_line_that_will_not_print` asserts the property the guard adds
rather than the abort, which cannot be asserted from inside the process: an allocation failure
aborts rather than unwinding, so a test that provoked it would take the whole test binary with it.
It checks both functions under `N`, `R` and `I` -- `R` being the one mode in which they disagree,
so "always `None`" and "always `Some`" both fail.

Mutation: make `intermediate_text` render unconditionally. 1262 passed / 1 failed, and the one is
this test. Sole catcher.

---

## 7. Mutations, all runs

Every run is the full workspace with `REXX_CORPUS_GATE=1` and `--no-fail-fast`, **73** process
headers in all of them. Every restore is from a copy taken before the mutation and verified with
`md5sum -c` afterwards; no mutation harness restored from git.

| # | mutation | passed / failed | catchers |
|---|---|---|---|
| M1 | routine lookup in front of the builtin step | 1254 / 1 | `the_resolution_order_is_label_then_builtin_then_routine` |
| M2 | routine lookup does not upcase | 1254 / 1 | `a_routine_lookup_upcases_both_sides_where_the_builtin_lookup_does_not` |
| M3 | routine shares the caller's pool | 1253 / 2 | `..._own_pool_...`, `..._sets_no_sigl_...` |
| M4 | routine inherits NUMERIC/ADDRESS/TRACE | 1254 / 1 | `a_routine_inherits_none_of_the_five_a_called_label_inherits` |
| M5 | routine inherits traps and condition | 1254 / 1 | `a_routine_does_not_inherit_the_callers_condition_traps` |
| M6 | `set_sigl` on the routine path too | 1254 / 1 | `a_routine_call_sets_no_sigl_where_a_called_label_does` |
| M7b | duplicate `::ROUTINE` not refused | 1254 / 1 | `a_duplicate_routine_directive_is_99_903_before_the_first_clause` |
| M8 | no excluded-builtin step | 1253 / 2 | new `eval` test **and** `the_exempt_set_matches_the_current_failures` |
| M9 | 43.1 back to the loud gap | 1249 / 6 | 5 unit tests + the exempt set |
| M10 | refuse on `::ROUTINE` presence | 1248 / 7 | `an_uncalled_routine_directive_changes_nothing` + 6 |
| M11 | `directive_gap` always `None` | 1256 / 1 | `every_directive_this_crate_cannot_install_refuses_...` |
| M12 | `directive_gap` always a gap | 1248 / 9 | `every_directive_this_crate_can_install_leaves_...` + 8 |
| M13 | never emit `>I>` | 1257 / 4 | both invocation tests, `corpus_differential`, `every_live_witness_...` |
| M14 | never emit `<I<` | 1257 / 4 | same four |
| M15 | `>I>` ignores `tracingLabels()` | 1260 / 1 | `..._fire_for_exactly_the_four_label_tracing_letters` |
| M16 | never clear `trace_entry_allowed` | 1260 / 1 | `..._are_gated_on_more_than_the_trace_letter` |
| M17 | a label does not clear it | 1260 / 1 | same |
| M18 | `<I<` does not re-read the setting | 1260 / 1 | same |
| M19 | `EXIT` in a routine ends the program | 1258 / 3 | `exit_inside_a_routine_...`, `corpus_differential`, `every_live_witness_...` |
| M20 | routine clause indent = caller + 2 | **1261 / 0** | none -- see below |
| M20b | the same, after the exact-stderr test landed | 1261 / 1 | `a_routines_own_clauses_echo_at_indent_zero_...` |
| S1-S7 | section 6.2 | see that table | existing tests only |
| S8 | six `result_text` -> `intermediate_text` | 1248 / 14 | 14 existing tests |
| S9 | `intermediate_text` renders unconditionally | 1262 / 1 | the new Step 9 test |

**M20 is the finding in this table.** The routine's indent-0 clause echo passed every harness
including the live corpus, because DEVIATION 0 normalises exactly that difference away. It is the
one place a corpus witness could not do the job, and the exact-stderr unit test was written because
of it rather than in anticipation.

**M8's honest reading**: the new `eval` test is not the sole catcher.
`keyword_assertions.rs`'s exempt-set test also goes red, because the derived owner for
`CALL::test_on_name` and `CALL::test_9` would move. The new test is kept anyway -- it is in-crate,
corpus-independent, and states the rule directly -- but it does not add coverage the suite lacked.

`corpus/keyword-exempt.txt` moved `CALL::test_literal` and `CALL::test_expression` from `4c` to
`RAISED`. Both call `::routine`s the ooTest extraction dropped, so both now agree with the oracle,
which raises Error 43 on them too; the file's own header already recorded that and is corrected to
say so. `CALL::test_on_name` and `CALL::test_9` stayed at `4c` because
`is_excluded_builtin` keeps `CHARIN`/`LINEIN` loud.

---

## 8. Verification

From `rust/`, at the final commit, after `cargo clean -p rexx-exec -p rexx-parse` so the lint
re-examined the code rather than reusing a warm result:

```
cargo fmt --all --check                                   exit 0
cargo clippy --workspace --all-targets -- -D warnings     exit 0
REXX_CORPUS_GATE=1 cargo test --offline --workspace --no-fail-fast
                                                          exit 0
```

1263 passed / 0 failed. 73 `Running`/`Doc-tests` process headers. Corpus **50 of 50** matching, up
from 49 of 49 -- the one new program is `lang/routine_dispatch.rex`.

Each exit status was read unpiped from its own command, never from a pipeline.

Tree at the start: 1246 passed / 0 failed, 73 headers, 49 of 49. The 17 net new tests are 15 in
`run.rs`, 1 in `eval.rs` (a second one replaced an existing test rather than adding to it), and 1
in `trace_oracle.rs`. A first draft of this sentence said 11/2/1/3, which was a count written from
memory rather than from the list; the arithmetic happened to reach 17 anyway, which is how that
kind of error survives.

---

## 9. Concerns

1. **`::REQUIRES` of an existing file is refused where the oracle runs it.** Section 1.1 has the
   evidence and the reasoning. It contradicts a measured row in the brief and it is the largest
   single judgement call in this task. If the phase would rather diverge silently on q1-shaped
   programs than refuse o4-shaped ones, this is one arm of `directive_gap` to flip.
2. **`::CLASS ... SUBCLASS object` is refused** for the same class of reason, and is a more common
   program than `subclass zzznotaclass`.
3. **The third memory cause is unassigned.** It is a real divergence -- seven measured programs
   abort where the oracle answers -- and it is not another phase's feature, so nobody currently
   owns it. It needs a decision, not a task.
4. **Commit 3 carries two changes, for convenience and not for the reason first given here.**
   `EXIT`-in-a-routine (`run.rs:3846`-`3866`) and the trace pair (`:6378`, `:6406`) touch disjoint
   code. The only thing coupling them is `corpus/lang/routine_dispatch.rex`, which **commit 3 itself
   authored** -- so splitting would have meant writing the witness with one block fewer and adding
   the block in the second commit, not leaving the corpus red. The original sentence here said the
   corpus would have gone red, which was false: it described a constraint that would only exist if
   the witness had predated the commit. The practical cost of the split was near zero and I did not
   pay it. (Recorded rather than quietly amended because it is this phase's recurring shape -- a
   correct decision shipped with a false justification, and the justification passes review on the
   decision's merits.)
5. **The `>I>` witness puts an absolute path in corpus output**, which the `phase-4c.txt` header
   otherwise forbids. It is sound -- both interpreters get the same path in the same run -- and the
   header now carries the exception and the reason, but it is a rule with one hole in it now.
6. **`earlyTraceEntry` is not implemented** and no probe in Phase 4 can separate it from the
   `setTrace` route. If Phase 5 adds guarded methods, that separation becomes observable and this
   is where to look.


---

# Fix round 1

Two Criticals and three Minors, all addressed. One commit:
`34eca951e802f5cc02d488e96a63859ba43a3b62`.

## C1: the decay was counting top-level instructions, not clauses

**The mechanism.** `TraceEntry`'s "am I still on the activation's first instruction" was spent in
`run_activation`'s own loop, which visits only *top-level* instruction positions. An `IF`
then-branch, a `DO` body clause, a `WHEN` body and a fragment's clauses all run through
`run_bounded`/`run_fragment` **inside** the enclosing instruction's `step`, so a `TRACE` in one of
them reached `trace_invocation_entry` before the clear ever fired.

The C++ lines followed:

* `RexxActivation.cpp:657`-`659` -- `if (current->getType() != KEYWORD_EXPOSE) traceEntryAllowed =
  false;` at the bottom of the instruction loop. An `IF`'s then-clause and a `DO` body clause are
  each their own `RexxInstruction` in that same flat loop, which is why the C++ needs no special
  case and this crate does: here they are nested.
* `RexxActivation.cpp:3644`-`3652` -- the `isInterpret()` arm, `traceEntry = tracingLabels() &&
  parent->isMethodOrRoutine()`, then `&& parent->traceEntryAllowed && !parent->traceEntryDone`.
* `RexxActivation.cpp:3664` -- `if (isInterpret()) parent->traceEntryDone = true`, which is why an
  announcement made inside a fragment still owes an `<I<` after the fragment ends.

**The fix.** The decay moved to `step_in_temps_frame`, which the file's own doc already calls the
one place a clause is stepped -- counting nested bodies, a callee's clauses and a fragment's. It
runs at the *top*, before the clause, because `Pending -> Allowed` is what a `TRACE` in that very
clause must see. `trace_entry_allowed`/`trace_entry_done` became one `TraceEntry` value, because
they are read together at every site and the pair is what let the "allowed" half be spent from a
place the "done" half could not see.

**`INTERPRET` needed more than the coordinator's shape, and n11 is why.** Following only
"give a fragment its own copy starting true, ANDed with the enclosing activation's saved value"
announces for `interpret "interpret 'trace l'"`, and the oracle writes zero bytes there. The reason
is the `parent->isMethodOrRoutine()` conjunct: an interpret activation is not a method or routine,
so a fragment inside a fragment can never announce however its own clauses fall.
`Interp::enter_fragment` takes that as its `nested` argument, read from the enclosing
`clause_line_override` -- which is already exactly "a fragment is running", is already saved at
that call site, and is already cleared for a callee (so a routine called *from* a fragment can
announce for its own `INTERPRET`, which is what the C++ parent chain gives too). No new state.

**Measured, 31 programs in a directory created for this round**, all matching byte for byte on all
three channels. The twelve that cross the two axes:

```
routine body                              oracle stderr   crate
if 1=1 then trace l                       -               - (was: both lines)
if 1=0 then nop; else trace l             -               -
do 1; trace l; end                        -               -
do i = 1 to 1; trace l; end               -               -
select; when 1=1 then trace l; end        -               -
if 1=1 then trace r                       "     5 *-* return"   same
interpret "trace l"                       both lines      both lines
interpret "nop; trace l"                  -               -
n0 = 0 / interpret "trace l"              -               -
if 1=1 then interpret "trace l"           -               -
interpret "interpret 'trace l'"           -               -
n0 = 0 / if 1=1 then trace l              -               -
```

The `if 1=1 then trace r` row is in the test as well as the silent ones: the setting still takes
effect for the clauses after it, so "announced nothing" is separated from "the TRACE did nothing".

**The test crosses the axes.**
`the_invocation_prefixes_are_not_announced_from_inside_a_nested_construct` is twelve rows of exact
stderr, eleven silent and one announced, and it is the neighbouring announced row (`trace l` flat)
that pins the silences to the nesting rather than to something else about the programs.

**Mutation transcript.** Full workspace, `REXX_CORPUS_GATE=1`, `--no-fail-fast`, **73** headers in
every run. Restores from a copy taken before each mutation, every one verified with `md5sum -c`
against `run.rs`, `activation.rs`, `lib.rs` and `eval.rs`.

| mutation | passed / failed | catchers |
|---|---|---|
| C1 -- the decay back in the top-level loop only (the pre-fix behaviour) | 1263 / 1 | the new test, alone |
| C1b -- fragment ignores the `nested` conjunct | 1263 / 1 | the new test, alone |
| C1c -- fragment gets no count of its own, just the enclosing decay | 1263 / 1 | the new test, alone |

Green before and after each: 1264 / 0.

## C2: the four false comments

None of the four had been touched, and the report claimed otherwise. Before and after:

**`lib.rs`, `Loud::unresolved_call`'s doc.**
Before: *"not an internal label of the calling body, so the next steps are the builtin table and
then external resolution, and both are 4c's."*
After: the constructor is re-scoped to what it now answers -- a builtin Phase 4 excludes outright,
where the oracle answers normally and this crate has no code -- with the four-step order stated and
external resolution named as **Phase 7's**. The paragraph also now says why a condition would be
the wrong answer for an excluded builtin, which is the reason the step sits in front of the
`::ROUTINE` lookup.

**`lib.rs`, `instruction_owner`'s `Call` arm.**
Before: *"A named call that resolves to no internal label still fails loudly, through
`Loud::unresolved_call`, naming `4c`: the builtin and external steps behind the label search are
that phase's."*
After: it raises the oracle's own 43.1; the one step behind the three that this crate skips is the
external file search, which is Phase 7's; so there is no residual claim on the `CALL` keyword.

**`lib.rs`, `expr_owner`'s `Call` arm.** Same sentence, same correction.

**`eval.rs`, `eval_call`'s doc, two paragraphs.**
Before: *"resolution is internal routine first (4b), builtin second (4c), external third (Phase
7)"* -- three steps, no `::ROUTINE`; and *"this stays the same loud `4c` fallback `CALL "SUB"`
already gets, not a fabricated 43.1"*, which described the behaviour as it was before this task.
After: four steps with `::ROUTINE` third and Phase 7 owning the file search; and the literal-target
paragraph now says 43.1 **is** this crate's answer, because with the two steps behind the label
search built, "not a label" and "not anything" are separable. That second paragraph is the mirror
shape the review named: the test five lines below it was updated in the original round and the
production doc above it was not.

`grep` for `both are 4c's`, `external steps behind the label search are that phase's` and
`fabricated 43.1` across `crates/rexx-exec/src/` returns nothing.

## M3: two counts of call sites in this repository

`plan.rs`'s `BodyKey::directive` said *"Two production sites, one per arm"* and
`activation.rs`'s `body` said *"pushed by exactly one place"*. Both are true today, which is the
failure mode `rust/CLAUDE.md` names. Both now name the sites without counting them.

## M4: two `DO` control-variable renders, and a false description

`run.rs`'s controlled-loop re-test rendered the control variable twice (once for the
`>V>`/`>>>` pair, once for the re-tested `>>>`) with no guard, and the arithmetic after them takes
the `ObjRef` rather than the bytes -- so they were the discard shape and the report's own
not-changed list described them wrongly. Both now go through `result_text` (`results` being the
weaker gate of the `>V>`/`>>>` pair). Exposure is nil, since a control value is necessarily
numeric, so this is consistency with `bind_control`'s own existing guard rather than a second
abort closed.

Witnessed, both sites, same protocol:

| mutation | passed / failed | catchers |
|---|---|---|
| the re-test `>V>`/`>>>` render never happens | 1259 / 5 | `controlled_loop`, `control_variable_reread`, `control_variable_novalue`, `corpus_differential`, `task_9s_two_new_indents_...` |
| the re-tested `>>>` render never happens | 1259 / 5 | the same five |

The report's §6.2 not-changed list is corrected in place.

## M5: the absolute-path exception now sits at the rule

`phase-4c.txt`'s determinism rule carries the carve-out directly, saying that the path is the
oracle's own doing in its `>I>`/`<I<` lines, that it stays comparable because both interpreters get
the same canonicalised path in one run, and that a program which *prints* a path itself is still
forbidden.

## Verification

From `rust/`, statuses read unpiped from their own commands:

```
cargo fmt --all --check                                   exit 0
cargo clippy --workspace --all-targets -- -D warnings     exit 0
REXX_CORPUS_GATE=1 cargo test --offline --workspace --no-fail-fast
                                                          exit 0
```

**1264 passed / 0 failed, 73 headers, corpus 50 of 50.** Baseline before this round was 1263 / 73 /
50-of-50; the one new test is C1's.

## One thing this round found that is not in the review

**Three probe files in the scratchpad had been overwritten by later probe batches with colliding
names**, and the first sweep of this round reported them as regressions. `pJ/j1.rex` held the
external-routine program, `pF/f3.rex` held `::class foo subclass object`, and `pF/f4.rex` held a
`::routine charin` shadowing check -- all three expected divergences, none of them the program the
filename had been measured under. Every sweep in this round was therefore re-run from directories
created and written fresh (`pFIX`, 31 programs; `pR2`, 12 programs), and the numbers above are from
those. This is the scratchpad hazard the brief warns about, arriving through filename collision
rather than through the external-routine search path.
