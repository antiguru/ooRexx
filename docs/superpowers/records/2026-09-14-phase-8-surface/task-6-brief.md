### Task 6: The exit and IO-redirector interfaces, as a command handler reaches them

`FUNCTION`'s `TestAddCommandEnvironment` (`testbinaries/orxfunction.cpp:750-768`) registers a
direct and a redirecting command handler through `AddCommandEnvironment`; the handlers receive a
`RexxExitContext` and a `RexxIORedirectorContext`. That is this phase's consumer. Exit
*registration* (`REGISTERED_EXITS`, `DIRECT_EXITS`, `RexxRegisterExitExe`) is embedding or RXAPI and
is not.

**Files:** `rust/crates/rexx-api/src/ffi.rs`, `src/layout.rs`, `rust/crates/rexx-exec/src/`

- [ ] **Step 1:** `RexxInstanceInterface::AddCommandEnvironment` and the call and method contexts'
      forwarders to it; the handler's registration meets Phase 7's `ADDRESS` handler table.
- [ ] **Step 2:** `ExitContextInterface` as a command handler receives it, and
      `IORedirectorInterface`, which meets Phase 7's `ADDRESS ... WITH` work.
- [ ] **Step 3:** Witnesses against the oracle through `orxfunction`'s handlers, including a
      redirecting handler reading and writing each stream. Commit.

---

