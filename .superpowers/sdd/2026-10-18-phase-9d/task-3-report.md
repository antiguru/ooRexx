# Phase 9d Task 3 report: the reachable refusals

Status: DONE. Base `258272061`. The table, probes and this report land in one commit.

## Files

* `docs/superpowers/records/2026-10-18-phase-9d/refusals.tsv`: the table.
* `docs/superpowers/records/2026-10-18-phase-9d/refusals-probes/sweep.py` and `sweep.tsv`: documented sends with required arguments.
* `docs/superpowers/records/2026-10-18-phase-9d/refusals-probes/sweep2.py` and `sweep2.tsv`: scope-override sends of every name each receiver's ancestry defines.
* `docs/superpowers/records/2026-10-18-phase-9d/refusals-probes/seeds/`: one probe per seed row, `run.sh`, and each probe's `.crate.{out,err,rc}` and `.oracle.{out,err,rc}`.

Binary: `cargo build --release -j 4 -p rexx-exec --bin rexx` under `memcap 8G`, `CARGO_INCREMENTAL=0`, target dir `scratchpad/target-9d-t3`, exit 0. Every crate run was under `memcap 2G` and `timeout`, every oracle run under the standard wrapper, from fresh empty directories. No RexxQueue receiver was built on the oracle.

## Seed rows

The five sites `grep -rn 'Loud::native_method(' rust/crates/rexx-exec/src` prints, one row per receiver:

| Site | Receiver | Reached | Proposed |
|---|---|---|---|
| `context.rs:1180` `StackFrame~executable` | a library routine's frame (`build_native_frame_at`, routine arm) | yes | implement |
| `object_protocol.rs:487` `OBJECTNAME=` | `.nil` through a `Message` scope override | yes | implement |
| `object_protocol.rs:658` `COPY` | `.nil`, a built-in class, a Rexx class, a RexxContext, `.RexxInfo`, each through a scope override | yes | implement |
| `object_protocol.rs:658` `COPY` | the REXX package | no, answers and agrees | guard |
| `object_protocol.rs:658` `COPY` | a `Body::Native` Directory or StringTable | no | guard |
| `hash.rs:214` | condition object, `.methods`, `.routines`, subclasses whose INIT does not forward, any `Body::Native` of a hash class | no, all agree | guard |
| `dispatch.rs:2500` | any | no | guard |

The route that reaches every `yes` row is a `Message` whose name is an array naming `.Object` as the scope, sent from outside any method. The oracle allows it: `.Message~new(.nil, .array~of('OBJECTNAME=', .Object), 'I', 'zz')~send` prints `sent zz`, rc 0, where `.nil~objectName = 'zz'` is 97.1 on both sides. Ordinary sends reach none of these rows: `.context~copy` and `.Array~copy` are 93.970 on both sides, `.nil~copy` is 97.1 on both.

`StackFrame~executable` is reached from the condition a library routine raises (`orxfunction TestRaiseException0`): the oracle's first frame is `ROUTINE R` and its executable is `.routines~r`. A live library routine frame was not built: `orxfunction TestStringArg` given an object with a `STRING` method raises 88.909 on both sides without calling it.

The queued item `2026-10-01-condition-object-directory-methods` already reads closed, and the probe agrees. The REXX package's copy, which `phase-9a-gate.md:291` left refused, now answers `copied Package 0 REXX` on both sides.

## Probe scripts and counts

Commands, from the repository root, with `S` the session scratchpad:

* `python3 -I docs/superpowers/records/2026-10-18-phase-9d/refusals-probes/sweep.py $S/target-9d-t3/release/rexx $S/t3/sweep1 $S/t3/sweep1.tsv`, exit 0. Every `class-methods.txt` row with a `class-set.txt` receiver, instance and class arm, sent with the main-line nonterminals of its railroad diagram (`oodocs/rexxref/en-US/images/classes/*.svg`), through a plain, a subclass (`::class ZzSub subclass <C>`) and an `enhanced` receiver. Class-side rows go to `.C` and to `.ZzSub`.
* `python3 -I docs/superpowers/records/2026-10-18-phase-9d/refusals-probes/sweep2.py $S/target-9d-t3/release/rexx $S/t3/sweep2 $S/t3/sweep2.tsv`, exit 0. For each `class-set.txt` receiver and the extra representations the script lists (`.nil`, a class object, the REXX and a program package, `.context`, a StackFrame, `.RexxInfo`, `.environment`, `.local`, `.methods`, `.routines`, a SmallInt, a long string, a number, a stem, a VariableReference, a started Message), every name `Class~methods(.nil)` gives for each class in the receiver's ancestry, enumerated on both sides, is sent through a `Message` with that class as scope, once with no argument and once with `'v'`, through the same three receiver variants.

