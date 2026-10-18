# Task 6.5 review: binary and write-only streams

Reviewed `93063dff5..06f0ce3af` (crate change `rust/crates/rexx-exec/src/dispatch/stream.rs` only). Crate built from HEAD `e1bc7e9f8`, whose `rust/crates` is identical to `06f0ce3af` (`git diff --stat 06f0ce3af HEAD -- rust/crates` is empty). Every probe ran on both sides, each from its own fresh `mktemp -d`, oracle under the standard wrapper, crate under `memcap 2G timeout 20`, stdout, stderr (directory normalised) and rc compared separately. Probe sources were in the session scratchpad (`rev65/probes/`), which is not durable, so the deciding lines are quoted below.

## Spec compliance

Spec compliant, with one ruling-scope question for Moritz (finding 1).

* All three clusters addressed, three witnesses, licence row present, cluster lines removed (per report, not re-run).
* `LINEOUT TEST_BINARY` re-run by hand: a replay of `LINEOUT.testGroup` lines 215-233, with `assertFile`'s own `arrayIn` readback. Crate passes 215, 217 (x5), 220, 229, 230 and 231 and fails only 233 with `1    22   three4444 55555`. The oracle passes 233 and gives `1         three     55555`. The licence row covers only the `@@wrong` assertion. (The ooTest harness itself was not re-run: a scratch layout stopped at `rxregexp.cls` error 98, the native `RegExp_Init`.)

## Strengths

* The record model follows `StreamInfo::linein/arrayin/lines/lineout/writeFixedLine` closely. Across every binary probe, the only disagreements were the ones Deviation 46 licenses. The probes covered exact, short and long records, CHAROUT partials, line and char positioning (`=`, `+`, `-`, `<`), positions past the end, RECLENGTH 1, BINARY with no RECLENGTH (read and both), a size that is not a multiple of n, ARRAYIN after CHARIN, ARRAYOUT with an over-long item, and LINES `C`/`N`.
* SEEK offsets match the oracle exactly, including its quirks: `=2**63-1`, `+2**63-1` (the oracle wraps to `-2`, and so does the crate), `<2**63-1`, `-2**63+1`, `=0`, leading zeros, and 93 for `2**63`, `+2**63`, `-2**63` and `2**64`.
* Forward and absolute line seeks over unterminated, terminated, empty and one-byte files, for `+1..5`, `=1..5`, `+9` then `-1`, and a write-line seek then LINEOUT: all 52 lines identical.
* QUERY HANDLE: unopened, open, after close, standard streams, nonexistent file and the write-only fallback stream all match.
* The write-only retry matches `WR_CREAT`, which does not truncate under REPLACE (file bytes identical). The read-only-directory refusal (`ERROR:13`), an existing writable file in a read-only directory, and `/dev/null` (`QUERY SIZE`, `STREAMTYPE`, close) all match.

## Issues

### Critical

None.

### Important

1. **Deviation 46 scope: no-data LINEOUT still overwrites the rest of a partial record.** At `stream.rs` `complete_record`, called from `lineout` and from `close_for_write`, a position-only or closing LINEOUT pads from the write position to the record end. It skips only the record-boundary case. Probe: the file is `AAAABBBBCCCCDDDD`, opened `both binary reclength 4`, then `charout(f,'x',5)` and `lineout(f)`. Both sides give `[AAAAx   CCCCDDDD]`, so `BBB` is blanked by a close. The same holds for `lineout(f,,1)` after `charout(f,'y',9)`.
   * The ruling says "positions and closes as documented and clears no record". The documentation (`funct.xml` LINEOUT, quoted in the test's `@@BUG`) says positioning writes nothing and the no-argument form closes. Nothing in `funct.xml` or `rexio.xml` describes padding a partial record on close.
   * So the report's justification, "the part the oracle and the documentation agree on" (report, root cause 2), is not supported by the docs. The decision may still be right, since it keeps the oracle's answer wherever the licence is silent. But it is the implementer's reading, not the ruling's words. It also does destroy existing record bytes.
   * The controller should put the case to Moritz: does Deviation 46 cover only the whole-record clear, or every byte a no-data LINEOUT writes? Also correct the report sentence.
2. **The write-only fallback leaves read-side queries answering as if the stream were readable.** The open sets `parsed.read_write = false` (`stream.rs` `open`, retry arm), but nothing downstream consults it. The probe file is `line1\nline2\n` with `chmod a-r`, then `open write`. Oracle vs crate:
   * `chars` answers `0` vs `12`. In `StreamInfo::chars`, a stream with `!read_only && !read_write` answers 0.
   * `lines` answers `0` vs `1`, by the same guard in `StreamInfo::lines`.
   * `charin` answers `[] ERROR:9 Bad file descriptor` vs `[] ERROR:0`.
   * `query position write line` answers `ERROR:9 Bad file descriptor` vs `1`, and the state is then `ERROR` vs `READY`.
   * `lineout(f,'x',2)` writes the same bytes on both sides, but `query position write` then answers `3` vs `15`.

   The task made these states reachable, since before it the open answered `ERROR:13`. The focus asked for QUERY and STATE after the fallback, and they diverge. None of these is in the ooTest cluster, so the suite cannot see them. Fix: honour the no-read mode in `chars`/`lines`, report the real errno from the failed read, and fix the line-position walk on an unreadable descriptor.

### Minor

3. **An implicit read open of a file without write permission: `query position write line` answers `2`, where the oracle answers `1`.** Probe: a two-line file, `chmod a-w`, then `say linein(f) stream(f,'c','query position write') stream(f,'c','query position write line')`. The oracle prints `abc 1 1` and the crate `abc 1 2`. The write position itself matches. Not attributed: the base `93063dff5` was not built, so whether this predates the `ensure_open` change is unknown.
4. **`complete_record` swallows a write failure.** In `stream.rs` `complete_record`, the `Err` arm raises NOTREADY and returns `Ok(())`. `lineout` then goes on to position, and `close_for_write` goes on to answer `0`, where the oracle's `completeLine` NOTREADY does not return to its caller. Not probed: it needs a failing write on a record stream.
5. **The device retry is missing.** The oracle retries `WR_CREAT` for `write_only || fileInfo.isDevice()`, and the crate only for `write_only`. So `open both` on a device that refuses read-write answers `ERROR:n` here and opens on the oracle. Not probed: no such device was at hand.

## Assessment

**Task quality:** Needs fixes.

**Reasoning:** The record model, the SEEK parser, line seeking and QUERY HANDLE match the oracle on every probe, and `TEST_BINARY` fails only at its licensed line. But the new write-only state answers CHARS, LINES, CHARIN status and line positions differently from the oracle (finding 2). The partial-record padding kept under Deviation 46 rests on a documentation claim that does not hold, and Moritz needs to rule on it (finding 1).
