### Task 4: A thread context that lives as long as its interpreter

`Contexts` (`rust/crates/rexx-api/src/ffi.rs`) owns the `RexxThreadContext_` and its table for one
`invoke::method`. The oracle's thread context is the activity's (`ActivityContext`, `interpreter/concurrency/ActivationApiContexts.hpp:64-68`),
valid for the attached thread, and `RexxPackageLoader` is handed one with no call in flight
(`oorexxapi.h:259`). The final review's forged extension, keeping `context->threadContext` across
two calls, aborts rc 134 where the oracle prints `x 42`; AddressSanitizer names
`stack-use-after-return` in `ffi::whole_number_to_object`.

**Files:** `rust/crates/rexx-api/src/ffi.rs`, `src/invoke.rs`, `rust/crates/rexx-exec/src/dispatch/library.rs`

- [ ] **Step 1:** Give the thread context and its table the interpreter's lifetime, and have each
      callback resolve the *current* native frame from the owner, which is what
      `contextToActivation` does for a thread context (`interpreter/concurrency/Activity.hpp:458`). A callback through a thread context with no
      native frame in flight gets the oracle's answer for that, measured.
- [ ] **Step 2:** A unit test that keeps the thread context across two calls, in the crate's own
      tests (not the corpus, which loads only the oracle's extensions). Under the old design it
      must fail; say how, given that the failure is a use-after-return and not an assertion. Run
      it under Miri as well.
- [ ] **Step 3:** The package `loader` hook, run with that thread context after the library's
      routines register (`interpreter/package/LibraryPackage.cpp`, `loadPackage`; print the lines),
      never for a library refused for its version; and the `unloader` at termination for every held
      library, measured on the oracle for order and for what a raise inside one does. Say which
      shipped extensions declare either hook, from their package entries, since a hook no shipped
      extension declares can be witnessed only by a unit test. **A hook runs extension code outside
      `invoke::run`**, which is today the only place a refusing slot's record is read (Task 2's fix
      round 1): a refusal recorded inside a hook must be read and refused loudly there too, not
      cleared unread, and the record cell's doc must say so. Its re-review named this path.
- [ ] **Step 4:** Commit.

---

