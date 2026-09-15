# Phase 8, Task 10: the `rxregexp` cases and the L2 measurement

Everything here was run on this host on 2026-09-14, against `rust/target/release/rexx-run` built
from the tree this document is committed with, and against the oracle at
`/home/moritz/dev/repos/ooRexx/build/bin/rexx`. Every oracle run used the standard wrapper
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib <binary> FILE )` from
a directory created empty for that run, with `timeout` on both sides, and stdout, stderr and exit
status kept on three descriptors. No `2>&1` appears anywhere below.

**L2 is not reached.** The rung reads *"`ooTest.frm` loads and a single test group executes"*, and
the framework now gets one step further than Phase 7 left it and stops somewhere new. Section 3 has
the blocker and section 4 says who owes it.

## 1. The extracted `rxregexp` cases

The set is what `ls rust/corpus-l1/ | grep rxregexp` names:

```
rxregexp_test_bug_1045.rex                       rxregexp_test_pattern_csv.rex
rxregexp_test_new_arg_one_null.rex               rxregexp_test_pattern_illegal_number.rex
rxregexp_test_new_arg_two_invalid.rex            rxregexp_test_pattern_illegal_set.rex
rxregexp_test_new_arg_two_maximal.rex            rxregexp_test_pattern_illegal_symbolic_name.rex
rxregexp_test_new_arg_two_minimal.rex            rxregexp_test_pattern_unexpected_eop.rex
rxregexp_test_new_no_args.rex                    rxregexp_test_pattern_unexpected_symbol.rex
rxregexp_test_new_three_args.rex                 rxregexp_test_position_one_arg.rex
rxregexp_test_parse_arg_one_null.rex             rxregexp_test_pos_no_args.rex
rxregexp_test_parse_arg_one_omitted.rex          rxregexp_test_pos_two_args.rex
rxregexp_test_parse_no_args.rex
rxregexp_test_parse_three_args.rex
```

### 1a. Run as they stand, they execute nothing

`rexx-extract` wraps each extracted method body in `::routine main public` and appends an assert
shim, and it emits no main section, so a file's whole content is directives. Running one is
therefore a no-op on either interpreter, and that is measured rather than reasoned:

```bash
for name in $(ls rust/corpus-l1/ | grep rxregexp); do
  d=$RUNROOT/${name%.rex}; mkdir -p "$d"; cp rust/corpus-l1/$name "$d/t.rex"
  ( cd "$d" && ( ulimit -v 1048576; LD_LIBRARY_PATH=$ORACLELIB timeout 30 $ORACLE/bin/rexx t.rex \
      </dev/null ) >o.out 2>o.err; printf '%s rc=%s out=%s err=%s\n' \
      "$name" "$?" "$(wc -c < o.out)" "$(wc -c < o.err)" )
done
```

Every row came back `rc=0 out=0 err=0`, and the set of files that wrote a byte to either descriptor
or exited non-zero, printed by the same loop, was empty. A differential over them as they stand
would compare two silences, which is why this task drove them instead.

### 1b. How they were driven

Two things stand between a file and a run, and both were measured before the driver was written.
The body is unreachable because nothing calls the routine, and the body's assertions are sent to
`self`, which is an ordinary uninitialised variable inside a `::ROUTINE` rather than a bound
receiver. Every file in the set sends to it: the loop
`for f in $(ls rust/corpus-l1/ | grep rxregexp); do grep -q -- 'self~' "$f" || echo "$f"; done`
printed nothing. Left alone, the oracle reports
`Error 97.1: Object "SELF" does not understand message "ASSERTEQUALS"` at rc 159.

The driver is a transform into a copy in the scratchpad, never an edit of `rust/corpus-l1/`:

```bash
{ echo 'call main'
  echo "::requires \"$REPO/extensions/rxregexp/rxregexp.cls\""
  sed "s|^::routine main public\$|::routine main public\n  self = .shim~new|" \
      "$REPO/rust/corpus-l1/$name"
} > t.rex
```

`::requires` takes an absolute path because this crate is installed nowhere and `rxregexp.cls`
ships with the interpreter, which is the same constraint Phase 7's close recorded. Both sides are
then run with `LD_LIBRARY_PATH` pointing at the oracle checkout's `build/lib`
(`/home/moritz/dev/repos/ooRexx/build/lib`, not this worktree's `build/lib`, which holds a second
build from the same sources), which is where `librxregexp.so` is; the library is loaded, never
rebuilt.

### 1c. The failing set, before

Driven this way at `d39dc3194`, five of the cases disagreed with the oracle, and the failing set was
printed rather than counted:

```
rxregexp_test_parse_arg_one_omitted.rex
rxregexp_test_parse_no_args.rex
rxregexp_test_parse_three_args.rex
rxregexp_test_pos_no_args.rex
rxregexp_test_pos_two_args.rex
```

All five had the same exit status on both sides and differed in one byte range of one stderr line.
`rxregexp_test_pos_no_args` is representative:

```
       *-* Compiled method "POS" with scope "RegularExpression".
     8 *-* p~pos
     1 *-* call main
