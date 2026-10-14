# Phase 9b1 Task 1 review

Diff `3e73e84b7..5292aad56`, read from `review-3e73e84b7..5292aad56.diff`. Reviewer did not edit or
commit. Every crate run used `target/release/rexx-run` after a
`CARGO_INCREMENTAL=0 timeout 300 memcap 8G cargo build -j 4 --release --bin rexx-run` that printed
`Compiling rexx-exec` (the binary was 6 minutes older than the commit, so it was rebuilt).

### Spec Compliance

- ✅ Spec compliant for Steps 1 to 5: Deviations 4, 31, 36 and 37, the reworded refusal, the witness,
  the gate test and the re-derived TSVs are all present and match the brief.
- ⚠️ The plan does not give the `ALLOWED` rows tagged "Task 8" an owner. Plan Task 8
  (`docs/superpowers/plans/2026-10-14-phase-9b1.md:277-292`) is a side-map table and fix. It does not
  remove the freed-handle guard ("a value whose object is no longer live") or the body-shape guards
  ("a stack frame / an object this crate did not build"). Task 9 (`:305`) requires `ALLOWED` to hold
  only the Deviation 32 and 36 sites. The implementer flagged this in a comment
  (`representation_refusals.rs:74-75`). The controller must either give these rows to a task or
  change the Task 9 criterion.

### Strengths

