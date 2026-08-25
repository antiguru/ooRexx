## Global constraints

**Differential correctness.** A change is right when stdout, stderr and exit status match the C++
oracle byte for byte. Oracle wrapper, three descriptors, never `2>&1`, fresh empty directory,
absolute paths. **Never run a program in `rust/corpus/oracle-crashes.txt`.** Bound anything that may
hang with `timeout -s KILL 10` until Task 1 lands the harness's own bound.

**Both engines, every construct, in the task that introduces it.** Never `Op::Generic` with a
promotion to follow.

**The five gate commands**, from `rust/`, at the end of every task:

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --release --workspace
REXX_CORPUS_GATE=1 cargo test --release --workspace
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast
```

**Every program in the corpus matches the pinned oracle, and the count must stay total and grow.** Do
not carry a remembered number for it -- read what the gate prints, and compare that against what it
printed before your change. A task that adds a corpus program adds it to `rust/corpus/phase-5a.txt`
in its own commit.

**There is no standing red and no exception to read past, so the two corpus gate commands exit zero
and "the corpus is green" means what it says.** For part of 2026-08-20 the corpus read 105 of 106 on
`corpus/lang/loop_control_rounding.rex` -- a Phase 4 loop-control divergence this crate owned, not the
oracle -- and `2eed4cad5` fixed it before this plan's Task 1. `corpus.rs` has no skip, exclusion or
known-mismatch mechanism, which is the right shape and is why **any red is a regression**: a task
that finds one stops and diagnoses it rather than matching it against a name it was told to expect.

**The performance guard.** Any task landing code in **`src/`** of `rexx-exec`, `rexx-core`,
`rexx-classes` or `rexx-lib` runs a two-build sitting before reporting done:

```
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-15a1ffa98 \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup \
    --axis dispatchclass --axis bench-rexxcps/rexxcps.rex \
    --rounds 5 --task <task> --commit <commit> --baseline bench-baselines/phase-5a-arms.tsv
