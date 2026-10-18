### Task 2: The queued inventory

**Files:** `docs/superpowers/records/2026-10-18-phase-9d/queued-inventory.tsv` (new), then each
`.superpowers/sdd/queued/*.md` after the ruling.

**Produces:** `queued-inventory.tsv`, with these columns:
* `item`: the file name, or `gate:<record>:<line>` for a carried item;
* `kind`: one of divergence, oracle bug, process, perf, design;
* `probe`: `file:line` of a recorded probe, or `-`;
* `proposed`: one of `9d-cluster:<name>`, `9d`, `deviation`, `phase:<name>`, `closed:<evidence>`,
  `not-a-divergence`;
* `reason`;
* `ruled`.

- [ ] **Step 1:** List every file in `.superpowers/sdd/queued/`, and every carried item in
      `phase-9a-gate.md`, `phase-9b1-gate.md` and `phase-9c-gate.md`. `phase-9b-gate.md` has no
      carried list. Commit the listing command beside the table.
- [ ] **Step 2:** For each item with a probe, run it on both sides. Where the sides agree, record
      `closed:<probe result>`. Otherwise propose a disposition with a one-line reason. An item with
      no probe gets a proposed disposition from its text, and the reason says no probe exists.
      Items that name a test in `clusters.tsv` get `9d-cluster:<name>`. Commit, and report to the
      controller.
- [ ] **Step 3** (controller): take the `proposed` column to Moritz as one batch, and record his
      ruling per row in `ruled`. Each row ruled `9d` becomes a Task 8 instance.
- [ ] **Step 4:** After the ruling, give each queued file a first line stating its disposition, in
      the shape of `2026-10-11-output-lost-on-kill.md`'s `**CLOSED <date> by ... (<sha>).**`.
      Where a file already states a resolution in its first lines, rewrite that line into this
      shape rather than add a second marker. Commit with `git add -f`.

