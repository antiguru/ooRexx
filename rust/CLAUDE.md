# Working in this crate tree

A clean-room Rust reimplementation of ooRexx.
Correctness is defined **differentially**: a change is right when its output matches the C++ interpreter byte for byte, not when it looks right.

Each rule below has already cost this project a session, a wrong measurement, or a shipped defect.

## The oracle

* **The C++ tree is read-only.** Never modify `interpreter/`, `samples/`, `build/`, `ootest/`.
* **Wrap every oracle run** as
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
  Without the `ulimit` the interpreter requests gigabytes mid-range and is OOM-killed; that has already ended a session and taken the machine's memory with it.
* **Read stdout, stderr and exit status as three separate descriptors.** Never capture `2>&1` as one string: the trace sink and stdout interleave undefinedly by design, and comparing them together produced two false regressions.
* Run `rexx-run` from `rust/`.

## Probes

* **Run every probe from a fresh empty subdirectory you `mkdir` yourself**, never the scratchpad root.
  The scratchpad is on the oracle's **external-routine search path** and holds hundreds of stale `.rex` files, so a probe calling any unresolved name finds one and executes it.
  Measured: the same three-line program reported `44.1 rc 212` from the root and `43.1 rc 213` from a clean directory.
* **Use absolute paths for every redirect.** A relative redirect inside a subshell that has `cd`'d writes where you did not intend; that produced three false mismatches against correct code and left stray files in the repository.
* **Never instantiate `.Package~new` on a file inside the repository** -- it executes that file's prolog and has written untracked files into the tree.
* **Never run `select; when 1 = 0 then; when 2 = 2 then nop; end`** -- it segfaults the oracle (upstream SF #2018).
* **Never set `NUMERIC DIGITS` above 1000** in a probe.
* A symbol named `x` or `b` immediately followed by a quoted string parses as a hex or binary literal, so `say '['x']'` is error 15.3 rather than concatenation. Use other names.

## Gates

* **`cargo fmt --all --check`** from `rust/`.
  **Not** `cargo fmt --edition 2024 --check`: `cargo fmt` has no `--edition` flag, that spelling exits 2 before doing any work, and a task once ran it and reported formatting clean.
  `--edition 2024` belongs to the standalone `rustfmt` binary.
* **`cargo clippy --workspace --all-targets -- -D warnings`** must be clean.
* **Read every exit status unpiped, and confirm the command actually ran.** A pipeline reports the last command's status, and a misspelled flag exits non-zero before doing any work -- which reads as failure in one context and is silently skipped in another.
* **`cargo test <name>` exits 0 when it matches nothing** -- `0 passed; 0 failed; N filtered out`, status 0. So a mutation harness that runs a test by name and reads only the status **cannot tell "passed" from "does not exist"**. Measured here after a harness restored a file with `git checkout --`, silently discarding uncommitted work, and then reported STAYED GREEN for a test that had ceased to exist. Assert that the run count is non-zero, restore from a copy rather than from git, and never let a mutation harness be the only thing that decides a test is load-bearing.

## Repository hygiene

* **No `unsafe`.** The workspace sets `unsafe_code = "forbid"`. If something appears to need it, stop and say so.
* **Never `git add -A`.** Stage the exact paths you changed. Never `git reset --hard`, never force-push.
* Scratch files live under the session scratchpad, **never in the repository**. Check `git status` before committing and remove anything stray.
* **Commit first, then read the hash back with `git log`**, then quote it. Hashes written from memory have gone into records wrong twice.
* Comments state the contract at the top and the reasoning at the decision point.
  Never delete a true comment to make a change easier -- but a comment that states something **false** must be corrected or removed, not hedged.
  Prefer `--` over an em-dash. There is no rule against semicolons in comments here.
* **A comment may say what the oracle does, what the C++ does, and what was measured. It may not say how many call sites there are, what every other site does, or what a gate currently totals.**
  Those change without the sentence being reread. **If such a claim is load-bearing, assert it in a test; if it is not, delete it.**
  Measured across one task: comments pointing at *immutable* referents -- oracle bytes, C++ citations, benchmark numbers -- needed **zero** corrections over four commits, while every false comment was a *mutable repo aggregate*. Six shipped, and each arrived in a commit correcting the previous one; corrections are this defect's habitat, because correcting a comment means writing about the code's context, which is exactly what you cannot see from the line you are editing.
* **A claim must not be falsifiable by the act of committing it.** Never quote a search command inside the file it searches: a comment reading ``grep -c 'foo()' this_file.rs`` -> 1 makes the answer 2 the moment it lands. Assertions are the one mechanism here with a clean record -- they are re-run, and a mutation proves they can fail.
* **Phase status is a mutable aggregate, and it is the one that actually rots.** "Owned by 4c", "not yet implemented", "in scope since Task 3", "the ten prefixes this crate emits today" are all facts about where the implemented/not-implemented boundary sits, and that boundary moves every time a task lands. The evidence is a contrast, not a count: the 796-row `corpus/keyword-exempt.txt` *derives* these facts from the loud message and polices them in both directions, and needed **no** correction across thirteen tasks, while `INSTRUCTION_WITNESSES`' one-line count comment stated the same kind of fact in prose and rotted **four times** -- 20, 19, 17/16, 14/14 -- each correction falsified by the next task. Same data, different medium, opposite outcome. (A first version of this rule cited "522 comment lines" as the size of the defect. That figure was the union of *names a phase* and *uses a word like "currently"*, and about half the second group turned out to be run-time state -- "the frame currently executing", "not yet asked" -- which was never the defect and cannot be told apart by grep. A count of a mutable in-repo aggregate, conflating two populations, inside the rule forbidding exactly that.) **Assert the boundary or do not write it down.** Where a property holds regardless of where the boundary sits, state the property: `tests/support/mod.rs`'s normalisation covers every marker `trace_prefix_table` lists, whether or not this crate emits it yet, and that sentence cannot go stale.
* **Comments say what the code does, not how it got there.** No task numbers, no "used to", no account of what moved or shrank. History belongs in the commit message and the SDD ledger -- both searchable, neither re-read on every edit. A measurement justifying the *current* design stays, because it is evidence for the contract: "measured, the oracle declines this too" earns its place. "Task 3 implemented `Call::Named` and Task 7 implemented `Call::Trap`, so three rows are gone and this expansion shrank with them" is history, and it was fourteen lines wrapped around a function with two arms.
* **An exhaustiveness claim is only as good as the enumeration behind it, and only if that enumeration lives outside this repository.**
  "All nine accepted `TRACE` letters" survived review because `TraceSetting::parseTraceSetting` enumerates them in one C++ switch -- an external, checkable referent. "All three ways a label is reached" was false (an internal function call is a fourth) because **nothing** enumerates routes; it was a universal quantifier wearing the same clothes as the safe kind.
  The refinement, earned separately: *something enumerating it* is not enough. `WITNESS_PREFIXES` genuinely enumerates, and "`>>>` appears in every witness below" still rotted -- because that `const` is in this repo and changed when a witness landed. **The axis is whether the enumeration can change without the sentence being reread.** Outside the repo -- the C++, the oracle's own tables -- cite it and prose is fine. Inside the repo, assert it.
  When neither is available, state the property and its citation rather than a count: "a label echoes when the `LABEL` instruction is executed, per `RexxInstructionLabel::execute`" is checkable; "the three ways" is not, and "the four ways" would only be the same mistake with a bigger number.

## Method

Four habits, each earned the expensive way.

* **When tempted to explain why something cannot happen, run it instead.** Six confident "X cannot be reached" claims have been wrong here, every one a sound inference from a *true* premise missing a second premise nobody knew to look for. More care produces the same answer with more confidence; only running separates them.
* **A test that cannot fail is a defect.** Eight have shipped, two of them fixes for earlier such findings. For every assertion ask: what degenerate implementation satisfies this, and would deleting its subject leave it green?
* **Pair a refusal with its adjacent success.** Three times the test that made a fix *correct* was not the failing case but the neighbouring passing one, because that is what pins the rule to the property you think it is rather than to something coincidental.
* **When a plan or brief is wrong, correct the plan -- not the message that carries the work.** A correction written into a dispatch, a review summary, or a chat reply is read once and then lost, because briefs regenerate from the plan, reviewers review against the brief, and a later fix round hands a *fresh* worker the same brief path. Measured here: a `trace r` divergence found during Task 7 was recorded only in a review summary, was absent from Task 9's brief when it arrived, and survived solely because it happened to be re-measured. The same applies to anything one task discovers that another task owns -- write it into the receiving task's text at the moment it is found. If a correction is worth making, it is worth making where the next reader will see it, and if the plan itself contradicts the tree or itself, fix the plan and say in it what was removed and why.
