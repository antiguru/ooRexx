# Task 8.8a: Performance fix round 1

## Confirmed gap

The matched arithmetic median increases by 0.34824337% libc/ld-subtracted against the pre-8.8 baseline, without a spread or output error. Existing-profile attribution records the new outlined `whole_value` adding 53.5 million self instructions while its caller falls by 13.5 million, explaining the net 40 million increase. `arith.rex` computes power with exponent 2 in its 500,000-iteration loop, so it directly exercises the changed conversion. Preserve the overflow and platform-limit corrections while avoiding unnecessary general conversion work for already integral exponent spellings.

## Narrow candidate and contracts

After the existing rounding, zero, precision and integrality guards in `pow.rs::as_whole`, evaluate reusing `Number::plain_integer` with the platform argument precision when the rounded exponent is nonnegative. Keep `whole_value(ARGUMENT_DIGITS)` for the fractional-zero spelling that needs its existing normalization. Prove equivalence of the nonnegative branch under those guards, including the 18-digit ceiling and sign, rather than copying a second accumulation loop or weakening the bounds. A different equally narrow reuse is acceptable if the actual instruction evidence supports it.

Read the original a report and regression tests before editing. Preserve fractional trailing-zero inputs, rounding under DIGITS, negative powers, zero, huge precision and rejected out-of-platform exponents. Add only meaningful boundary coverage missing from those committed tests. Do not broaden this repair into general whole-number conversion semantics or numeric model changes.

## Verification and handoff

The controller grants source/Git ownership only after the preceding reviewed handoff. Preserve a pre-repair runtime pin and revision, commit before verification, and use the common capped foreground workflow. Run the complete rexx-num tests, formatting and affected all-target clippy, plus the existing power corpus/oracle witness where necessary. Retain a release pin and build/source hashes for the controller's matched all-program comparison.

Write `task-8.8a-perf-fix-report.md` beside this brief with the equivalence argument, commands, statuses and source commits. Store durable outputs under `docs/superpowers/records/2026-10-18-phase-9d/t8.8/pow/perf-fix-r1/`. Root owns the all-program comparison and broad final gates, and the repair requires scoped review. Preserve the failed measurements, add no dependency or unsafe site, and do not spawn agents.
