# SDD ledger -- plan: docs/superpowers/plans/2026-08-04-pre-4c-consolidation.md

Branch `plan/rust-rewrite`, worktree `/home/moritz/dev/repos/ooRexx-rust-rewrite`.
Runs between Phase 4b (closed at `a7f1a020`) and Phase 4c, deliberately, while nothing
depends on the ownership harness being stable.

BASE: `222d438d`.

## Task 1: complete

**`Task 1: complete`** at **`f9cfb060`**. Commits: `337217da` (re-grain + assert),
`f9cfb060` (three false ownership sentences the re-graining exposed).

**Verified by me at `f9cfb060`, isolated worktree:** **1020 passed / 0 failed**, fmt 0, clippy 0,
`REXX_CORPUS_GATE=1` 9/9, `REXX_ASSERTIONS_GATE=1` 5/5, `REXX_KEYWORD_GATE=1` 7/7,
`mutate-4b.sh` **12 of 12 as declared**, tree clean. Net **-67 lines** (263 insertions, 330
deletions).

**Both reconciliation functions are gone**: `expand_for_witnesses` and `instruction_arm`, zero
occurrences. That is the outcome the 4b gate predicted was unavailable.

**The falsification, run by me rather than taken from the report, and the first attempt tested the
wrong thing.** Changing an owner string in `owners.rs` turned two *pre-existing* committed-
expectation tests red -- which proves nothing about the new equality. The assertion lives in
`loud.rs`, not `owners.rs`. Testing the direction that matters, `lib.rs` drifting from `owners.rs`:

    rexx_parse::Call::Qualified { .. } => Some("4c")      // was Some("Phase 5")

    Call::Qualified (Phase 5): stderr does not end with " is not implemented (Phase 5)":
      "rexx-exec: CALL is not implemented (4c)\n"

`owner` comes from `owners.rs`; the suffix comes from the running executor, built from `lib.rs`'s
own answer. **Cross-checked per row at runtime with no hand-maintained table between them.** Both
files restored byte-identical and re-verified green.

**The assertion's own comment is honest about its blind spot**, which is worth keeping: it covers
loud variants only, so an owner wrongly written as `None` for something loud is caught, while a
phase wrongly written *onto* an implemented variant is unreachable data no assertion can see. It
names `corpus.rs`'s differential as the instrument covering that direction.

**Note for the next probe of this kind:** "the mutation went red" is not "the new assertion went
red". Name the test you expect to fail before running it, and check it is the one that did.

## Task 2: complete

**`Task 2: complete`** at **`8c9321c2`**. Seven commits: `fd0bcee6` (the triage table, committed
before any edit), then one per file largest-first -- `b77e3c0a` `run.rs`, `5916a5f4` `lib.rs`,
`0fb06ca6` `coverage.rs`, `574bbf5a` `trace_oracle.rs`, `514051f1` `owners.rs`, `8c9321c2`
`loud.rs`.

**Verified after every one of the six**, each exit status read unpiped: **1020 passed / 0 failed**,
fmt 0, clippy 0, `REXX_CORPUS_GATE=1` 42 of 42 (9 passed), `REXX_ASSERTIONS_GATE=1` 4224 of 4259
(5 passed), `REXX_KEYWORD_GATE=1` 100 of 896 / 713 of 1773 (7 passed), `mutate-4b.sh` **12 of 12 as
declared**. Nothing moved a single test. Tree clean.

**257 lines triaged: 46 deleted, 141 converted, 70 kept.** Net **-138 lines** (342 insertions, 480
deletions). Crate-wide boundary prose **476 -> 300**; the six files **257 -> 81**.

**Two structural checks, because "nothing moved" is the claim worth being able to fail.** Zero
non-comment lines changed in any of the six commits, and `owned_message`'s two `format!` literals
hash identically at `f9cfb060` and at head -- the frozen message the keyword gate derives 790 rows
from.

**The search was re-derived, not inherited**: it returns exactly 522 at `e96f3435`, which is what
identifies it as the same search, and 476 at `f9cfb060`.

