**OPEN: requested by Moritz 2026-10-19, not yet scheduled.** Replace the IR driver's macros with functions rustfmt can format.

# The IR driver's macros

Moritz, 2026-10-19: the macros in `rust/crates/rexx-exec/src/ir/drive.rs` make the driver
borderline unreadable, and they break rustfmt. Revisit them.

## The macros

Measured at `a5266a662`:
* `region_ops!` spans lines 364 to 1681. It holds the per-op arms of a clause region, so most of
  the op dispatch is a macro body.
* `open_clause!` spans lines 1686 to 1822.
* `clause_region!` spans lines 1828 to 1929.

Their call sites take many identifiers as arguments, because a
macro shares the caller's locals where a function would need them passed or grouped.

## Why rustfmt cannot see them

rustfmt does not format a `macro_rules!` body: it is a token tree. So line width, indentation and
wrapping inside these bodies are maintained by hand. One instance of the damage:
`drive.rs:1092` inside `region_ops!` is a 149-column string literal,
`"op reads register {register} outside the region <long run of spaces> the chunk reserved"`, where
a hand rewrap put the indentation inside the string. The message text is wrong as a result.

## The check, and what it misses

Moritz's check is `cargo fmt --all --check -- --config error_on_line_overflow=true`, run from
`rust/`. At `a5266a662` it exits 1 with 62 overflow errors: 58 in `rexx-api/src/layout.rs` (a
macro-invocation table), and one each in `rexx-core/src/heap.rs:464`, `rexx-exec/src/dispatch.rs:285`,
`rexx-exec/src/scheduler.rs:2408` and `rexx-exec/src/ir/drive.rs:3252`. The last four are
`matches!` or `let`-`else` lines rustfmt formatted but could not fit.

**The check does not see inside a `macro_rules!` body.** `error_on_line_overflow` reports only
lines rustfmt formatted, and it never formats a macro body, so `drive.rs:1092` is not among the
errors. A gate for this work needs a second instrument, for example a plain line-width scan over
the macro spans, or the removal of the macros, after which the first check covers the code.

## Why the macros exist, and Moritz's reading

Moritz, 2026-10-19: code reached only from a macro is invisible to the IDE (`run_loop_with_header`'s
only call site is inside `region_ops!`), and the macros should go. His reading of the reason: a
macro lets control leave from the middle of a region through the caller's labels, where a function
would need a return value, and that is a poor performance trade.

What the tree shows:
* `region_ops!` breaks to labels its caller owns (`'cold`, `'region`, `'park`) and continues
  `'region_loop`. `clause_region!` and `open_clause!` are each expanded at two call sites in the
  driver, so the macros also share one body between them.
* The `'cold` comment at `clause_region!` says a `Result<RegionEnd, Failure>` "costs a 24-byte value
  built and moved per clause". `spills.md` in the driver-spike records measured those return types
  at 24 bytes, returned in memory (sret), and its candidate s1, which moved `Flow` out of the
  return value, cost +1.1% on rexxcps.
* No record measures a macro against an `#[inline(always)]` function returning an enum. After
  inlining, LLVM sees the same control flow, so the sret cost need not appear. That comparison is
  the first spike to run.

## The work

* Spike first: replace `region_ops!` alone with an `#[inline(always)]` method answering
  `enum RegionExit { Next(u32), Cold(Result<RegionEnd, Failure>), Park(Park) }` (or similar), and
  A/B it with callgrind on `nop`, `assign`, `rexxcps`, `interpret` and `sayloop` against the
  current head. Then the other two macros.
* Turn the macros into functions or methods, grouping the shared locals into a struct where the
  argument list would be long. Earlier rounds measured outlining arms with `#[cold]` at +0.94% and
  one function per op at +10% on rexxcps (`docs/superpowers/records/2026-09-23-driver-spikes/`), so
  `#[inline(always)]` functions are the likely shape.
* `run_loop_with_header` (`run/loops.rs:1083`) is reached only from `region_ops!`'s `Op::LoopRun`
  arm, after `flat_loop_start` answers `Fallback`, which it does only for a labelled `DO` block
  (`LoopKind::Simple`, `run/loops.rs:1738`). Its repeating arms (`Forever`, `Count`, `Controlled`,
  `Over`, `With`) look dead. This is read, not run: confirm with `unreachable!()` in each arm under
  a debug corpus gate and suite run, then reduce it to the labelled block.
* Make `cargo fmt --all --check -- --config error_on_line_overflow=true` exit 0, including
  `layout.rs`'s table, and add it to the gates.
* Fix the `drive.rs:1092` message string.
