**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** a command whose background child keeps stdout open: crate waits to timeout rc 124, oracle returns at once.

# A command whose background child keeps stdout open makes the crate wait

Found 2026-10-18 by Phase 9d Task 2. The probe is
`docs/superpowers/records/2026-10-18-phase-9d/queued-probes/bgcmd/bgcmd.rex`:

    address system 'mkfifo ff1'
    address system '(sleep 0.3; printf abc > ff1) &'
    v = charin('ff1',,3)
    say 'v=['v']' stream('ff1','S')

The oracle returns from the second command at once and prints `v=[abc] READY`, rc 0. The crate
waits until the timeout (rc 124, the 4.1 halt). The crate seems to wait for EOF on the command's
stdout, which the background child keeps open. The Task 2 review measured that this is not the
`redirected-command-holds-baton` cluster's mechanism. Probe with a short timeout: the background
child outlives its memcap group, so kill any leftover by PID.
