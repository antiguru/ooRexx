# Task 3 fix round 2 re-review: fcfb94f39..6a3091cc6

Ours: `rexx-run` built in release from a worktree at `6a3091cc6`, its own target dir (both since removed).
Oracle: the standard wrapper. Each probe ran from its own empty dir through `/tmp/claude-1000/p61/t3rr2/cmp.sh`.
Probes are in `/tmp/claude-1000/p61/t3rr2/probes/`, outputs in `/tmp/claude-1000/p61/t3rr2/out/<probe>/`.

### Finding Verdicts

- **Critical: a heap TO or BY left unrooted after a numeric loop's control switched to an object** --
  ADDRESSED.
  - The switch builds no objects now: `object_control_from_numbers` keeps TO and BY as
    `LoopBound::Number` (`run/loops.rs:2994-3021` at HEAD). Each pass makes the object it sends inside
    `object_control_advance`'s temps frame (`loop_bound_object`, `:519-532`, called from
    `object_control_pass`).
  - A header's own `+` answers are written into the header's registers (`ObjectHeader::home`, `:337-351`;
    `flat_loop_start`, `:1752-1758`).
  - `dead2`, `dead3`, `dead7` from the first re-review: identical on all three descriptors, rc 0 both
    (1 run each here; the report claims 3 of 3).
  - Named risk, register reuse. Read `ir/compile.rs:270-375`: the header registers are allocated in the
    enclosing scope before `header_top`, a `debug_assert_eq!` holds the top at exactly the header's own
    registers, and they are released at `end + 1`. Body clauses and a nested loop's header allocate above
    them. `LoopRun` hands back `Fallback` only for `LoopKind::Simple` (`run/loops.rs:1735`), so every
    object-controlled loop goes through `flat_loop_start` and is homed.
  - Probe `reg1.rex`: an object header with heap TO and BY answers, string temps in the body, a nested
    object loop with its own object header, and a recursive internal routine running an object loop,
    all with allocating `+` and `>`. Identical, rc 0 both, `end k41.5 k3.0 486`. Identical again under
    `REXX_SWITCH_MODE=every`.
  - `reg2.rex` (the same, shorter) under `run_program_collect_every_alloc`, from a throwaway test in the
    worktree copy only: stdout `end k8.5 k3.0 45`, the oracle's; stderr empty; 375 collections.
- **Important: `is_true_object` accepts any one-byte '1' where the oracle requires TheTrueObject; ruled a
  deliberate deviation** -- NOT ADDRESSED. The row exists (Deviation 25, `phase-4-exclusions.txt`, diff
  `:134-157`) with citations and a `licensed_divergences` witness, and there is no behaviour change, as
  asked. But the row and the doc state a rule the crate does not follow, and the ruling was given on that
  statement:
  - The ruling (progress.md, Moritz 2026-10-08) reads "a computed '1' is true in a DO TO test", on the
    ground that Rexx logical values are the strings 0 and 1. The row's title says "TAKE ANY LOGICAL TRUE".
    `is_true_object`'s doc says "logical true, the one-byte '1'" (`run/loops.rs:2979`).
  - What the crate does is an identity test against the inline handle `ObjRef::inline_byte(b'1')`. A
    computed `0 + 1` or `1 * 1`, a literal `1` and a literal `'1'` are all logical true and all
    one-byte '1', and none ends the loop.
  - Measured, `true1.rex`: a `>` under `to 3 for 3`, passes run per answer.

    | answer | oracle | ours |
    |---|---|---|
    | `0 + 1` | 3 | 3 |
    | `1 * 1` | 3 | 3 |
    | `1` | 3 | 3 |
    | `'1'` | 3 | 3 |
    | `left('12', 1)` | 3 | 0 |
    | `abbrev('abc', 'a')`, `2 > 1`, `.true` | 0 | 0 |

  - So the crate is a second identity test with a different representative, not the truth-value test
    the ruling describes. The row's own measurement paragraph lists `0 + 1` among the agreeing cases.
    That contradicts its title and its WHY. `do_object_compare_true.rex`'s comment says a literal 1, a
    literal '1' or `0 + 1` "is not" logical true. That also contradicts the WHY.
  - Resolution needs a ruling, not a wording fix. Either the crate tests the value, as WHILE does, and
    `0 + 1` and `'1'` end the loop: that is a behaviour change and a wider divergence than the one
    measured. Or the row says the crate matches an inline '1' only, and the WHY ("oracle defect,
    logical values are strings") no longer justifies it.

### New Breakage in the Fix Diff

1. **Minor: a TO or BY that converted as a number is a new object at every pass, where the oracle passes
   the same object each time.**
   - `LoopBound::Number` makes its object per send (`loop_bound_object`, `run/loops.rs:519-532`).
   - Measured, `ident2.rex`: `>` and `+` compare `o~identityHash` with the previous call's. There are two
     cases: an object header with `to 3.5`, and a numeric loop switched to an object with `to 4.5 by 1.25`.
     - Oracle: `to same 1` on every pass after the first; in the switched loop, `by same 1` too.
     - Ours: `to same 0` throughout; in the switched loop, `by same 0` too.
     - stdout only, rc 0 both.
   - `ident.rex` uses `==` instead and is identical. Only identity can see this.
   - Fix round 1 made each object once (`control_number_object`, removed in diff `:513-518`). So this is
     new in this round by reading the code; I did not run the base.

### Out-of-Scope Observations

None.

### Checks run

- `true1.rex`, `reg1.rex` (plain and `REXX_SWITCH_MODE=every`), `reg2.rex` (stress, worktree-only test
  `tests/zz_probe.rs`, 375 collections), `dead2`, `dead3`, `dead7`, `ident.rex`, `ident2.rex`; one run per
  engine each.
- Report checks: the fix report names fmt, clippy, the workspace tests, both corpus gates, the gated
  `concurrency_tests` at `308386167` and `collect_stress`, with exit statuses. Not re-run.
- `licensed_divergences` row `do-compare-computed-true` (diff `:827-836`) matches `true1`'s `left` case.

### Verdict

**Fix round:** Findings remain open. Finding 2 is open: Deviation 25 and the `is_true_object` doc
describe a logical-true test, and the crate runs an identity test on the inline '1' (`0 + 1` and `'1'` do
not end the loop). The ruling needs to be revisited with that table. The fix also added Minor 1.
