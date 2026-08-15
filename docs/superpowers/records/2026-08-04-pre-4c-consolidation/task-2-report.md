# Task 2: cull the boundary prose

**Status: complete.** Base `f9cfb060`, head `8c9321c2`, tree clean.

| commit | file | hits | D | C | K |
|---|---|---:|---:|---:|---:|
| `fd0bcee6` | the triage table | -- | -- | -- | -- |
| `b77e3c0a` | `src/run.rs` | 82 | 10 | 60 | 12 |
| `5916a5f4` | `src/lib.rs` | 71 | 11 | 41 | 19 |
| `0fb06ca6` | `tests/coverage.rs` | 38 | 4 | 13 | 21 |
| `574bbf5a` | `tests/trace_oracle.rs` | 24 | 2 | 13 | 9 |
| `514051f1` | `tests/owners.rs` | 22 | 8 | 10 | 4 |
| `8c9321c2` | `tests/loud.rs` | 20 | 11 | 4 | 5 |
| | **the six** | **257** | **46** | **141** | **70** |

Net over the six code commits: **342 insertions, 480 deletions, -138 lines.**
The boundary-prose count across `rust/crates/` falls **476 -> 300**; within the
six files, **257 -> 81**.

## Verification

Run in full after **every** commit, each exit status read unpiped:

| | baseline | after each of the six |
|---|---|---|
| `cargo test --workspace` | 1020 / 0 | 1020 / 0 |
| `cargo fmt --all --check` | 0 | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | 0 |
| `REXX_CORPUS_GATE=1` | 42 of 42, 9 passed | 42 of 42, 9 passed |
| `REXX_ASSERTIONS_GATE=1` | 4224 of 4259, 5 passed | 4224 of 4259, 5 passed |
| `REXX_KEYWORD_GATE=1` | 100 of 896, 713 of 1773, 7 passed | identical |
| `./scripts/mutate-4b.sh` | 12 of 12 as declared | 12 of 12 as declared |

**Nothing moved a single test**, which is the bar the plan set.

Two structural checks beyond the gates, because "nothing moved" is the claim
most worth being able to fail:

* **Zero non-comment lines changed**, per file and over the whole range.
  `git diff -U0 ... | grep -v '^[+-]\s*\(//\|/\*\|\*\)'` counts **0** across all
  six commits, so no string literal was touched by accident.
* **The loud message text is byte-identical.** `owned_message`'s two `format!`
  literals hash the same at `f9cfb060` and at head. That is the constraint 790
  of `keyword-exempt.txt`'s 796 rows depend on, and the keyword gate reporting
  its baseline figure is the second, independent witness.

## The search, re-derived rather than inherited

The plan's 522 comes from

```
grep -rnE '^\s*(//|/\*|\*)' --include='*.rs' rust/crates/ |
  grep -Ei '\b(4a|4b|4c|Phase 5)\b|\btoday\b|\bcurrently\b|\bnot yet\b|\bso far\b|\bfor now\b|\bas of\b'
```

which returns exactly **522** at `e96f3435` -- the commit whose `rust/CLAUDE.md`
records that number -- confirming it is the same search. At `f9cfb060` it
returns **476**; Task 1 removed 46. The four files whose per-file count differed
from the plan's figures are exactly the four Task 1 edited.

## Five sentences were false, not merely stale

Each was checked against the tree rather than read, and each is a case where
the prose had rotted past "out of date" into "wrong":

| where | the claim | why it is false |
|---|---|---|
| `run.rs:1733` | "the other two arms of `rexx_parse::Call` stay loud" | `Call::Trap` has a real arm (`exec_condition_trap`) and `owners.rs` lists it `InScope`. One arm stays loud. The same header also omitted `CALL ON`/`CALL OFF` from what the match handles. |
| `run.rs:3229` | "`USE ARG` and `ARG()` ... both are still loud" | `USE ARG` is implemented. The next paragraph in the same comment already said so. |
| `run.rs:5671` | "Today that is `run_fragment` alone; Task 3's `CALL` is the next" | `resolve_and_run_call` seals too, at `run.rs:3450`. |
| `trace_oracle.rs:261` | "the two prefix operators 4a implements, `+` and `\`" | Three `PrefixOp` variants are in scope (`Plus`, `Minus`, `Not`). Two is what the *witness file* covers. |
| `owners.rs:213` | "the five that still fail loudly" | Four are loud, and the marker sat above `VariableReference`, which is in scope. Outside the measured set, but on a line being edited; corrected rather than left. |

## `run.rs:4107` -- corrected as far as it can be, and flagged

