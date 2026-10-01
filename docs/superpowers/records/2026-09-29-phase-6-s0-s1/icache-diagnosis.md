# sendloop front-end regression (controller, 2026-09-30)

AMD Ryzen AI MAX+ 395 (Zen 5). Four release builds from touched git-archive trees, own target dirs.

L1-icache-load-misses, sendloop (two runs each): base 1754a3b5a 1.0-1.4M; T7 8d26923bb 1.0M;
T8 4b0128186 29-31M; T9 8d65d9406 54-56M. fibfunc: 50M / 65M / 67M / 84M. dispatch: 135M / 114-119M /
162-170M / 200-240M.
Op-cache misses (raw r20000048F), sendloop: base 247M, T7 358M, T8 330M, T9 558M. Cycles: T7 3.19G,
T8 3.38G, T9 4.36G (load 14-25 during the runs).

Misses are spread over every hot line (no single conflicting line), so the per-send hot path no
longer fits the front end. Per send (callgrind, n=20000, calls == n), T9 runs drive (0x5d20 bytes),
begin_invoke (0x5a84), ops_loop_steady (0x33c8), resume_region (0x2e3d, a second expansion of the
region op arms), finish_send (0x9b1), finish_send_op, and pushes a ParkedCall and a CallTail. T7 ran
invoke (0x5942) and run_ops_from only. Hot per-iteration code: 426-451 64-byte lines (~27 KB) in
all three; the difference is which large functions the lines sit in.

Direction for the perf round: the caller's continuation after a send/call resumes in the same op
loop expansion that runs the callee (one expansion of the region op arms, no resume_region copy on
the per-send path), and the begin halves stay small (cold arms out of line). Measure with
L1-icache-load-misses and r20000048F on sendloop, fibfunc, dispatch, plus wall clock, not Ir alone.
