# Task 5 report: `~define` with source text

**Base commit.** `02f13726e`, branch `plan/rust-rewrite`.

**Commits.**

* `a4ee9284f483d1fbcf6ca23e22e5f166a264d1a5` -- `a4ee9284f`, "Compile a method from source text at
  ~define and ~defineMethods".
* `71327dc46` -- "Say what the two method-name spellings buy, and cite the string test". Two doc
  comments in `dispatch.rs`, no code; `git diff a4ee9284f 71327dc46` over the file has no line that
  is not a `///` line.

The status line and its evidence are at the end of this file.

---

## 1. How everything below was measured

Probes ran from `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/cd8e0d76-.../scratchpad/t5probe`
and a `clean/` subdirectory under it, both created with `mkdir -p`, with absolute paths on every
redirect. Every oracle run was wrapped as

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )
```

and every crate run as `REXX_ENGINE=<engine> timeout -s KILL 20 ./target/release/rexx-run FILE` from
`rust/`. stdout, stderr and the exit status went to three separate files and were compared with
`cmp`, never `2>&1`. The driver is `t5probe/tri.sh`, which runs all three sides and prints an
`AGREE`/`DIVERGE` verdict per engine.

`parse version` on the oracle behind `build/bin/rexx` was not re-checked this session; `rust/CLAUDE.md`
records the 2026-08-20 swap to a version-matched 5.3, and every figure below was taken today against
whatever binary that path holds.

## 2. The row

`rust/corpus/gate-tables/concepts/methna.rex`, oracle **rc 0**, four lines on stdout and nothing on
stderr:

```
id COST
operator-name The Method class
added-lowercase The Method class
matched-in-uppercase The Method class
```

Before the change, `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` both gave rc **120** and
`rexx-exec: a method built from source text is not implemented (Phase 5)`. After it, both engines are
byte-identical to the oracle on all three descriptors -- `t5probe/tri.sh
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/corpus/gate-tables/concepts/methna.rex` prints
`ir: AGREE` and `tw: AGREE`.

Under `REXX_CORPUS_GATE=1 REXX_PHASE_GATE=5a cargo test -p rexx-exec --test gate_table_c` the row's
line reads

```
  agree          loud=no  5a   methna Method Names   depth 1 parent xcremet provide.xml:456 methna.rex
```

and the run reports `gated by this run: 0 row(s) whose owning phase is closing or closed and whose
verdict is not agree`.

## 3. What was built

`Class~define` and `Class~defineMethods`, handed anything that is not already a `Method` object,
compile it. That is `MethodClass::newMethodObject`'s second arm
(`classes/MethodClass.cpp:462`-`:485`), and it reaches this crate through one new function,
`dispatch::compile_method_source`.

### 3.1 The source model is `ArrayProgramSource`, and that is load-bearing

`processExecutableSource` (`execution/BaseExecutable.cpp:169`) wraps a string source in a
**one-element array** and hands an array to `LanguageParser::createMethod`, which builds an
`ArrayProgramSource` over it. One element is one physical line, and nothing inside an element can
divide it.

That is not the same as joining the elements and letting the scanner find the boundaries, and the
difference is a silent wrong answer rather than a cosmetic one. Measured, oracle **rc 243**:

```
src = 'say 1' || '0a'x || 'say 2'
.cost~define("m", src)
```

```
     1 *-* say 1

       *-* Compiled method "DEFINE" with scope "Class".
     2 *-* .cost~define("m", src)
Error 13 running m line 1:  Invalid character in program.
Error 13.1:  Incorrect character in program "
" ('0A'X).
```

A build that joined first would have compiled that at rc 0. The same bytes inside one element of an
array source give the identical report.

So `rexx-parse` gains a third `SourceKind`, `Lines`, a `ProgramSource::from_lines` that takes the
line index rather than scanning for it, and a `parse_lines` entry point. The scanner already reads a
line at a time through `line_span`, so the `\n` separators the buffer carries between elements are
never scanned; a `\n` *inside* an element sits inside that element's own range and the scanner
rejects it exactly as the oracle does.

Two behaviours come with the kind rather than with the caller:

* **A `#!` first line is skipped**, as a file's is and unlike an `INTERPRET`'s.
  `ArrayProgramSource::setup` takes that branch whenever `interpretAdjust` is zero
  (`parser/ProgramSource.cpp:594`-`:603`), so `scanner.rs`'s `first_line` now asks
  `!= SourceKind::Interpret` where it asked `== SourceKind::Program`. Measured, oracle rc 0:
  `.cost~define("sh", '#!/bin/sh')` compiles.
