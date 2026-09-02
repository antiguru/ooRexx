# Task 3 fix round -- report

BASE `e30055b37`, branch `plan/rust-rewrite`. Landed as `903c20283`, `08fba0f61` and `fc8532c9e`.

Status: DONE. Every finding is answered, all five gates are green at `fc8532c9e`, the phase gate
exits 101 by design with `usesem` still `agree`, and the sitting is taken.

## Status

- [x] CRIT-1 -- `Class~enhanced`'s methods are a level of their own; both engines now agree
- [x] IMP-1 -- `setmethod_hidden.rex` makes the 97.1 send its comment names, and reddens under HIDESEND
- [x] IMP-2 -- measured, controlled, and **licensed** rather than reclaimed, with the figures in the code
- [x] IMP-3 -- `setmethod_precedence.rex` carries a scope override, and reddens under SCOPEOVR
- [x] Minors 1-6
- [x] Gates: five green, phase gate 101 by design
- [x] Sitting, plus two controls for a cycles move it exposed

## CRIT-1: three levels, not two

### The C++, read here rather than taken from the review

`RexxClass::enhanced` (`classes/ClassClass.cpp:1440`-`:1479`) creates a dummy subclass, builds a
method dictionary from the enhancing table with a `.nil` scope, merges it into that subclass's
`instanceMethodDictionary`, adds it to the subclass's instance behaviour, then **clears that
behaviour's method dictionary and rebuilds it** (`:1462`-`:1463`,
`setMethodDictionary(OREF_NULL)` then `createInstanceBehaviour`). The rebuild is what matters: the
`instanceMethods` tracking table `addInstanceMethods` filled is thrown away with the dictionary it
was a field of, so the enhancing methods survive as ordinary entries of the dummy subclass's
behaviour with nothing tracking them. `NEW` is then sent to the dummy subclass and the resulting
object's owning class is set back to the receiver (`:1470`, `:1474`).

`RexxObject::setMethod` reaches `defineInstanceMethod` (`classes/ObjectClass.cpp:2297`), which copies
the behaviour and calls `MethodDictionary::addInstanceMethod`: that creates the `instanceMethods`
table if absent, removes a previous *setMethod* entry of the same name from the main dictionary, and
`addFront`s the new one. `unsetMethod` reaches `deleteInstanceMethod` (`:2328`) and
`MethodDictionary::removeInstanceMethod` (`behaviour/MethodDictionary.cpp:359`-`:371`), which removes
from the main dictionary **only when `instanceMethods` held the name**.

So the oracle's single dictionary plus its tracking table is observationally two levels on the
object -- what `setMethod` wrote, and what `enhanced` put behind it -- above the class. The review's
citations are correct.

### Reproduced at BASE `e30055b37`, my own probe

`crit1.rex`, fresh directory, three descriptors separate, both engines:

```
oracle       rc 0  1 enhanced-mm / 1h 1 / 2 one-off-mm / 2h 1 / 3 enhanced-mm / 3h 1 /
                   4 enhanced-mm / 5 class-mm
ir, tw       rc 0  1 enhanced-mm / 1h 1 / 2 one-off-mm / 2h 1 / 3 class-mm    / 3h 1 /
                   4 class-mm    / 5 class-mm
```

`crit1b.rex`, the four further shapes the design has to answer -- oracle rc 0 throughout, and the
crate at BASE dies at rc 159 on the second line:

```
oracle   a enh-e1 / ah 1 / b enh-e1 / bh 1 / c one-2 / d enh-e1 / eh 0 / f enh-e1 / g obj-1 / h enh-e1
crate    a enh-e1 / ah 1, then 97.1 on `e~e1` at rc 159
```

`b`: `unsetMethod` for an enhancing name nothing set leaves it. `c`/`d`: two `setMethod`s and one
`unsetMethod` reveal the enhancing method, so at most one `setMethod` entry stands per name.
`eh`/`f`: the no-method form hides an enhancing method and `unsetMethod` reveals it. `g`/`h`: an
`OBJECT`-scope one-off over an enhancing name is taken back the same way.

`pool.rex`, the control that says the storage's *scope* was already right and only its level was
wrong: an enhancing method's `EXPOSE`d name and a `FLOAT` one-off's are the same pool, and neither is
the class's. Oracle rc 0, and the crate **agreed at BASE**:

```
ew ok / er enh-reads[enh-wrote] / float-reads float[enh-wrote] / object-reads object[V] /
class-reads class[V] / er2 enh-reads[enh-wrote]
```

