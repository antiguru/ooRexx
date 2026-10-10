# Task 12 carry-ins (controller)

* **Queue (Step 1), part one: the brief's own list.** Create a file for each item in the brief's Step 1.
* **Queue, part two: the ledger.** `progress.md` has lines starting `- Task 12 queue add:` (grep for them). Each item in them gets one of three outcomes:
  * Already has a file in `.superpowers/sdd/queued/`: name the file.
  * Fixed during 6.1: name the commit, after re-running the item's probe at HEAD against the oracle.
  * Neither: give it a new file, in the style of the existing ones (title line, finder and date, probe, both engines' output, suspected site).

  Some were resolved in Tasks 11a and 11b, among them the STRING answer, COPIES doubling, the UNINIT drain and the Array append. Put the full mapping in the report as a table with one row per item.
* **Queue items from Tasks 11a and 11b already exist.** These are the `2026-10-10-*` files. Leave them as they are.
* **Bench program.** Commit `dirread.rex` from this directory as a bench program next to the others in `rust/bench-programs/`, so perf runs cover the store-backed directory read. Its base61 figure can be measured with the base binary.
* **Perf (Step 4).**
  * **Base.** The base is base61 `/tmp/claude-1000/p61/t1/bin/base/rexx-run`. Verify its sha256 against the gate record's `## Task 1`. Use `rust/bench-programs/callgrind.sh` on every program.
  * **Current figures.** At Task 11b's close the worst figure was rexxcps +0.4268%.
  * **Wall clock.** A ledger ruling covers this: pingsem, pingguard and emptyloop wall clock sit outside ±4% of base61 at the same instruction count. emptyloop was +5.84% at 11b, pingsem and pingguard about +8.9% and +5.0% earlier. Investigate with `perf stat` (cycles, instructions, context switches, cpu-migrations) and with a layout-pad control: a base61 build with a dead-code pad, or a HEAD build with the pad removed, to show whether layout alone moves it. Code layout alone can move cycles by up to 4% on one axis, so a figure under that is not evidence. Interleave runs: base, head, base, head. State the run counts. Record the attribution, or "unattributed" with the evidence. Do not optimise.
* **Gates (Step 4).** Use `.superpowers/sdd/2026-10-07-phase-6-1/p61-gates/bggates.sh` with `-j 4`.
  * **Commit before gating.** A long gate run can outlive your turn, so commit first, start the gates in the background, and keep the tree frozen until the status file says finished.
  * **Phase 6 criterion 3 TSan run.** The command is in the Phase 6 gate record (`.superpowers/sdd/2026-10-01-phase-6-s2-s5/` or `docs/superpowers/plans/phase-6-*gate*.md`); find it by grep.
  * **Every interpreter run under memcap.**
* **Roadmap (Step 2).** Update rows 6.1 and 9 in `docs/superpowers/plans/2026-07-27-rust-rewrite.md`. Row 9 also absorbs the remaining "Phase 9" refusals still named in `refusal-sites.tsv` or the exclusions file. List those by grep. Do not change any refusal's text.
* **Gate record.** Record each spec section 8 criterion in `docs/superpowers/plans/phase-6-1-gate.md` with its evidence: a command and its output, or a commit. Do not write a result line before the run that produces it.
* **Out of scope.** Any defect you find goes into a queued file, not a fix. Message the controller (SendMessage to "main") if a gate goes red.
* **Report** to `task-12-report.md`. Its last section is "For Moritz": the open questions, the rulings made on his behalf (grep the ledger for `Ruling`), and every parked item. Give one line each, with a pointer.
