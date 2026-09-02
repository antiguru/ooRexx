# Task 3 report: per-object methods, their scopes, and the restricted-private check

Status: DONE. `usesem` agrees on both engines, all five gates are green, the sitting is taken.

BASE: `f65694c8b` on `plan/rust-rewrite`. Landed as `4e613c349`, `8bcf6375e` and `d4fac6707`.
Every `file:line` citation below was taken against BASE and has moved since.

The five gates were read at **`8bcf6375e`**; `d4fac6707` after it changes only
`docs/superpowers/plans/2026-08-27-phase-5b.md` and `bench-baselines/phase-5b-arms.tsv`, neither of
which any test reads (`/bin/grep -rn "bench-baselines" crates/ --include=*.rs` matches comments and
`rexx-arms`'s own usage line, and nothing else).

## Premises re-measured (all four hold)

Oracle `parse version` = `REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026`, rc 0.
Crate binary `rust/target/release/rexx-run` rebuilt at BASE (`cargo build --release -p rexx-exec
--bin rexx-run` reported `Finished` with nothing to do, so the on-disk binary is the BASE tree's).

1. **`usesem` itself.** Oracle rc 0, stdout `setmethod one-off` / `not-shared 0` /
   `enhanced enhanced` / `still-not-shared 0`, stderr empty. Crate rc 120 on both engines, stdout
   empty, stderr `rexx-exec: method "SETMETHOD" of class "Object" is not implemented (Phase 5)`.
2. **`[]=` exists for no receiver kind.** `/bin/grep -an '"\[\]="' crates/rexx-exec/src/dispatch.rs`
   exits 1 with no output. `StringTable`'s native rows are `[]`, `AT`, `PUT`, `UNKNOWN`
   (`dispatch.rs:455`-`:459`), no setter.
3. **The syntax parses.** `a = (7,8,9)` / `say a[2]` / `a[2] = 5` / `say a[2]`: oracle rc 0 `8` `5`;
   crate rc 120 both engines, stdout `8`, stderr
   `rexx-exec: method "[]=" of class "Array" is not implemented (Phase 5)`. The read printing first
   is what says the parse is fine.
4. **`compile_method_source` exists**, `crates/rexx-exec/src/dispatch.rs:3872`.

## Oracle measurements taken before building

Each run from its own fresh directory, three descriptors read separately, crate on both engines.

| probe | oracle | crate (both engines) |
|---|---|---|
| `usesem.rex` | rc 0, four lines | rc 120 `SETMETHOD ... not implemented` |
| two `FLOAT` one-offs on one object, plus a second instance | rc 0 `one w:written` / `two r:written` / `other r:FV` | rc 120 |
| `OBJECT` scope writing a name the class method reads | rc 0 `class-sees class-v` / `float-sees [V]` / `object-sees [obj-v]` / `class-sees2 obj-v` | rc 120 |
| one-off shadowing a class method, then `unsetMethod` | rc 0 `before class-mm` / `after object-mm` / `has 1` / `revealed class-mm` / `has2 1` / `other class-mm` | rc 120 after printing `before class-mm` |
| `unsetMethod` for a name only the class defines | rc 0 `still class-mm` | rc 120 `UNSETMETHOD ... not implemented` |
| `unsetMethod` for a name nothing defines | rc 0, no error | rc 120 |
| `.Object~new~setMethod(...)` from a program | rc 159, `97.2 ... cannot accept private message "SETMETHOD" from this context.`, **no** method frame | rc 120 |
| `.Other~poke(o)` class method, `o` not an `Other` | rc 158, `98.991 Method SETMETHOD may only be invoked ...`, **with** `Compiled method "SETMETHOD" with scope "Object".` | rc 120 |
| the same from a class method of `o`'s own class | rc 0 `r ALLOWED` / `v x` | rc 120 |
| `setMethod('MM')` with no method argument | hides: `has 0`, then the send is `97.1` at rc 159 | rc 120 |
| `setMethod(..., 'BOGUS')` | rc 168, `88.916 Argument 3 must be one of "FLOAT" or "OBJECT"; found "BOGUS".`, with the `Compiled method` frame | rc 120 |
| a one-off whose body is `return 1/0` | rc 214, `1 *-* return 1/0` then **`Error 42 running MM line 1:`** | rc 120 |
| `parse source` inside a one-off | rc 0 `LINUX METHOD MM` | rc 120 |
| a one-off shadowing an **inherited** method, then `unsetMethod` | rc 0 `before base-mm` / `after one-off` / `revealed base-mm` | rc 120 |
| `.stringtable~new`, `~put`, `[]`, `[]=` | rc 0 | rc 120 `NEW of class "StringTable"` |
| `.stringtable~new('abc')` | rc 163, `93.923 Invalid length argument specified; found "abc".` with `INIT` and `NEW` frames | rc 120 |
| `.K~enhanced(t, 'arg1')` where `t` carries `INIT` and `PEEK` | rc 0: the **enhanced** `INIT` runs and the class's does not (`kinit SEEN`), `~class~id` is `K`, `~isA(.K)` is 1 | rc 120 |
| `~copy` of an object carrying a one-off | rc 0 `copy one-off` / `has 1` / `ident 0` | `~copy` is not implemented here at all, so this stays out of scope |

## Two premises the brief does not carry, found by re-measuring

**`compile_method_source` exists but keeps no body**, and its own doc comment says so
(`dispatch.rs:3846`-`:3859`: "The compiled body is validated and not retained"). So the brief's
premise 4 is true as stated and not sufficient: `usesem`'s `setMethod("EXTRA", "return 'one-off'")`
has to *run*, and nothing in the tree can run a method compiled from source text. Making one runnable
is this task's, and it is the largest single piece of it.

**A compiled method reports under its own name, not the program's path.** Measured above: `Error 42
running MM line 1:` and `parse source` answering `LINUX METHOD MM`. So a runnable compiled body also
needs a per-body reported name, which no program in this crate has ever needed.

## What was built

* **`Body::Instance` gains `own: Option<Box<ObjectMethods>>`** (`rexx-core/src/body.rs`), the
  dictionary `setMethod` writes. Boxed, so an object that never takes one costs a pointer and
  `size_of::<Body>() <= 80` still holds. `Body::trace` traces each entry's scope, in the position
  `ScopePools::trace` is in and for its reason.
* **`MethodId` moved to `rexx-core::body`** and `rexx-classes` re-exports it, which is exactly what
  `BehaviourHandle`'s own doc says was done for it and for the same reason: `Body::Instance` has to
  name the type and `rexx-classes` depends on `rexx-core`. The dead `behaviour.rs` declaration of the
  same name is gone; `BehaviourTable` now uses the moved one.
* **`Interp::lookup` searches the object's own dictionary first**, gated on a monotone
  `Interp::object_methods` bool so a program that never sends `SETMETHOD` pays one load and one
  branch per send. A scope override (`o~m:.K`) skips it, which is what `superMethod` does.
* **`compile_method_source` now retains a runnable body.** `Interp::record_compiled_body` files the
  parsed `main` as a `::METHOD` directive in a synthetic program of its own and hangs the
  `InstalledMethodBody` on the `Method` object, so a send enters it through the existing path.
* **A compiled method reports under its own name.** `FailureSite::Sourceless` is renamed `Named` and
  its doc widened to the two levels that reach it; `Interp::compiled_method_names` supplies the name,
  and `PARSE SOURCE`'s third word reads the same map.
* **`Setup.cpp`'s `AddPrivateMethod` rows are now derived** rather than folded into `AddMethod`
  (`rexx-classes/build.rs`), so `Object~RUN`, `Object~SETMETHOD` and `Object~UNSETMETHOD` carry
  `Access::Private` and the 97.2 refusal happens at dispatch, with no method frame, exactly as
  measured.
* **New natives:** `Object~SETMETHOD`, `Object~UNSETMETHOD`, `Class~ENHANCED`,
  `StringTable~NEW` (class side), `StringTable~INIT`, and `[]=` on `StringTable` and `Directory`
  (one donated `HashCollection::putRexx` under a second name).
* **`native_new` split**, so `enhanced` takes `completeNewObject`'s steps and puts the enhancing
  methods in before the `INIT` send.

## The probes above, re-run against the built crate

Every probe in the table above now agrees with the oracle on **both engines**, byte for byte on all
three descriptors, except `~copy`, which this crate does not implement at all and which stays the
loud refusal it was.

## Probing past the row: what it found

Each shape below was run against both engines and the oracle, three descriptors, fresh directory.

**A one-off `UNINIT` was a silent wrong answer and is fixed.** `self~setMethod('UNINIT', ...)` runs
the finalizer on the oracle, both from a forced collection and at termination, and the first build of
this task ran nothing at rc 0 with empty stderr on both sides. `RexxObject::defineInstanceMethod`
calls `checkUninit` (`ObjectClass.cpp:2314`) and `deleteInstanceMethod` calls it again (`:2338`), so
both mutators re-ask the question; `Interp::check_uninit` does the same, and `answers_uninit` now
consults the object's own dictionary so a hidden `UNINIT` cancels a class's.

**`Class~enhanced`'s two argument refusals were one.** `.K~enhanced` and `.K~enhanced()` are both
`93.901 Not enough arguments for method; 1 expected.` at rc 163 -- the second because a trailing
omission is dropped from the count -- while `.K~enhanced(, 'x')` is `88.901 Missing argument;
argument methods is required.` at rc 168. The first build answered 93.903 for all three.

**`setMethod` on a class object was loud with a false message.** `self~setMethod(...)` from a class
method of the receiver's own class is reachable and the oracle answers rc 0. This crate keeps the
dictionary in `Body::Instance` and has nowhere to put one on a class object, so it refuses; the first
build refused with `a value whose object is no longer live`, which is untrue, and it now says `a
receiver with no scope of its own`.

**An in-crate refusal test lost a row, because the shape it pinned now agrees.** `.k~define("m",
'return 1')` then putting `.k~method("M")` into `.methods` and building a class from it was a loud
refusal because no body was held; it now answers, and agrees with the oracle byte for byte including
`Error 42 running m line 1:` for a failing body. The row is replaced by
`corpus/lang/method_source_reported_name.rex`, which is a differential witness where the in-crate row
could only assert a refusal.

**Item A is untouched.** `.K~define('X', "return 'from-source'")` then `.K~new~x` is still rc 120
`method "X" of class "K" is not implemented (Phase 5)` on both engines, because
`Interp::define_method_object` files no `method_bodies` row. That is Task 9's, and this task makes it
a smaller change rather than doing it.

### Shapes checked and found to agree

`setMethod` on an instance whose class later gains the same method (the instance keeps the behaviour
it was built with, so `unsetMethod` afterwards leaves 97.1); a one-off shadowing an inherited method;
`unsetMethod` for a name only the class defines and for a name nothing defines; a one-off calling
another method through `self`; a class method calling a one-off through `self`; a one-off taking
arguments through `USE ARG`; an array source; a lower-case name through both the symbol and the
literal send; `.K~method` not seeing a one-off; `.local['X'] = 'v'` and `.environment['X'] = 'v'`
through the `[]=` row this task adds; `.stringtable~new(10)` and `.stringtable~new('abc')`.

### Known loud divergences this task leaves

* **`self~setMethod(...)` with a class object as the receiver** is rc 120 `a receiver with no scope
  of its own`, where the oracle answers rc 0. Loud.
* **`self~setMethod('MM', .nil)`** is rc 120 `a method source that is neither a string nor an array`,
  where the oracle raises `93.974 The method argument must be a string, array, or method object.` at
  rc 163. This is `compile_method_source`'s existing refusal reached by a new route: the same arm
  covers `.environment`, where the crate's own reason for refusing rather than raising holds, and
  narrowing it to `.nil` alone would change `~define` and `~defineMethods` as well. Loud.
* **`~copy`** is not implemented at all, so an object's own scope surviving a copy is out of reach.
  Loud.

## Controls, recorded as run

Committed at `4e613c349`. Every arm below ran in a `git archive 4e613c349 | tar -x` extract with its
own `CARGO_TARGET_DIR`, never in the worktree, under

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test corpus --test gate_table_c --test gate_table_d --no-fail-fast
```

The extract's own baseline, run twice and identical both times: **298 of 298 matching**; gate table C
`5a: 0 not yet agree`, `5b: 6 rows, 1 not yet agree` (`methodsbyclass`, which is Task 8's); gate
table D `5a: 0`, `5b: 0`. Exit 101 throughout, by design, because `methodsbyclass` is red.

| arm | what it changes | corpus | which rows reddened | gate rows moved |
|---|---|---|---|---|
| **MC1** | `setMethod` writes into the class's instance dictionary and repoints the receiver at the class's new behaviour, so the definition reaches the class's other instances | 293 of 298 | `usesem`, `setmethod_precedence`, `setmethod_float_scope`, `setmethod_object_scope`, `setmethod_uninit` | table C 5b **1 to 2** not-agree (`usesem`); 5a unmoved on both tables |
| **MP1** | the class behaviour is searched before the object's own dictionary | 296 of 298 | `setmethod_precedence`, `enhanced_scope` | **none**: table C 5b stays at 1 not-agree, so `usesem` still agrees; 5a unmoved on both tables |
| **MF** | a `FLOAT` one-off gets a scope of its own rather than the shared `.nil`, so the pool is one per method | 297 of 298 | `setmethod_float_scope` | none |
| **MO** | `OBJECT` is read as `FLOAT` | 297 of 298 | `setmethod_object_scope` | none |
| **MU** | `check_uninit` returns without asking | 297 of 298 | `setmethod_uninit` | none |
| **MH** | `setMethod` with no method argument removes the entry instead of hiding the name | 296 of 298 | `setmethod_hidden`, `setmethod_uninit` | none |
| **MPRIV** | the natives' access scopes are not filed, so `SETMETHOD` is not private | 297 of 298 | `setmethod_private_refusal` | none |
| **MRES** | `check_restricted_method` allows everything | 297 of 298 | `setmethod_restricted_refusal` | none |
| **MENH** | the enhancing methods are installed after the `INIT` send | 297 of 298 | `enhanced_scope` | none |
| **MNAME** | a compiled method's frame reports the program's path | 297 of 298 | `method_source_reported_name` | none |

**MC1 is `usesem`'s own committed control** and it fires: gate table C's 5b column goes from one
not-agree to two, the new one being `usesem`.

**MP1 is the search-order control the plan names, and its second half holds.** Every gate row that
agreed before the mutation still agrees after it -- `5a: 0 not yet agree` on both tables, table C's
5b still at its one red row (`methodsbyclass`) and table D's 5b still at none -- while
`setmethod_precedence` reddens. `usesem` agreeing under MP1 is the plan's own claim, measured.

**"Can fail" is not "adds coverage", checked for every arm.** No arm reddens a program that predates
this task: the mismatch list under each mutation contains only rows this commit adds, and the corpus
count under the base is 298 where it was 287 before. `MRES` is the sharpest of these -- it reddens
`setmethod_restricted_refusal` and leaves `setmethod_restricted_allowed` green, which is what pins
the refusal to `checkRestrictedMethod`'s rule rather than to a blanket refusal.

## Two rows checked against the nearest wrong explanation

Both are the "agrees for the wrong reason" hazard, and both are program-level controls rather than
crate mutations, because the question is what the *oracle* is doing.

**`setmethod_uninit`'s second half** prints nothing after `p~hide`, and the wrong explanation is that
no finalizer would have run anyway. Control: the same program with the `hide` call removed prints
`class uninit` on the oracle and on both engines. So the row is green because hiding the name
cancels the registration.

**`enhanced_scope`'s `class-pool SEEN`** is the class's variable reading as its own name, and the
wrong explanation is that no `INIT` ran at all. Control: the same program with `INIT` taken out of the
enhancing table prints `class init` and `class-pool k-init` on the oracle and on both engines. So the
row is green because the enhancing `INIT` shadowed the class's, which is what the comment claims.

**`setmethod_uninit`'s ordering is reproducible on the oracle**, ten runs of ten, all
`one-off uninit|first done|second done` -- D61's requirement, and why each object is dropped and
forced on its own.

## Commits

* **`4e613c349`** `Give an object methods of its own` -- the whole of the build, the ten corpus
  programs, their `sourceline_oracle` expectations, `corpus/phase-5b.txt` and `EXPECTED_SUBSET_5B`
  together, and the plan correction.
* **`8bcf6375e`** `Name the sets these comments counted` -- `rust/CLAUDE.md`'s no-cardinality rule
  applied to the comments the commit above added.

## The plan, corrected where it was wrong

`docs/superpowers/plans/2026-08-27-phase-5b.md`, Task 3, gains two paragraphs, per the phase rule
that a wrong claim is corrected in the plan and not in the report:

* that `compile_method_source` keeps no body, so making one runnable is the largest single piece of
  this task, and that a compiled method reports under its own name rather than the program's path,
  with the measurements;
* that a one-off `UNINIT` is this task's, with `checkUninit`'s two call sites and both measured
  directions.

## Gates

Each status read unpiped from its own file, never chained, at `8bcf6375e` with the tree clean.

```
cargo fmt --all --check                                        exit 0
cargo clippy --workspace --all-targets -- -D warnings          exit 0
cargo test --release --workspace                               exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace            exit 0
```

The gated release run prints **`298 of 298 matching`**, where the tree before this task printed
`287 of 287`.

The phase gate, `REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test
gate_table_c --test gate_table_d --no-fail-fast`, **exit 101 by design**:

```
  agree          loud=no  5b   usesem Defining Instance Methods with SETMETHOD or ENHANCED ...
  diverge-both   loud=yes 5b   methodsbyclass Class Library Notes ...
  table C   5a: 135 rows, 0 not yet `agree`     5b: 6 rows, 1 not yet `agree`
  table D   5a: 36 rows, 0 not yet `agree`      5b: 2 rows, 0 not yet `agree`
```

`usesem` reads `agree`; the one 5b row still red is `methodsbyclass`, which is Task 8's.

```
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast    exit 0
```

**That fifth gate needed two runs and the first one's failure is not this task's.** The first
reported `builtin::datetime::tests::time_e_does_not_reset_the_anchor_time_r_does` with
`e2 = 0.161732, e3 = 0.230071`, whose assertion is `e3 > e2 * 1.5` over two real wall-clock burns,
while the run itself was carrying a fifteen-minute load average of **28.19** (one-minute **35.94**,
all of it the gate's own parallel test binaries). Re-run standalone: **ten passes of ten**. The whole
command re-run: **exit 0**, 103 `running` sections and 103 `test result:` lines, no failures. This
crate's records carry the same test failing the same way twice before, in Phase 5a's Task 14 and in
the gate-close plan's Task 4, each time under load and each time green on a retry. Nothing here
touches `TIME`.

## The performance sitting

The pin is current: `git log --oneline f558ea501..HEAD -- ':/rust/crates' ':/rust/Cargo.toml'
':/rust/Cargo.lock'` (with the `:/` prefix, from the repository root) lists `3148d8cdd`,
`3290d5763` and `f65694c8b`, each of which this phase's ledger names, and
`sha256sum bench-baselines/pinned/rexx-run-f558ea501` is
`857787f797e88dd94ab839d7b66f1e08cd1a08d22c9e7978d05cbfd49f70494e`, which is what `PINNED.md`
records.

```
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-f558ea501 \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup \
    --axis dispatchclass --axis bench-rexxcps/rexxcps.rex \
    --rounds 5 --task 3 --commit 8bcf6375e --baseline bench-baselines/phase-5b-arms.tsv
```

Machine quiet, one-minute load average 2.47 at the start. 380 rows appended to
`bench-baselines/phase-5b-arms.tsv`, which held its header row and nothing else beforehand and holds
381 lines afterwards; nothing else under `bench-baselines/` changed.

`instructions:u`, head against the pinned build, the `across_builds` rows:

| axis | tw small | ir small | tw large | ir large |
|---|---|---|---|---|
| alloc4c | -0.011% | -0.020% | -0.024% | -0.037% |
| arith | +0.025% | +0.009% | +0.024% | +0.024% |
| compound | +0.008% | +0.010% | +0.004% | +0.006% |
| emptyloop | +0.013% | +0.013% | +0.005% | +0.005% |
| strings | -0.015% | -0.023% | -0.016% | -0.025% |
| varlookup | +0.003% | +0.006% | +0.002% | +0.003% |
| **dispatchclass** | **+0.090%** | **+0.089%** | **+0.089%** | **+0.089%** |
| rexxcps | +0.003% | +0.005% | -- | -- |

**Every axis is under a tenth of a percent, so under the guard's own 1% threshold nothing here is a
finding.** The loudest axis is the one the change is on: `dispatchclass` is 4,000,000 sends and moves
+0.09%, which is `Interp::access_scope_of`'s early return no longer firing. The natives' access
scopes are now always filed, so `special_methods` is never empty and every send pays a binary search
over its rows instead of one `is_empty` branch. The per-object dictionary itself costs less than
that: it is behind a bool that is false for every program in this table.

## What is left open, and for whom

* **`methodsbyclass`** is the one 5b row of gate table C still red. Task 8's, unchanged by this task.
* **Item A** -- a `~define`d method from source text is still rc 120 on an instance -- is Task 9's,
  and this task makes it smaller: `compile_method_source` now files a body against the `Method`
  object, so `Interp::define_method_object` has a row to copy into `method_bodies`.
* **`~copy`** is not implemented, so the spec's measured "a copy carries the object's own scope with
  it" has no witness. It is a loud refusal, not a wrong answer.
* **`self~setMethod(...)` on a class object** is a loud refusal here and rc 0 on the oracle. Giving a
  class object a dictionary of its own means somewhere other than `Body::Instance` to keep one,
  since a class identity names no arena slot.
* **`self~setMethod('MM', .nil)`** is a loud refusal where the oracle raises 93.974, for the reason
  `compile_method_source`'s existing refusal gives about `.environment`.
* **A synthetic program per compiled source is never reclaimed.** A loop calling `setMethod` with a
  source string grows `Interp::programs` without bound, which is the shape
  `Interp::hold_method_object`'s own doc already records for `~define`. Not measured against a
  program that provokes it.
