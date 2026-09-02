# Task 3 review: per-object methods, D66 and D67

Range under review: `f65694c8b..d4fac6707` (`4e613c349`, `8bcf6375e`, `d4fac6707`) on
`plan/rust-rewrite`.

Status: COMPLETE. **1 Critical, 3 Important, 6 Minor.** Every claim below names what was run and
what it printed; anything I could not verify is in "What I did not verify".

| # | severity | finding |
|---|---|---|
| CRITICAL-1 | Critical | `Class~enhanced`'s methods share `setMethod`'s dictionary, so `unsetMethod` deletes them and cannot reveal them. Silent wrong answer at rc 0, no committed row sees it. |
| IMPORTANT-1 | Important | The hidden-name form's effect on the **send** has no witness: a mutation deleting it leaves 298 of 298 green. |
| IMPORTANT-2 | Important | The synthetic program costs ~18.4 KB per compiled source and is not reclaimed even when the definition it holds is replaced; measured against the oracle, which stays flat. |
| IMPORTANT-3 | Important | A scope override skipping the object's dictionary also has no witness: 298 of 298 green under the mutation. |
| MINOR-1..6 | Minor | A stale justification naming an unlisted loud divergence, two more unlisted loud divergences, an overstated doc sentence, two near-identical function names, a dead arm, and a one-caller helper. |

The build itself is right on everything else I could reach: D66 both refusals with their frames and
four senders no row uses; D67's per-object pool including the two cases nobody wrote; the send path
for every non-instance receiver kind with the flag set; the library-package traceback unchanged; and
`collect_stress` finds no missed root. **Every one of the eleven new rows reddens** under at least
one of fifteen mutation arms, `usesem.rex` included.

## Method

Read-only against the live worktree: nothing was edited there and no `cargo` ran there. Three
`git archive` extracts under
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/ae5f3c99-11ad-4b29-9dfe-f3d991a1a398/scratchpad/t3rev`,
each with its own `CARGO_TARGET_DIR`:

* `base/` at `d4fac6707`, target `target-base` -- the binary every probe below uses;
* `parent/` at `f65694c8b`, target `target-parent` -- the before-and-after arm;
* `mut/` at `d4fac6707`, target `target-mut` -- the mutation workspace, one edit at a time,
  restored from `pristine/` copies between arms.

Every probe ran from a directory holding only its own `.rex` file (`cmp.sh`), so nothing else sat
on the external-routine search path; stdout, stderr and exit status were captured to three separate
files and compared with `cmp`, never merged. Both engines on every probe:
`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`. Oracle:
`/home/moritz/dev/repos/ooRexx/build/bin/rexx`. Nothing on `corpus/oracle-crashes.txt` was run.

Corpus arms:

```
cd <extract>/rust && CARGO_TARGET_DIR=<own> cargo build --release -p rexx-exec --bin rexx-run
cd <extract>/rust && CARGO_TARGET_DIR=<own> REXX_CORPUS_GATE=1 \
    cargo test --release -p rexx-exec --test corpus --no-fail-fast
```

Unmutated baseline in the mutation workspace, run before any arm: **`298 of 298 matching`**,
`exit 0`. `usesem.rex` is inside that set (`corpus/phase-5b.txt` carries
`gate-tables/concepts/usesem.rex`), so the corpus arm reads it too and no separate gate-table run
was needed to see it redden or stay green.

## Findings

### CRITICAL-1. `Class~enhanced`'s methods are stored where `setMethod`'s are, so `unsetMethod` deletes them and cannot reveal them. Silent, rc 0.

The oracle does not put an enhancing method in the object's own instance dictionary. `RexxClass::
enhanced` merges the table into a **dummy subclass's** `instanceMethodDictionary` and rebuilds that
subclass's instance behaviour (`classes/ClassClass.cpp:1454`-`:1461`), and `RexxObject::unsetMethod`
goes to `MethodDictionary::removeInstanceMethod`, which removes only when the object's own
`instanceMethods` table held the name (`behaviour/MethodDictionary.cpp:363`-`:371`). So on the oracle
there are **three** levels -- the object's `setMethod` dictionary, then the enhanced behaviour, then
the class -- and `unsetMethod` can only take away the first.

`install_enhanced_methods` (`dispatch.rs:5598`) calls `Interp::write_object_method`, which is the
same `Body::Instance::own` dictionary `setMethod` writes and `unsetMethod` removes from, so this
crate has two levels.

The one-program witness, all three descriptors, both engines, fresh directory:

```rexx
t = .stringtable~new
t['MM'] = "return 'enhanced-mm'"
e = .k~enhanced(t)
say '1' e~mm
e~shadow
say '2' e~mm
e~drop1
say '3' e~mm
say '4' .k~new~mm
::class k
::method mm
  return 'class-mm'
