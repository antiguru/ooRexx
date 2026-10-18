# Task 6.5 fix round 1: re-review

Reviewed `e1bc7e9f8..76acd13a3` (code only in `rust/crates/rexx-exec/src/dispatch/stream.rs`). Crate built from HEAD `270207ffa`, whose `rust/crates` equals `76acd13a3`. Each probe ran on both sides from its own fresh `mktemp -d`. Finding 1 is not reviewed (pending Moritz).

**Verdict: approve.**

## Finding 2: write-only read side, fixed

File `line1\nline2\n`, `chmod a+w,a-r`, then `open write`. BIF and method forms (`~chars`, `~lines`, `~lines('N')`, `~charin`, `~linein`, `~arrayin`, `~query`, `~lineout`, `~description`) match the oracle on stdout and stderr, except one licensed line:

* `chars`/`lines`/`lines N` 0, `charin` `ERROR:9 Bad file descriptor`, `linein` `NOTREADY:EOF`, `arrayin` 0 items.
* `query position read line` answers 1 with `ERROR:9`.
* `lineout(f,'x',2)` then `query position write` answers 3, and the file bytes match.
* The only diff is `stream(f,'c','query position write line')` returning `ERROR:9 ...` (oracle) against the null string (crate), the RESULT-of-RAISE gap that Task 6.9 owns. The state and description are `ERROR:9` on both.

## Finding 3: implicit read open, fixed

Two-line file, `chmod a-w`: `say linein(f) query position write / write line / state` gives `abc 1 1 READY:`, identical on both. Also identical: `chars lines`, `query position write line`, `write` and `read line` on an implicit read of a read-only file, and the method form.

## Ordinary read-write streams: no regression

`open both` on a 3-line file, then `chars`, `lines`, `lines N`, the four position queries, `linein`, `charin`, `lineout` by line and append, then `chars`/`lines`, `linein(f,1)`. `open read` of a missing file, `open write` of a writable file, and a method-form `both`.

Diffs against the oracle exist, but a crate built from `06f0ce3af` (before the fix) gives byte-identical output to the fixed crate. So they are not regressions.

## Open findings

None from the fix.

## Out-of-scope observation (pre-existing, not from this round)

On an `open both` stream, `lineout(f,'new',2)` followed by `lineout(f,'tail')` differs. The oracle writes `tail` over the following line, leaving `line1\nnew\ntail\ne3\n`. The crate writes `tail` at the tracked write position (the end plus the written bytes), leaving `line1\nnew\n2\nline3\n\0\0\0\0tail\n`. `QUERY POSITION WRITE` answers 23 and 28 on both, so the position matches and the oracle's next write does not use it. The crate before the fix behaves the same. This is not part of the Task 6.5 clusters.
