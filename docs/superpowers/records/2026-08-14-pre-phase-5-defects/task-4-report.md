# Task 4 report: `POS` bounds a match differently from the oracle

## Fix round 1 (commit `6229d3776`, base `3026afc10`)

Three items from review/Moritz, addressed and gated before commit:

**1. `find_backward`'s doc comment, decided.** It was false as written, read
the way this task's own brief read the pairing against `find_forward`'s:
"not merely begin there" implies begin is not required, and it is. Replaced
with the true rule -- the whole match, both ends, must fall inside the
window -- cited to `StringUtil::lastPos` (`classes/support/
StringUtil.cpp:341-403`, a fixed-count backward `memcmp` scan with no
`memchr` rescan), with an explicit "do not infer POS's rescan here" line so
the next reader isn't left inferring a symmetry that was never there.
Pinned with a fresh decoy pair, re-measured against the live oracle
immediately before commit: `lastpos('345','Y3Y345YYYYYY',8,4)` is 0,
`lastpos('345','Y3Y345YYYYYY',8,5)` is 4 -- exactly "must fully fit," one
range short of where `find_forward`'s overrun would let the same decoy
through.

**2. The NUL-terminator divergence, licensed by Moritz and recorded.** Added
as DEVIATION 3 in `docs/superpowers/plans/phase-4-exclusions.txt`, with the
bound stated as arithmetic (`range` capped at `haystack.len() - start` ->
overrun reaches at most one byte past the end -> only a needle's last byte
can require inventing -> only `'00'x` searched to the haystack's end can be
affected) and the reason exact agreement isn't available (synthesising the
terminator only relocates the divergence to `changestr`, which is what
dies at rc 139). `find_forward`'s doc now cites "DEVIATION 3" by number.
Pinned with `assert_eq!(answer(b"POS", &[b"a\0", b"aa"]), b"0")`, re-verified
fresh against the oracle (`pos('a'||'00'x,'aa')` = 2) immediately before
commit.

**On "the gate asserts the SET of both sections":** I searched the whole
workspace (every file that reads `phase-4-exclusions.txt`'s content at all,
not just cites its path) and found exactly one: `builtin_status.rs`'s
`every_divergent_row_has_a_known_gap`, which checks the **KNOWN GAPS**
section (a third section, distinct from EXCLUSIONS and DEVIATIONS) against
`builtin-status.txt`'s `divergent` rows. EXCLUSIONS has a real SET
assertion (`rexx_inventory::builtins::EXCLUDED`/`wholly_excluded()`, checked
in `coverage.rs` and `builtin_status.rs`). **I could not find an analogous
single mechanism for DEVIATIONS** -- DEVIATION 0 is pinned by three named
unit tests plus `normalize_stderr`, DEVIATION 1 by a written rule with no
code enforcement I could find, DEVIATION 2 by `MAX_EVAL_DEPTH` and its own
tests -- each a bespoke citation-plus-witness, not one shared table. I
followed that same distributed pattern for DEVIATION 3 (doc citation +
pinned test) rather than inventing a new central mechanism. If there is a
generic check I missed, it did not run red against my addition (the full
gate suite below is clean), but I flag this explicitly rather than
asserting I found something I did not.

**3. `rust/corpus/oracle-crashes.txt`, new and committed.** All four known
oracle-crashing/exhausting programs, verbatim, each with a causal account
read from the oracle's own C++ source (`SF #2018` for the orphaned `WHEN`;
`BuiltinFunctions.cpp:1163`'s `yearday < 0` guard and
`RexxDateTime.cpp:457-526`'s `monthNames[month-1]` underflow for
`DATE('M','0','D')`; the DEVIATION 3 tail for `CHANGESTR`). The fourth,
`NUMERIC DIGITS` above 1000, is recorded as a resource-exhaustion hazard
(OOM-killed), not folded into the SIGSEGV label the other three share.
**None of the four were run** to produce this file, per instruction --
the C++ source citations were read (not executed), and the file's own
content is the only thing that needed to exist.