**Five sentences were false rather than stale**, each checked against the tree: `run.rs:1733`
(`Call::Trap` is in scope, so one arm stays loud not two), `run.rs:3229` (`USE ARG` is
implemented), `run.rs:5671` (`resolve_and_run_call` seals too, `run.rs:3450`),
`trace_oracle.rs:261` (three prefix operators are in scope, not two), `owners.rs:213` ("the five
that still fail loudly" is four, and sat above an in-scope row).

**One open item, deliberately not closed here** -- `run.rs:4107`. Its justification ("no `SIGNAL
ON`/condition trapping exists yet") is false, and the consequence is **not measurable today**: the
only producer of a non-zero `indent_offset` is the absorbed-`WHEN` false branch, which this crate
deliberately does not route to `OTHERWISE` (SF #2018 segfaults the oracle one line away, and
probing it is out of scope by standing rule). Confirmed by probe. The sentence is narrowed to "a
raise **that is not trapped** is fatal", which is true and self-supporting. **Whoever closes the
absorbed-`WHEN` deviation must re-check that line**, and `lib.rs`'s `indent_offset` doc has the
same dependency on `END`'s 7.3 being fatal.

**Scope**: the six files the plan lists under **Files:**. The other 38 files' **219** hits are
triaged at file grain in the committed table and left alone; **roughly 110 are genuine targets**.
Recommended as a follow-up, not a 4c blocker -- `src/queue.rs` and `src/activation.rs` are the two
most likely to go actively false when 4c lands.

**What the triage found that the plan did not anticipate**: about half the crate-wide hits are not
boundary prose at all. "the frame currently executing", "the DIGITS currently in force", "not yet
asked" are run-time state, and the search cannot tell them from a phase claim. A phase name inside
a frozen message contract and a filename containing a phase are the other two keep-classes.

## Task 1 review + fix round: complete

Review: **spec PASS, quality PASS with findings** -- 3 Important, 4 Minor, none weakening the new
assertion. Fixed at `7900224e`, with one residual I corrected myself at `3e1bbb4a`.

**The reviewer settled the gate's prediction with a measurement, not an argument.** Same coherent
`owners.rs`-side mutation on both trees: at `f9cfb060` only the new assertion goes red; at
`222d438d` the whole pre-task suite is **1020 passed / 0 failed on a table that contradicts
`lib.rs`**. The old witness carried its own `owner:` literal and was checked against
`SPLIT_TABLE_PHASES` and `lib.rs`, **never against `owners.rs`**. So the assertion adds coverage,
not merely the ability to fail.

It also strengthened Step 6 beyond what I checked: `git diff 222d438d f9cfb060 -- src/lib.rs`
contains **no non-comment line**, so the frozen message could not have moved *by construction*.

**I3 is the finding worth carrying forward, and it corrects something I relayed.** The paragraph
the task added to bound its own claim said `corpus.rs` covers the uncovered direction. Measured
false: giving `InstructionKind::Say` an owner leaves the workspace **1020 / 0 green**, `corpus.rs`
included. `corpus.rs` covers a different risk -- an implemented construct starting to emit a gap
message. **A disclaimer that overclaims is worse than no disclaimer**, because it stops the next
reader looking. The replacement states the measurement, says nothing needs to cover it because the
value is unreachable, and adds an exception the finding did not know about: `Do`/`Loop` *do* reach
that function through `run_loop`'s two edge cases.

**I1 was in a live file**, `rust/corpus/phase-4b.txt`'s header -- read by `corpus.rs` and
`coverage.rs`, and the standing rule for what may be listed in the 4b subset. The sweep commit that
existed to catch exactly this class did not reach it.

---

## Verification integrity: clippy was reporting green without linting

**Found while verifying the fix round.** `cargo clippy --workspace --all-targets -- -D warnings`
reported exit 0 many times this session, including at `a7f1a020` and `96ad0d15`; re-running the
identical command at those same commits **fails**. `rexx-parse/tests/scanner.rs:995` trips
`clippy::byte_char_slices` and has done since **2026-07-30**, Phase 4a -- five days before this
session, and unrelated to any 4b or pre-4c work.

Cause not established: cargo reusing a per-crate result for an unchanged crate, or a toolchain that
moved mid-session. **It does not matter** -- both produce a command that runs, exits 0, and has not
linted the code.

Fixed at `a7b84d4a`, rule added at `4095f612`: **run the lint from a clean target directory at every
phase boundary; treat a same-session green as provisional.** Baseline then re-established from a
genuinely clean target (3.1 GB removed, full rebuild): clippy **exit 0, zero warnings**.

**Scope of the doubt, stated so nobody over-corrects:** test counts, `fmt`, the three gates and the
mutation script all re-ran real work every time and their outputs varied with the tree. Clippy was
the one instrument that could silently no-op, and only clippy results from this session before
`a7b84d4a` should be treated as unverified.

**Also the fourth time today a count including non-data misled me.** `grep -c 'Owner::Phase("4b")'`
returned 1; the hit was the **assertion** filtering for it, not a data row. Same family as the
`call raiser` comment and the `522` conflation.

---

## PRE-4C CONSOLIDATION COMPLETE

**HEAD `3e1bbb4a`.** Verified: **1020 passed / 0 failed**, fmt 0, clippy 0, `REXX_CORPUS_GATE=1`,
`REXX_ASSERTIONS_GATE=1`, `REXX_KEYWORD_GATE=1` all green, `mutate-4b.sh` **12 of 12 as declared**.

**Task 1**: `owners.rs` arm-grained, `lib.rs` asserted against it per row at runtime,
`expand_for_witnesses` and `instruction_arm` deleted. Net **-67 lines**. The third copy is now a
derived fact.

**Task 2**: boundary prose culled in the six scoped files, triage table committed **before** editing.
Net **-138 lines**. Concentration in those six fell from 54% of the crate total to 27%.

**Carried to 4c**, all recorded rather than assumed:

* **219 boundary-prose hits across 38 files were never in my scope** -- I sized the task from the
  concentrated head and treated the measurement as the whole, the same error as 4b's incomplete
  roll-up. Roughly 110 are genuine targets. `src/queue.rs` and `src/activation.rs` are the two most
  likely to go **actively false** when 4c lands, since both describe `PULL`/`QUEUED()` and
  `::routine` dispatch as not existing.
* `read_subset`'s three byte-identical copies, one tested.
* `corpus/README.md`'s stale programs table.
* The twelve deferred minors the 4b final review triaged "fix in 4c".
* The compound-`DO` fix, owned by 4c, in `EXCLUSIONS`.
