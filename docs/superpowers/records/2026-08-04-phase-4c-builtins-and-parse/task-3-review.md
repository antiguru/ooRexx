# Task 3 review: `builtin/string.rs`, the 22 remaining string builtins

Reviewed commit `63a9ea9f` against parent `0c58926e` (also HEAD), brief
`task-3-brief.md` (both halves), report `task-3-report.md`.

**Spec compliance: FAIL.** **Quality: CHANGES-REQUESTED.**

The value semantics are, as far as I can measure, right: 2,464 hand-designed
differential programs covering every optional position, every interior
omission, non-default pads, empty strings and non-ASCII subjects came back
with **0 mismatches**, and 2,328 of 2,363 programs extracted from the 23
committed `ootest/ooRexx/base/bif/*.testGroup` files matched byte for byte.
The two failures below are both on paths the brief named explicitly: the
allocation guard the report added in §5.3, and the message substitution the
report settled in §2.

---

## 1. Re-run verification (each status unpiped, restored tree)

| Command | Exit | Result |
|---|---|---|
| `cargo test --offline --workspace --no-fail-fast` | **0** | 1055 passed, 0 failed |
| `cargo fmt --all --check` | **0** | clean |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | **0** | clean |
| `REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus` | **0** | `mode: STRICT (the gate)`, `42 of 42 matching` |

`cargo test -p rexx-exec --test keyword_assertions` exits 0 and reports 778
bodies not passing, matching the committed exempt file exactly (778 rows: 772
`4c` + 6 `defect:compound-do-control-variable`).

The report's four numbers reproduce. `git status --short` is empty at the end
of this review; the one file I mutated (`string.rs`) was restored from a copy
and its md5 re-checked against the pre-mutation copy.

---

## 2. Findings

### F1 (Critical) -- the allocation guard is on the wrong allocation, and eight builtins still abort the process

`buffer()` reserves the result with `try_reserve_exact`, but every builtin
then hands the finished bytes to `Interp::text`, whose body is
`bytes.to_vec()` (`src/value.rs:46`) -- a second, **infallible** allocation of
the same size. The guard therefore covers half the requirement, and any size
that fits once but not twice aborts.

Measured at the project's own mandated `ulimit -v 1048576`, oracle against
`rexx-run`, one program per fresh directory:

```text
say length(copies('a',400000000))    oracle 400000000 rc 0   rust: SIGABRT rc 134
                                     "memory allocation of 400000000 bytes failed"
```

