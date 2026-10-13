# Phase 9b Task 5 re-review: fix round 1

Reviewer: t5 re-reviewer, 2026-10-13. Range `cc6e05caa..4764c16ae`. `rexx-run` rebuilt at HEAD
(`Compiling rexx-exec` seen). Runs from fresh empty directories, crate under `memcap 4G`.
I did not re-run `cargo test` (the implementer's gate records are the evidence for it).

## Verdict: not yet. One Important finding; the rest is resolved.

## Ruled items

1. Deviation 34: resolved. The entry cites `normalizePathName` at `SysFileSystem.cpp:687-776`
   and line 706 for the `previousSlash` condition. Both are correct in the oracle tree
   (`/home/moritz/dev/repos/ooRexx`: function 687-776, comment 705, condition 706). Minor: the entry
   adds "the same in this checkout's interpreter/", whose line numbers are 604-693 (condition
   at 623). The rule it states (set `previousSlash` only when the next byte is not `.`, before
   dropping a repeated `/`; `..` returns there) matches the C++. `normalize_bytes` is unchanged
   (the diff touches only its doc). The agreeing rows are in `platform_qualify_setlocal.rex`:
   oracle and crate stdout, stderr and rc identical (rc 0). The ten differing rows are in
   `a_parent_reference_removes_one_component_where_the_oracle_does_not`, which names the
   deviation and asserts `oracle != crate` per row.
2. Brackets `[.c.]` and `[=c=]`: partly resolved, see Important 1.
3. `entries()` stats only wanted names: resolved. The first pass filters by `matches`; the
   recursion pass uses `|_| true`, so no directory is dropped. Checked against the oracle on a
   tree (`Sub/Deep/Target.TXT`, `other/target.txt`, `top.TXT`) with masks
   `*.txt t* SUB deep x* [T]*` and options `so soi sbi sdi sfi s sfo`: all 42 rows equal on both
   sides (the `b` rows differ only in the run directory's own path prefix).
4. Entry 40 retitled and the loop probe recorded: resolved.

## Checks

- Witnesses `rexxutil_file_tree.rex` and `platform_qualify_setlocal.rex`, oracle vs crate: stdout
  and stderr identical; rc 216 and 0 on both; both run directories empty afterwards.
- `[a-[=c=]]` matches nothing on the oracle (`o` and `oi`): confirmed, and the crate agrees.
- Every bracket row in the witness is equal on both sides.
- My own fuzz: 1500 generated patterns over `[ ] . = - a b c A ! ^ :`, with `o` and `oi`, against
  the names `a b c A - ] ^ ! [ = x.y`. Of 3000 rows, 2995 are equal and 5 differ.

## Important

1. **A malformed `[=` after earlier bracket items diverges.** The new doc and the report say "a
   `[=` not closed by `=]` two bytes on is a literal `[`". That holds only as the first item. On
   the oracle, everything accumulated before such a `[=` is dropped from the set (non-negated
   bracket); the crate keeps it. Names `a b c A - ] ^ ! [ = x.y`, option `o`, oracle then crate:

   | pattern | oracle | crate |
   |---|---|---|
   | `[b[=a]` | `= [ a` | `= [ a b` |
   | `[bc[=a]` | `= [ a` | `= [ a b c` |
   | `[abc[=x]` | `= [` | `= [ a b c` |
   | `[a-c[=x]` | `= [` | `= [ a b c` |
   | `[b[=.]` | `= [` | `= [ b` |
   | `[b[=]` | `= [` | `= [ b` |
   | `[b[=ab]` | `= [ a` | `= [ a b` |
   | `[b[:alpha:][=x]` | `= [` | `= A [ a b c` (the class is dropped too) |
   | `[!b[=x]` | same | same (the one negated row I ran shows no difference) |

   The same holds under `oi`. Agreeing controls: `[[=a]`, `[[=.]`, `[b[=a`, `[b[.a]`,
   `[b[=ab=]]`. The witness's `[[=a]` row cannot see this because it has no earlier item. Fix: in
   `bracket`, a `[=` that is not a well-formed `[=c=]` clears `hit` before continuing with the
   literals. Measure the negated case first (`[!bc[=x]`), since my one negated row does not
   separate the readings. Add the rows to the witness's `patterns` and correct the doc sentence.

## Minor

- Deviation 34's "same in this checkout" line numbers (above).

## No new defects found in the rest of the diff

`collating` and `equivalence` index safely (`get(..)?`, `get(at..at + 5)?`). `collating` searches
from `at + 2` and returns `None` on a missing `.]`, so the whole pattern fails, as measured. The
`entries` signature is used consistently at both call sites and in the unit test.
