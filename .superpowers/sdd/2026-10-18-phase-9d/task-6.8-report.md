# Task 6.8 report: variable references

Clusters `variable-reference-list` and `use-arg-reference-conflict`. Every test of both clusters
passes. The whole suite changes in exactly those rows.

## Commits

* `8a0269f0b` Phase 9d Task 6.8: variable references. The selector read and the exposure rule,
  the three witnesses with their `sourceline_oracle/` files and `phase-9.txt` lines, and the
  closed tests out of `suite-9d-set.txt`.
* `33298eb3b` an empty stem of another name is not referenceable. The suite run on `8a0269f0b`
  left `VarRef TESTDOUBLEARGSTEM` red; this commit closes it and extends the USE ARG witness.
* The records commit that follows holds `docs/superpowers/records/2026-10-18-phase-9d/t6.8/`,
  this report, and `TEST_TRACE_EXPOSE`'s line out of `suite-9d-set.txt` (it passed in both suite
  runs; see below).

## Root causes and fixes

### variable-reference-list

`RexxVariableReference::list` (`IndirectVariableReference.cpp:126`) reads a `(name)` selector
with `variableObject->evaluate`, which does the NOVALUE checks and traces. This crate read it
with `Interp::read` and dropped the `Novalue` flag, so there was no 98.986 under `::OPTIONS
NOVALUE SYNTAX` (or `ALL SYNTAX`), no raised NOVALUE under `SIGNAL ON NOVALUE`, and no `>V>`
line. `read` also took a stem or compound selector as a simple slot: `drop (lst.)` with
`lst. = 'x y'` dropped `LST.` itself where the oracle drops `X` and `Y`, and `drop (q.1)` read
a variable named `Q.1` rather than the tail.

Fix: `Interp::read_list_selector` (`variables.rs`) reads the selector through `read_symbol` by
its shape (simple, stem or compound, the last tracing its own `>C>`), then `echo_symbol_read`.
`DROP` (`run/interpret.rs`), `EXPOSE` and `PROCEDURE EXPOSE` (`run.rs`) call it. `EXPOSE` still
binds the selector before reading it, so the read goes through the object's pool, as before.

### use-arg-reference-conflict

`RexxLocalVariables::aliasVariable` (`RexxLocalVariables.cpp:544`) refuses with 98.995 unless
the target is `isAliasable` (`RexxVariable.cpp:257`): a local variable (`creator != NULL`),
unset, or for a stem empty with its value being the variable's own name. Two parts were missing:

* **Exposure.** An object variable is never local, so a name `EXPOSE` or `USE LOCAL` bound to the
  object's pool is refused whether or not the object variable is set. `target_is_uninitialised`
  (`run.rs`) now answers false when `exposure(frame, index)` is `Some`. This also covers a target
  an earlier `USE ARG >q` bound to a caller's object variable.
* **The stem's name.** `use arg abc., >abc.` binds `ABC.` to the caller's empty stem named `A.`
  and must then refuse. `is_uninitialised_stem` (`stem.rs`) now also requires the stem's name to
  equal the target's.

## Witnesses, red then green

Corpus programs in `rust/corpus/phase-9.txt`, each with its `sourceline_oracle/` file from that
test's oracle driver. Parent is `f2444c6ae` (its `rust/` is identical to `bacf18d5a`'s).
`t6.8/witnesses/summary.tsv`, run by `t6.8/witnesses/run3.sh` on `33298eb3b`:

| program | oracle | parent | task |
|---|---|---|---|
| `variable_list_selector_trace` | rc 0 | rc 0, stdout and stderr differ | identical |
| `variable_list_selector_novalue` | rc 0 | rc 0, stdout differs | identical |
| `use_arg_reference_exposed` | rc 0 | rc 0, stdout differs | identical |