::method shadow
  self~setMethod('MM', "return 'one-off-mm'")
::method drop1
  self~unsetMethod('MM')
```

```
oracle        rc 0, stderr empty   1 enhanced-mm / 2 one-off-mm / 3 enhanced-mm / 4 class-mm
ir            rc 0, stderr empty   1 enhanced-mm / 2 one-off-mm / 3 class-mm    / 4 class-mm
tree-walker   rc 0, stderr empty   1 enhanced-mm / 2 one-off-mm / 3 class-mm    / 4 class-mm
```

Line 3 is a **silent wrong answer**: same exit status, empty stderr on both sides, different stdout.

Two simpler faces of the same defect, both measured the same way:

```rexx
t = .stringtable~new                          oracle rc 0:  before enh / has 1 / has2 1 / after enh
t['E1'] = "return 'enh'"                      crate  rc 159: before enh / has 1 / has2 0, then
e = .k~enhanced(t)                                          97.1 on `e~e1`
say 'before' e~e1
say 'has' e~hasMethod('E1')
e~drop1                                       ::method drop1
say 'has2' e~hasMethod('E1')                    self~unsetMethod('E1')
say 'after' e~e1
```

and `setMethod` over an enhancing name followed by `unsetMethod`, which the oracle answers `enh` and
this crate answers 97.1. In the `hasMethod` form the divergence is again silent before it becomes
loud: `has2` is `0` here and `1` on the oracle at rc 0 with empty stderr.

**No committed row sees any of this.** `usesem.rex` and `enhanced_scope.rex` both build an enhanced
object and neither sends `unsetMethod` to one, and `enhanced_scope.rex`'s `shared 0` line asks a
*different* object. The `LEAK` arm below shows the corpus does hold `enhanced` to "does not reach
the class's other instances"; nothing holds it to "is not the object's own `setMethod` entry".

**The code comment states the C++ fact that licences the storage and it does not licence it.**
`native_enhanced`'s doc quotes `ClassClass.cpp:1457` -- the methods are added with `.nil` scope "so
that these additional methods will look like they were added with setMethod" -- and then says "which
is what this crate stores them as". The quoted sentence is about the **scope** (and D67's pool
selection, which this crate gets right), not about which dictionary holds them.

### IMPORTANT-1. The hidden-name form's effect on the **send** has no witness: a mutation that deletes it leaves the whole corpus green.

`setmethod_hidden.rex`'s header says "hasMethod reports 0 and the send is 97.1 even though the class
defines the method". The row asks `hasMethod` twice and **never makes that send**: between `o~hide`
and `o~reveal` it only prints `has2` and `.k~new~mm`.

The behaviour is right. Probe, oracle rc 159, both engines byte-identical:

```rexx
o = .k~new ; o~hide ; say 'has' o~hasMethod('MM') ; say o~mm
```
```
has 0
     4 *-* say o~mm