Error 88 running <ORACLE NAMES rxregexp.cls / WE NAMED t.rex>:  Invalid argument.
Error 88.901:  Missing argument; argument 1 is required.
```

### 1d. The defect, cut to two files

The five all reach the same site, and the smallest program that shows it puts the `EXTERNAL`
directive in a package a `::REQUIRES` loaded:

```rexx
/* re.cls */
::class Re public subclass Object
::method init external "LIBRARY rxregexp RegExp_Init"
::method uninit external "LIBRARY rxregexp RegExp_Uninit"
::method doparse external "LIBRARY rxregexp RegExp_Parse"
```

```rexx
/* t.rex */
r = .Re~new('a*b')
say r~doparse()
::requires 're.cls'
```

Oracle rc 168, `Error 88 running <dir>/re.cls:  Invalid argument.`; this crate rc 168, the same
report naming `<dir>/t.rex`. **A raise the native boundary makes for itself is reported against the
package the refused method was declared in, and we were reporting the running program.** The rule is
not "the innermost native frame's package", and that was settled by running the adjacent case rather
than assumed: the same required package's `RegExp_Init` raising 38.0 for a template it cannot parse
is `Error 38 running <dir>/t.rex line 1` on the oracle, the program and its line, because the
extension raised that condition itself and it propagated to the caller.

The two halves line up exactly with the report's existing `Delivery::lineless` flag, which was
already set on the boundary's own argument raises and on nothing else. So the flag now decides the
reported name as well as the suppressed line, which is one decision rather than two:
`FailureSite::Rendered` carries the package the `EXTERNAL` directive was written in,
`Interp::external_packages` records it at install time for both the `LIBRARY REXX` and the
other-library forms, and `Raised::report` prefers it when the delivery is `lineless`. Since
`03ceb04df` the flag is also set on the boundary's 88.909 (`native_argument_needs_a_string_value`)
and on the parameter-side 93.968 (`incorrect_method_signature`), while the result-side 93.968
(`incorrect_method_result_signature`) keeps its line against the sender, as measured through a
forged extension (the ledger's `final-fix-report.md`, F2).

Three neighbouring shapes were measured to bound the change, each on two files with the declaration
in the required package:

* `::METHOD SEP CLASS EXTERNAL "LIBRARY REXX file_separator"` sent an argument it does not take is
  `Error 88 running <dir>/k.cls` with no line, the same rule. This crate named the program before
  the change and names the package after it.
* `'abc'~length(1)` is `Error 93 running <dir>/u.rex line 1` on both sides, before and after. An
  internal native method was declared by no directive, so nothing records a package for it and the
  report falls through to the program, which is what the oracle prints.
* A written `::METHOD` in a required package raising 93.901 already named `<dir>/lib.cls line 3` on
  both sides, and still does. That path goes through `required_package_site`, which this change does
  not touch.

### 1e. The witnesses and the negative control

Three corpus programs went in, each with a `.d/` holding the required package, the first two with a
`.env` putting the oracle's library directory on the in-process side's search path (the third loads
no shared object, and its `.env`, inert, was removed at `cf92ff4fb`):

* `lang/library_method_package_blame.rex` -- the boundary raise, reported against the package.
* `lang/library_method_program_blame.rex` -- the extension's own raise from the same package,
  reported against the program and its line. This is the neighbour that pins the rule to the
  boundary rather than to "a native method is on the stack".
* `lang/external_method_package_blame.rex` -- the `LIBRARY REXX` form of the first.

**Prediction, written before the control was run:** with the package lookup disabled, the first and
third diverge, the second still matches, and no other corpus program changes. Replacing
`if self.delivery.lineless` with `if false` in `Raised::report` and running
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` gave exit 101 and

