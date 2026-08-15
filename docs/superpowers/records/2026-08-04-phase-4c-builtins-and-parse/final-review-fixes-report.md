# Phase 4c final-review fixes

Scope: the 10 Important findings, the 4 Minors ruled FIX NOW, the 3 additions from work that
finished after the review began, and 2 further additions handed over mid-round from the parallel
triage of the deferred minors.

Branch `plan/rust-rewrite`, from `64ce92fe` to `91aa2162`, five commits.

## Commits

| sha | subject |
|---|---|
| `0575e18e` | Pin the three derived exempt attributions, the scan floor, and the binary baseline |
| `bc71b9c4` | State the inheritance contract as a property, and delete two in-repo counts |
| `f0122420` | Record the internal-package routines, and correct two containment arguments |
| `d2610496` | Say what three criteria do not cover, and record which tree each figure is from |
| `91aa2162` | Record the rexxcps measurement, which the 4a spec defined and nobody ran |

The ledger edit (I9) is not in any commit: `.superpowers/` is in `.gitignore`, so
`progress.md` is a working artifact rather than tracked content. The change is on disk.

## Verification, each read unpiped

| command | result |
|---|---|
| `cargo test --workspace --no-fail-fast` | **1,289 passed / 0 failed**, 75 binaries, exit 0 |
| `REXX_CORPUS_GATE=1 ... --test corpus` | 50 of 50 matching, exit 0 |
| `REXX_KEYWORD_GATE=1 ... --test keyword_assertions` | 888 of 896 bodies, 1,737 of 1,773 calls, exit 0 |
| `REXX_ASSERTIONS_GATE=1 ... --test assertions` | 4,224 of 4,259, exit 0 |
| `... --test bif_assertions` (ungated) | 4,920 of 4,999 value, 184 of 186 raise, 6,293 calls, exit 0 |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, **61 crates checked from an `rm -rf`'d `CARGO_TARGET_DIR`**, 0 warnings |
| `bash scripts/mutate-4c.sh` at `bc71b9c4` | **9 of 9 as declared**, exit 0; both baselines 50 of 50 and 75 binaries / 1,289 passed / 0 failed; tree left byte-identical |
| `unsafe` | zero |
| em-dashes in changed Rust files and in `mutate-4c.sh` | zero |

The clippy run was from a genuinely clean target directory: `rm -rf` on the directory first, and
the run reported 61 crates rather than the handful a warm cache produces.

The mutation script was re-run because one of these fixes changes the script itself. It ran at
`bc71b9c4`, which carries every source and script change; the two later commits are documents
only.

## Per-finding disposition

### Important

**I1 — criterion 11's "every row" claim. FIXED.** `phase-4c-gate.md`'s criterion and its §11
assessment now state the split: 66 in-scope rows derived by running each name's probe through
both interpreters, 15 excluded rows derived from `wholly_excluded()` with nothing run, which
`builtin_status.rs:492-504` asserts by pinning the oracle invocation count at 66. The `LINES`
failure scenario is written at the criterion. One correction to the review's evidence:
`bif-exempt.txt` holds **6** `LINES::` rows, not seven, so the gate says "`bif-exempt.txt`'s
`LINES` rows" without a count.

**I2 — criterion 5's falsification does not falsify. FIXED, as a stated limitation.** Reproduced
the review's measurement myself rather than taking it: `eval.rs:372`'s
`_ => Err(Loud::expression(&expr.kind).into())` replaced by `_ => Ok(self.text(b""))` leaves
`cargo test -p rexx-exec --test loud --test owners --no-fail-fast` at exit 0, 8 passed and 5
passed. The same mutation over the workspace exits 101 at 1,287 passed / 2 failed, the catchers
being `eval::tests::a_dot_variable_beyond_the_three_fails_loudly` and
`bif_assertions::the_exempt_set_matches_the_current_failures` — exactly the two the review named.
File restored from `cp` backup and verified with `sha256sum -c`.

