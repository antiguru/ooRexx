# Phase 9a Task 8b re-review (fix round 1, 823db3ab0..8aefd10dc)

Read-only, by reading the diff and the tree at 8aefd10dc; no suite reruns, no probes.

* **I1: addressed.** `class_protocol.rs` (new_subclass) splits the filter: non-class answer keeps `Loud::receiver_class`
  ("other than a class"), a class that is not `is_cloned` gets `Loud::metaclass_new_reused_class` (`lib.rs:531`).
  Deviation 32 gains the "one level up" paragraph with `metaarr`/`metaold` figures and Moritz 2026-10-12; crash entry 8's
  consequence paragraph is narrowed and points at Deviation 32; `refusal-dispositions.tsv` has a DEVIATION row for the
  new constructor and the `receiver_class` row now says "something that is not a class"; a recorded `refusal-sites.tsv` row exists.
* **I2 quadratic: addressed.** `RootSet::globals` is a `NameMap` (`roots.rs:67`, `add_global` get_mut-or-insert, `:128`),
  `which_package` reads the `object_packages` reverse map (`identities.rs:1348`). Figures in the report: 80000 copies 75.66 s -> 0.90 s.
* **I2 retention: addressed as ruled (queued, not fixed).** Sibling entry in `queued/2026-10-12-gc-trigger-blind-to-class-weight.md`
  with 20000 copies 133972 kB / 0.22 s, 80000 copies 479528 kB / 0.90 s, ~5.8 kB per copy, oracle 20316/20324 kB.
* **Minor 1: addressed.** `assert_eq!(programs.len(), copy.0)` before the push (`identities.rs`, ahead of `programs.push`).
* **Minor 2: addressed.** `Heap::names` is a `NameMap` (`heap.rs:85`), `is_empty` early-out kept, `prune_names` uses `retain(|r,_|)`
  with the same slot+generation test (`heap.rs:434`), so no dangling key survives a collection. `copy_object` is unchanged and
  copies through `object_name`/`set_object_name` (`object_protocol.rs:700`), so it still copies the row.
* **Minor 4: addressed.** `activation.rs` executable doc now names `Routine` and `Routine~call`.
* **Minor 5: addressed.** The three ` -- ` comments reworded (`class_graph.rs`, `body.rs`, `lib.rs`); the diff adds no new ones.
* **Minor 6: addressed.** Deviation 33 no longer names `lang/package_writes.rex` or says "today"; Deviation 8's narrating sentence is gone.
  Residual `ord.rex` figures are measured probes, not mutable state.

## Specific checks

(a) Consistent. `package_objects` and `object_packages` have exactly one insert site, adjacent (`identities.rs:144-145`, inside
`package_object`, which the copy path also goes through) and no removal anywhere; the copy's destructuring lists both with `_`.
`RootSet::globals` is never removed from (no `remove_global`/`retain` in the tree), so the old Vec's duplicate and order
semantics reduce to "a name added again replaces its value", which `add_global` keeps; only iteration order changed, and
`RootSet::iter` is consumed for marking (`scheduler.rs:237`) where order is not observable.
(b) Holds, see Minor 2. `ObjRef` keys carry the generation (`prune_names` decodes it).
(c) Holds. The dispatch pin loops `'x'` (receiver_class text "...other than a class"), `.array` and `.Foo` (new text), and keeps the
forwarding neighbour (`id k`); `.Foo` is declared by the program, so the `is_cloned` false path is exercised for a directive class too.
(d) Holds, see I2 retention.

## New findings

None Critical or Important.

* Minor: the Deviation 32 paragraph says "Not witnessed in the corpus" while the refusal-sites row is a recorded witness of the
  refusal itself; same wording as the existing paragraph, harmless.
