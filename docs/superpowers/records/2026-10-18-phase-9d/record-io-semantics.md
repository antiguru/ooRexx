# Record I/O and stream close: derived semantics

Written 2026-10-18 for the ruling on how far Deviation 46 reaches.
The case: a file `AAAABBBBCCCCDDDD` opened `BOTH BINARY RECLENGTH 4`, then `charout(f, 'x', 5)` and `lineout(f)`.
The oracle ends with `AAAAx   CCCCDDDD`, and the task statement reports the same result for this crate.
Deviation 46 (`docs/superpowers/plans/phase-4-exclusions.txt:2376`) covers the case where the write position sits on a record boundary, so the oracle blanks a whole record.
This record asks whether blanking the rest of a *partial* record on close is record semantics or the same defect.

Sources, in the order read:

* `oodocs/rexxref/en-US/` (reference) and `oodocs/rexxpg/en-US/rexio.xml` (programming guide), paths relative to `oodocs/`.
* The ANSI X3.274 draft, `j18pub.txt` in the session scratchpad. Sections cited by number, short phrases only.
* `/home/moritz/dev/repos/ooRexx/interpreter/streamLibrary/StreamNative.cpp` at the oracle's checked-out revision, and its git history.
* `ootest/ooRexx/base/bif/LINEOUT.testGroup`, `CHAROUT.testGroup`, `base/class/Stream.testGroup`.
* Oracle probes, listed in the last section with the program that produced them.

## The documented model

A record stream is a binary stream with a record length.
`BINARY` makes line-end characters ordinary data, and `RECLENGTH` "allows the specification of an exact length for each line", so "line operations on binary-mode streams" operate "on individual fixed-length records" (`rexxref/en-US/funct.xml:4337-4356`, repeated at `streamclasses.xml:1312-1318`).
Without `RECLENGTH`, line operations treat the whole file as one record, "as if the RECLENGTH option were specified with a length equal to that of the file" (`funct.xml:4354-4357`).
Line positioning on a record stream "is an extension of character positioning": line *n* is character `(n-1)*reclength+1` (`iostrms.xml:465-472`).
A record is therefore only a unit of addressing: the documentation describes no record header, terminator or fill character stored in the file.

LINEOUT on a record stream writes a full record.
The programming guide's `direct.rex` example writes short strings with `lineOut` and says "The records are not padded to 60 characters, Rexx itself handles that" (`rexxpg/en-US/rexio.xml:562-565`).
The reference says LINEOUT output "can be added to a partial line previously written to the stream with the CHAROUT routine" and "LINEOUT stops a line at the end of each call" (`funct.xml:3193-3196`).
So LINEOUT with a string completes the current record, padding it, and that padding is documented.

CHAROUT writes characters and nothing else.
"the write position is increased by the number of characters written" (`funct.xml:1399-1402`).
The guide states the consequence for records directly: "Unlike LINEOUT, you need to ensure that the string being written with CHAROUT is padded to the appropriate record length. Otherwise, part of the record being replaced remains in the file" (`rexxpg/en-US/rexio.xml:613-616`).
That sentence describes exactly the case under ruling, a short CHAROUT into an existing record, and says the rest of the record stays.

Closing writes nothing.
LINEOUT with neither string nor line: "the stream is closed. Again, the function returns 0" (`funct.xml:3214-3217`, and the overview at `funct.xml:3181-3185`).
LINEOUT with only a line: "the write position is set to the start of the line that was given, nothing is written to the stream" (`funct.xml:3211-3215`).
CHAROUT with neither string nor start: "the stream is closed" (`funct.xml:1414-1418`).
The `close` method "Closes the stream" (`streamclasses.xml:652`), and the `STREAM` `CLOSE` command "closes the named stream" (`funct.xml:4376-4381`).
The implementation overview treats LINEOUT with only a name as one of several equivalent ways to close (`iostrms.xml:518-529`).
No passage gives a close through LINEOUT a different effect from any other close, and no passage says a close completes a record.

## What the ANSI draft says

The draft has no records in its stream model.
Section 5.8 defines persistent and transient streams and "binary and character streams", nothing record-shaped.
Section A.5.8.10 says the standard "does not define how end-of-line is detected" and that on record systems a line end should correspond to "a junction between records".
Section 9.7 lists properties every successful I/O operation must have, two of which settle the case:

* CHAROUT is "reflective": reading a persistent stream from position 1 returns the concatenation of what CHAROUT put.
* "When a persistent stream is repositioned and written to with CHAROUT, the previously written data is not lost, except for the data overwritten by this latest CHAROUT."

Section 5.8.9, `Config_Stream_Close`, only resets the write positions to the end of the existing data, and A.5.8.9 says it need not have all the characteristics of an operating system close.
Neither writes data.
Under the draft, `AAAAx   CCCCDDDD` loses `BBB`, which no CHAROUT overwrote.

