# Pre-4c consolidation: collapse the third ownership copy, and cull boundary prose

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:subagent-driven-development.

**Goal:** remove the two things that caused most of Phase 4b's correction churn, before Phase 4c is planned, while nothing depends on the harness being stable.

**Why now:** the 4b gate deferred this with a reason that has since expired -- "consolidating the ownership harness while ten tasks depend on it is a collision, and it would move every gate figure between here and now."
The phase is closed, the gate is committed and measured, and no task is in flight.
Doing it during 4c would reintroduce exactly the collision the gate was avoiding.

## What the evidence says, so the work targets the right thing

Phasing is not the defect. Facts *about* the phase boundary, written in prose, in more than one place, unasserted, are the defect.

* `corpus/keyword-exempt.txt` carries 796 rows of exactly these facts, **derives** them from the loud message and polices them in both directions. It needed **no** correction across thirteen tasks.
* `INSTRUCTION_WITNESSES`' count comment carried one such fact in prose. It rotted **four times** -- 20, then 19, then 17/16, then 14/14 -- each correction falsified by the next task.

Same data, different medium, opposite outcome. Everything below follows from that.

## Global constraints

* **The emitted loud message text must not change.** 790 of `keyword-exempt.txt`'s 796 rows derive their owner by parsing it out of that message. Any change to the wording or the owner strings turns that file red and would have to be regenerated, which defeats the point of touching this at all. Treat the message as frozen.
* The C++ tree at `/home/moritz/dev/repos/ooRexx` is the oracle and is never modified; neither is `ootest/`.
* No `unsafe`; the workspace sets `unsafe_code = "forbid"`.
* Never `git add -A`, never `git reset --hard`, never force-push.
* Scratch files go only in the session scratchpad, never the repository.
* `rust/CLAUDE.md`'s comment rules govern this work, including the two added at `e96f3435`.

---

### Task 1: Re-grain `owners.rs` to arm granularity and assert `lib.rs` against it

**Files:**
- Modify: `rust/crates/rexx-exec/tests/owners.rs`
- Modify: `rust/crates/rexx-exec/tests/loud.rs`

**The gate priced this at half a day against the general problem, and the general problem is one arm.** Measured before this plan was written:

* `instruction_owner` (`src/lib.rs`) has exactly **one** nested match -- `InstructionKind::Call(call)`, splitting `Named`/`Dynamic`/`Trap` -> `None` from `Qualified` -> `Some("Phase 5")`.
* `expr_owner` has **none**; it is flat.
* `expand_for_witnesses` (`tests/loud.rs`) has **one** real entry, `"Call" => vec!["Call::Qualified"]`, with an identity fall-through.

So the shape mismatch that made a straight equality assertion "not available" is a single variant, forced by the language: qualified calls need the object model, so `Call`'s arms cannot all land in one phase.

- [ ] **Step 1: Confirm the three measurements above against the tree before changing anything.** They are inherited. If any disagrees, stop and report rather than substituting your own.

- [ ] **Step 2: Make the `owners.rs` table arm-grained where `lib.rs` is.** Split the `Call` row into its four arms. Leave every other row alone.

- [ ] **Step 3: Assert `lib.rs`'s `instruction_owner`/`expr_owner` equals the `owners.rs` table directly**, with no expansion step between them. This is the assertion that makes the third copy a derived fact instead of a maintained one.

- [ ] **Step 4: Delete what the assertion makes dead.** `expand_for_witnesses` is the expected casualty; `instruction_arm` may survive if it still has a display role. Determine which by removing them and reading the compiler, not by reasoning.

- [ ] **Step 5: Prove the new assertion can fail.** Change one owner string in `owners.rs` and confirm it goes red; restore. A consolidation whose assertion cannot fail has moved the duplication rather than removed it, which is the outcome the gate predicted and this task exists to beat.

- [ ] **Step 6: Verify the loud message text is byte-identical**, then commit.

---

### Task 2: Cull the boundary prose

**Files:** the six that carry it, in this order: `src/run.rs` (82 lines), `src/lib.rs` (73), `tests/owners.rs` (48), `tests/coverage.rs` (42), `tests/loud.rs` (34), `tests/trace_oracle.rs` (24).

**522 comment lines** across the crate name a phase or use a boundary-time word. Find them with the search that produced that number, then triage **each one** into exactly one of:

1. **Delete** -- it is history ("Task 3 implemented X, so three rows are gone"), or a status fact nothing needs.
2. **Convert to a property** -- the sentence is trying to say something true regardless of where the boundary sits. `tests/support/mod.rs`'s fix is the worked example: "the ten prefixes this crate can emit today" became "every marker `trace_prefix_table` lists, whether or not this crate emits it yet", which cannot go stale.
3. **Keep** -- it is already asserted in the same file, or it cites something *outside* this repository (the C++, the oracle's tables, a measured transcript). These never rotted and are not the target.

- [ ] **Step 1: Produce the triage table before editing anything.** One row per line: file, line, category, and for category 2 the replacement sentence. Commit the table with the work so a reviewer can check the judgement rather than re-derive it.

- [ ] **Step 2: Apply it, largest file first, one commit per file.**

- [ ] **Step 3: Re-run the full verification after each commit.** A cull that changes behaviour has gone wrong; nothing here should move a single test.

**`rust/CLAUDE.md` says never delete a true comment to make a change easier. That rule is not suspended and does not conflict with this task**: deleting valueless true prose *is* this change, not a shortcut taken while making another one, and it was authorised deliberately with the measurement in hand. If you find yourself deleting a comment because it is inconvenient to update, that is the rule biting correctly -- stop and keep it.

---

## Verification for both tasks

`cargo test --workspace`, `cargo fmt --all --check` from `rust/`, `cargo clippy --workspace --all-targets -- -D warnings`, `REXX_CORPUS_GATE=1`, `REXX_ASSERTIONS_GATE=1`, `REXX_KEYWORD_GATE=1`, and `./scripts/mutate-4b.sh`. Each exit status read unpiped.

Baseline at `e96f3435`: **1020 passed / 0 failed**, gates 9/9, 5/5, 7/7, mutation **12 of 12 as declared**.

**The keyword gate is the one that matters here.** It is the instrument that would notice if the loud message text or an owner string moved, because 790 of its rows are derived from that message.

## Not in scope

* Any interpreter behaviour change. This is harness and prose only.
* `read_subset`'s three byte-identical copies (`collect_stress.rs`, `corpus.rs`, `coverage.rs`), which the 4b final review recorded for 4c. Related, but a different file set and a different risk.
* `corpus/README.md`'s stale programs table, also recorded for 4c.
* The twelve deferred minors triaged "fix in 4c" by the 4b final review.
