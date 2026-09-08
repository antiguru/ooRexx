# The concurrency measurements behind the research direction

Programs for the tables in
`docs/superpowers/specs/2026-09-08-concurrency-research-direction.md`, sections 1 and 4.
Kept here and **not** in `corpus/`: every one of them is nondeterministic by construction,
which `corpus/README.md`'s determinism rule forbids.

Run against the oracle from a fresh directory:

    ( ulimit -v 1048576; LD_LIBRARY_PATH=<oracle>/build/lib <oracle>/build/bin/rexx FILE )

with `/usr/bin/time -f "%e wall %P cpu"` for the timings. Machine had 32 cores.

| program | what it shows |
|---|---|
| `one.rex`, `ser.rex`, `par.rex` | one activity; two serial; two via `~start`. 0.36 / 0.72 / **1.15 s** |
| `ser4.rex`, `par4.rex` | 1.43 / **1.52 s** |
| `ser8.rex`, `par8.rex` | 2.85 / **3.00 s**, 98% CPU. Eight unguarded methods on eight objects, 32 cores, no speedup |
| `race.rex` | read-modify-write in one clause, 20,000 iterations. Zero lost updates -- but too short for the activities to overlap at a 24 ms time slice, so it witnesses little on its own |
| `race1big.rex` | the same in one clause, 2,000,000 iterations. Zero |
| `race3.rex` | split across **two** clauses, 2,000,000 iterations. Zero, three runs -- and this is the one that matters, because the language does not protect a read-modify-write spanning two sends |
| `interleave.rex` | switch detector, pure computation. **0, 0, 0** |
| `interleave2.rex` | **the positive control** -- the same detector with one `charout` per iteration. **653, 708, 605** |
| `docex.rex` | the reference manual's own default-concurrency example, which interleaves because its loop body is a `SAY` |

`interleave2.rex` is the reason `interleave.rex`'s zero can be believed. Without it a zero is
indistinguishable from a detector that cannot fire.

**Do not add a `GUARD ... WHEN` shape here without reading `corpus/oracle-crashes.txt` entry
7 first**: a `GUARD ON WHEN` whose condition cannot become true blocks the oracle forever,
producing no output and no exit status.

Section 4's allocation counts came from a temporary test calling
`rexx_exec::run_program_collect_every_alloc`, under which every allocation collects and
`Outcome::collections` is therefore an exact allocation count. The programs are four lines
each and are quoted in the spec.
