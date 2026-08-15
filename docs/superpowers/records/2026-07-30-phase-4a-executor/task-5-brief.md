### Task 5: Stems and compound variables

**Spec:** D15a in full.

**Files:**
- Create: `rust/crates/rexx-exec/src/stem.rs`
- Test: `rust/crates/rexx-exec/tests/stem.rs`

**Interfaces:**
- Produces: `tail_key(&mut self, &Plan, frame, SymbolId) -> Vec<u8>` resolving a compound's tail pieces; `stem_get`, `stem_set`, `stem_drop_tail`, `stem_assign`, `stem_drop`.

**Why:** four measured behaviours that the obvious model gets wrong, and the derived-name rule that every uninitialised read depends on.

- [ ] **Step 1: Write the failing tests from the six transcripts**

Each of these is a measured oracle transcript and the test asserts the same bytes:

```rust
// u. = 'd' ; u.1 = 'one' ; drop u.1  ->  u.1 is U.1, u.2 is d
// a. = 1 ; b. = a. ; a.1 = 2         ->  b.1 is 2      (one shared object)
// r. = 'rd' ; u = r. ; drop r.       ->  u is rd       (drop rebinds)
// s. = 'def' ; t = s. ; s. = 'other' ->  t is def      (assign rebinds)
// say q.                             ->  Q.            (name, with the period)
// i = 'abc' ; v.i = 'val'            ->  v.ABC is V.ABC (keys are verbatim)
// i = 1 ; j = 2 ; a.i.j = 'deep'     ->  a.1.2 is deep  (pieces joined by '.')
```

- [ ] **Step 2: Run them to watch them fail**

- [ ] **Step 3: Implement**

A dropped tail is `Some(key) -> None`, a **tombstone** that does not take the default; an absent key does. `stem_assign` and `stem_drop` **replace the Stem object** and rebind the variable, leaving the old object for anything that aliased it; a tail assignment mutates in place. Tail keys are the resolved piece values verbatim and case-sensitively, joined with `.` for a multi-level tail.

- [ ] **Step 4: Verify against the oracle** — same rule as Task 4, all seven transcripts re-run and pasted into the report.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/stem.rs rust/crates/rexx-exec/tests/stem.rs
git commit -m "Stems: tombstones, aliasing, and a name the object carries itself"
```

---

