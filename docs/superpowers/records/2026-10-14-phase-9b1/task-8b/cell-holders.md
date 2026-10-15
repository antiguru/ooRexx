# Who holds a variable-reference cell

Run from the repository root, at `25968aeba`:

```sh
grep -rn 'SlotRef\|VarRefHome::Cell\|VarHome::Slot\|Target::Cell\|\.promote(\|alias_slot(\|slot_ref(\|FrameAliases\|move_frame(\|ActivityRoots' rust/crates --include=*.rs | grep -v '^rust/crates/rexx-core/tests/' | grep -v ':\s*///' | grep -v ':\s*//'
```

and, for every place a `Body::VarRef` is built, read or copied:

```sh
grep -rn 'Body::VarRef\|VarRefHome\b\|as_variable_reference' rust/crates --include=*.rs | grep -v ':\s*///'
```

## Holders that outlive a clause

| holder | where | traced by |
|---|---|---|
| a `Body::VarRef` whose home is a cell | `run/call.rs` (`>name`), `dispatch/library/surface.rs` (the native API's reference), any `~copy` of one | the heap's trace of the body |
| an alias entry in the running activity's `ActivityRoots` | `RootSet::promote` (the promoted slot), `run.rs` `PROCEDURE EXPOSE` (`alias_slot` to an outer slot that already resolves to a cell), `run.rs` `USE ARG >` (`alias_slot` to the argument's cell) | `ActivityRoots::iter` |
| an alias entry in an idle activity's `ActivityRoots` | `scheduler::Idle::roots` | `Idle::object_roots`, which takes `ActivityRoots::iter` |
| an alias entry in a `REPLY` continuation's `ActivityRoots` | `dispatch.rs` `spawn_continuation` moves the frame with `move_frame`, and the continuation is an `Idle` held by the activation | the same |

`FrameAliases` is built and spent inside `ActivityRoots::move_frame` and holds nothing across an
allocation.

## Holders inside one step

Each of these holds a `SlotRef` in a Rust local while something else in the table above holds the
same cell, so none needs tracing of its own:

- `run.rs`, `PROCEDURE EXPOSE`: the `bindings` vector holds the outer frame's resolved target until
  the alias is installed, and the outer frame's alias holds the cell meanwhile.
- `run.rs`, `USE ARG >`: `bound.home` is the argument's, and the argument is rooted.
- `dispatch.rs`, `VariableReference~value=`: `reference.home` is the receiver's, which is rooted
  across `stem_assignment_value`.
- `run/call.rs` and `dispatch/library/surface.rs`: `promote` installs the alias before the
  `VarRef` is allocated.
- `value.rs` `referenced_value`: reads only.

## Shapes

`crates/rexx-exec/tests/collect_stress.rs`, each measured against the oracle 3 of 3 with 300 passes
of `copies('x', 100000)` as the filler, from a fresh empty directory:

| test | shape |
|---|---|
| `a_reference_outlives_its_procedure` | a reference held past its frame |
| `a_reference_argument_aliases_the_callers_variable` | passed as an argument, `USE ARG >` |
| `a_reference_in_a_collection_outlives_its_procedure` | stored in a collection |
| `a_variable_another_activity_references_survives` | a frame's alias the only holder, in a `REPLY` continuation while the first activity runs |
| `a_reference_outlives_its_dropped_variable` | used after `DROP`, and a variable whose reference was dropped |

The first shape's first draft assigned through the reference and then collected. The oracle
segfaults on that, 3 of 3; `rust/corpus/oracle-crashes.txt` entry 49 has the program. The test
assigns last instead.
