# Task 8.6 fix round 1 re-review

Reviewed `f7be2ca1e..2a42ffe4a` (code `4f079c0d2`; `4ffc82a21` and `2a42ffe4a` are records). Verdict
**APPROVED** (0 Critical, 0 Important, 1 Minor). Every brief item ADDRESSED.

## What I ran

* HEAD built from the checkout, `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/claude-1000/t86-rr-target
  memcap 8G cargo build --release -j 4 --bin rexx`, exit 0 (working tree: only `progress.md` modified,
  no `rust/` change since `4f079c0d2`). Deleted at the end.
* Probes in `t86-rereview-scratch/probes/`, runner `t86-rereview-scratch/run.sh`: each probe in a
  fresh empty directory per binary (oracle under the standard wrapper; base
  `rexx-base-0fd10a074`, prior head `rexx-head-b53b1de0c` and HEAD under `memcap 2G timeout -k 5
  20`), stdin `/dev/null`, stdout a pipe, a file, `/dev/full` or a closed pipe (`| true`); stdout,
  stderr and rc compared separately. Outputs under `t86-rereview-scratch/runs/`.
* `cargo test -j 4 --release --no-fail-fast -p rexx-exec --test stdout_write_errors --test
  standard_stream_reopen --test stream_model --test stream_handle --test program_end`: all ok
  (4, 2, 26, 6, 8 passed).
* `bench-programs/callgrind.sh -r 1 -j 4 -p "interpret pingguard sendloop"
  pin=.../rexx-head-4f079c0d2 mine=<my build>`: interpret -0.0000%, pingguard +0.0000%, sendloop
  +0.0000% libc subtracted, whole process within 0.0000%. So the binary the report measured is
  `4f079c0d2`'s code.

## Verdicts

1. **Important 1 (APPEND): ADDRESSED.** `open` adds `.append(parsed.append)` to both the read-write
   open and the write-only retry (`dispatch/stream.rs:738-760`). Oracle and HEAD agree on the review's
   `append.rex` (`cmd1|b1|b2|`, `c1|c2|`, `e1|d1|d2|`), `append2.rex`, the witness
   `corpus/lang/stream_append_held.rex` (prior head `b1|1|b2|`, `c1|2|c2|`, `d1|d2|`), and on my
   `a2.rex`: `both append` with reads between writes (`read3 cmd`, file `r1|r2|cmd|A1|A2|`, `rpos 7
   wpos 17`, `lines 3 chars 10`), a stream reopened from `read` to `write append` (`h0|hcmd|h1|h2|`),
   `stream(..., 'c', 'open write append')` then `lineout`/`charout` by name (`kcmd|k1|k2`), an
   implicit LINEOUT on a name after a close, which is not APPEND (`m1|m2|m3|`; base `m1|m2|m3|d|`),
   and `command('open write append')` on a closed object (`n1|ncmd|n2n3|`). `a3.rex`'s 5000-byte
   write past the buffer: oracle and HEAD `big 5003 csz ze` (prior `5002 szz`). Held appends survive
   a SYNTAX end (`a4.rex`: `cmdq|q1|q2|q3|` on both) and `raise halt` (`a5.rex`:
   `z|cmdz|z1|z2|` on both). SEEK and positioned writes on an APPEND stream (`a1.rex`) answer as base
   and the prior head did; see Out of scope.
