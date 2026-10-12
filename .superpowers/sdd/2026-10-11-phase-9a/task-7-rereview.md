# Task 7 fix round 1 re-review (40278c99e..1ff284fd9)

* Important 1: ADDRESSED. `install.rs:263` now reads "`Interp::lostdigits_armed`, under the same set-once rule" (no "only write"); true, since the other writer `options.rs:543` is `|=` (also set-once). `lib.rs:1655-1658` names both routes (installed `::OPTIONS`, `Package~options` write). Writers of `lostdigits_armed`, grep once: `install.rs:269`, `options.rs:543`, init `lib.rs:2177`; no others. The two neighbouring "third"/"other" edits (`install.rs:253`, `run/condition.rs:30`) are true: `reqstr_armed` also has `options.rs:542`, `install.rs:1575`, so the old counts were already wrong.
* Minor 1: ADDRESSED. `introspection-arguments.tsv` header no longer names `Package~defaultOptions` (row at :121 intact).
* Minor 2: ADDRESSED. `rexx-num/src/settings.rs` `from_parts` doc: caller owns `digits >= 1`, `fuzz >= digits` allowed. Matches review's focus 1.
* Minor 3: ADDRESSED. Oracle run of `package_default_options.rex` (fresh dir /tmp/claude-1000/p9/t7rr, companion .cls copied): rc 0, stderr empty, 54 lines. Printed: `digits 3 fuzz 5: 3 5 0 0 1`, `digits 5 fuzz 5: 5 5 1 1 0`, 33.1 for bare `NUMERIC DIGITS` at digits 3/fuzz 5, `inherited: 14 ENGINEERING INHERIT`, `not inherited: 9 SCIENTIFIC`, `main activity: 7 ENGINEERING C -2`, `third activity: 7 ENGINEERING C -3`. All match the report. The corpus records no expected stdout (phase-9.txt:22 lists the file only; differential is oracle vs crate), so I did not rerun the crate. `sourceline_oracle/package_default_options.txt` count 106 is consistent with the added 27 lines (79+27).
* Minor 4, 5: not in scope, untouched.

New defects: none.
