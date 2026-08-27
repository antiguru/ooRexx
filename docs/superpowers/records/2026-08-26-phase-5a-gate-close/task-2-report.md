# Task 2 report: `Method~scope`

**Base** `f81131f9d`. **Commits** `aa96cf05d` (the mechanism) and `637d0dd64` (the plan
correction and three quantifiers the first commit shipped).

## The row

`corpus/gate-tables/concepts/xscope.rex` agrees on both engines, three descriptors read
separately, oracle **rc 159**.

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  timeout -s KILL 10 /home/moritz/dev/repos/ooRexx/build/bin/rexx \
  /home/moritz/dev/repos/ooRexx-rust-rewrite/rust/corpus/gate-tables/concepts/xscope.rex )
( memcap 1G env REXX_ENGINE=ir          timeout -s KILL 20 rust/target/release/rexx-run <same> )
( memcap 1G env REXX_ENGINE=tree-walker timeout -s KILL 20 rust/target/release/rexx-run <same> )
```

```
oracle rc 159   ir rc 159 (stdout SAME, stderr SAME)   tree-walker rc 159 (stdout SAME, stderr SAME)
stdout: base BASE
        sub SUB
stderr:        *-* Compiled method "METHOD" with scope "Class".
             6 *-* say 'inherited-is-not-own' .sub~method("BASEONLY")
        Error 97 running <path> line 6:  Object method not found.
        Error 97.1:  Object "The SUB class" does not understand message "BASEONLY".
```

`REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test gate_table_c
--no-fail-fast` prints the row as

```
  agree          loud=no  5a   xscope Scope   depth 1 parent xcremet provide.xml:423 xscope.rex
