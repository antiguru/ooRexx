# How this project measures a performance change

Derived from the 2026-09-09/10 round, in which every rule below was learned by
getting it wrong first. Each is a command or a check, not a principle.

## Measure retired instructions, not wall clock

`perf stat -e instructions:u`. One event per run so nothing is multiplexed.

**Wall clock cannot see a 2% change on this machine and inverted the sign of
one.** Medians of seven alternating runs put `dispatch` at +3.5% while a
*control the change could not reach* moved +3.4%: a band of about 9%. The same
change on retired instructions was **-2.0%**, with controls flat to **0.003%**.

Wall clock is still what the parity ratio is quoted in -- that is a different
question, asked once, at the end. See [[measuring-optimisation-work]].

## Always run a control the change cannot reach

A program whose profile contains none of what you touched. `varlookup` and
`compound` served for the hashing work; they read 1.0000 while `dispatch` moved
9%, which is what makes a 0.3% figure elsewhere readable.

**State when a program stops being a control.** The trace-op split reached every
evaluated node, so those two became subjects for that change and were reported
as such rather than offered as evidence.

## Compare the sha256 of the two binaries

```sh
sha256sum a b
```

An A/B in this round was void because the "baseline" was copied from
`target/release` without rebuilding after an unrelated ablation was reverted.
Its hash matched the ablation's exactly, which is the only thing that caught it.
A clean `git status` says nothing about `target/`.

## Ablate to a ceiling before building anything

Make the thing free -- unsoundly if necessary -- and read the bound. Path B's
entire temp-rooting tax removed is **1.4%**; that closed a path in an hour
rather than a phase. Check the output is still byte-identical, or the number
compares nothing.

Two spikes in this project built first and measured after, and both were
retracted.

## Sample on the event you are bound by

This interpreter is instruction-bound, so cycle-sampling ranks the wrong things:
cheap instructions that retire at high IPC are nearly invisible to it. The same
run by cycles against `instructions:u` moved `run_ops_from` 9.3% -> 16.0% and
brought `memmove` in at 6.1% from nowhere.

`samply` records cycles. Use `perf record -e instructions:u` for this.

## Use a precise event before blaming one instruction

A non-precise profile put **6.41%** on a single 16-byte spill. The same
instruction under `cycles:pp` is **0.07%** -- with a deep pipeline the sample
lands a few instructions past the one that stalls.

`instructions:pp` does not exist on this AMD part; `cycles:pp` does. One refusal
is not proof precise sampling is unavailable.

## Expand inlines before calling a profile flat

`access_scope_of`'s binary search showed as 4.3%; the other 10.9 points sat in
`select_unpredictable` and `get_unchecked`, std's binary-search internals,
attributed to themselves. `pollard`'s `expand_inlines: true` surfaced it.

## Measure a struct's cost per byte by padding it

To value a change that shrinks a struct or removes a copy, add a
`[u8; N]` field and measure the slope. `Activation` +256 bytes cost `dispatch`
+0.94%, so its 416-byte per-call copy is worth about 1.5%. That is an upper
bound -- padding also grows the cache footprint.

## Check whether a field is hot before treating it as cold

`Activation::extra` was boxed as a rarely populated field and cost **+1.95%**;
it is populated on every send that binds a name, and giving it the right hasher
won **-8.8%**. Boxing makes an *absence* cheap. A smaller struct is not
automatically a faster one.

## Count failing suites, not lines matching FAILED

`grep -c FAILED` matches both the per-test line and the suite summary, so one
failing test reads as two. `grep -c '^test result: FAILED'` counts suites.
Record `ok=` beside every exit status: a run that never starts reports zero
failures and passes an exit-code check.
