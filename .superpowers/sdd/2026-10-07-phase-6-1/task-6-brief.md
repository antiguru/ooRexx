### Task 6: Interactive debug: pause placement and `.DebugInput`

**Files:** `ir/drive.rs` (hot exit `:1905`, `RegionEnd::Flowed` `:1930`, recompile on trace change
`:1899-1901`), `run/call.rs` (`finish_call`, the pause after a CALL's return), `run/interpret.rs`
(`debug_pause_after_clause` `:202`, its read `:230`, the INTERPRET pause), `input.rs` (`linein_line`
`:310-325` as the model), a pause table beside the IR instruction kinds, corpus programs with
`.stdin` files.

**Interfaces:** `fn pauses_after(kind) -> bool` from the oracle's per-instruction list (scout B
section 2: pauses after Address, Assignment, Call, Drop, Expose, Forward, If, Leave, Message, Nop,
Numeric, Options, Parse, Procedure, Queue, Reply, Say, Trace in debug, Use, UseLocal, WhenCase; not after Then, Else, End, Exit, Return, Signal, Raise, Guard, Interpret, Select, Otherwise
or the DO family). Consumes Task 1's `DEBUG_PAUSE` bit.

- [ ] **Step 1:** Witnesses with `.stdin` files: `pa`, `pakinds`, `di`, `eof`, `dbgcall` (scout B
      appendix). They differ as recorded.
- [ ] **Step 2:** The debug read sends `LINEIN` to `.local~DEBUGINPUT` (`local_route(b"DEBUGINPUT")`,
      under `pinned!`; `.nil` or no answer is the empty line); a condition the read raises (NOTREADY)
      propagates into the activation's traps through `debug_pause_after_clause`'s `Result`.
- [ ] **Step 3:** Pause placement per `pauses_after`, at both region exits, after a CALL's return in
      the caller, and before an INTERPRET fragment's first clause; `=` re-executes from either exit.
      First design: no branch on the Flowed arm in the default mode (lower the flowed ops differently
      when the chunk is compiled under `?`, using the existing recompile on trace change). If that
      fails, add the branch and measure it (R7).
- [ ] **Step 4:** Witnesses agree; the per-task check; perf on `rexxcps`, `emptyloop` under
      `## Task 6`, the branch's cost stated apart if R7 applied. Commit.