Read stdout and stderr, each path printing:
* `variable_list_selector_trace` (under `::OPTIONS TRACE I`): `>V>   LIST => "a b"` under `drop
  (list)`, `>V>     LIST => "c"` at the callee's indent under `procedure expose (list)`, `>V>
  LST. => "x y"` under `drop (lst.)`, `>C>   Q.1 => "Q.1"` then `>V>   Q.1 => "z"` under `drop
  (q.1)`, `>V>   ZZ => "w"` under `expose zz (zz)`. Stdout `LIT LIT`, `3`, `LIT LIT VAR`,
  `LIT VAR`, `w`: the stem and compound selectors dropped the names their values spell.
* `variable_list_selector_novalue`: 98.986 naming `NOLIST`, `NOLIST2` and `NOLIST3` for DROP,
  PROCEDURE EXPOSE and EXPOSE; the set selectors answer `LIT`, `set in ps`, and `object v`
  through an exposed object variable; `NOVALUE NOLIST4 57` from `SIGNAL ON NOVALUE` over DROP.
* `use_arg_reference_exposed`: 98.995 for `EXPOSE`d and `USE LOCAL` auto-exposed simple and stem
  targets, set and unset object variables (`unsetPool`, `setPool`), an assigned target, a target
  holding the caller's empty stem `UNSET.` or `.stem~new('XYZ.')`, a second `>q` after a reference
  to an object variable (`twiceExposed`), and 88.930 before 98.995 for a kind mismatch. The
  adjacent successes: a `USE LOCAL abc` target, a dropped stem, and a second `>abc` to an unset
  local variable.

## The tests

`suite-9d-set.txt` loses `base/bif/CONDITION` `TEST_NOVALUE_DROP`, `TEST_NOVALUE_METHOD_EXPOSE`,
`TEST_NOVALUE_PROEDURE_EXPOSE`; `base/keyword/TRACE` `TEST_TRACE_DROP`, `TEST_TRACE_EXPOSE`;
`base/keyword/VarRef` `TESTDOUBLEARGSTEM`, `TESTREFAUTOEXPOSED`, `TESTREFAUTOEXPOSEDSTEM`,
`TESTREFEXPOSED`, `TESTREFEXPOSEDSTEM`. Each passes in the whole-suite run on `33298eb3b`.

No queued item closes: `queued-inventory.tsv` holds none of these clusters' items. No exclusion
row is touched.

## Changed rows

Against `t6.7/r1/suite/tests.tsv`, by the command at the top of `t6.8/suite/changed-rows.txt`:
exactly the tests above, each `failed` to `passed` with no row. No other row changed, and the
`SysFileTree TEST_TREE_DIRECTORY` flake did not show. Suite: 402 groups, 15 red (16 at 6.7 r1).

## Checks

From `rust/`, `CARGO_TARGET_DIR=<scratchpad>/t68-target`, `CARGO_INCREMENTAL=0`, each exit
status read unpiped, on `33298eb3b`:

* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --lib`: exit 0, 1152 passed.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --test refusal_sites --test owners --test
  closed_phases --test phase_ten_reasons --test representation_refusals --test
  refusal_dispositions --test coverage --test rexxc --test licensed_divergences`: exit 0. No
  `Loud` site was added or removed, so `refusal-sites.tsv` is unchanged.
* `memcap 16G cargo test -j 4 --release -p rexx-parse`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites`: exit 0.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  1096 of 1096 matching.
* `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=docs/superpowers/records/2026-10-18-phase-9d/t6.8/suite/table.tsv
  memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite`: exit 101 as
  expected, 402 groups, 15 red, 4m38s. The same command on `8a0269f0b` (table overwritten by this
  run): exit 101, 16 red, every row above changed except `TESTDOUBLEARGSTEM`.

## Perf

`t6.8/callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -p "startup interpret parse churn
sayloop" -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.8-run pin=.../rexx-run
parent=.../cg-9d/t6.7-r1/rexx-head head=.../cg-9d/t6.8/rexx-head`, exit 0, 3 rounds. Head is
`33298eb3b`. Own delta is this figure against the pin minus `t6.7/r1/callgrind.txt`'s head
figure, the last perf-checked task.

