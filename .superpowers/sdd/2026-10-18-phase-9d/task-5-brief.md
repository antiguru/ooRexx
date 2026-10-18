### Task 5: Deviations 42 and 43 name themselves

**Files:**
* `rust/crates/rexx-exec/src/lib.rs`: `named_semaphore` at `:540`, owner `:548`, and `fork_child`
  at `:557`, owner `:565`.
* `rust/crates/rexx-exec/tests/refusal_dispositions.rs:783-793`: `the_scanner_reads_each_owner_form`
  pins `named_semaphore` as Phase 9's. Move that assertion to a constructor that still names a
  phase.
* The texts pinned at `rust/crates/rexx-exec/src/run/tests/directives.rs:321-355` and elsewhere. Find them
  with `grep -rn 'named semaphore\|forked child' rust/crates`.
* `rust/corpus/refusal-dispositions.tsv` and `rust/corpus/refusal-sites.tsv`.

- [ ] **Step 1:** Make each refusal's message name its Deviation (`Deviation 42`, `Deviation 43`)
      where it named Phase 9, and build it with no owner, as the other Deviation refusals do. Find
      that shape with `grep -rn 'Deviation' rust/crates/rexx-exec/src/lib.rs`.
- [ ] **Step 2:** Add the `refusal-dispositions.tsv` rows for both constructors, citing the
      Deviations. Re-derive `refusal-sites.tsv`, and update the pinned texts and the scanner test.
      Commit.

