# Task 8 review (9b, named semaphores deferred)

Verdicts: open_sem body PASS with one finding; 251-byte bound on the shared check FAIL on one path (NUL in a create name).

1. Valid names refuse loudly on open and create (abc, //abc, 251 bytes, /+251 bytes, ".", ".." all hit the named-semaphore refusal). Open is NUL-truncated like the oracle's CSTRING.
2. Error order matches SysOpenEventSem/SysOpenMutexSem (88.901 / 88.909 argument check before the 88.922 count); witness output cmp-identical to oracle.
3. 251 is right: oracle created a 251-byte name (checked here), none at 252. Leading-slash rules match the oracle on the witness shapes. No unnamed or valid-name path changed.
4. Witness run on both sides (memcap 4G, fresh dir): stdout identical, stderr empty, rc 0; /dev/shm empty afterwards.
5. Mutation not run (no-edit rule); by inspection, dropping or moving the bound or the arity check changes witness lines (252/300-byte create/open, e4/e5).

Finding (MEDIUM, silent wrong answer): refuse_name (rexxutil.rs:323) tests the full text, not the NUL-truncated C string. SysCreateEventSem('t8nul'||'0'x||'/b') : oracle creates sem.t8nul and answers a handle (prints 0 for =='' ); crate answers '' (prints 1). Same for NUL + 300 bytes. Open is fine. Fix: pass c_string(text) to sem_open_rejects in refuse_name; add the two shapes to the witness (oracle creates /dev/shm files, so the witness must not create; use only shapes where truncation yields a reject, e.g. '/'||NUL||'abc' -> oracle name "/" rejected, crate currently refuses loudly).
Pre-existing for the slash check; the new bound widens it.
