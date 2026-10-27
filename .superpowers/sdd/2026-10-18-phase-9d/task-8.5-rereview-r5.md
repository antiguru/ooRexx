# Task 8.5 re-review, fix round 5

Reviewed `7f8ebd872..f3c91f7e3` (code `2cda11d12`). HEAD was built from the checkout (only `progress.md`
modified) with `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/claude-1000/t85-rr5-target memcap 8G cargo build
--release -j 4 --bin rexx`, exit 0. Its `.text` section is byte-identical to the measured
`cg-9d/t8.5/rexx-head-2cda11d12` (`objcopy -O binary --only-section=.text`, md5 `a1b237cf...` both), so the
probes ran on that pinned binary; earlier answers come from `rexx-head-6a8290bb6` (round 4),
`rexx-head-4a6321376` and `rexx-base-6e0bf5e5b`. Every crate run is `memcap 2G timeout -k 5 20` in a fresh
`mktemp -d` under `t85-rereview5-scratch/`; stdout, stderr, rc and peak RSS (`/usr/bin/time -f %M`) compared
separately. The oracle ran only acyclic programs (`chains.rex` at 1000 links of each kind, `h1.rex` at 2000
and 20000, `h2.rex`). Probe paths below are relative to `t85-rereview5-scratch/`; round 4's probe set is
copied to `r4p/`.

## Verdict

**CHANGES REQUESTED.** Critical none, Important 1, Minor 1. New oracle defects none.

## Verdicts on the fix brief

1. **A chain of `Stem` subclass instances is walked once: ADDRESSED.** Each conversion, rendering and
   nested forward is linear in the chain; but the outermost forward's walk makes a forward that stops after
   one hop cost a walk of the whole chain (Important 1, a new problem from round 4 that round 5 kept).
   * `p/chains.rex KIND N` builds an acyclic chain ending at `7` and times 17 positions alone (`length()`,
     concatenation, arithmetic, comparison, `~string`, `~length`, `~request('STRING')`, `~makestring`, a
     tail, `datatype`, `.array~of(first)~toString`, a compound read through `c. = first`,
     `.context~variables`, `parse var`, `value()`, `IF`, `trace r`). Kinds: `ref` (`v1 = >v2` ... `vn = 7`),
     `sub` (Stem subclass instances), `mix` (instance, reference to a stem, instance, ...), `stemref`
     (`s1. = >s2.` ...). Every position answers as the oracle at 1000 links. 20000 to 100000 links scale by
     5 to 9 on every position, and at 100000 head is within a factor of 3 of `4a6321376` everywhere
     (largest: `mix` comparison 0.191 s head, 0.169 s `4a6321376`; `stemref` `~length` 0.227 s, 0.230 s;
     `sub` arithmetic 0.098 s, 0.112 s). `6a8290bb6` is rc 137 (killed at 25 s) on `sub`, `mix` and
     `stemref` at 100000.
   * The report's own probes on the pinned binary: `subchain2.rex`, `fwd.rex` and `subchain3.rex` as the
     report records.
   * `cargo test --release -p rexx-exec --lib -- reference cyclic long_chain`: `19 passed; 0 failed`,
     including `a_long_chain_of_stem_subclass_instances_is_walked_once` and
     `a_reference_forwards_to_a_stem_whose_default_is_cyclic`.