2. **Important 2 (reopened `.STDOUT`/`.STDERR`): ADDRESSED.** reo1 to reo5, stdout a pipe and a
   file: HEAD `x y z` (reo4 on stderr), the same as base; reo6 1002 lines, `a` first, `end` last
   (oracle 820). My shapes, HEAD equal to base on all three descriptors: `r7.rex` (`open write
   replace`, `.output~lineout`, `command('open write')`, `charout 'STDOUT'`, `close` then writes:
   `a READY: b c d READY: e READY: f g`; prior `c` before `b` and `f` lost); `r9.rex` and `r10.rex`
   (reopen, then `raise halt` or a SYNTAX error: `x y z`, rc 252 and 215; prior `y`); `r11.rex`
   (`exit 3` from a subroutine: `x y`); `r12.rex` (`.stderr~open('write')` with `.error~lineout` and
   `lineout 'STDERR'`: `x y z`; `.stdout~open('write')` around a command: `o1 cmd o2`). Trace order,
   stdout and stderr into one file (`t1.rex`): base and HEAD both `trace lines|e1|o1|s1|o2|c1|o3`,
   Deviation 40's interleave. A program-made `.Stream~new('STDOUT')` stays buffered: `r8.rex` and
   `r13.rex` oracle and HEAD `y w x z` and `s1 p1 s2 s3 lower` (base `x y z w`); `StreamState`'s
   `Clone` clears `local` (`body.rs:737`).
3. **Minor 1 (size): ADDRESSED.** `NativeState::Stream(Box<StreamState>)` and `const _: () =
   assert!(size_of::<NativeState>() <= 72)` (`body.rs:181`, `:310`). 200000
   `.mutablebuffer~new('x')` in an array, max RSS three runs each: base 133936/133812/134288 KB,
   prior 143412/143752/143244 KB, HEAD 111968/111624/110580 KB. `StreamState` holds no `ObjRef`, so
   the box changes no rooting; `init` still releases `held_bytes()` as before.
4. **Minor 3 (FLUSH): ADDRESSED.** `f1.rex` (program-made STDOUT, `.STDOUT`, an unopened one and a
   `NOBUFFER` one), oracle and HEAD byte-identical on stderr: pipe `0 READY: READY: | 0 READY: READY:
   | 0 READY: | READY: | READY: 0 READY: | 0 READY:`; `/dev/full` `0 ERROR:28 ERROR:28 No space left
   on device | 1 READY: ERROR:28 No space left on device | 1 ERROR:28 No space left on device |
   ERROR:28 | READY: 0 ERROR:28 | 1 READY:`; closed pipe the same with `ERROR:32 Broken pipe`. Base
   and prior answer `READY:` for every flush there. The review's `werr.rex`: HEAD's FLUSH column now
   `ERROR:28`/`ERROR:32` as the oracle's; the write columns are candidate A's. Files and FIFOs are
   unchanged (`f3.rex`: FIFO `got fifo1|`, `/dev/full` file `df ERROR:28`, `write append` on
   `/dev/full` `af ERROR:28`, oracle and HEAD equal).
5. **Minor 2 (pingguard): ADDRESSED.** `r1/callgrind-r1.txt` has every program, three rounds each
   (a `head.rN.cg` per program and round in `cg-9d/t8.6/r1/`), against `rexx-base-0fd10a074`, no OVER or SPREAD.
   Libc subtracted the largest is interpret +0.0153%, then define +0.0042%; pingguard +0.0001%,
   pingsem +0.0000%. Every program is under +0.05%, and my spot check above ties the measured binary
   to `4f079c0d2`.

## Concern 1 (`.STDOUT~flush` answers `READY:` after a refused hand-over)

