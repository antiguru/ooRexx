**DEFERRED to a later research phase, ruled by Moritz 2026-10-18.**

# Randomise hash iteration order per run

Moritz 2026-10-12: "hash iteration order isn't specified, and ideally changes on every run. The
Oracle has more predictable iteration order, but that is a mis feature in my eyes."

The crate reproduces the oracle's string-key bucket geometry today, so programs and witnesses can
silently depend on order. Goal: a per-run (per-interpreter) seed for every hash collection's
iteration.

Prerequisite: no corpus program, sourceline expectation or suite comparison depends on hash order.
Find them by running the corpus with two different seeds and diffing; each dependent program either
sorts its output or is compared as a multiset under the standing hash-order Deviation.

Cost to measure: the seed enters the hash or the bucket index on every put/get (callgrind, hashnew
and dirread first). Owner: unassigned.

Note 2026-10-12 (Phase 9a Task 8b): Deviation 33 is the standing licence; its HASH_ORDERED_STDOUT list has no test that an entry needs it (Moritz ruled raw difference is not required). The two-seed corpus diff this item needs is also the check that keeps that list honest.