* **No Ctrl-Z truncation**, which is a `BufferProgramSource` rule. Measured on the `INTERPRET` side,
  which shares the property: `'1a'x` inside the text is 13.1, and inside a literal it survives as
  data.

`SourceKind` is only ever compared, never matched exhaustively, so adding a variant compiles with no
other change forced. `first_line` in `scanner.rs` was then changed on purpose, not by the compiler:
`/bin/grep -rn "\.kind() ==\|\.kind() !=" rust/crates/rexx-parse/src/*.rs` finds 8 comparison sites,
and every one but that is an `== SourceKind::Interpret` that means the same thing under the new
variant as under the old.

### 3.2 What the compiled object is, and what it is not

`compile_method_source` decodes the source to lines, parses them, and answers a `Method` object
carrying **no scope**. `Interp::define_method_object`'s existing `newScope` then fills the defining
class in and keeps the object rather than copying it, which is the oracle's own path. Measured,
oracle rc 0 and both engines identical:

```
.cost~define("upper", 'return "U"')
say .cost~method("UPPER")~scope~id     ->  COST
say .cost~method("upper")~class~id     ->  Method
```

and the unscoped half of the pair, which an unattached `::METHOD` already witnesses at rc 0 on all
three sides: `.methods~z~scope` is `The NIL object`.

**The compiled body is validated and then discarded.** This is the task's one real judgement call and
it is argued rather than assumed:

* `~define` and `~defineMethods` install into a class's **instance** dictionary. `~new` is not built
  for a class a program declares -- measured, `.cost~new` is `rexx-exec: method "NEW" of class
  "Object" is not implemented (Phase 5)` at rc 120 -- so no instance exists and no send this phase
  can make reaches an instance-side entry.
* The one route a send *can* take to a body compiled from source text is `~subclass`'s class-method
  table, and it is reachable: measured, oracle rc 0, `.methods~put('return 42', 'M')` then
  `.object~subclass("k", .Class, .methods)` then `k~m` prints `42`.
* On that route the oracle reports a failure inside the body **against the method**, where a
  program's own clause reports its path. Measured, rc 214, with the body `return 1/0`:

  ```
       1 *-* return 1/0
       3 *-* say 'answer' k~m
  Error 42 running M line 1:  Arithmetic overflow/underflow.
  Error 42.3:  Arithmetic overflow; divisor must not be zero.
  ```

  Nothing in this crate answers `running M`. `FailureSite::Sourceless` carries a package name in
  place of a path, but it is the no-source-available shape and its text is a catalogue line, not a
  clause; a level with both a real clause and a name override has no representation, and every
  `FailureSite::Clause` construction would have to learn to supply one --
  `/bin/grep -rn "FailureSite::Clause {" rust/crates/rexx-exec/src/lib.rs
  rust/crates/rexx-exec/src/run.rs` finds five.
  `SOURCELINE`, `PARSE SOURCE` and `.context~package` inside such a body are each a further
  unmeasured surface.

So retaining the body would trade a loud refusal for a wrong answer on that route. Both halves of it
refuse loudly instead: `compile_method_source` keeps no body, and `install_enhancing_methods` declines
the install for a source-text value **and** for a `Method` object with no body row -- the second is
the round trip `.k~define("m", 'return 1')` then putting `.k~method("M")` into the table, which is
oracle rc 0 and answers.

What the parse still buys is the oracle's **timing**: a source that does not parse fails at `~define`
time and not at send time, for a body no send ever reaches.

### 3.3 The array form: built, not refused

