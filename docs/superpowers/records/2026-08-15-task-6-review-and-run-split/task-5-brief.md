### Task 5: the ANSI spec's premise, the `clause.rs` comment the drain falsified, and the `ITERATE` shape

Review finding **N2**. Its first half, two conditions pending at one boundary losing one, was closed
by `56d9d1c86`, which replaced the single `Option` with a `VecDeque` drained at the boundary. Three
things are left.

#### 1. The ANSI spec records the wrong side of the disagreement

`docs/superpowers/specs/2026-08-15-ansi-condition-delivery.md:9-10` lists as a known
ANSI-versus-ooRexx disagreement: "8.2.4 drains a boundary where **ooRexx and this crate** deliver one
condition and do not re-check". Both halves are now false. The reviewer measured that ooRexx drains:

```rexx
call on user c1 name h1
call on user c2 name h2
call on user c3 name h3
do zi = 1 to 2
  zr = raiser()
end
say 'after' zr
```
(with `h1`/`h2` each `raise ... return`, `h3` plain)

```
oracle                     h1 5 / h2 6 / h3 5 / h1 5 / h2 6 / after 5 / h3 7
01f8010b7, both engines    h1 5 / h2 6 /         h1 5 / h2 6 / after 5 / h3 7
```

On the second pass the oracle's clause at line 5 delivers `h3`, left over from the previous pass's
`END`, and then `h1`, queued by that same clause, both reporting `SIGL 5`, before `END`'s boundary
takes the next. Only what a handler queues *during* delivery is deferred. And this crate drains too
since `56d9d1c86`. So this was never a licensed deviation from ANSI, it was a divergence from
ooRexx, and Phase 7 would design against a false premise.

Re-measure the program above against the live oracle and against both engines at the tree you find,
then correct the spec. If the corrected reading means ANSI 8.2.4 and ooRexx now *agree* on this
point, say so plainly and remove it from the disagreement list rather than restating it.

#### 2. `clause.rs`'s "delivers at most one and does not re-check"

Around `crates/rexx-exec/src/clause.rs:455-459`: "That boundary had already taken one -- this
function delivers at most one and does not re-check -- so the wait is the design rather than a
missing call, and the oracle waits too." The clause in the middle is false since `56d9d1c86`. The
surrounding point may still be sound for a trap queued *during* a delivery, which is what it was
measured on. Establish which part survives, correct the false clause, and leave the rest bounded to
what was measured. Check the whole comment block, not only that sentence.

#### 3. The `ITERATE` shape, to record and not to fix

Pre-existing, byte-identical before and after `e74780054`, both engines. With `do while zn < 2` on
line 5, `zn = zn + 1` on 6, `zr = raiser()` on 7, `iterate` on 8, `end` on 9 and `say 'after' zr` on
10: the oracle defers the second requeue past the re-test and delivers it at the *next pass's* first
body clause (`h3 6`), then leaves the last one for the `say` (`h3 10`); this crate delivers both at
the `ITERATE`'s own line (`h3 8`, twice) and prints the last one before `after`.

Reproduce it at the tree you find, on both engines and all three descriptors, and record it where
this project records found-and-not-fixed divergences. **Do not fix it.** If `56d9d1c86`'s drain
changed it, that is the news: report the new transcripts rather than the ones above.

#### Steps

1. Re-measure all three items at the tree you find. Report every transcript.
2. Correct the spec, correct the `clause.rs` comment, record the `ITERATE` divergence.
3. Gates as in Global Constraints.

---