The comment justified leaving `indent_offset` unrestored on the raising path
with "a raise in 4a is always fatal (**no `SIGNAL ON`/condition trapping exists
yet**)". Trapping exists, so the justification no longer holds.

I tried to measure the consequence rather than reason about it, and **could
not**, for a documented reason. The only producer of a non-zero `indent_offset`
is the absorbed-`WHEN` false branch (`run.rs`, `self.indent_offset = 4`), and
that path is a **deliberate, documented deviation**: this crate does not route
a false absorbed plain `WHEN` to `OTHERWISE` at all, because the oracle
segfaults one line away from that shape (SF #2018) and probing it is explicitly
out of scope per `phase-4-exclusions.txt`'s standing rule. My probe confirmed
the deviation -- ours goes to `exit` where the oracle goes to `otherwise` --
which means no program reaches `run_otherwise` with a trap armed, so the stale
offset has no observable path today either way.

So the sentence is narrowed to what is true and self-supporting -- "a raise
**that is not trapped** is fatal, so nothing runs afterward to see a stale
value" -- and the open question is recorded here rather than asserted in the
comment. **Whoever closes the absorbed-`WHEN` deviation should re-check this
line**: at that point a trapped condition could reach `run_otherwise`, and
whether the offset needs restoring becomes answerable.

`lib.rs`'s `indent_offset` doc carries the same shape of reasoning about
`END`'s 7.3 being fatal. It is not in the measured set (no phase word, no
boundary word), so it is untouched, but it has the same dependency.

## What "keep" turned out to mean

70 of 257 lines are kept byte-identical, and the plan's two keep-criteria did
not cover the largest group. **About half the remaining hits across the crate
are not boundary prose at all**: "the frame currently executing", "the DIGITS
currently in force", "a target that is not currently unset", "not yet asked" --
these are *run-time state*, and the search cannot distinguish them from a claim
about the phase boundary. They were never the defect and are counted in the
522.

Two more keep-classes, both real:

* **A phase name inside a message contract.** `"naming \`4c\`"` and the quoted
  `"routine \"NAME\" is not implemented (4c)"` are the emitted bytes, which
  `loud.rs` pins with `ends_with` and `keyword-exempt.txt` parses. The comment
  saying so is the reason the spelling cannot drift, so deleting it would
  remove the only prose defending a frozen string.
* **A filename that contains a phase.** `phase-4a.txt`, `phase-4b.txt`,
  `mutate-4b.sh`, `2026-07-30-phase-4a-executor-design.md`. The artefact is the
  referent, not the boundary.

## One thing moved rather than being deleted

`loud.rs`'s witness array carried the reason `4b` stays a valid owner string in
`SPLIT_TABLE_PHASES` even though no row uses it. That is a non-obvious decision
about `owners.rs`'s constant, sitting in the wrong file, at the end of a block
that is otherwise pure history. It now sits on the constant, stated as a
property: **a name is here because the split table names it, not because a row
currently uses it**, and dropping a name that owes nothing right now would turn
a later reassignment into a test failure with no defect behind it.
Added in `514051f1` (owners.rs) before being deleted in `8c9321c2` (loud.rs), so
no commit in the range is missing it.

## Scope: the 219 lines in the other 38 files

The plan's Task 2 lists six files under **Files:**, in the order it gives them,
one commit each; that is what is edited. The remaining **219** hits across 38
files are triaged at file grain in the last section of the committed table and
are **not** edited.

Of those 219, **roughly 110 are genuine targets** of the same three kinds. That
110 is an estimate from the triage, not a counted figure -- the counted figures
are the per-file hit totals, which are `src/error.rs` 21,
`tests/assertions.rs` 19, `src/trace.rs` 18, `src/eval.rs` 16,
`tests/corpus.rs` 14, `src/activation.rs` 14, `tests/keyword_assertions.rs` 13,
`src/stem.rs` 12, `src/roots.rs` 11, `src/queue.rs` 10, and a tail of 28 files
with fewer than 10 each. The balance of each file's hits is the run-time-state
and committed-artefact keeps described above; `src/roots.rs` and
`src/settings.rs` are almost entirely those.

**Recommendation:** worth a follow-up task, not worth blocking 4c. The
concentration is gone -- the six files held 54% of the total and now hold 27% --
and none of the remaining 110 is in the ownership harness that caused 4b's
correction churn. `src/queue.rs` and `src/activation.rs` are the two most likely
to go actively false when 4c lands, since both describe `PULL`/`QUEUED()` and
`::routine` dispatch as not existing.