```

## The brief was wrong by one write, and the plan is corrected

The brief and the plan said Task 21 had put `scope: Option<ObjRef>` on `NativeObject`, "so the
field exists; what is missing is the reader."

**False.** `Interp::method_object` -- the `Class~method` route, and the only caller -- minted a
method object and never wrote its scope. A reader alone would have answered `.nil` for every name
a class's own dictionary holds, and for the row's first two lines with them. The scope was written
only where `method_new_scope` ran, which is the `~define` family. So the build is a reader **and**
one write, and `native_method` now passes the dictionary slot's scope down.

Corrected in `docs/superpowers/plans/2026-08-26-phase-5a-gate-close.md`, in Task 2's **Build**
paragraph, saying what was removed and why -- not in this report, which the next reader of the
task will not see.

Nothing else in the brief conflicted. `.Method~id`, `Method~class` and the row's error tail were
already there, as it said, and `~id` on the class `~scope` answers is `Class~ID`, which needed no
work.

## Where the scope comes from, and why not the receiver

`native_method` reads `MethodSlot::Defined { scope, .. }` and hands that to `method_object`,
rather than assuming the class the send went to. The two differ only where
`MethodDictionary::setMethodScope` has rewritten a dictionary to another class's scope, which
`RexxClass::inheritInstanceMethods` (`classes/ClassClass.cpp:560`-`:563`) does to its **donor**.
No program can reach that: `removeSetupMethods` deletes the method from the saved image, and this
crate's row for it lives in `SETUP_METHODS`, not the program-visible table. The mixin classes it
runs over in `CoreClasses.orx:80`-`:87` are unreachable too -- measured, oracle rc 0:
`.environment~hasindex("MANYITEMMIXIN")` and `.local~hasindex("MANYITEMMIXIN")` are both `0` and
`.ManyItemMixin` renders as its own dotted text. So the choice is unobservable today; the slot
already carries the answer, and taking the receiver instead would be a second model of the same
fact.

## Probed past the row: every route to a method object

The brief's hazard is that turning a loud refusal into an answer is where a silent wrong answer at
rc 0 gets introduced. Every route this phase can produce a method object by was run against the
oracle and both engines, three descriptors, from a fresh empty directory with absolute paths. All
byte-identical.

| route | oracle |
| --- | --- |
| unattached `::METHOD z` through `.methods~z~scope` | `The NIL object` |
| the same, `~scope~class~id` | `Object` |
| `~define` with that object, read off the object | `K2` -- filled in place |
| `~define` with that object, read through `~method` | `K2` |
| `~define` with an object that already carries a scope | `K3` -- a copy |
| the donor of that copy, afterwards | `BASE` -- untouched |
| `~defineMethods` | `K4` |
| the object `~defineMethods` was handed, afterwards | `K2` -- it copied |
| `::METHOD` on a class, and the overriding `::METHOD` on its subclass | `BASE`, `SUB` |
| `::ATTRIBUTE`'s getter and setter, `::CONSTANT`, `::METHOD ... ABSTRACT` | `BASE` |
| `.Array~method("APPEND")` | `Array` |
| `.Directory~method("AT")`, donated out of `StringTable` | `Directory` |
| `.Relation`/`.Bag`/`.Set`/`.Supplier`, built by `~inheritInstanceMethods` | their own ids |
| a `::CLASS ... MIXINCLASS`'s own method, and its inheritor's own | `MX`, `USER` |
| `~scope` on a class object's own dictionary entry, rendered | `The BASE class` |
| `~define` a second class from the same object | the object stays `K2`, `.k5~method('W')` is `K5` |
| a handle held across `~delete` and a redefinition of the same name | `K2` both times |
| `~define` with the method omitted, then `~method` | `The NIL object` |

And the refusals, also byte-identical on both engines:

| route | oracle |
| --- | --- |
| `~scope(1)` | 93.902 `Too many arguments in invocation of method; 0 expected.`, rc 163 |
| `.routines~r~scope` | 97.1 `Object "a Routine" does not understand message "SCOPE"`, rc 159 |
| `.base~package~scope` | 97.1 `a Package`, rc 159 |
| `.nil~scope` | 97.1 `The NIL object`, rc 159 |

`Scope` is bound at `Method` alone -- `memory/Setup.cpp:1113`, inside
`StartClassDefinition(Method)` at `:1089`; `Routine`'s block at `:1126` has no such row -- and the field is
`MethodClass`'s (`classes/MethodClass.hpp:168`), not `BaseExecutable`'s, which is why `Routine`
and `Package` raise rather than answer `.nil`.

**A route can produce a method object with no scope**, and the answer is `.nil`. That is
`resultOrNil(getScope())` (`classes/MethodClass.cpp:361`), and the crate matches it.

## The corpus program, and why it is not redundant with the row

`corpus/lang/method_scope.rex` (new, registered in `corpus/phase-5a.txt` and
`coverage.rs`'s `EXPECTED_SUBSET_5A`) carries the rows above that the gate row cannot: the `.nil`
scope, the in-place-versus-copy split of the two `~define` shapes, `~defineMethods` copying, the
accessor and constant names, the library and donated classes, and the argument refusal with its
frame line. Oracle **rc 163**, both engines byte-identical.

It allocates, so it is correctly absent from `collect_stress.rs`'s `NO_ALLOCATION_PROGRAMS` --
that list is asserted in both directions and the test passes with the program in the subset and
out of the list.

## Controls, run and inverted live

**Control 1, the one the brief names.** In `native_method`, answer the ancestor that also defines
the name rather than the slot's scope:

```rust
let scope = match interp.classes().instance_super_scope(class, scope) {
    Some(above) if interp.classes().own_instance_slot(above, &text).is_some() => above,
    _ => scope,
};
```

Rebuilt, run, and it behaves exactly as the brief predicted: `base BASE` stays right and
`sub BASE` is wrong.

```
=== ir rc=159 (rc:SAME stdout:DIFF stderr:SAME)
base BASE
sub BASE
```

Gate table C reports the row as `diverge-stdout` under it (it is not an exit status yet, because
5a is not in `CLOSED_PHASES` until Task 6). `REXX_CORPUS_GATE=1 memcap 8G cargo test --release
-p rexx-exec --test corpus --no-fail-fast` exits **101** at `258 of 259 matching`, the one
mismatch `lang/method_scope.rex`, whose `library` row reads `OrderedCollection` instead of
`Array` -- `.Array`'s ancestor defines `APPEND` too. So the mutation is caught twice over, at two
different rows.

**Control 2, which asks whether the new program adds coverage rather than merely being able to
fail.** In `Interp::method_scope`, answer the receiver's own class where there is no scope:
`native.scope().unwrap_or(native.class())`, so an unattached method answers `The Method class`
instead of `The NIL object` -- a silent wrong answer at rc 0, the exact defect class the brief
warns about. Under it the xscope row still reads **`agree`**, and the corpus gate exits **101**
with `lang/method_scope.rex` the only mismatch. The gate row cannot see this; the new program is
the only thing that can.

**Restored from the scratchpad copies, not from git.** `cp` back, then `sha256sum` matched
(`320c21436ebe043fffa8aa27c2d5e98d78ebc39b76d3e5deaf2762cdfd829767` for `dispatch.rs`), then
`git status --porcelain` printed nothing and `git diff --stat` printed nothing: the tree was
byte-identical to `HEAD` before the gates were run. `rexx-run` was rebuilt after each restore and
both rows re-measured green, so no gate below ran against a mutated binary.

## A false comment my change finished falsifying, deleted

`corpus/lang/class_method_own_dictionary.rex` closed with a block headed "WHAT A METHOD OBJECT
DOES NOT DO", listing four sends as `rc 120` refusals here:

```
     .Array~method('APPEND')~class        oracle `The Method class`, rc 0
     .Array~method('APPEND')~class~id     oracle `Method`, rc 0
     .Array~method('APPEND')~isA(.Method) oracle `1`, rc 0
     .Array~method('APPEND')~scope        oracle `The Array class`, rc 0