The brief left this open. It is built, and the reason is that it is measured working, reachable in
this phase, and the same code path as the one-line case.

* Reachable: `.array~new` is a loud refusal here, but an array literal is not. Measured, both
  engines rc 0, `b = ('x', 'y')` then `b~items` prints `2`.
* Measured on the oracle, rc 0: `.cost~define("dbl", ('use arg q', 'return q * 2'))` then
  `.cost~method("DBL")~scope~id` is `COST`.
* Refusing it would have been a special case *inside* the line model rather than instead of it: a
  string is the one-element array, so the array form is the general shape and the string is the
  narrow one.

`stringArrayArgument` (`classes/StringClassUtil.cpp:417`) walks `1..=lastIndex()`, so the walk ends at
the last item and a longer array with nothing beyond it is fine. Both halves measured:

| program | oracle |
| --- | --- |
| `.array~new(3)` with only `[1]` assigned | rc 0 |
| `('return 1', , 'nop')` -- `~items` 2, last index 3 | rc 163, `93.952` |
| an array holding a class object | rc 163, `93.952` |

`Raised::method_source_not_all_strings` is the new constructor. Its position string is not the same
for every caller and that is measured, not assumed: `~define` reports `Method argument **method** is
an array ...` and `~defineMethods` and `~subclass`'s class-method table both report `method source`.
`RexxClass::defineMethod` passes `"method"` (`classes/ClassClass.cpp:849`) where
`createMethodDictionary` passes `"method source"` (`:1265`).

### 3.4 What refuses loudly, and what the oracle does instead

Every one of these is a **loud** refusal at `NOT_IMPLEMENTED_EXIT` (rc 120) with a `rexx-exec: ` line
on stderr, never a Rexx condition the oracle does not raise. Each row's oracle answer was measured.

| source shape | oracle | here |
| --- | --- | --- |
| neither a string nor an array | `.k~define("m", .environment)` is **rc 0** -- `requestArray` on a directory answers its index list, and every index is a string. `.k~define("m", .cost)` is rc 163, `93.974`. | `a method source that is neither a string nor an array` |
| does not parse | rc 221, `Error 35 running bad line 1:` then `35.901 Prefix operator "+" is not followed by an expression term.` | `reporting a method source that does not parse (bad, 35.901: Invalid expression.)` |
| carries a directive | rc 0; the directive installs into the method's own package and `.zz` is 97.1 in the caller afterwards | `a method source that carries a directive` |
| a class-side install | rc 0, and the body answers | `a class method built from source text` |
| a compiled method object put in a class-method table | rc 0, and the body answers | `a class method whose body this crate does not hold` |

Two of those deserve their reason rather than just their row.

**"Neither a string nor an array" is refused rather than raised 93.974**, even though 93.974 is what
the oracle raises for a class object, because the oracle's answer for this whole class of value is
decided by `requestArray` and `makeString` and this crate models neither. Two shapes it *can* build
answer **rc 0** on the oracle from a membership this crate does not share -- `.environment` and a
`StringTable` compile their own index lists -- so a blanket 93.974 would be a wrong answer for those.
One refusal for one unmodelled mechanism.

**A source that does not parse is refused rather than raised**, and what is missing is precisely the
*report*: `ParseError` carries `code`, `sub` and a byte and no substitution values, so 35.901 would
read `Prefix operator "&1"` where the oracle reads `"+"`. That is the same gap `Interp::run_fragment`
already records for `INTERPRET`, and it is visible today: `interpret 'this is not rexx +++'` is rc 221
on both sides but the crate prints `Prefix operator "&1"` and omits the fragment's own clause echo.
Reproducing the `~define` transcript would also need a `FailureSite` that carries a clause *and* a
name (`Error 35 running **bad** line 1`), which is the same missing piece section 3.2 describes.

### 3.5 Annotations

Every `Method` object this crate builds carries an annotation table, and
`Interp::annotations_table`'s doc says so. A compiled method has no directive and no dictionary entry
to key one by, so `Annotated` gains a `Compiled(usize)` variant counted by a new `Interp` field.

