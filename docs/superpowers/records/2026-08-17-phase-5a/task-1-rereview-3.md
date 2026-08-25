# Task 1, fix round 3 -- re-review

Scope `63a49c9b9..56caf9d08` (`7a2f6a414`, a controller plan-correction touching only
`docs/superpowers/plans/2026-08-17-phase-5a.md`, ignored per instruction; `56caf9d08`, the
implementer's round-3 commit, the only one reviewed here). Standard is
`task-1-fixround-3-brief.md`'s rulings on `task-1-rereview-2.md`'s D1-D4. Round 2's approved work is
not reopened.

## Verdict

**APPROVE.** All four ruled findings are closed the way the brief asked. No new defect.

## D1-D4

* **D1 -- CLOSED.** `corpus.rs:663`: `"106 of 106 matching"` replaced with `"a fully-matching
  corpus"`. No count survives anywhere in the sentence or its neighbours (collapsed-comment diff,
  below). The replacement is also true: it no longer names *any* specific headline, so it cannot be
  falsified by the `105 of 106` a non-finish actually prints -- the second defect the ruling
  described is gone along with the first, not just relabelled.
* **D2 -- CLOSED.** `support/oracle.rs`: the sentence "The pre-channel code named the path in
  exactly this case (`.join().unwrap_or_else(...)`)" is deleted. Read the surrounding paragraph
  whole: "A disconnected channel means the sender end was dropped without sending -- the reader
  thread panicked before it could report its buffer -- which `recv_timeout` distinguishes from an
  ordinary `Timeout` and this function does too, rather than reading both the same way. Folding it
  into a `TimedOut` classification instead would trade a named harness bug for a silent
  misclassification." Complete argument, nothing lost by the strike-test.
* **D3 -- CLOSED**, and the scope is correct. `ir_dual_oracle.rs`: `let mut in_file = 0usize;` is
  declared *inside* `datadriven::walk`'s outer closure body, before `file.run(...)`. Checked
  `datadriven` 0.9.0's own source (`walk_exclusive`, `src/lib.rs`): the outer closure `f` is invoked
  once per file inside `for file in test_files(...) { run(file); }`, so `in_file` is a fresh
  stack binding on every call -- it cannot be shared or leaked across files, and needs no explicit
  reset beyond the `let`. `in_file += 1` happens beside `checked += 1` in the same inner closure, so
  `checked` still increments once per stanza; its consumer, `assert_eq!(oracle.invocations(),
  checked, "every stanza must reach the oracle exactly once")` at `:248`, is untouched by the diff
  and still reads `checked`, not `in_file`. `CASE_DIR` (`ir_dual_cases/`) has no subdirectories --
  confirmed by listing it -- so `filename` is unique across the whole walk and `{filename}#{in_file}`
  cannot collide between files either. The unchanged doc comment above the function, "`label`
  identifies the stanza," stays true.
* **D4 -- CLOSED, both halves.**
  * `ir_dual_oracle.rs`'s comment no longer claims "the shape every oracle-invoking harness in this
    crate uses for the same event" -- reworded to describe only this call site.
  * `input_oracle.rs:558-563` (new): `assert!(!did_not_finish(&cpp), ...)` before the byte
    comparison in `an_unreadable_console_is_end_of_input`, same shape as the sibling fix at `:449`
    (checks `cpp`, the only side that goes through the oracle's own deadline machinery -- the `rust`
    side here is a bare `Command::output()` with no analogous wrapper, unchanged by this diff, so
    checking only `cpp` is not an omission). Ran under the gate: `an_unreadable_console_is_end_of_input
    ... ok`.

## Verification I ran

| command | result |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --release --workspace` | exit 0; `106 of 106 matching -- REPORT MODE, NOT THE GATE`; 0 `test result: FAILED` anywhere |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | exit 0; `mode: STRICT ... 106 of 106 matching`; `oracle_deadline.rs`: `14 passed; 0 failed`, `finished in 10.01s`; `an_unreadable_console_is_end_of_input ... ok`; `every_recorded_expectation_is_still_what_the_oracle_produces ... ok`; 0 `FAILED` anywhere |
| collapsed-comment-block diff (fixed a `mawk`-vs-`\s` bug in my own first attempt -- see below), `63a49c9b9` vs `56caf9d08`, all four changed files | exactly the four hunks D1-D4 describe, nothing else |
| `datadriven` 0.9.0 source read directly (`walk_exclusive`) | confirms the outer closure runs once per file |
| `/bin/grep -ra 'REXX_PHASE_GATE\|CLOSED_PHASES' rust/crates rust/corpus` | empty -- phase-gate command still vacuous, as every prior round found |
| `git status --short` | clean -- nothing left edited |
| `which memcap` | present at `/home/moritz/.local/bin/memcap` |

**A note on my own method, since the brief warned about exactly this class of miss:** my first
collapsed-comment-diff pass used `awk` with `\s` in the regex and came back clean on all four files
-- including on `corpus.rs`, where I could see with my own eyes in the raw diff that the sentence had
changed. `mawk` (this host's `awk`) does not support `\s`; the pattern silently matched nothing, so
every comment line was skipped rather than compared, and an empty diff read exactly like "no
history/cardinality drift" instead of "the check did not run." Rewriting the character class as
`[ \t]` fixed it and reproduced the four expected hunks. Recording this because a check that returns
a clean result without having examined its subject is the more dangerous failure than one that
returns a dirty one.

## New defects

None found. I read every changed comment line in full (not filtered through a keyword list) against
the four files' collapsed diff above, checked the global constraints doc's set-cardinality and
history-framing rules against each, and confirmed the round touches only `tests/` and one `docs/`
file (via the plan-correction commit, ignored per instruction) -- no `src/` of `rexx-exec`,
`rexx-core`, `rexx-classes` or `rexx-lib`, so no performance-guard sitting is owed and the report is
right not to run one. No `unsafe` introduced. No ownership-list item touched.

## What I could not check

* **Gate commands 5 and 6** (`REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast`,
  and the `REXX_PHASE_GATE=5a` variant). I did not run either myself; I ran fmt, clippy, and both
  release-profile test commands, all reproducing the report's figures exactly (`106 of 106`,
  `10.01s`, 0 `FAILED`), and independently confirmed `memcap` is present and the phase-gate grep is
  still empty, which is what the report's claims about those two commands rest on.
* **A reader thread actually panicking** to observe `support/oracle.rs`'s `Disconnected` branch
  fire. Unchanged from round 2's re-review: unreachable through any path I can name without editing
  the harness, so this round's comment deletion is verified by reading, not by triggering the branch.
* **Whether `progress.md` needs the moved before-state measurement.** Unchanged from prior rounds;
  still the controller's call.