### What was built

`ObjectMethods` (`rexx-core/src/body.rs`) gains a second level. `set` is `setMethod`'s, `enhanced` is
`Class~enhanced`'s; `get` reads `set` first and `enhanced` behind it, `remove` retains over `set`
alone. `Interp::write_object_method` takes an `ObjectMethodWrite` naming which of the three
operations it is, and `install_enhancing_object_methods` passes `Enhance`. The enhancing entries keep the
`ObjRef::NIL` scope they had, which is what `pool.rex` says is right.

**Rejected: an actual dummy subclass.** The oracle's own mechanism, and it would give the middle
level for free through the class hierarchy. It moves the enhancing methods' scope to that subclass,
where the oracle's is `.nil` -- `pool.rex` above is the measurement that kills it, and this crate's
class dictionaries have no way to carry a `.nil`-scoped entry.

**Rejected: one level with a per-entry origin tag.** Closer to the C++'s literal shape (one chain,
one tracking table) and the same behaviour, but every reader would then have to filter, and `remove`
would have to know which origin it may take. Two vectors say the same thing in the type.

The false half of the code comment is fixed with it: `native_enhanced`'s doc no longer says the
`.nil` scope is "what this crate stores them as", and says instead that the scope is D67's pool
selection while the level is the dummy subclass's behaviour.

### After the change

`crit1.rex`, `crit1b.rex` and `pool.rex` all agree on both engines, three descriptors byte for byte.

## IMP-2: the unreclaimed synthetic program -- licensed, with the controls re-measured

Re-measured at BASE with `/usr/bin/time -f '%M'`, `REXX_ENGINE=ir`, each program in its own fresh
directory, the oracle wrapped in `ulimit -v 1048576`:

| program | crate maxrss | oracle maxrss |
|---|---|---|
| `say 'done'` | 16,556 KB | 10,864 KB |
| 20,000 `setMethod`, 20,000 distinct names | 384,804 KB | 129,312 KB |
| 20,000 `setMethod`, one name, each replacing the last | 382,788 KB | 20,636 KB |
| 20,000 `setMethod`, one name, a nine-clause source | 434,372 KB | 20,840 KB |
| 20,000 of the **no-method form**, one name | **17,192 KB** | 20,416 KB |
| 20,000 of the no-method form, 20,000 distinct names | **18,368 KB** | 24,888 KB |
| 20,000 `setMethod`, one name, `call gc 'force'` every thousandth | **377,044 KB** | -- |

The review's first three rows reproduce. **Its fourth row does not**, and its replacement is the
finding's real control. The review reports "20,000 re-attachments of one pre-built `Method` object
stay at 18,444 KB", and no route to a pre-built `Method` object runs on this crate at all: `m =
.k~method('SPARE')` then `self~setMethod('MM', m)` is rc 120 `a one-off method whose body this crate
does not hold`, and `.methods` is a plain `String` outside directive processing on the oracle too
(`97.1 ... Object ".METHODS" does not understand message "AT"`, both sides agreeing). 18,444 KB is
within noise of the 18,368 KB a run of 20,000 dictionary writes that compile nothing costs, so what
that row measured was the dictionary, which is what it concluded -- the conclusion holds and the
route named for it does not.

**The two controls that pin the cause.** The no-method form does the same 20,000 dictionary writes
under the same name and never enters `compile_method_source`: 17,192 KB, flat against an empty
program. And a forced collection every thousandth iteration moves 382,788 KB to 377,044 KB, so what
is held is not collectable arena garbage; it is `Interp::programs`, which nothing removes from.

**Licensed rather than reclaimed, and the license is in the code** at `Interp::record_compiled_body`
with the figures above beside it. **What a long-running program pays:** about 18 KB per compiled
method source, unbounded, whether or not the definition it holds has been superseded -- so a program
that compiles method sources in a loop is eventually killed where the oracle is flat, and one that
never passes a source string to `setMethod`, `~define` or `~defineMethods` pays nothing.

**Why not reclaimed.** A `ProgramId` is an index into `Interp::programs` and a `BodyKey`'s first
half, so freeing a slot means it must stay addressable, and the program must not be freed while an
activation is running it -- a one-off that replaces itself is exactly that case. The holders of one
compiled body are `method_bodies`, `table_method_bodies`, the plan cache and the activation stack,
which is a refcount or a collector sweep over programs, not a line in `ObjectMethods::set`. That is
a piece of work of its own and not a fix round's; the measurement is recorded so the decision to do
it can be taken on figures.

I also checked the one hazard the growth suggested and found it absent: `table_method_bodies` is
keyed by the `Method` object's `ObjRef` and keeps rows for objects long dead, but `Heap::resolve`
compares a generation, so a recycled slot can never answer for a stale key.

## IMP-1 and IMP-3: the two rows that could not fail

`setmethod_hidden.rex` said "the send is 97.1 even though the class defines the method" and never made
that send. It now ends with one, after re-hiding the name, so the row's stderr and exit status carry
it: oracle rc 159, `14 *-* say 'send' o~mm` then `Error 97 ... Error 97.1: Object "a K" does not
understand message "MM".`, both engines byte-identical.

`setmethod_precedence.rex` gains `say 'override' o~mm:.k` after the one-off is attached: a scope
override starts inside the class hierarchy and answers `class-mm` with the one-off in place. I dropped
a second spelling (`self~mm:super` from a subclass method) after measuring it: it agrees, but it goes
through the same `start_scope` guard, so it reddens under the same arm and adds nothing.

## Controls, recorded as run

Two `git archive | tar -x` extracts, each with its own `CARGO_TARGET_DIR`, restored from a pristine
copy of `rust/crates` between arms; nothing ran in the live worktree.

* `base/` at `903c20283` -- this commit. Baseline **`299 of 299 matching`, exit 0**.
* `parent/` at `e30055b37` -- BASE, the tree before this round. Baseline **`298 of 298 matching`,
  exit 0**.

```
cd <extract>/rust && CARGO_TARGET_DIR=<own> REXX_CORPUS_GATE=1 \
    cargo test --release -p rexx-exec --test corpus --no-fail-fast