Keying by the dictionary entry instead -- `Annotated::Member(class, false, name)`, which
`Interp::method_object` uses -- was rejected: a `~define` over a name a `::ANNOTATE METHOD` had
already annotated would then answer the directive's pairs, where the oracle gives every compiled
method an empty table. Measured, oracle rc 0 and both engines identical:

```
.cost~define("upper", 'return "U"')
say .cost~method("UPPER")~annotation('zz')          ->  The NIL object
.cost~method("UPPER")~annotations~put('v', 'N')
say .cost~method("UPPER")~annotation('N')           ->  v
.cost~define("other", 'return "O"')
say .cost~method("OTHER")~annotation('N')           ->  The NIL object
```

## 4. What is pinned, and where

### 4.1 Two corpus programs

`rust/corpus/lang/method_from_source.rex` and `rust/corpus/lang/method_from_source_table.rex`, both
registered in `rust/corpus/phase-5a.txt` and in `EXPECTED_SUBSET_5A`
(`crates/rexx-exec/tests/coverage.rs`), both with a `crates/rexx-parse/tests/sourceline_oracle/`
expectation captured by the driver in that test's own module comment.

`_from_source` is the `~define` caller: the upcasing pair through `~method("TYPE")`,
`~method("type")` and `~method("Type")`, a name no symbol could hold (`%`), a number source, a `#!`
source, the array-of-lines source, an empty line inside one, the scope a taking class sets against
the `The NIL object` an untaken method carries, the per-method annotation table, `~define`'s 91.999
for sitting in an expression, and the 97.1 for a name the other class never got. It ends **untrapped**
on `('return 1', , 'nop')` so the `method` position string is on stderr. Oracle rc 163.

`_table` is the `~defineMethods` caller: a string entry and an array entry compiled through the
supplier walk, the directive entry beside them, and the scope the table's own object comes away with.
It ends untrapped on the same malformed array so the `method source` position string is on stderr.
Oracle rc 163.

Both are byte-identical to the oracle on all three descriptors on both engines.

### 4.2 In-crate tests

* `run::tests::the_method_source_shapes_this_task_leaves_refuse_loudly` -- one case per row of the
  table above, with the parse-failure row split in two so that both the 35.901 source and the 13.1
  one are asserted. Each case asserts the exit code **and** the message, and the doc comment carries
  the oracle's own answer for each. In-crate rather than corpus rows for the reason the neighbouring
  `the_refusals_this_task_leaves_where_the_oracle_answers_still_fire` gives: the oracle answers every
  one of them, so a differential row could never agree.
* `crates/rexx-parse/tests/program.rs` gains five tests for the line model: a terminator inside an
  element (13.1, both `\n` and `\r`) against two elements being two lines, a Ctrl-Z inside an element
  against a program truncating at one, the `#!` skip against `INTERPRET`'s 13.1, directives and
  labels accepted against `INTERPRET`'s 99.914 and 47.1, and an empty element list against two empty
  elements in a row keeping their places.

### 4.3 A test that stopped being true

`run::tests::the_refusals_this_task_leaves_where_the_oracle_answers_still_fire` carried a row for
`.K~define("SRC", "say 'x'")` and a doc bullet for it. Both are gone: that source now compiles, and a
row asserting a refusal that no longer happens is a test asserting the opposite of the truth.

## 5. Controls

### 5.1 The plan's named control, and the correction it needed

The plan asked for: *keeping the as-written spelling as the dictionary key makes `~method("TYPE")`
raise where the oracle answers, and the row reddens.* **Run as written, at the site it names, it is a
no-op.**

* Mutation 1, `native_define` passing the as-written spelling instead of the upcased one:
  `REXX_CORPUS_GATE=1 REXX_PHASE_GATE=5a cargo test -p rexx-exec --test gate_table_c` still printed
  `agree ... methna`, and `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` still printed
  `264 of 264 matching`.
* Mutation 2, `MethodDict::replace_method` keeping the key it is given instead of upcasing it: same
  two commands, same two results. The name arriving there is already upcased.
