### Task 5: The `native_method` family, mechanical batch

**Files:** `dispatch/context.rs` (`StackFrame`, `:34`), the `RexxInfo` rows (`dispatch.rs:2409`'s
fall-through), `dispatch/object_protocol.rs` (`copy` `:621`, `OBJECTNAME=` `:466`), `dispatch/array.rs:700`
(`MAKEARRAY` of an `~UNKNOWN` argument list), `dispatch/class_protocol.rs:774` (a metaclass with its own
`NEW`), corpus programs in `phase-9.txt`.

**Interfaces:** None new.

- [ ] **Step 1:** Corpus programs from scout D section 1a's probes, each run against the oracle first
      (stdout read): `RexxInfo~executable` and `~libraryPath` (the oracle prints its own paths; the
      witness prints a property that holds on both sides, such as `~executable~right(5)` equal to
      `/rexx` for the crate's own binary name, decided after reading both outputs, and the gate record
      says what is compared); `StackFrame~executable` at depth 0 and 1; `Object~copy` on a Method,
      Routine, Package, `.context` (93.970) and `.rexxinfo` (rc 163); `'abc'~objectName = 'x'` and the
      SmallInt, Object, Array, VariableReference and Stem receivers; `.environment~unknown('ARRAY','y')`;
      the metaclass `NEW` probe (`id k`).
- [ ] **Step 2:** Implement each as the oracle answers; delete each refusal and its pins. A receiver
      with no variable pool for `OBJECTNAME=` gets the oracle's answer through a side table keyed by
      `ObjRef` if the class has no pool; if that is not mechanical, report it (Global Constraints,
      mechanical batches).
- [ ] **Step 3:** The two unreached sites, `SUPPLIER` in `~defineMethods` (`class_protocol.rs:577`) and
      `Message~notify` (`object_protocol.rs:1220`): one probe per shape tried, run on both sides; reached
      means implemented, unreached means a refusal constructor with no phase and a GUARD row in
      `refusal-dispositions.tsv` naming the probes. Refresh tables. Per-task check, with the
      `Message`, `Object`, `StackFrame`, `RexxContext`, `samples/samples` and
      `doc/rexxref/chapter5/Section1` groups through the harness. Commit.

