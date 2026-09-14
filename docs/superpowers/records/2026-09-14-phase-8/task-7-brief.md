### Task 7: The seven context functions, `.Pointer`, and `CSELF`

**Files:**
- Modify: `rust/crates/rexx-api/src/ffi.rs`, `rust/crates/rexx-api/src/values.rs`
- Modify: `rust/crates/rexx-core/src/body.rs` (if `.Pointer` needs a body)
- Modify: `rust/crates/rexx-exec/src/dispatch.rs` (`.Pointer`'s rows)

**Interfaces:**
- Produces: `SetObjectVariable`, `DropObjectVariable`, `WholeNumber`, `StringData`,
  `StringLength`, `NewPointer` and `RaiseException` behind the method-context table.
- Consumes: Tasks 3, 4, 5, 6.

These seven are what `extensions/rxregexp/rxregexp.cpp` calls, counted in the scoping survey's
section 2.

- [ ] **Step 1: Settle `.Pointer` first**, because the spec records it as unmeasured. The class is
      in `native_classes.rs` with `NEW` unsupported to match the oracle's 93.967. Measure what an
      instance is on the oracle -- `~string`, `~==`, what a `CSELF` pointer prints as -- and make
      the internally minted instance answer the same.
- [ ] **Step 2: `NewPointer` and `SetObjectVariable`** together are how `CSELF` gets stored, and
      the object-variable pool is `Body::Instance`'s `pools` (D40, one pool per scope). The scope a
      native method writes into is the one that matters and is easy to get wrong: measure it.
- [ ] **Step 3: The `CSELF` conversion row** from Task 5 reads that variable back
      (`NativeActivation.cpp:294`), and a missing one is not a crash.
- [ ] **Step 4: `StringData` and `StringLength`** hand out a pointer into a Rexx string. Its
      lifetime is the call, and the object must be rooted for that long -- the same constraint as
      Task 5 step 4, and the same test shape.
- [ ] **Step 5: `RaiseException`, `RaiseException0`, `RaiseException1`** raise the numbered
      condition in the caller's context. `rxregexp` uses `Rexx_Error_Incorrect_method`. Number,
      sub-number, message text and descriptor all as the oracle produces them.
- [ ] **Step 6: `WholeNumber` and `DropObjectVariable`**, which are the small two.
- [ ] **Step 7: Tests** per function, plus one that survives a collection between two calls on the
      same object -- that is the `CSELF` survival criterion from the spec's gate, and its negative
      control is running it with collection forced off.
- [ ] **Step 8: Commit.**

---
## Global Constraints

* **`unsafe` lives in exactly two files**, `rust/crates/rexx-api/src/ffi.rs` and
  `rust/crates/rexx-api/src/load.rs`, each block carrying a `SAFETY:` note that names the invariant
  it relies on and who establishes it (D-U1, Moritz 2026-09-14). Everything past either boundary is
  safe Rust on `ObjRef`. `crates/rexx-core/tests/unsafe_sites.rs` is the record and must be updated
  in the same commit as the first grant, never after.
* **One new dependency, `libloading` 0.8.9**, and no other. Phase 7's "nothing beyond `rustix`" was
  that phase's own constraint and does not bind here; this one replaces it.
* **The headers are frozen.** No edit to anything under `api/`. A change that seems to need one
  reopens D5 as a Section 1 decision and stops the task.
* **The C++ tree is read-only**, and so are `samples/`, `build/`, `ootest/` and `oodocs/`. Phase 8
  reads `build/lib/librxregexp.so` and must not rebuild it -- loading the oracle's own compiled
  extension is the instrument (D5 amendment, 2026-09-14), and a rebuild throws it away.
* **No process-global state is mutated**: no `std::env::set_var` / `remove_var`, no
  `set_current_dir`, no write to file descriptor 0, 1 or 2 from interpreter code. A test that needs
  a loader search path passes an absolute path or sets the variable on a child it spawns.
* **Correctness is byte-identical stdout, stderr and exit status against the C++ oracle**, read on
  **three separate descriptors, never `2>&1`**. Oracle runs use the standard wrapper
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
  from a fresh empty directory.
* **Probe hazards.** Read `rust/corpus/oracle-crashes.txt` before an unusual probe and never run its
  entries. Never `select; when 1 = 0 then; when 2 = 2 then nop; end`. Never `NUMERIC DIGITS` above
  1000. Never `.Package~new` on a file inside the repository. Never `::OPTIONS TRACE ?<letter>`,
  which blocks indefinitely. A symbol `x` or `b` directly followed by a quote is a hex or binary
  literal. Put `timeout` on both sides of any probe that loops or reads input.
* **A loaded library is not unloaded while anything it produced is reachable.** An extension's
  `UNINIT`, its `CSELF` block and any function pointer we hold all die with the library, so unload
  ordering is a correctness constraint and not a tidiness one.
* **Four gates, all green at the closing commit:** `cargo fmt --all --check`;
  `cargo clippy -j 4 --workspace --all-targets -- -D warnings`;
  `cargo test -j 4 --release --workspace --no-fail-fast -- --test-threads=8`;
  `REXX_CORPUS_GATE=1 cargo test -j 4 --workspace --no-fail-fast -- --test-threads=8`. Record each
  exit status unpiped and the count of `ok` result lines. **Commit before a long gate run and leave
  the tree frozen until it finishes.** Wait on a running gate in ~1-hour stretches, not 10-minute
  polls.
* **Comments are minimal** (`rust/CLAUDE.md`, Moritz 2026-08-27): a one-sentence overview,
  parameters/returns/panics, and properties the implementation does not show. No narrative, no
  history, no counts of in-repo sets.
* **Every extent claim is derived and its command committed.** No "all", "none", "every" or a count
  in prose without the enumeration that produced it.
* **Write each negative control's prediction before running it**, and mark each part confirmed,
  falsified, or unobservable.
* **A method that exists and does nothing is not implemented.** Build the body or leave the name
  refusing loudly.
* **A refusal names the phase that owes the work.** `Phase 10` for RXAPI, the external queues and
  the RexxUtil remainder; `Phase 9` for the embedding API. After the surface half closes, no
  refusal names `Phase 8`; during the L2 slice, the ones this plan does not reach still may.
* Never `git add -A`; stage explicit paths. Never amend; never force-push; never
  `git reset --hard`; never `git checkout -- <path>` on an edited file; never bare `git stash` /
  `git stash pop`. Commit messages are written to a file and passed with `-F`.
* Scratch files live in the session scratchpad, never in the repository.

---
