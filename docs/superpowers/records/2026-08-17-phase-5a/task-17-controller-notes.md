# Task 17: what the controller measured before dispatch

Every crate-side and oracle-side reading the brief quotes was re-measured at `08965a822`,
from a fresh empty directory, absolute paths, three descriptors read separately, both sides
bounded. **All four reproduce exactly.** They are current, not inherited.

| program | oracle | crate, both engines |
| --- | --- | --- |
| `.local~MYTHING` set, `.environment~MYTHING` set, `say .MYTHING` | rc 0 `from local` | rc 120 |
| `say .environment~local~class` / `say .environment~hasMethod("LOCAL")` | rc 0 `The Directory class` then `0` | rc 120 |
| `say .methods~class` / `~z` / `["Z"]` / `~q` / `~hasMethod("Z")` with a floating `::METHOD Z` | rc 0 `The StringTable class`, `a Method`, `a Method`, `The NIL object`, `0` | rc 120 |
| `say .METHODS` / `say .ROUTINES`, no floating directive | rc 0 `.METHODS` then `.ROUTINES` | **rc 0, byte-identical** |

**The refusals come from two different routes, which the brief does not say.** The first two
programs stop at `rexx-exec: method "UNKNOWN" of class "Directory" is not implemented (Phase 5)`.
The third stops at `rexx-exec: a message send to one of the interpreter's own objects is not
implemented (Phase 5)`. So the environment-order work and the `.METHODS` work do not unblock each
other, and a fix that clears one leaves the other refusing. Plan for both.

The fourth row is the agreement to preserve. It is green today on both engines and must stay green;
it is the control that catches an over-eager `.METHODS` resolution swallowing the literal spellings.

**Prerequisites confirmed present in the tree**, not just named in the plan:
`crates/rexx-exec/tests/environment_seam.rs` exists, `mod env_seam` is at
`crates/rexx-exec/src/environment.rs:95`, and `StringTable` is already in `rexx-classes`'s registry
and native classes. Task 11's Directory `~put` landed. Nothing here is a stub to be built first.

**The seam obligation is real and this task is the one that can break it.** `environment_seam.rs`
pins an `env_seam::admit(` call, an `env_seam::directory(` call, and a bound on what lives inside
`mod env_seam`. It fails loudly, so it is an obligation to state in the report rather than a hole.

## The seam assertion, stated exactly, because this task can trip it

`tests/environment_seam.rs`'s `the_seam_module_holds_only_the_items_it_is_designed_around` strips
comments from `mod env_seam`'s body and then pins, by keyword: `struct ` twice, `fn ` four times,
and `impl `, `const `, `static `, `mod `, `macro_rules!`, `trait `, `union `, `enum ` at zero each.
A sibling assertion forbids `derive` anywhere in the module, on the ground that a `Copy` or `Clone`
clearance would let one trip through the seam serve two lookups.

Its companion, `directory_lookup_passes_through_exactly_one_chokepoint`, pins one `env_seam::admit(`
call and one `env_seam::directory(` call across the crate's sources, and
`the_scan_can_tell_a_present_token_from_an_absent_one` is the control that keeps the scan from
passing vacuously.

**If your change adds an item to that module, this fails loudly with the module body in the message.**
That is the designed behaviour. Do not bump the pinned number to make it pass. Either keep the new
work outside the module, or state in the report why the seam now needs another item and what stops
the second one from being a second chokepoint -- the whole point of the bound is that a reader can
settle that by reading the module.