Counts, each by its command in `refusals-probes/`:

* `tail -n +2 sweep.tsv | wc -l`: 3564 sends.
* `awk -F'\t' '$7=="yes"' sweep.tsv | wc -l`: 0 end in the fallthrough's text.
* `tail -n +2 sweep2.tsv | wc -l`: 22086 sends.
* `awk -F'\t' '$7=="yes"' sweep2.tsv | wc -l`: 8 end in a `native_method` text. `awk -F'\t' '$7=="yes"{print $1,$2,$3,$4,$5}' sweep2.tsv` names them: `COPY` on `.nil`, a class object (twice, the `Class` and `ClassObject` receivers), a RexxContext (twice, `RexxContext` and `Context`) and `.RexxInfo` (twice, once per receiver label), and `OBJECTNAME=` on `.nil` with an argument. Every one is a plain receiver, so no row is reached only through a subclass or an `enhanced` instance (Review Focus 5).

`sweep2.tsv`'s `probe` column names the generated file in the scratch run directory, which the script regenerates.

## Rows by proposed disposition

* `implement`: `StackFrame~executable` on a library routine frame; `OBJECTNAME=` on `.nil`; `COPY` on `.nil`, on a built-in class object, on a Rexx-defined class object, on a RexxContext and on `.RexxInfo`.
* `guard`: `COPY` on the REXX package (answers and agrees); `COPY` on a `Body::Native` Directory or StringTable; every `hash.rs:214` receiver; the `dispatch.rs:2500` fallthrough. Each row cites the dispatch path that keeps it out of reach.
* `deviation` and `phase`: none.

The subclass and `enhanced` variants do not apply to the `implement` rows' receivers: no program builds a subclass or an `enhanced` instance of `.nil`, a RexxContext, a StackFrame or `.RexxInfo`. A Rexx-defined class object is the class-object row's subclass case.

## Suggested Task 7 instances

1. **Object natives through a scope override.** `COPY` on `.nil`, a RexxContext and `.RexxInfo`, and `OBJECTNAME=` on `.nil`. Files: `rust/crates/rexx-exec/src/dispatch/object_protocol.rs` (`copy_object`, `native_object_name_set`), `rust/crates/rexx-exec/src/environment/identities.rs` for the native body clone. `OBJECTNAME=` on `.nil` changes what `.nil~objectName` answers everywhere afterwards in the oracle (`sent zz`).
2. **`COPY` of a class object.** Built-in and Rexx-defined. The oracle's copy is a distinct class with the same id, its class methods answer (`c~of(1, 2)~items` is 2), and its instances run its methods while their `~class` is not the copy (`c~new~class == c` is 0). Files: `object_protocol.rs`, `rust/crates/rexx-classes/src/registry.rs`, `rust/crates/rexx-exec/src/dispatch/class_protocol.rs`.
3. **`StackFrame~executable` on a library routine frame.** Files: `rust/crates/rexx-exec/src/dispatch/context.rs` (`build_native_frame_at`'s routine arm, which sets no `EXECUTABLE`), `environment/identities.rs` (`library_routine_object`).

Instances 1 and 2 share `copy_object`, so they run in that order or as one instance.

## Concerns

* `.Array~methods` (no argument) enumerates 76 names here and 107 on the oracle; `.Array~methods(.nil)` agrees at 36. Probe `refusals-probes/seeds/methods_count.rex`. No queued item names it.
* Deleting the `guard` rows' sites is not this task's: each still needs the controller's decision on whether it becomes `unreachable!` or stays loud, and `refusal-sites.tsv` re-derives in that commit.