I chose the "say plainly what it cannot cover" branch, not the "make the instrument reach it"
branch, and the reason is scope rather than difficulty. Making it reach means giving
`DotVariable`'s second arm an `Owner::Phase` row, which changes `expr_owner`, which changes the
derived attribution of nine `bif-exempt.txt` rows from `UNATTRIBUTED:an environment symbol` to a
phase, which changes criterion 12's own breakdown. That is a deliberate ownership decision the
gate explicitly hands to Phase 5, and taking it here would be deciding it by side effect. The
criterion now names the arm, gives the measurement, names the two catchers, and says what closing
it costs; the assessment row reads "MET, with one split arm stated as uncovered".

**I3 — criterion 4's bar was lowered. FIXED.** An AMENDMENT paragraph at criterion 4 quotes the
plan's wording (`:1945`, "a root the **builtin's own result** holds") beside the gate's, says
which is weaker and why the plan's proved unsatisfiable. The fixed-in-advance claim at `:10-11`
is withdrawn for that criterion by name, and distinguishes it from criteria 1 and 4's later
*pins*, which strengthen rather than move a bar.

**I4 — `BEEP` owned by nothing. FIXED, and wider than reported.** New KNOWN GAP row in
`phase-4-exclusions.txt`. Following the enumeration rather than the one name found two more:
`interpreter/runtime/NativeFunctions.h` lists the portable internal routines in three lines —
`Directory`, `Filespec`, `Beep` — and this platform's `SysNativeFunctions.h` adds none. Measured
all three from a fresh directory: oracle `beep(262,1)` rc 0 and null string,
`filespec('name','/a/b/c.txt')` → `c.txt` rc 0, `directory()` → the cwd rc 0; `rexx-run` raises
43.1 at rc 213 on each. Only `BEEP` reaches an instrument, and the row says why that is an
accident of the ooTest corpus rather than a difference in kind: `FILESPEC`'s 76 `assertSame`
calls all fall to named `DropReason`s and yield no row, `DIRECTORY` has no `.testGroup`. The
gate's `BEEP` paragraph, its findings bullet and its inheritance bullet all cite the register row.

**I5 — the provenance sentence and criterion 6's stale figure. FIXED.** Checked the history
rather than the review's summary: the gate did not exist at `1c94e50b` (it was written at
`89debc85`, one commit later) and recorded 1,286 there. `d0102460` refreshed it to 1,287.
`80ff9c1f`/`64ce92fe` added two more tests, giving 1,289, which the gate never recorded. §6 now
carries a per-tree table (1,286 / 1,287 / 1,289, 75 binaries throughout) and the provenance
sentence names what holds at `1c94e50b` and what does not.

**I6 — `MISMATCH` whitelisted. FIXED, using the device the review named, and falsified.**
`every_exempt_attribution_is_a_known_phase_or_a_declared_outcome` now pins the three derived
categories at exact counts — `MISMATCH` 0, `RAISE-MISMATCH` 0, `ANOMALY` 3 — mirroring
`extract_bif.rs:219-224`. `ANOMALY` is pinned too because it is the same shape and the same
assertion.

Falsified against the review's own scenario rather than by editing an attribution: with
`builtin::string::lower` made to append a space, five `LOWER` rows report `MISMATCH`; applying
the obvious repair (adding those five rows to `bif-exempt.txt` with attribution `MISMATCH`)
leaves `the_exempt_set_matches_the_current_failures` **green** and the count pin as the only red
test in the binary, with the message naming the file and the count. Source and corpus file
restored from `cp` backups, both verified with `sha256sum -c`. The exempt file's header now says
what to do when the pin fires: rule on the row first, then move the number deliberately.

**I7 — the D11 containment premise. FIXED.** The DATE/TIME KNOWN GAP now says the old sentence was
false, gives the real containment (the probes chose conversion forms and `builtin-probes.txt:47-58`
states that choice), and tells whoever strengthens a probe to come back to the row first. Citations
verified: `builtin-probes.txt:91` and `:118` carry the probes, `builtin_status.rs:226-239` runs
each through `run_program` and `oracle.run`. The bare `D11` citation is disambiguated by document,
since two live plan documents define a decision by that number (N12).