Error 97 ... line 4:  Object method not found.
Error 97.1:  Object "a K" does not understand message "MM".
```

But nothing pins it. Arm **HIDESEND**: `Interp::lookup` matches only `Some(Some(entry))` from the
object's dictionary, so a hidden name falls through to the class instead of hiding it;
`native_has_method` and `answers_uninit` are untouched, so both still answer from the entry.

```
CARGO_TARGET_DIR=<own> REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus --no-fail-fast
298 of 298 matching        corpus exit 0
```

**Green over a deleted behaviour.** This is the phase's recurring failure shape, on a row whose own
comment names the behaviour it does not test. The fix is one line in `setmethod_hidden.rex` -- a send
of `MM` between `o~hide` and `o~reveal` -- plus its `sourceline_oracle` expectation.

### IMPORTANT-2. The unreclaimed synthetic program costs about 18 KB per compiled source and is not reclaimed even when the definition it holds has been replaced.

The report leaves this "not measured against a program that provokes it". Measured, `/usr/bin/time`,
`REXX_ENGINE=ir`, base binary:

| program | crate maxrss | oracle maxrss |
|---|---|---|
| `say 'done'` | 16,524 KB | -- |
| 20,000 `setMethod` calls, 20,000 **distinct** names | 385,032 KB | 111,904 KB |
| 20,000 `setMethod` calls, **one** name, each replacing the last | 383,452 KB | 20,588 KB |
| 20,000 `setMethod` calls of **one pre-built `Method` object** | 18,444 KB | 20,740 KB |

At 200 / 2,000 / 20,000 distinct names the crate is 20,316 / 54,300 / 385,032 KB, so the growth is
linear at about 18.4 KB per compiled source.

The third row is the finding rather than the second. Twenty thousand definitions of twenty thousand
names is state the oracle holds too (112 MB). Twenty thousand definitions of **one** name is a single
live one-off: the oracle stays flat at 20.6 MB and this crate still pays the whole 367 MB. The fourth
row is the control that says where it comes from: the same loop with a `Method` object rather than a
source string never enters `compile_method_source` and stays flat at 18.4 MB, so all of it is the
per-source synthetic program plus what hangs off it, and none of it is the per-object dictionary.

This is a shape, not a wrong answer, and D59-style licensing is the controller's call. What is worth
saying is that the superseded case is the one that costs, and it is the cheap one to fix.

### IMPORTANT-3. A scope override skipping the object's own dictionary also has no witness.

`Interp::lookup` guards the new branch with `start_scope.is_none()`, so `o~mm:.k` and `self~mm:super`
search the class hierarchy and never see a one-off. That is right -- probe, oracle rc 0, both engines
byte-identical, `plain object-mm` / `probe super=k-mm self=object-mm` / `ext k-mm` / `extkid kid-mm`
for a `kid` instance carrying a `MM` one-off over `k~mm` and `kid~mm`.

Arm **SCOPEOVR**: drop `start_scope.is_none()` from the guard, so an override finds the one-off.

```
298 of 298 matching        corpus exit 0
```

Green over a deleted behaviour, and the deletion is a silent wrong answer at rc 0. Same shape as
IMPORTANT-1 and the same one-line fix: a `:super` or `:.class` send in `setmethod_precedence.rex`.

## Priority 4: every new row reddened, re-run rather than trusted

Fifteen arms, each one edit applied to a `git archive d4fac6707` extract, built into one
`CARGO_TARGET_DIR` and restored from a pristine copy afterwards, then

```
CARGO_TARGET_DIR=<own> REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus --no-fail-fast
```

Unmutated baseline, run first, twice at the start and again after the last arm: **`298 of 298
matching`, exit 0**. `gate-tables/concepts/usesem.rex` is in `corpus/phase-5b.txt`, so the corpus run
reads it and no separate gate-table invocation was needed.

| arm | what it deletes | matching | rows reddened |
|---|---|---|---|
| **PREC** | the class behaviour is searched first, the object's dictionary only as a fallback | 296 | `setmethod_precedence`, `enhanced_scope` |
| **FLOAT** | a `FLOAT` one-off gets a scope of its own, so the pool is one per method | 297 | `setmethod_float_scope` |
| **OBJ** | `OBJECT` is read as `FLOAT` | 297 | `setmethod_object_scope` |
| **HIDDEN** | `setMethod` with no method removes the entry instead of hiding the name | 296 | `setmethod_hidden`, `setmethod_uninit` |
| **UNINIT** | `check_uninit` returns without asking | 297 | `setmethod_uninit` |
| **PRIV** | the natives' access scopes are not filed, so `SETMETHOD` is not private | 297 | `setmethod_private_refusal` |
| **RES** | `check_restricted_method` allows everything | 297 | `setmethod_restricted_refusal` |
| **RESALLOW** | the ancestor arm goes: only the object's own class may send | 297 | `setmethod_restricted_allowed` |
| **ENH** | the enhancing methods are installed after the `INIT` send | 297 | `enhanced_scope` |
| **NAME** | `program_display_name` always answers the program's path | 297 | `method_source_reported_name` (stdout half) |
| **NAME2** | `compiled_method_site` answers `None` | 297 | `method_source_reported_name` (stderr half) |
| **LEAK** | a `setMethod` definition reaches every object, not just the receiver | 294 | `usesem`, `setmethod_precedence`, `setmethod_hidden`, `enhanced_scope` |
| **HASMETHOD** | `native_has_method` does not consult the object's dictionary | 297 | `setmethod_hidden` |
| **HIDESEND** | a hidden name falls through to the class on a send | **298** | **none** |
| **SCOPEOVR** | a scope override reaches the object's dictionary | **298** | **none** |

**Every one of the eleven new rows reddens under at least one arm**, `usesem.rex` included. No arm
reddened a program that predates this task: the mismatch list under every arm names only rows this
commit adds, and 5a and earlier stayed at zero throughout.

Notes worth carrying:

* **`RESALLOW` fills the gap in the report's own table.** The report's ten arms redden every new row
  except `setmethod_restricted_allowed`, and it says so only obliquely ("MRES ... leaves
  `setmethod_restricted_allowed` green"). Deleting the ancestor arm of `checkRestrictedMethod` while
  keeping the exact-class arm reddens it, and reddens only it: `self by-self` and `class ALLOWED`
  survive and `ancestor` does not. So the row **can** fail, and its load-bearing half is the ancestor
  send. The `self` and `class` halves add nothing over `usesem.rex`, which already sends from `self`.
* **`PREC` reproduces the plan's own claim.** `usesem.rex` stays green while `setmethod_precedence`
  reddens, which is exactly what the brief says `usesem` cannot see, measured rather than argued.
  `enhanced_scope` reddening under `PREC` as well is not noise: its enhancing `INIT` has to beat the
  class's, which is a precedence question.
* **`LEAK` is an independent equivalent of the report's `MC1`.** Rather than repointing behaviours it
  makes the per-object dictionary consulted for any instance that has none of its own, so a
  definition reaches the class's other instances. `usesem` goes to `not-shared 1` /
  `still-not-shared 1`, which is the row's own claim failing.
* **`HASMETHOD` shows how narrow `hasMethod`'s own-dictionary arm is pinned.** Only
  `setmethod_hidden` reddens, and only on `has2 0`. `setmethod_precedence`'s `has 1` and `has2 1`
  lines cannot fail -- the class answers `1` for `MM` either way -- and the *positive* direction (a
  one-off under a name the class does not define reporting `1`) is not pinned by any row. It is
  correct: `o~hasMethod('MM')` is `1` on both engines and on the oracle for exactly that shape.

## Priority 5: the other eight rows against the nearest wrong explanation

Each row below names the nearest wrong explanation for its own output and what separates it. Where
the separator is a mutation arm it is the arm above; where it is a program, that program was run.

* **`setmethod_precedence`.** Wrong explanation for `after object-mm`: the definition *replaced* the
  class's entry rather than shadowing it. Separated inside the row: `other class-mm` (a fresh instance
  still answers the class's) and `revealed class-mm` after `unsetMethod`. Its `has 1` / `has2 1` lines
  are the ones that cannot fail -- `HASMETHOD` shows the class answers `1` either way. Its `inherited`
  pair is real coverage: a one-off over an *inherited* method.
* **`setmethod_float_scope`.** Three wrong explanations, all separated: the `FLOAT` pool is the
  class's (`class-pool class-fv` would move), one per method (`FLOAT` arm), one per program
  (`other r:FV` would read `r:written`). One that the row alone does **not** separate: `EXPOSE` in a
  one-off doing nothing at all would also print `r:FV` -- except that `one w:written` / `two r:written`
  in the same row requires a shared pool, so it is separated after all.
* **`setmethod_object_scope`.** Wrong explanation for `float-sees [V]`: `EXPOSE` in a one-off does
  nothing, so `v` reads as its own name. Not separated **within this row** -- `[V]` is what an unbound
  local prints too. It is separated by `setmethod_float_scope`, whose one-offs read each other's
  writes. Worth knowing that the separation is cross-row rather than in-row. Its `fresh-class-sees
  class-v` is not vacuous, contrary to a first reading: an `OBJECT` pool keyed per class rather than
  per object would print `obj-v` there.
* **`setmethod_hidden`.** See IMPORTANT-1: the comment's 97.1 send is not in the row and has no
  witness anywhere.
* **`setmethod_private_refusal`.** Wrong explanation: the refusal comes from anywhere else -- an
  unbuilt `.Object~new`, an unimplemented `SETMETHOD`, or the restricted check. The `PRIV` arm settles
  it in the sharpest possible way: with the access scopes unfiled the row does not merely redden, it
  produces **the other refusal** -- `98.991` at rc 158 under a `Compiled method "SETMETHOD" with scope
  "Object".` frame where the oracle has `97.2` at rc 159 and no method frame. So the row is green
  because the private check fires *first*, which is D66's whole claim, and the frame is what
  distinguishes them.
* **`setmethod_restricted_refusal`.** Wrong explanation: any refusal at all. Separated by
  `setmethod_restricted_allowed` staying green under `RES`, measured above.
* **`setmethod_restricted_allowed`.** Wrong explanation: the check allows everything. Separated by
  `RES` reddening its sibling. Its own load-bearing half is the ancestor arm (`RESALLOW`).
* **`enhanced_scope`.** The report's control re-run here: with `INIT` taken out of the enhancing
  table the oracle and both engines print `class init` / `peek Q` / `also also:Q` /
  `class-pool k-init`, agreeing byte for byte. So `class-pool SEEN` is green because the enhancing
  `INIT` shadowed the class's, as claimed. Its `class K` and `isa 1` lines cannot fail here (this
  crate builds no dummy subclass to be visible), but they are a real oracle property and guard a
  future implementation that does.
* **`method_source_reported_name`.** Wrong explanation for `named MM`: the third word of `parse
  source` is something else that happens to spell `MM`. It cannot be the program's path, which is what
  `NAME` shows. The row reads only the third word, so it does not pin the second: measured separately,
  the whole string inside a one-off is `LINUX METHOD MM` on the oracle and on both engines, and inside
  a `::METHOD` it is `LINUX METHOD <path>`. Both halves of the row now have an arm (`NAME`, `NAME2`).
* **`setmethod_uninit`.** The report's control re-run here. With `p~hide` removed, the oracle and both
  engines print `class uninit`, as the report says. Note for anyone re-running it: the *full*
  three-descriptor comparison of that control program diverges, because `p` is then inside the
  oracle's `Memory::SaveStackSize` window and the finalizer runs at termination rather than at the
  forced collection -- the artifact `uninit_instance_collected.rex`'s own header documents. With
  fifteen padding clauses between `~new` and `DROP` the control agrees on all three descriptors. The
  committed row is unaffected: its second half prints nothing either way.

## Priority 1: the send path for receiver kinds that are not instances

`Interp::own_method_entry` returns early on `!self.object_methods` and otherwise matches only
`Body::Instance { own: Some(_), .. }`, so a receiver that is not an arena instance takes the branch
and answers `None`. The flag is set by `Interp::write_object_method` on every non-removing write and
never cleared, which is what makes it sound: it is set whenever any entry exists.

Probed with the flag **set** -- one one-off installed first, then every other receiver kind sent to.
Oracle rc 0, both engines byte-identical on all three descriptors:

```
oneoff one-off / str 3 3 / nil The NIL object / cls K 0 / env 1 / stem x / other-inst 0
self-again 1 / cls-obj 0 / string-obj 0 / nil-obj 0 / int-obj 0 / arith 9 / concat ab
```

covering a String, `.nil`, a class object, `.environment`, a stem tail, a second instance, integer
arithmetic and concatenation, plus `hasMethod` asked of every one of those receiver kinds for the
one-off's name.

Sends that no longer resolve through the class alone, all agreeing:

* **a one-off shadowing a native**: `STRING` and `OBJECTNAME` set on one object, then `o~string`,
  `say o`, `o~objectName` and `'x' || o` -- oracle rc 0, `one-off-string` in all four positions.
  The rendering path really does send.
* **a hidden name reaching `UNKNOWN`**: `before class-mm` / `after unknown:MM` / `has 0`, rc 0.
* **`SETMETHOD`, `UNSETMETHOD` and `RUN` from a program context** on a String, `.nil`, a class object,
  an integer and an instance: `97.2 ... cannot accept private message` at rc 159, agreeing. This is a
  *gain* -- before this task `RUN` from a program context was rc 120.
* **a receiver with an `UNKNOWN` method**: `o~setMethod(...)` and `o~run(...)` from a program context
  answer `unknown SETMETHOD` / `unknown RUN` at rc 0, agreeing. The private refusal routes to
  `UNKNOWN` exactly as the oracle's does.
* **`.environment~setMethod(...)`** is rc 120 `method "SETMETHOD" of class "Directory" is not
  implemented`, where the oracle answers rc 0. Not a defect of this task and not Object's private
  method at all: `Setup.cpp:937`-`:938` gives `Directory` its **own** `SetMethod`/`UnsetMethod`
  (`AddProtectedMethod`, `DirectoryClass::setMethodRexx`), which is a different mechanism -- a
  directory entry that runs a method. The oracle's own traceback says so: `Compiled method
  "SETMETHOD" with scope "Directory".` Loud, pre-existing, and unowned by this task.

Only three `Setup.cpp` rows are `AddPrivateMethod` (`Run`, `SetMethod`, `UnsetMethod`, all on
`Object`), so the blast radius of filing the access scopes is those three names. `Object~RUN` is
still unimplemented, so `check_restricted_method`'s `run` arm is unreachable: a class method of an
unrelated class sending `obj~run(...)` is rc 120 here where the oracle is 98.991 at rc 158. Loud, and
`run` is not this task's.

`send`/`sendWith` are still rc 120 (`method "SEND" of class "Object" is not implemented`) where the
oracle answers from a program context. That is D64's list, owned by another 5b task, and the brief
asked it be checked: it is loud, not silent.

**No missed root.** `collect_stress` and `ir_dual` on the base extract:

```
cd base/rust && CARGO_TARGET_DIR=<own> REXX_CORPUS_GATE=1 \
    cargo test --release -p rexx-exec --test collect_stress --test ir_dual --no-fail-fast
