# Task 8 report: named semaphores

**Status: BLOCKED at the Step 1 gate.** A semaphore name on Unix is a POSIX named semaphore
(`sem_open`), a system object shared between processes. Nothing was implemented. No source file
changed.

## Step 1: what a name means in the oracle

Oracle tree `/home/moritz/dev/repos/ooRexx`, HEAD `4b66b3558f7c`,
`interpreter/platform/unix/SysRexxUtil.cpp`:

* `:622`-`:626` the handle is a malloc'd `RXSEMDATA { bool named; sem_t *handle; }`, returned to
  Rexx as its address (`context->Uintptr`).
* `:668` `SysCreateEventSem(name)`:
  `sem_open(name, (O_CREAT | O_EXCL), (S_IRWXU | S_IRWXG), 0)`; on `SEM_FAILED` it returns `''`.
* `:698` `SysOpenEventSem(name)`: `sem_open(name, 0)`; on `SEM_FAILED` it returns `0`.
* `:893` `SysCreateMutexSem(name)`: the same `sem_open(..., O_CREAT | O_EXCL, ...)`, then
  `sem_post` at `:901`.
* `:925` `SysOpenMutexSem(name)`: `sem_open(name, 0)`.
* `:761`-`:797` `SysCloseEventSem` on a named handle calls `sem_close` only. No routine calls
  `sem_unlink` (`grep -n sem_unlink interpreter/platform/unix/SysRexxUtil.cpp` prints nothing), so
  a name outlives the process that created it, in `/dev/shm/sem.<name>`, until something outside
  ooRexx removes it.

So the name is neither an interpreter-wide nor a process-wide table. It is not the rxapi daemon
either. It is the kernel's POSIX named-semaphore namespace (glibc: a mapped file in `/dev/shm`).

## Measurement: two oracle processes, one name

Every run: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )` from a fresh empty directory,
`/dev/shm` checked free of the probe names before and cleaned with literal `rm -f` after.

Probe programs:

```rexx
/* a.rex */
h = SysCreateEventSem('t8xproc')
say 'A create nonempty:' (h \== '')
say 'A post rc:' SysPostEventSem(h)
say 'A close rc:' SysCloseEventSem(h)

/* b.rex */
h = SysOpenEventSem('t8xproc')
say 'B open nonzero:' (h \= 0)
call time 'R'
say 'B wait rc:' SysWaitEventSem(h, 500)
say 'B waited under 0.3s:' (time('E') < 0.3)
say 'B create again empty:' (SysCreateEventSem('t8xproc') == '')
say 'B close rc:' SysCloseEventSem(h)

/* w.rex (w0.rex is the same with 1500 for 8000) */
h = SysCreateEventSem('t8xwait')
say 'W create nonempty:' (h \== '')
call time 'R'
say 'W wait rc:' SysWaitEventSem(h, 8000)
say 'W elapsed under 5s:' (time('E') < 5)

/* p.rex */
h = SysOpenEventSem('t8xwait')
say 'P open nonzero:' (h \= 0)
say 'P post rc:' SysPostEventSem(h)