**I8 — `mutate-4c.sh`'s self-disabling truncation guard. FIXED.** `BASELINE_BINARIES` is now the
empty string when unmeasured — a value no measurement can produce — and a baseline that measures
zero binaries is fatal with a message naming the cause. The script header's device-2 section and
the gate's device-(e) bullet both record the failure mode. Validated by running the whole script:
9 of 9 as declared, exit 0, baseline 75 binaries.

**I9 — the ledger's handoff undercount. FIXED, with a larger count than the review's.** The
review said "roughly thirty-three" under two phrasings. There are **five**:
`**Deferred minors:**`, `**Deferred minors, for the 4c final review:**`,
`**Deferred, for the 4c final review:**`, `**Deferred to the final review:**`, and
`**Task N: minor (deferred):**` — the review's own block list included `:359` and `:462`, which
neither of its two phrasings matches. The handoff sentence now gives 37 items across 12 blocks
(the count the parallel triage verified), names all five phrasings, and carries a `grep` that
returns all sixteen hits. I did not triage the items or edit their entries.

**I10 — `Activation::nested`'s inheritance contract. FIXED, by stating the property.** The doc
now says it inherits every field `Inherited` carries and names the struct as the enumeration,
rather than listing three of five. The `cached_clock` comment no longer counts inheritances, the
`::ROUTINE` paragraph no longer says "the five fields", "the sixth difference" is now "a further
difference", and `Inherited`'s own doc no longer argues from a past parameter count. No behaviour
change; a doc-only fix, which is why the message is about what the next author would have dropped.

### Minors ruled FIX NOW

**`:659` — the "eleven operations that move them" count. DELETED, not corrected**, in
`parse_template.rs`'s module doc. The count was never in a test; two enumerations of it existed
(the task report's and `impl Cursor`'s) and they disagreed. I left "the five positions", which the
review did not rule on and which the module doc enumerates by name in its own section two
paragraphs down.

**`:1351` — "twelve confidently-wrong rows". FIXED, with the derivation stated.** The gate now
reads eighteen rows, twelve of them mismatches, and gives the arithmetic from the drop table
(`NonUtf8Source` 11 + `SideEffectingAssertion` 6 + `ClockDependent` 1 = 18) so a reader can
re-check without re-measuring. It also connects those twelve to I6: they are the only `MISMATCH`
rows this project has produced, which is the evidence for pinning the category at zero.

**`:1354` — `MISMATCH` whitelisted.** See I6.

**`:1359` — the assessment table's missing denominator. FIXED.** Row 12 now reads "4,920 of 4,999
value rows, 184 of 186 raise rows, out of 6,293 `assertSame` calls, 1,294 of which the extractor
dropped".

### Additions

**A1 — the third-memory-cause entry. FIXED.** `phase-4-exclusions.txt` now records: the coupling
(a perfect third-cause fix cannot reach rc 0 at N=400,000,000, because two live copies of 400 MB
is ~763 MiB against a ~465 MB usable budget, and `reverse` sits at that theoretical minimum today
and still aborts); `PARSE`'s unnamed second copy at `parse_template.rs:670`; the `Vec`
growth-policy penalty on `||` and SAY's trailing newline, with the measured 2N allocation
requests; the three measured threshold bands; and the type-level obstacle — `Interp::to_text`
returns a `Cow::Borrowed` tied to `&mut self` and `Cow::into_owned()` has no fallible form, so
`try_reserve` cannot be pasted in. The SECOND CAUSE's owner paragraph now says "not yours" is true
of the cause and false of the outcome.

**Provenance caveat**: the threshold sweeps and the RSS re-measurements are folded in from
`memory-gap-investigation.md`, dated in the file as 2026-08-06. I did not re-run them. I did
independently verify the citations they rest on (`parse_template.rs:670`, `builtin/mod.rs`'s
`try_reserve` idiom, the absence of any `text_from`/`concat_text` helper).

**A2 — the "unexplained 5.7 MB". RETIRED.** The RSS paragraph now records the explanation: 6.1 MB
on re-measurement, a 5.6 MB baseline gap on `say 1` which allocates no large string, a 572 kB
residual below the instrument's own 792-1760 kB noise floor, the shared-library mechanism offered
as the best fit rather than proof, and the three hypotheses ruled out with how. The ledger's Task
13 entry is untouched, since it belongs to the parallel triage.