2. **Forwarding through a reference takes the reference end: ADDRESSED.** Round 4's `r4p/cyc2/d01.rex`
   (`q~items`), `d03` (`q~allIndexes~items`) and `d05` (`q~hasIndex(1)`) print `0` / `end` on head (`6a8290bb6`:
   `SYNTAX 11.1`; `4a6321376`: `0` / `end`). `d04` (`q~put(1, 'a')` then `say s.a`) is `SYNTAX 11.1` on head
   and `6a8290bb6`, which is right: `put` stores at index `a`, `s.a` reads tail `A`, unset, so it converts the
   cyclic default. `ref_items.rex` matches the report. Every other file of round 4's set (`r4p/cyc/c01` to
   `c60`, `cyc2/d02`, `d06` to `d14`, `acy/a1` to `a4`) is byte-identical in stdout and stderr (outside the
   run directory's name) and rc on head and `6a8290bb6`.
3. **The acyclic answers that moved are named: ADDRESSED.** `moved.rex` on head: `q~string` and `r~string`
   `an Array`, `t.~string`, `u~string`, `say q`, `say r` the items joined, `trace r` of `x = r` and `x = q`
   `>>> "an Array"`; on `4a6321376` all items joined and both TRACE lines `"one` / `two"`. That is what the
   report says. rexxref `instrc.xml:3697-3699` is the quoted sentence. The `Object~string` citation
   `fundclasses.xml:3474-3486` is the section's heading and figure; the quoted sentence is at
   `fundclasses.xml:3487-3490` (not a finding).

**Hop guard (the report's last paragraph).** A cycle made under the outermost walk ends in 11.1 and never
overflowed the native stack: `p/f1.rex` (an `UNKNOWN` reached through `s.~foo` builds `t[] = t` and sends
`t~bar`), `f2` (`x. = >x.; return x.~bar`), `f4` (the same under a trap in the method: `caught 11.1`, `end`),
`g1` to `g7` (`x.~request('foo')`, `x.~send('BAR')`, `.message~new(x., 'BAR')~send`, under `trace i`, a
reference first, `t[] = >t`, `x.~start('BAR')~result`) are all `SYNTAX 11.1` on head and `6a8290bb6`, and
rc 134 `thread 'rexx-interp' has overflowed its stack` on `4a6321376` (f1, f2, f4, g1). HALT interrupts:
`f5.rex` (each iteration traps that 11.1 inside the method) under `timeout -s INT -k 5 5` ends `Error 4.1`,
rc 124, as `6a8290bb6` does; round 4's `halt.rex` under `timeout -s TERM -k 5 3` prints
`HALT HALT after 88647`, rc 124, and `halt.rex 20000` peaks at 36408 KB. The cost is Minor 1.

**Perf: confirmed.** `r5final/` holds 99 head and 99 base `.vg` files (33 programs, three rounds); every
head `Command:` line names `cg-9d/t8.5/rexx-head-2cda11d12`, whose `.text` matches my HEAD build, and every
base line `rexx-base-6e0bf5e5b`. In `r5/callgrind.txt`, libc subtracted, the largest rises are fibfunc
`+0.0213`, fibcall `+0.0209`, pingmsg `+0.0006` and define `+0.0001`; every program is under +0.05%, the
spread columns at most `0.0019`. No benchmark program forwards through a chain of stems, so the table
cannot see Important 1.

## Issues

### Critical

None.

### Important

1. **A forward that one hop answers walks the whole rest of the chain, so a loop over a chain's nodes is
   quadratic.** `Interp::forwarded` (`value.rs:1273`) runs `chain_is_cyclic(receiver)` at every outermost
   forward, which walks every link to the chain's end, however soon the message is answered.
   `p/h1.rex N`: a chain `A1 -> B1 -> A2 -> B2 ... -> 'leaf'` of Stem subclass instances, where class `b`
   defines `foo` and `a` does not, so `Ai~foo` forwards one hop to `Bi`; the program sends `foo` to each `Ai`
   once and sums the answers.

   | N | head `2cda11d12` | `6a8290bb6` | `4a6321376` | base | oracle |
   |---|---|---|---|---|---|
   | 2000 | 0.0717 s | 0.0734 s | 0.0014 s | 0.0014 s | 0.0007 s |
   | 10000 | 1.78 s | 1.79 s | 0.0067 s | 0.0069 s | |
   | 20000 | 7.10 s | 7.19 s | 0.0150 s | 0.0136 s | 0.0122 s |

   Every side prints `sum 200010000` at 20000. The time grows with N squared on head and round 4 and with N
   at `4a6321376`, base and the oracle, so this is a regression on a shape that worked at `4a6321376`,
   introduced by round 4's `forwarded_value` and kept by round 5. A reference into such a chain (`q = >s.`
   with the chain on `s.`'s default) reaches the same outermost walk. Fix: do not walk at the outermost
   forward; test for a cycle among the nested forwards instead, for example a walk from the receiver when
   `stem_forwarding` reaches a power of two (a forward answered after k hops then walks O(k log k) links at
   most, and a cycle made under the outer forward is found at the next power of two rather than by the
   stack, which also answers Minor 1).

### Minor

1. **A cycle made under the outermost forward costs the whole native stack.** Nested forwards skip the walk
   (`stem_forwarding > 0`, `value.rs:1273`) and stop only at `stack_exhausted`, 32 MB short of the 512 MB
   interpreter stack. Peak RSS and wall time per 11.1, head against `6a8290bb6`: `f1` 580176 KB 0.44 s
   against 20128 KB 0.13 s; `f2`, `g2` to `g7` 582 to 583 MB and 0.46 to 0.58 s against 20 MB and under
   0.08 s; `g1` (the forward is `REQUEST`, which also allocates a text per hop) 1152024 KB 1.66 s against
   20268 KB 0.06 s, and under `memcap 1G` it is OOM-killed (rc 137) where it answered 11.1 in 20 MB at round 4.
   `f5.rex 20` (twenty such traps) takes 6.50 s and 589780 KB here, 0.03 s and 20008 KB on `6a8290bb6`. The
   memory is flat across iterations and HALT interrupts, and `4a6321376` aborted on these shapes with a
   stack overflow, so none of the Important criteria holds. The same holds for any program whose code runs
   inside a stem's forward (an `UNKNOWN` or `REQUEST` reached through a stem), not only a cycle built for
   the purpose. Fix: Important 1's.

## Out of scope

* A chain of references (`chains.rex ref 100000`) is up to 3 times slower than `4a6321376` on `length()`
  (0.0164 s against 0.0060 s), concatenation and `parse var`; `6a8290bb6` is the same as head, so it dates
  from round 4, and it stays linear.
* A plain `.stem~new` taking another stem as its default (`t[] = u`) is `SYNTAX 98.976` on every side and
  on the oracle.
* An intermediate Stem subclass that defines `makestring` and `string` (`p/h2.rex`) does not change a
  conversion or `~string` of the instance before it: `leaf` on every side and the oracle.

## Queue candidates

None new. The stem's own string value with an array default stays queued by the controller.

## New oracle defects

None. The oracle ran no cyclic program; on the acyclic ones it answered `leaf`, `7` and the sums above.
