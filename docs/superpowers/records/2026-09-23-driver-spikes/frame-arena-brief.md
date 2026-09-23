# Spike: frame-arena -- statically sized, validated register frames

Read `common.md` beside this file first; it binds you.

## The idea (from Moritz)

A C++ function call is cheap because its locals live on the native stack at
offsets known at compile time. Our IR keeps its registers in a growable heap
`Vec` (`RootSet::temps`) addressed as `temps[frame.0 + index]` behind an
`assert!`, re-deriving base pointer and length on every access, and the Vec can
reallocate whenever a nested activation reserves more, so no pointer into it can
be held across a call.

Each chunk already knows its register count statically (`chunk.registers`,
the allocator's high-water mark, set in `ir/compile.rs`). So:

1. **A frame arena**: register frames allocated from a linked list of
   fixed-capacity blocks that never move. A frame is carved from the current
   block if it fits, else from a fresh block; blocks are retained for reuse.
   A frame's base address is stable for its lifetime.
2. **A validator**: when a chunk is built, check once that every register
   operand of every op is `< chunk.registers`, and make the driver unable to
   run an unvalidated chunk (a type, not a comment: e.g. only a validated
   wrapper exposes the op slice the driver walks).
3. **Encapsulated unchecked access**: a `Frame` handle the driver holds, whose
   `get(i)`/`set(i, v)` read and write through the stable base pointer without
   a bound check, sound because of (2). All `unsafe` lives in one new module.
4. **The collector** walks the live frames in the arena as roots, exactly as it
   walks `temps` now (`roots.rs` near `.chain(self.temps.iter().copied())`).

`push_temp` and the other non-register uses of `temps` may stay on the Vec;
only the driver's register frames need to move. Your call.

## Unsafe exception, this spike branch only

You may use `unsafe` in **one new module** you create, suggested
`rust/crates/rexx-core/src/frame.rs`, every block with a `SAFETY:` note naming
the invariant it relies on. Nowhere else. This exception does not extend to
`plan/rust-rewrite`; if the spike wins, whether the rule changes is Moritz's
decision.

## What to measure, beyond the common contract

* The cost the idea targets is `RootSet::temp_at`/`set_temp` at 28.9
  instructions per clause. Report the head's cost for the replacement accessors
  the same way (inclusive, per clause of `rexxcps`, 20,000,000 clauses).
* **Control**: this edit touches every register access in `run_ops_from`, so
  the whole result is inside the regalloc noise floor unless separated. Build a
  second head where the `Frame` accessors keep a bound check (safe indexing
  through the same stable pointer or slice). Checked-arena vs unchecked-arena
  prices the check; base vs checked-arena prices the arena. Report all three.
* Also try, if time allows, passing the frame base as a local the driver holds
  (so it can live in a register across the region walk) versus re-reading it
  from `self` per access. Say which you measured.

Spike name: `frame-arena`.
