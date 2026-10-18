**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** a test-harness path like arity-harness-ignores-target-dir; dispatch/library/tests.rs:26 canonicalizes ../../../build/lib.

# Queued: the suite passes only in a checkout that has built `build/lib`

Found 2026-09-16 by the `run/tests.rs` split task, as an aside it first called a
spurious artifact and then withdrew that word after measuring. Pre-existing, not
caused by any file-split work.

## The measurement

Fourteen tests in `rexx-exec` fail in any checkout that has not built the
worktree-local extensions. The mechanism is one thing, not fourteen: ten reach it
through `worktree_library_directory()` in `dispatch/library.rs:791-796` and four
through an inline copy of the same three lines in `run/tests/routines.rs`. All do

    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../build/lib").canonicalize().expect(..)

`CARGO_MANIFEST_DIR` is baked in at compile time and `canonicalize` requires the
path to exist, so the call panics with `NotFound` before any test body runs. The
dependency is not on the path string but on whether
`<checkout-root>/build/lib/librxregexp.so` has ever been built in that checkout.
That is the second, worktree-local build of the extensions, never the oracle's
own `build/`, a documented decision (the D5 amendment,
`docs/superpowers/specs/2026-09-14-phase-8-native-api.md:103-111`). `build/` is
git-ignored, so `git worktree add` never materialises it.

Checked rather than assumed: none of the six sibling worktrees
(`ooRexx-5i-gates`, `ooRexx-gates`, `ooRexx-gates-b`, `ooRexx-ir-spike`,
`ooRexx-step4-base`, `ooRexx-vm-spike`) has that file. Only the primary checkout
does.

## Why it matters

This project builds pinned worktrees at specific commits to verify gates, and
three of the six are named for that. A suite that passes only in the primary
checkout means every such verification either carries fourteen failures nobody
attributed, or was never run there. It is the same shape as the gate table C
defect closed the same day: a green reading that is a property of one directory,
invisible to every run made in it.

## What the work is

Not "make the tests skip". A test that quietly skips is a check blind to its own
subject, and fourteen silent skips in a gate verification are worse than fourteen
panics.

The shape to aim for: one precondition, asserted once where the suite can say so,
that names the build command a fresh checkout must run, instead of fourteen
`NotFound` panics that read like unrelated failures. Whether that precondition
builds the artifact, or refuses with an actionable message, is the open design
question. The gate policy in `rust/CLAUDE.md` should then say that a checkout
must satisfy it before its gate readings mean anything.

## Extent, not yet derived

`rexx-api/tests/{load,invoke,context}.rs` carry the identical pattern, confirmed
by grep and not by running. The count beyond `rexx-exec` is unknown. Derive it by
running, not by grepping, before the work is scoped: the first task is to
enumerate every test that cannot run in a fresh checkout, and the enumeration is
the deliverable, not a sample of it.
