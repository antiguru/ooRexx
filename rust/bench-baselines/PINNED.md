# The pinned builds the performance guard measures against

`pinned/` holds whole `rexx-run` binaries and is **git-ignored** -- a 17 MB artifact per pin does not belong in history.
So the provenance lives here, in a tracked file, and a pin nobody can identify is a pin nobody should trust.

Each is named for the commit it was built from, so the artifact's own name carries its pin and a stale one cannot be silently reused under a name that says nothing.

## `rexx-run-f558ea501` -- the Phase 5b pin, current

| | |
|---|---|
| commit | `f558ea501` (`Give an instance the behaviour its class held when it was built`) |
| built | 2026-09-01 08:53, from a clean tree |
| profile | `release`: `debug = true`, `lto = "fat"`, `codegen-units = 1` |
| rustc | `rustc 1.98.0 (88d9e12ae 2026-08-18)` |
| sha256 | `857787f797e88dd94ab839d7b66f1e08cd1a08d22c9e7978d05cbfd49f70494e` |

Rebuild it with `cargo build --release` at that commit and compare the sha256.

**Why this pin replaced `rexx-run-15a1ffa98`.** The same defect that retired the pin before it, caught
the same way. By 2026-09-01 the 5a pin was **153 commits** stale
(`git log --oneline 15a1ffa98..HEAD -- ':/rust/crates' ':/rust/Cargo.toml' ':/rust/Cargo.lock'`, and
note the `:/` prefix -- the same command without it, run from `rust/`, answers `0` and reads exactly
like a current pin). Task 2 declined its sitting on those grounds and was right to.

**What the guard could not see, and why that is not a defect in any task.** A drift sitting taken
against the old pin before retiring it, recorded in `phase-5b-drift.tsv`, reads (instructions:u):

| axis | tw | ir | | axis | tw | ir |
|---|---|---|---|---|---|---|
| strings | +19.66% | +32.58% | | dispatchclass | +8.27% | +8.48% |
| rexxcps | +13.04% | +16.36% | | compound | +7.85% | +11.26% |
| alloc4c | +9.01% | +13.11% | | arith | +3.23% | +2.34% |
| emptyloop | -1.60% | -2.93% | | varlookup | -0.91% | -1.84% |

Every per-task gate reading across 5a was honest and under half a percent. **A per-step threshold
that always passes still permits unbounded cumulative regression**, and roughly twenty steps of it
compound to the table above. Ruled by Moritz 2026-09-01: this is known, it is not any one task's
defect, and it is handled by **a non-SDD performance iteration round after Phase 5 completes** rather
than by widening any gate now.

**Two things that round must not take on trust.** First, the table above **conflates a compiler
change with the code**: the old pin was built with `rustc 1.97.1 (8bab26f4f 2026-07-14)` and this one
with `rustc 1.98.0 (88d9e12ae 2026-08-18)`. The experiment that separates them is to rebuild
`15a1ffa98` with 1.98.0 and re-run one axis -- `strings`, the worst -- against the 1.97.1-built pin;
until that runs, no share of the drift is attributable to either cause. Second, the figures are
**instructions**, which are deterministic here to about seven significant figures; the `cycles`
column in the same file moves several percent on a do-nothing control and is not readable at this
resolution.

## `rexx-run-15a1ffa98` -- the Phase 5a pin, retired 2026-09-01

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
| `phase-5a-arms.tsv` | `2026-08-17-phase-5a.md` | `rexx-run-15a1ffa98` |
| `phase-5b-arms.tsv` | `2026-08-27-phase-5b.md`, the current one | `rexx-run-f558ea501` |
| `phase-5b-drift.tsv` | none -- a one-off phase-level reading, not a task guard | `rexx-run-15a1ffa98` |

`phase-5b-drift.tsv` is the sitting that retired the 5a pin, and it is **not** a task record: its
`task` column reads `5b-drift` for every row, which is not a task number, and its figures span 153
commits and attribute to nothing. It is kept because a pin retired without a reading of what it was
retired over leaves no evidence for the round that has to act on it. Do not append to it, and do not
compare a task's sitting against it.

The first was called `phase-5a-arms.tsv` until 2026-08-20 and holds that plan's Tasks 2 to 8.
The second starts empty, because a guard's readings are only comparable with others taken against the same pin, and mixing two pins in one file makes the `build` column the only thing standing between a reader and a false comparison.