```
529 of 531 matching
mismatches (2):
  [UNCLASSIFIED] lang/library_method_package_blame.rex: stderr differ
  [UNCLASSIFIED] lang/external_method_package_blame.rex: stderr differ
```

All three parts **confirmed**: the two named programs are the two mismatches, the third is not among
them, and the total is otherwise unchanged. The edit was reverted by re-editing.

### 1f. The failing set, after

The same driver over the same set, at the tree this document is committed with, printed `AGREE` for
every row, and the set of rows the comparison called `DIVERGE` was empty. The comparison is stdout,
stderr and exit status as three files through `cmp`.

## 2. The chain, walked again

Phase 7's close left three steps. Each was re-run, in that order, and then the driver itself.

**Step 1, `.ENDOFLINE`.** `say 'endofline' c2x(.endOfLine)` is `endofline 0A` at rc 0 on both sides.
Phase 7 built it and it is still there. This is no longer a blocker.

**Step 2, `rxregexp.cls`.** A program whose only content is
`::requires "<repo>/extensions/rxregexp/rxregexp.cls"` is rc 0 on both sides, stdout equal, stderr
empty. The absolute path is still required for the reason Phase 7 gave.

**Step 3, `::METHOD INIT EXTERNAL "LIBRARY rxregexp RegExp_Init"`.** Passed. Step 2's run installs
that directive and no longer stops on it, and section 1's cases exercise `RegExp_Init`,
`RegExp_Parse`, `RegExp_Pos` and `RegExp_Match` end to end against the oracle. **This is the part
Phase 8 owed, and the chain no longer stops here.**

**Step 4, `ooTest.frm`.** The framework loads its two dependencies by bare name, so this was run
from a directory holding a copy of each:

```bash
cp $REPO/extensions/rxregexp/rxregexp.cls $REPO/ootest/framework/OOREXXUNIT.CLS .
cat > t.rex <<EOF
say 'ooTest.frm loaded, version' .ooTest_Framework_version
::requires "$REPO/ootest/ooTest.frm"
EOF
```

Oracle: rc 0, `ooTest.frm loaded, version 1.0.1_4.0.0`, stderr empty. This crate: rc 120, stdout
empty, stderr `rexx-exec: method "HASENTRY" of class "Directory" is not implemented (Phase 5)`.
`::requires "OOREXXUNIT.CLS"` and `::requires "rxregexp.cls"` are directives of `ooTest.frm` and are
installed before its prologue runs, so reaching the prologue at all is evidence that both completed.

**Step 5, the driver with one test group.** `testOORexx fileName` is the framework's own single-group
form:

```bash
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout 120 \
  <binary> $REPO/ootest/testOORexx.rex \
  $REPO/ootest/ooRexx/extensions/rxregexp/rxregexp.testGroup </dev/null )
```

The oracle runs it, rc 0, stderr empty:

```
Searching for test containers..
Executing automated test suite..

ooTest Framework - Automated Test of the ooRexx Interpreter

Interpreter:        REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026
OS Name:            LINUX
SysVersion:         Linux 7.1.12+deb14-amd64

Tests ran:          33
Assertions:         11741
Failures:           0
Errors:             0
```

This crate: rc 120, stdout empty, stderr the same one line as step 4.

## 3. Where it stops, exactly

The stop is `ooTest.frm:49`, the first executable clause of the framework's own prologue. A `Loud`
refusal carries no traceback, so the site was found by running rather than by reading: a copy of
`ooTest.frm` in the scratchpad with `::options trace i` appended, required from a driver program,
ends

