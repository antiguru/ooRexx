# Task 4 report: `FORWARD` and `DELEGATE` (D62)

BASE `ccc6dbd13` on `plan/rust-rewrite`. Report opened before any reading of the tree; appended as
the work proceeds.


## What I re-measured before building

Oracle `parse version`: `REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026`. Every probe below ran from a
fresh empty directory under `( cd "$D"; ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib timeout -s
KILL 20 .../build/bin/rexx FILE )`, three descriptors read separately, and the crate side ran on both
`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` from the same kind of directory.

**The brief's central fact reproduces.** `corpus/gate-tables/directives/attribute__delegate__subkeyword.rex`
as committed is `main` at rc 0 with empty stderr on the oracle and on both engines. A probe that
actually sends is rc 0 `attr helper-at` / `meth helper-m` on the oracle and rc 120
`rexx-exec: a ::ATTRIBUTE with no body of its own is not implemented (Phase 5)` on both engines.

**`FORWARD` as an instruction reproduces.** The plan's `forward class (super)` program is oracle rc 0
`a base-cm` against rc 120 `rexx-exec: FORWARD is not implemented (Phase 5)` on both engines.

**Plain `::ATTRIBUTE v` already works**, rc 0 printing `7` on all three sides, so the gap is the
bodiless generated forms and not attributes.

### `keyForward`'s six options, my own measurements

One probe each, a method forwarding out of `m` to a target that is never `m` on the same receiver, so
no probe is of the shape the crash entry names.

| option | probe | oracle |
|---|---|---|
| `TO` | `forward to (t)` where `t` is a `Target` instance with its own `m` | rc 0, `target-m` |
| `CLASS` | `forward class (super)` in `K SUBCLASS Base` | rc 0, `a base-cm` |
| `MESSAGE` | `forward message('OTHER')` | rc 0, `other-m` |
| `ARGUMENTS` | `forward message('OTHER') arguments (.Array~of(1,2))` | rc 0, `other 1 2` |
| `ARRAY` | `forward message('OTHER') array(1,2)` | rc 0, `other 1 2` |
| `CONTINUE` | `forward message('OTHER') continue` then `return 'after' result` | rc 0, `after other-m` |

All six answer rc 0 with empty stderr, and all six are rc 120 `FORWARD is not implemented (Phase 5)`
on both engines today.

**One disagreement with the brief's table, and it is a probe difference rather than a contradiction.**
The brief records `CONTINUE` as rc 165. Mine is rc 0. Measured, the rc 165 shape is a continued
forward with **nothing after it**: `::method m` whose whole body is `forward message('OTHER')
continue` is `91.999 Message "M" did not return a result.` at rc 165, because execution resumes after
the `FORWARD` and then falls off the end of the method. That is the same fact the brief's own gloss
states, read through a probe with no `return`; adding `return 'after' result` makes it rc 0
`after other-m`.

A first explanation of that rc here was wrong and is worth keeping as the correction it needed: it
said reading `RESULT` after a continued send that answered nothing is itself 91.999. It is not.
Measured, `RESULT` is **dropped** in that case and reverts to its uninitialised literal --
`forward message('OTHER') continue` over an `::method other` that ends in a bare `return`, with
`RESULT` assigned `preset` beforehand, leaves `symbol('RESULT')` at `LIT`, and a method reading it
prints `RESULT`. That is `ForwardInstruction.cpp:256`'s `dropLocalVariable`, and it is a
second thing this phase owes a witness.

### The equivalence in the plan is not observably exact, and the plan is wrong to state it as the build

`dire.xml`'s stated equivalence -- `expose delegateName` plus `forward to(delegateName)` -- and the
oracle's own `DELEGATE` differ **on the traceback**. Same failing inner method, same sending clause:

```
::method m delegate d            ::method m ; expose d ; forward to (d)
     5 *-* return 1/0                 5 *-* return 1/0
                                     13 *-* forward to (d)
     2 *-* say 'a' o~m                2 *-* say 'a' o~m
```

Both rc 214, both `Error 42.3`. The written-out form leaves its own `forward` clause on the
traceback; `DELEGATE` leaves nothing, because the C++ implements it as `DelegateCode::run`
(`execution/CPPCode.cpp:605`), a primitive with no Rexx activation, which reads the attribute out of
the receiver's pool at the method's scope and sends the message on. So building `DELEGATE` by running
a synthetic `expose`/`forward` body would ship a differential mismatch on stderr.

`DELEGATE` is therefore built here as a generated method beside the accessor pair -- the same shape
`read_attribute` and `write_attribute` already have, which is also "no activation and no frame" and
is measured as such in their own docs. The plan is corrected where it states the equivalence as the
build instruction.