```

| arm | what it deletes | on `parent` | on `base` |
|---|---|---|---|
| **ENHLEVEL** | `Class~enhanced` writes `setMethod`'s level, which is what BASE did | -- (BASE **is** this arm: `298 of 298`, exit 0, with the defect live) | **`298 of 299`**, `enhanced_unset` |
| **ENHSCOPE** | an enhancing method's scope is the object's class rather than `.nil` | `298 of 298`, exit 0 | **`298 of 299`**, `enhanced_unset` |
| **HIDESEND** | `lookup` matches only `Some(Some(..))`, so a hidden name falls through to the class | `298 of 298`, exit 0 | **`298 of 299`**, `setmethod_hidden` |
| **SCOPEOVR** | `start_scope.is_none()` goes, so an override reaches the object's dictionary | `298 of 298`, exit 0 | **`298 of 299`**, `setmethod_precedence` |

Every arm reddens exactly one row, and the `parent` column is the **"can fail is not adds coverage"**
check: each mutation applied to the tree *before* this round leaves the whole corpus green, so no
pre-existing row catches any of the four. For `ENHLEVEL` the check needs no mutation at all -- BASE
already answered `revealed class-mm` where the oracle answers `revealed enhanced-mm`, at rc 0 with
empty stderr, and its corpus was `298 of 298`.

What each arm printed, so the direction is on the record and not just the count:

* **ENHLEVEL**, `enhanced_unset`: `revealed class-mm` / `still class-mm`, then 97.1 at rc 159 on
  `e~ew`, where the oracle has `revealed enhanced-mm` / `still enhanced-mm` / `kept wrote` at rc 0.
* **ENHSCOPE**, `enhanced_unset`: `pool float[V]` against the oracle's `pool float[enh-wrote]`, every
  other line identical, rc 0 both sides -- so the arm moves the `pool` line and nothing else, which
  is what that line was added for.
* **HIDESEND**, `setmethod_hidden`: `send class-mm` at rc 0 with empty stderr, against the oracle's
  97.1 at rc 159. A silent wrong answer turned loud by the row.
* **SCOPEOVR**, `setmethod_precedence`: `override object-mm` against the oracle's `override class-mm`,
  rc 0 on both sides.

**One reading in this block was void and is recorded rather than dropped.** Re-running the `base`
baseline straight after restoring from the pristine copy printed `298 of 299` with
`setmethod_precedence` still failing -- `cp -a` restores the pristine mtimes, which are older than
the artifacts the `SCOPEOVR` build left, so cargo rebuilt nothing (`Compiling` appears zero times in
that log) and the mutated binary answered. `touch`ing the restored sources and re-running gives
`299 of 299`, exit 0, with eight crates compiled. The eight arm readings above are unaffected: each
arm's own log shows its crate compiling, because a patch writes the file and makes it newer.

## Minors

1. **`Loud::method_from_source`'s class-side bullet.** The false reason is deleted rather than
   rewritten, and the intra-doc link it existed for went with it. The bullet now says only that
   nothing files a class-side body here, and keeps the oracle measurement. The two clauses removed
   were "where a program's own clause reports its path" (which this task made false) and
   "`~subclass`'s enhancing table is the one route a send can take to one" (which
   `Interp::table_method_bodies`'s own doc contradicts by naming `Class~defineClassMethod`).
   Its divergence is on the list below.
2. **Two more unlisted loud divergences**, both re-measured here, both on the list below.
3. **`install_object_model`'s doc.** "They are also the lowest identities any run holds" is replaced
   by "Every identity minted afterwards is higher", which is the property the sentence is for and is
   what `record_access_scope`'s own `debug_assert` polices.
4. **The near-identical names** are now `install_enhancing_class_methods` and
   `install_enhancing_object_methods`.
5. **The dead arm in `write_object_method` is gone**: `receiver_kind` answers `Primitive::Instance`
   only after reading the same handle out of the arena, so the `heap.get_mut` miss and the
   non-`Instance` body are one impossibility, and it is now one `unreachable!` in the neighbouring
   line's style.
6. **`program_display_name`'s doc** now names its caller's question -- "`PARSE SOURCE`'s third word"
   -- rather than making the general claim the traceback path does not go through.

## Loud divergences this task leaves, the list the review said was short

Each re-measured here at BASE, fresh directory, three descriptors, both engines agreeing with each
other and not with the oracle. All loud: an `rexx-exec:` refusal at rc 120 where the oracle answers.

* **A class method built from source text.** `.methods~put('return 1/0', 'M')` then
  `.object~subclass("k", .Class, .methods)` then `zk~m`: oracle rc 214, `1 *-* return 1/0` and
  `Error 42 running M line 1:`; crate rc 120 `a class method built from source text`.
* **A `setMethod` source that does not parse.** `self~setMethod('MM', 'this is not rexx +++')`:
  oracle rc 221, `Error 35 running MM line 1:` with `35.901 Invalid expression.` and a
  `Compiled method "SETMETHOD" with scope "Object".` frame; crate rc 120
  `reporting a method source that does not parse (MM, 35.901: Invalid expression.)`.
* **`Object~RUN` from a class method of an unrelated class**, `check_restricted_method`'s third
  subject: oracle rc 158, `98.991 Method RUN may only be invoked ...` under a
  `Compiled method "RUN" with scope "Object".` frame; crate rc 120
  `method "RUN" of class "Object" is not implemented`.
* **A `Method` object from `Class~method` as `setMethod`'s second argument**, which the review's own
  memory control needed and which is not on any earlier list: `m = .k~method('SPARE')` then
  `self~setMethod('MM', m)` is oracle rc 0 and crate rc 120
  `a one-off method whose body this crate does not hold`. `Class~method` hands out a `Method` object
  with no `table_method_bodies` row, so the body cannot be copied into the dictionary entry.

The three the previous report already listed -- a class object as `setMethod`'s receiver,
`setMethod(name, .nil)`, and `~copy` -- are unchanged.

## Commits

* **`903c20283`** `Give Class~enhanced's methods a level of their own` -- the build, the three corpus
  programs with their `sourceline_oracle` expectations, `corpus/phase-5b.txt` and
  `EXPECTED_SUBSET_5B` together, the minors, the IMP-2 licence, and the plan paragraph.