```

**This file is not the authority on that command; the plan is**
(`docs/superpowers/plans/2026-08-17-phase-5a.md`, the guard block). This copy carried the six-axis
form after `dispatchclass` and `bench-rexxcps/rexxcps.rex` were added to the plan, and Task 18's
sitting ran six axes because the implementer was handed this file as its binding constraints. The
omission hid nothing -- the contribution arm was flat on both missing axes -- but a guard that can
silently narrow is not a guard. **Check this block against the plan's before running a sitting**, and
if they disagree the plan wins.

**That pin has been rebuilt, and the one this plan was first written with was the fifth instance of
the defect the whole document is being swept for.**
`bench-baselines/pinned/rexx-run-pre-phase-5` -- the name that command line carried until this
round, and the file is still there, built 2026-08-15 at 17:41 -- was built at `b029abe77`, and the
optimisation body `d6870a358`..`53674c4fe` landed after it -- `b029abe77` is an ancestor of
`d6870a358`, checked. So the guard as pinned read *"no 5a task may be slower than the tree was
before the optimisation work"*, and it had that entire body as headroom to burn: a 5a task could
give back every gain in it and the guard would stay green. **Nothing about the control was badly
written -- it fired correctly on the day it was written, and an external change moved the world out
from under it.** That is the variant of the defect a review cannot find by reading the control: the
question is not only *can this fire as written* but *does it still have a live referent*.

**What the replacement is**, stated so it can be checked rather than assumed -- and it exists, which
is what the entry condition at the top of this file rests on:

* a **release `rexx-run` built from the crate source at the commit Phase 5a starts from** -- the crate
  source at `15a1ffa98`, which is the branch head's, built from a clean tree with profile `release`
  (`debug = true`, `lto = "fat"`, `codegen-units = 1`) under `rustc 1.97.1 (8bab26f4f 2026-07-14)`;
* stored as `bench-baselines/pinned/rexx-run-15a1ffa98`, **named for that commit**, so the artifact's
  own name carries its pin and a stale one cannot be silently reused;
* with its provenance in the tracked `bench-baselines/PINNED.md` -- commit, build date, profile,
  `rustc`, and the **sha256 of the binary**,
  `141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`. `pinned/` is git-ignored, so a
  17 MB artifact stays out of history and the thing that identifies it does not. Recording the sha256
  is what `perf-baseline.md`'s counted baseline does for both of its builds and is the fix for this
  whole class of problem;
* and `bench-baselines/phase-5a-arms.tsv` **starts empty against it** -- header row and nothing else.
  The rows it held from the superseded plan moved to `phase-5a-native-layer-arms.tsv`, named for the
  plan that produced them, because **a row measured against the old pin is not comparable with one
  measured against the new one** and a delta across that boundary is the same mistake as a ratio
  across the oracle swap.

**The staleness test, which each task runs before trusting the pin:** the pin's commit is an ancestor
of the task's base, **and** every commit this lists is one this plan's ledger records:

```
git log --oneline <pin-commit>..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml
```

A rebase, or any unrelated crate work landing beneath the phase, puts a commit in that list that the
ledger does not name -- and the pin must then be rebuilt and the baseline restarted rather than
reasoned about.

**This was a `git diff ... is empty` test until Task 6, and the correction is worth keeping because
the reasoning that produced the wrong test was sound.** The objection it was answering is real: a
test phrased over *which commits exist* between the pin and the base fails on every document-only
commit, of which this plan produces many, and would mandate a rebuild that reproduces the same
binary. What the diff form missed is that **the phase's own tasks change the crate source -- that is
what the pin is for measuring** -- so from the first task that touches `src/`, the diff is non-empty
by design and the test can never pass again. Restricting the pathspec does not save it; it only moves
the task at which it starts failing.

Phrasing it over the commits that touch the crate source answers both at once. A document-only commit
never appears in that list, so the original objection is met rather than ignored, and the phase's own
task commits appear and are recognised, because the ledger names them. The pathspec stays wide on
purpose: over-listing costs one commit to recognise, where under-listing hides the foreign commit the
test exists to catch.

**And the rule that generalises it, because this is the second artifact in this plan to go stale
under a control that could not tell: a control that names something outside this document states what
makes it stale and how the task checks that before trusting it.** The artifacts this plan names such
a check for, each with the check:

* **the pinned binary** -- the staleness test above, and the sha256 in `PINNED.md` against a rebuild
  at the pin's own commit;
* **the baseline TSV** -- one file per pin, with `PINNED.md`'s table saying which pin each baseline
  file's rows were taken against. **The `build` column will not do it**: measured, it holds the
  sitting's own labels (`pinned`, `head`, and the comparison rows' `pinned>head`) and reads the same
  against either pin, which is why the file starts empty under a new pin instead of gaining rows;
* **`perf-baseline.md`'s standing** -- cited by exact section heading and not by date, because that
  file carries several 2026-08-20 sections and the superseded ones say so in their own headings;
* **the committed row sets** -- Task 3's both-directions re-derivation check and its D56 revision
  stamp, and Task 2's ledger row for the authority behind each;
* **the oracle behind `oracle_root()`** -- `crates/rexx-exec/tests/parse_version_oracle.rs`, which
  runs `parse version` through both interpreters and compares three descriptors, and
  `crates/rexx-exec/tests/ir_dual_oracle.rs`, which re-reads every `ir_dual_cases` stanza from the
  live oracle rather than from this crate. Both run under `REXX_CORPUS_GATE=1`, so **the two corpus
  gate commands are the check** and no task has to remember to make it. Neither holds a copy of the
  bytes it checks, and **no task adds an assertion that does**: an in-crate `parse version` assertion
  can only compare the recorded string against a second copy of itself, which is the antipattern
  `parse_version_oracle.rs`'s own doc rejects.

**A `file:line` citation is deliberately not on that list.** This plan cites into `rust/` and
`interpreter/` and nothing here can check one; what checks them is the per-task rule under "What I
ran" -- the task that relies on a citation re-reads it before trusting it. Listing them here as
though this document covered them is the decoration this plan's own constraint forbids.

Instrument **`instructions:u`**. Under 1% on an axis is not a finding; at or above 1% the task says
whether it is the change or the layout, with an interleaved control build as the tie-breaker.
**A `cycles:u` figure from this bench is not a result on its own**: across three sittings on this
plan's predecessor the loudest `cycles:u` rows were **+2.020%**, **-5.497%** and **+2.792%** while
`instructions:u` stayed flat to a thousandth every time, and one of those changes added no work to
any executed path. A task touching only `tests/`, `corpus/` or `docs/` runs no sitting and says why:
the release binary the axes measure is byte-identical, so the sitting would measure noise.

**No `unsafe`, and this plan adds no exception.** The workspace lint is **`unsafe_code = "deny"`**
(`rust/Cargo.toml:31`) -- **not `forbid`**, which it was until 2026-08-20, when Moritz replaced it
because `forbid` cannot be overridden by an inner `#[allow]` and that is exactly what an approved
site needs. **So the lint does not make `unsafe` impossible; what stands in its way is
`rust/CLAUDE.md`'s exception process, which is Moritz's decision per site**, with the granted set
recorded as the `#[allow(unsafe_code)]` attributes and asserted by
`crates/rexx-core/tests/unsafe_sites.rs`. The practical rule for a task is unchanged -- stop and say
so rather than reach for it -- but **a comment claiming the lint "rules out" a construct is false**,
and this bullet said `forbid` in three places in the tree before Task 4's review resolved it. A
dev-dependency of a test harness is not covered by the lint and is not an exception either -- say
which crate and why if one is taken.

