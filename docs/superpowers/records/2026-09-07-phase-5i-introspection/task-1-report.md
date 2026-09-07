# Task 1 — `ARG` option `"A"`, and the eight `of` rows it unblocks

BASE `a634231dd`, tree clean at start. The controller's plan and Task 0 follow-up commits landed in
this worktree while the task ran, so the work is committed on top of `6f5a60db2`; of those, only
`introspection-arguments.tsv`, `introspection_arity.rs` and `tests/support/arity.rs` touched
`rust/`, and every table below was re-refreshed at that tip.

## 1. Pre-flight: the brief read against the tree

Raised with the controller before any code was written.

**1a. The touch list was missing `crates/rexx-exec/tests/state_builtin_oracle.rs`.** Its
`DECLARED_GAPS` names `"arg_option_array"`, whose case is `say arg(1,'A')`, and that file's module
doc makes a declared gap that starts *agreeing* red on purpose. Measured on the oracle, rc 0: that
program prints one empty line, because at the top level with no arguments `arg(1,'A')` is
`new_array(0, arglist)` -- size 0, whose string value joins to nothing -- and this crate prints the
same once the arm works. Confirmed by running: with the arm implemented and the row still in
`DECLARED_GAPS`, `every_state_builtin_case_matches_the_oracle_except_the_declared_gaps` fails; with
the row removed it passes. The row is deleted here.

**1b. The commit is red by construction on the four controller-owned artifacts.** Named to the
controller before implementing, and one of them confirmed by running rather than argued:

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus   -> exit 101
  every_lang_program_is_run_or_named_unfiled FAILED
  {"lang/arg_option_array.rex", "lang/map_collection_of.rex"} are in .../corpus/lang and
  named by no phase subset file
