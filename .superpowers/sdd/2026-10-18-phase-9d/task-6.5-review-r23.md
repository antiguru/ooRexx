# Task 6.5 review, fix rounds 2 and 3

Reviewed `git diff 270207ffa 5d2bdbcf3 -- rust/ docs/superpowers/plans/phase-4-exclusions.txt`.
Crate built from the working tree at `5d2bdbcf3` (`rust/` clean) into a scratch target dir.
Oracle `/home/moritz/dev/repos/ooRexx/build/bin/rexx` under `ulimit -v 1048576`.
Every probe ran from its own fresh empty directory under the session scratchpad (`rev65b/`).

## Verdicts

* Round 2, spec compliance: **issues.** The named sequence no longer writes NULs, but the round reproduces an oracle defect instead of removing data damage. The defect corrupts data in sibling sequences, and the round contradicts Moritz's standing principle. The implementer flagged it as a concern and did not raise a Deviation.
* Round 2, task quality: **needs rework.** The model is faithful to the buffered oracle (139 of my 140 scenarios identical, including files over 4096 bytes). It ignores `NOBUFFER`, where the oracle gives the documented answer and the crate does not.
* Round 3, spec compliance: **compliant.** It implements Deviation 46 as extended.
* Round 3, task quality: **approved.** The witness row is well formed and reproduces on both sides.

## Recommendation

Replace the buffer model with the documented semantics and ask Moritz for a Deviation.
Evidence follows.

## 1. Documented semantics on a `BOTH` stream

* Read and write positions are separate (`rexxref iostrms.xml:226-227`, "It is separate from the current read position").
* "Any write to the stream starts at the current write position", and CHAROUT increases it "by the number of characters written" (`funct.xml:1399-1402`). LINEOUT sets it "to the beginning of the line following the one just written" (`funct.xml` LINEOUT, write position paragraph).
* SEEK `+`/`-` is relative to "the current read or write position" (`funct.xml` SEEK, `+` and `-` entries). QUERY POSITION WRITE "returns the current write position".
* ANSI 9.7, cited in `record-io-semantics.md`: repositioned writes lose no data except what they overwrite.

No passage makes a position depend on a buffer.

## 2. Where the oracle's buffer changes the answer

The root defect is `SysFile::write` (`common/platform/unix/SysFile.cpp`, the `!writeBuffered` branch). On the switch from read to write it does `lseek64(fileHandle, offset, SEEK_SET)` and never sets `filePointer = offset`. `getPosition` then answers `filePointer - bufferedInput + bufferPosition` from the stale `filePointer`, and `writeBuffer` stores that answer as `charWritePosition`. The Windows `SysFile.cpp` has the same branch.

The cases, each contradicting the docs:

* (a) A LINEOUT with a line number on a buffered stream. Its line walk fills the read buffer, so QUERY POSITION WRITE reports the buffer's end plus the bytes written. Line 2 of a 5400-byte file reports 4101, where the docs give 14. A three-line file reports 23, where the docs give 11.
* (b) The next unplaced write lands correctly, but the reported position stays wrong, and so does every relative SEEK taken from it.
* (c) After (a), any read, CHARS or LINES makes `writeSetup` seek to the wrong `charWritePosition`. The next write then lands there, which damages data:
  * On a small file, the write lands past EOF and fills the gap with NULs (`line1|new|2|line3|@@@@tail|`).
  * On a 5400-byte file, `lineout(f,'new',2); linein(f); lineout(f,'tail')` overwrites bytes 4101-4105, inside line 456, a place the program never named (targeted t1, t2, t4, t7).
  * Line 457 and later produce NULs past EOF (t6, t16-t18).
* (d) A relative SEEK moves from `SysFile`'s tell (`SysFile::seek`, `SEEK_CUR`), not from the read or write position it names.
* (e) With `OPEN BOTH NOBUFFER` the oracle has no buffer, and it gives exactly the documented answer: `w1=11 w2=16`, `line1|new|tail|e3|`. **The crate gives the buffered answer there** (`w1=23 w2=28`, NULs), so it now differs from the oracle on that stream and damages data the oracle keeps.

Case (e) shows the documented semantics is the oracle's own unbuffered behaviour. The buffered behaviour is an accident of one missing assignment.

## 3. The implementer's NUL sequences

Both were run on both sides (`gen/q3.rex`). `lineout(f,'new',2)`, then `linein(f)` or `chars(f)`, then `lineout(f,'tail')` gives `w1=23 w2=28 size=27 [line1|new|2|line3|@@@@tail|]` on both sides. That is a defect, not documented behaviour. The docs place `tail` at 11 and report 11 then 16. On a file over 4096 bytes the same sequence overwrites unrelated data instead of padding (case c).

