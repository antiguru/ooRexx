**CLOSED 2026-10-30 by Phase 9d Task 8.8e1 (probe agrees).**

# Named RexxUtil semaphores: POSIX sem_open

Moritz 2026-10-13: deferred, the refusal stays. Found at 9b Task 8's Step 1 gate
(.superpowers/sdd/2026-10-13-phase-9b/task-8-report.md, commit 50147a7be).
Oracle (interpreter/platform/unix/SysRexxUtil.cpp): create calls sem_open(name, O_CREAT|O_EXCL, ...)
(:668 event, :893 mutex); open calls sem_open(name, 0) (:698, :925); close calls only sem_close, and
there is no sem_unlink anywhere, so names persist in /dev/shm/sem.<name> after exit. These are
cross-process; measured with a second oracle process that opens the name and sees the first one's post (5/5).
Design when built: one module of libc sem_* calls (sem_open, sem_post, sem_timedwait, sem_close),
each unsafe site with a SAFETY note and listed in unsafe_sites. It interoperates with oracle processes.
Witnesses are cross-process (two rexx-run processes) and clean up /dev/shm. rustix 1.1.4 has no
sem_open. No ooTest test uses SysOpenEventSem/SysOpenMutexSem or a named create.


## Task 8.8e1 evidence

Named event and mutex create/open all refuse at status 120 with Deviation 43. The unsupported ruling is unchanged. See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8e1-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/records/e1/`.
