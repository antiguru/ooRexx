# Task 11b carry-ins (controller)

* **Probes and sites.** `scout-e1-report.md` items 1 and 6 give the probe text, both engines' output, and the crate sites at `f7170918c`. Find each site by name, because the line numbers have moved since. The full-pool oracle run ends only outside `ulimit -v`. Record the oracle command you use, and keep the crate run under `memcap`.
* **Sim mode as built.** `REXX_SWITCH_MODE=sim:SEED[,policy][,knobs]`:
  * `collect_on_baton` bounds the inline wait under sim (`sim_block_bound`);
  * a new park-for-a-worker path is a scheduling event, so it must appear in the trace and replay must reproduce it;
  * `task-9-report.md` and `task-9-fix1-report.md` give the trace format.

  The seeded gate is `the_seeded_gate` in `tests/concurrency_tests.rs`, and its tables are `sim-gate.tsv` and `sim-exempt.tsv`.
* **Byte accounting (heapshape round).** Live body bytes are a running figure: charged where they are charged, subtracted when freed. The debug assertion in `Heap::collect` checks this. UNINIT resurrection keeps a body live, and R10 counts resurrected bodies in `bytes_due`. If draining UNINITs at activation return frees bodies earlier, the running figure must follow. Add a test that reaches a debug collection after a drain.
* **Perf.** The budget is +0.5% against base61 `/tmp/claude-1000/p61/t1/bin/base/rexx-run` on the eight programs (`pingmsg pingguard pingsem alloc alloc4c heapshape rexxcps emptyloop`), measured with `rust/bench-programs/callgrind.sh`. After 11a, rexxcps is at +0.4394%, which leaves about 0.06% of margin, and alloc is at +0.336%. A check at every activation return hits rexxcps directly. Keep the common path to one predictable test, such as a pending-count or a non-empty bit with the drain out of line and `#[cold]`. If any program goes over +0.5%, stop and message the controller with the cgdiff. Do not optimise elsewhere to make room.
* **Wall clock.** emptyloop and pingguard wall clock differ from base61 at flat Ir (11a report). That is Task 12's investigation, not yours, so report the figures and leave it.
* **Output order changes** caused by the UNINIT drain: name each changed corpus program and witness it against the oracle in 5 runs. GC timing is a licensed divergence (`oorexx-gc-ordering`), but a corpus expectation changes only where the oracle agrees.
* **Mutants run in the tree, then are restored.** Never use `git checkout`, `restore`, `stash` or `reset`. Reverse each mutation by an exact edit, then confirm with `git diff --stat`.
* **Commit per step**, each with its test. The per-task check includes `cargo clippy -p rexx-exec --all-targets --features pinning,sharing -- -D warnings`.
* **Defects found beyond the items:** message the controller (SendMessage to "main") with the evidence before widening scope.