## What the oracle implements

Only `lineout` with no string pads.
`StreamInfo::lineout` (`StreamNative.cpp:2016-2045`), on the no-data path, calls `completeLine(binaryRecordLength - (charWritePosition - 1) % binaryRecordLength)` for any record stream, under the comment "if this is a binary stream, we may have a line to complete", and only then closes or repositions.
`completeLine` (`StreamNative.cpp:1091-1103`) writes that many blanks at the write position.
`charout` with no string (`StreamNative.cpp:1576-1600`) closes or positions without padding.
`streamClose` (`StreamNative.cpp:2130-2141`), behind both the `close` method and `STREAM ... CLOSE` (`stream_close`, `StreamNative.cpp:2160`) and the `uninit` method (`stream_uninit`, `StreamNative.cpp:2177-2191`), calls `close()` (`StreamNative.cpp:487-502`), which pads nothing. Probes b4, w4 and n4 show that a stream left open at program end is not padded either.

The padding formula has no zero case.
When the write position is on a record boundary, `(charWritePosition - 1) % binaryRecordLength` is 0 and the padding is a full record of blanks.
That is the Deviation 46 defect: positioning or closing blanks the whole next record.
When the position is inside a record, the padding is the rest of that record, which is the case under ruling.
Both cases come from the same expression, and the comment's "may have" implies a test for whether a line is pending that the code never makes.

The padding is old.
The block is present at the 2008-06-28 commit `32fea3ff2` ("merge of opt sandbox changes back into trunk"), the earliest commit `git log -S'line to complete'` reaches.
Commit `4b9c8aef1` (2021-02-05, "fix code for [bugs:#1742] Stream RECLENGTH 1 issues") rewrote the formula from `reclength - ((pos % reclength) - 1)` to the current one.
The old formula also gave a full record at a boundary, and underflowed when `pos % reclength` was 0.
The fix was for `RECLENGTH 1` arithmetic, not a decision about close semantics.
History therefore shows the padding is deliberate in the sense of written on purpose, with no recorded reason, and the history does not reach before 2008.

The oracle departs from its documentation in three ways:

* LINEOUT with only a line writes blanks, where the reference says "nothing is written" (`funct.xml:3211-3215`).
* LINEOUT with only a name writes blanks, where every other documented close does not.
* A short CHAROUT into an existing record loses the rest of that record when LINEOUT closes, where the guide says it "remains in the file" (`rexxpg/en-US/rexio.xml:615-616`).

## What ooTest pins

`LINEOUT.testGroup:212-233`, `test_binary`, is the boundary case.
It writes five records with `lineout(name, n~copies(n), n)` and asserts `1    22   333  4444 55555`, the documented padding by LINEOUT with a string.
It then marks `lineout(name, , 3)` "@@instead clears record 2" and `lineout(name)` "@@clears record 4", quoting the reference passages as the reason (`@@BUG`, lines 222-228).
The expected file it comments out is `1    22   three4444 55555`, and the oracle's actual file is asserted with `-- @@wrong`.

No test pins the partial-record case.
`CHAROUT.testGroup:910-937`, `test_simple_bin`, writes 26 characters to a `RECLENGTH 8` stream, closes with `STREAM CLOSE`, and reads back 26 characters, so the close command is tested on a partial last record and pads nothing visible to that test.
`CHAROUT.testGroup:939-965`, `test_pos_bin`, asserts `query size` equals the characters written to a `RECLENGTH 23` stream, without a close.
`Stream.testGroup` opens `BINARY RECLENGTH` streams only for reads and `arrayin` (`Stream.testGroup:636-760`).

## Probes

Oracle: `/home/moritz/dev/repos/ooRexx/build/bin/rexx`, run 2026-10-18 from a fresh empty directory under the session scratchpad, rc 0, no NOTREADY.
Each variant creates `AAAABBBBCCCCDDDD`, closes it, opens it with the options shown, writes `x` at character 5 (variants b7 and b8 write `xxxx`), then runs the action.
Blanks are shown as `_`.
The "documented" column applies the reference and guide passages above: CHAROUT writes only its characters, LINEOUT with a string completes the record, every close writes nothing.