```
    49 *-* if \ .local~hasEntry('OOTEST_FRAMEWORK_VERSION') 
       >E>   .LOCAL => "The Local Directory"
       >L>   "OOTEST_FRAMEWORK_VERSION"
       >A>   "OOTEST_FRAMEWORK_VERSION"
rexx-exec: method "HASENTRY" of class "Directory" is not implemented (Phase 5)
```

**The blocker is not one method.** `.local` and `.environment` are `Directory`s the bootstrap builds
on `NativeObject`'s own entry map rather than on the hash store, and `dispatch::hash`'s `owns` turns
every store-backed entry point away from such a receiver. Sending each name to `.local` as a
one-line program, oracle against this crate:

| send | oracle | this crate |
|---|---|---|
| `at('NOPE')` | `The NIL object` | `The NIL object` |
| `put(1,'ZZ')` | 91.999 | 91.999 |
| `hasIndex('NOPE')` | `0` | refuses, `HASINDEX` |
| `entry('NOPE')` | `The NIL object` | refuses, `ENTRY` |
| `hasEntry('NOPE')` | `0` | refuses, `HASENTRY` |
| `setEntry('ZZ',1)` | 91.999, having stored | refuses, `SETENTRY` |
| `items` | `10` | refuses, `ITEMS` |
| `index(1)` | `The NIL object` | refuses, `INDEX` |
| `remove('ZZ')` | `The NIL object` | refuses, `REMOVE` |
| `supplier` | `a Supplier` | refuses, `SUPPLIER` |
| `allIndexes` | ten lines, `SYSCARGS` first | refuses, `ALLINDEXES` |

Each program is `say .local~<send>` with the send as the row spells it. The `allIndexes` answer,
re-run 2026-09-15, is `SYSCARGS`, `INPUT`, `TRACEOUTPUT`, `DEBUGINPUT`, `STDOUT`, `OUTPUT`,
`STDERR`, `STDIN`, `STDQUE`, `ERROR`, one per line; the table's first version showed the first
line without saying so. Every refusal in that column names `Phase 5`. The rows that agree are
`at`, which answers, and `put`, which stores and then trips 91.999 on the `say`; those are what the
environment seam already serves. The oracle's `setEntry` row is 91.999 for the same reason and is a
send that worked.

## 4. Who owes it

**Phase 5, as an unpaid debt rather than as open work**, and the tree already says so.
`crates/rexx-exec/tests/closed_phases.rs`'s `CLOSED` list holds `Phase 7` alone, and its own doc
comment explains the omission: *"`Phase 5` is deliberately absent, and its absence is a debt rather
than a licence. That phase closed leaving refusals that name it -- a send to an unimplemented
`Directory` method is one -- and re-homing them was never Phase 7's task."* The blocker on the L2
rung is exactly the example that comment names.

So the rung is not blocked on anything Phase 8 was scoped to build, and it is not blocked on an open
phase either. **No phase currently in the plan owns the work that would unblock it.** That is the
finding, and it is the one thing in this document that a reader should act on: the roadmap's `Rung`
column cannot move L2 to another phase's row until some phase takes the `Directory` protocol on the
environment directories.

*Acted on 2026-09-14:* Moritz ruled that Phase 8 takes it, recorded in D-L2 and carried as Task 1 of
`docs/superpowers/plans/2026-09-14-phase-8-surface.md`. This section is the finding as it was made.

## 5. Three other divergences the walk found, two of them not Phase 8's

Each was reduced to a two-file program and measured on both sides. None was fixed here; the third
was partly Phase 8's, and its library rows were fixed at `40093e99b`, below.

**The `>I>`/`<I<` package name.** A routine or method of a required package announces itself under
the running program's path where the oracle names the package. Measured with a `::ROUTINE HELPER` in
a `lib.cls` carrying `::options trace i`, required from `t.rex`:

```
oracle:  >I> Routine "HELPER" in package "<dir>/lib.cls".
this:    >I> Routine "HELPER" in package "<dir>/t.rex".
```