The unset-variable arm pins the same thing from the other side: `::method m delegate d` with nothing
assigned to `d` is `97.1 Object "D" does not understand message "M"` at rc 159, the delegate variable
rendering as its own name and the message name being the one the send used.

## What was built

**`FORWARD` as an instruction**, in `Interp::exec_forward` (`crates/rexx-exec/src/run.rs`), reached
from `step`'s own dispatch and so from both engines: `Op::Generic` hands a `FORWARD` clause to
`step_in_temps_frame`, which is the same call the tree-walker makes. It is not promoted to an IR op
and is not planned to be.

* Legality first, `Raised::forward_outside_method`, 98.947 at rc 158.
* Options in the C++'s own order -- `TO`, `MESSAGE`, `CLASS`, then `ARGUMENTS` or `ARRAY` -- each with
  its `>K>` line, rendered through `Interp::string_value_text` the way `traceKeywordResult` renders
  through `stringValue()`.
* The three defaults `RexxActivation::forward` supplies: receiver, the name the method was entered
  under, and the arguments it was entered with.
* `CONTINUE` sets `RESULT`, or **drops** it when the send answered nothing, and traces the `>>>`;
  without `CONTINUE` the send's value is the method's, which `Flow::Return` already means.

**`DELEGATE`** as a new `GeneratedKind::Delegate`, beside `Getter`, `Setter`, `Abstract` and
`Constant` -- so both halves of the pair a `::ATTRIBUTE ... DELEGATE` or a `::METHOD ... DELEGATE
ATTRIBUTE` generates are answered out of `Interp::generated_methods` instead of reaching
`method_body_gap`'s refusal. `Interp::send_to_delegate` (`dispatch.rs`) reads the delegate variable in
the declaring scope's pool on the receiver and re-sends under the arriving name with the arriving
arguments, which is `DelegateCode::run` (`execution/CPPCode.cpp:605`). No activation, so no traceback
frame -- see the equivalence measurement above.

## Differential, at the implementation commit, three descriptors read separately on both engines

Every row below is oracle against `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, all three sides
byte-identical unless the row says otherwise.

| probe | shape | oracle |
|---|---|---|
| `fwd_class` | the plan's `forward class (super)` | rc 0 `a base-cm` |
| `opt_to` | `forward to (t)` | rc 0 `target-m` |
| `opt_message` | `forward message('OTHER')` | rc 0 `other-m` |
| `opt_arguments2` | `arguments ((1,2))` | rc 0 `other 1 2` |
| `opt_args_trim` | `arguments ((1,,3,,))` | rc 0 `other 3 1 3`, the trailing-omission trim |
| `opt_array` | `array(1,2)` | rc 0 `other 1 2` |
| `opt_continue` | `continue` then `return 'after' result` | rc 0 `after other-m` |
| `cont_fallthrough` | `continue` and nothing after it | rc 165, 91.999 |
| `cont_noresult` | continued send answering nothing, `RESULT` read | rc 0 `a after RESULT` |
| `cont_dropped` | the same with `RESULT` preset, `symbol('RESULT')` | rc 0 `a dropped` |
| `e_notmethod` | `forward` at program level | rc 158, 98.947 |
| `e_routine` | `forward` in a `::ROUTINE` | rc 158, 98.947 |
| `e_noclass` | `class (5)` | rc 168, 88.914 |
| `e_argsnil` | `arguments (.nil)` | rc 158, 98.946 |
| `e_argsstr` | `arguments ('abc')` | rc 0 `a o abc`, one argument |
| `e_noresult` | non-continuing forward to a body answering nothing | rc 165, 91.999 |
| `tr_in` | `trace i` over `to`/`message`/`array` | rc 0, whole transcript |
| `tr_in3` | `trace i` over `message`/`arguments`/`continue` | rc 0, whole transcript |
| `send_attr` | the brief's own sending probe | rc 0 `attr helper-at` / `meth helper-m` |
| `d_unset` | `::method m delegate d`, `d` unassigned | rc 159, `Object "D" ... message "M"` |
| `d_raise` | delegated-to method divides by zero | rc 214, and **no** frame between the two clauses |
| `d_args` | `o~m(3,4)` through a delegate | rc 0 `a inner 3 4` |
| `d_string` | delegate to a string, `~length` | rc 0 `a 6` |
| `d_case` | `delegate Dd` over `expose dD` | rc 0 `a 6` |
| `d_scope` | the delegate variable read at the declaring scope | rc 0 `a 1` |

The two `trace i` rows are the strongest of these: they compare the whole stderr transcript, including
each option's `>K>` line, the `ARRAY` items' `>A>` lines ahead of their keyword line, and the `>>>`
that only a continued forward writes.

**One probe of mine had to change and the reason is not this task's.** `arguments (.Array~of(7))`
is rc 120 `method "OF" of class "Array" is not implemented (Phase 5)` on both engines against the
oracle's rc 0. `.Array~of` is a native class method this phase has not built; the probe was rewritten
over an array literal (`arguments ((1,2))`) and a parenthesised value (`arguments ((7))`), which
exercise the same two limbs of `requestArray`. Recorded here because it is a loud divergence that
belongs to whoever owns `Array`, not because Task 4 needs it.

## The self-forward shape, run on the crate ALONE

Never handed to the oracle; the entry in `corpus/oracle-crashes.txt` is the oracle's half and was not
re-run. The crate, both engines:

```
o = .K~new ; say o~m ; ::CLASS K ; ::METHOD m ; forward
  ir, tree-walker   rc 245, stdout empty, stderr ending
                    Error 11 ... Control stack full.
                    Error 11.1:  Insufficient control stack space; cannot continue execution.
