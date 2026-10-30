Rexx instruction cost inventory

The catalog maps all 40 parser instruction families to representative workloads.
It contains 132 positive repeated-kernel measurements, an unresolved asynchronous REPLY row, and one end-to-end EXIT row.
Context requirements are explicit: calls, returns, guards, forwards, loops and exception delivery are composite operations.
Marker instructions are covered through enclosing control flow, and no isolated marker cost is inferred.

The same frozen Rust executable and version-matched optimized C++ oracle were used throughout.
Rust source revision is unverified because the existing release executable was copied while another agent owned the working tree.
The oracle flags contain -O3 and LTO, and environment.json records executable/library/image hashes.
The first CPU-2 sweep was superseded after sibling interference and process-startup cancellation were detected, and its raw evidence remains in the parent directory.

Each normal kernel has fixed operand values and a fixed outer iteration count shared by both interpreters.
The body is unrolled at 1 and 16 sites, and marginal cost is (high - low) / (COUNT * 15 * units_per_body).
Three alternating-order timing pairs follow calibration and a warmup, with elapsed time read inside Rexx using TIME(R)/TIME(E).
Instructions and cycles are separate, nonmultiplexed whole-process user-mode counter pairs, so their high-minus-low differences can retain small translation and initialization residuals.

Measurements use CPU 13 on an AMD RYZEN AI MAX+ 395, with a 2 GiB cgroup memory cap around the sequential harness.
Sibling CPU 29 utilization is recorded per case and values above 10 percent are flagged.
The noisy flag means the observed sample range exceeds 15 percent of its median, and sample ranges are not confidence intervals.
These are context-specific costs on a shared machine, not portable constants or universal opcode prices.

Queue roundtrips are excluded from like-for-like interpretation because Rust uses an in-process queue and lacks RXAPI integration.
SAY uses an eight-byte line redirected to /dev/null and includes each runtime's buffering policy, while command rows include a shell process.
REPLY's process-wall differences were inconsistent, so its public cost and ratio are blank and its raw sender-loop/counter evidence is retained.
EXIT measures complete program startup and termination, so it does not establish the cost of the EXIT instruction alone.

The inventory covers instruction families and selected subforms rather than the complete language/library surface.
It does not enumerate every builtin, every method, all operand sizes, message setters, USE aliases/defaults/LOCAL, ADDRESS redirection, or contended GUARD.
Numeric cases record initialized operand spelling, precision, and separately computed integer operands where relevant.
The CSV and interactive report expose fixtures, units, sample ranges, quality flags and counter qualifiers so those boundaries remain visible.

Files
* costs.csv: complete published cost table, with invalid costs withheld.
* catalog.json: full annotated catalog.
* instruction-coverage.csv and .json: all 40 instruction-family mappings.
* environment.json: provenance, formulas and measurement limitations.
* report.html: standalone sortable and filterable table.
* comparison.svg: selected fixed-kernel time ratios.
* <case>/runs.json: raw timings, correctness status and counter records.
* <case>/width-1.rex and width-16.rex: exact measured programs.
* results.json: raw sweep output, including the failed label probe and invalid REPLY subtraction.
