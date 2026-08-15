### Task 3: the two false passages in `loop-header-boundaries`

Review findings **I1** and **I2**, both in
`rust/crates/rexx-exec/tests/ir_dual_cases/loop-header-boundaries`. Both are transcripts that were
true when written and were falsified by `e74780054`. Re-measure everything below against the live
oracle at the tree you find before you write a word.

#### I1: the zero-pass bullet, and the re-measurement claim under it

At `56d9d1c86` this is around `:44-53` and `:62-67`. The bullet records `do i = 1 to raiser()` with
`raiser` returning 0 and a requeueing handler: "the oracle prints `after` and then `G ran 6` ... and
this crate prints `G ran 3` and then `after`". The reviewer reconstructed that program from the
bullet's own numbers (`do` on line 3, `g:` reachable, `say 'after'` on line 6) and measured:

```
oracle                     after / G ran 6
1f4176b47, both engines    G ran 3 / after
01f8010b7, both engines    after / G ran 6      <- matches the oracle
```

So the divergence the bullet records was closed by `e74780054`. The sentence below it, "Re-measured,
and both engines still print what that bullet records", is false. The section framing above
("LOOP-HEADER BOUNDARIES THAT DIVERGE FROM THE ORACLE ARE NOT ROWS HERE ... Each was measured on both
engines, and the engines agree on each") no longer holds for its first bullet.

**Fix by deleting the bullet and its paragraph and promoting the program to a row**, not by
rewriting the claim in place.

#### I2: the "runs the other way" paragraph

At `56d9d1c86` this is around `:89-101`:

```
#   oracle       after unset / G ran 7
#   this crate   G ran 6 / after set
```

Both lines are wrong. The oracle half was corrected in the previous plan itself ("CORRECTED: this
line read `after unset` when it was written, and the oracle prints `after set`") and the case file,
the other place the same transcript lives, kept the uncorrected wording. The crate half is closed:
the `DO UNTIL` stanza roughly fifty lines below in the same file records `H ran 6` / `after set` /
`G ran 7`, and the reviewer confirmed the corpus program's F block prints the same against the live
oracle. The surrounding sentence, "Every other shape recorded in this file has this crate delivering
*later* than the oracle. This one delivers *earlier*, on both engines", is false about a shape the
file now pins as agreeing two hundred lines away.

#### Steps

1. Re-measure both programs against the live oracle on both engines, all three descriptors, at the
   tree you find. Report the transcripts.
2. Delete I1's bullet and paragraph; promote its program to a row.
3. Resolve I2 by deletion where deletion loses nothing, and by a rewrite that names its subject
   where it does not. Say in the report which you chose for each half and why.
4. Re-read the section framing at the top of that block after your edits and confirm every sentence
   in it is true of what remains. Report the sentences you checked.
5. `cargo test --release --workspace` plus the corpus gate under `REXX_CORPUS_GATE=1`, and fmt and
   clippy. A case-file edit changes what the harness compares, so the suite is evidence here.

---

