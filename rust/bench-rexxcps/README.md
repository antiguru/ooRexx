# REXXCPS, canonicalised

`rexxcps.rex` is REXXCPS 2.2 with its loop counts fixed.
`rexx-bench-suite` runs it on both interpreters and reports each side's wall time and the clauses-per-second figure the program prints for itself.

## What changed from `samples/rexxcps.rex`, and why

The original **calibrates itself against the clock**: it runs a first trial, times it, and if that came in at or under a second it runs a second trial at a scaled `count`.
Two consequences, and the suite used to carry both.

* **The two sides did different amounts of work.** A fast interpreter finishes the first trial sooner, scales its count differently, and the wall times stop being comparable. The suite's report said so, and quoted each side's `Averaged:` line so a reader could see the asymmetry rather than infer it.
* **A run was not reproducible across days.** The count depended on how fast the machine was at that moment, so two reports taken a week apart could not be compared even on one interpreter.

Here `count` and `averaging` are constants and the trial loop is gone.
Both sides do identical work, so the wall times are directly comparable, and a figure taken today can be compared with one taken next month.
The counts are the ones the original's own calibration settled on for this machine, so the amount of work is close to what earlier reports measured.

The **1000-clause timed body is REXXCPS 2.2's, verbatim**.
That is the part the measurement is about and it is not ours to rewrite.
What is dropped is the trial loop and the count arithmetic; what is kept is the empty-loop calibration, the timing, and the clauses-per-second arithmetic that turns it into the figure the suite parses.

The `Averaged:` line no longer carries the elapsed time.
With the counts fixed that line is the same on both sides and in every run, which is the point: a reader comparing two reports is comparing the work and not the clock.

## Why this is not in `bench-programs/`

`rexx-bench-suite` asserts its axis list against that directory **in both directions** -- a program there with no entry in `AXES` fails, and an entry with no program fails.
A file added there becomes a dimension of the committed baseline, measured as iterations per second read out of an `n = <digits>` line.
REXXCPS is neither: it is one program reported on its own terms, with its own throughput figure, and it has no such line.

`bench-control/` is next door for a different reason of the same shape, and its README says so.

## Why a copy rather than a path into the C++ tree

The constant in `rexx-bench-suite.rs` used to name `samples/rexxcps.rex` in the read-only oracle checkout, on the argument that a copy can drift from the original.
That argument traded one risk for a larger one.
The oracle tree is **machine state that no file in this repository records** -- it was replaced wholesale on 2026-08-20, from a 5.0 interpreter to a 5.3 one, and nothing in the checkout noticed.
A benchmark whose program lives outside the repository cannot be reproduced from the repository.

Drift is not the concern it looks like either, because this file is **not** a copy that is supposed to track the original: it is a derived artifact with a stated provenance, and the original is expected to move without it.