| Id | Open | Action | Oracle | Documented | Agree |
|---|---|---|---|---|---|
| b1 | `BOTH BINARY RECLENGTH 4` | `lineout(f)` | `AAAAx___CCCCDDDD` | `AAAAxBBBCCCCDDDD` | no |
| b2 | `BOTH BINARY RECLENGTH 4` | `lineout(f, 'y')` | `AAAAxy__CCCCDDDD` | `AAAAxy__CCCCDDDD` | yes |
| b3 | `BOTH BINARY RECLENGTH 4` | `stream(f, 'C', 'CLOSE')` | `AAAAxBBBCCCCDDDD` | `AAAAxBBBCCCCDDDD` | yes |
| b4 | `BOTH BINARY RECLENGTH 4` | none, program end | `AAAAxBBBCCCCDDDD` | `AAAAxBBBCCCCDDDD` | yes |
| b5 | `BOTH BINARY RECLENGTH 4` | `charout(f)` | `AAAAxBBBCCCCDDDD` | `AAAAxBBBCCCCDDDD` | yes |
| b6 | `BOTH BINARY RECLENGTH 4` | `lineout(f, , 4)`, then `lineout(f)` | `AAAAx___CCCC____` | `AAAAxBBBCCCCDDDD` | no |
| b7 | `BOTH BINARY RECLENGTH 4`, `xxxx` at 5 | `lineout(f)` | `AAAAxxxx____DDDD` | `AAAAxxxxCCCCDDDD` | no |
| b8 | `BOTH BINARY RECLENGTH 4`, `xxxx` at 5 | `stream(f, 'C', 'CLOSE')` | `AAAAxxxxCCCCDDDD` | `AAAAxxxxCCCCDDDD` | yes |
| w1 | `WRITE BINARY RECLENGTH 4` | `lineout(f)` | `AAAAx___CCCCDDDD` | `AAAAxBBBCCCCDDDD` | no |
| w2 | `WRITE BINARY RECLENGTH 4` | `lineout(f, 'y')` | `AAAAxy__CCCCDDDD` | `AAAAxy__CCCCDDDD` | yes |
| w3 | `WRITE BINARY RECLENGTH 4` | `stream(f, 'C', 'CLOSE')` | `AAAAxBBBCCCCDDDD` | `AAAAxBBBCCCCDDDD` | yes |
| w4 | `WRITE BINARY RECLENGTH 4` | none, program end | `AAAAxBBBCCCCDDDD` | `AAAAxBBBCCCCDDDD` | yes |
| w5 | `WRITE BINARY RECLENGTH 4` | `charout(f)` | `AAAAxBBBCCCCDDDD` | `AAAAxBBBCCCCDDDD` | yes |
| n1 | `BOTH BINARY` | `lineout(f)` | `AAAAx___________` | `AAAAxBBBCCCCDDDD` | no |
| n2 | `BOTH BINARY` | `lineout(f, 'y')` | `AAAAxy__________` | `AAAAxy__________` | yes |
| n3 | `BOTH BINARY` | `stream(f, 'C', 'CLOSE')` | `AAAAxBBBCCCCDDDD` | `AAAAxBBBCCCCDDDD` | yes |
| n4 | `BOTH BINARY` | none, program end | `AAAAxBBBCCCCDDDD` | `AAAAxBBBCCCCDDDD` | yes |
| n5 | `BOTH BINARY` | `charout(f)` | `AAAAxBBBCCCCDDDD` | `AAAAxBBBCCCCDDDD` | yes |
| t1 | `BOTH` | `lineout(f)` | `AAAAxBBBCCCCDDDD` | `AAAAxBBBCCCCDDDD` | yes |
| t2 | `BOTH` | `lineout(f, 'y')` | `AAAAxy` LF `BCCCCDDDD` | same | yes |
| t3 | `BOTH` | `stream(f, 'C', 'CLOSE')` | `AAAAxBBBCCCCDDDD` | `AAAAxBBBCCCCDDDD` | yes |
| t4 | `BOTH` | none, program end | `AAAAxBBBCCCCDDDD` | `AAAAxBBBCCCCDDDD` | yes |
| t5 | `BOTH` | `charout(f)` | `AAAAxBBBCCCCDDDD` | `AAAAxBBBCCCCDDDD` | yes |
| u1 | `WRITE` | `lineout(f)` | `AAAAxBBBCCCCDDDD` | `AAAAxBBBCCCCDDDD` | yes |
| u3 | `WRITE` | `stream(f, 'C', 'CLOSE')` | `AAAAxBBBCCCCDDDD` | `AAAAxBBBCCCCDDDD` | yes |

All files are 16 bytes.
Every disagreement is a LINEOUT without a string on a binary stream, and the open mode does not matter.
Text streams never pad.
`n1` shows the reach of the same expression without `RECLENGTH`: the record is the whole file, so `lineout(f)` blanks everything after the write position.

The probe program, `probe.rex`, run as `rexx ../../recio-src/probe.rex` from the empty directory:

```rexx
signal on notready name nr
init = 'AAAABBBBCCCCDDDD'
call v 'b1', 'BOTH BINARY RECLENGTH 4', 'call lineout f'
call v 'b2', 'BOTH BINARY RECLENGTH 4', 'call lineout f, "y"'
call v 'b3', 'BOTH BINARY RECLENGTH 4', 'call stream f, "C", "CLOSE"'
call v 'b4', 'BOTH BINARY RECLENGTH 4', 'nop'
call v 'b5', 'BOTH BINARY RECLENGTH 4', 'call charout f'
call v 'b6', 'BOTH BINARY RECLENGTH 4', 'call lineout f, , 4; call lineout f'
call v 'b7', 'BOTH BINARY RECLENGTH 4', 'call lineout f', 'xxxx'
call v 'b8', 'BOTH BINARY RECLENGTH 4', 'call stream f, "C", "CLOSE"', 'xxxx'
call v 'w1', 'WRITE BINARY RECLENGTH 4', 'call lineout f'
call v 'w2', 'WRITE BINARY RECLENGTH 4', 'call lineout f, "y"'
call v 'w3', 'WRITE BINARY RECLENGTH 4', 'call stream f, "C", "CLOSE"'
call v 'w4', 'WRITE BINARY RECLENGTH 4', 'nop'
call v 'w5', 'WRITE BINARY RECLENGTH 4', 'call charout f'
call v 'n1', 'BOTH BINARY', 'call lineout f'
call v 'n2', 'BOTH BINARY', 'call lineout f, "y"'
call v 'n3', 'BOTH BINARY', 'call stream f, "C", "CLOSE"'
call v 'n4', 'BOTH BINARY', 'nop'
call v 'n5', 'BOTH BINARY', 'call charout f'
call v 't1', 'BOTH', 'call lineout f'
call v 't2', 'BOTH', 'call lineout f, "y"'
call v 't3', 'BOTH', 'call stream f, "C", "CLOSE"'
call v 't4', 'BOTH', 'nop'
call v 't5', 'BOTH', 'call charout f'
call v 'u1', 'WRITE', 'call lineout f'
call v 'u3', 'WRITE', 'call stream f, "C", "CLOSE"'
exit 0

v: procedure expose init
  parse arg name, opts, action, data
  if data == '' then data = 'x'
  f = name'.dat'
  call stream f, 'C', 'OPEN WRITE REPLACE'
  call charout f, init
  call stream f, 'C', 'CLOSE'
  r = stream(f, 'C', 'OPEN' opts)
  c = charout(f, data, 5)
  interpret action
  say name 'open='r 'charout='c 'state='stream(f, 'S')
  return

nr:
  say 'NOTREADY' condition('D') 'at line' sigl
  exit 1
```

## Conclusion (my reading)

Padding a partial record on close is a defect, the same defect as Deviation 46, not record semantics.
The documentation pads in exactly one place, LINEOUT with a string, and says in so many words that CHAROUT does not pad and that the rest of the record "remains in the file".
Every documented close, LINEOUT's included, is described as a close and nothing else, and the ANSI draft requires that a CHAROUT loses no data beyond what it overwrote.
The oracle itself pads on only one of its four close paths, so a program that closes with `STREAM CLOSE`, `charout(f)` or by ending keeps `BBB`, and one that closes with `lineout(f)` loses it.
The padding is deliberate code, present since at least 2008, but no source states the intent, and the boundary case proves the expression was never tested against a stream that had no pending line.

The justifiable semantics: LINEOUT with a string completes its record by padding, and nothing else ever pads.
A close or a LINEOUT positioning writes nothing, whether the write position is on a record boundary or inside a record.
On this reading Deviation 46's licence extends to the partial case, and `AAAAxBBBCCCCDDDD` is the result for the case under ruling.
One argument for the oracle exists: "may have a line to complete" suggests the authors meant LINEOUT to finish a CHAROUT-started record before closing, by analogy with LINEOUT's documented behaviour on a string.
I reject it because the guide's CHAROUT sentence speaks to this case directly, and because a closing call that changes data the program never wrote, depending on which of the equivalent close spellings was used, is not a semantics a program can rely on.

## Second question: a FIFO write after a buffered write

The documentation settles it: a write to a transient stream has no position, so nothing may seek.
"In a transient stream, no read position is available", and SEEK and POSITION "are not applicable to transient streams" (`iostrms.xml:186-197`).
Write positions are documented only "For persistent streams" (`funct.xml:1399`, `funct.xml:3192`), and only persistent streams can be repositioned (`iostrms.xml:226-237`, `iostrms.xml:449-451`).
ANSI 5.8.5 distinguishes writing "to a transient stream" from writing "at position" of a persistent stream.

The oracle is internally inconsistent here too.
`writeBuffer` skips position bookkeeping for a transient stream (`StreamNative.cpp:1024-1034`), and `setCharWritePosition` and `setLineWritePosition` raise an error for one (`StreamNative.cpp:1272-1275`, `1292-1297`).
`writeSetup` (`StreamNative.cpp:918-944`) compares the OS position against `charWritePosition` and seeks with no transient check, which is the `ERROR:29 Illegal seek` in the queued note.
This is an oracle defect, and a Deviation that writes sequentially to a FIFO matches the documentation.
