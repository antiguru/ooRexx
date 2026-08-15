### Task 10: The 4b corpus and the collector

**Files:**
- Create: `rust/corpus/proc/*.rex`
- Modify: `rust/corpus/phase-4b.txt` -- **Task 1 created it** with `lang/interpret_dynamic.rex` as its only entry, because moving `Interpret` in scope needed a witness and the 4a subset excludes `INTERPRET` by definition. This task grows it and extends its header.
- Modify: `rust/corpus/phase-4a.txt:18`, `rust/corpus/README.md:108-109`
- Modify: `rust/crates/rexx-exec/src/lib.rs` if the collector sweep finds a second under-rooting site

**Interfaces:**
- Consumes: Task 0's subset union.

**The corpus rules for 4b, binding every program in `phase-4b.txt`:**

* **No `DO OVER` on a stem.** The oracle walks a balanced tree and we use a hash map; measured, tails 1, 2, 3, 10, ZZ, B yield `1 B 3 2 ZZ 10`. Such a program could never pass.
* **No builtin calls.** 4b depends on 4c nowhere, and a program written naturally will reach for builtins to make a routine do something observable. `say`, assignment and arithmetic are enough.
* **No `PARSE`.** 4c's.
* **No `::routine` or other directive**, unless `assert_program_has_no_directives` (`tests/coverage.rs:331`) is amended first -- it rejects any corpus program with a directive, and that is a deliberate property of the 4a subset.

**Inherited items this task pays for:**

* **I19.** `EXIT`'s result is under-rooted from the temps-frame pop to `exit_code_for` -- under-rooting, the direction that breaks when a collector lands, and longer than any window the crate documents. Harmless today only because nothing between that pop and `exit_code_for` calls `alloc_with`. The pointer was deliberately placed on `Heap::collect` in `rexx-core` rather than at the leak site, because the person who turns this into a use-after-free is whoever wires a collector in -- **and anyone doing 4b work in `rexx-exec` will never see it.** That instruction also says to sweep `rexx-exec` for the same shape first. **Do the sweep** and report what it found, including "nothing".
* **I20.** The collect-on-every-allocation mode has never seen a call frame. Criterion 4 passed on 29 programs, all 4a-shaped. `collect_stress` must run the **union**, and 4b's programs must include calls, arguments and exposure.
* **I26 and D7's documentation half.** Correct both `List` comments to say Phase 5.

- [ ] **Step 1: Write the 4b corpus programs**

One per construct at minimum, and **at least three that combine two**: a raise inside a routine inside a loop; an exposed stem mutated by a callee; a trap that resumes and then raises again. The combinations are the point. 4a's whole-branch review found two Criticals that survived 824 tests, a 29-of-29 byte-identical corpus, nine per-task reviews, seven gate criteria and a nine-mutation script -- because the coverage criterion enumerates variants and asserts nothing about combinations. `Stem` had a witness. Arithmetic had witnesses. `Stem` as an arithmetic operand had none, and `a. = 5; say a. + 1` aborted the process.

- [ ] **Step 2: Run the corpus in report mode, then strict mode**

Run: `cargo test -p rexx-exec --test corpus`, then `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`. A failure naming `4c` is expected; one naming `4b` is this plan's work.

- [ ] **Step 3: Sweep `rexx-exec` for the I19 under-rooting shape, and report the result**

- [ ] **Step 4: Run `collect_stress` over the union**

- [ ] **Step 5: Correct the two `List` comments, commit**

---