Every caller of `Interp::trace_invocation` in `run.rs` passes `self.program_path` unconditionally
(`git grep -n 'trace_invocation(' crates/rexx-exec/src`).
The fix is not the same one section 1d made: the correct name has the same three-way shape the error
report already has (a package with no source, a body compiled from source text, a required package),
and the oracle's spelling for the first two under this line is unmeasured. This is a silent wrong
answer rather than a refusal, so nothing in the tree records it; that is why it is written down here.

**The required package's own prologue announcement.** On the same program the oracle emits a
`>I>`/`<I<` pair for `lib.cls`'s prologue, named by its file, before the pair for `HELPER`. This
crate emits neither. Same family, same owner.

**Directive-time attribution in a required package.** An error raised while installing a directive of
a required package carries the right clause and line and the wrong file. Measured with
`::class A public subclass NoSuchClassHere` in a `k2.cls`:

```
oracle:  Error 98 running <dir>/k2.cls line 1:  Execution error.
this:    Error 98 running <dir>/t2.rex line 1:  Execution error.
```

`Interp::blame_directive_in` exists and does the right thing; some install paths call
`Interp::blame_directive` instead. The 98.909 above is not a Phase 8 error, which is how this was
shown to be wider than the boundary: the same shape appears on
`::method len class external "LIBRARY REXX xxx_no_such"`, whose 90.998 is Phase 8's, and on a class
directive whose superclass does not resolve, which is nobody's. **What the final review added
(B6):** the rows that name a library other than `REXX` in a required package -- `::method x
external "LIBRARY zorkolib z"` (98.903), `"LIBRARY rxregexp NoSuchEntry"` (90.998), and the
`::routine` (90.999) and `::attribute` forms -- were loud refusals naming Phase 8 at `659312de0`
and became this divergence in this slice, so that part was Phase 8's; `40093e99b` made
`resolve_directive_library` blame through `blame_directive_in`, and the four two-file witnesses
`library_required_*_missing.rex` pin it. What still names the program, re-measured 2026-09-15, is
the `LIBRARY REXX` form (`::method x external "LIBRARY REXX nosuchentry"` at `pk.cls` line 3:
oracle `Error 90 running <dir>/pk.cls line 3`, this crate `<dir>/main.rex line 3`, 90.998 and rc
166 on both) and the superclass form above (`k2.cls line 1` against `main.rex line 1`, 98.909, rc
158 on both), through the `unresolved_external` and class-install arms that predate this phase.

## 6. A hazard for whoever drives the framework next

`testOORexx.rex` with no arguments runs the whole suite, and the suite writes fixture files into
`ootest/` as it goes. An oracle run of that form left artefacts under `ootest/ooRexx/`; `svn status`
in `ootest/` afterwards shows them as `?` and reports no modified versioned file, and `ootest/` is
in `.gitignore`, so nothing reached the repository's own state. They were left in place rather than
deleted, because the tree is read-only for this phase and artefacts of the same kind predate this
session. **Prefer the single-group form**, which wrote nothing outside its own run directory, or a
copy of the suite.

## 7. Readings

Recorded from the run, unpiped, at the tree this document is committed with.

| command | exit | figures |
|---|---|---|
| `cargo fmt --all` | 0 | |
| `cargo clippy -j 4 --workspace --all-targets -- -D warnings` | 0 | |
| `cargo test -p rexx-api` | 0 | |
| `cargo test -p rexx-core --test unsafe_sites` | 0 | |
| `cargo test -p rexx-parse --test sourceline_oracle` | 0 | |
| `cargo test -p rexx-exec --lib` | 101 | 814 passed / 3 failed |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | **531 of 531** |

`cargo test -p rexx-exec --lib`'s failing set was printed rather than counted, and every member
predates this task: `ir::drive::tests::a_call_site_resolves_once_and_answers_from_what_it_kept`,
`ir::drive::tests::a_long_constant_is_built_once_however_many_passes_read_it` and
`ir::drive::tests::the_ir_engine_steps_an_ifs_chosen_branch_from_the_chunk`.

The four phase gates are Task 11's and are not run here.