* Mutation 3, **both together**: `gate_table_c` printed

  ```
    diverge-both   loud=no  5a   methna Method Names   ...
  gated by this run: 1 row(s) whose owning phase is closing or closed and whose verdict is not `agree`
  ```

  and failed with `1 row(s) of gate table C owned by a closing or closed phase do not agree with the
  oracle; the first of them are ["gate-tables/concepts/methna.rex"]`. **One row, nameable**, which is
  the shape the plan asks a control to have.

Inverted live: with all three mutations reverted from the scratchpad copies (never `git checkout --`),
`REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` printed `264 of 264 matching` again and
exited 0.

The corpus tells the same story from the other side: under mutation 3, `264` became `252 of 264`, and
the twelve red programs include `class_method_own_dictionary`, `method_scope`, `array_make_string`
and `library_bootstrap_state` -- so the *upcasing* is already witnessed by programs that predate this
task, and mutation 3 is a control for the row rather than coverage for the new ones.

Both the plan (`docs/superpowers/plans/2026-08-26-phase-5a-gate-close.md`) and gate table C's own
`control` string for the `methna` row now say that both sites have to go together, with the
measurement.

### 5.2 Does anything I added *add coverage*?

Two mutations chosen to be invisible to everything except the new programs. Each was applied, the
whole corpus run, and the red set read:

| mutation | `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` |
| --- | --- |
| swap the two position strings (`method` <-> `method source`) | `262 of 264 matching`; red: `method_from_source.rex` (stderr), `method_from_source_table.rex` (stderr) |
| `Annotated::Compiled(0)` -- every compiled method shares one table | `263 of 264 matching`; red: `method_from_source.rex` (stdout) |

Nothing else in the corpus sees either. `/bin/grep -rn "method source" --include=*.rs crates/` finds
the string only in `dispatch.rs`, `error.rs` and one doc line of the new test, so no in-crate test
asserts it either: without the two new programs, the first mutation ships green. That is the
difference between "can fail" and "adds coverage", and both programs clear it -- and each clears it
separately, since the first mutation reddens one on each side of the position-string split.

### 5.3 The plan's other correction

The plan's third paragraph opened *"since the body is real Rexx and reaches the interpreter"*. That
premise is false for this row and for this task, for the reason section 3.2 measures. The sentence is
corrected in place rather than deleted, because the paragraph's *requirement* -- say what a compiled
body can and cannot do -- is still the right requirement; what was wrong was the premise it rested
on.

## 6. What I decided not to build, and why

**`Method~new`.** The brief's addendum measures `d = .method~new("q", 'return 1')` then `d~scope` as
`The NIL object` and calls the pair this task's cross-task interaction with Task 2. I probed the pair
and did not build the factory. Four reasons, in order of weight:

1. **The `.nil` half is already witnessed at rc 0 without it.** `.methods~z~scope` on an unattached
   `::METHOD` prints `The NIL object` on the oracle and on both engines today -- measured, `AGREE` on
   all three descriptors -- so `Method~new` adds no witness for "a method object no class has taken
   carries no scope". The positive half, a compiled method answering its defining class, is built and
   is in both new corpus programs.
2. **It is a class-side native, and this crate implements no class-side native at all.**
   `AddClassMethod("New", MethodClass::newRexx, A_COUNT)` (`memory/Setup.cpp:1091`) puts `NEW` in
   `.Method`'s own class dictionary, and `ObjectModel::build` wires `NATIVE_METHODS` through
   `lookup_instance_method` only. A second table and a second lookup is new plumbing, not this
   mechanism.
3. **Its row is not gated by 5a.** `method__instance.rex`'s probe opens with `o = .Method~new`, and
   `gate_table_c.rs`'s `METHOD_PHASE` is `"5c"` with its own comment saying method rows "are reported
   here and never gated by a 5a run".
4. **A partial build would be a wrong answer where a refusal is available.** `newRexx` takes an
   optional third `sourcecontext` argument with `PROGRAMSCOPE`/`Method`/`Routine`/`Package`
   semantics (`execution/BaseExecutable.cpp:249`-`:306`) and then sends `INIT` with the leftover
   arguments, through `RexxClass::completeNewObject` (`classes/ClassClass.cpp:1882`, called at
   `classes/MethodClass.cpp:517`). Building the two-argument form and refusing the third would turn a
   loud row into an answering one in a phase that does not own it.

