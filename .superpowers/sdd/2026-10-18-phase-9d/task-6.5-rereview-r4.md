# Task 6.5 re-review, fix round 4

Diff `5d2bdbcf3..76644b0a2`. Crate built at HEAD into a scratch target dir (deleted). Probes ran from fresh empty dirs.

## Finding verdicts

* Buffer model removed: ADDRESSED. `land_write`, `note_read`, `oracle_tell`, `read_window`, `write_skew` have no remaining hits under `rust/crates`. `git diff 270207ffa HEAD -- rexx-core/src/body.rs` is empty.
* Documented positions on a BOTH stream: ADDRESSED. Crate gives `11 16`, `tail` at 11, `seek <1 write` 18, 3-line file `small 11 16 21 ...6E65770A7461696C0A6D6F72650A` (no NULs). 5400-byte file: write position 14, size 5400, line 456 intact. Relative SEEK `+3 read` after a read at 1 gives 4 (from the named position), on both `both` and `both nobuffer`. The crate differs from the oracle exactly on the cases row 49 states.
* Licensed rows: ADDRESSED. `write-position-after-buffered-read` and `relative-seek-from-named-position` are well formed. Both markers are in row 49. `licensed_divergences` test: 22 passed. My own run of the same programs reproduces the recorded oracle and crate stdout (oracle `small 23 28 33`, `big 4101`, `<1 write` 18/20, `+3 read` 23/25).
* Phase 7 program on NOBUFFER: ADDRESSED. Its purpose (LINEOUT with a line number overwrites in place and leaves the uncovered tail, write and line positions, byte-level result) is kept. It no longer reproduces the buffered oracle's skewed pointer, which Deviation 49 deliberately drops. Run directly, it is byte-identical to the oracle, as is the new `stream_nobuffer_positions`. `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`: 1083 of 1083 matching, 29 passed.
* No regression on sequential read/write and append: ADDRESSED. Probe of replace-write, append, linein/lines/size, and a BOTH write after reads is identical to the oracle. Corpus gate green.

## New breakage in the fix diff

None found.

## Out-of-scope observations

* Deviation 49 prose has the sentence "(Task 6.5 fix round 4 fuzz, 1280 scenarios against the NOBUFFER oracle)" mid-paragraph with an unwrapped long line before "Measured by". Cosmetic. I did not re-run the 1280-scenario fuzz.

## Verdict

Approve. No open findings.
