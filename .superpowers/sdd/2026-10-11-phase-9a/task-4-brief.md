### Task 4: D3 reasons and the relabels

**Files:** `rexx-exec/src/internal_routines.rs` (`util` `:85`), `dispatch/native.rs` (`deferred`
`:122`), `lib.rs` (`owned_message` `:946`, `Loud::redirection` `:561`, `package_option_write` `:662`,
`named_semaphore` `:511`), `directives.rs:80` (`gap`), `environment.rs:363`, `dispatch/library.rs:838`,
`redirect.rs:426,521`, `rexx-inventory/src/lib.rs` (`EXCLUDED`), `rexx-api/src/layout.rs:378`
(`REFUSING_MEMBERS`), `rexx-api/src/ffi.rs:4187,4229`, `tests/internal_routines.rs:107` (`PHASES`),
`src/dispatch/native/tests.rs:150` (`OPEN`), a new structural test `tests/phase_ten_reasons.rs`,
`docs/superpowers/plans/phase-4-exclusions.txt:5516`.

**Interfaces:** Produces `pub enum Deferral { Rxapi, Extension, Row10NativeApi, Embedding }` in
`rexx-core` (the one crate both `rexx-api` and `rexx-exec` depend on; `rexx-inventory` depends on
none of them, so its `EXCLUDED` column holds the variant's name as a string, which the structural
test checks against the enum), and the owner parameter of each constructor as `Owner::Phase9` or
`Owner::Phase10(Deferral)`.

- [ ] **Step 1:** The structural test first: it scans `rexx-exec/src` and `rexx-api/src` (the crates
      `closed_phases` scans) for `"Phase 10"` literals outside `#[cfg(test)]` modules and `tests.rs`
      files, and fails on any not passed through a constructor carrying a `Deferral`. It fails today.
- [ ] **Step 2:** Classify every site (`grep -rn '"Phase 10"' rust/crates/*/src`, the command and its
      output in the report): the seven macrospace `util` rows and the queue `deferred` rows are
      `Rxapi`; `REGISTERED` (`directives.rs:80`) and `.STDQUE` (`environment.rs:363`, which becomes a
      constructor call) are `Rxapi`; the 30 non-macrospace `util` rows, `package_option_write`,
      `named_semaphore` are Phase 9; `handle_set`, the classic library site and the redirect sites are
      classified with a probe or a reading of the oracle each, the reason in the report. A site that
      fits no reason is Phase 9's. `phase-4-exclusions.txt:5516` is classified the same way.
- [ ] **Step 3:** Thread the owner through each constructor; `EXCLUDED` and `REFUSING_MEMBERS` gain a
      reason column; the two `ffi.rs` texts are built by one constructor taking the reason, found with
      `closed_phases`' `mentions` over `rexx-api`. Relabel R4's strings to Phase 10 `Embedding`.
- [ ] **Step 4:** Add `"Phase 9"` to `tests/internal_routines.rs`'s `PHASES` and
      `src/dispatch/native/tests.rs`'s `OPEN`. Refresh `refusal-sites.tsv`; diff it: only phase names
      change (Review Focus 5). Per-task check with `owners`, `closed_phases`, `refusal_dispositions`,
      `refusal_sites`, `internal_routines`, `phase_ten_reasons`, and `rexx-api`'s tests. Commit.