- The gate is a real scanner, not a grep. It reads string literals, skips comments, handles char
  literals, raw strings and `\` continuations, and is pinned by its own test
  (`representation_refusals.rs:290-305`). The stale-row direction (`:264-280`) makes each later task's
  "shrink ALLOWED" step enforced instead of advisory.
- The witness is well built. Each `show` line covers 14 paths, the "two" lines are 0, which proves
  the comparison is live, and every hash comparison uses `==`, which avoids the DIGITS 9 trap.
- Deviation 37 was measured again rather than copied from the earlier review, and the
  untrapped-crash variant was correctly identified as `oracle-crashes.txt` entry 37 and dropped.

### Checks run

- **Witness**, both sides, each from a fresh empty dir: the oracle under the standard wrapper, the
  crate under `timeout 20 memcap 2G`. Oracle rc 0, crate rc 0, both stderr 0 bytes, stdout `cmp`
  identical. The 17 stdout lines are exactly the ones the report quotes.
- **Deviation 37 probe** `stem_sort_emptied.rex`, 3 runs a side: oracle `S.0 [S.1] [S.2]` rc 0 every
  run, crate `S.0 [b] [c]` rc 0 every run. With the STRING element first (a sed copy in the
  scratchpad), both print `syntax 40.924` with rc 0. The C++ citations `StemClass.cpp:1659-1680` and
  `:1730-1736` read as Deviation 37 describes.
- **Deviation 4's merge sentence**, crate: `'1000'` vs `999+1` answers 1, `'12345678'` vs
  `12345677+1` answers 1, and `'abcdefgh'` vs `'abcdefg'||'h'` answers 0. The sentence is accurate.
- **Gate standalone build**: `representation_refusals.rs` compiled with `rustc --edition 2024 --test`
  and `CARGO_MANIFEST_DIR` pointed at a `git archive 5292aad56` copy of `crates/*/src` in the
  scratchpad. HEAD copy: 3 passed. Mutants, each on its own copy:
  - m1: `object_name_of_a_value` reverted to `owned_message("OBJECTNAME= on a string or a number", None)`.
    FAILED, stale row `rexx-exec/src/lib.rs:OBJECTNAME= on a String is refused because a String is
    value-based`.
  - m2: appended `"a routine whose body this crate did not record"` to `environment.rs`. **Passed,
    so the mutant escapes the gate.**
  - m3: appended `"an identity this crate does not keep"` to `rexx-core/src/lib.rs`. FAILED, the
    literal is listed.
- **Dispatch unit test** at HEAD: `cargo test -p rexx-exec --lib a_conversion_this_phase_does_not_model`
  passed (1 test). It is an exact equality on stderr against the new text, so the old message fails
  it. I did not run that mutant because the brief forbids editing the tree.
- **Whether the ALLOWED rows match the grep**: the brief's grep, with comment lines removed, gives
  exactly the files and literals of the 15 `ALLOWED` rows. No literal is lost to scanner desync, and
  no `br"` literal exists under `crates/*/src`.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

1. **The phrase set misses refusals that already exist and name the representation**
   (`representation_refusals.rs:31-36`). A broader grep
   (`grep -rnE --include=*.rs '"[^"]*this crate[^"]*"' crates/*/src`, test paths excluded) finds four
   refusal literals that the gate cannot see:
   - `dispatch/context.rs:513` `"a program this crate did not load"` reads `interp.programs.get(program.0)`.
     That is the stale-ProgramId case Task 6 creates (Review Focus 4), so the report's "no row owned
     by Task 6" is true only because of the wording.
   - `dispatch/class_protocol.rs:995` `"defineClassMethod with a method object whose body this crate
     did not record"`. It belongs to the Method-map group, Task 7.
   - `environment/identities.rs:1006` `"a .ROUTINES entry that is not a routine this crate built"`.
     It is the receiver_class / Deviation 28 crash guard, phrased as representation.
   - `lib.rs:689` `"a directory whose entries this crate does not fill"` (`unreadable_collection`).
     Scout-leaks judged it Phase 10's STDQUE gap, not a leak.

   Mutant m2 confirms that a new refusal in this shape passes the gate. When Task 9 says "ALLOWED
   holds only Deviations 32 and 36", that would not mean "no representation refusal remains".

   Fix: add `this crate did not`, `this crate does not` and `this crate built` to `PHRASES`. All the
   `rexx-bench` "this crate" literals are report labels and none uses these forms. Then tag the four
   rows (Task 6, Task 7, Deviation 28, and Phase 10 or a rewording). Adding Deviation 28 and Phase 10
   to `OWNERS` is a Task 9 criterion change, so it is the controller's call.
2. **Task 8 owner rows have no task that removes them** (see ⚠️ above). This is plan-level, not
   implementer error. The implementer tagged them honestly and commented on them. Leaving them
   unaddressed means Task 9's `ALLOWED` criterion cannot be met as written.

#### Minor (Nice to Have)

1. **"subclass two" and "enhanced two" (`value_based_identity.rex:22`, `:32`) compare two
   temporaries whose lifetimes need not overlap.** A spec that gives "full identity" does not promise
   different hashes for objects that are never live at the same time. Today it holds on both sides.
   The crate's hash includes the slot generation (`object_protocol.rs:167`, `handle.rs:110-113`), and
   20000 oracle iterations never matched. To pin only the specified shape, hold both objects in
   variables first: `a = .mystr~new('abc'); b = .mystr~new('abc')`.
2. **Deviation 36 "STILL SPECIFIED" item 2** ("a WeakReference to a value never clears while the
   value is reachable") is false of the crate today (L1, Task 2). The text is a decision, and
   "The first and third are witnessed" is accurate. Nothing in the record says the second is not
   yet met, so add "L1, Task 2" next to it.
3. **Deviation 4 lost its provenance.** The rewrite removed the ootest checkout revision (r13169 /
   r13178) behind "measured at the checked-out suite", and entry 59's −6.448% / −6.154% figures. The
   26-of-42 count now has no revision attached.
4. **`ALLOWED` keys by file and literal** (report concern 3). A new site with an already-allowed
   text in an already-allowed file is invisible until that row is deleted. This is acceptable while
   every row is scheduled for deletion.
5. **The witness does not depend on the change.** The subclass `objectName=` path and identity
   preservation both worked at the base. This is correct for a pin of a specified guarantee, so the
   witness was not mutated. The dispatch test and the gate are the change's failing tests.

### Brief questions

**Q1 (the dropped "is not implemented" suffix):** no gate loses coverage.
- `is_loud` (`gate_tables/mod.rs:140-146`) keys on rc 120 and the `rexx-exec: ` prefix.
- The suite harness keys on the whole `refusal: rexx-exec: …` line.
- `refused_construct` and `owner_from_stderr` feed only report text: the `[construct]` annotation,
  "what the loud rows are waiting on", the corpus mismatch reason, and the `method_bodies` evidence
  column, which falls back to `-`.
- No probe reaches this refusal. No corpus program, gate-table probe or `method-bodies.txt` row does:
  row 152 is `objectName=` on an `.Object` instance. The one ooTest send
  (`MethodArgs.testGroup:148`) is on `.Object~new`.
- `chunk_refused` (`lib.rs:824`) is already a suffix-less LIMIT.

Recommendation: keep the new wording. "is not implemented" would be false for a licensed rule, and
the spec asks the message to name the rule.

**Q2 (is `representation_refusals.rs` a real gate?):**
- Yes for its own phrases, and both directions go red (m1, m3).
- It covers every `crates/*/src` of the 11 crates. It excludes `build.rs`, `benches/` and
  `examples/`, and none of these holds a match.
- It can be escaped by wording: there are four existing escapes (Important 1).
- The owner tags are correct against scout-leaks for Task 4 (plan Task 4 Step 4 names that row) and
  for Task 7 (the guards on `executable_sources`). The Task 8 tags have no removing task (Important 2).
- "No Task 2, 3, 5 or 6 row" holds for the four phrases. L1-L3 are wrong answers, L10 is a Raised
  88.914, L11 is a missing frame, and L6 is `native_method` "COPY". But `context.rs:513` is a Task 6
  site that the wording hides.

**Q3 (Deviations 4, 36 and 37 against the spec):**
- They are accurate against spec section 1 and the probes.
- Deviation 36 explicitly keeps "never clears while reachable" (L1) and subclass/enhanced full
  identity (L2) specified.
- Deviation 4 and Deviation 36 are scoped to `.String` with no subclass or enhancement, so L2 is
  not licensed. L3, L10 and L11 are not touched. Deviation 37 is confined to SysStemSort write-back
  after mid-sort mutation.
- Nothing licenses a listed defect.

**Q4 (the witness):**
- It covers every specified guarantee in criterion 3:
  - reads, alias, `arg(1)` and `use arg`;
  - internal, `::ROUTINE` and method returns;
  - stem, Array, Directory, IdentityTable (index and item) and Set;
  - subclass and enhanced `objectName=` through a routine and an Array, plus distinctness.
- It avoids comparing separately produced equal values, apart from Minor 1's lifetime caveat.
- Its output is identical on both sides (see Checks run).

**Q5 (do the tests fail without the change?):**
- The gate: yes (m1, m3).
- The dispatch test: yes by construction, an exact string equality against the new text. Not mutated.
- The witness: no, and it does not need to (Minor 5).

### Assessment

**Task quality:** Needs fixes

**Reasoning:** The rule, the Deviations, the witness and the rewording are correct and verified on
both interpreters. The new gate is sound for its four phrases, but four existing representation
refusals escape it, Task 6's stale-program guard among them. Also no task removes the rows tagged
Task 8, so as written the gate cannot support the Task 9 criterion it exists for.
