# Task 8 mutations

Each by exact edit from a saved copy, restored by copy, `git diff --stat` checked after; harness
`mutate.py` in the session scratchpad, debug builds, `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4`.

| id | mutation | test | result |
|---|---|---|---|
| M1 | `prune_dead_object_rows` orphans a dead record's identity whether or not it owns it | `collect_stress` `a_swept_rebuilt_executable_leaves_its_identity_alone` | red: rc 120 after `foo Method` |
| M1 | same | `run::tests` prune and rebuild tests, `a_dropped_method_copy_keeps_running` | green: no shape there swept a non-owning record whose identity stays in use |
| M2 | `hold_method_object` does not make the class hold the copy | `a_dropped_method_copy_keeps_running`, `a_swept_rebuilt_executable_leaves_its_identity_alone` | both red (rc 120, at `old h` and at the second line) |
| M3 | an `ObjectMethod`'s `executable` is not traced (`rexx-core` `body.rs`) | `a_dropped_method_copy_keeps_running` | red: rc 120 after `unset ...` |
| M4 | a `run` activation does not hold its `Method` object | `collect_stress` `a_run_activation_holds_the_copy_it_runs` (fix round 1) | red: `exe Method 0 1` against `exe Method 1 1`; the copy is swept mid-call and a rebuilt executable answers the template's flags. The two earlier copy tests stay green, because a variable there also holds the copy |
| L1 | `run_fragment` does not release its chunk's literals | `an_interpreted_literal_is_released_with_its_chunk` | red: 3 against 1 |
| L2 | `prune_program_rows` does not release its chunks' literals | `a_freed_programs_literals_are_released_with_it` | red: 35 against 33 |
| L3 | a literal `::CONSTANT` is interned immortal again | same | red: 36 against 34 |
| B1 | `ClassGraph::expunge` empties only the two current behaviours | `expunge_empties_the_behaviours_define_retired` | red |
| R1 | `ClassGraph::expunge` does not free the slots | `a_new_class_reuses_an_expunged_classs_behaviours` | red |