* **`08fba0f61`** `Name the set this comment counted` -- `rust/CLAUDE.md`'s no-cardinality rule
  applied to the doc the commit above added.

The plan gains one paragraph at Task 3, per the phase rule that a wrong or missing claim is corrected
in the plan and not in the report: that `Class~enhanced`'s methods are a level of their own, with the
two C++ citations and both measured directions.

## Gates

At `08fba0f61`, tree clean, each status read unpiped from its own file, never chained.

```
cargo fmt --all --check                                        exit 0
cargo clippy --workspace --all-targets -- -D warnings          exit 0
cargo test --release --workspace                               exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace            exit 0
```

The gated release run prints **`299 of 299 matching`**, where the tree before this round printed
`298 of 298`.

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_c --test gate_table_d --no-fail-fast     exit 101 by design
```

```
  agree          loud=no  5b   usesem Defining Instance Methods with SETMETHOD or ENHANCED ...
  diverge-both   loud=yes 5b   methodsbyclass Class Library Notes ...
  table C   5a: 135 rows, 0 not yet `agree`     5b: 6 rows, 1 not yet `agree`
  table D   5a: 36 rows, 0 not yet `agree`      5b: 2 rows, 0 not yet `agree`
```

`usesem` still reads `agree` and the one red 5b row is `methodsbyclass`, which is Task 8's --
unchanged by this round.

## Each new or changed line against the nearest wrong explanation

* **`enhanced_unset`'s `revealed enhanced-mm`.** Nearest wrong explanation: `unsetMethod` removed
  nothing at all, so what is answered is whatever was there before. Separated inside the row by
  `shadowed one-off-mm` two lines earlier (the `setMethod` entry *was* in force) and by
  `setmethod_precedence`'s `revealed class-mm`, where the same call over a class-defined name does
  remove the one-off.
* **`enhanced_unset`'s `still enhanced-mm` and `kept wrote`.** Nearest wrong explanation: the second
  `unsetMethod` is a no-op because the *name* was already gone. Separated by `kept wrote`, which
  unsets a name that only the enhancing table ever defined and finds it still answering -- a
  different name, a different history, the same rule.
* **`enhanced_unset`'s `pool float[enh-wrote]`.** Nearest wrong explanation: every method on an
  object shares one pool, so any two would read each other. **Not separated inside this row**, and
  it is separated across rows rather than within: `setmethod_object_scope.rex`'s `float-sees [V]` is
  a `FLOAT` one-off failing to see an `OBJECT` one-off's write on the same object. Worth knowing
  that the separation is cross-row, as the review said of that row's own.
* **`enhanced_unset`'s `not-shared 0`.** Adds nothing over `usesem.rex`'s `still-not-shared 0`, and
  is there because a row that shows a definition surviving `unsetMethod` should say in the same
  breath that it never left the object.
* **`setmethod_hidden`'s `send`.** Nearest wrong explanation: the 97.1 comes from the name never
  having been defined anywhere. Separated inside the row by `before class-mm` and `revealed
  class-mm`, which are the same send answering from the class on either side of the hide.
* **`setmethod_precedence`'s `override class-mm`.** Nearest wrong explanation: the override answers
  the class's because the one-off is not in force yet. Separated by `after object-mm` on the line
  above, which is the same object and the same message name without the override.

```
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast    exit 0
```

103 `test result:` lines, all `ok`, no failures, one pass. The gated debug run took about eleven
minutes at a fifteen-minute load average in the twenties, which is this gate's own parallel width
(`ir_dual` alone runs at about thirty cores) and not contention: nothing else was running on the
machine.

## The performance sitting

The pin is current: `sha256sum bench-baselines/pinned/rexx-run-f558ea501` is
`857787f797e88dd94ab839d7b66f1e08cd1a08d22c9e7978d05cbfd49f70494e`, which is what
`bench-baselines/PINNED.md` records (that file is at `bench-baselines/PINNED.md`, not
`bench-baselines/pinned/PINNED.md` as the previous report's path had it).

Taken only after gate 5 had exited and the machine was confirmed quiet: no `cargo` and no `rexx-run`
process, the top process `claude` at 3.2% CPU, one-minute load average 5.86 and falling from the
gate.

```
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-f558ea501 \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup \
    --axis dispatchclass --axis bench-rexxcps/rexxcps.rex \
    --rounds 5 --task 3 --commit fc8532c9e --baseline bench-baselines/phase-5b-arms.tsv
