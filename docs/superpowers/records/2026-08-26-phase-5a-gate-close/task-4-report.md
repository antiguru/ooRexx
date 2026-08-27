# Task 4 report: `.RexxInfo`

**Status.** DONE (the status line is restated with its evidence at the end of this file).

**Base commit.** `edc9e69c3`, branch `plan/rust-rewrite`.
**Commit.** `02f13726e38a7c35108b8f235f24476481d49340` -- `02f13726e`, "Build the RexxInfo class and
the .RexxInfo instance entry".

---

## 1. What the addendum said, re-measured

The controller's addendum told me to treat the brief's "what is already there" paragraph as a claim
to re-measure. I re-ran every figure in it. All of them reproduce.

Probes ran from a fresh empty directory, `/tmp/.../scratchpad/t4probe`, created with `mkdir -p`, with
absolute paths on every redirect. Every oracle run was wrapped as
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
/home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`, and every crate run as
`REXX_ENGINE=<engine> timeout -s KILL 20 rust/target/release/rexx-run FILE`, with stdout, stderr and
exit status read as three separate files and compared with `cmp`/`diff`, never `2>&1`.

### The row itself

`( ulimit -v 1048576; ... rexx t4probe/row.rex )` -- a copy of
`rust/corpus/gate-tables/classes/rexxinfo.rex` -- prints

```
entry a RexxInfo
class-of-entry RexxInfo
```

on stdout, raises `Error 97.1: Object "a RexxInfo" does not understand message "ID".` on stderr, and
exits **159**. Confirmed.

### The Build paragraph's last sentence is false, confirmed independently

`.RexxInfo~hasMethod(name)` over the addendum's own candidate list, oracle rc 0: every name answers
`1` except `ID` and `FILESEPARATOR`, which answer `0`. And they are live, not merely present.
Sweeping every method name in `Setup.cpp`'s `RexxInfo` block one program at a time,
`say .RexxInfo~<name>`, the oracle answers at **rc 0** for all of them but `COPY`:

| method | oracle | method | oracle |
| --- | --- | --- | --- |
| `ARCHITECTURE` | `64` | `MAJORVERSION` | `5` |
| `CASESENSITIVEFILES` | `1` | `MAXARRAYSIZE` | `100000000000000000` |
| `COPY` | **rc 163**, `93.970` | `MAXEXPONENT` | `999999999` |
| `DATE` | `30 Jul 2026` | `MAXPATHLENGTH` | `4096` |
| `DEBUG` | `0` | `MINEXPONENT` | `-999999999` |
| `DIGITS` | `9` | `MODIFICATION` | `0` |
| `DIRECTORYSEPARATOR` | `/` | `NAME` | `REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026` |
| `ENDOFLINE` | (a bare newline) | `PACKAGE` | `The REXX Package` |
| `EXECUTABLE` | the `bin/rexx` path | `PATHSEPARATOR` | `:` |
| `FORM` | `SCIENTIFIC` | `PLATFORM` | `LINUX` |
| `FUZZ` | `0` | `RELEASE` | `3` |
| `INTERNALDIGITS` | `18` | `REVISION` | `0` |
| `INTERNALMAXNUMBER` | `999999999999999999` | `VERSION` | `5.3.0` |
| `INTERNALMINNUMBER` | `-999999999999999999` | | |
| `LANGUAGELEVEL` | `6.06` | | |
| `LIBRARYPATH` | the `build/lib` path | | |

So the brief's "refuses everything else with the oracle's own 97.1" would have made every one of
those a **silent wrong answer at rc 0** about what the language understands. The addendum's ruling
stands and is what I built to: **the unbuilt surface refuses loudly at rc 120**, `ID` keeps its
genuine 97.1.

### Plan correction

Corrected in `docs/superpowers/plans/2026-08-26-phase-5a-gate-close.md`, not here:

* The Build paragraph's false last sentence is **deleted** and replaced with the loud-refusal ruling
  and the six measurements that justify it.
* The sentence citing `native_classes.rs`'s `DEFERRALS` as the record of the `addToSystem` split had
  its `DEFERRALS` clause **deleted**, because this task retires that entry and the citation would
  otherwise point at nothing. The `Setup.cpp:1737` citation beside it is untouched and still true.

---

## 2. What I built

### The split the oracle actually has

`Setup.cpp:390`-`:398` has two closing macros, and which one closes a block is the whole of the
registration difference:

* `EndClassDefinition(name)` -> `completeSystemClass(#name, currentClass)` -> the class goes into
  `.environment` under its own name.
* `EndSpecialClassDefinition(name)` -> `addToSystem(#name, currentClass)` -> the class goes into the
  kernel directory, which no `.NAME` reaches.

`RexxInfo`'s block is closed by the second (`Setup.cpp:1285`), and `Setup.cpp:1736`-`:1737` builds
one instance of it and `addToEnvironment`s that instead. Measured consequences on the oracle:
`.RexxInfo` renders `a RexxInfo`, and `::CLASS K SUBCLASS RexxInfo` is
`99.949 "REXXINFO" is not a valid class` at rc 157 -- the class object is genuinely unreachable by
name.

### The change, file by file

* **`rust/crates/rexx-classes/build.rs`** -- each derived `ClassDefinition` now carries
  `system_only`, read off which of the two macros closed the block. Derived from `Setup.cpp`, not
  listed anywhere in `src/`, so a block that changes its closing macro changes the registry with it.
* **`rust/crates/rexx-classes/src/registry.rs`** -- `define_system_class` and `system_lookup`, over a
  second name table (`by_system_name`). `lookup` and `registered` are untouched, and `registered` is
  what `.environment` is populated from, so a system class never gains an environment name.
* **`rust/crates/rexx-classes/src/native_classes.rs`** -- `build` picks the constructor off
  `def.system_only`; the `RexxInfo` deferral is **retired** (not narrowed -- the class is now built,
  wired and `REXX_DEFINED` exactly like every other checklist entry). The module doc's deferral
  bullet is rewritten: `RexxInteger`/`NumberString` stay deferred for the class-identity masquerade,
  which is the reason that still stands, and the `.NAME`-reachability clause is deleted from both
  their `reason` strings because this commit makes it not a reason to defer.
* **`rust/crates/rexx-exec/src/dispatch.rs`** -- `ObjectModel::rexx_info`, `Primitive::RexxInfo`, and
  its arm in each of the four `Primitive` matches the compiler named.
* **`rust/crates/rexx-exec/src/environment.rs`** -- the instance, allocated **after** every other
  allocation `build_environment` makes and put straight into the environment's entry table, since the
  environment is its only root and `alloc_with` collects before it allocates.

### Why the deferral is retired rather than narrowed

The dispatch expected narrowing, on the reading that "a surface that still refuses loudly is still
deferred". I retired it instead, and the reason is what `DEFERRALS` means everywhere else in that
file: it is `native_classes.rs`'s own record of the checklist entries **this module does not build**,
and `native_classes.rs` now builds `RexxInfo` completely -- the class object, `Setup.cpp`'s whole
instance dictionary, the `.Object` superclass edge, the `.Class` metaclass, the class-behaviour
merge and the `REXX_DEFINED` flag. Nothing about it is missing at this layer.

What is missing is the *implementation* of those methods, which lives in `rexx-exec`'s
`NATIVE_METHODS` -- a different table in a different crate -- and which is missing for **every** class
in this registry: `Class~defaultName`, `Class~queryMixinClass` and `Array~new` are all loud rc 120
today and none of their classes is deferred. Keeping `RexxInfo` in `DEFERRALS` on that ground would
give the table a second, incompatible meaning, and it would break the invariant
`every_setup_class_is_native_or_deferred_with_a_reason` rests on: deferred implies not registered in
either directory.

The loud-refusal state is recorded where it is actionable instead -- `Primitive::RexxInfo`'s own doc
in `dispatch.rs` says that a name this class's dictionary holds with no `NativeMethod` behind it is
this crate's gap and refuses loudly, and quotes the oracle's answers for two of them.

### What falls out, and why it is right rather than lucky

Nothing implements `~digits`, `~version` or the rest. The refusal shape is a consequence of the
dispatch this crate already had:

* `~id` and `~new` are not in `RexxInfo`'s instance dictionary and not in `.Object`'s, so
  `Interp::lookup` misses, `.Object` answers no `UNKNOWN`, and the send is the oracle's own 97.1.
* `~digits` **is** in the dictionary and has no `NATIVE_METHODS` row, so `Interp::invocable` falls
  through to `Loud::native_method` -- `rexx-exec: method "DIGITS" of class "RexxInfo" is not
  implemented (Phase 5)` at rc 120.

Measured over every name in `Setup.cpp`'s `RexxInfo` block on both engines -- 58 sends -- every one
is rc 120 with empty stdout and the expected `method "<NAME>" of class "RexxInfo"` refusal on stderr:
**0 mismatches**. The sweep is section 3.

---

## 3. Differential measurements

### The row, three descriptors, both engines

`rust/corpus/gate-tables/classes/rexxinfo.rex`, copied to the probe directory and run on the oracle
and on each engine, with stdout, stderr and status in separate files and `cmp` over each pair:

```
ir out IDENTICAL
ir err IDENTICAL
tree-walker out IDENTICAL
tree-walker err IDENTICAL
```

Exit status **159** on the oracle and on both engines. Stdout is `entry a RexxInfo\nclass-of-entry
RexxInfo\n`; stderr is the 97.1 traceback quoting `"a RexxInfo"` and `"ID"`.

**The gate table's own verdict.** The `edc9e69c3` column is from
`cargo test --release -p rexx-exec --test gate_table_c` run at the base commit before any edit; the
`now` column is read out of gate **G4**'s own log
(`REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast`), so it is the shipped tree's
reading and not a separate run's:

| line | at `edc9e69c3` | now |
| --- | --- | --- |
| `RexxInfo ... instance ... rexxinfo.rex` | `diverge-both loud=yes 5a`, `crate rc=120 err="rexx-exec: environment symbol \".REXXINFO\" ..."` | **`agree loud=no 5a`** |
| `RexxInfo ... rexxinfo__instance.rex` (5c) | `loud=yes`, `crate rc=120` against `oracle rc=159` | `loud=no`, **crate `rc=159`**, the same stderr prefix as the oracle |
| the loud-waiting tally | `environment symbol ".REXXINFO": 29` | the entry is gone from that list |
| 5a summary line | not read at base | `5a: 135 rows, 1 not yet agree` |
| `gated by this run` | `0 row(s)` | `0 row(s)` |

The one 5a row still not `agree` is `methna`, whose refusal is `a method built from source text` --
Task 5's. I did not read the base run's own 5a summary line, so that cell says so rather than
carrying a number I would be reconstructing.

### The whole surface, past the row

A sweep of `say .RexxInfo~<name>` for every method name in `Setup.cpp`'s `RexxInfo` block, one
program each, oracle and both engines side by side (the table in section 1 has the oracle column). A
shell loop asserted, per send, that the exit status is 120, that stdout is empty, and that stderr is
exactly `rexx-exec: method "<NAME>" of class "RexxInfo" is not implemented (Phase 5)`:

```
sends checked: 58   mismatches: 0
```

**Not one answers rc 0 with a value.** That is the addendum's ruling holding across the whole unbuilt
surface, and it is the measurement that says no silent wrong answer was introduced.

### A wider probe, trapped, both engines

A `signal on syntax` ladder of 37 rows over the entry, the class object and the messages `.Object`
answers -- rendering, `~string`, `~objectName`, `~class` and its whole wiring, `~package`,
`~baseClass`, `~isSubclassOf`, `~isA` both ways, `~isNil`, `hasMethod` positives and negatives,
`~request` in three shapes, `.environment['REXXINFO']`, `~at`, `~method`, and the 97.1s for `~id`,
`~new` and `~method("ID")`.

Four rows had to come out, because a loud refusal ends the program rather than being trapped, so each
one hid everything after it: `Class~defaultName`, `Class~queryMixinClass`, `RexxInfo~copy` and
`datatype(.RexxInfo~identityHash, 'W')`. **With those four removed, every remaining row matched the
oracle's stdout byte for byte on both engines at rc 0**, and each of the four was then run and
accounted for on its own:

* `Class~defaultName` and `Class~queryMixinClass` are pre-existing `.Class` gaps, reachable from any
  class object and nothing to do with this task.
* `RexxInfo~copy` is this task's own unbuilt surface -- oracle rc 163, `93.970` (see section 6.2).
* `datatype(.RexxInfo~identityHash, 'W')` is `1` here and `0` on the oracle. **Pre-existing and
  general**, not introduced here: measured, `datatype(.environment~identityHash, 'W')` is the same
  `1` against `0`, and `native_identity_hash`'s own doc records this as deviation 4's licence.

A separate `hasMethod` sweep over every name in the block plus negatives (`ID`, `FILESEPARATOR`,
`ZORK`, and class-side names like `SUBCLASS`, `NEW`, `DEFINE`), asked on **both** the instance and
the class object: **stdout identical to the oracle on both engines**, rc 0 everywhere.

### The corpus

`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` prints **262 of 262 matching**
and exits **0**. The new program is `corpus/lang/rexxinfo_entry.rex`, registered in
`corpus/phase-5a.txt` and in `EXPECTED_SUBSET_5A`, with its
`crates/rexx-parse/tests/sourceline_oracle/rexxinfo_entry.txt` regenerated by that test's own
`.Package~new` driver (`count 78`); `cargo test -p rexx-parse --test sourceline_oracle` passes.

---

## 4. Controls

**All three controls were run at `02f13726e` itself**, after the five gates and after the commit, so
the tree they mutated is exactly the tree this task ships. After each one the tree was restored and
`git status --short` printed nothing.

### Control A -- the brief's named one, and its inversion

**Mutation.** In `build_environment`, `(b"REXXINFO", rexx_info)` -> `(b"REXXINFO",
rexx_info_class)`: register the class object under `.RexxInfo` instead of the instance.

**Predicted by the brief:** `entry` reads `The RexxInfo class` and the row reddens. Measured,
`REXX_ENGINE=<engine> target/release/rexx-run <probe>/row.rex`, identical on both engines:

```
rc=0
entry The RexxInfo class
class-of-entry Class
id RexxInfo
class The Class class
superclass The Object class
superclasses The Object class
metaclass The Class class
isa-class 1
```

against the oracle's two lines and rc 159. **This is the plan's own worst defect class made
concrete**: rc 0, eight plausible lines, and a wrong answer about what `.RexxInfo` is -- and note
that the mutated build answers *more* of the row, not less.

`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` under the mutation:
**exit 101**, `261 of 262 matching`,
`[method "NEW" of class "RexxInfo"] lang/rexxinfo_entry.rex: stdout, stderr, exit code differ`.

**Inverted live.** Restored by copying the file back from the scratchpad, rebuilt, and the same
command re-run: **exit 0**, `262 of 262 matching`. So the red was the mutation and not a broken
harness. `git status --short` printed nothing after the restore.

### Control B -- mine: does the new corpus program *add* coverage?

"Can fail" is not "adds coverage", so the mutation was applied to the suite **without** the new
program first.

**Mutation.** `NativeObject::new(rexx_info_class, b"a RexxInfo")` -> `b"an RexxInfo"`. One byte pair
in the instance's rendering, which is the thing the row and the corpus program read. Whether
anything else in the workspace asserts it is the question the without-run below answers.

| run | command | result |
| --- | --- | --- |
| **without** `lang/rexxinfo_entry.rex` (removed from `phase-5a.txt` *and* `EXPECTED_SUBSET_5A`, so the subset check is not itself the catch) | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **exit 0**, `261 of 261 matching`, **no `failures:` block anywhere in the workspace** |
| **with** it registered, same mutation | the same command | **exit 101**, `261 of 262 matching`, one failing binary -- `test result: FAILED. 16 passed; 1 failed` for `corpus_differential` -- naming `lang/rexxinfo_entry.rex: stdout differ`, `rust: stdout="1 entry an RexxInfo\n..."` |

**The new program is the only thing in the whole gated release workspace that catches it**, which is
the coverage claim made as a measurement rather than asserted.

**One process note against myself.** Putting the corpus program's registration back for the second
arm, I used `git checkout -- rust/corpus/phase-5a.txt rust/crates/rexx-exec/tests/coverage.rs`, which
this project's rules tell me not to do. Nothing was lost -- both files had been committed at
`02f13726e` minutes earlier and the only difference was the control's own deletion, so the checkout
restored exactly the committed bytes -- but the correct move was the scratchpad copy, which is what
every other restore in this task used.

### Control C -- the discriminator behind the new unit tests

`system_only` is what the three-way registration test reads, so a `system_only` that answered `false`
everywhere would leave every assertion satisfied by the arm it took before the split existed.
`system_only_separates_the_two_closing_macros` asserts it **both ways** off the derived table:
`RexxInfo`'s block is `true` (`EndSpecialClassDefinition`, `Setup.cpp:1285`) and `StackFrame`'s is
`false` (`EndClassDefinition`). `a_system_class_is_absent_from_the_environment_registration` does the
same for the registry: `RexxInfo` answers `system_lookup` and neither `lookup` nor `registered`,
`StackFrame` answers `lookup` and `registered` and not `system_lookup`.

---

## 5. Gates

All five run from `rust/`, over the working-tree bytes that became `02f13726e`. G1-G4 finished before
the commit; G5 was launched before it and still running when `git commit` ran, which changes no file
in the working tree, and the file hashes were recorded before the chain started and checked against
the tree afterwards. Each exit status was written unpiped into its own file with
`echo "EXIT=$?" > <file>` and read back from that file. No wrapper ends in an `echo`.

| | command | exit | `failures:` blocks | binaries reporting FAILED |
| --- | --- | --- | --- | --- |
| G1 | `cargo fmt --all --check` | **0** | 0 | 0 |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** | 0 | 0 |
| G3 | `cargo test --release --workspace --no-fail-fast` | **0** | 0 | 0 |
| G4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** | 0 | 0 |
| G5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** | 0 | 0 |

The failure-block and FAILED-binary columns are `grep -cE '^failures:'` and
`grep -cE '^test result: FAILED'` over each gate's own log, which is the stable count -- the
`test result: ok` lines are not, because parallel targets interleave on one pipe.

`262 of 262 matching` appears in G4's log (line 1305) and G5's (line 1306), both from the gated
`corpus_differential`. G3's log carries the same figure under `REPORT MODE, NOT THE GATE`.

**G2 re-examined the changed code rather than reusing a cached green.** Every file this task touched
was `touch`ed before the run, and G2's own log opens
`Compiling rexx-classes ... Checking rexx-exec ...`.

**Three things that went wrong on the way to these readings, and how each was handled.**

1. **A run against a tree I then edited.** A first five-gate chain was killed on purpose after I
   found two imprecise `Setup.cpp` line citations in comments and fixed them: the gate readings
   would have described a tree that no longer existed. The table above is from the chain started
   after the last edit.
2. **A wall-clock flake, retried.** An earlier G3 failed on
   `builtin::datetime::tests::time_e_does_not_reset_the_anchor_time_r_does` with `e2 = 0.004306,
   e3 = 0.005776` -- its assertion is `e3 > e2 * 1.5` over two real burns, and the machine was
   carrying a fifteen-minute load average of 5.84 from other work at the time. Re-run standalone ten
   times: **ten passes**. The G3 in the table above, a full workspace run, also passes it. Nothing in
   this task touches `TIME`.
3. **G5 needed three attempts, for environmental reasons, and the earlier two produced no reading
   rather than a bad one.** The first was killed with the background task that carried it; the
   second hit `timeout 3000` and exited **124** with the log ending mid-`ir_dual` -- neither is a
   test failure and neither is quoted as one. The third was launched under `setsid` with
   `timeout 7200` so the harness's task lifetime could not reach it, and is the **exit 0** above.

---

## 6. Findings for the consolidated review

These are findings, not work I did. None of them is in this task's scope.

### 6.1 A handful of `RexxInfo`'s methods read off constants this crate already holds

The addendum asked me to say so with the measurement if I found it. I did.

`crates/rexx-exec/src/parse_template.rs:97` holds
`const VERSION: &[u8] = b"REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026"`, and measured, `parse
version v` is byte-identical on the oracle and both engines at rc 0. `.RexxInfo~name` **is that exact
string** on the oracle. Reading off the same constant, or off `NUMERIC` state this crate already
answers correctly:

| method | oracle | where the value already is here |
| --- | --- | --- |
| `NAME` | `REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026` | `parse_template.rs`'s `VERSION`, verbatim |
| `LANGUAGELEVEL` | `6.06` | a field of the same constant |
| `DATE` | `30 Jul 2026` | a field of the same constant |
| `VERSION` / `MAJORVERSION` / `RELEASE` / `MODIFICATION` | `5.3.0` / `5` / `3` / `0` | fields of the same constant |
| `DIGITS` / `FORM` / `FUZZ` | `9` / `SCIENTIFIC` / `0` | measured, `digits() form() fuzz()` already agrees byte for byte |
| `INTERNALDIGITS` | `18` | the same bound `rexx-num`'s `settings.rs:130` tests (`if digits >= 18`), and the width of the `INTERNALMAXNUMBER`/`INTERNALMINNUMBER` the oracle answers |

That is a real chunk of the surface available cheaply. **I did not build any of it** -- the plan's
"and nothing else" stands, and every one of these is still a loud rc 120 refusal.

### 6.2 `RexxInfo~copy` is a refusal on the oracle, not an answer

Measured, oracle rc 163: `.RexxInfo~copy` is `93.970 COPY method is not supported for object a
RexxInfo`. It is in `RexxInfo`'s own dictionary (overriding `.Object`'s), and it is the one name in
the block the oracle does not answer at rc 0. Here it is a loud rc 120, which is honest but is a
divergence a later task could close for the cost of one refusal.

