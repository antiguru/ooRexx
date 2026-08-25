# Task 21, fix round 1

Review: `.superpowers/sdd/2026-08-17-phase-5a/task-21-review.md`. Both verdicts CHANGES REQUIRED.
Read the whole review; this brief is the work list, not a substitute for it.

**The review's "What I verified that held" section is long and is not up for revision.** The replay,
the lock, the eighteen corpus rows on both engines, the three controls, determinism, the pin and
almost every citation were all checked against the tree and the oracle and hold. Do not redo them.

I independently reproduced both criticals before writing this. C1: oracle prints `tagged` then `1`,
crate prints `a RexxContext` then `0`, both rc 0. C2: oracle rc 163 with `93.974` under the
`DEFINEMETHODS` frame, crate rc 0 printing `survived`. Both are silent wrong answers your commits
introduced by turning a loud refusal into an answer.

## Must fix

**C1. `.context` is a fresh object per evaluation.** Adding `Primitive::Context` turned on every
Object-inherited method for that receiver, so `~objectName`, `~objectName=` and `~identityHash` now
answer from a throwaway. Cache one `RexxContext` per activation the way `package_objects` caches a
package; if an activation has nowhere to root one, refuse those three loudly instead of answering
from a throwaway. Either way `Interp::context_object`'s doc at `environment.rs:675`-`:686` must stop
saying the difference is unobservable -- both of its premises are false as of your commit. Add the
corpus row the stable identity makes available.

**C2. `~defineMethods` reads an unbuilt directory as an empty one.** `.local`'s entries are the
`unbuilt` table `hash_entry_read` refuses per name; `native_keys` reads the map and does not ask.
Consult the same knowledge one level up: refuse loudly if the scope holds any `unbuilt` entry.
Correct D7's sentence in your report -- being able to walk a map is not being able to read the
collection.

**M1. `~inherit`'s non-class position reports the first argument's error.** Drop the
`class_receiver` conversion for the position and let `inherit_at` do the identity search; a non-class
never matches and falls out as `98.945`. `ClassGraph::inherit_at`'s own doc already has this right.
Add the corpus row the refusal shape lacks -- its absence is why this shipped.

**M2. Two doc comments assert the deferral this task removed:** `dispatch.rs:940`-`:944` and
`lib.rs:648`-`:653`, both saying `MethodDict` models no removal and `.Stem` is therefore unbuilt.
The conclusion still holds (a stem value still has no receiver arm, still loud); only the reason is
wrong. `/bin/grep -a "models no removal"` over `crates/` finds both in one pass. **Run that grep
rather than fixing the two I named** -- the review found these by searching, and the search is the
control, not the list.

**m1, m2. Four citations off the line they name:** `ClassClass.cpp:826`-`:828` -> the call is
`:831`-`:832`; `:963` -> `:961`; `:986` -> `:987`; `Setup.cpp:1216` -> `:1218` (`:1216` is blank).
Re-verify each with `/bin/grep -n` before writing the replacement.

**m3. Two comments name a set's size** in `native_classes_wiring.rs` ("the nine names", "the six
names") where the citation already names the set. Your own commit made this rename elsewhere.

**m4. Record `hasUninitDefined` in D4's undistinguished list** (`ClassClass.cpp:852`, `:854`-`:857`).
No behaviour change; a reader of D4 would otherwise think the `.nil` arm was fully analysed.

**m6. `drop_method_object`'s doc reads as though the object is released.** `roots.rs` has no
`remove_global`, so it is not. Say what actually happens.

## Report-only

**m5.** Section 7 says six axes match Task 20 to six decimals; `arith` moves one unit in the sixth
decimal, so it is five. And the "task 20" column draws from three different sittings in the TSV. The
perf conclusion is accepted and no new sitting is owed -- correct the prose.

## Also

Concern 2's improvement is worth taking: add in-crate assertions that `Loud::rexx_package_classes`
and the source-text refusal actually fire, so something notices if they become answers.

## How to close

All five gates from `rust/`, and re-run the two reproductions above on both engines. Any control you
invert, `cp` the file to your scratchpad first and restore from that copy -- never `git checkout --`.
Append to your existing report rather than rewriting it; I have already read it and verified parts.