```

380 rows appended to `bench-baselines/phase-5b-arms.tsv`, which held 381 lines beforehand and holds
761 afterwards; nothing else under `bench-baselines/` changed.

`instructions:u`, head against the pinned build, the `across_builds` rows:

| axis | tw small | ir small | tw large | ir large |
|---|---|---|---|---|
| alloc4c | -0.016% | -0.030% | -0.020% | -0.032% |
| arith | -0.140% | +0.012% | +0.010% | +0.006% |
| compound | +0.007% | +0.011% | +0.003% | +0.006% |
| emptyloop | +0.015% | +0.013% | +0.006% | +0.006% |
| strings | -0.014% | -0.023% | -0.016% | -0.025% |
| varlookup | +0.003% | +0.007% | +0.002% | +0.003% |
| **dispatchclass** | **+0.090%** | **+0.090%** | **+0.088%** | **+0.088%** |
| rexxcps | +0.005% | +0.006% | -- | -- |

**Every axis is well under the guard's 1% threshold, and `dispatchclass` is where the previous
sitting left it**: the rows the earlier sitting appended under commit `8bcf6375e` are +0.090% /
+0.089% / +0.089% / +0.089% on the same axis, so this round's own change adds nothing measurable to
it. That is what the design predicts -- the second level is inside the same `Interp::object_methods`
bool, and no bench program on this table attaches a method to an object, so `ObjectMethods::get` is
never called.

### A cycles move on `dispatchclass`, chased down rather than waved past

The sitting above also records `cycles:u`, and there `dispatchclass` reads +7.0% and +6.9% for `ir`
against the pinned build where the previous sitting's rows in the same file read -1.7% and -0.5%.
**Comparing those two sittings is not a valid measurement** -- they are separate runs on separate
days, and this project has already invented a 5% regression that way. So it was measured
interleaved, in one run, on a machine confirmed quiet, with `alloc4c` alongside as a second axis and
without `--baseline`, so nothing was appended to the committed file:

```
./target/release/rexx-arms --build base=<rexx-run built from e30055b37> \
                           --build head=target/release/rexx-run \
    --axis dispatchclass --axis alloc4c --rounds 5 --task 3 --commit fc8532c9e
