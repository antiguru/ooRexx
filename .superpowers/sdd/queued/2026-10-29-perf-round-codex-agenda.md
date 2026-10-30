**OPEN, queued 2026-10-29 by the controller from an external (codex) review of 9af69058d; Moritz 2026-10-29 "lgtm" to the placement: perf round after 9d.**

# Perf round agenda from the external review

* Collection sweeps every arena slot up to the historical peak (`rexx-core/src/heap.rs:376`): benchmark a large burst, release, then a small steady workload; occupied-slot indexing or reclaimable segments.
* An arithmetic site gives up small-integer arithmetic after one miss (`ir/drive.rs:865`): measure a retry with backoff against the per-op branch cost already measured (about 0.5% on rexxcps).
* Method sends repeat lookup where routine calls cache resolution (`dispatch.rs:3940`): a guarded send cache.
* Worker pool overflow is unbounded, 512 MiB stack reservation each (`scheduler/pool.rs:149`): admission rules that keep progress.
* Measure against a version-matched -O3 oracle: wall time, instructions, allocation volume, GC pauses, one workload matrix.

## Cheaper state under exclusive ownership (external review by Codex, pasted by Moritz 2026-10-30)

Keep the single-owner interpreter and green threads; construct, copy and inspect less state while ownership is exclusive. Priority as given: a compact ordinary-call representation, then guarded scalar fast paths, then targeted inheritance sharing. Polling frequency left alone.

* Ordinary internal calls: one compact continuation for same-body Rexx calls instead of overlapping CallTail, ParkedCall and ParkedLevel; clause and program information derived from a stable return PC. Preconditions: register and temporary-root floors, activation identity and exact unwind state kept; the general path kept for other entry protocols; the three stacks are not assumed equivalent.
* Per-call ownership and inheritance: stable program and plan metadata shared through lifetime-managed IDs; rarely mutated inherited tables shared, not copied (user-trap maps are copied on call entry where builtin trap tables are already copy-on-write). Preconditions: code pinned for active, parked, replied and captured frames; every mutation detaches, automatic trap-state changes included; numeric settings and traps inherit, variable bindings and exposures share differently. Helps trap-heavy programs, not the empty-call benchmark.
* Tracing and stack inspection: a small execution record, richer metadata rebuilt when observed; mode checks hoisted across regions where the mode cannot change. Preconditions: enough PC and frame information for exact errors and stack inspection; revalidated after guest calls, callbacks, request servicing and resumption.
* Register initialization and GC roots: reserve fewer registers or scan only live ones. Preconditions: liveness covers arbitrary label entry, SIGNAL, nested calls, handlers, INTERPRET and REPLY; every observable root valid and published before allocation, reentry or yielding. Skipping initialization alone is unsafe.
* Scalar operations: short guarded paths for unary arithmetic and common primitives; arithmetic sites specialize again after transient misses (with 2026-10-29-unary-integer-specialization.md and the retry item above). Preconditions: current precision, overflow, representation and operator dispatch checked; LOSTDIGITS, coercion and rounding kept; fall back before anything that could run guest code or allocate.
* Guard and watch lifecycle: stores stop going through watcher machinery after the last watcher goes; possibly the single-activity guard shortcut recovered after concurrency ends. Preconditions: watch counts cover every variable and scope, posted wakeups survive cleanup; guard quiescence also accounts for parked activities, pending starts, native calls, callbacks and reply continuations.

Bookkeeping is delayed only until its first possible observer (GC, a condition handler, stack inspection, native reentry), not until the next yield.

## Instruction-cost inventory (Codex, pasted by Moritz 2026-10-30)

Codex's per-instruction cost experiment, extracted from rexx-instruction-cost-inventory.zip (sha256 b7aa471a96c83498b85e1b2001c7eab19affd293cd16b615ef9c1485a847a651) into docs/superpowers/records/perf-round/2026-10-29-instruction-cost-inventory/. Rust binary from around 9af69058d, revision unverified; oracle a version-matched -O3 LTO build:
* inventory.py: workload generation, paired timings, correctness checks, perf counters.
* inventory-cases.json: workload definitions.
* publish-inventory.py: report generation.
* inventory/README.txt: methodology and limitations.
* inventory/report.html: existing results.

Experiment scripts: interpreter paths, CPU 13 and its sibling CPU 29 are hardcoded, and the runner expects an unpacked rust-rexx (the preserved executable is rust-rexx.gz). Parameterize those and use fresh output directories. Keep the qualifications: queue cases lack equivalent RXAPI integration; REPLY has no valid isolated cost; EXIT measures the whole process. Use inventory/instruction-coverage.json; the root coverage file has stale Label metadata.

First read (controller): 126 comparable kernels, median Rust/oracle 1.53, Rust faster on 19 (integer arithmetic, 0.39 to 0.63). The slowest rows (NUMERIC DIGITS/FUZZ/FORM, OPTIONS, prefix +, ADDRESS VALUE, 3.6x to 6.9x) all land near 65 ns or 24 ns here against 6 to 16 ns there; a shared cost is a hypothesis to test, not a finding.
