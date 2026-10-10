# Task 11a fix round 2 (controller)

Read the "## Re-review 1" section of `task-11a-review.md`. The probes are in `/tmp/claude-1000/p61/t11ar/probes/r1/`. The rules from fix round 1 still apply.

The rule the rulings below enforce is that a site joins the `.nil` set only when the oracle's answer is defined. Defined means two things:
* No input distinguishes the answer from a read of `.nil` through the string layout.
* It is witnessed over varied inputs, not over one or two.

Two inputs that happen to match do not show an answer is defined.

* **N1. Ruling:** the ordering operators (`<`, `>`, `<=`, `>=`, `<<`, `>>`, `<<=`, `>>=`) refuse with the R11 loud refusal. Equality (`=`, `==`, `\=`, `\==`) stays in the `.nil` set, keeping the primitiveIsEqual rule. Give the witness's ordering line a refusal row, and add a crate test that shows `"" << o` refuses.
* **N2. Ruling:** for PARSE VALUE and PARSE ARG, a template with exactly one variable and no pattern assigns the `.nil` object itself, so `p == .nil` is 1 as on the oracle. Every other template refuses, including any word, position or literal split and PARSE UPPER. Witness each kind on the oracle in 3 runs. If assigning `.nil` there needs more than the one site in `parse_template.rs`, refuse that form too and say so.
* **N3. Ruling:** OPTIONS goes back to the refused set. Nothing observable can show that its answer is defined.
* **N4:** correct the classification table, Deviation 30 and the R11 line in the spec to match.

Append a `## Fix round 2` section to the report. Run the per-task check, the corpus pair, and whole_groups plus the seeded gate in release. Run callgrind on the eight programs if `eval.rs` changes. Commit, then return the status, commits, test line, perf line and concerns.