```

`corpus.rs:845` requires every `corpus/lang/*.rex` to be named by a phase subset file or by
`corpus/unfiled.txt`; `collection_arity.rs:87` and `method_bodies.rs`'s header assert the committed
tables equal the live run. All of those files are the controller's. See section 10.

**1c. A brief gap that would have produced a wrong implementation.** The brief gives the
past-the-end case as wanting `dimensions: None` and cites `arg(4,'A')` on a three-argument call. It
does not say what `arg(1,'A')` answers when the call has **no** arguments -- numerically also past
the end, and exactly the path all eight `of` rows take, since `method-bodies.txt` sends `of` with no
arguments and the body's first statement is then `arg(1,'a')` on an empty list.
`BuiltinFunctions.cpp:930` tests `position == 1` *before* `position > size`, so that case goes to
`new_array(size, arglist)` = `ArrayClass(objs, 0)`, and `ArrayClass.cpp:314`-`:323` sets
`dimensions = new(1) NumberArray(1)` precisely when `count == 0`.

**1d.** The global constraints' bullet about adding a witness's file to `dispatch_seam.rs`'s
`CLEARANCE_CONSUMERS` does not apply: that list (`dispatch_seam.rs:365`) names the *source* files
allowed to mention the `Cleared` seam token, and a builtin in `builtin/state.rs` takes no clearance.
Nothing was added to it.

## 2. `MapCollection~of` walked statement by statement

`CoreClasses.orx:1238`-`:1266`, every send in it run here on both engines against the oracle before
any code was written. Each row below is three descriptors identical unless said otherwise.

| send | crate vs oracle |
| --- | --- |
| `arg(1, 'a')` | crate rc 120 `ARG option "A" ...`, oracle rc 0 -- **the only blocker** |
| `self~new` | identical |
| `arg()` | identical |
| `args~last` | identical (`3` on `.array~of('k',,'v')`) |
| `args~hasIndex(i)` | identical (`1` at 1, `0` at 2) |
| `args[i]` | identical |
| `arg~isA(.array)` | identical |
| `arg~dimension` | identical |
| `arg~items` | identical |
| `collection~put(value, index)` | identical |
| `raise syntax 93.903 array(i)` | identical, caught by a trap: `93 SYNTAX` |
| `.context~name` | crate rc 120 `method "NAME" of class "RexxContext" is not implemented (Phase 5)`, oracle rc 0 |

So the task's row count holds: `arg` is the single unimplemented send on the path the eight rows
take, and `.context~name` -- reached only from the 88.923/88.924 arms -- is Task 6's, exactly as
the brief says.

**The eight rows close on the zero-argument path.** `method-bodies.txt` sends `of` with no
arguments, so `arg() == 0` and the body returns the empty collection at `CoreClasses.orx:1246`
without reaching any raise. The argument-error path is exercised here by
`corpus/lang/map_collection_of.rex` for 93.903 only; the two `.context~name` raises are Task 6's, so
these eight closed rows are not a fully exercised body.

## 3. The oracle's semantics, re-measured

Oracle 2026-09-08, from a fresh empty directory, wrapped per the constraint. From `call r 'a', , 'c'`:

```
p1  3 2 1 0 [a] [c]      arg(1,'A')  size items dimension hasIndex(2), then [1] and [3]
p2  2 1 1                arg(2,'A')
p3  1 1 1                arg(3,'A')
p4  0 0 0 The Array class    arg(4,'A')
p99 0 0 0                arg(99,'A')
count 3
```

The brief's table reproduced exactly. From `call r` with **no** arguments, same run, rc 0:

```
zero-p1 0 0 1 The Array class    arg(1,'A')
zero-p2 0 0 0 The Array class    arg(2,'A')
count 0
```

That second block is finding 1c: position 1 answers a *shaped* array even with nothing to put in
it, and any other position answers an unshaped one. `native_array_of`'s
`args.is_empty().then(|| [0])` is therefore the right shape for position 1 and the wrong one for
past the end -- the brief's warning about that function is right about the arm it names and does
not cover this one.

## 4. The change

`crates/rexx-exec/src/builtin/state.rs`, the `Some(b'A')` arm of `arg`, mirroring
`BuiltinFunctions.cpp:928`-`:948` arm for arm: position 1 takes the whole list, a position past the
end takes an empty unshaped array, anything else takes the sub-list from that position. `slots` is a
`Vec<Option<ObjRef>>` copied straight out of `interp.call_context.arguments`, so an omitted argument
stays an empty slot -- the same copy-the-pointers-through `new_array(size, arglist)` does with
`OREF_NULL`. Nothing is sent `INIT`, because `new_array` does not; `ofRexx` is the caller that does.
The array is rooted with `push_temp` as its only root.

`positive_integer` and the two checks in front of the option switch are untouched, so the four
measured orderings in `arg`'s own doc still hold -- asserted, see section 5.

Also in this file: the module doc's "what is not here" paragraph no longer claims `ARG(n,'A')` is
loud, the `arg` doc gains the `'A'` rows and the position-1 rule, and
`the_object_valued_options_are_loud` loses its `ARG` row. `crates/rexx-exec/tests/state_builtin_oracle.rs`
loses `"arg_option_array"` from `DECLARED_GAPS` (finding 1a).

## 5. Witnesses

Two corpus programs, each byte-identical on stdout, stderr and exit status across
`REXX_ENGINE=ir`, `REXX_ENGINE=tree-walker` and the oracle.

**`rust/corpus/lang/arg_option_array.rex`**, rc 0. Covers `arg(n,'A')` at n = 1, an interior
position, exactly the count, count + 1 and 99, asserting `~size`, `~items` and `~dimension` for
each; the omitted middle argument through `~hasIndex(1..3)` and the values at 1 and 3; the
no-arguments call at positions 1 and 2, which is the shaped/unshaped pair; and `arg(0,'A')` and
`arg('nan','A')` under traps, printing `condition('E')` so 14 and 12 are told apart rather than
both reading as `rc 40`. Its output, all three sides:

```
whole 3 2 1 Array
holes 1 0 1
values [p1] [The NIL object] [p3]
from 2 2 1 1 [The NIL object] [p3]
from 3 1 1 1 [p3]
past the end 0 0 0 Array
far past the end 0 0 0
count 3 exists 0 omitted 1
none at 1 0 0 1 Array
none at 2 0 0 0 Array
position zero 40 SYNTAX 14
position not a number 40 SYNTAX 12
```

**`rust/corpus/lang/map_collection_of.rex`**, rc 163. All seven concrete mapped classes end to end,
each **read back with a second send** per the plan's NEW constraint -- `made['k1']`, `made['k2']`
and `made~hasIndex('k2')` on the collection `of` built, not only its `~items`; the short argument
list (`of` with none) and a subclass; `MapCollection` itself reaching its own abstract `PUT`; and,
uncaught at the end, the omitted-argument 93.903 that only a real hole can produce. Its output and
stderr, all three sides:

```
Directory Directory 2 [v1] [v2] 1
IdentityTable IdentityTable 2 [v1] [v2] 1
Properties Properties 2 [v1] [v2] 1
Relation Relation 2 [v1] [v2] 1
Stem Stem 2 [v1] [v2] 1
StringTable StringTable 2 [v1] [v2] 1
Table Table 2 [v1] [v2] 1
empty 0 Table 0
subclass K
mixin 93 SYNTAX 965
--- stderr
  1254 *-*       Method OF with scope "MapCollection" in package "REXX" (no source available).
    33 *-* say .Directory~of(.Array~of('k', 'v'), , .Array~of('k2', 'v2'))~items
Error 93 running REXX line 1254:  Incorrect call to method.
Error 93.903:  Missing argument in method; argument 2 is required.
```

Both have a `crates/rexx-parse/tests/sourceline_oracle/<name>.txt` regenerated with the
`.Package~new` driver from that test's module comment; `sourceline_matches_the_interpreter_for_every_corpus_program`
passes.

Unit tests in `builtin/state.rs`: `arg_option_a_copies_the_list_holes_and_all` and
`arg_option_a_shapes_position_one_even_with_no_arguments`. The second is the adjacent success for
the first -- same arm, same option, a call with nothing to lose.

## 6. Red controls -- predictions written before the runs

**Written 2026-09-08 before either control was applied.** Both controls vary the
change itself, not something about it.

### Control 1 -- revert the `'A'` arm to `Loud::builtin_option_object`

Predicted:

| subject | prediction |
| --- | --- |
| `corpus/lang/arg_option_array.rex`, both engines | rc 120, empty stdout, stderr `rexx-exec: ARG option "A" answers an Array, which is not implemented` -- RED |
| `corpus/lang/map_collection_of.rex`, both engines | rc 120, empty stdout, the same stderr -- RED |
| `state::tests::arg_option_a_copies_the_list_holes_and_all` | RED (`output` asserts exit 0) |
| `state::tests::arg_option_a_shapes_position_one_even_with_no_arguments` | RED, same reason |
| `state::tests::the_object_valued_options_are_loud` | GREEN -- it no longer lists `ARG` |
| `state_builtin_oracle::arg_option_array` | RED, because the row was removed from `DECLARED_GAPS` |

### Control 2 -- the `'A'` arm answers a hole-free array (`ObjRef::NIL` in place of an empty slot)

This is the control that proves the witness set can see the hole.

Predicted:

| subject | prediction |
| --- | --- |
| `corpus/lang/arg_option_array.rex` | RED. `holes` becomes `1 1 1` where it is `1 0 1`, `whole` items becomes 3 where it is 2, `from 2` items becomes 2 where it is 1. rc stays 0. |
| `corpus/lang/map_collection_of.rex` | RED, but by a different route: `args~hasIndex(2)` is now 1, so the 93.903 is never raised; the body reaches `\arg~isA(.array)` on `.nil`, takes the 88.923 arm, and sends `.context~name` -- Task 6's gap. So rc 120 with `method "NAME" of class "RexxContext" is not implemented (Phase 5)`, stdout matching through the `mixin` line. |
| `state::tests::arg_option_a_copies_the_list_holes_and_all` | RED |
| `state::tests::arg_option_a_shapes_position_one_even_with_no_arguments` | GREEN -- that call has no arguments, so it has no hole to lose |
| `state::tests::the_object_valued_options_are_loud` | GREEN |
| `state_builtin_oracle::arg_option_array` | GREEN -- `say arg(1,'A')` at top level has no hole either |

### Results -- every prediction confirmed, none falsified, none unobservable

**Control 1**, applied by restoring the `Loud` refusal and rebuilding `rexx-run`:

| subject | predicted | measured | |
| --- | --- | --- | --- |
| `arg_option_array.rex`, ir and tree-walker | rc 120, empty stdout, the loud message | rc 120, 0 bytes of stdout, `rexx-exec: ARG option "A" answers an Array, which is not implemented` | confirmed |
| `map_collection_of.rex`, ir and tree-walker | same | same | confirmed |
| `arg_option_a_copies_the_list_holes_and_all` | RED | FAILED | confirmed |
| `arg_option_a_shapes_position_one_even_with_no_arguments` | RED | FAILED | confirmed |
| `the_object_valued_options_are_loud` | GREEN | ok | confirmed |
| `state_builtin_oracle`'s declared-gaps test | RED | FAILED (`state_builtin_oracle.rs:542`) | confirmed |

`cargo test --release -p rexx-exec --lib builtin::state --no-fail-fast` -> exit 101, 23 passed /
2 failed. `--test state_builtin_oracle` -> exit 101, 19 passed / 1 failed.

**Control 2**, `ObjRef::NIL` in place of every empty slot:

| subject | predicted | measured | |
| --- | --- | --- | --- |
| `arg_option_array.rex` | RED: `whole` items 3 not 2, `holes` `1 1 1` not `1 0 1`, `from 2` items 2 not 1, rc still 0 | exactly those three lines differ, rc 0 | confirmed |
| `map_collection_of.rex` | RED by the 88.923 route: stdout matching through `mixin`, then rc 120 `method "NAME" of class "RexxContext" is not implemented (Phase 5)` | stdout byte-identical, rc 120, that message | confirmed |
| `arg_option_a_copies_the_list_holes_and_all` | RED | FAILED | confirmed |
| `arg_option_a_shapes_position_one_even_with_no_arguments` | GREEN | ok | confirmed |
| `the_object_valued_options_are_loud` | GREEN | ok | confirmed |
| `state_builtin_oracle`'s declared-gaps test | GREEN | ok, 20 passed | confirmed |

`--lib builtin::state --no-fail-fast` -> exit 101, 24 passed / 1 failed.

**"Can fail" is not "adds coverage", so the extra run.** With control 2 still applied and the two
new witnesses and their fixtures moved out of the tree,
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus --no-fail-fast` is **exit 0,
23 passed, 0 failed**. Nothing that existed before this task sees the hole-free array: the only
place the suite called `arg(1,'A')` was `state_builtin_oracle`'s top-level case, which has no
omitted argument to lose. So the witnesses add coverage rather than merely being able to fail.

Both controls were reverted by `cp` from the copy in the task's scratch directory (never
`git checkout --`), the file re-diffed against the copy to confirm it came back identical,
`rexx-run` rebuilt, and both witnesses re-run: identical again on all three sides, rc 0 and rc 163.

## 7. Shared-table verdicts, before and after

**The controller-owns-the-shared-tables constraint was revoked at this task's pre-flight** (finding
1b), so the four artifacts are committed here rather than filed for a later commit.
`.superpowers/sdd/2026-09-07-phase-5i-introspection/task-1-tables.md` records the same moves and
says the constraint was lifted.

**The failure text before refreshing**, which is the evidence a green cell afterwards cannot give.
`cargo test --release -p rexx-exec --test collection_arity --no-fail-fast` on the changed crate
against the committed table -> **exit 101**, 22 passed / 1 failed:

```
the_table_matches_the_three_sides FAILED, collection_arity.rs:88
corpus/collection-arity.tsv disagrees with the interpreters.
  Row { class: "Properties", method: "setLogical", arm: "instance",
        verdict: "agree",        evidence: "rc0" },
  Row { class: "Properties", method: "setLogical", arm: "instance",
        verdict: "send-differs", evidence: "oracle rc0 SENT; crate rc120 rexx-exec:
                                            ARG option \"A\" answers an Array, ..." },
```

One row, and it names the change. Same shape for
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` -> exit 101,
`every_lang_program_is_run_or_named_unfiled` naming both new witnesses as filed nowhere.

### `corpus/method-bodies.txt`

`REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies`

| class | method | arm | before | after |
| --- | --- | --- | --- | --- |
| MapCollection | of | class | `loud` ARG option "A" answers an Array, which | `answers` rc 0 |
| Directory | of | class | `loud` (same) | `answers` rc 0 |
| IdentityTable | of | class | `loud` (same) | `answers` rc 0 |
| Properties | of | class | `loud` (same) | `answers` rc 0 |
| Relation | of | class | `loud` (same) | `answers` rc 0 |
| Stem | of | class | `loud` (same) | `answers` rc 0 |
| StringTable | of | class | `loud` (same) | `answers` rc 0 |
| Table | of | class | `loud` (same) | `answers` rc 0 |

Those are the whole diff of the file; no other row moved.

**These eight rows are `answers` about a zero-argument send**, per the table's own header and this
report's section 2 -- the body returns the empty collection before it reaches any raise. What makes
them more than that here is `corpus/lang/map_collection_of.rex`, which sends real arguments to every
one of the seven concrete classes and reads an entry back out.

### `corpus/collection-arity.tsv`

`REXX_COLLECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec --test collection_arity`

| class | method | before | after |
| --- | --- | --- | --- |
| Properties | setLogical | `send-differs` oracle rc0 SENT; crate rc120 `ARG option "A" ...` | `agree` rc0 |

The whole diff of the file. `Properties~save` stays `send-differs` for `stream_init` (Phase 7).

### `corpus/introspection-arity.tsv`

`REXX_INTROSPECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec --test introspection_arity`
-> exit 0, 25 passed, and `diff` against the committed file is **empty**. No introspection row turns
on `ARG` option `"A"`. Read its header before citing it: it carries `no-value` and `unstable`
verdicts that `collection-arity.tsv` does not.

### `corpus/phase-5c.txt` and `EXPECTED_SUBSET_5C`

Two lines each, in the same order, appended after `lang/do_over_request_array.rex`:
`lang/arg_option_array.rex` and `lang/map_collection_of.rex`.

## 8. `Properties~setLogical`

**It moves, and it is the only row in the table that does.** Refreshed with
`REXX_COLLECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec --test collection_arity`
(exit 0, 23 passed), the whole diff of `corpus/collection-arity.tsv` is one line:

```
< Properties	setLogical	send-differs	oracle rc0 SENT; crate rc120 rexx-exec: ARG option "A" answers an Array, which is not implemented
> Properties	setLogical	agree	rc0
```

The attribution in the scope note was right. Walked first, for the same reason `of` was: the body
(`CoreClasses.orx:2133`-`:2145`) needs `arg(1,'A')`, `Array~[]=` and `forward message 'SETPROPERTY'
arguments (args)`, and the last two were measured working here before the change --
`p~setLogical('flag',1)` was the crate's rc 120 `ARG option "A"` against the oracle's `[true]`, and
is `[true]`/`[false]` on both engines now.

`Properties~save` stays `send-differs` and is Phase 7's (`stream_init`), unchanged by this task.

## 9. Gates

**Fast checks in the working tree, before the commit**, all read unpiped:

```
rustfmt --edition 2024 <each file I edited>              exit 0
cargo fmt --all --check                                  exit 0
cargo clippy --workspace --all-targets -- -D warnings    exit 0
cargo test --release --workspace --no-fail-fast          exit 0
    116 `test result: ok` lines, 0 `test result: FAILED`
```

**The seven gates run in the gate worktree** `/home/moritz/dev/repos/ooRexx-5i-gates`, pinned to
this task's commit sha, per the constraints. Cells are
filled from the status file after the run, per the controller's ruling that the implementer writes
its own gate lines.

| gate | command | result |
| --- | --- | --- |
| G1 | `cargo fmt --all --check` | **G1** |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | **G2** |
| G3 | `cargo test --release --workspace --no-fail-fast` | **G3** |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | **G4** |
| G5 | `cargo test --release -p rexx-exec --test collection_arity` | **G5** |
| G6 | `cargo test --release -p rexx-exec --test introspection_arity` | **G6** |
| G7 | `cargo test --release -p rexx-exec --test introspection_scopes` | **G7** |

## 10. Concerns

1. **The eight `of` rows read `answers` about a send with no arguments, and that is worth saying
   out loud.** `method-bodies.txt`'s header says as much for every row; here it means the body
   returns at `CoreClasses.orx:1246` without executing the loop, the `hasIndex` check, the two
   `isA`/`dimension` checks or any `put`. `corpus/lang/map_collection_of.rex` is what actually
   exercises those, for all seven concrete classes with a second send each. The two argument-error
   raises that send `.context~name` (88.923, 88.924) remain refused here and are Task 6's.

2. **`Properties~save` is still `send-differs`** in `corpus/collection-arity.tsv`, for
   `stream_init` (Phase 7). It was already the other non-stream row and this task does not touch it.

3. **G1 and G2 read off a warm target directory**, per `rust/CLAUDE.md`'s rule that a same-session
   green is only evidence if the linter re-examined the code. Task 9's clean-target run at the phase
   boundary is where that reading becomes real.

4. **`memcap 8G` is too tight for the release fast check on this machine, and it fails in a way
   that reads as a test failure.** `memcap 8G cargo test --release --workspace --no-fail-fast`
   came back exit **137** with zero `test result` lines and
   `memcap: OOM-killed at the 8G cap (peak 8.0G)`, the last thing on stderr being
   `Compiling rexx-exec` -- so the cap killed the *compile*, not a test. Re-run without the cap it
   completes. The gate table's G3 has no `memcap` and G4 (debug) does, which is the right split;
   this is a note for anyone tempted to add one to G3.

5. **The `push_temp` on the fresh array has no witness that it is load-bearing.** Nothing else names
   the array between `alloc_with` and the return, so it is the only root the value has -- the same
   argument `dispatch.rs`'s `missed` root stands on, and like that one, removing it reddens nothing
   I ran. It is kept because the value would otherwise be unrooted, not because a stress run caught
   it.
