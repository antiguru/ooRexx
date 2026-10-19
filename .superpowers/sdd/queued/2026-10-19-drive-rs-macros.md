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

## The work

* Turn the macros into functions or methods, grouping the shared locals into a struct where the
  argument list would be long. The macros exist for codegen reasons (see
  `docs/superpowers/records/2026-08-12-condition-promotion/vm-comparison.md` section 6.3 and the
  driver-spike records in `docs/superpowers/records/2026-09-23-driver-spikes/`), so each step needs
  a callgrind A/B against the Phase 9 pin. Earlier rounds measured outlining arms with `#[cold]` at
  +0.94% and one function per op at +10% on rexxcps, so `#[inline(always)]` functions are the
  likely shape.
* Make `cargo fmt --all --check -- --config error_on_line_overflow=true` exit 0, including
  `layout.rs`'s table, and add it to the gates.
* Fix the `drive.rs:1092` message string.
