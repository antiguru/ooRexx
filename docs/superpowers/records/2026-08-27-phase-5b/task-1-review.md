# Task 1 review: `~new`, `INIT`, and the instance seam

Range `6fc65c487..c245dc418` (`f7c38531c` + Moritz's comment-only `c245dc418`), branch
`plan/rust-rewrite`. Reviewed against `docs/superpowers/specs/2026-08-27-phase-5b-instances.md`,
the plan's Task 1 section, `.superpowers/sdd/2026-08-27-phase-5b/task-1-brief.md`, the 5b and 5a
global constraints, and `rust/CLAUDE.md`.

**The worktree was read-only to me.** A debug corpus gate was running in it. Nothing under
`/home/moritz/dev/repos/ooRexx-rust-rewrite/` was edited, staged or committed; this file is the only
one written, and every cargo run used a `CARGO_TARGET_DIR` under my own scratch space. The brief
expected the two controls to be unre-runnable for that reason; they were not, because
`git archive c245dc418 | tar -x` into scratch gives a mutable copy that touches nothing here. All
four of the report's controls were re-run there.

Probes were run from a fresh empty scratch directory with absolute paths, `timeout -s KILL 20`,
three descriptors read separately, never `2>&1`, on `REXX_ENGINE=ir` **and**
`REXX_ENGINE=tree-walker`, against
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
/home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`. Nothing in `rust/corpus/oracle-crashes.txt`
was run.

## Verdicts

**Spec compliance: does not contradict any decision, but see the D58 note.**
**Quality: Changes requested.**

Two defects this commit introduces, neither seen by any gate:

* a named instance is used as the operand's value in arithmetic, non-strict comparison, the
  logical operators and the prefix operators, where the oracle raises 97.1 (Critical);
* a debug build aborts on a legal program once `~new` has built enough instances of a class that
  defines `UNINIT` to drive one collection (Critical).

Everything the brief's "Done when" list asks for is otherwise met, and I re-measured all five
items, the two controls included.

**Counts: 2 Critical, 4 Important, 3 Minor.**

---

## Critical

### C1. `~objectName=` makes an instance an operand: silent wrong answers in arithmetic, comparison, logical and prefix operators

`rust/crates/rexx-exec/src/value.rs:1160`-`:1163` (the `Body::Instance { name: Some(bytes), .. }`
arm of `Interp::heap_to_number`) and `rust/crates/rexx-exec/src/eval.rs:1593` (`logical_values_body`)
and `:1184` (`arith_left_operand`) and `:1486` (`compare_values`).

The oracle sends an operator to its left operand as a **message**, so an instance is `97.1 ... does
not understand message "+"` however it renders. `Interp::operator_operand_gap`
(`eval.rs:1659`) knows that and its arm is correct -- but **every caller reaches it only on a path
that has already failed**: `arith_left_operand` asks after `to_number` answers `NotNumeric`,
`compare_values` asks only `if left_number.is_err()`, and `logical_values_body` and `PrefixOp::Not`
ask only when `logical_value` answers `None`. The new `heap_to_number` arm and the new
`try_text`/`to_text` arms make a **named** instance succeed on all three of those paths, so the gap
is never consulted and the operator answers.

Measured, probes run from a fresh empty directory, three descriptors separately, both engines:

```
$ cat p06_named_arith.rex
o = .K~new
o~objectName = '123'
say 'a' datatype(o)
say 'b' o + 1
::CLASS K
```
oracle rc 159, stdout `a NUM`, stderr `Error 97.1:  Object "123" does not understand message "+".`
crate rc 0 on `REXX_ENGINE=ir` **and** `REXX_ENGINE=tree-walker`, stdout `a NUM` / `b 124`, stderr empty.

```
$ cat p08_named_compare.rex     (o named '123')
say 'a' (o = 123)
```
oracle rc 0, stdout `a 0`; crate stdout `a 1`, both engines. **rc 0 on both sides** -- nothing loud
anywhere, which is this phase's worst defect class.

```
$ cat p14_logical.rex           (o named '1')
say 'a' (o & 1)
```
oracle rc 159, `97.1 ... message "&"`; crate rc 0, `a 1`, both engines.

```
$ cat p36_prefix.rex            (o named '1')
say 'a' (\o)
say 'b' (-o)
```
oracle rc 159, `97.1 ... message "\"`; crate rc 0, `a 0` / `b -1`, both engines.

Deterministic: `p06` and `p08` repeated three runs each on both engines give the identical rc and
stdout every time.

The strict comparison forms are unaffected -- `(o == '123')` with `o` named `'123'` is still the
loud rc 120 refusal, because `compare_values` does not parse the left operand as a number under
`strict`, so the gap check still runs. The silent surface is non-strict comparison, arithmetic,
the logical operators and the prefix operators.

An **unnamed** instance is correct on every one of these -- it refuses loudly at rc 120
(`p07_unnamed_arith.rex`, `p09_unnamed_compare.rex`, `p10_unnamed_streq.rex`,
`p17_named_nonnum_cmp.rex` with a non-numeric name) -- so the whole defect is introduced by giving
an instance a name -- which `instance_naming.rex` does, and then uses only in `~string`,
`~objectName` and `~defaultName` sends.

**The tree already asserts the invariant this breaks.** `eval.rs:1486`'s `debug_assert!` says "a left
operand that parsed as a number reported an operator gap", and its own comment argues the skip is
safe because "Every shape this gap names ... is a shape `Interp::to_number` answers `NotNumeric`
for". The commit adds a shape for which that is false, and the assertion fires:

```
$ REXX_ENGINE=ir <debug>/rexx-run p08_named_compare.rex
rc=101
thread 'rexx-interp' panicked at crates/rexx-exec/src/eval.rs:1486:9:
a left operand that parsed as a number reported an operator gap
```
(both engines; `cargo build --locked -p rexx-exec --bin rexx-run`, debug profile.)

No gate sees any of this. The evidence is not a grep over the corpus but the gated run itself:
`REXX_CORPUS_GATE=1 cargo test --release --locked --workspace --no-fail-fast` prints
`270 of 270 matching` at this same commit, on both engines, so no committed program puts a named
instance in an operator. (I did not run the `memcap 8G` debug gate; the report claims it green, and
C2 below is the reason that claim will not survive the first instance-`UNINIT` corpus program.)

**The `heap_to_number` arm's own stated reason does not hold.** Its comment says it is there so that
"`datatype(o)` is `CHAR` for an untouched instance and `NUM` after `o~objectName = '123'`". I deleted
the arm in a scratch copy of the tree (`Body::Instance { .. } => Err(NotNumeric)`) and rebuilt:
`datatype(o)` still answers `NUM`, because the required-string protocol converts the instance before
`DATATYPE` sees it, while `o + 1` and `(o = 123)` become the licensed loud refusal at rc 120.

```
$ <scratch-copy>/rexx-run p06_named_arith.rex   # arm deleted
rc=120   stdout: a NUM
stderr:  rexx-exec: the operator `+` applied to an instance of a user class is not implemented (Phase 5)
```

**The fix was tried and costs nothing.** With that arm deleted in the scratch copy,
`REXX_CORPUS_GATE=1 cargo test --release --locked -p rexx-exec --no-fail-fast` prints
`270 of 270 matching` and every test binary passes; the only failures in the first attempt were
eight `cannot read .../ootest/...` panics from `rexx-extract/src/lib.rs:578`, because `git archive`
cannot carry a git-ignored working copy, and with `ootest/` symlinked in the two binaries concerned
read `5 passed; 0 failed` each at exit 0. So nothing in the suite depends on an instance's name
parsing as a number.

**Fix.** Delete the `Body::Instance { name: Some(bytes), .. }` arm of `heap_to_number` -- it buys
nothing and is what breaks the assertion's premise. That closes the arithmetic and comparison
surfaces. The logical and prefix-`\` surfaces are separate and need
`Interp::logical_values_body` and `PrefixOp::Not` to consult `operator_operand_gap` **before**
`logical_value`, at least for an instance receiver, since a name of `0` or `1` is a valid truth
value and no numeric parse is involved. Then correct the arm comment at `eval.rs:1654`-`:1658`,
which today describes behaviour the code does not have ("and the same after
`o~objectName = '123'`"), and the report's "Operators" paragraph, which states the same.

### C2. A debug build aborts once a class defines `UNINIT`

`rust/crates/rexx-exec/src/dispatch.rs:3962` (`interp.heap.set_uninit(object)`) against
`rust/crates/rexx-exec/src/lib.rs:6447`-`:6450` (the `debug_assert!` in `Interp::collect_now`).

`native_new` now sets `Object::has_uninit`, which is step three of `completeNewObject` and is what
the brief asks for. The commit updates the comment beside `collect_now`'s assertion from "Nothing in
this crate sets `Object::has_uninit` ... so the list is empty" to "`native_new` sets
`Object::has_uninit` ... and this is the site that owes the delivery" -- and leaves the assertion in
place. It is now reachable from an ordinary program:

```
$ cat p11_uninit.rex
n = 0
do i = 1 to 200000
  o = .K~new
  n = n + 1
end
say 'made' n
::CLASS K
::METHOD uninit
  nop
```
```
$ REXX_ENGINE=ir  <debug>/rexx-run p11_uninit.rex   -> rc=101
$ REXX_ENGINE=tree-walker <debug>/rexx-run p11_uninit.rex -> rc=101
thread 'rexx-interp' panicked at crates/rexx-exec/src/lib.rs:6447:9:
an object was resurrected for UNINIT and nothing here runs a finalizer
```
Release is rc 0 with `made 200000`, and so is the oracle
(`( ulimit -v 1048576; LD_LIBRARY_PATH=... rexx p11_uninit.rex )`, rc 0, `made 200000`), so the
differential is clean and only the debug build dies.

The fifth gate (`REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast`) is a **debug**
run and is green only because no committed program defines `UNINIT` on a class it then instantiates.
Task 5 owns delivery, but a `debug_assert` that an ordinary program trips -- all it takes is a class
with a `::METHOD uninit` and enough instances to drive one collection -- is this commit's to resolve: either deliver nothing and downgrade the assertion to a documented no-op until Task 5, or
do not set the flag until Task 5 sets it and reads it together. Leaving it as it stands means the
first 5b task that adds an instance-`UNINIT` corpus program reddens the debug gate for a reason that
has nothing to do with that task.

## Important

### I1. The error-message substitution divergence is real, and the plan records only the trace one

The commit's plan edit adds "**Two divergences are already found and belong on this task's list**"
to Task 9 -- `TRACE`'s value lines and `USE STRICT ARG`. The report names a third, this crate's own
error-message substitutions, and says "no probe was constructed". One exists and it diverges:

```
$ cat p20_err_subst.rex
o = .DN~new
say 'a'
o~zzz
::CLASS DN
::METHOD defaultName
  return 'overridden'
```
oracle rc 159, stderr `Error 97.1:  Object "overridden" does not understand message "ZZZ".`
crate rc 159 on both engines, stderr `Error 97.1:  Object "a DN" does not understand message "ZZZ".`
The same program without the override agrees byte for byte (`p21_err_subst_plain.rex`).

It is loud rather than silent, so it is a smaller problem than C1, but it is a measured divergence
that the plan's own list is short by. Add it beside the trace one in Task 9's paragraph.

### I2. `objcla` moved from a loud refusal to a silent wrong answer

`5b: 6 rows, 4 not yet agree` under
`REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast`
(exit 101), and the row reads

```
  diverge-stdout loud=no  5b   objcla Object Classes
      oracle rc=0    out="before 0\nafter 0\nfresh 1\n" err=""
      crate  rc=0    out="before 0\nafter 1\nfresh 1\n" err=""
```

This is exactly what D58 predicts for a live class-graph walk, it is recorded in the report and
written into the plan's Task 2, and Task 1 could not have avoided it without the crate-layering move
Task 2 owns. Recorded here rather than raised as a defect, but it is worth saying plainly that the
tree currently ships a silent wrong answer on a gate row and that Task 2 is the only thing standing
between it and phase close.

### I3. Two rooting sentences are wider than what holds, and the `INIT` case is the counterexample

`docs/superpowers/plans/2026-08-27-phase-5b.md`, in the paragraph this commit adds at Task 1:
"`Interp::message_term` takes `push_temp(receiver)` before it resolves anything and **both engines
enter a method body through that one function**". And `rust/crates/rexx-exec/src/run.rs:3073`-`:3077`
(`Interp::pool_owner`'s new doc): "The `SELF` slot is not that root ... and the temporary
[`Interp::message_term`] takes over the sending clause **is**."

`message_term` is entered from `eval.rs:646` (tree-walker) and `run.rs:2752`/`:2769` (ir), so both
sentences hold for a **program's** message term. Neither holds for method-body entry in general.
Outside tests, `Interp::send_message` is also called from `required_string_dispatch`
(`dispatch.rs:2773`, `STRING`), `send_make_string` (`:3014`), `native_new` (`:3965`, `INIT`),
`class_factory` (`:4012`, `INIT`), `native_string` (`:4854`, `OBJECTNAME`), `native_object_name`
(`:4900`, `DEFAULTNAME`), `native_request` (`:5032`) and `lib.rs:5223`; the `STRING`, `INIT`,
`OBJECTNAME` and `DEFAULTNAME` ones are added by this commit.

**The commit's own witness is the counterexample to `pool_owner`'s sentence.** The `INIT`-side
`SELF` clobber in `instance_self_reassigned.rex` reaches `pool_owner` with the new object as the
receiver, and there `message_term`'s temporary holds the *class*, not the object -- the root is
`native_new`'s own `push_temp`, which is exactly what control 4 shows. So the doc names the wrong
root for the one case the task built a witness for. Say "the temporary the sending clause takes over
the receiver, or `native_new`'s own during `INIT`", and narrow the plan's sentence to "a program's
message term".

Pattern searched: `/bin/grep -rn "\.send_message(" crates/rexx-exec/src/` -- 14 hits, 6 of them
under `dispatch.rs:5255`'s `#[cfg(test)]`.

### I4. The ordinary-send half of the rooting claim has no witness

The plan's new paragraph and `pool_owner`'s new doc both rest on `message_term`'s
`push_temp(receiver)`. In a scratch copy of the tree at `c245dc418` I removed that one line
(`dispatch.rs:2473`) and rebuilt:

```
$ cargo test --release --locked -p rexx-exec --test collect_stress --no-fail-fast
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

So the claim the plan now asserts is unwitnessed under the harshest collector setting this project
has, and the `go`-side `SELF` clobber in `instance_self_reassigned.rex` -- the ordinary-send half of
the witness program -- contributes nothing to the rooting question (it still pins stdout). The report
says the same thing about the third root it deleted and applied the rule there; the same rule points
at this line. Either add a case that reddens when it goes, or say in the plan that the ordinary-send
root is argued rather than witnessed.

## Minor

### M1. `text_len_inner` gained a `Redirect` arm and not a body arm

`rust/crates/rexx-exec/src/value.rs:602`-`:607` adds `Redirect::InstanceDefault`, but the body match
at `:611`-`:628` still ends in `unreachable!("the value model only creates Text, Num, Stem, Array and
Native, got {other:?}")`. `Redirect::of` (`:1417`-`:1419`) answers `InstanceDefault` only for
`name: None`, so a **named** instance reaching `text_len_inner` falls to that `unreachable!`. Its
three sibling functions (`to_text`, `try_text`, `heap_to_number`) all got a `name: Some` arm; this
one did not.

I could not construct a program that reaches it: `length(o)` and `length(a.)` with a named instance
both agree with the oracle (`p31`, `p33`: `a 3` and `a 10` / `abcdefghij`), because `reqstr_armed` --
which `native_new` now sets unconditionally -- converts the instance through the `STRING` send before
`LENGTH` sees it. So this is latent rather than live. Add the arm anyway, or the next change to the
protocol's arming turns it into a panic.

### M2. An `objectName` or `string` override that answers an object: oracle exhausts, crate answers

```
$ cat p19_objname_self.rex
o = .K~new
say 'a' o~objectName
say 'b' o
::CLASS K
::METHOD objectName
  return self
```
oracle rc 251, `Error 5 ... System resources exhausted.`; crate rc 0 on both engines, `a a K` /
`b a K`. Same with `::METHOD objectName return .MS~new` where `MS` has a `makeString`
(`p22`, oracle rc 251, crate rc 0 `a a MS`).

`native_string` sends `OBJECTNAME` and then renders the answer with the infallible
`Interp::string_value_text` instead of re-entering the protocol, so it stops after one hop where the
oracle recurses until the stack goes. A deliberately pathological shape, and the crate's answer is
the friendlier one; recorded because the divergence is at rc 0 against rc 251 and is not written down
anywhere.

### M3. The commit message understates what is left on the wrong side of the rendering line

"`Interp::to_text` derives the article-and-id rendering for an unnamed instance, which leaves only
this crate's own renderings on the wrong side of that line." `TRACE`'s value lines are on the wrong
side too, and they are the oracle's rendering shown to the user, not this crate's own. The commit's
own code comment at `value.rs:803`-`:810` says it correctly ("what is left is every rendering
reached through an infallible function, which is `TRACE`'s value lines and this crate's own
error-message substitutions"); the commit message is the loose one.

---

## The brief's "Done when", re-measured

Every cargo command below ran with
`CARGO_TARGET_DIR=<scratch>/review-target` (or `<scratch>/mut-target` for the mutated copy) and
`--locked`, from `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust`.

### 1. Both rows agree on both engines under the phase-gate command -- MET, re-measured

```
$ REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release --locked -p rexx-exec \
      --test gate_table_c --test gate_table_d --no-fail-fast
exit 101
  agree          loud=no  5b   abscla Abstract Classes   ... abscla.rex
  agree          loud=no  5b   creo Initialization       ... creo.rex
  5b: 6 rows, 4 not yet `agree`            (table C)
  5b: 2 rows, 0 not yet `agree`            (table D)
```
The 101 is the four rows Tasks 2, 3, 5 and 8 own -- the panic names
`["gate-tables/concepts/objcla.rex", "gate-tables/concepts/usesem.rex",
"gate-tables/concepts/obdes.rex", "gate-tables/concepts/methodsbyclass.rex"]`. "Both engines" is
carried by the harness itself: `gate_tables/mod.rs:195`'s `run_on_both_engines` runs each probe under
`Engine::TreeWalker` and `Engine::Ir` in process and fails structurally if they disagree.

### 2. The `SELF`-clobbering program -- MET, re-measured on all three parts

* **Three descriptors, both engines.** `corpus/lang/instance_self_reassigned.rex` against the
  oracle: oracle rc 0, `a kept-yzyz...yz 192 clobbered`, empty stderr; `REXX_ENGINE=ir` and
  `REXX_ENGINE=tree-walker` identical on stdout, stderr and exit status.
* **In `corpus/phase-5b.txt`**: present, sixth entry, and `EXPECTED_SUBSET_5B`
  (`crates/rexx-exec/tests/coverage.rs:1228`) lists the same six in the same order.
* **Targeted `run_program_collect_every_alloc` case.**
  `cargo test --release --locked -p rexx-exec --test collect_stress --no-fail-fast` -> exit 0,
  `8 passed; 0 failed`, including `a_method_that_assigns_over_self_keeps_its_exposed_variables`.

### 3. The instance-naming witnesses -- MET, re-measured

All three agree byte for byte on stdout, stderr and exit status on **both** engines:

| program | oracle |
|---|---|
| `corpus/lang/instance_naming.rex` | rc 0, `object an Object` ... `renamed zed zed a K` ... `article an EGG` |
| `corpus/lang/instance_naming_overrides.rex` | rc 0, `dn overridden / overridden / overridden / overridden` ... `ms a MS / a MS / from-makestring / from-makestring` |
| `corpus/lang/instance_naming_raises.rex` | rc 214, stdout `a`, stderr the `OBJECTNAME`/`STRING` `Compiled method` frames above `Error 42.3` |

The brief's stated oracle figures for the instance receiver all reproduce: `an Object`; `a K` for
`~string`/`~defaultName`/`~objectName`; after `~objectName = 'zed'`, `zed zed a K`.

### 4 and 5. The two controls -- MET, and I re-ran both rather than judging the transcripts

The report carries a transcript for each, and both transcripts are consistent with a real run (they
carry the row's three descriptors on both sides in the harness's own format, and the neighbour row's
verdict). I did not have to take that on trust: I extracted `c245dc418` into a scratch copy with
`git archive` (which writes nothing to the worktree), reproduced the baseline there, and applied each
mutation to the copy.

Baseline in the copy: `5b: 6 rows, 4 not yet agree`, `abscla` and `creo` both `agree`, exit 101 --
identical to the worktree.

**Control 1, `checkAbstract` deleted from `native_new`:** exit 101, `5b: 6 rows, 5 not yet agree`.
```
  diverge-both   loud=no  5b   abscla Abstract Classes
      oracle rc=158  out="declared AB Class\n" err="       *-* Compiled method \"NEW\" ..."
      crate  rc=0    out="declared AB Class\ninstance an AB\n" err=""
  agree          loud=no  5b   creo Initialization
```
The abstract class constructs; `creo` untouched. Matches the report exactly.

**Control 2, the `INIT` send deleted from `native_new`:** exit 101, `5b: 6 rows, 5 not yet agree`.
```
  diverge-stdout loud=no  5b   creo Initialization
      oracle rc=0    out="type a savings account\nbalance 1000.00\nrate 6.25\n" err=""
      crate  rc=0    out="type a savings account\nbalance BALANCE\nrate INTEREST_RATE\n" err=""
  agree          loud=no  5b   abscla Abstract Classes
```
`creo` reddens silently -- rc 0, empty stderr; `abscla` untouched. Matches the report exactly.

I also reproduced the report's controls 3 and 4, which are the rooting ones:

**Control 3, `Body::trace`'s `Instance` arm stops tracing the pools:**
`cargo test --release --locked -p rexx-exec --test collect_stress --no-fail-fast` ->
`5 passed; 3 failed`, all three on `a live value`
(`a_method_that_assigns_over_self_keeps_its_exposed_variables`,
`a_parked_reply_keeps_its_variables_across_a_collection`,
`the_l0_subset_passes_again_under_collect_on_every_allocation`).

**Control 4, `native_new`'s `push_temp` removed:** `6 passed; 2 failed`, both panicking at
`crates/rexx-exec/src/lib.rs:6099:14`, `an exposed variable's owner is a rooted Body::Instance`
(the new case and the subset run).

So the rooting fix is real for the window it claims: the `INIT`-side clobber is what control 4
turns on, and it is the only root the new object has while `INIT` runs. See I4 for the half that is
not witnessed.

## The gates, re-measured

| command | status | reported |
|---|---|---|
| `cargo fmt --all --check` | 0 | 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` (cold target dir) | 0 | 0 |
| `REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release --locked -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | 101 | 101 |
| `REXX_CORPUS_GATE=1 cargo test --release --locked --workspace --no-fail-fast` | 0 | 0 |
| `cargo test --release --locked -p rexx-exec --test collect_stress --no-fail-fast` | 0 | -- |

The workspace run finished with **exit 0**, no `FAILED` line anywhere, 102 `test result: ok` lines,
and `270 of 270 matching` printed by the corpus differential -- so the report's corpus figure and
its third and fourth gate lines are re-measured. I added `--locked` and `--no-fail-fast`, which only
make the command stricter than the one the report quotes.

## Other checks

* **Rustdoc intra-doc links.** Every link the diff adds resolves: `ClassDef::is_abstract`,
  `ClassGraph::make_abstract`, `ClassGraph::is_abstract`, `crate::CLASS_SLOT_BASE`
  (`rexx-core/src/handle.rs:88`), `ObjectModel::build` (`dispatch.rs:611`), `NATIVE_METHODS`,
  `native_object_name`, `Interp::message_term` (`dispatch.rs:2459`), `Interp::pool_owner`
  (`run.rs:3080`), `rexx_core::Body::Instance`, `Interp::instance_name` (`value.rs:857`).
  Checked with `/bin/grep -rn` per item over `crates/rexx-{core,exec,classes}/src`.
* **ASCII, em-dashes, historical framing, set cardinality.** No non-ASCII byte and no em-dash in any
  added line (`/bin/grep -n "^+" <diff> | LC_ALL=C /bin/grep -P "[^\x00-\x7F]"` -> nothing;
  `/bin/grep -c "—" <diff>` -> 0). The only "before this"/"no longer" phrasings in added lines are
  in the plan file, in a runtime message string, and in a comment describing a control mutation --
  none is history in a source comment. `c245dc418` removed the two cardinalities that were there
  ("whose four steps", "the same two mutations").
* **C++ anchors.** `Setup.cpp:514` is `AddClassMethod("New", RexxObject::newRexx, A_COUNT)`;
  `ClassClass.cpp:1741` `checkAbstract`, `:1754` `makeAbstract`, `:1882` `completeNewObject` whose
  first statement is `checkAbstract()`; `ObjectClass.cpp:1157` `stringValue` is
  `sendMessage(GlobalNames::OBJECTNAME)`, `:1695` `objectName`, `:1712` the `DEFAULTNAME` send,
  `:2630` `newRexx` with `ProtectedObject p(newObj)` at `:2637`. All as cited.
* **The parent-commit claims the commit message makes.** At `6fc65c487`,
  `install_class_directive`'s comment did claim "any other class takes the keyword by setting a flag
  whose reader is `~new`" while the code only raised for the metaclass case; and `value.rs` carried
  four `unreachable!("the value model only creates Text, Num, Stem, Array and Native, ...")` sites,
  of which the commit gives an instance arm to three (see M1 for the fourth).
* **The plan's two new Task 9 measurements are accurate.** `objcla` reads `diverge-stdout`, rc 0 on
  both sides, empty stderr, `after 1` against `after 0` (quoted under criterion 1 above). And the
  `USE STRICT ARG` pair reproduces on the exact shape the plan states -- `::METHOD m CLASS` with
  `use strict arg v`, sent as `.K~m`: oracle rc 163, `Error 93 ... Incorrect call to method.` /
  `Error 93.901:  Not enough arguments for method; 1 expected.`; crate rc 216, `Error 40 ...
  Incorrect call to routine.` / `Error 40.3:  Not enough arguments in invocation of M; minimum
  expected is 1.`, both engines, with the two traceback frames above the error identical.
* **The two remaining commit-message claims.** `NATIVE_METHODS` is resolved through
  `lookup_instance_method` (`dispatch.rs:621`) and the new `NATIVE_CLASS_METHODS` through
  `lookup_class_method` (`:643`), so "the existing native table cannot express" a class-side row is
  right. `bench-programs/dispatch.rex` now exits 0 with `5000000` on this crate, which is what makes
  `Role::Blocked` false for it and what
  `every_blocked_axis_still_fails_on_this_crate` reported.
* **No unbounded root growth from `native_new`'s `push_temp`.** 1,000,000 `~new` calls in a loop peak
  at 23,532 kB against 16,996 kB for the same loop assigning a literal
  (`/usr/bin/time -v`, release, `REXX_ENGINE=ir`), so the temporaries frame heals per clause as
  `RootSet::pop_frame`'s doc says.
* **Instance-side shapes that agree** and are worth recording as probed: `~isA` with a non-class
  argument (88 on both), the arity errors for `~isA()`/`~defaultName(1)`/`~string(1)`/`~objectName(1)`
  (88/93/93/93), `~objectName =` an object (88), an `objectName` override
  (`o~objectName`/`o~string`/`say o`/`length(o)` all follow it, `~defaultName` does not),
  `.SUB~new` and `.AB~subclass('KID')~new` on an `ABSTRACT` parent (both construct; the flag is not
  inherited, as `ClassDef::is_abstract`'s comment says), `.Object~new(1)` (rc 163 with the
  `INIT`-above-`NEW` frames), a stem whose default is an instance, `if o` / `do i = 1 to o`, an
  instance as the **right** operand of `&`, `=` and `+`, `parse value o`, `o~request('STRING')`,
  and a `defaultName` returning an object.
* **`Object~defaultName`, built from zero, across receiver kinds.** `'abc'`, `5` and `1.5` answer
  `a String`; `.nil` `an Object`; `.environment` `a Directory`; `.local` `a Directory` while its
  `~objectName` is `The Local Directory`; `.String`, `.Object` and `.Class` answer
  `The String class` / `The Object class` / `The Class class`; `.methods` `a String`; `.context`
  `a RexxContext`; `.rexxinfo` `a RexxInfo`. All byte for byte against the oracle on both engines.
* **`INIT` semantics.** An `INIT` that returns a value is ignored and `~new` answers the object; an
  `INIT` that raises reports its own clause under `Compiled method "NEW" with scope "Object".` at
  rc 214; a subclass `INIT` that omits `self~init:super` leaves the superclass's exposed variable
  uninitialised (`v=V`) and one that calls it reads `base-init`. All agree on both engines, which is
  `native_new`'s doc's "nothing chains them" measured.

## What I could not re-measure

* **The `memcap 8G` debug gate.** I ran `cargo fmt --all --check` (0),
  `cargo clippy --workspace --all-targets --locked -- -D warnings` from a cold target directory (0),
  and `REXX_CORPUS_GATE=1 cargo test --release --locked --workspace --no-fail-fast` (0). The debug
  run was left to the controller's own gate, which was running in the worktree while I worked; C2
  predicts it stays green only until an instance-`UNINIT` corpus program exists.
* **The report's "Probing past the rows" enumeration of newly-agreeing classes**: not re-derived in
  full. Spot-checked: `.Collection~new`, `.Comparable~new` and `.Object~new` agree; `.Alarm~new` is
  the loud wrong-error-family case the report describes.
  (The "270 of 270 matching" corpus figure **is** re-measured -- it is printed by
  `REXX_CORPUS_GATE=1 cargo test --release --locked --workspace --no-fail-fast`, which I ran.)
* **The performance sitting**, which the report says could not be taken because the machine exposes
  one usable hardware counter. I did not re-check the counter availability; the report's framing
  (a probe before and after showing `instructions:u` alone at 100.00% and the pair multiplexing to
  ~50%) is the right shape for that claim and is the project's own rule for separating a transient
  shortage from a machine property.

---

## Spec compliance

**No decision is contradicted.** Read against D57-D69 and D59a and against the spec's "Instance
construction, in the order `completeNewObject` fixes" and "The native `~new` split":

* **D57** -- instances exist, and the mechanism set this task takes from it is built:
  `checkAbstract`, the behaviour, the `UNINIT` registration, the `INIT` send, in that order, at
  `dispatch.rs:3936`-`:3967`. The order is checked observably: `.Object~new(1)` reports
  `Compiled method "INIT" with scope "Object".` above `Compiled method "NEW" with scope "Object".`,
  rc 163, agreeing on both engines.
* **The native `~new` split** -- only `Object~NEW` is built, through the new `NATIVE_CLASS_METHODS`
  table resolved by `lookup_class_method`. `StringTable`, `Array` and the `Message` object are left
  to Tasks 3, 8 and 7. The classes that inherit `Object~NEW` and now construct are the "5b unblocks
  the remainder" half of that section, and the twenty-three that refuse a bare `~new` still refuse:
  spot-checked, `.Alarm~new` is a loud failure on both sides (the error *family* diverges -- oracle
  93.901, crate 40.3 -- which is the 5a `USE STRICT ARG` defect the report found and wrote into
  Task 9, with the traceback frames above it agreeing), and `.Collection~new`, `.Comparable~new` and
  `.Object~new` agree byte for byte on both engines.
* **D64** -- the task's own witnesses are in `corpus/phase-5b.txt` in `phase-5a.txt`'s shape, with
  `EXPECTED_SUBSET_5B` updated in the same commit, and the 5b constraint that a row's probe path
  moves into the phase file in the task that makes the row agree is obeyed for both `creo.rex` and
  `abscla.rex`.
* **D59/D59a/D60/D61** -- untouched. `native_new` sets the instance flag; nothing here delivers or
  orders a finalizer, and no committed program depends on an `UNINIT` ordering. See C2 for the
  consequence of setting the flag while `collect_now`'s assertion still says nothing delivers.

**D58 is not satisfied and the commit says so.** `Body::Instance` carries its class and dispatch
resolves against that class's instance behaviour live, which is the "live walk of the class graph on
every send" D58 names as the implementation that "silently fails the `~define` case at rc 0". That is
exactly what `objcla` now reads. The plan assigns the fix to Task 2, the commit writes the new shape
into Task 2's section, and the layering reason it could not be done here (`rexx-core`'s `Body` cannot
name a `rexx-classes` `BehaviourHandle`) is real. So this is a sequencing consequence rather than a
contradiction -- but it is the one thing in the tree that must not be forgotten, because the phase
cannot close over it and no gate exits non-zero on it until `CLOSED_PHASES` gains `"5b"`.

Where the commit's prose overstates its own reach is I3 and M3.

---

## Where the evidence is

Everything below is outside the worktree, under this review session's scratchpad
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/ae5f3c99-11ad-4b29-9dfe-f3d991a1a398/scratchpad/task1-review/`:

* `probes/p01..p50*.rex` -- the probe programs named in this report, and `diff.sh`, the driver that
  runs each one against the oracle and both engines with three descriptors read separately.
* `out/` -- every probe's six output files.
* `gate-ab.out` / `gate-ab.err` -- the phase-gate run.
* `mutcopy/` -- `git archive c245dc418` extracted (with `ootest/` and `oodocs/` symlinked in, since
  `git archive` cannot carry a git-ignored working copy), used for the four control mutations and
  the `heap_to_number` diagnostic; `mut-base.err`, `mut-c1.err`, `mut-c2.err`, `mut-c3.out`,
  `mut-c4.out`, `mut-c5.out`, `mut-fix.out` and `mut-fix2.out` are their readings.
* `ws.out` / `ws.err` -- the gated release workspace run.
* `review-target/` and `clippy-target/` -- the cargo target directories, kept off the worktree's.

Nothing under `/home/moritz/dev/repos/ooRexx-rust-rewrite/` was written except this file.

## One correction to the report

`.superpowers/sdd/2026-08-27-phase-5b/task-1-report.md`'s "Operators" paragraph says the crate
"refuses loudly" where the oracle raises 97.1, and cites `o~objectName = '123'` in the same sentence.
That is true for an **unnamed** instance and false for the named one it names: measured, `o + 1`
after `o~objectName = '123'` is rc 0 and `124` here against the oracle's rc 159. Same for the arm
comment at `eval.rs:1654`-`:1658`. This is C1.
