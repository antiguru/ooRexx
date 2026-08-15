# Dispatch preamble for Phase 4b tasks

The `task-brief` script extracts only a task's own section, so the plan's Global
Constraints never reach an implementer. I paste them per dispatch. Keeping the text here
means it is one thing to maintain rather than twelve, and a dropped line is a real risk:
Task 0's format check was reported clean having never run, because the constraint that
named it was wrong in the plan and got copied faithfully.

Amend this file when a constraint changes. It is not tracked in git -- the plan is the
tracked record; this is scaffolding for dispatching it.

---

## Two warnings from this phase's own record

Both cost time already, and the same instruction has now been needed three times, which
is why it is standing text rather than something I remember to add.

1. **When you are tempted to explain why something cannot happen, run it instead.** In
   Task 0 an implementer argued at length that a code path was structurally unreachable
   and wrote that into three comments; a one-line program refuted it in ten seconds.
   Earlier the same day the controller made the identical mistake about which trace
   prefixes exist, concluding from an instrument that could never have produced the
   answer. Both were confident, both were wrong, and in both cases the confident
   explanation was more damaging than the gap it explained: an open gap invites scrutiny,
   a comment saying "understood, deferred" stops a reader looking.
2. **Choose probe values that make competing hypotheses produce different bytes.** An
   unset Rexx variable reads as its own uppercased name, so a probe whose expected value
   could also be a derived name proves nothing. A callee setting `x = 1` where the caller
   also has `x = 1` cannot distinguish isolation from sharing.

## Global constraints

* **No `unsafe`.** The workspace sets `unsafe_code = "forbid"`. If the task appears to
  need it, stop and report BLOCKED.
* **The C++ tree is the oracle and is never modified**: `interpreter/`, `samples/`,
  `build/`, `ootest/` are read-only.
* **Wrap every oracle invocation** as
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
  Without the ulimit the interpreter requests gigabytes mid-range and is OOM-killed, which
  has already cost a session and the machine's memory.
* **Read stdout, stderr and exit status as separate descriptors.** Never capture `2>&1` as
  one string and compare it: the interleaving of the trace sink and stdout is undefined by
  design, and comparing them as one string produced two false regressions in 4a.
* Never set `NUMERIC DIGITS` above 1000 in a probe. Never instantiate `.Package~new` on a
  file inside the repository -- it executes that file's prolog. Never run
  `select; when 1 = 0 then; when 2 = 2 then nop; end`, which segfaults the oracle
  (upstream SF #2018).
* A symbol named `x` or `b` immediately followed by a quoted string parses as a hex or
  binary literal, so `say '['x']'` is error 15.3, not concatenation. Use other names.
* Scratch files go in
  `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/`,
  never in the repository.
* **Never `git add -A`.** Stage the exact paths you changed. Never `git reset --hard`,
  never force-push, never use sudo.
* Comments state the contract at the top and the reasoning at the decision point. **Never
  delete a true comment to make a change easier** -- but a comment that states something
  false must be corrected or removed, not hedged. Prefer `--` over an em-dash. There is no
  rule against semicolons in comments in this repository.
* **Check formatting with `cargo fmt --all --check` from `rust/`.** NOT
  `cargo fmt --edition 2024 --check`, which exits 2 on an unknown flag and verifies
  nothing. `cargo clippy --workspace --all-targets -- -D warnings` must be clean.
  **Read every exit status unpiped, and confirm the command actually ran.**
* All 29 Phase 4a corpus programs must still be byte-identical to the oracle when you
  finish. If one moves, that is a defect in your change, not an expectation to update.
* **Commit first, then read the hash back with `git log`, then write it into your report.**
  Do not write a hash from memory; two ledger entries in this project carried invented
  hashes that way.

## Standing facts later tasks need

* **Loud-message format** (Task 0, `0197b360`): `"{name} is not implemented ({owner})"`,
  rc 120. Live: `rexx-exec: CALL is not implemented (4b)`,
  `rexx-exec: a message send is not implemented (Phase 5)`. The owner is
  `Option<&'static str>` and `None` means "implemented in this crate", so moving a variant
  in scope means setting `None`, not a phase string. `loud.rs` asserts stderr **ends with**
  ` is not implemented ({owner})`.
* **Owner data lives in three places**: `tests/owners.rs`, `loud.rs`'s witness rows, and a
  match in `src/lib.rs` (production code cannot reach a test module). The stderr assertion
  is what keeps the third copy honest.
* **`EXPECTED_SUBSET` stays pinned to `phase-4a.txt` alone.** It catches drift in that one
  file's line list. The witness check and `collect_stress` read the union.
* **Two sites in `run.rs`** -- the `DO`/`LOOP` `COUNTER`/`WITH` check and the stem-target
  `DO OVER` deviation -- carry an owner-less message on purpose: they are in-scope
  constructs that are locally blocked, not out-of-scope ones.
