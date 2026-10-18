### Task 3: The reachable refusals

**Files:** `docs/superpowers/records/2026-10-18-phase-9d/refusals.tsv` (new),
`docs/superpowers/records/2026-10-18-phase-9d/refusals-probes/` (new).

**Produces:** `refusals.tsv`, with these columns:
* `site` (`file:line`);
* `class`, `method`;
* `receiver` (a representation or `any`);
* `reached` (`yes` or `no`);
* `probe`, a file under `refusals-probes/`;
* `oracle`, one line;
* `disposition`: one of `implement`, `deviation:<n>`, `guard`, `phase:<name>`.

- [ ] **Step 1:** Seed the table from the `Loud::native_method` call sites
      (`grep -rn 'Loud::native_method(' rust/crates/rexx-exec/src`):
      * `dispatch/context.rs:1180`: `StackFrame~executable` on a library routine frame, which 9a
        recorded as owed to 9d (`phase-9a-gate.md:287`);
      * `dispatch/object_protocol.rs:487`: `OBJECTNAME=` on the receivers this site refuses. A
        value-based String already has its own constructor under Deviation 36
        (`object_name_of_a_value`, `refusal-dispositions.tsv:51`). `phase-9a-gate.md:292` has the
        history;
      * `dispatch/object_protocol.rs:658`: `COPY`, including the REXX package's copy
        (`phase-9a-gate.md:291`);
      * `dispatch/hash.rs:214`: per receiver representation, covering the condition object and
        `.methods` (`.superpowers/sdd/queued/2026-10-01-condition-object-directory-methods.md`);
      * `dispatch.rs:2500`: the generic fallthrough, for any resolved method that no body map
        holds (`dispatch.rs:2472-2500`).

      Each known receiver gets its own row.
- [ ] **Step 2:** Find the fallthrough's reach by probe. `rust/corpus/method-bodies.txt` already
      records a no-argument send of every instance method and shows no `native_method` hit, so
      this step covers what it does not: the class-side rows of `rust/corpus/docs/class-methods.txt`
      (`awk -F'	' '$3=="class"'`), sent to the class object, and each instance method sent with
      the arguments its documentation requires. A script records which sends end in the
      fallthrough's text. Commit the script under `refusals-probes/`, and add each such method as a
      row.
- [ ] **Step 3:** Probe each row through a plain instance, a subclass instance and an `enhanced`
      instance, on both sides. Record `reached`, the oracle's answer, and a proposed `disposition`.
      Commit, and report to the controller. The controller takes each `deviation` or `phase`
      proposal to Moritz. Each `implement` row becomes part of a Task 7 instance.