It stays a loud refusal: `rexx-exec: method "NEW" of class "Method" is not implemented (Phase 5)`,
against the oracle's rc 168 `88.901 Missing argument; argument name is required.`

## 7. Two divergences found in passing, neither mine

Recorded because they are real and nobody owns them yet, not because this task touched them.

1. **`INTERPRET`'s parse-failure transcript.** `interpret 'this is not rexx +++'` is rc 221 on both
   sides, and the crate's stderr differs twice: it omits the oracle's echo of the *fragment's* own
   failing clause (`     1 *-* this is not rexx +++`), and it renders 35.901 as `Prefix operator
   "&1"` where the oracle renders `Prefix operator "+"`. The second is `ParseError` carrying no
   substitution values; `Interp::run_fragment`'s own comment records the first. Both are the reason
   section 3.4 refuses the `~define` parse failure rather than raising it.
2. **`condition('A')` is a loud refusal**, `CONDITION option "A" answers an Array or .NIL, which is
   not implemented`, where the oracle answers the substitution array. It is why the two new corpus
   programs pin the 93.952 position strings by ending untrapped rather than by printing
   `condition('A')` in a trap.

## 8. The five gates, at `71327dc46`

Four of them ran through `scratchpad/controller-gates.sh 71327dc46
<logdir>/gates3-71327dc46`, which gates a committed SHA in two dedicated worktrees. Both worktrees
report the same commit: `cat gates3-71327dc46/checked-out-sha` and `checked-out-sha-b` each print
`71327dc46d9dd6e234e0dc86ba0fe3ee8716339b`. **Every status below was read from its own `.rc` file,
never from a completion notice.** `cat gates3-71327dc46/elapsed.seconds` prints `3911`.

| # | command | file | exit |
| --- | --- | --- | --- |
| 1 | `cargo fmt --all --check` | `g1.rc` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | `g2.rc` | **0** |
| 3 | `cargo test --release --workspace` | `g3b.rc` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | `g4.rc` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | `g5.rc` | **0** |

Gate 3 is not one of the four the script runs, so it ran in the main worktree, from `rust/`, with
`git status --short` empty and `git rev-parse --short HEAD` printing `71327dc46`:
`cargo test --release --workspace > g3b.log 2>&1; echo $? > g3b.rc`.

**The controller's reading that G4 covers G3 holds, and it is now checked rather than assumed.** The
claim is that `REXX_CORPUS_GATE=1` only adds checks, so a test cannot run under G3 and be skipped
under G4. Every reader of the variable was read:
`/bin/grep -rn "REXX_CORPUS_GATE\|CORPUS_GATE_ENV\|GATE_ENV" crates/ --include=*.rs` finds it in
`tests/corpus.rs`, `tests/gate_tables/mod.rs` and `tests/oracle_deadline.rs`.

* `corpus.rs`'s `gate_mode()` is read at exactly two places, `:730` and the assertion below it. It
  labels the report and guards the final verdict assertion; the structural check above it is
  unconditional and its own comment says `gate_mode()` "exists to relax a *verdict* comparison"
  rather than to decide whether a run happens.
* `gate_tables/mod.rs` has the same shape: `assert_no_structural_failures` is called unconditionally
  and before any gated assertion.
* `oracle_deadline.rs` runs **only** with the variable set, so G4 has a test G3 skips rather than the
  other way round.
* `/bin/grep -rn "!gating()\|!strict\|!gate_requested" crates/rexx-exec/tests/` matches nothing:
  no test body runs only when the variable is absent.

So G3 is redundant with G4 and the wall-clock is not owed. It was run here anyway -- it had already
finished before the ruling arrived -- and its exit 0 is recorded above rather than discarded.

