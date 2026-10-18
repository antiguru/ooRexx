**CLOSED: Deviation 48, ruled by Moritz 2026-10-18.** This crate writes to a transient stream without seeking. The oracle defect is not copied.

# Oracle: writes to a FIFO fail after a buffered write

Found 2026-10-18 by Phase 9d Task 6.4 fix round 1 (`docs/superpowers/records/2026-10-18-phase-9d/t6.4/r1/`).
On a FIFO, each write that follows a still-buffered write fails with `NOTREADY ERROR:29 Illegal
seek`, and the write after that succeeds. A close after a buffered write raises NOTREADY the same
way. The cause: `SysFile::getPosition` answers the buffer offset, while `charWritePosition` never
advances for a transient stream, so `writeSetup` tries to seek. This crate does not emulate this:
its FIFO writes succeed. CHARS and LINES on a FIFO depend on the same buffer. While the write was
buffered, the oracle answered 0 and then hung on LINES. Here they answer from the file size.
Proposed: a numbered Deviation (an oracle bug, not copied). No ooTest test reaches it.
