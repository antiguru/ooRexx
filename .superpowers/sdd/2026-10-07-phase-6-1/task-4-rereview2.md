# Task 4 re-review, fix round 2: 2a03b64c1..66d9eac18

Ours: `rexx-run` (release) built from `git archive 66d9eac18 rust interpreter`, own target dir, one
`Compiling rexx-exec` line. Collect-every-alloc: the previous re-review's `zz_probe_t4rr` harness
(`run_program` against `run_program_collect_every_alloc`; rc, stdout, stderr compared), copied into
the scratch tree only. Oracle: the standard wrapper. Each probe ran from fresh empty directories
through `cmp.sh` (ours plain, ours `REXX_SWITCH_MODE=every`, oracle). Probes and outputs:
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t4rr2/`
(`probes/`, `out/<probe>/`). The scratch target dirs are deleted. Oracle: one run per probe.

### Finding Verdicts

- **Critical 1: a `Directory~setMethod` entry that unsets or replaces itself while it runs, then a
  collection.** ADDRESSED.
  - `activity.rs:641` now roots each running or suspended activation's `cold.executable`.
  - Probes `c5`, `c6`, `c9`, `c10`, `c13`, `c17`, `c18`, `c19` agree with the oracle on stdout, stderr
    and rc, in both engine modes, and are identical plain against collect-every-alloc (up to 400606
    collections). The earlier rc 101 and rc 120 results are gone.
  - `c5` differs only in the stderr traceback line `Compiled method "UNKNOWN" with scope
    "Directory"`, the recorded exclusion; its stdout and rc 163 agree.
  - My own probes, each agreeing with the oracle (both engine modes) and with itself under
    collect-every-alloc:
    - `n1`: an entry that unsets itself, `REPLY`s, forces a collection and reads
      `.context~executable` twice on the continuation (4029 collections with an allocation loop).
    - `n2`: `start` on a self-unsetting entry, with the result read back.
    - `n3`: entry A sets B; B unsets A and B and asks for its executable; A then forces a collection
      and asks again.
    - `n7`: A calls B, B unsets A, A then forces a collection and asks.
  - The six new witnesses (`directory_entry_*`, `directory_unknown_unset_while_running`,
    `setmethod_unset_identity_after_collection`) are identical to the oracle in both modes; I read
    each stdout. They reach the unset/replace and then print the original source.
- **New Minor 1: the rebuilt executable has no stable identity across a collection.** ADDRESSED.
  `dispatch/context.rs:428-435` stores the first answer in `cold.executable`. `c6` prints `1 The NIL
  object ...`, as the oracle does (it printed `0` at `2a03b64c1`).
- **New Minor 2: false clause in `oracle-crashes.txt` entry 30.** ADDRESSED. `:1198` now reads "the
  class defines no `MAKESTRING`", which matches the entry's program.
- **New Minor 3: dead classes and the "sparse array" header in `method_source_shapes.rex`.**
  ADDRESSED. Only `a` and `bn` remain, both used; the header says "an array with no items"; the
  `.object~enhanced` MAKESTRING line it also names is present and prints `ok 1 [return 'ENH']`.
  Identical to the oracle in both modes.

### New Breakage in the Fix Diff

None. Caching the first `.context~executable` answer cannot give a wrong one in the cases I could
build, all identical to the oracle:
- `n4` (a `Directory` entry that replaces itself, then asks, before and after a collection),
- `n5` (an `Object~setMethod` entry that replaces itself, asks, collects, asks again: `e == e2` is `1`
  and both answer the running method's source, not the replacement; a second call and the replaced
  method also agree),
- `n6` (ask, replace, collect, ask: `e1 == e2` is `1`).
The cache is per activation, so a later activation of the replacement builds its own.
`n4` stops at rc 159 on both sides for its `Object` half (`SETMETHOD` is private from the
top level; my probe's error, the same on both) and `n5` covers that shape from inside a method.

### Out-of-Scope Observations

- The previous round's observation stands in part: `cold.executable` is now rooted by the activity
  walk, but I did not check the other `cold` fields (`auto_expose`'s owner and scope) for the same
  gap. Not examined.

### Verdict

**Fix round:** All findings addressed, no new Critical/Important breakage. Critical 1 and New Minors
1-3 are closed. New Breakage: none.
