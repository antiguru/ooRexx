# Upstream: Bag~hasItem's documentation typo and its `.nil` answer

These are for the ooRexx project, not for this crate. Found 2026-10-18 during Phase 9d Task 1
(`docs/superpowers/records/2026-10-18-phase-9d/clusters.md`, `bag-hasitem-nil`).

* **Documentation typo.** `oodocs/rexxref/en-US/collclasses.xml:2918-2920` (`mthBagHasItem`)
  says "If *index* is specified, it should be the same object as *index*", which should read
  "the same object as *item*".
* **Code against its contract.** `BagClass::hasItemRexx`
  (`interpreter/classes/BagClass.cpp:173`) returns `TheNilObject` when the index differs from the
  item, where its doc comment and the reference manual both say `.false`.
  `.Bag~of(1,1,1)~hasItem(1, .nil)` answers `The NIL object`. This crate answers 0 under
  Deviation 45.

Not filed. Check for a duplicate before filing: scan the SourceForge ticket titles, as
`oorexx-filed-tickets-2026-08` describes.
