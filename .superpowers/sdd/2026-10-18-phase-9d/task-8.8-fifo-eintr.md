# Task 8.8 FIFO EINTR and HALT proof

The required behavior is not satisfied by the frozen release pin. A signal delivered to the verified thread blocked in a named-FIFO read produces EINTR, but the crate immediately reads again and requires a controlled writer unblock. It then returns the supplied byte and handles HALT with `READY:`, without reporting `ERROR:4`. The oracle reports `ERROR:4`, returns no bytes, handles HALT and exits without another write, so the queued item must remain open.

## Provenance and scope

The executable is the private copy `/tmp/t88-fifo-eintr-20261030-1H/rexx-frozen`, copied from `/tmp/codex-p9d-t88-pins/8.8e1-rexx`. Its verified SHA-256 is `434d6567458248fdb804eebe9edb715a54ddb9ded650fcca994eed6c4b691e06`, with runtime source `1d8b03220` as recorded by Task 8.8e1. The diagnostic reads the full queued `2026-10-05-stream-fifo-terminal-seek.md`, the original Task 21 re-review F-2 and the crash exclusions before invoking the oracle. No crash-exclusion program, source change, build, index change, HEAD change or numbered Deviation was part of this task.

The oracle is `/home/moritz/dev/repos/ooRexx/build/bin/rexx` with `LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib`. Its bounded `-v` invocation returned status 0 and reported Open Object Rexx Version 5.3.0 r0, build date Jul 30 2026, 64-bit. Oracle children use `RLIMIT_AS=1073741824`, equivalent to the documented `ulimit -v 1048576`, and fresh owned run directories. Each interpreter has a 15-second diagnostic deadline with owned cleanup, the crate additionally runs under `memcap 2G`, and all foreground drivers run under `memcap 8G`.

## Readiness and signal ownership

Each side receives its own named FIFO and an owned `O_RDWR | O_NONBLOCK` writer descriptor. The driver supplies exactly `abc`, and the Rexx program completes `charin(f,,3)` before writing and closing a separate regular-file `READY` marker. The final oracle control waits for the exact complete `READY\n` marker, and both retained successful descriptors record that content. The driver then finds a thread in its own un-reaped traced child with syscall 0, resolves that syscall's descriptor to the exact owned FIFO path, and checks the task's TGID, executable, start identity and descendant relationship before signaling it.

The decisive crate run is `crate-r2`, with TGID 1020957, start tick 37504350, TID 1020958 named `rexx-interp` and FIFO descriptor 6. The decisive oracle run is `oracle-r3`, with TGID/TID 1021451, start tick 37505332 and FIFO descriptor 5. The checked Python libc `tgkill(tgid, tid, SIGINT)` call returned 0 for each, and both traces identify delivery as `SI_TKILL`. No process was selected by a name match, and the verification audit finds no remaining `/proc` entry for any PID retained by the diagnostic's traces.

## Observed ordering

The crate's reading-thread trace is `/tmp/t88-fifo-eintr-20261030-1H/crate-r2/trace.1020958`. It first reads the exact warmup bytes, then blocks in the intentionally empty FIFO read after the ready marker. SIGINT causes `rt_sigreturn` to return `-1 EINTR`, followed by a new `read` on the same FIFO descriptor. The driver observes that new syscall blocked, verifies that the handler-event file does not exist, and only then writes `Z` to let the run finish.

```text
16:58:03.344616 read(6<.../crate-r2/pipe>, "abc", 3) = 3
16:58:03.344975 read(6<.../crate-r2/pipe>, ..., 1) = ? ERESTARTSYS
16:58:03.354800 --- SIGINT ... SI_TKILL ... ---
16:58:03.355058 rt_sigreturn({mask=[]}) = -1 EINTR
16:58:03.355086 read(6<.../crate-r2/pipe>, "Z", 1) = 1
```

The crate exits with status 0 in 0.158 seconds and empty stderr. Its exact stdout below contains no `ERROR:4` or NOTREADY event. The separate handler-event file contains only `HALT READY:`, and its write follows the successful `Z` read in the trace. Thus this is a real userspace retry after EINTR, with HALT delayed until the controlled unblock.

```text
warm [abc]
read [Z]
HALT READY:
description READY:
```

The oracle's reading-thread trace is `/tmp/t88-fifo-eintr-20261030-1H/oracle-r3/trace.1021451`. Its warmup read asks for 4096 bytes and receives exactly the same three warmup bytes, after which the intended second read blocks. SIGINT produces `rt_sigreturn(...)= -1 EINTR`, with no subsequent read of that FIFO before close. The oracle then exits status 0 in 0.046 seconds with empty stderr, without a controlled writer unblock.

```text
16:58:13.093478 read(5<.../oracle-r3/pipe>, "abc", 4096) = 3
16:58:13.093804 read(5<.../oracle-r3/pipe>, ..., 4096) = ? ERESTARTSYS
16:58:13.098807 --- SIGINT ... SI_TKILL ... ---
16:58:13.098838 rt_sigreturn({mask=[]}) = -1 EINTR
16:58:13.100357 close(5<.../oracle-r3/pipe>) = 0
```

The oracle stdout records NOTREADY with `ERROR:4 Interrupted system call`, then `read []`, then HALT with that same stream description. Its separate handler-event file records NOTREADY before HALT, preserving the order independently of the stdout buffering. The only writer data for this run is the initial `abc`, so EOF or a later byte cannot explain its completion. This is the behavior the original F-2 requirement asks the crate to preserve.