**Read-only trees, at the paths they are actually at.** The C++ is
`/home/moritz/dev/repos/ooRexx/interpreter/`. **`ootest/` and `oodocs/` are *not* beside it** -- they
are working copies inside this worktree, `ootest/` and `oodocs/` relative to the repository root, and
`/home/moritz/dev/repos/ooRexx/ootest` does not exist. Nothing in this plan writes to any of the
three.

**Both `ootest/` and `oodocs/` are git-ignored**, so like the pinned binary they are machine state
that no file in the checkout records, and the same rule applies: a task that depends on one checks it
rather than assuming it. The check is `svn info`, which prints the revision the stamps below are
written against -- **`oodocs/rexxpg` and `oodocs/rexxref` at r13198, `ootest/` at r13178**, verified
2026-08-21. `oodocs/` has no `svn info` at its own top level; the two subdirectories are separate
working copies and carry it. If either is absent, the reading rows that depend on it are `not done`
and say so -- they are never satisfied from memory or from the correlation record.

**A comment may not name the size of a set.** Name the set; true counts included. Measurements keep
their numbers.

**Where a task is told to "record" a before state, a measurement or a route it rejected, that means
the report and the ledger -- not a comment in the source.** `rust/CLAUDE.md`'s rule is that comments
say what the code does and not how it got there, and the two instructions collide in every task that
measures the state it is about to change. They are not in tension once the split is named: a
measurement justifying the design **as it now stands** earns its place in a comment, because it is
evidence for the contract a reader is about to rely on; an account of what the code did **before**,
or of the shape that was replaced, is history and belongs where history is searchable and is not
re-read on every edit. The test that decides a borderline case: **strike the historical framing and
see whether the sentence still says the same thing about the code as it is.** If it does, the framing
was decoration and goes. Ruled at Task 1, where a doc comment carried both shapes at once.

**Ownership moves in one commit.** `owners.rs`'s five pinned items (`EXPECTED_OUT_OF_SCOPE`,
`coverage.rs`'s `EXPECTED_SUBSET`, `variant_counts_match_the_audited_split`, `loud.rs`'s witness
tables, `lib.rs`'s `instruction_owner`/`expr_owner`) move together, and so do `corpus/bif-exempt.txt`,
`corpus/keyword-exempt.txt`, `corpus/builtin-status.txt`, `assertions.rs`'s `EXEMPT` and
`trace_oracle.rs`'s `PREFIX_COVERAGE` when a row starts passing. **`coverage.rs`'s
`every_in_scope_variant_is_witnessed_by_the_phase_subsets` demands a subset witness the moment a
variant moves.**

**The oracle can be wrong, and this phase is where that is most likely.** Three signals together
before recording a behaviour instead of building it: it is **inconsistent** in a way no rule explains;
**`ootest` does not pin it**, checked at `ootest/` directly rather than through the correlation
record; and the neighbouring code is already upstream-flagged. One signal is not enough. Task 2 is
what makes the second signal checkable.

**Where a task changes what is refused, it says what instrument catches a regression.** The corpus
gate cannot see a clean refusal becoming a wrong answer -- a refusal the oracle does not share is not
expressible as a differential row -- and that defect class survived five fix rounds on the old plan's
Task 8. If the honest answer is "an in-crate test only", the task says exactly that.

**Say what a check could not see.** For any claim resting on a check, state what that check would
have done had the claim been false. If the answer is "the same thing", it is decoration.


**A rustdoc link to an item that does not exist is not a compile error.** `fmt`, `clippy` and the
test suite all pass over `[`Interp::invoke_method`]` when the method is called `enter_method_body`.
Only `cargo doc --workspace --no-deps` sees it, and that is not one of the gates -- its warning would
land among the others already there and fail nothing. So **an intra-doc link is a claim you must
check by other means**: `/bin/grep -n` the item name before you write the link. Task 19 caught one of
its own before committing; nothing in the gates would have caught it after.

**Gates and tree mutations must not overlap in this checkout.** Every agent on this plan works in the
same worktree and shares one `target/`. A reviewer testing a control mutates a source file, runs the
suite, and restores it. **A gate run that lands inside that window reads someone else's tree** -- a
red that reproduces nowhere, or worse a green over a mutated source. Task 19 saw exactly one such red
and could not reproduce it in 26 further runs. Before treating a one-off gate failure as real, check
who else was live; before running a gate, prefer a moment when no other agent is mutating.