/* c.rex */
say 'C open never-created:' SysOpenEventSem('t8xnone')
say 'C mutex open never-created:' SysOpenMutexSem('t8xnone')
```

### Sequential: A runs and exits, then B runs (1 run)

```
A create nonempty: 1
A post rc: 0
A close rc: 0
A status 0
-rwxrwx--- 1 moritz docker 32 Oct 13 17:40 sem.t8xproc     <- ls -l /dev/shm between the runs
B open nonzero: 1
B wait rc: 0
B waited under 0.3s: 1
B create again empty: 1
B close rc: 0
B status 0
```

B opens a name a process that has already exited created, consumes the post that process made,
and cannot create the name again. The name and its count outlive the creating process.

### Concurrent: W creates and waits, P starts 1 s later, opens and posts (5 of 5 runs)

```
run 1: W create nonempty: 1|W wait rc: 0|W elapsed under 5s: 1|W status 0| P open nonzero: 1|P post rc: 0|
run 2: (identical)
run 3: (identical)
run 4: (identical)
run 5: (identical)
```

(runs 2-5 printed the same line as run 1 byte for byte.)

### Controls

* No poster: `w0.rex` alone, 1500 ms wait: `W wait rc: 121` (timeout), status 0. So W's rc 0 above
  is P's post, not a wait that cannot block.
* A name nobody created: `c.rex` prints `C open never-created: 0` and
  `C mutex open never-created: 0`, status 0. So B's and P's nonzero open is the other process's
  object, not an open that always succeeds.

## Why this stops here

The brief: cross-process and not the daemon means a Moritz ruling. A process-local name table
would answer differently from the oracle for every program that shares a name with another
process, or that runs twice (the second run's create returns `''` on the oracle because the first
run never unlinks).

What a real implementation would need, checked against the constraints:

* `rustix` 1.1.4 (already a dependency) has no `sem_open`:
  `grep -rln sem_open ~/.cargo/registry/src/*/rustix-1.1.4/src` prints nothing.
* `libc` 0.2.186 (already a dependency) declares `sem_open` as an `extern` function
  (`src/unix/linux_like/linux_l4re_shared.rs:1873`), so a call needs a new `unsafe` site, which
  global constraints forbid without a ruling.
* Interoperating with oracle processes (the same name seen by both) needs glibc's `sem_open`, or
  a reimplementation of its `/dev/shm/sem.<name>` layout over a shared mapping, which is also
  `unsafe` (mmap) and ties the crate to glibc's private layout.

Options for the ruling, none measured beyond the above:

1. One `unsafe` site wrapping `sem_open`/`sem_close`/`sem_post`/`sem_trywait` through `libc`,
   recorded in `unsafe_sites.rs`. Cross-process and oracle-interoperable.
2. Keep the refusal (`lib.rs:506` `named_semaphore`, `Owner::Phase9`) and move it to a later phase
   with a reason naming the system object.
3. A divergence licence for a process-local name table. The brief's gate rules this out for this
   task; listed only so the ruling can reject it explicitly.

## Files changed

This report only.

## Tests

None run: no code changed. The oracle measurements above are the Step 1 evidence.

## Deferral

Moritz 2026-10-13: named semaphores are deferred and the refusal stays (`Owner::Phase9`). The design
is queued at `.superpowers/sdd/queued/2026-10-13-named-semaphores-design.md`.

Commit `db87a3b53` "Phase 9b Task 8: named semaphores deferred, the refusal names sem_open":

* `rust/crates/rexx-exec/src/lib.rs:507` `Loud::named_semaphore`: the message now reads
  `routine "X" with a named semaphore, a POSIX sem_open semaphore shared between processes, is not
  implemented (Phase 9)`, and the doc points at the queued design.
* `SysOpenEventSem` and `SysOpenMutexSem` were `util(.., Owner::Phase9)` rows, refused with the
  bare `routine "X" is not implemented (Phase 9)`. They are now `util_run` rows over
  `builtin::rexxutil::open_sem`, which takes the same named-semaphore refusal. Before refusing, the
  body answers what the oracle answers without reaching a semaphore:
  * 88.909, 88.901, then 88.922: the argument is checked before the count. The oracle gives
    `SysOpenMutexSem(, 'b')` 88.901 and `SysOpenEventSem(.nil, 'b')` 88.909.
  * `0` for a name `sem_open` rejects.
* `sem_open_rejects`, shared by the create refusal (`refuse_name`) and the open: nothing left once
  leading slashes go, a slash after them, or more than 251 bytes after them. The length bound is
  new. Oracle measurement: a 251-byte name created `/dev/shm/sem.<251 bytes>`, and 252 and longer
  created nothing (`SysCreateEventSem(copies('r', 252)) == ''` is `1`). Before this change the
  crate refused those names instead of answering `''`.
* Witness `rust/corpus/lang/sys_semaphore_names_rejected.rex`, listed in `rust/corpus/phase-9.txt`,
  with its `sourceline_oracle/sys_semaphore_names_rejected.txt` generated by that test's oracle
  driver. Oracle: 3 runs, stdout identical across runs, stderr empty, status 0. Crate
  (`target/release/rexx-run`, rebuilt, `Compiling rexx-exec` seen): stdout `cmp`-identical to the
  oracle's, stderr empty, status 0. Every stdout line was read: each create and open shape prints
  `1` or `0`, and each of the five error traps prints its code. `/dev/shm` is empty before and after.
* `rust/corpus/refusal-sites.tsv` was re-derived with `REXX_REFUSAL_SITES_REFRESH=1`. Only line
  numbers in `lib.rs` moved.
* `src/run/tests/directives.rs`: the refusal expectations for the open and the creates carry the
  new message, with a `SysOpenMutexSem('/sem')` case added.

No two-activity witness was added: it would need a working named semaphore.

### Checks (all foreground, CARGO_INCREMENTAL=0, memcap 8G, -j 4)

* `cargo fmt --all --check`: exit 0.
* `cargo clippy -p rexx-exec --all-targets -- -D warnings`: exit 0.
* `cargo test -p rexx-exec --lib`: 1116 passed, exit 0.
* `cargo test -p rexx-exec --test refusal_sites --test refusal_dispositions --test closed_phases
  --test owners --test phase_ten_reasons --test internal_routines --test builtin_status --test loud`:
  all ok (5, 3, 8, 6, 5, 2, 26, 9 passed), exit 0.
* `cargo test -p rexx-parse --test sourceline_oracle`: 1 passed, exit 0.
* `cargo test -p rexx-exec --test corpus` (not gate mode): 29 passed, 1 ignored, exit 0.

The suite harness was not run. No ooTest test uses `SysOpenEventSem`, `SysOpenMutexSem` or a named
create (the queued design's survey).