### 6.3 A `::CLASS ... SUBCLASS <non-class environment entry>` is 98.909 here and 99.949 there

**Pre-existing, general, and untouched by this commit** -- reported because I ran into it and it is
not written down anywhere I could find.

`Interp::directive_class` steps over a directory entry that is not a class object and falls through
to the native class table, where the miss becomes `98.909 Class "X" not found` at run time, rc 158.
The oracle's `ClassDirective` finds the non-class entry and raises `99.949 "X" is not a valid class`
at **translate** time, rc 157. Measured, oracle against `REXX_ENGINE=ir`, over three entries that
were in this crate's `.environment` **before** this commit:

| program | oracle | crate |
| --- | --- | --- |
| `::class K subclass NIL` | rc 157, `99.949 "NIL" is not a valid class.` | rc 158, `98.909 Class "NIL" not found.` |
| `::class K subclass TRUE` | rc 157, `99.949` | rc 158, `98.909` |
| `::class K subclass ENVIRONMENT` | rc 157, `99.949` | rc 158, `98.909` |
| `::class K subclass Zork` (no entry at all) | rc 158, `98.909 Class "ZORK" not found.` | rc 158, `98.909` -- **agrees** |

The discriminator is whether the name resolves to a **non-class** entry, not whether it resolves at
all: `Zork` and `Integer` resolve to nothing and both sides answer 98.909 there. `.RexxInfo` joins the
non-class set with this commit and answers exactly what it answered before it (98.909, rc 158,
measured), so nothing here changed -- the set it joins was already divergent.