```text
warm [abc]
NOTREADY /tmp/t88-fifo-eintr-20261030-1H/oracle-r3/pipe ERROR:4 Interrupted system call
read []
HALT ERROR:4 Interrupted system call
description ERROR:4 Interrupted system call
```

## Partial-data boundary control

The controller requested one additional boundary control before implementation, using `charin(f,,2)` after the same three-byte warmup. The driver supplies a known `P`, verifies a completed one-byte FIFO read containing it, and then targets the verified next blocked read with SIGINT. The crate run `partial-crate-r1` uses TGID 1028056, start tick 37517875, TID 1028057 and descriptor 6, while `partial-oracle-r1` uses TGID/TID 1028068, start tick 37517892 and descriptor 5. Both use the same complete-marker readiness and checked `tgkill` ownership procedure as the clean empty-read witness.

The partial crate trace records `read(...,"P",2)=1`, a blocked remaining-byte read, `rt_sigreturn(...)= -1 EINTR`, and another remaining-byte read that needs the driver's `Z`. It exits status 0 in 0.191 seconds with `read [PZ]`, `HALT READY:` and empty stderr, and no handler-event file exists before the unblock. The oracle trace records `read(...,"P",4096)=1`, then the blocked read and EINTR, with no subsequent FIFO read before close. It exits status 0 in 0.045 seconds with NOTREADY `ERROR:4`, `read []`, HALT `ERROR:4` and empty stderr, without another write, establishing that this error path discards the accumulated `P` from the returned string.

## Source interpretation and bounded follow-up

At `1d8b03220`, `rust/crates/rexx-exec/src/dispatch/stream.rs:857` routes transient reads through `take(...).read_to_end(&mut out)`, while persistent reads use `lines.rs:35` and retain seeking. The observed fresh syscall after `rt_sigreturn` is consistent with the transient `read_to_end` call retrying Interrupted before the existing `charin` error mapping at `dispatch/stream.rs:1320` can see it. The crate's `signal.rs:62` installs handlers with flags 0, and the actual sigaction and rt_sigreturn traces rule out kernel `SA_RESTART` as the explanation. The oracle's `common/platform/unix/SysFile.cpp:402` and `:441` read loops instead retain errno and return failure after a negative read, matching the measured error path.

The smallest responsible follow-up is to preserve bounded accumulation for transient reads while propagating Interrupted to the existing stream error handler, then use these owned FIFO witnesses as regression checks. That follow-up must verify `ERROR:4`, an empty read result even after a partial byte was consumed, no new FIFO read after EINTR and subsequent HALT handling without a writer unblock. This diagnostic does not extend P60 or Deviation 12, and it makes no claim about all process-directed signal schedules or PTY EINTR behavior. The separate oracle closing-LINEOUT byte-consumption defect and Deviation 46 remain unchanged and controller-owned.

## Commands and retained evidence

The exact interpreter commands and working directories are retained in each `descriptor.json`, with distinct `stdout`, `stderr`, source and per-thread `trace.*` files. The command shape is `strace -ff -tt -T -yy -e trace=read,write,openat,close,lseek,rt_sigaction,rt_sigprocmask,rt_sigreturn -o DIR/trace memcap 2G PRIVATE_REXX DIR/probe.rex` for the crate, and the same strace options followed by the oracle executable and source for the oracle. The Python driver applies the oracle library path and address-space limit before exec, so no shell redirection or environment prefix is needed. Raw evidence remains under `/tmp/t88-fifo-eintr-20261030-1H`, with source snapshots, hashes, verification results and a SHA-256 manifest for preservation by the controller.

| Foreground command | Status | Result |
| --- | --- | --- |
| `memcap 8G python3 /tmp/t88-fifo-eintr-20261030-1H/driver.py` initial run | 1 | Source-only harness error: adjacent `x` parsed as a hexadecimal-string suffix, before readiness or signal. Crate status 241 and trace retained under `crate/`. |
| Same command, r1 | 0 | Counterexample on both sides. An unrelated explicit `OPEN READ NOBUFFER` clause reports ERROR:2 before the successful warmup, so it was removed from the final witness. |
| Same command, r2 | 1 | Decisive crate counterexample status 0. Oracle control hit a marker-file existence versus flush race before signal, then its verified owned child was killed and the tracer reaped. |
| `memcap 8G python3 /tmp/t88-fifo-eintr-20261030-1H/driver.py oracle` r3 | 0 | Complete-marker readiness control, oracle status 0 and required ERROR:4/HALT sequence. |
| `memcap 8G python3 /tmp/t88-fifo-eintr-20261030-1H/driver.py --partial` | 0 | Single requested partial-data boundary. Both status 0, crate retries and returns PZ after unblock, oracle reports ERROR:4 and returns no accumulated data without unblock. |
| `memcap 8G python3 /tmp/t88-fifo-eintr-20261030-1H/verify.py` | 0 | Checks executable hash, exact outputs, read/EINTR ordering, statuses, deadlines and absence of remaining traced PIDs. |

All driver invocations completed in the foreground in less than one second, below the 540-second outer bound. The failed controls and their raw outputs remain retained, and no interpreter exceeded its 20-second limit. Each crate/oracle pair executes the same warmup/read/trap source except for its private absolute paths, with the partial boundary changing only the requested read count and supplied byte sequence. The required pass is disproved rather than inferred from eventual HALT handling after an external unblock.
