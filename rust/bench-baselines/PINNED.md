# The pinned builds the performance guard measures against

`pinned/` holds whole `rexx-run` binaries and is **git-ignored** -- a 17 MB artifact per pin does not belong in history.
So the provenance lives here, in a tracked file, and a pin nobody can identify is a pin nobody should trust.

Each is named for the commit it was built from, so the artifact's own name carries its pin and a stale one cannot be silently reused under a name that says nothing.

## `rexx-run-15a1ffa98` -- the Phase 5a pin

| | |
|---|---|
| commit | `15a1ffa98` (`Count cycles and instructions beside every wall time`) |
| built | 2026-08-20 21:22, from a clean tree |
| profile | `release`: `debug = true`, `lto = "fat"`, `codegen-units = 1` |
| rustc | `rustc 1.97.1 (8bab26f4f 2026-07-14)` |
| sha256 | `141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b` |

Rebuild it with `cargo build --release` at that commit and compare the sha256.
A rebuild that does not reproduce the hash is not this pin, and the guard's readings are not comparable with anything taken against it.

**Why this pin exists at all.** The guard compares a task's build against a pinned one to say the task made nothing slower.
The pin it carried before this one, `rexx-run-pre-phase-5`, was built at `b029abe77` on 2026-08-15 -- *before* the optimisation body `d6870a358`..`53674c4fe` landed, `b029abe77` being an ancestor of `d6870a358`.
Measured against that, the guard read "no Phase 5a task may be slower than the tree was before eighty optimisations", with all of them as headroom to burn.
Nothing was wrong with the control; an external change moved the world out from under it, which is the variant of the defect that reading the control cannot find.

## `rexx-run-pre-phase-5` -- stale, kept, not to be measured against

Built 2026-08-15 at 17:41 at `b029abe77`.
**It is the stale pin described above and must not be used for a Phase 5a sitting.**
It stays because `phase-5a-native-layer-arms.tsv` and a dozen records cite it by that name, and a file renamed out from under a citation is worse than one kept and labelled.

## The two `phase-5a` baselines

| file | plan | pin |
|---|---|---|
| `phase-5a-native-layer-arms.tsv` | `2026-08-15-phase-5a-native-layer.md`, superseded | `rexx-run-pre-phase-5` |
| `phase-5a-arms.tsv` | `2026-08-17-phase-5a.md`, the current one | `rexx-run-15a1ffa98` |

The first was called `phase-5a-arms.tsv` until 2026-08-20 and holds that plan's Tasks 2 to 8.
The second starts empty, because a guard's readings are only comparable with others taken against the same pin, and mixing two pins in one file makes the `build` column the only thing standing between a reader and a false comparison.