test result: ok. 8 passed; 0 failed ...        (collect_stress)
test result: ok. 9 passed; 0 failed ...        (ir_dual)
exit 0
```

`ObjectMethods::trace` pushes each defined entry's `scope`, which is `ObjRef::NIL` for `FLOAT` and the
object's class for `OBJECT`; the pools themselves are `ScopePools` on the same `Body::Instance` and
were already traced. Nothing else in the new dictionary names an arena slot.

## Priority 2: D67

`FLOAT` is one pool per object, shared across that object's `FLOAT` one-offs and separate from the
class's. The committed rows use two methods and two instances, as the brief requires, and the `FLOAT`
arm reddens `setmethod_float_scope` when the pool is made one per method.

**The third case nobody wrote**, constructed here -- two objects of the same class, each with `FLOAT`
one-offs that *write*, plus a one-off added later to one of them:

```rexx
a = .k~new ; b = .k~new ; a~mk ; b~mk
say 'a-w' a~w1('A-value') ; say 'b-w' b~w1('B-value')
say 'a-r' a~r2           ; say 'b-r' b~r2
a~mk2                    ; say 'a-r3' a~r3
say 'b-r3-missing' b~hasMethod('R3')
```
```
oracle rc 0, both engines identical:
a-w w:A-value / b-w w:B-value / a-r r:A-value / b-r r:B-value / a-r3 r3:A-value / b-r3-missing 0
```

So the pools do not bleed between objects, and a `FLOAT` one-off added later joins the same
per-object pool rather than a new one. Agrees.

**A second case nobody wrote**, which separates "the object's own class" from "the defining scope of
the method that installed it": an `OBJECT`-scope one-off installed by a method **of the parent class**
on an instance of a **subclass**.

```
oracle rc 0, both engines identical:
k-sees-before k:k-v / kid-sees-before kid:kid-v / oneoff [obj-v] / k-sees k:k-v / kid-sees kid:obj-v
```

The pool is `kid`'s, not `k`'s, even though `k~go` is what called `setMethod`. `set_method_scope`
uses `interp.class_of_value(receiver)`, which is `classObject()`, and that is right. No committed row
has a subclass in it, so this is unwitnessed but correct.

## Priority 3: D66

Both refusals verified with their frames, and the `PRIV` arm above is what pins the discriminator
rather than the numbers. Four further senders, none of which any committed row uses, all agreeing on
both engines:

| sender | oracle | crate |
|---|---|---|
| a method of a **different instance** | rc 158, 98.991, with the `Compiled method "SETMETHOD" with scope "Object".` frame | agrees |
| a class method of a **subclass** of the receiver's class | rc 158, 98.991, same frame | agrees |
| a class method of an **unrelated** class reached from the receiver's own method | rc 158, 98.991, same frame | agrees |
| a class method of an **ancestor** class | rc 0 | agrees (and `RESALLOW` reddens it) |

`Class~enhanced`'s three argument refusals, each with the `Compiled method "ENHANCED" with scope
"Class".` frame, all agreeing: `93.901` for `.k~enhanced` and `.k~enhanced()`, `88.901` for
`.k~enhanced(, 'x')`, and `97.1 ... does not understand message "SUPPLIER"` for a String or `.nil` in
the table position.

## Priority 6: the `Sourceless` to `Named` widening

The library-package site is unchanged. `say .Validate~number('LENGTH', 'abc')`, which is the exact
program `sourceless_site`'s own doc names:

```
oracle rc 168, both engines identical:
  3700 *-*       Method NUMBER with scope "Validate" in package "REXX" (no source available).
     1 *-* say .Validate~number('LENGTH', 'abc')
