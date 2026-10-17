### Task 7: Standard stream buffering

**Ruled by Moritz, 2026-10-16** (Ruled 2): the convention of C stdio and Python. Standard
error and a standard output that is a terminal are handed to their descriptor at each line end.
A standard output that is a pipe or a file stays buffered. Every stream is handed over, as now,
before a read, before the thread idles and at exit. Output that a kill finds still buffered on a
redirected standard output is lost, and Deviation 40 licenses it.

**Files:** `rust/crates/rexx-exec/src/input.rs` (`hand_over_output`), `src/clause.rs`
(`idle_until`), `src/bin/rexx.rs` (the sinks and the per-descriptor terminal test), a new test in
`tests/program_end.rs`, `docs/superpowers/plans/phase-4-exclusions.txt` (Deviation 40). Source:
`.superpowers/sdd/queued/2026-10-11-output-lost-on-kill.md`.

- [ ] **Step 1:** Measure the oracle and record it in the records directory. Shapes:
      `.error~say('started X'); do forever; end`, the same with `say`, and
      `do i = 1 to 300; say 'line' i; end; do forever; end`, each with the descriptor on a pipe and
      on a file, under `timeout -s KILL 2`. Also `say 'a'; 'echo b'; say 'c'` and
      `say 'a'; call lineout 'STDERR', 'b'; say 'c'` with stdout and stderr both on one pipe
      (`2>&1 | cat`), which an unbuffered oracle orders as written.
- [ ] **Step 2:** Implement the ruling. The terminal test is per descriptor and true only for a
      terminal (`isatty`), not for a pipe (the existing `with_standard_transient` test counts a FIFO
      as transient, so it is not the test). A redirected standard output is also handed over when
      its buffer passes a fixed bound (64 KiB), so a long-running program's output stays bounded in
      memory and reaches the reader as it runs. Before an `ADDRESS` command starts, both buffers are
      handed over, so a command's own output lands after what the program wrote before it. A
      standard error write does not hand standard output over (Moritz, 2026-10-17: "copy Python's
      behavior"), so `utilities/rexx/rexx_command` `TEST_PASSINGARGS7`, which merges a child
      `rexx`'s OUTPUT and ERROR into one array and asserts program order, stays red under
      Deviation 40: list it in `suite-licences.txt` with that reason. Nothing is added to the `say`
      path for a redirected standard output beyond the bound check.
- [ ] **Step 3:** The test spawns `CARGO_BIN_EXE_rexx` on each Step 1 shape, kills it 2 s after the
      write, and asserts: standard error keeps every line, as on the oracle. A redirected standard
      output keeps what was handed over, and nothing written after it. The two ordering shapes
      match the oracle byte for byte on the shared pipe, except where Deviation 40 licenses a
      difference: the `lineout 'STDERR'` shape interleaves differently on a shared pipe, because
      the program's standard output is buffered and its error is not. Record the shapes that
      differ. Show the standard error case red with the change reverted. Close the queued item in
      the report.
- [ ] **Step 4:** Write Deviation 40 in `phase-4-exclusions.txt` after Deviation 39, in its
      style, from this draft, filling in the Step 1 and Step 3 measurements:

      ```
       40. A REDIRECTED STANDARD OUTPUT IS BUFFERED (Phase 9c). OWNER: none.

           Ruled by Moritz 2026-10-16. The oracle writes each SAY's text and
           its newline with a write(2) of its own, whatever the descriptor.
           This crate hands standard error, and a standard output that is a
           terminal, to the descriptor at each line end. A standard output
           that is a pipe or a file is buffered, and handed over before a
           read, before the thread idles, before an ADDRESS command, when the
           buffer passes 64 KiB, and at exit. That is the convention of C
           stdio and Python 3 (Python 3.14.8 measured: print to a pipe or a
           file is lost under SIGKILL, stderr is kept).
           What a program can see: output that a SIGKILL, a crash or the OOM
           killer finds still buffered is lost, and standard output and
           standard error sharing one pipe or file interleave by hand-over
           rather than by write (ooTest rexx_command TEST_PASSINGARGS7). A
           program that ends normally writes the same bytes to each
           descriptor as the oracle.
           Measured: <Step 1 and Step 3 shapes, oracle then this crate>.
      ```
- [ ] **Step 5:** Measure the cost at the task, against the pin. Build `rexx` at the task's
      commit, copy it to `/home/moritz/dev/repos/ooRexx-p9-pin/rexx-9c-t7-<sha>`, and from the
      repo's `rust/` run

      ```
      bench-programs/callgrind.sh -B 0.5 -p 'sayloop startup' -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9c/t7 \
        pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=<copy>
      ```

      `sayloop`'s output is redirected under callgrind, so it measures the buffered path. An `OVER`
      gets fix rounds. Copy the table into `docs/superpowers/records/2026-10-15-phase-9c/t7/`.
      Per-task check. Commit.

