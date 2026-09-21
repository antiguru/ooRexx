# CREXX design comparison, 2026-09-17

A read-only comparison against `https://github.com/adesutherland/CREXX`, Adrian
Sutherland's from-scratch Rexx in C with a compiler to bytecode and a VM. The
clone, the build and the probe programs were in session scratch on a tmpfs and
are gone; the report and the derived candidate list are here because nothing
else records what was looked at or why three of the four techniques did not
transfer.

| file | what it is |
|---|---|
| `crexx-comparison-report.md` | the full comparison: their architecture, their measurements, and the probes run against both interpreters |
| `2026-09-17-crexx-derived-candidates.md` | the candidates derived from it, each with the run that would size it, plus the two later revisit notes |

## The comparability rule this produced, which is the part worth keeping

No CREXX benchmark is comparable to `rexxcps` or to any of this project's axes,
because every figure was taken on a program their compiler type-checked end to
end: no send, no method lookup, no class-library call, no `INTERPRET`, no `DROP`.
`INTERPRET` and `DROP` are commented out of their scanner, and a variable may not
be retyped, which is what makes their typed opcodes possible and makes the
resulting numbers describe a different language.

The one figure worth carrying across is their **internal** A/B, `rxtvm` against
`rxbvm`: same tree, same build, same machine, same program, isolating one
technique with everything else held fixed.

## Status of each candidate, as of 2026-09-21

* **A, per-send method cache**: open, and narrowed rather than settled. It
  executes a recorded revisit condition, since D28 declined a per-call-site
  cache and D29 added `Behaviour::version` so D28 would be cheap to revisit. The
  clean per-iteration measurement that decides it has not been taken, and
  whoever takes it has to establish which of the two hash symbols belongs to the
  method dictionary rather than assuming from the type name.
* **B, moving ops off the AST re-entry path**: open, and may be subsumed by A.
  `Op::Exec` is `INTERPRET` and has to stay an AST re-entry, because the
  instruction it runs does not exist until run time.
* **C, fusing a comparison into the branch**: open, and gated on an enumeration
  of every comparison operator's path to `logical()` that has never been run. It
  is item 4 of the performance to-do.
* **D, an ahead-of-time-linked library image**: rejected. Their `rxvme` costs
  about 580M instructions to print `1`, against ordinary `rxvm`'s 3.8M and the
  C++ oracle's ~15M, so packing the library in did not make it cheap.
* **E, their hand-tiered handler panel**: rejected twice, and the second
  rejection is the informative one. Their evidence does not isolate outlining;
  and the principled version available here, cold arms moved to one
  `#[inline(never)]` non-generic helper, costs retired instructions on three
  axes while leaving 64-way I1 unmoved. `dispatchclass` fits in 32 KiB and drops
  to zero misses at 64-way, so its misses are conflict rather than capacity, and
  shrinking the driver was never the lever for them.
