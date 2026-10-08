### Task 4a: One truth judgment

**Survey:** `.superpowers/sdd/2026-10-07-phase-6-1/truthiness-survey.md` (taken at 485319e20; its
rows 1-2 were fixed by Task 3 fix rounds 3-4). Re-derive the site list at dispatch with its two
`git grep` commands and the extra pattern `to_text\(.*\)` near `== b"1"`.

**Rule (R9):** every place that decides whether a value is logically true judges it as the oracle's
`truthValue` does (`ObjectClass.cpp:523-528`): identity with the logical true and false objects first,
else the value's string by `requestString` (a user object is sent `STRING`; an Array joins its items),
then exactly `0` or `1`; anything else raises Error 34 with the caller's sub-number. One function
decides it; the call sites choose only the sub-number.

**Files:** `eval.rs` (`logical_value` `:1436`, prefix `\` `:674`, `&`/`|`/`&&` `:1166-1169`, list
elements `:1356`), `run.rs` (`condition_value`/`condition_holds` `:3636-3725`, `test_case_when`
`:3732`), `run/loops.rs` (`loop_truth`, the DO WITH `AVAILABLE` test), `dispatch/collection.rs:215`
(`same_item`), `dispatch/string.rs:1256`, `security.rs:124`, `ir/drive.rs` (`:1477` IF/WHEN quick
path, `register_holds` `:3911`), `docs/superpowers/plans/phase-4-exclusions.txt` (Deviation 25), a new
datadriven file under `rexx-exec/tests/` or `src/.../testdata/`, corpus programs.
`semaphores.rs:272`'s `unwrap_or(false)` is an absent answer, not a truth judgment; leave it.

**Interfaces:** Produces `Interp::truth(value: ObjRef, raise: fn(&[u8]) -> Raised) -> Result<bool,
Failure>` (name may follow local idiom) as the only path from a value to a `bool`; `logical_value`
becomes private to it. Fast paths stay where they are and are checked against it.

- [ ] **Step 1: Measure.** Oracle and crate, from a fresh empty directory, one table: values `'1'`,
      `1`, `0+1`, `'1'||''`, `left('12',1)`, `1~string`, `.true~copy`, `.array~of(1)`, `'0'`,
      `.array~of(0)`, `'banana'`, `' 1'`, `.array~of(1,2)`, an object whose `STRING` answers `1`, one
      whose `STRING` answers `0`, one with the default `STRING`; each fed through IF, WHEN, WHILE,
      UNTIL, `\`, `&`, a comma-list IF, SELECT CASE (a CASE object whose `==` answers the value),
      DO TO (user `>` answering it), BY (user `<`), and `hasItem`/`index` on Array, List and Table
      (an item whose `==` answers it). Record rc, output and error code per cell in the gate record
      under `## Task 4a`. The oracle's DO TO/BY cells differ by Deviation 25; every other cell is the
      target.
- [ ] **Step 2: Failing tests.** A datadriven table (the crate's `datadriven` dev-dependency) with one
      case per value and context, expected true / false / `34.n`; invariant: every context agrees on
      true/false/error for a value (sub-numbers per keyword). Corpus programs for the cells where the
      oracle agrees, including SELECT CASE on an object (oracle sends `==` to the CASE value,
      `WhenCaseInstruction.cpp:157`, and judges with `Error_Logical_value_when_case`) and `hasItem`
      with a non-logical `==` answer (oracle `isEqual`, `ObjectClass.cpp:180-194`: raises 34.901).
- [ ] **Step 3: Implement.** The one function; route every site above through it; `same_item` raises
      instead of `unwrap_or(false)`; `test_case_when` sends `==` to the CASE value where the oracle
      does (keep the text compare as the fast path for string CASE values only if it is provably the
      same answer); delete `loop_truth`'s own read. Add `debug_assert_eq!` refinement checks at the
      fast paths (`drive.rs:1477`, `register_holds`, `condition_holds(checked)`) against the one
      function. Deviation 25 drops "the crate's WHILE ignores a user STRING".
- [ ] **Step 4:** Witnesses agree; the datadriven invariant passes; a mutation that makes one site
      call `to_text` instead of the function turns the table red (record it in the gate record, then
      revert). Perf against the task's base with Task 2's callgrind command on `rexxcps`, `emptyloop`,
      `decloop` and `dispatch` under `## Task 4a`, budget +0.5%. The per-task check;
      `whole_groups` if a whole-group expectation line changes. Commit.