Error 88 running REXX line 3700:  Invalid argument.
Error 88.902:  The LENGTH argument must be a number; found "abc".
```

`record_failure_at` asks `sourceless_site` **before** `compiled_method_site`, so a library level can
never be renamed by the new arm, and the whole 298-program corpus is green, which is the wider
evidence.

Mixed frames, both agreeing on all three descriptors:

* a one-off calling a class method that fails: `Error 42 running <path> line 6:` -- the innermost
  level is the program's, so the compiled method's echoed clause does not take the name;
* a class method calling a one-off that fails: `Error 42 running MM line 1:`;
* a one-off calling another one-off that fails: `Error 42 running INNER line 1:`.

## Priority 7: covered under IMPORTANT-2 above.

## Minor findings

**MINOR-1. `Loud::method_from_source`'s class-side bullet now carries a false reason, and the
divergence it covers is not on the report's open list.** `lib.rs:933`-`:937` says
"[`install_enhancing_methods`] declines the install rather than answering `running <path>`". That
reason is exactly what this task removed: `record_compiled_body` and `compiled_method_site` make the
crate answer `running <name>`, which `method_source_reported_name.rex` demonstrates for the
Method-object route (`Error 42 running lower line 1:`). Measured for the source-string route:

```rexx
.methods~put('return 1/0', 'M')
zk = .object~subclass("k", .Class, .methods)
say 'answer' zk~m
::method z
  return 1
