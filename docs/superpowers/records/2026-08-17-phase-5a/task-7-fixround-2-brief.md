# Task 7, fix round 2 -- prose, one false sentence, one dead branch

**The mechanism is ACCEPTED.** Both Important findings and all four minors are closed. The re-review
re-took the probe and confirms **you were right and my brief was wrong**: it added the control I did
not ask for -- the same program with an *instance* `uninit` prints twice where the class-side spelling
prints once -- which is what makes the measurement conclusive rather than suggestive.

Two things it established that are worth having in the report, because they are load-bearing for 5b:

* **`inherit` does reach `checkUninit`, indirectly**, via `updateSubClasses()` at
  `ClassClass.cpp:1359` into `:1052`. So `:1364` is not the whole mixin story, and your fix gets that
  case right *because* it keys on the flattened behaviour rather than on the propagation. Measured: a
  `MIXINCLASS Object` with an instance `uninit`, inherited by `K`, fires on `.K~new`.
* **The post-pass cannot diverge from the oracle's incremental computation.** The oracle finishes each
  class -- create, `INHERIT`, `defineMethods` -- before starting the next, in dependency order, so both
  flags are a fixpoint your pass recomputes. Nine shapes measured and agreed, negatives included.

Your "no differential row could witness this" claim **holds**, and it was checked rather than taken:
nothing outside tests reads either flag, `rexx-core`'s `Object::has_uninit` is a different flag this
crate never sets, and every observing program needs `~new`, which is `rc 120` on both engines.

## 1. A false sentence about your own mutation -- fix this one first

`rexx-exec/src/lib.rs:5411-5413` says that without the pass, *"every assertion below that expects
`true` reads `false` instead"*. The reviewer ran it: `has_uninit(base)` still reads **true**, because
`ClassGraph::define` sets it, and the test fails at the `kid` row.

Say what the mutation actually does. And note the stronger fact the reviewer found while checking,
which is worth having in its place: deleting the whole pass, deleting only `check_uninit`, and
deleting only `refresh_parent_has_uninit` each redden the test **at a different row**, so both halves
are independently witnessed. That is a better sentence than the one being replaced, and it is true.

## 2. The set-size rule, in the round that applied it elsewhere

New this round, all in code you touched:

* `class_graph.rs:151` -- "**Two** oracle sites set it and this crate has both". Names a size,
  enumerates the members, and is contestable besides: there are four `setHasUninitDefined()` call
  sites, two of them tautological.
* `class_graph.rs:275` -- "the two sites".
* `class_graph.rs:308` -- "the three constructors".
* `lib.rs:5440` -- "the two that inherit it". Mildest, because the assertions below it enforce the
  claim, but it is the same shape.

**The rule, and it is the one that finally held on the previous task after three rounds failed to:
delete the enumeration, do not recount it.** Say what the set is and what puts something in it, and
point at the code that decides membership. This has now recurred in four consecutive rounds across two
tasks, every time inside a round that was striking the same shape somewhere else. Before you report,
re-read every sentence you wrote this round and ask whether it names a size, lists members, or
describes what the code did before.

## 3. A citation off by a line

`class_graph.rs:155` cites `ClassClass.cpp:1214`, which is a comment. The setter is `:1217`.

## 4. The dead branch

`lib.rs:3547-3549`'s `else { continue; }` cannot execute: `:3444` inserts every `order` index into
`classes`. Remove it.

**Ruling on what that does to the sitting:** the sitting tagged `--commit c35ba11cc` still stands, and
say so in the report with this reason -- the branch is provably unreachable, so no path the benchmarks
take changes. Do not run a fresh sitting for this round. If you find the branch is *not* unreachable,
that is a different finding and you should stop and say so rather than deleting it.

## Verification this round owes

* The five gate commands, each status read **unpiped**.
* Item 1's replacement sentence checked by running the three mutations it describes, not by reasoning
  about them.
* Write the report section **before** you commit.
* No sitting, with the reason stated.