```

which is the ruling's target and is `forward continue`'s own answer on the oracle. The mechanism is
that the send happens with the forwarding activation still on the stack, so
`Interp::enter_method_body`'s `MAX_ACTIVATION_DEPTH` guard counts the recursion. The oracle stops its
activation before the send, so its guard never sees the frames.

## Which of `keyForward`'s six options this phase owes a witness, and which it does not

The plan's explicit instruction. **All six are built and all six have a witness**, so the phase owes
none of them to a later one. What follows names the witness for each, because "built" and
"witnessed" are different claims and the plan asked for the second.

| option | built | witness |
|---|---|---|
| `TO` | yes | `corpus/lang/forward_options.rex`'s `to` line, and `forward_frame.rex` |
| `CLASS` | yes | `corpus/lang/forward_class_super.rex`, both a class method and an instance method |
| `MESSAGE` | yes | `forward_options.rex`'s `message` line, and every `arguments`/`array` line under it |
| `ARGUMENTS` | yes | `forward_options.rex`'s `arguments` and `trimmed` lines, plus `forward_arguments_not_an_array.rex` for the conversion's two outcomes |
| `ARRAY` | yes | `forward_options.rex`'s `array` and `omitted` lines |
| `CONTINUE` | yes | `corpus/lang/forward_continue.rex`, all four arms |

**The defaults are the seventh thing and they are witnessed too**, which the plan did not ask for and
which the option table alone would hide: each send in `forward_options.rex` leaves exactly one of the
three unwritten, so the line reports the default it took. **The all-defaults shape -- a bare
`FORWARD` -- is deliberately in no program here**, because that is the shape
`corpus/oracle-crashes.txt` names.

**Two arms are owed to nobody but are worth naming as reached rather than built.** `CLASS` accepts
only a class object and reports 88.914 otherwise -- the same raiser a `~name:scope` override already
uses, which is why it gets no corpus row of its own; and a `FORWARD` outside a method is 98.947,
which `forward_outside_method.rex` carries.

## The corpus and the two gate rows

`corpus/phase-5b.txt` gains the same entries `EXPECTED_SUBSET_5B` does, in one commit -- the two
replaced table D probes, three `DELEGATE` witnesses and six `FORWARD` ones, eleven lines on each
side, read off `git diff d8e48d353~1 d8e48d353` rather than counted by hand. Every new
`corpus/lang/*.rex` also gains its `crates/rexx-parse/tests/sourceline_oracle/<name>.txt`, captured
with the driver that module's own comment prints, run from a scratch directory.

**Gate readings, each command quoted beside its figure.**

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_c --test gate_table_d --no-fail-fast
  exit 101, by design while `methodsbyclass` is red
  table D: 5b 2 rows, 0 not yet `agree`      (was 2 rows, 0 -- over probes that asked nothing)
  table C: 5b 6 rows, 1 not yet `agree`      (`methodsbyclass`, Task 8's, unchanged)
  table C: 5a 135 rows 0 not agree, 5c 1347 rows 912 not agree -- both unmoved
  table D: 5a 36 rows 0 not agree, 5c 38 rows 35 not agree -- both unmoved
```

```
REXX_CORPUS_GATE=1 cargo test --release --workspace
  310 of 310 matching
```

Re-run at `acb015bd4` and again at `8e891cd5a`, the phase gate reads the same three times: exit 101,
table D 5b `2 rows, 0 not yet agree`, table C 5b `6 rows, 1 not yet agree`, 5a and 5c unmoved on both
tables; and the corpus is `311 of 311 matching` at the last two.

## The five gates, at both implementation commits

Run at each of the three commits that change the crate, from `rust/`, each status read unpiped from its own
file, none chained with `&&`.

| gate | command | at `d8e48d353` | at `acb015bd4` | at `8e891cd5a` |
|---|---|---|---|---|
| 1 | `cargo fmt --all --check` | 0 | 0 | 0 |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | 0 | 0 |
| 3 | `cargo test --release --workspace` | 0 | 0 | 0 |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace` | 0, `310 of 310 matching` | 0, `311 of 311 matching` | 0, `311 of 311 matching` |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 0, `310 of 310 matching` | 0, `311 of 311 matching` | 0, `311 of 311 matching` |

Gate 5's own arithmetic, every time: 103 `running` sections and 103 `test result: ok` lines,
`test result: FAILED` zero times, so every section reported and nothing ran silently.

**Gate 2 was red once at `acb015bd4` and the fix is in the commit**:
`clippy::needless_pass_by_ref_mut` on `forward_after_reply`, which takes `&self`.

**Intra-doc links.** `RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" cargo doc --no-deps -p
rexx-exec --document-private-items` exits 101 on this tree, with unresolved links at eighteen
`file:line` sites. **None of the eighteen is in code this task wrote** -- they are in
`builtin/datetime.rs`, `builtin.rs`, `environment.rs`, `trace.rs`, and `lib.rs:937`/`:2019`/`:2952`
and `run.rs:5253`/`:5702`/`:8000`/`:11055`, all outside the ranges this commit adds. Each item this
task's own new links name was checked to exist by `/bin/grep` before the link was written.

## Controls: what reddens, and what the old probes do not see

Every arm below is a `git archive d8e48d353 | tar -x` extract with its own `CARGO_TARGET_DIR`; the
live worktree was never mutated. Two commands per arm, each status read unpiped:

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_d --no-fail-fast
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus --no-fail-fast
```

### The two replaced rows, and the thing the task exists for

| arm | what it deletes | table D's two 5b rows | corpus |
|---|---|---|---|
| M1 | `send_to_delegate` does not send: it answers the delegate object | **both `diverge-stdout`**, exit 101 | 305 of 310, exit 101 |
| M1' | the same build, with the **pre-Task-4 probes restored** and Task 4's corpus entries removed | **both `agree`**, exit **0** | **299 of 299**, exit **0** |

That pair is the whole argument for the task. Under a build where `DELEGATE` does nothing at all,
the committed rows this task replaced read `agree` and the corpus as it stood is completely green.
M1's five reddened rows are exactly the five `DELEGATE` ones -- the two gate probes,
`delegate_variable.rex`, `delegate_no_frame.rex` and `delegate_private.rex` -- and none of the six
`FORWARD` rows, which is the right split.

### The wrong-variable arm, and why the spec's own probe value had to change

| arm | table D's `::METHOD` row | `::ATTRIBUTE` row |
|---|---|---|
| M2, committed probe (`'abcdefgh'`) | `diverge-stdout`, `main 6` against `main 8` | `diverge-both` |
| M2, **the spec's probe (`'abcdef'`)** | **`agree`** | `diverge-both` |

M2 makes `send_to_delegate` read the **directive's own name** instead of the `DELEGATE` symbol.
Measured, not inferred: with the spec's `d = 'abcdef'` the `::METHOD` row reads `agree` under that
build, because the unset variable renders as `length`, which is also six bytes. The committed probe
sees it. M2' -- old probes, Task 4's corpus entries removed -- is `agree`/`agree` at exit 0 and
`299 of 299` at exit 0, the same blindness M1' has.

### `FORWARD` has no gate-table row at all, which is why it needs the corpus

| arm | what it deletes | table D | corpus |
|---|---|---|---|
| M3 | the `step` arm, so `FORWARD` is loud again | **`agree`/`agree`, exit 0** | 304 of 310, exit 101 |
| M3' | the same build, Task 4's corpus entries removed | -- | **299 of 299, exit 0** |

M3's six reddened rows are exactly the six `FORWARD` ones. Table D **cannot see `FORWARD` at all**,
which is the plan's own reason for giving it a corpus witness rather than letting it ride on the
`DELEGATE` rows, measured rather than assumed.

### One option at a time

| arm | what it deletes | rows reddened |
|---|---|---|
| M4 | `CONTINUE` is ignored; every `FORWARD` returns | `lang/forward_continue.rex` **alone**, 309 of 310 |

| M5 | `TO` is evaluated and then discarded | `forward_options.rex`, `forward_frame.rex`, 308 of 310 |
| M6 | `MESSAGE` is evaluated, converted, upcased and then discarded | `forward_options.rex`, `forward_continue.rex`, `forward_arguments_not_an_array.rex`, 307 of 310 |
| M7 | `CLASS` is evaluated and checked and then dropped from the send | `forward_class_super.rex` **alone**, 309 of 310 |
| M8 | `ARGUMENTS`' converted values never reach the argument list | `forward_options.rex` **alone**, 309 of 310 |
| M9 | `ARRAY`'s evaluated items never reach the argument list | `forward_options.rex` **alone**, 309 of 310 |

**Every arm leaves table D's two `DELEGATE` rows at `agree` and gate `gate_table_d` at exit 0**, which
is the same fact M3 states from the other side: no gate-table row anywhere sees `FORWARD`.

**Every option has a row that reddens when that option alone is broken**, and every row this task adds
is reddened by at least one arm:

| row | reddened by |
|---|---|
| `method__delegate__subkeyword.rex` | M1, M2 |
| `attribute__delegate__subkeyword.rex` | M1, M2 |
| `lang/delegate_variable.rex` | M1, M2 |
| `lang/delegate_no_frame.rex` | M1, M2 |
| `lang/delegate_private.rex` | M1, M2 |
| `lang/forward_class_super.rex` | M3, **M7 alone** |
| `lang/forward_options.rex` | M3, M5, M6, **M8 alone**, **M9 alone** |
| `lang/forward_continue.rex` | M3, **M4 alone**, M6 |
| `lang/forward_outside_method.rex` | M3 |
| `lang/forward_arguments_not_an_array.rex` | M3, M6 |
| `lang/forward_frame.rex` | M3, M5 |

**The two refusal rows are the two whose "adds coverage" claim rests on construction rather than on
an arm**, and it is checkable rather than asserted: `/bin/grep -rlnE '^[[:space:]]*forward([[:space:]]|$)'`
over `corpus/lang` and `corpus/gate-tables` names the six `forward_*` files and nothing else, so no
other program in the corpus contains a `FORWARD` clause at all, and 98.947 and 98.946 have no other
raiser anywhere.

## Probing past the row

Four shapes no acceptance criterion asked for, three byte-identical on all three descriptors on both
engines and one a divergence that is not this task's.

* **`FORWARD` inside a loop** unwinds past it: `do i = 1 to 3 ; if i = 2 then forward
  message('OTHER') ; end` never reaches the `return` after the loop. rc 0 `a other-m`.
* **`TO` and `CLASS` together**: `forward to (t) class (.Base)` reaches `Base`'s method with `t` as the
  receiver, so `self~kind` inside it answers `Other`'s override. rc 0 `a base-m other-kind`.
* **`CLASS`'s own `>K>` line** under `trace i`: `>V>   SUPER => "The BASE class"` then
  `>K>   "CLASS" => "The BASE class"`, whole transcript identical.
* **`FORWARD` inside `INTERPRET`** is 99.923 on both sides at rc 157 with the same message -- and the
  oracle's traceback carries **three** clause lines where this crate carries two. **Pre-existing and
  not `FORWARD`'s**, established two ways rather than assumed: `interpret "guard on"` has the
  identical one-line difference with no `FORWARD` in it, and the pre-Task-4 pinned binary
  `bench-baselines/pinned/rexx-run-f558ea501` produces the same two-line traceback for the same
  program. Recorded on Task 9's list.

## The licensed divergence, recorded

Written into **Task 9's list** in `docs/superpowers/plans/2026-08-27-phase-5b.md`, in the terms that
list already uses for the oracle's `Error 5` stack overflows -- "**Recorded, not to be built**:
matching an oracle stack overflow is not a target, and this row is here so the audit does not read
the divergence as an unowned defect" -- with the crate-side measurement above beside it.

`corpus/oracle-crashes.txt`'s own entry is corrected in the same commit. Its last paragraph read
"`FORWARD` is rc 120 ... today, so the shape is unreachable here. Task 4 builds `FORWARD`, and when
it does ..." -- a sentence this task makes false, and the rule is that a false comment is corrected
or removed rather than left. It now carries the measured rc 245 and the mechanism.

**No program in `corpus/`, in either gate table, or in any probe run for this task has that shape**,
and the oracle was never handed one. `corpus/lang/forward_options.rex`'s own comment says why a bare
`FORWARD` is not in it.

## Corrections made to the plan, and one the controller owns

Per the rule that a wrong plan is corrected in the plan and not in the report:

* **Task 4's `Build` paragraph** told the implementer to build `DELEGATE` as `dire.xml`'s stated
  equivalence. That would ship a differential mismatch on the traceback, measured. The paragraph now
  carries the measurement and says to build it as a directive-implemented method instead.
* **Task 4's `Done when`** gains the paragraph on the `::METHOD` probe's own value and why
  `'abcdef'` is green over a wrong-variable build.
* **Task 9's divergence list** gains three entries: the self-forward licence, `.Array~of` being
  unimplemented where `FORWARD ARGUMENTS` reaches it, and the `INTERPRET` traceback line.

**One correction is the controller's**, because it is in the spec rather than the plan and a task
does not edit the spec: `docs/superpowers/specs/2026-08-27-phase-5b-instances.md`'s D62 quotes the
`::METHOD` replacement probe with `d = 'abcdef'`, which is the value measured green over a build
reading the wrong variable. The committed probe assigns `'abcdefgh'`.

## The performance sitting

Owed, because this task lands code in `src/` of `rexx-exec`. Run on a quiet machine with no gate
running, after the five gates had finished.

**The pin is current, checked rather than assumed.** From the repository root, with the `:/` prefix
`PINNED.md` warns about:

```
git log --oneline f558ea501..HEAD -- :/rust/crates :/rust/Cargo.toml :/rust/Cargo.lock :/Cargo.toml
```

answers ten commits -- `3148d8cdd`, `3290d5763`, `f65694c8b`, `4e613c349`, `8bcf6375e`, `76a669c24`,
`903c20283`, `08fba0f61`, `fc8532c9e` and this task's `d8e48d353` -- and the ledger names every one
of them in its Task 5, Task 5 fix, Task 3 and Task 3 fix entries. `sha256sum` on
`bench-baselines/pinned/rexx-run-f558ea501` is
`857787f797e88dd94ab839d7b66f1e08cd1a08d22c9e7978d05cbfd49f70494e`, which is `PINNED.md`'s recorded
value.

```
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-f558ea501 \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup \
    --axis dispatchclass --axis bench-rexxcps/rexxcps.rex \
    --rounds 5 --task 4 --commit d8e48d353 --baseline bench-baselines/phase-5b-arms.tsv
```

exit 0. `rexxcps` is given as a path and not a stem, which is the form that finds it. The baseline
file gained rows under `task=4` and holds nothing but `task=3` and `task=4`.

**`instructions:u`, head against the pin** (`across_builds` is `head / pinned`, so above 1 is more
work here):

| axis | tw small | tw large | ir small | ir large |
|---|---|---|---|---|
| alloc4c | 0.999352 | 0.999258 | 0.999783 | 0.999677 |
| arith | 0.999722 | 0.998268 | 1.000048 | 0.999951 |
| compound | 0.999405 | 0.999368 | 1.000114 | 1.000051 |
| emptyloop | 0.997848 | 0.997759 | 1.000098 | 1.000071 |
| strings | 0.999393 | 0.999372 | 0.999775 | 0.999753 |
| varlookup | 0.998894 | 0.998870 | 1.000083 | 1.000044 |
| dispatchclass | 1.001744 | 1.001738 | 1.001884 | 1.003792 |
| rexxcps | 0.999723 | -- | 1.000025 | -- |

**No axis reaches 1%, so nothing here is a finding** and no change-against-layout attribution is
owed. The loudest row is `dispatchclass` on the compiled engine at **+0.379%**, and it is the axis
this change would move if it moved one: `dispatchclass` is four million sends to a method body, and
this commit adds one arm to the `Invocable::Generated` match those sends walk past. It is under half
a percent and under the threshold the guard is written against.

**The `cycles:u` column is recorded and is not a result.** Its extremes are **0.982622**
(`alloc4c` ir large, -1.74%) and **1.038206** (`compound` tw small, +3.82%), with intervals that
overlap 1.0 in both directions -- `compound` tw small is `[0.992105..1.074886]` -- on axes whose
`instructions:u` is flat to a ten-thousandth. That is the pattern this plan's predecessor measured
three times and the reason the guard's instrument is instructions.

## A correction to my own second commit message, which the commits cannot carry

`164d5c106`'s message says "The plan's Task 4 section is corrected in two places" and "Task 9's
divergence list gains three entries". **Both belong to `d8e48d353`, not to that commit.** Checked
with `git diff d8e48d353 164d5c106 -- docs/superpowers/plans/2026-08-27-phase-5b.md`: the plan change
in `164d5c106` is the `INTERPRET` traceback entry and nothing else; the two Task 4 corrections and
the other two Task 9 entries went in with the implementation. The commits are not amended, so the
correction lives here and in the message to the controller. The work itself is where the message says
it is -- only its attribution to a commit is wrong.

## Commits

| commit | what |
|---|---|
| `d8e48d353` | the implementation, the two replaced gate probes, eleven corpus entries and their `EXPECTED_SUBSET_5B` lines, the ownership move, the two Task 4 plan corrections, and Task 9's self-forward and `.Array~of` entries |
| `164d5c106` | the first sitting's rows, and Task 9's `INTERPRET` entry |
| `acb015bd4` | 98.937 for a `FORWARD` that answers a sender a `REPLY` already answered, `Activation::replied_a_value`, and the twelfth corpus entry |
| `8e891cd5a` | five C++ citations pointed at the lines they name; comment-only, and the release binary's sha256 is the same either way after a forced recompile, so it owes no sitting |

## Left open, none of it silent

* **`.Array~of` is not implemented**, oracle rc 0 against rc 120 here, reachable through
  `FORWARD ARGUMENTS` and through anything else that names it. On Task 9's list.
* **A translation error inside `INTERPRET` reports one traceback clause fewer than the oracle**,
  pre-existing, established on a sibling refusal with no `FORWARD` in it and on the pre-Task-4 pinned
  binary. On Task 9's list.
* **The self-forward shape's licensed divergence**, on Task 9's list and in
  `corpus/oracle-crashes.txt`.
* **The spec's D62 probe still spells `d = 'abcdef'`**, which is the value measured green over a
  wrong-variable build. The plan says so; the spec is the controller's to edit.

## The divergence probing past the row found in my own instruction, and the fix

**Found after the first two commits, by asking a question no criterion asked.** A non-continuing
`FORWARD` after a `REPLY` that carried a value:

```
::METHOD m ; reply 'replied' ; forward message('OTHER')
  oracle            rc 0, stdout `a replied` / `b`, stderr the 98.937 report
  ir, tree-walker   rc 0, the same stdout, stderr EMPTY
```

Same status, same stdout, a whole report missing on `stderr`. `RexxActivation::forward`
(`execution/RexxActivation.cpp:1370`-`:1374`) asks `settings.isReplyIssued() && result != OREF_NULL`
before it sends, and `exec_forward` asked nothing.

**The condition is the replied value and not the reply**, measured on the adjacent success: a **bare**
`reply` followed by the same `FORWARD` agrees byte for byte on both sides, `stderr` empty. That is
`RexxActivation::reply` assigning `result = resultObj` (`:1061`), which is null for the bare form.
`CONTINUE` is exempt on both sides and that was measured too, rc 0 and empty `stderr` on all three
sides.

Fixed in `Interp::forward_after_reply`, asked after every option and before the send, which is the
C++'s own position. `Activation` gains `replied_a_value`, which is that `result` field read as a
bool -- keeping the object would root a value the sender already holds.
`corpus/lang/forward_after_reply.rex` carries both arms, and the bare one is what pins the rule to
the value rather than to the keyword.

## The second sitting, and one environmental failure recorded rather than smoothed

The fix commit changes `src/` again, so it owes its own sitting. The first attempt, same command with
`--commit acb015bd4`, **aborted on the eighth axis**:

```
rexx-arms: rexxcps: round 3/5
rexx-arms: `perf stat` produced no cycles:u and instructions:u pair for head ir small
SITTING2=1
```

**Nothing was written.** `rexx-arms` collects its rows and appends at the end, so the abort left
`bench-baselines/phase-5b-arms.tsv` holding exactly its three earlier sittings --
`3/8bcf6375e`, `3/fc8532c9e` and `4/d8e48d353`, 380 rows each -- and `git status` clean. That is the
opposite of the partial-sitting hazard `PINNED.md` records, and it is worth stating because a run
that fails at axis eight of eight could have left seven axes in the canonical record.

**The cause is the machine, established with a probe rather than inferred from the message.** Straight
after the abort, `perf stat -e cycles:u,instructions:u -x, /bin/true` answers a cycles figure and
`<not counted>,,instructions:u,0,0.00` -- five runs of five, on a program that does nothing. So
`instructions:u` was not being counted at all at that moment, on any workload, which is not a property
of this change and not a property of the axis. It counted for the whole first sitting and for seven
of eight axes of this one.

**Sharper than a re-probe: each counter works alone and the pair does not.** `perf stat -e
instructions:u -x, /bin/true` answers a figure and so does `perf stat -e cycles:u -x, /bin/true`; the
two named together give a cycles figure and `<not counted>` for instructions. So the PMU can schedule
one general-purpose counter and not two at the moment, which is a machine state and not a permission
or a kernel setting -- `/proc/sys/kernel/perf_event_paranoid` is `-1`.

**What the fix could plausibly move, measured without `perf`.** No axis of the eight contains a
`FORWARD` or a `REPLY`, so the only path the fix adds to that any axis walks is the activation
constructors, which now store one more `bool`. `size_of::<Activation>()` is the thing that would make
that expensive, and it is **384 bytes at `d8e48d353` and 384 at `acb015bd4`** -- read out of the
compiler by giving each extract a `const _SIZE_PROBE: [(); 0] = [(); size_of::<Activation>()];` and
reading the E0308 it produces, in two `git archive` extracts with their own target dirs. The new
field landed in existing padding, so the struct every activation allocates is the same width it was.
That bounds the risk; it does not replace the sitting.

**The waiter itself was ruled out as the consumer.** With no `perf` process of mine running -- checked
with `ps -eo pid,args | /bin/grep -c '[p]erf stat'` answering only the grep's own shell -- three more
probes give the same `<not counted>` for `instructions:u`. Nothing else on this machine holds a
counter, so the shortage is above it.

**Status of the second sitting: BLOCKED on the machine, and left unmeasured rather than claimed.**
The gates are all green at `acb015bd4`; what is missing is the eight-axis reading of that commit
against the pin. The first sitting's reading stands for `d8e48d353`, which is where all the new
`FORWARD` and `DELEGATE` code is; `acb015bd4` adds one `bool` store per activation and a branch on a
path no axis walks, over a struct whose width did not change. A retry is armed and this section says
so rather than a later reader having to infer it.

## Mutation arm for the fix

| arm | what it deletes | rows reddened |
|---|---|---|
| M10 | `forward_after_reply` always answers `Ok`, so the check never fires | `lang/forward_after_reply.rex` **alone**, 310 of 311, table D `agree`/`agree` at exit 0 |

## Every C++ citation this task wrote, checked against the line it names

Written before the check and corrected after it, because this project has shipped the defect twice
recently -- `fc8532c9e` is a commit called "Point these C++ citations at the lines they name".
Each was read with `sed -n '<n>p'` on the file it names:

| as first written | what that line actually is | corrected to |
|---|---|---|
| `ForwardInstruction.cpp:123` | the doc comment `* Execute a FORWARD instruction.` | `:128`, the `execute` signature |
| `ForwardInstruction.cpp:184`-`:188` | `traceKeywordResult(ARGUMENTS, ...)` and a comment | `:189`-`:191`, the `TheNilObject`/`isMultiDimensional` test and its raise |
| `RexxActivation.cpp:1370`-`:1374` (twice) | `setForwarded` and `stopExecution` | `:1367`-`:1369`, the `isReplyIssued` test and its raise |
| `DirectiveParser.cpp:830` | the comment above it | `:831`, the `getRetriever(delegateName)` call |
| `DirectiveParser.cpp:2441` (twice) | the comment above it | `:2442`, `new DelegateCode(retriever)` |

Four citations were already right and stayed: `ForwardInstruction.cpp:171` (the `isInstanceOf` check),
`:179`-`:210` (the whole `ARGUMENTS` block), `RexxActivation.cpp:1335`-`:1347` (the three defaults) and
`CPPCode.cpp:605`-`:628` (`DelegateCode::run`). **`acb015bd4`'s own commit message carries the wrong
`:1370`-`:1374` range**, written before the check; the commits are not amended, so it is corrected
here.

## Status

DONE, at `8e891cd5a`, with one thing owed and named: the second sitting.

All five gates are green at `8e891cd5a` and were green at `acb015bd4` and `d8e48d353` before it. The phase gate exits
101 by design, with table D's two 5b rows at `agree` and table C's 5b unchanged at one not-agree
(`methodsbyclass`, Task 8's). Ten mutation arms ran in `git archive` extracts: the two rows this task
replaced are green under a build where `DELEGATE` does nothing at all, the corpus without this task's
rows is 299 of 299 under that same build, and every row this task adds is reddened by at least one
arm.

**Owed:** the eight-axis sitting at `acb015bd4`. It aborted on its last axis because this machine's
PMU stopped scheduling `cycles:u` and `instructions:u` together, which is established with probes and
is not a property of the change. Nothing was written to the baseline. The reading at `d8e48d353`,
which carries all of the `FORWARD` and `DELEGATE` code, stands.