```
```
oracle       rc 214   1 *-* return 1/0 / 3 *-* say 'answer' zk~m / Error 42 running M line 1:
crate, both  rc 120   rexx-exec: a class method built from source text is not implemented (Phase 5)
```

The refusal is right to stay (nothing files a class-side body), but its stated reason is not, and
`docs`-level review will not catch it. The same bullet's "`~subclass`'s enhancing table is the one
route a send can take to one" sits beside `Interp::table_method_bodies`'s own doc naming
`Class~defineClassMethod` as such a route. Prefer deleting the justification to rewriting it.

**MINOR-2. Two loud divergences this task makes newly reachable are not on the report's list.** Both
are pre-existing refusals reached by a new route, both loud:

* a `setMethod` source that does not parse -- `self~setMethod('MM', 'this is not rexx +++')` is
  `Error 35 running MM line 1:` at rc 221 on the oracle and rc 120 `reporting a method source that
  does not parse (MM, 35.901: Invalid expression.)` here. The oracle's `running MM` is the very name
  this task learned to report, so this one is closer to done than it reads;
* `Object~RUN` from a method context, `check_restricted_method`'s third subject, which is rc 120
  where the oracle answers 98.991 at rc 158. `run` is not this task's, but D66 is, and the D66 check
  currently has two of its three subjects.

**MINOR-3. `install_object_model`'s doc overstates.** "They are also the lowest identities any run
holds" is false as written: `Setup.cpp` defines the whole `Class` class (`:455` onward) before
`Object`'s three private rows at `:549`-`:551`, so lower `MethodId`s exist. The load-bearing property
-- lower than every identity minted afterwards, so `record_access_scope` can keep appending in key
order -- is true and is what the sentence is for.

**MINOR-4. `install_enhancing_methods` and `install_enhanced_methods` differ by one character and do
different things** (a class-side install from `~subclass`'s table; an object-side install from
`Class~enhanced`'s). Both are cited from doc comments in the other's file. A rename would cost
nothing now and less than later.

**MINOR-5. A dead arm.** `write_object_method` returns `Loud::receiver_class("a value whose object is
no longer live")` when `heap.get_mut` misses, after `receiver_kind` has already answered
`Primitive::Instance` for the same handle. The very next line uses `unreachable!` for the identical
kind of impossibility. One of the two is the crate's style; this is the other.

**MINOR-6. `program_display_name` has one caller and reads as though it had two.** `PARSE SOURCE`
calls it; the traceback path reads `compiled_method_names` directly in `compiled_method_site`. Its
doc ("The name a program reports under") is the general claim, and the traceback does not go through
it.

## What I did not verify

* **The five gates and the phase gate**, per the brief: the controller re-ran the phase gate at
  `d4fac6707` and the gate-5 flake is fixed in `76a669c24`. Nothing here re-reads them.
* **The performance sitting.** Not re-run. The `+0.09%` on `dispatchclass` is consistent with the
  mechanism the report names (`access_scope_of`'s emptiness test never firing once the natives file
  rows), and that mechanism is real in the code, but I did not measure it.
* **`287 of 287` before the task.** Not run; 298 minus the eleven rows this commit adds is 287, which
  is consistent, and I did not build the parent's test binaries to confirm it.
* **`~copy` carrying an object's own scope.** `~copy` is unimplemented, as the report says.

## Corrections to my own probes, recorded because they nearly became findings

An early probe had an enhanced object's `UNINIT` running at a forced collection here and at
termination on the oracle -- ten oracle runs, deterministic. It is **not** a defect: the object was
inside the oracle's `Memory::SaveStackSize` window, which `uninit_instance_collected.rex`'s own
header documents. The same probe with a plain `~new` object and no `setMethod` at all diverges
identically on the **parent** build, and with fifteen padding clauses between construction and `DROP`
both the enhanced and the plain form agree on all three descriptors. The control is what separated
them; the first reading would have been a wrong finding about `enhanced`.
