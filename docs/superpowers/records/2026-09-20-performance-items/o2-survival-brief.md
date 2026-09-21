# Does an optimisation found at `-O2` survive into `-O3` with fat LTO?

One question, answered by measurement: **can we explore at a cheaper
optimisation level and trust the result to hold in the profile we ship?**

If yes, exploration gets faster and the comparison against the oracle gets
fairer, since `build/` is `-O2 -g -DNDEBUG` in a shared library and we are
`-O3`, `lto = "fat"`, `codegen-units = 1`, statically linked. If no, that is
worth knowing before anyone reasons from an `-O2` profile again.

## The two profiles

Ours today, `Cargo.toml`:

    [profile.release]
    debug = true
    lto = "fat"
    codegen-units = 1

with `opt-level` defaulting to 3. Build the comparison profile **without
editing `Cargo.toml`** -- add a profile in a way that leaves the tree clean, or
pass the settings through the environment, and say in the report exactly how you
did it. `opt-level = 2`, `lto = false`, `codegen-units` at its default.

## The experiment

Two landed optimisations, each measured under both profiles against its own
parent. Both parents are separated from their commits by documents only, so the
code difference is the commit's.

| commit | what | measured at release |
|---|---|---|
| `725aa8863` vs `bc26d7936` | `Condition` + `JumpUnless` fused into one op | **-1.6747%** on `rexxcps` |
| `26ccef4ee` vs `0e9c9e062` | the driver's ops read through a slice cut to the loop's bound | -0.0794% `rexxcps`, **-0.8643%** `varlookup`, **-0.7501%** `emptyloop` |

Four builds per commit pair is eight builds. **Give every build its own
`CARGO_TARGET_DIR`**, print each binary's sha256, and confirm the measured
binary is the committed tree by comparing `.text` against a rebuild:

    objcopy -O binary --only-section=.text <binary> <out> && sha256sum <out>

A whole-file sha256 cannot do this: `debug = true` puts the target directory's
path into the debug info, so an identical rebuild elsewhere hashes differently.

Measure with `valgrind --tool=callgrind` on the pinned
`rust/bench-rexxcps/rexxcps.rex` and on `bench-programs/varlookup.rex` and
`emptyloop.rex`. Interleave the arms; give the within-build spread.
**`rexxcps` carries about 0.01% intrinsic spread because it renders `TIME()`
into its own output; `emptyloop` and `varlookup` reproduce to 0.0001%**, so a
sub-percent question is decided on the narrow axes.

## The three questions, in order of what they decide

1. **Does the sign survive?** An optimisation that helps at one level and hurts
   at the other would end exploration at `-O2` outright.
2. **Does the magnitude survive, and by how much does it differ?** Say the
   ratio between the two deltas for each commit. A consistent factor is usable;
   a scattered one is not.
3. **Does the profile rank the same candidates?** Take the top functions by
   self cost under both profiles at the same commit, and say whether the order
   and the shares agree. This is the one that decides whether an `-O2` profile
   can be used to *choose* work, which matters more than whether it can confirm
   it.

Also report the whole-program instruction count at each level. If `-O2` without
LTO costs materially more instructions overall, say how much: that is the price
of the cheaper exploration loop.

## What would make me distrust your answer

Two commits is a small sample and both are in the interpreter's dispatch path.
**Say so in the report** rather than generalising to optimisations of every
shape. If you have budget for a third that is not a dispatch change,
`5e765dc5a` against `19a3018a0` -- reading a `PARSE` source in place rather than
copying it, **-0.6761%** -- is the one to add, and it is the most different in
character.

## Constraints

* **Do not edit any tracked file** and do not commit. You have a worktree.
* The suite does not pass in a fresh worktree; known, recorded, and you do not
  need it.
* Scratch under the session scratchpad in a subdirectory of your own. Never `rm`
  with a star glob. Watch disk: eight release builds is tens of gigabytes, so
  delete each target directory by explicit path when its binary is extracted,
  and check `df -h /tmp` before starting if you put them there.
* Do not wait on a `pgrep -f` for your own jobs: the waiting shell's argv
  contains the pattern, so the count never reaches zero and two runs deadlocked
  that way. Append `finished` to a status file and wait on that.

## Report

`.superpowers/sdd/2026-09-22-o2-survival.md`, or `.txt` if your harness refuses
`.md`. **Replies truncate at about 8 KB**: the answers to the three questions
first, the table second, caveats last. Quote the command beside every figure and
give the exit status you observed.