| program | vs pin | t6.7 r1 vs pin | own delta (ledger) | vs parent, same run |
|---|---|---|---|---|
| startup | +0.0414 | +0.0432 | -0.0018 | -0.0018 |
| interpret | -2.1620 | -2.1660 | +0.0040 | +0.0040 |
| parse | -0.0861 | -0.0860 | -0.0001 | -0.0001 |
| churn | +0.1142 | +0.1114 | +0.0028 | +0.0028 |
| sayloop | +0.0459 | +0.0470 | -0.0011 | -0.0011 |

Every own delta is under +0.05%, and every figure against the pin is under +0.5%. The
libc-subtracted churn figure moves +0.0052; the others by at most 0.0002. The changed paths are
the list-selector read and the `USE ARG >name` check, neither on an ordinary variable read. No
error construction was added: 98.986 and 98.995 use the existing `Raised` constructors.

## Concerns

* **A failure inside a `PROCEDURE EXPOSE` list, trapped by an inherited `SIGNAL ON`, runs the
  handler in a different frame on each side.** Pre-existing and outside these clusters' tests:
  with `sel = '9bad'`, `pg: procedure expose c d (sel)` under `SIGNAL ON SYNTAX` raises 31.2 on
  both, and the handler reads `c=C d=D` on the oracle (an isolated frame where `c` and `d` are not
  exposed, though the selector is: a handler assignment to it reaches the caller) and `c=3 d=4`
  here (the caller's frame, since this crate isolates only after every name resolves). This task
  makes NOVALUE reach the same path, so `procedure expose c (nolist)` under `SIGNAL ON NOVALUE`
  shows the same split (`t6.8/probes/p9`, `p10`, `p11`). The oracle keeping the selector's
  exposure and losing the earlier names looks like an artefact rather than a rule; it needs a
  ruling, and the witnesses avoid it.
* **The oracle compares an empty stem's value with the variable's name by identity.**
  `isAliasable`'s `stem->getValue() == variableName` is a pointer comparison, so `use arg abc.,
  >abc.` with `.stem~new('ABC.')` binds on the oracle (the literal is the same interned string as
  the symbol) and with `.stem~new('AB' || 'C.')` refuses 98.995. This crate compares the names'
  bytes and binds both (`t6.8/probes/p16`). Copying identity would make a representation
  observable, so the computed-name case is a Deviation candidate, not witnessed.
* **`>a.` on an unset caller stem.** The oracle materialises the caller's stem: after `use local
  abc.; use arg >abc.`, `abc.1` reads `A.1` and the caller's `symbol('a.')` is `VAR`; here they
  are `ABC.1` and `LIT` (`t6.8/probes/p5`). Pre-existing, outside these clusters; proposed as a
  queued item.
* **A reference passed on from an alias of an object variable.** A method whose `use arg >q`
  bound `q` to the caller's exposed `v` passes `>q` on; the next callee's `use arg >r` reads `R`
  here and the object variable's value on the oracle (`t6.8/probes/p14`, same on the parent).
  Pre-existing; proposed as a queued item.
* `TEST_TRACE_EXPOSE` was expected to need `dynamic-package-name` too (a `.Method~new` body's
  `>I>` names the creating file here and `""` on the oracle, `t6.8/probes/p3`), but it passes:
  `assertTraceOutput` ignores the package token on a Method `>I>`/`<I<` line
  (`TRACE.testGroup:185`). The `>I>` difference itself
  stays with `dynamic-package-name`.

## Fix round 1

Review: `task-6.8-review.md`. Code commit `a88871bb3`; the records commit that follows holds
`t6.8/r1/` and this section. Round 0 is `8b344309d` (its `rust/` is `33298eb3b`'s).

### Important 1: a compound PROCEDURE EXPOSE selector reads in the procedure's frame

`RexxVariableReference::procedureExpose` (`IndirectVariableReference.cpp:199`) exposes the selector,
then reads it with `list(context)` in the new activation. `expose_names` read it in the caller's
frame. Now a compound selector is first exposed as a tail and its `>C>` traced at that point,
then `procedure_selector_value` (`run.rs`) reads it as the new frame sees it. A tail variable has
a value only if an earlier name in the list exposed it. The compound has a value only if the
list already exposed its stem or the same resolved tail, which includes the selector itself.
Otherwise it is unset, with the NOVALUE check and the `>C>`/`>V>` lines. The later tail pass skips
the line already traced. A simple or stem selector is exposed before it is read, so reading it in
either frame gives the same value, and it keeps `read_list_selector`. DROP and EXPOSE still read
in the current frame. Rerunning round 0's probes (`t6.8/probes/`) leaves every output as round 0
recorded it.

Measured on `t6.8/r1/probes/q1`-`q3` (oracle, round 0, task). `i = 2; a.2 = 'p q'; a.I = 'r'`:
* `procedure expose (a.i)` gives `LIT LIT VAR LIT` (round 0: `LIT VAR VAR LIT`).
* `(a.i) i` gives `LIT LIT VAR 2`.
* `(a.2)` gives `LIT VAR`.
* `(a.i.i)` and `(a.k)` with `k` unset match.
* The trace is `>C> A.I => "A.2"`, `>C> A.I => "A.I"`, `>V> A.I => "A.I"`, `>C> A.I => "A.2"`.
* Under `::OPTIONS NOVALUE SYNTAX` it is 98.986 for `"A.I"` (round 0: no condition).

Each of these is identical to the oracle. `procedure expose i (a.i)` and `a. (a.i)` still differ.
In source order the earlier `i` or `a.` is exposed before the read, and the oracle runs the
selector first (review Minor 3, the queued Deviation candidate). The visible-tail branch can only
be reached through that ordering, so no witness reaches it against the oracle.

### Important 2: a second `>stem.` compares the aliased variable's name

`isAliasable` compares the stem with the name of the variable in the slot, which after `use arg
>abc.` is the caller's. `bind_use_target` now records, in `ActivationCold::reference_names`, the
caller variable's name for each slot it aliases to a cell. `target_is_uninitialised` compares
the stem with that name when the slot has one, and with the target's name otherwise. Measured on
`t6.8/r1/probes/q4`, against both the oracle and round 0. A second `use arg >abc.` binds after the
caller's `say a.`, `drop c.` and a dropped tail; round 0 gave 98.995. It still refuses for a caller
`z. = e.`. A record is lost when an internal call that does not isolate its frame returns, so a
caller's later `use arg >x.` on that slot compares with its own name again. Not measured.

### Witnesses

* `variable_list_selector_trace` gains `procedure expose (a.i)`, `(a.i) i` and `(a.2)` with their
  `symbol` lines and trace.
* `variable_list_selector_novalue` gains 98.986 for `"A.I"` and `procedure expose (a.2)`
  answering `set in pn`.
* `use_arg_reference_exposed` gains `twiceStem` after `say c.`, `drop d.` and a dropped tail, each
  `bound twice` with the caller reading `set`, and the refusal for `f. = g.`.

`sourceline_oracle/` files regenerated. `t6.8/r1/witnesses/summary.tsv` (oracle, round-0 binary
`33298eb3b`, task): round 0 differs on stdout for all three and on stderr for the trace witness.
The task is identical to the oracle on all three.

### Checks, on `a88871bb3`

* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --lib`: exit 0, 1152 passed.
* The refusal tables and `licensed_divergences` (same command as above): exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-parse`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites`: exit 0.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  1096 of 1096 matching.
* The whole suite with the table at `t6.8/r1/suite/table.tsv`: exit 101 as expected, 402 groups,
  15 red, 4m40s. Against `t6.8/suite/tests.tsv` no row changed (`t6.8/r1/suite/changed-rows.txt`
  holds only its command).
* Perf, `t6.8/r1/callgrind.txt`: pin, round 0 (`33298eb3b`) and `a88871bb3` in one run, exit 0.
  Whole process against the pin, r0 then head: startup +0.0415 / +0.0433, interpret -2.1620 /
  -2.1689, parse -0.0861 / -0.0860, churn +0.1142 / +0.1146, sayloop +0.0459 / +0.0473. The
  ledger's own delta against `t6.8/callgrind.txt`'s head figures: startup +0.0019, interpret
  -0.0069, parse +0.0001, churn +0.0004, sayloop +0.0014. Every one is under +0.05%, and the
  libc-subtracted figures move by at most 0.0001.

## Fix round 2

Re-review: `task-6.8-rereview.md`. Code commit `f48b4d866`; the records commit that follows holds
`t6.8/r2/` and this section. Round 1 is `0b405bf1e` (code `a88871bb3`).

### Importants 1 and 2: the name comes from the cell, not from an activation

`ActivationCold::reference_names` is deleted. `Body::Cell` (`rexx-core/src/body.rs`) is now
`Cell(u32, Box<[u8]>)`: the cell's storage index and the name of the variable whose slot
`promote_slot` moved into it. `promote_slot` takes that name from its two callers, the `>name`
term (`run/call.rs`) and the native API's reference (`dispatch/library/surface.rs`). It creates a
cell only when the slot does not already resolve to one, so the name is always the owning
variable's own: a slot that resolves to frame storage is either the variable itself or a
`PROCEDURE EXPOSE` alias of the same name. `cell_owner_name` (`variables.rs`) follows every alias
from the target slot to its cell and answers that name. `target_is_uninitialised` compares the
stem with it, and with the target's own name when the slot resolves to frame storage. The name is
bytes inside a heap object, so there is nothing new to root. The collector's three `Body::Cell`
arms (`heap.rs`) ignore the name.

Measured on `t6.8/r2/probes/s1` (oracle, round 1 `r`, task `t`, pre-task `p`). The caller has run
`say` on each stem:
* `innerThenOuter` (a frame-sharing internal call binds, then the procedure binds).
* `innerTwiceOuter` (the internal call binds twice).
* `deep1` (a reference passed on through three procedures, bound twice at the bottom).
* `deepInner` (a passed-on reference bound in a frame-sharing call, then by its caller).
* `outerThenInner`, `interpTwice`.

Each of these binds and the caller reads `set`, on the oracle and the task. Round 1 gave 98.995
for the first four. `dropThenRebind` still gives 98.995 and `n.1 ABC.1` as on the oracle; the
pre-task binary bound.

### Witnesses

`use_arg_reference_exposed` gains `innerThenOuter`, `innerTwiceOuter`, `passedOnTwice` (three
procedures down, bound twice) and `innerOfPassedOn`. Each answers its bound line, and the caller
reads `set` in all four. On round 1 each is 98.995 and the caller reads its own derived names.
The `sourceline_oracle/` file is regenerated. `t6.8/r2/witnesses/summary.tsv` (oracle, round-1
binary, task): the task is identical to the oracle on all three witnesses. Round 1 differs only
on `use_arg_reference_exposed`'s stdout; the other two witnesses do not reach this change.

Round 0's and round 1's probes (`t6.8/probes/`, `t6.8/r1/probes/`) rerun on `f48b4d866`. Every
stdout and stderr is identical to the task output those rounds recorded. `p12` and `p13` have
only oracle records: `p12` matches the oracle, and `p13`'s stderr is the `>I>` package name
`dynamic-package-name` owns.

### Checks, on `f48b4d866`

* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-exec -p rexx-core --all-targets -- -D warnings`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --lib`: exit 0, 1152 passed.
* `memcap 16G cargo test -j 4 --release -p rexx-core`: exit 0 (includes `unsafe_sites`, `roots`
  and `collect`).
* The refusal tables and `licensed_divergences` (same command as above): exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-parse`: exit 0.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  1096 of 1096 matching.
* The whole suite with the table at `t6.8/r2/suite/table.tsv`: exit 101 as expected, 402 groups,
  15 red, 4m37s. Against `t6.8/r1/suite/tests.tsv` no row changed.
* Perf spot check, `t6.8/r2/callgrind.txt`: `-r 1 -p "interpret churn"`, pin, round 1 and
  `f48b4d866`, exit 0. Against the pin, r1 then head: interpret -2.1689 / -2.1653, churn +0.1139 /
  +0.1143. The round's delta is +0.0036% and +0.0004%. The libc-subtracted figures move by at
  most 0.0001.
