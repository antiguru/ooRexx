# Task 10 report: the 20 `rxregexp` cases, and the L2 measurement

Committed at `c9616b3d0e28143312ef256b44da86372ca77034`, one commit, on top of
`d39dc3194`. The report itself is not committed: `.gitignore:30` ignores
`.superpowers/`.

**The measurement is `docs/superpowers/plans/phase-8-l2.md`, written from the
run.** This report is the working record behind it and does not repeat it.

Files added: `rust/corpus/lang/library_method_package_blame.rex`, `.env` and
`.d/re.cls`; `rust/corpus/lang/library_method_program_blame.rex`, `.env` and
`.d/re.cls`; `rust/corpus/lang/external_method_package_blame.rex`, `.env` and
`.d/k.cls`; the three matching
`rust/crates/rexx-parse/tests/sourceline_oracle/*.txt`;
`docs/superpowers/plans/phase-8-l2.md`.
Files modified: `rust/corpus/phase-8.txt`,
`rust/crates/rexx-exec/src/dispatch.rs`,
`rust/crates/rexx-exec/src/error.rs`,
`rust/crates/rexx-exec/src/lib.rs`.

Nothing under `ootest/`, `oodocs/`, `samples/`, `build/` or the C++ tree was
modified. `svn status` in `ootest/` reports no versioned file changed; see
section 6 of the measurement for the artefacts the framework's own full-suite
run left there.

---

## 1. Headline

**L2 is not reached, and the chain stops somewhere new.** Phase 8's own step of
it is passed: `::METHOD INIT EXTERNAL "LIBRARY rxregexp RegExp_Init"`, which is
where Phase 7's close left the framework, no longer stops anything. The new
stop is `ooTest.frm:49`, `.local~hasEntry('OOTEST_FRAMEWORK_VERSION')`, and it
is not one method but a family: the `Directory` protocol on the environment
directories, every member of which refuses naming `Phase 5`.

**No open phase owns that work.** `closed_phases.rs`'s `CLOSED` list holds
`Phase 7` alone and its doc comment already names "a send to an unimplemented
`Directory` method" as Phase 5's unpaid debt. So the honest statement about the
`Rung` column is that L2 cannot be re-homed to an existing row until some phase
takes that debt.

## 2. Step 1, and what it turned up

The set is `ls rust/corpus-l1/ | grep rxregexp`. **Run as they stand they
execute nothing**, which was measured over the whole set rather than inferred
from one: every file is directives only, and the loop in the measurement's
section 1a returned `rc=0 out=0 err=0` for each with an empty set of files
writing any byte. Driving them needs two things, both measured first: a `call
main` because nothing calls the routine, and a bound `self` because every file
sends its assertions to a name that is an ordinary variable inside a
`::ROUTINE`. The transform goes into a scratchpad copy; `rust/corpus-l1/` was
not edited.

The failing set at `d39dc3194` was `rxregexp_test_parse_arg_one_omitted`,
`rxregexp_test_parse_no_args`, `rxregexp_test_parse_three_args`,
`rxregexp_test_pos_no_args` and `rxregexp_test_pos_two_args`, all one defect,
all fixed. After the change the comparison printed `AGREE` for every member and
the `DIVERGE` set was empty.

**The defect was invisible to Task 8's own witness, and the reason is worth
carrying forward.** `lang/library_method_missing_argument.rex` declares its
`::METHOD ... EXTERNAL` in the program that sends to it, so the package name and
the program name are the same string and the wrong one reads as right. It took a
two-file program to see it. That is the same blindness shape as
"one send to a fresh receiver": the witness cannot distinguish the two answers
because its fixture collapses them.

## 3. What was deliberately not fixed

Three divergences the walk found, each reduced to a two-file program and each
recorded with its transcript in the measurement's section 5. None is Phase 8's
and none is in the L1 failing set.

* **`>I>`/`<I<` names the running program** where the oracle names the package a
  required routine or method came from. A silent wrong answer, so nothing in the
  tree records it. Not fixed because the correct name has the same three-way
  shape the error report already has and the oracle's spelling of the other two
  branches under this line is unmeasured.
* **The required package's own prologue `>I>`/`<I<` pair is not emitted at all.**
* **A directive-time error in a required package names the program.**
  `blame_directive_in` exists and does the right thing; some install paths call
  `blame_directive`. Shown to be wider than Phase 8 by reproducing it on
  `::class A public subclass NoSuchClassHere`, whose 98.909 is nobody's.

## 4. Readings

| command | exit | figures |
|---|---|---|
| `cargo fmt --all` | 0 | |
| `cargo clippy -j 4 --workspace --all-targets -- -D warnings` | 0 | |
| `cargo test -p rexx-api` | 0 | |
| `cargo test -p rexx-core --test unsafe_sites` | 0 | |
| `cargo test -p rexx-parse --test sourceline_oracle` | 0 | |
| `cargo test -p rexx-exec --lib` | 101 | 814 passed / 3 failed |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | 531 of 531 |

`--lib`'s failing set, printed rather than counted, every member older than this
task: `ir::drive::tests::a_call_site_resolves_once_and_answers_from_what_it_kept`,
`ir::drive::tests::a_long_constant_is_built_once_however_many_passes_read_it`,
`ir::drive::tests::the_ir_engine_steps_an_ifs_chosen_branch_from_the_chunk`.

The corpus figure moved from 528 to 531 with this task's three programs. The
four phase gates are Task 11's and were not run here.

`fmt` and `clippy` were re-run after a late rename of one parameter, and the
release binary was rebuilt before the last corpus run and the last L1 run, so
neither figure was read off a stale build.

## 5. For Task 11

* The `Rung` column cannot move L2 anywhere useful yet. Section 4 of the
  measurement says why.
* Section 5's three divergences want homes.
* Section 6 is a hazard for anyone who drives the suite: the no-argument form
  writes fixture files into `ootest/`; the single-group form does not.