```

| axis | instrument | tw small | ir small | tw large | ir large |
|---|---|---|---|---|---|
| dispatchclass | instructions:u | -0.072% | +0.001% | +0.000% | -0.000% |
| dispatchclass | cycles:u | **+3.9%** | **+3.7%** | **+5.9%** | **+3.2%** |
| alloc4c | instructions:u | -0.007% | +0.002% | +0.003% | +0.000% |
| alloc4c | cycles:u | -0.4% | +0.2% | -0.3% | +1.7% |

So the move is real, is specific to `dispatchclass`, and comes with **no instruction change at all**.
That rules out work being added and points at code layout, which needs its own control rather than an
argument -- a control that varies something about the change proves nothing.

**The do-nothing control.** A `git archive e30055b37` extract with two `pub fn`s added to
`ObjectMethods` in `rexx-core/src/body.rs` -- the same file and the same shape as this round's
additions -- that nothing calls, against the unmodified BASE binary, same command, same machine:

| axis | instrument | tw small | ir small | tw large | ir large |
|---|---|---|---|---|---|
| dispatchclass | instructions:u | +0.000% | -0.001% | -0.000% | +0.000% |
| dispatchclass | cycles:u | -1.1% | **-4.1%** | +0.6% | **-4.3%** |
| alloc4c | instructions:u | -0.002% | +0.003% | -0.003% | -0.002% |
| alloc4c | cycles:u | +0.0% | -2.0% | +0.3% | -1.2% |

**Adding code that never runs moves the same axis by up to 4.3% in the opposite direction.** The band
layout alone spans on `dispatchclass` here is about five percentage points, and this round's +3.2% to
+5.9% sits inside it. Both arms have identical instruction counts, which is the instrument this
project A/Bs with and the one the 1% guard is read on. So the cycles figure is not attributable to
work this change does, and it is recorded here rather than in a comment because it is a fact about
this machine and this axis, not about the code.

`bench-baselines/phase-5b-arms.tsv` is the only file the sitting touched; neither control wrote to it.

## Commits, final

* **`903c20283`** `Give Class~enhanced's methods a level of their own`
* **`08fba0f61`** `Name the set this comment counted`
* **`fc8532c9e`** `Point these C++ citations at the lines they name`
* **`ccc6dbd13`** `Record Task 3's fix-round sitting against the Phase 5b pin`

The five gates and the phase gate were read at `fc8532c9e`; `ccc6dbd13` after them adds rows to
`bench-baselines/phase-5b-arms.tsv`, which no test reads. Tree clean.

## What is left open, and for whom

* **`methodsbyclass`** is the one red 5b row of gate table C. Task 8's, untouched here.
* **The synthetic program per compiled method source is licensed, not fixed.** Reclaiming it is a
  refcount or a sweep over `Interp::programs`, sized in the IMP-2 section above, and belongs to
  whoever takes memory shape as a task.
* **Four loud divergences** are listed above, one of them (`Class~method`'s `Method` object as
  `setMethod`'s second argument) named for the first time here.
* **`~copy` carrying an object's own scope** is still out of reach, and the enhancing level would
  need copying with it when `~copy` lands.