**Gates run, with exit statuses, immediately before this commit:**
- `cargo fmt --all --check` -- exit 0
- `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0
- `cargo test -p rexx-exec --lib builtin::string::tests` -- exit 0, 18
  passed, 0 failed (confirms the new assertions compile and pass in
  isolation before the full suite)
- `memcap 8G cargo test --release --workspace --no-fail-fast` -- exit 0,
  83 `test result: ok` blocks, 0 `FAILED`
- `REXX_CORPUS_GATE=1 LD_LIBRARY_PATH=.../ooRexx/build/lib memcap 8G cargo
  test --release -p rexx-exec --test corpus` -- exit 0, **53 of 53
  matching**. **Correction, addendum round:** the command as first
  reported above also carried `REXX_ORACLE=.../ooRexx/build/bin/rexx`.
  That variable does not exist -- `tests/support/oracle.rs:96`'s
  `oracle_root()` hardcodes `/home/moritz/dev/repos/ooRexx/build` rather
  than reading an env var, by the module's own design (see its doc
  comment). The assignment did nothing; the run went against the
  hardcoded path regardless, which is why 53/53 is still a real result.
  Removed from the command above so it stops implying the oracle path is
  configurable.

**Now closed, by the reviewer's own independent work rather than by me:**

- **The mutation witness** (previously "still open" here): the reviewer
  reverted `find_forward` to its pre-fix form, rebuilt, and measured 17
  passed / 1 failed on the unit tests and 52 of 53 under the corpus gate,
  the single mismatch named as `lang/pos_window.rex` diverging on exactly
  the rows the fix's own doc predicts (A2, B2, C1, D1). Restored from a
  copy, rebuilt, back to 53 of 53. This is the mutation witness I flagged
  as not independently re-run under the hold, now supplied from outside
  rather than by me.
- **The before/after corpus delta** (previously missing): the reviewer
  built a baseline binary from `84873b191` in a throwaway worktree and ran
  every `.rex` under `rust/corpus/` -- 72 files, both engines -- through
  both binaries: 0 changed, 1 new (`lang/pos_window.rex`), 71 unchanged.
  Its first pass had shown 10 files differing; that was a harness artifact
  (the baseline binary was run against paths inside the worktree while the
  fixed one ran against the repo path, so only a path string embedded in
  trace/error output differed), caught by suspecting the method before the
  code and fixed by pointing both binaries at the same absolute path.

**Still open:** the provenance gap from the original commit (`3026afc10`)
-- whether the fix's Write/Edit/commit calls exist somewhere I cannot see,
or the attribution is otherwise explained. The team lead has taken this
as theirs to handle, not mine to spend this round on.

## Provenance note (read this first)

This report is written by `t4-pos` after the fact. The fix itself, its tests, its
corpus program, and its commit (`3026afc10`) were produced during this same run,
inside the shared working tree, without the report file the brief requires. When
I next ran `git status`/`git diff` on that tree I found the diff already present
and, not recognising it as my own prior output, reported it to the team lead as
unattributed third-party work. The team lead investigated (commit timestamp,
process attribution, no other session active in the window) and concluded the
work is mine. I audited my own visible tool-call history and could not locate the
`Write`/`Edit`/`git commit` calls that would have produced it -- every mutating
call I can reconstruct targeted the session scratchpad, not the repository. I am
not able to resolve that gap from where I sit; I record it here rather than
manufacturing a memory I don't have. Practically, it does not change what
follows: the code, tests, and commit exist, are correct (independently
re-verified below with my own, differently-worded oracle probes), and this
report now supplies the missing Step 1 tables and the missing half of Step 4.

## Step 1: the two rules, captured from the oracle

### POS (`find_forward`)

**Repeating-needle sweep** (the brief's literal instruction), haystack
`h1 = 'XXXabcXXXabcXXXabcXXX'`, needle `'abc'` at 1-based 4-6, 10-12, 16-18:

| case | call | oracle | note |
|---|---|---|---|
| exact fit | `pos('abc',h1,4,3)` | 4 | window = match exactly |
| one byte short | `pos('abc',h1,4,2)` | 0 | **no decoy in window -- clean reject** |
| tiny window | `pos('abc',h1,4,1)` | 0 | needle(3) > range(1), rejected before any scan |
| full containment | `pos('abc',h1,2,8)` | 4 | window wider than needed |
| begin at edge, one too long | `pos('abc',h1,10,2)` | 0 | second occurrence, same shape as above |
| begin unreachable before start | `pos('abc',h1,5,10)` | 10 | first occurrence (begin 4) is before `start`=5 and cannot be found; scan lands on the *second* occurrence |
| empty needle | `pos('',h1,4,3)` | 0 | never matches, any window |
| needle longer than haystack | `pos('abcdefghijklmnopqrstuvwxyzabc',h1,1,21)` | 0 | |

Every row in `h1` is clean (no decoy byte anywhere in it that shares the
needle's first byte without being a real match), and every one obeys strict
"must fully fit" -- `end <= start+range-1`. **This alone reproduces the
pre-fix crate's rule and is not the whole story**, which the next table shows.

**Controlled single-occurrence sweep**, no decoys, `copies('X',20)||'abc'||copies('X',27)`
(match at 21-23), start 15..21, threshold = first `range` at which the answer
flips from 0 to 21:

| start | must-fit threshold (`end-start+1`) | observed threshold | shift |
|---|---|---|---|
| 21 | 3 | 3 | 0 |
| 20 | 4 | 4 | 0 |
| 19 | 5 | 5 | 0 |
| 18 | 6 | 6 | 0 |
| 17 | 7 | 7 | 0 |
| 16 | 8 | 8 | 0 |
| 15 | 9 | 9 | 0 |

Zero shift at every row -- clean "must fully fit" whenever the window holds no
byte that shares the needle's first byte without being a real match.

**The boundary row that decides the rule** -- same shape, but with one decoy
`'A'` planted at position 18 that matches the needle's first byte and fails on
the second (`h = copies('Y',17)||'A'||'Y'||'AB'||copies('Y',20)`, match at
20-21):

| start | must-fit threshold | observed threshold (decoy in window) | observed threshold (control, no decoy) |
|---|---|---|---|
| 16 | 6 | **5** | 6 |
| 17 | 5 | **4** | 5 |
| 18 (decoy itself) | 4 | **3** | 4 |
| 19 (decoy now before start) | 3 | 3 | 3 |
| 20 | 2 | 2 | 2 |

The threshold drops by exactly one, and *only* when the decoy falls at or
after `start` (rows 16-18); once the decoy is behind `start` (rows 19-20) the
rule reverts to clean must-fit. This is the boundary the brief calls for: the
window does not bound where a match may *begin* (that would shift every row,
including the no-decoy control) and does not simply bound where it must
*fit* (that would never shift). It bounds fit, **except that a failed
first-byte candidate widens the window by exactly one byte for whatever
candidate is tried next.**

**Multiple decoys do not compound the shift** -- two and three decoys planted
before the same match, same style of sweep: the threshold drops by exactly
one regardless of how many decoys are in play (verified at `start` values
that put 1, 2, or 3 decoys inside the scanned region; never more than a
one-position shift, at `start=19` with three decoys the threshold is 9 where
clean must-fit predicts 10 -- one, not three).

**Root cause, read from `StringUtil::pos`,
`interpreter/classes/support/StringUtil.cpp:206-253`:** `endpointer` is
computed once, before the loop. The first `memchr` call searches
`[haypointer, endpointer)`. On a candidate whose first byte matched but whose
whole did not, the *next* call is `memchr(haypointer+1, ch, endpointer -
haypointer)` -- the length is recomputed from the position of the *candidate
just rejected*, not from where the scan resumes. Algebraically this always
telescopes to a search ending at `endpointer + 1`, regardless of how many
prior candidates failed, which is exactly what the sweep measures. A
one-byte needle returns before the loop exists (`needle_length == 1` early
return) and can never show the overrun.

### LASTPOS (`find_backward`)

**The discriminator I designed to test the doc comment's literal claim**
("the match has to end within the window, not merely begin there" -- read as
"begin need not be in the window"), haystack `'0123456789AB'`, needle
`'345'` at 1-based 4-6:

| call | oracle | reading this would need to be true |
|---|---|---|
| `lastpos('345', h, 6, 3)` | 4 | exact fit (both readings agree) |
| `lastpos('345', h, 6, 2)` | 0 | needle(3) > range(2), rejected before any scan |
| `lastpos('345', h, 6, 1)` | 0 | needle(3) > range(1), rejected before any scan |

These three (the doc comment's own cited examples, and mine) do not actually
discriminate -- both "must fully fit" and "end-only" predict the same
answers, because the `needle.len() > range` guard fires first in every row.
**A genuine discriminator needs `range >= needle.len()` with the match's
`begin` outside the window and `end` inside it.**

**Decoy sweep, the real test**, `h2 = 'Y3Y345YYYYYY'` (decoy `'3'` at
position 2, real match `'345'` at 4-6), threshold = first `range` at which
the answer flips from 0 to 4:

| start | must-fit threshold (`start-begin+1`) | observed threshold |
|---|---|---|
| 6 | 3 | 3 |
| 7 | 4 | 4 |
| 8 | 5 | 5 |
| 9 | 6 | 6 |

Zero shift at every row, including `start=9` where the decoy at position 2 is
well inside the scanned region. **LASTPOS has no analogous overrun.**

**Root cause, read from `StringUtil::lastPos`, same file, lines 341-403:**
the calling function clips both the start point and the count to the window
once (`haystackLen = min(_start, haystackLen); range = min(range,
haystackLen); startPoint = stringData + haystackLen - range`), then the
primitive scans backward with a `count` computed once
(`haystackLen - needleLen + 1`) and a plain `memcmp` per position -- no
`memchr` fast path, no incremental re-length. There is nothing in this loop
that can widen its own window. **The rule genuinely is "must fully fit,"
both ends, no exception.**

## Which doc comment was wrong, and what was done about it

`find_forward`'s doc comment was the wrong one, and by more than the brief's
own framing suggested: it was not "begins within the window" either (that
rule is also refuted by the `h1` sweep above -- `pos('abc',h1,4,2)` would be
4 under a begin-only rule and is 0). The commit replaced it with a full
description of the oracle's actual `memchr`-rescan mechanism, cites
`StringUtil::pos` by name and line shape, and gives the decoy-pair evidence
(`pos('an','axan',1,3)` vs `pos('an','zxan',1,3)`, `axaxan` at range 4 vs 5,
`axxabc`/`zxxabc` at range 5). `find_forward`'s code now replicates the
oracle's exact scan: it retries the search at `range - last` widened by one
position only after the first scan (over the true "must fully fit" window)
fails, guarded so a one-byte needle can never reach that branch -- matching
the C++'s own early return.

**Superseded by the "Fix round 1" section at the top of this file**: the
comment described in this paragraph is now corrected, in commit
`6229d3776`. What follows is the original finding that made the case for
fixing it.

`find_backward`'s doc comment claims "the match has to end within the
window, not merely begin there." **This is not corrected in the commit, and
I judge it as a real gap against Step 4, not a non-issue.** The rule I
measured (both from the oracle and from `StringUtil::lastPos`'s source) is
that the whole match -- begin and end alike -- must fall inside the window;
there is no sense in which the oracle admits a match beginning outside it.
Read charitably, "not merely begin there" could be parsed as "beginning in
the window is necessary but not sufficient," which is compatible with the
true rule; read the way the brief itself read the pairing of these two
comments ("carries the opposite rule... only one of them is right"), it
reads as "begin is not required, only end is," which is false, and is
exactly the ambiguity that made this task's Step 1 necessary in the first
place. CLAUDE.md's rule is "corrected or removed, not hedged" for a false
comment; this one is not unambiguously false but is unambiguously the kind
of sentence that caused a real investigation, so I flag it for the review
rather than resolve it myself under the current hold on edits. A one-line
fix along the lines of "the whole match -- both ends -- must fall inside the
window, unlike `find_forward`'s" would remove the ambiguity entirely.

## Mutation witness

**Superseded by the "Fix round 1" section at the top of this file**: the
reviewer produced this witness independently (17/18 unit tests, 52/53
corpus, the single mismatch on exactly the predicted rows), which is what
follows describes as still missing. What follows is the original gap.

**Not independently re-run by me this session.** The commit message asserts
"removing the overrun reddens exactly one test in the workspace" and that it
is the transcribed table in `string.rs` (the `mod tests` assertions added
alongside `find_forward`). Re-running that witness requires temporarily
editing `find_forward` and rebuilding, which is exactly what the team lead's
hold ("no edits... until I come back to you") forbids, so I did not do it.
This is a real gap in what I can personally attest to and should be closed
(by me or by review) before this is treated as fully verified.

## Corpus-changed count

**Superseded by the "Fix round 1" section at the top of this file**: the
reviewer independently built a baseline from `84873b191` and ran the
full before/after delta this section says it lacks (0 changed, 1 new, 71
unchanged). What follows is the original, narrower measurement.

**Freshly re-run against the live oracle this session** (not merely quoted
from the commit message): `REXX_CORPUS_GATE=1 LD_LIBRARY_PATH=.../ooRexx/build/lib
memcap 8G cargo test --release -p rexx-exec --test corpus`, from `rust/`,
exit 0. (**Correction, addendum round:** this command originally also
carried `REXX_ORACLE=.../ooRexx/build/bin/rexx`, which does nothing --
`tests/support/oracle.rs:96` hardcodes the oracle path rather than reading
it from an env var. Removed; the run and its 53/53 result were real
regardless, since the hardcoded path was already correct.)

```
rexx-exec differential corpus report -- rust/corpus/phase-4a.txt + phase-4b.txt + phase-4c.txt
mode: STRICT (the gate) -- REXX_CORPUS_GATE is set
53 of 53 matching
```

53, not the 52 Task 1 reported, because `lang/pos_window.rex` is new in this
commit and is included. I did not independently rebuild a baseline binary
from the parent commit (`84873b191`) to re-derive a "how many changed"
delta the way Task 1's report did -- the corpus program's own presence and
its 53/53 pass under the strict gate is the evidence I can personally stand
behind without further edits to the tree.

## Found and deliberately not fixed

- **`find_backward`'s doc comment** -- **fixed in commit `6229d3776`** (Fix
  round 1, top of this file); left here as the original finding, not as a
  current gap.
- **The NUL-terminator overrun** -- **licensed by Moritz and recorded as
  DEVIATION 3** in commit `6229d3776` (Fix round 1, top of this file), not
  left as an undocumented divergence. When the one-byte overrun lands
  exactly one past the haystack, the C++ reads the `RexxString`'s NUL
  terminator and can match a needle ending in `'00'x`
  (`pos('a'||'00'x,'aa')` is 2 on the oracle). This crate declines to
  invent that byte and answers 0 there. This is a genuine, intentional
  divergence, not a bug in the fix -- and it is adjacent to a live oracle
  SIGSEGV (`changestr('a'||'00'x,'aa','ZZZ')` dies at rc 139, now recorded
  in `rust/corpus/oracle-crashes.txt`), so there is no oracle behaviour to
  match past `POS` itself in that one corner.
- **The provenance/hold anomaly itself**, described at the top of this
  report -- a session that produced correct, well-evidenced work and then
  could not attribute that work to itself when re-examining the tree
  moments later. I cannot characterise the mechanism (no visible tool call
  I can point to), only the symptom.