```

All four answer. Three did before this task; `~scope` is mine. The block's second half was false
independently of me -- it said `native_method` "mints a fresh object per send", and measured,
`.Array~method('APPEND')~objectName = 'x'` then reading the object back prints `x` on the oracle
and on both engines, and `~identityHash` compares `==` equal. `Interp::method_object` has been
caching one object per dictionary entry since it was written, and its own doc says so.

Deleted rather than restated, per `rust/CLAUDE.md`'s preference for deleting over rewriting: what
remained true of it was `==` on a method object still being rc 120, which is a phase-boundary fact
about a mutable in-repo aggregate and is the shape the same file's rules say not to write down.
`crates/rexx-parse/tests/sourceline_oracle/class_method_own_dictionary.txt` regenerated with the
driver `sourceline_oracle.rs`'s module comment documents, `count 112` -> `count 82`, and the
regenerated body was `cmp`-checked against the source file before installing.

## Two things I found and did not change

* **`gate_table_c.rs`'s xscope `control` field** says "answer `~method` from the flattened
  all-scopes dictionary, so an inherited name answers where the oracle raises -- Task 9". That is
  still a true control for the row and I did not touch it, but "Task 9" names a task of the
  superseded plan, and the same stale form is on other rows (`usingcl` says "Task 7"). Cheap to
  sweep once, and not mine to sweep alone.
* **`corpus/lang/class_method_own_dictionary.rex`'s opening comment** says there is no table C row
  for the scope question, "so this program and `class_method_class_side_raises.rex` beside it are
  the whole of the protection", and `gate_table_c.rs`'s "What this table cannot see" bullet says
  the same. The `xscope` concept row's third line does exercise `~method`'s own-dictionary rule,
  so both sentences are arguable -- but they were arguable before this task in exactly the same
  way (the row and its probe existed at my base; only its verdict moved), and the moment they
  become load-bearing is Task 6's flip, not mine. Rewriting them is where a false statement gets
  introduced, so I left them for the consolidated review with this note.

## Files

* `rust/crates/rexx-exec/src/dispatch.rs` -- the `("Method", "SCOPE", Arity::Fixed(0),
  native_scope)` row, `native_scope`, and the slot's scope threaded through `native_method`.
* `rust/crates/rexx-exec/src/environment.rs` -- `Interp::method_scope`, the reader; and
  `Interp::method_object` gains the `scope` parameter and writes it.
* `rust/corpus/lang/method_scope.rex` -- new.
* `rust/corpus/lang/class_method_own_dictionary.rex` -- the false block deleted.
* `rust/corpus/phase-5a.txt`, `rust/crates/rexx-exec/tests/coverage.rs` -- registration.
* `rust/crates/rexx-parse/tests/sourceline_oracle/method_scope.txt` (new) and
  `.../class_method_own_dictionary.txt` (regenerated).
* `docs/superpowers/plans/2026-08-26-phase-5a-gate-close.md` -- Task 2's Build paragraph corrected.

## Gates

All five run at `637d0dd64`, from `rust/`, each status read unpiped out of its own file.

| | command | exit |
| --- | --- | --- |
| G1 | `cargo fmt --all --check` | **0** |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| G3 | `cargo test --release --workspace --no-fail-fast` | **0** |
| G4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** |
| G5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |

`command -v memcap` answers `/home/moritz/.local/bin/memcap`, so the substitute `ulimit` form was
not needed. `memcap` on G3 or G4 is **wrong** and I hit it: the `--release` build itself peaks over
the cap and `memcap 8G cargo test --release --workspace` dies at exit 137, `peak 8.0G`, during
`Compiling rexx-exec`. The cap belongs on the debug gate, which is where `rust/CLAUDE.md` puts it.

Figures, each beside the command that printed it:

* `/bin/grep -ac 'test result: ok' <log>` -> **102** for each of G3, G4 and G5, and
  `/bin/grep -ac 'FAILED' <log>` -> **0** for each.
* `259 of 259 matching` -- from G4's and G5's own logs, the corpus harness's line. Task 1 left it
  at 258; `lang/method_scope.rex` is the 259th.
* `agree loud=no 5a xscope Scope ... xscope.rex` -- from G4's and G5's logs, gate table C's report.

The two figures I did **not** read from these five, and where they came from instead:

* `258 of 259 matching` under each control mutation, and gate table C's `diverge-stdout` under
  control 1, came from `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test
  corpus --test gate_table_c --no-fail-fast`, exit **101**, run twice against a deliberately
  mutated tree and never against the committed one.
* An earlier G3 attempt at `aa96cf05d` was **killed by me** mid-run at `G3 exit 143` so the plan
  correction could land in the same gate sweep. It is not a reading of anything and is not counted
  above; every row in the table is from the run at `637d0dd64`.

**Both rows re-measured last, against a forced rebuild**, because a stale `target/release/rexx-run`
outlives a revert and reads exactly like a fresh one: `touch crates/rexx-exec/src/environment.rs`
then `cargo build --release --bin rexx-run` recompiled `rexx-exec` and rewrote the binary, and
`xscope.rex` at rc 159 and `method_scope.rex` at rc 163 are byte-identical to the oracle on both
engines with it. `git status --short` prints nothing at `637d0dd64`.
