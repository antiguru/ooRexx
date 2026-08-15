### Task 8: Write the gate

**Files:**
* Create: `docs/superpowers/plans/phase-4d-gate.md`

- [ ] **Step 1: Write the criteria before reading Task 6's numbers again**

Model the document on `docs/superpowers/plans/phase-4c-gate.md`, which renders **MET** or not per criterion in a table.

**The bar is parity, unamended** -- Global Constraints `:39`, decided 2026-08-08. Not 4a's R2 threshold of 1.5, which `:40` scopes to Phase 1's viability check.

- [ ] **Step 2: Write each bar's derivation, not only its value**

"Axis X's bar is the ratio implied by removing cause C's measured self-time share" is checkable by a later reader. A bare number is not. This is the step that makes the bar a prediction rather than a description.

- [ ] **Step 3: Apply the vacuity test to every criterion**

**Ask of each: what degenerate execution satisfies this, and would deleting its subject leave it green?** This project has shipped criteria satisfied by shrinking their own denominator, and one whose stated falsification procedure was measured and did not falsify.

Two specific to this gate:

* **A criterion that gates on a ratio containing process startup is improvable by cutting startup with nothing landing in the interpreter.** The spec makes the offset visible; visibility is not closure. **Decide here** whether the gated measure excludes the fixed offset, and say which.
* **`startup` is not comparable at 4d and must not be recorded as passing.** This crate has no `CoreClasses.orx` bootstrap, so it starts fast by doing none of the work the oracle does. Record it as not comparable, name D2's absolute target of about 55 ms for 5,203 lines (`plan:157`), and gate nothing on it. D2's decision is already made and is **(a), no saved image** (`plan:151`).

- [ ] **Step 4: Record what cannot be measured, by name**

`dispatch` goes to Phase 5: a dispatch benchmark that avoids message sends is a different benchmark. State it as a D9 dimension this phase does not cover.

**macOS is in the parity gate's text** and is not available in this session. Record Linux parity as met or not, and macOS as outstanding, with the gate explicitly incomplete on that axis rather than silently Linux-only.

- [ ] **Step 5: Record the stopping rule and the amendment rule for 4d-2**

4d-2 ends when every axis meets its bar, or when a task's measured result contradicts the attribution -- whichever comes first. Any later change to a bar carries **both wordings and the reason**, the way `phase-4c-gate.md` recorded its criterion 4 amendment, which is the only reason that amendment survived review.

- [ ] **Step 6: Commit, and state that 4d-2 is not planned here**

---

## Explicitly not in scope

* **Any optimisation.** Prototypes are published and reverted.
* **`dispatch`**, which needs Phase 5.
* **Planning 4d-2**, which is planned from the attribution once this unit closes.
* **Adopting an allocator**, which is 4d-2's decision against the bar.