**A3 — `rexxcps`. RECORDED in `perf-baseline.md`.** New section stating in its first sentence that
the criteria were written in Phase 4a's design spec (`:508-509`), which calls the program "the
end-of-4c gate", and were never run until now. R1 passes. **R2 fails at ratio 10.02** against a
1.5 threshold, with the 9.93-10.11 band and the independent re-measurement at 10.09. R3's external
cross-check gives 10.92, inside the spec's own 10 % agreement bar. R4's "measured at gate time,
not reused" is recorded as satisfied and the section says it is not itself a reusable baseline. No
optimisation attempted or proposed. The file's intro flags the section as not Task 0.7's, and the
"what is still missing" bullet that said there was nothing Rust to compare against is corrected
rather than left contradicting the new section.

### Handed over mid-round

**Addition 1 — the vacuous scan test. FIXED.**
`a_scan_takes_apart_every_text_the_number_parser_accepts` (`builtin/convert.rs`) asserted only
`scanned || !parsed` over 35 subjects, which a `to_number` accepting nothing satisfies 35 times
over. The count of subjects that parse is now pinned. Exact rather than a floor, because
`subjects` is a literal in the same function and cannot move without the number being looked at.
The value, 19, was read off the assertion's own failure message rather than counted by eye.

**Addition 2 — `rust/CLAUDE.md:57`. DELETED, not corrected.** The illustration's row count for
`keyword-exempt.txt` had been written twice (796, then 16) and was wrong both times; the tree
holds 8. The parenthetical now records that it was written twice and wrong twice and that it is
deleted rather than corrected a third time, because the contrast it draws — a derived, policed
table against a prose count — holds at every size. No count replaces it.

## One unscoped correction I made anyway

**N8**, the gate's "`--no-fail-fast` on every run", which the review ruled Minor. It is a verified
false statement in a document I was already correcting for honesty, and the fix is two words:
device (d) is now scoped to the SUITE runs that carry it, with the reason the CORPUS runs do not
need it (`--test corpus` selects one target, so there is no later binary to truncate). I judged
leaving a known-false sentence in a document whose subject is honesty to be worse than the small
risk of touching one more line.

## Findings I believe are wrong, or where I disagree

None of the ten Important findings is wrong. Two corrections to their evidence, neither changing
a disposition:

* **I1's "seven `LINES` rows"** is six (`/bin/grep -c "^LINES::" rust/corpus/bif-exempt.txt`).
  The gate now avoids the count.
* **I9's "two different phrasings"** is five, and the review's own block list proves it: `:359`
  and `:462` are in the list and match neither phrasing it names. Had the fix named only two, the
  next reader's grep would still have missed two blocks — which is the exact mechanism I9 is
  about.

One place where I did not do what the finding's fix sentence suggested, deliberately:

* **I2** offered "make the instrument reach it or say plainly what it cannot cover". I took the
  second. Reaching it requires giving `ExprKind::DotVariable`'s out-of-scope arm an owner, which
  moves nine `bif-exempt.txt` rows off `UNATTRIBUTED:` and rewrites part of criterion 12's
  breakdown. The gate explicitly hands that decision to Phase 5, and settling it as a side effect
  of a documentation round would be the wrong way to settle it. The criterion now states the arm,
  the measurement, the two catchers and the cost of closing it, which is what "say plainly" asks
  for; it does not restate the claim in softer words.

## What a reader should still not assume

* The mutation script's 9 of 9 was reproduced at `bc71b9c4`. The two commits after it change only
  `docs/` and are not in `MUTATED_FILES`, so the result carries — but it was not re-run at
  `91aa2162`.
* The memory-threshold figures in A1 are the investigation's, dated in the register, not
  re-measured here.
* Nothing in this round changes interpreter behaviour. Every workspace figure is identical to the
  one the review reproduced, which is the intended result: two assertions were added inside
  existing tests, so the test count did not move either.