The FLUSH answer itself agrees with the oracle: `f4.rex` (`say 'x'`, then `.stdout~flush`) and
`f5.rex` (`.stdout~lineout('x')`, then `.stdout~flush`) under `/dev/full`: FLUSH answers `READY:` on
both. What differs besides the write answers is the stream's state between the hand-over and the next
write: oracle `READY: ERROR:28 No space left on device` (flush, then DESCRIPTION), HEAD `READY:
READY:`; the next `.stdout~lineout('y')` answers `1 ERROR:28` on both. The ruled sentence ("a write
error on a buffered standard output is answered by the stream writes after the hand-over that met it")
covers this by implication, since FLUSH and DESCRIPTION are not writes. It need not name FLUSH for the
answer, but I recommend it say so outright, as the reader rexxref leaves with is that FLUSH reports
the operating system's error: for example "the hand-over, a FLUSH of `.STDOUT` or `.STDERR` among
them, answers `READY:` and leaves the stream READY until that write". Witness lines: `f4.rex`
`/dev/full` oracle `READY: ERROR:28 No space left on device | 1 ERROR:28 No space left on device |
READY:`, HEAD `READY: READY: | 1 ERROR:28 No space left on device | READY:`.

## New problems

### Critical

None.

### Important

None.

### Minor

1. **APPEND on a file without read permission is a new, unreported divergence from the oracle.**
   The write-only retry now opens `O_APPEND` (`dispatch/stream.rs:755-759`); the oracle's retry is
   `open(WR_CREAT, RX_S_IWRITE, shared)` (`StreamNative.cpp:2466`), which drops `RX_O_APPEND`, so
   its held writes land at the offset the end had at open. `a3.rex`: `printf "y\n" > wo.txt; chmod
   222 wo.txt`, `w~open('write append')`, `w~lineout('w1')`, `'echo wcmd >> wo.txt'`,
   `w~lineout('w2')`, `w~close`: oracle `y|w1|w2|` (the command's line overwritten), base
   `y|w1|wcmd|w2|`, prior `y|w1|d|w2|`, HEAD `y|wcmd|w1|w2|`. HEAD follows the rule (rexxref APPEND
   positions the write pointer at the end, and the read-write case agrees with the oracle), so the
   code is right; but the report says the open is `O_APPEND` "as `SysFile::open` makes it, ... also
   on the write-only retry" without saying the oracle's retry is not. Under ruling 13 this is a
   Deviation candidate for Moritz, or a sentence in an existing one, and an entry in the oracle
   defects list (below). Fix: name it in the report as a candidate; no code change.

## Observations (no finding)

* **The stream box costs streams what it saves elsewhere.** A loop of `.stream~new('f.txt')`,
  `open('write append')`, `lineout`, `close` is flat (base 88164/88136/88160 KB at 160000, 320000,
  640000 iterations; HEAD 92384/93176/92456 KB), about 5% higher than base from the second
  allocation per stream. A loop of unclosed streams, a LINEOUT by name and a program STDOUT CHAROUT
  levels at about 103 MB on both (base 102664/102824 KB at 160000/320000, HEAD 105424/103588 KB).
* **The held bytes' recorded offset goes stale on an APPEND stream that another writer extended.**
  `OpenFile::write` (`body.rs:646-655`) flushes when the new landing does not follow the held bytes
  and then records `held_at` from the offset computed before that flush, so after a command's append
  the stream's write position is neither the file's end nor the oracle's own count: `a4.rex`
  (`q1`, command appends `cmdq`, `q2`) `query position write char` oracle `7`, base `12`, prior and
  HEAD `9`. `O_APPEND` puts the bytes at the end regardless, so the file agrees with the oracle.
  rexxref's positioning rests on nothing else writing the file meanwhile (iostrms.xml:495-500), and
  the prior head answers the same, so it is not from this round.

## Out of scope (predate the round)

1. **Positioned writes on an APPEND stream.** `a1.rex` (file `aaaa|bbbb|`, `open('write append')`):
   after `seek('=1 write char')` (answered `1` on all), oracle reports `pos3 4`, `charout('C', 1)`
   `0` then `pos4 2`, `lineout('L4', 1)` `0`, file `aaaa|bbbb|L1|L2|CL4|`; base, prior and HEAD
   report `pos3 17`, accept the CHAROUT (`0`, `pos4 18`) but refuse the LINEOUT (`w4 1`), file
   `aaaa|bbbb|L1|L2|C`. rexxref funct.xml:4309-4312 says the write pointer cannot move into the file
   as it was opened, which neither the oracle's SEEK nor this crate's SEEK and CHAROUT enforce, and
   this crate's CHAROUT and LINEOUT disagree. Queue candidate.

## New oracle defects

1. **APPEND is lost when the file opens write-only.** `StreamNative.cpp:2466` retries a failed
   read-write open with `WR_CREAT` alone, dropping `RX_O_APPEND`, so `SysFile`'s `flags` lack
   `O_APPEND` and a buffered write lands at the end the file had at open, overwriting what another
   writer appended (`a3.rex`, oracle `y|w1|w2|` where `wcmd` was appended between).