### 6.4 `identityHash` on a `Body::Native` receiver

`datatype(.environment~identityHash, 'W')` is `1` here and `0` on the oracle, and the same holds for
the new `.RexxInfo`. Pre-existing, licensed by deviation 4 at `native_identity_hash`'s own doc, and
recorded here only because a reviewer probing past the row will hit it.

---

## 7. Status, and what to re-run

**DONE.**

The row: `rust/corpus/gate-tables/classes/rexxinfo.rex` is byte-identical to the oracle on stdout,
stderr and exit status read separately, on `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, exit 159
on all three. Re-verified against the shipped binary after every control was restored.

The gates: all five exit 0 at `02f13726e`, zero `failures:` blocks in every log, `262 of 262
matching` in G4's and G5's.

The controls: the brief's named one reddens the row and the corpus gate (exit 101, `261 of 262`) and
goes green again inverted (exit 0, `262 of 262`); mine shows the new corpus program is the only thing
in the gated release workspace that catches a one-byte change to the instance's rendering (exit 0 and
`261 of 261` without it, exit 101 and `261 of 262` with it).

`DEFERRALS`' `RexxInfo` entry is retired, with the reason in section 2.

**Concerns, all of them small and none blocking:**

* `.RexxInfo~copy` refuses loudly at rc 120 where the oracle raises `93.970` (section 6.2). Licensed
  by the addendum's ruling -- the surface is unbuilt and refuses honestly -- but it is the one name
  in the block where the oracle's own answer is itself a refusal, so it is the cheapest thing left to
  close.
* `::CLASS K SUBCLASS <non-class environment entry>` is 98.909 here against 99.949 on the oracle
  (section 6.3). Pre-existing, general, and measurably unchanged by this commit -- `.RexxInfo`
  answered 98.909 before it and answers 98.909 after -- but it is a real defect nobody has written
  down.
* I used `git checkout --` once during Control B where a scratchpad copy was the rule (section 4).
  Nothing was lost; recorded rather than omitted.
