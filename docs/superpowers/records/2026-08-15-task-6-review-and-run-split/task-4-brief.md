### Task 4: the false attribution in the plan, and three comments naming the wrong function

Review findings **I3** and **N1**. Both are claims about which code does what. Neither is a
behavioural change. Verify each against the tree before rewriting it.

#### I3: "repeating loops with a real header were never wrong on this route"

`docs/superpowers/plans/2026-08-14-pre-phase-5-defects.md`, in Task 6's record, around `:464`. The
reviewer measured at `1f4176b47` against the oracle, both engines, the same `raise ... return` route
with one requeue in the loop body:

| program | oracle | `1f4176b47` |
|---|---|---|
| `do zi = 1 to 1 / zr = raiser() / end / say 'after' zr` | `h1 5` `h2 6` `after 5` `h3 7` | `h1 5` `h2 6` `h3 6` `after 5` |
| `do 1 / zr = raiser() / end / ...` | same | same divergence |
| `do while zn < 1 / ... / zr = raiser() / end / ...` | `h1 7` `h2 8` `after 5` `h3 9` | `h1 7` `h2 8` `h3 8` `after 5` |
| `do forever / zr = raiser() / leave / end / ...` | `h1 5` `h2 6` `after 5` `h3 8` | `h1 5` `h2 6` `h3 6` `after 5` |

All four agree at `01f8010b7`. So three of the four shapes the bullet names by name
(`do zi = 1 to 2`, `do while`, `do 2`) *were* wrong on this route, and the stated reason, that
`run_repeating` "drains the queue there", is the wrong mechanism: `run_repeating` drains the queue
the body left at the *next* header clause, and the requeue the handler left behind was still being
taken by the step boundary. The third change is what closed them.

The fourth name is wrong differently. `do label zl` with no header is `LoopKind::Simple`
(`crates/rexx-parse/src/instruction.rs:960-975`) and never reaches `run_repeating` at all. Measured:

```
oracle                     h1 3 / h2 4 / body / after 5
1f4176b47, both engines    h1 3 / body / h2 5 / after 5
01f8010b7, both engines    h1 3 / h2 4 / body / after 5
```

Delete the bullet, or replace it with the measured statement: every `LoopKind` was wrong on the
trailing delivery, and the step boundary was the single cause.

#### N1: three comments name `run_loop` as the function that opens the header and `END` clauses

At `56d9d1c86`: `crates/rexx-exec/src/run.rs:5184` ("is opened by `run_loop`: the header's, once
before the body runs"), `crates/rexx-exec/src/clause.rs:520` ("both of which `run_loop` opens as
clauses in their own right"), and the same claim in the previous plan around `:457`.

`run_loop` (`run.rs:5961`) evaluates the header plan and delegates. The `Simple` arm that opens both
clauses is in `run_loop_with_header`; a repeating loop's are in `run_repeating`. The compiled engine
never calls `run_loop` at all: `Op::LoopRun` enters `run_loop_with_header` directly
(`crates/rexx-exec/src/ir/drive.rs:1337`). A reader who follows the pointer lands in a function with
no `in_clause` in it.

While in `clause.rs`: the sentence around `:519-524` generalises to `DO`/`LOOP`, and for a repeating
loop `END` has no clause of its own. The next header re-test carries `END`'s line
(`HeaderClause::End`). The sentence is true of `Simple`, which is what the paragraph is about, so
bound it rather than deleting it.

#### Steps

1. Confirm each claim above against the tree before acting, including that `Op::LoopRun` enters
   `run_loop_with_header` directly and that `run_loop` contains no `in_clause`. Report what you
   checked and what you found, including anything that has moved since these line numbers were taken.
2. Fix I3 by deletion or by the measured replacement. Fix the three N1 sites.
3. `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
   `cargo doc --no-deps` so that a moved intra-doc link is caught. `cargo test`'s doc-test pass
   compiles doc code blocks and does not check links, and `rustdoc::broken_intra_doc_links` is
   warn-by-default and denied nowhere in this workspace.

---