## 4. Files over 4096 bytes

The file had 600 lines, 5400 bytes, and the probe was `gen/targeted.rex`. It had 20 directed scenarios at lines 2, 455, 456, 457, 590 and 600, at chars 4090-4100, and with reads at 4090, 4093, 4200 and 5000. **All 20 are byte-identical to the oracle**, so the 4096 model holds there. The `LINE_POSITIONING_BUFFER` reasoning is redundant but harmless, because `SysFile::read` always refills a whole `bufferSize`.

## 5. Random sequences

`gen/gen.py` covers LINEIN, LINEOUT (with and without a string or line), CHARIN (with and without a start), CHAROUT, CHARS, LINES, SEEK (=, <, +, -, read, write, line) and QUERY POSITION.

| Run | File | Scenarios | Differ |
|---|---|---|---|
| `big1` | 5400 bytes | 40 | 1 |
| `big2` | 5400 bytes | 40 | 0 |
| `small3` | 18 bytes | 40 | 0 |

The one difference, `big1` s18, is `S -2 write` driving the position to 0 and the next `charin(f,,2)`. The oracle answers `00` and the crate `l0`, which is the documented answer. It falls in the implementer's "position driven negative" class.

I did not re-run the implementer's 640 scenarios, because its harness needs a round-1 binary. Its `summary.txt` is consistent with what I measured.

## Proposed Deviation (for Moritz)

On a persistent stream, a write lands at, and QUERY POSITION WRITE reports, the documented write position. A relative SEEK moves from the read or write position it names. This is what the oracle does on a `NOBUFFER` stream.

The oracle's buffered path loses its file pointer on the switch from read to write (`SysFile::write`). After a line-numbered LINEOUT it reports a position past the write. After a following read it writes there, past EOF with NUL fill, or over unrelated data in files larger than its 4096-byte buffer.

The cost of the change:
* Delete `read_window`, `write_skew`, `oracle_tell` and the helpers built on them.
* Restore `write_position = at + len`.
* Make SEEK `+`/`-` relative to the named position.
* Turn `lang/stream_write_position_skew.rex` into a licensed-divergence row, or narrow it to the cases with no buffer in play.

Round 2's suite run changed no ooTest row while moving 235 fuzz scenarios, so ooTest does not pin this behaviour.

## Round 3

* `complete_record` is gone, and `close_for_write` and `lineout` without data write nothing. `write_record` (LINEOUT with a string) is unchanged. This matches row 46 as extended.
* The witness program from `licensed_divergences.rs` ran by hand on both sides. The oracle printed `r AAAAx___CCCCDDDD / p AAAAx___CCCC____ / n AAAAx___________` and the crate printed `AAAAxBBBCCCCDDDD` three times. Both match the row's `oracle_stdout` and `crate_stdout`, with rc 0 on both.
* The marker line is in row 46, and the name is on one line.

## Findings

* **Important**, round 2, `land_write`/`note_read` (`dispatch/stream.rs`). The crate deliberately reproduces `SysFile::write`'s lost `filePointer`, including overwriting unrelated bytes (line 456 of a 5400-byte file) and NUL fill. This contradicts `funct.xml` and needs a Deviation, not emulation.
* **Important**, round 2. `OPEN BOTH NOBUFFER` followed by `lineout(f,'new',2); linein(f); lineout(f,'tail')` gives the oracle `w1=11 w2=16`, no NULs. The crate gives `w1=23 w2=28` with NULs. The model ignores NOBUFFER.
* **Important**, round 2, `position()`. A relative SEEK from `oracle_tell` contradicts `funct.xml`'s "+/- from the current read or write position". It is the same defect class.
* **Minor**, round 2, `body.rs`. `OpenFile` in `rexx-core` gains three fields that only mirror oracle internals, outside the task's files. This goes away with the recommendation.
* **Minor**, round 2, `lineout`. The line-count cache grows when `write_position == size` ("one short of the end"). This copies an oracle off-by-one, and its observability is unmeasured.
* **Minor**, round 2. The `land_write` doc comment states the oracle's corrupting answer as the measured rule, with no mention that it is a defect.
* **Minor**, round 2, `big1` s18. After a negative relative SEEK, the crate's `charin` answers `l0` and the oracle answers `00`. This is pre-existing ("position driven negative"), and the crate's answer is the documented one.
* **None**, round 3. Compliant, and the witness is well formed and reproduced.

Probe programs are in the session scratchpad under `rev65b/gen/`, and the outputs are in `rev65b/o/` (oracle) and `rev65b/c/` (crate).
