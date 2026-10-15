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
| a `Body::VarRef` only an object awaiting `UNINIT` reaches | an instance variable of such an object | `Heap::collect`'s resurrection pass, which marks a cell's value as the mark loop does |

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

## The cell

A cell is a `Body::Cell(index)` object. Its storage stays in `RootSet::cells`, so the variable
accessors keep their signatures. A first version kept the value in the heap object itself, which
put `&Heap` on `RootSet::frame_slot` and its siblings, and cost assign +4.88%, varlookup +2.74% and
compound +1.08% in callgrind against the pin (`cg-9b1/t8b/step3-vars`). `Heap::collect` marks a
cell's value when it marks the cell's object, and reports the freed objects' cells, which
`collect_now` releases. A cell's generation moves on at release, so a `SlotRef` the collector did
not see panics on its next read rather than reading the variable that took the cell next.

## Shapes

`crates/rexx-exec/tests/collect_stress.rs`, each measured against the oracle 3 of 3 with 300 passes
of `copies('x', 100000)` as the filler, from a fresh empty directory. Each value is computed by
`copies`, so only the cell holds it: a literal is interned and a short string is inline, and the
first draft of these shapes used literals and stayed green with the cell values unmarked.

| test | shape |
|---|---|
| `a_reference_outlives_its_procedure` | a reference held past its frame |
| `a_reference_argument_aliases_the_callers_variable` | passed as an argument, `USE ARG >` |
| `a_reference_in_a_collection_outlives_its_procedure` | stored in a collection |
| `a_variable_another_activity_references_survives` | a frame's alias the only holder, in a `REPLY` continuation while the first activity runs |
| `a_reference_outlives_its_dropped_variable` | used after `DROP`, and a variable whose reference was dropped |
| `a_finalized_objects_reference_keeps_its_variable` | held by an object awaiting `UNINIT` |

## Mutations

Each applied to the tree on the final shapes, run with `cargo test --release -p rexx-exec --test
collect_stress -- reference variable_another`, and reversed by an exact edit:

| mutant | red |
|---|---|
| `Body::VarRef` traces no cell | `a_reference_outlives_its_procedure`, `a_reference_in_a_collection_outlives_its_procedure`, `a_variable_another_activity_references_survives`, `a_finalized_objects_reference_keeps_its_variable` |
| `ActivityRoots::iter` yields no alias cell | every shape in the table above |
| the mark loop does not mark a cell's value | `a_reference_outlives_its_procedure`, `a_reference_in_a_collection_outlives_its_procedure`, `a_variable_another_activity_references_survives`, `a_reference_outlives_its_dropped_variable`, and `rexx-core`'s `a_cells_value_lives_as_long_as_its_object` |
| the resurrection pass does not mark a cell's value | `a_finalized_objects_reference_keeps_its_variable` (a panic) |

The first shape's first draft assigned through the reference and then collected. The oracle
segfaults on that, 3 of 3; `rust/corpus/oracle-crashes.txt` entry 49 has the program. The test
assigns last instead.
