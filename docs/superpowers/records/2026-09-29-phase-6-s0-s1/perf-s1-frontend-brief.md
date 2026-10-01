# Perf round: the per-send front-end regression (S1, after Task 9)

Goal: bring sendloop, dispatch, dispatchclass and fibfunc wall clock back inside their bars without
reintroducing Rust recursion on any stackless path, and without losing pinning truthfulness.

Diagnosis (read first): `icache-diagnosis.md` in this directory. Summary: from Task 8 on, a send or
call runs through `drive`, `begin_invoke`, `ops_loop_steady`, `resume_region` (a second expansion of
the region op arms) and `finish_send`; the hot code no longer fits the front end.

Direction (outcome is binding, mechanism is yours):
- One expansion of the region op arms on the per-send and per-call path: the caller's continuation
  resumes in the same op loop that runs the callee.
- Begin halves small, cold arms out of line (`#[cold]`/`#[inline(never)]` helpers).
- alloc +0.10% Ir (7 Ir per `.string~new`, from the shared `begin_init`) is also yours.

Measure (per program: sendloop, dispatch, dispatchclass, fibfunc, fibcall, rexxcps, alloc):
- `perf stat -x, -e L1-icache-load-misses` and `-e r20000048F` (op-cache misses), one event per run.
- Wall clock with `rust/bench-programs/wallclock.sh` only when `/proc/loadavg` 1-min is under 2.
- Callgrind (`rust/bench-programs/callgrind.sh`) against base 1754a3b5a.
- Exploring: target program plus one control, `-r 1`, variants built in parallel target dirs and
  measured in one batch. Full program set only for the round you commit.
Current figures (Task 9 head 8d65d9406): sendloop L1i misses 55M (base ~1M), op-cache misses 558M
(base 247M); wall sendloop +19.7%, dispatch +19.0%, dispatchclass +8.8%, fibfunc +8.2%.

Up to three committed rounds. Record figures and commands under `## S1 front-end round` in
`docs/superpowers/plans/phase-6-perf.md`.