**Figures, each beside the command whose log it came from.** The count of `test result: ok` lines is
not stable across parallel targets, so what is counted is failure blocks:
`/bin/grep -c '^failures:$'` prints **0** for `g4.log` and **0** for `g5.log`, and
`/bin/grep -cE '^test result: FAILED'` prints **0** for each. `g3b.log` likewise has **0** failure
blocks.

The corpus differential, from `g4.log` and `g5.log`: `264 of 264 matching`. The two new programs are
two of those 264.

Gate table C's own line for the row, from `g5.log`:

```
  agree          loud=no  5a   methna Method Names                      depth 1 parent xcremet provide.xml:456 methna.rex
```

and both tables report `gated by this run: 0 row(s) whose owning phase is closing or closed and whose
verdict is not agree`.

**No sixth 5a row appears**, which is what Task 6 needs to know before the flip.
`/bin/grep -hcE "^  (agree|diverge-[a-z]+) +loud=(yes|no) +5a " g5.log` prints **171** and
`/bin/grep -hcE "^  agree +loud=(yes|no) +5a " g5.log` prints **171**: every 5a row in both tables
agrees. The same scan over the whole file finds `diverge-` rows only at `5b` and `5c`.

### Clippy from a clean target directory

`rust/CLAUDE.md` treats a same-session green clippy as provisional, because a warm target directory
can return exit 0 without re-linting. Gate 2 above took 1.51s and re-checked only `rexx-exec`, so it
was backed by a second run from a `CARGO_TARGET_DIR` that did not exist beforehand, in the main
worktree at `71327dc46`:

```
CARGO_TARGET_DIR=<scratch>/clean-target cargo clippy --workspace --all-targets -- -D warnings
```

exit **0** (`clippy-clean.rc`). The log runs from `Compiling memchr v2.8.3` through
`Compiling rexx-classes`, `Checking rexx-parse`, `Checking rexx-core` and `Checking rexx-exec`, and
the directory it built into is 382 MB, so the linter did re-examine the code rather than reuse a
result.

### The gate worktrees, and a collision worth recording

The first run I launched collided with an in-flight run of the same script for `02f13726e`: the
script had no lock at that moment, both worktrees were clean checkouts so its dirt guard passed, and
my checkout moved the tree under the other run's `cargo test`. I killed my own run, told the
controller at once, and did not touch the other. The controller added `flock` to the script and ruled
that gating Task 5's final commit covers Task 4's tree
(`git merge-base --is-ancestor 02f13726e a4ee9284f` holds).

Two further runs of mine are void and their logdirs are renamed so no status can be read out of them
by accident: `VOID-collided-gates-a4ee9284f` (the collision), `VOID-lockrefused-gates2-a4ee9284f`
(exit 3 against the new lock) and `VOID-killed-gates3-a4ee9284f` (a run at `a4ee9284f` that I killed
once `71327dc46` existed, so that the gated SHA and the reported SHA are the same one). Each kill was
a process-group kill followed by `pgrep -af "ooRexx-gates"` to check for orphaned test binaries,
which is the check the controller's own progress note calls for.

---

## Status

**DONE.**

The evidence, restated: `corpus/gate-tables/concepts/methna.rex` is byte-identical to the oracle on
stdout, stderr and exit status read separately, on `REXX_ENGINE=ir` and on `REXX_ENGINE=tree-walker`;
the upcasing pair is pinned by `corpus/lang/method_from_source.rex`, which is registered in
`phase-5a.txt` and in `EXPECTED_SUBSET_5A` and has a `sourceline_oracle` expectation; the plan's
named control was run live in three variants, the third of which reddens exactly the one gate row and
was reverted to green again; two further controls show the new corpus programs catch mutations the
existing suite does not; and all five gates exit 0 at `71327dc46`, each status read from its own file.

**Concerns, all recorded above rather than left for a reviewer to find:**

* The compiled body is deliberately not retained, so `~subclass`'s class-method table refuses loudly
  where the oracle answers. Section 3.2 has the measurement and the argument.
* `Method~new` is not built. Section 6 has the four reasons.
* Two divergences found in passing and owned by nobody: `INTERPRET`'s parse-failure transcript and
  `condition('A')`. Section 7.
