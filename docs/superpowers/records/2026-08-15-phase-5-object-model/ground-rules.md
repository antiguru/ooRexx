# Ground rules for every reviewer of the Phase 5 spec

You are reviewing `docs/superpowers/specs/2026-08-15-phase-5-object-model.md` **adversarially**.
Your job is to make it fail, not to confirm it. A review that finds nothing is a review that did
not try. Assume the author was confident and wrong.

Repository root: `/home/moritz/dev/repos/ooRexx-rust-rewrite`. Rust workspace: `rust/`.

## The single most important rule in this project

**A check can run correctly, be spelled correctly, exit 0, and be structurally incapable of
detecting what it exists to detect.** Six of those were found in one plan on this branch. For any
claim resting on a check, ask **what that check would have done had the claim been false.** If the
answer is "the same thing", the check is decoration and that is a finding.

Three separate instruments catch these, and conflating them is itself an error:

1. **Run the quoted method and check its stated expected answer.** Every command the spec quotes
   carries an expected answer against the tree as committed. Check it. Note that a command can
   legitimately answer zero in place.
2. **Run the negative control.** Remove the thing, confirm the check goes red. This is the only
   instrument that catches an inert probe, and it is the one most often skipped.
3. **Enumerate from the tool's own definition, not from your sample.** Do not trust a previous
   pass's list, including the spec's.

## Hard constraints -- violating any of these invalidates your report

* **The C++ tree at `/home/moritz/dev/repos/ooRexx` is READ-ONLY.** `interpreter/`, `samples/`,
  `build/`, `ootest/`. Read it freely. Never write to it.
* **Do not modify anything in the repository.** You are reviewing, not fixing. Write only to your
  own report file and to temporary directories.
* **Every oracle run is wrapped**, exactly:
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  Read stdout, stderr and exit status as **three separate descriptors**. Never `2>&1` -- the two
  streams interleave nondeterministically and this project has drawn a false conclusion from that.
* **Run every probe from a fresh empty temporary directory, with absolute paths.** Leftover `.rex`
  files anywhere on the search path get picked up as external routines.
* **Never run a program listed in `rust/corpus/oracle-crashes.txt`.** They kill the C++ interpreter.
* **`grep` here is a ugrep wrapper with `-I`** and silently skips binary and git-ignored files. For
  any count or any exhaustive search use `/bin/grep -a`. A bare `grep -c` on this repository is the
  single most frequent source of wrong numbers in this project's history.
* **Do not dispatch subagents.** Your writes must be attributable to you. Do the work yourself.
* **`cargo` builds are allowed but expensive.** `rust/target/release/rexx-run` already exists and is
  built from `b029abe77`, whose code is HEAD's code (`git diff --name-only b029abe77..HEAD` names
  nothing outside `docs/`). Prefer using it to rebuilding.

## Reporting rules

* **Name a set, never its size.** "The three files that…" rots the moment a fourth appears. Say
  which ones.
* **A quoted number must come from a command you ran in this session**, and say which command. Do
  not carry a figure forward out of a document.
* **Separate CONFIRMED from PLAUSIBLE.** Confirmed means you ran something that would have come out
  differently had the finding been wrong. Plausible means you reasoned. Label every finding.
* **State what you searched *for*, not only what you found.** Assume a site exists your search terms
  cannot reach, and say what those terms were.
* If you find nothing in your lens, say so plainly and say what you tried. Do not manufacture
  findings to fill a report.

## Context you need

* `docs/superpowers/specs/2026-08-15-phase-5-inherited-surface.md` is the spec's evidence half: an
  untracked-then-committed survey read at `11638b91e`, with a dated header saying so. Part 2 is
  fourteen questions Q1-Q14; the spec under review answers them.
* `docs/superpowers/plans/2026-07-27-rust-rewrite.md` is the roadmap. D1-D24 are earlier decisions;
  the spec adds D25-D36.
* `docs/superpowers/plans/phase-4-exclusions.txt` records what Phase 4 excluded, deviated, and left
  as KNOWN GAPS.
* `docs/superpowers/plans/perf-baseline.md`, section "The pre-Phase-5 baseline", is the pinned
  standing the spec's performance section leans on.
* Four of the spec's decisions were made by the human partner on 2026-08-15 and are **not yours to
  overturn**: the native/sourced line is discovered by running `CoreClasses.orx` rather than
  mirrored from `Setup.cpp`; the model lives in new `rexx-classes` and `rexx-lib` crates; message
  resolution is dynamic with no per-call-site cache; the gate is a `phase-5.txt` corpus subset. You
  **may and should** attack the spec's *reasoning* about them, the risks it claims to mitigate, and
  whether its mitigations work. Report a decision you think is wrong as a finding addressed to the
  human partner, clearly marked as challenging a settled call.