The same abort at the same size for `left`, `right`, `center`, `space`,
`substr`, `insert` and `overlay` -- all eight of the builtins the report's
§5.3 lists as guarded, and the oracle answers all eight successfully. This is
exactly the outcome §5.3 says the guard exists to prevent ("an unguarded `Vec`
of that size in Rust *aborts the process*, which is strictly worse than any
divergence"). The window is not exotic: it is roughly (limit/2, limit) for
whatever the process's memory limit is, so it is reachable on any machine, and
it is reachable at the ulimit every probe in this project is required to use.

The upper end diverges in the other direction too, for the same reason:
`say length(copies('a',900000000))` is `900000000` on the oracle and `Error 5`
rc 251 under `rexx-run` at the same 1 GB limit.

The 251/Error-5 boundary the task did verify (`999999999999999999` versus
`1234567890123456789`) is real and correct -- I reproduced both. It is the
*middle* of the range that is unguarded, and no committed test covers it: the
unit test `a_result_too_large_to_allocate_is_the_oracles_own_error_5` only
exercises sizes that fail the first reservation.

### F2 (Critical) -- every new raiser's `found "..."` substitution is byte-wrong for any non-printable-ASCII byte

The seven new raisers put user bytes into the message through
`String::from_utf8_lossy` into a `Vec<String>`. Two separate divergences fall
out, both measured byte for byte with `od`:

* **Bytes >= 0x80**: the oracle emits the raw byte; this crate emits U+FFFD
  (`ef bf bd`). Measured: `say copies('ab','FF'x)` -- oracle
  `... found "` `\377` `".`, rust `... found "` `\357\277\275` `".`, same rc
  216, same everything else.
* **Control bytes 0x00-0x1F except 0x09, 0x0A and 0x0D**: the oracle
  substitutes `?` (0x3F); this crate passes the byte through raw. Measured
  across the whole range: `00..08, 0b, 0c, 0e..1f` -> `3f`; `09`, `0a`, `0d`,
  `7f` and everything >= 0x80 -> raw.

I generated 692 error-path programs (every bad-number spelling, every bad pad,
every bad option letter, every arity from 0 to max+1, and every
interior-omission position for all 23 names) and ran them differentially:
**118 mismatches, and every single one is this class** -- no other error-path
divergence exists. Twenty of the 23 names are affected; 40.12, 40.23 and
93.915 are the three sub-codes that carry a value.

This is not hypothetical for the next task. Nine cases in the **committed**
ooTest groups already reach it: `COPIES` test095/test231/test372/test538,
`DELSTR` test17, `INSERT` test035/test066, `SUBSTR` test019/test048. Per the
brief, "a case it covers that the implementation misses is a defect now, not
later" -- these will surface at Task 15 as exempt rows that should not be
there.

The `Vec<String>` + lossy shape predates this task (21 sites in `error.rs`,
including 4b's 88.928), so the fix is plumbing: carry substitutions as bytes
and apply the oracle's control-byte rule at render time. But this task is what
made it reachable, and it is this task's raisers the ooTest groups exercise.

### F3 (Important) -- the TRANSLATE divergence has no `KNOWN GAP` row, and is broader than reported

The reproducer holds, and I found four producers of the oracle's null-string
singleton rather than the report's two:

```text
zz = ''               ; translate('abcdef','123',zz)  ->  [abcdef]   (both agree)
zz = left('abc',0)    ; ...                           ->  [      ] vs [abcdef]
zz = copies('a',0)    ; ...                           ->  [      ] vs [abcdef]
zz = substr('abc',1,0); ...                           ->  [      ] vs [abcdef]
zz = strip('  ')      ; ...                           ->  [      ] vs [abcdef]
```

Documenting rather than implementing is the right call -- reproducing it needs
string identity to be observable, which nothing else in the value model has.
But `phase-4-exclusions.txt` is untouched by this commit, and the status row
stays `implemented` rather than `divergent`, which is precisely the mechanism
`builtin_status.rs` built to make absorbing a divergence cost more than a
one-line edit (`a divergent row requires a KNOWN GAP: <NAME> marker`). The
report's §5.1 states the gap and then leaves it to "a future task" -- that is
an unowned gap, the shape this project has paid for twice. It needs a
`KNOWN GAP` row naming TRANSLATE, `strip`/`substr`/`copies`/`left` as the
producers, and the owner.

### F4 (Important) -- the 62,144 + 3,105 sweep is structurally broad and byte-narrow, so its zero is weaker than it reads

The corpora are still on disk (`scratchpad/chunks`, `scratchpad/echunks`), so
this is measured rather than inferred. The value sweep's entire operand
alphabet is seven strings -- `'ab'`, `'a'`, `'abcdef'`, `'banana'`, `'aXbXc'`,
`'a b  c '`, `'  ab  '` -- plus `''`, and its numbers are `-0`, `0`, `1`, `1.0`,
`2`, `3`, `7`, `9`, `'  2  '`. Across **both** sweeps combined there is **not
one hex literal and not one byte >= 0x80**. All 23 names are reached in both,
and the omitted-position and arity structure genuinely is exhaustive -- that
part of the claim holds.

So "62,144 programs, 0 mismatches" means "0 mismatches over printable ASCII
and small integers". That is exactly the blind spot F2 lives in, and it is why
the three seeded mutations (a good negative control, and I reproduced its
logic independently) could not have caught it either: all three mutations are
in value paths the alphabet does reach. The report should say what the sweep
does not cover rather than present the zero unqualified.

### F5 (Minor) -- nothing else

Everything else in the report checks out, including several claims I expected
to be soft:

* **`check_arity` shape (brief priority 2).** None of the 22 has `DATE`'s
  conditional shape. My 692-program error sweep drove every interior-omission
  position of every name at full arity -- the strongest form of the DATE trap
  -- with zero mismatches outside the F2 class.
* **The 40.12/40.23 substitution question (priority 3).** Both halves
  reproduce exactly: `numeric digits 3 ; zz = 2/3 ; numeric digits 9 ; say
  left('ab',zz)` gives `found "0.667"` (not `zz`, not `0.666666667`), and the
  same value in the pad position gives the same string under 40.23. The
  93.9xx family does report the converted value (`left('ab','-1.0')` ->
  `found "-1"`). The doc comments on `argument_not_whole`,
  `argument_not_a_pad`, `invalid_length`, `invalid_position`,
  `argument_not_non_negative` and `invalid_option` record measurements, and I
  re-ran a sample of the specific programs they quote -- all correct,
  including the three `Method argument 1/2/3` positions and
  `space('a b c',1,'')` -> 40.23 `found ""`.
* **`ARGUMENT_DIGITS` = 18.** The report's correction of a subagent's wrong
  "NUMERIC DIGITS sensitive, default 9" is right in both directions;
  `numeric digits 2 ; left('ab','1.0000001')` is 40.12 and
  `numeric digits 30 ; left('ab','1.0000000000000000000004')` is `a`.
* **`keyword-exempt.txt` (priority 6).** Exactly 17 rows removed, none
  re-attributed (the diff is pure deletions). The harness asserts the set in
  **both** directions (`is failing ... and is not on the committed exempt
  list` / `now PASSES but is still on the committed exempt list`), so a
  removed row that still failed would go red -- and the run is green at 778
  failing bodies against 778 committed rows. The header's blocker list is now
  accurate: the live run shows `routine "ARG"` (51) still blocking and no
  `COPIES` or `SUBSTR` at all.
* **The two out-of-brief files (priority 7).** Both necessary and minimal.
  `Builtin::run` taking the row's name is forced -- I confirmed
  `centre('ab',6,'--')` names `CENTRE` where `center(...)` names `CENTER`, so
  one implementation with two names cannot work otherwise, and the
  alternative (each implementation naming itself) is a second copy of the
  table's own string. `STRING_FAMILY` + `every_string_builtin_is_implemented`
  is Step 4's explicit requirement, its 23 names are exactly the brief's list,
  and the doc comment correctly argues why a *derived* check could not
  substitute.
* **The mutation-found coverage gap (priority 9), checked with the rule the
  brief names.** I applied CENTER's odd-pad-goes-left mutation and confirmed
  the suite goes RED (1 failed of 324). Then I removed the six added
  odd-width assertions **and kept the mutation**, and re-ran
  `cargo test --workspace --no-fail-fast`: **exit 0, everything green**. So
  the six cases are not merely able to fail -- they are the only thing in the
  whole suite that catches it. That is the "can fail is not adds coverage"
  check passing, not just being claimed.
* **No forbidden allocation, no `unsafe`.** Every result goes through
  `Interp::text`, which is `alloc_with`; no `Heap::alloc` or
  `alloc_with_uncollected` anywhere in the three changed source files. No
  em-dashes; no comment naming a task number, a phase boundary or a repo
  aggregate.

---

## 3. Cannot verify from the diff

* Nothing. The report's sweep scripts and corpora turned out to be readable
  (`scratchpad/bin/worker.sh`, `scratchpad/chunks`, `scratchpad/echunks`), so
  F4 is measured rather than a guess. The one number I did not reproduce is
  the report's "473 hand-built probes" -- the count itself is unchecked, but
  every transcript I sampled from its tables reproduced.

---

## 4. What has to change

1. Guard the copy inside `Interp::text` (or have builtins hand over an owned
   `Vec` rather than a slice), so the Error-5 path covers the whole
   requirement instead of half of it. Add a test at a size that fits once and
   not twice -- the current test cannot reach the abort.
2. Carry error substitutions as bytes, and apply the oracle's rule: control
   bytes 0x00-0x1F other than 0x09/0x0A/0x0D render as `?`, everything else
   renders raw. Add the `'FF'x` and `'0012'x` cases from the ooTest groups.
3. Add a `KNOWN GAP` row for the TRANSLATE null-string-singleton divergence,
   naming all four producers, or flip the status row to `divergent` and let
   the existing marker requirement do its job.
4. Qualify the sweep claim in the report with the alphabet it actually used.
