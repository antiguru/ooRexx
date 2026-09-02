## Phase 5b review fix round

**BASE:** `60256a8cc`, the commit named in your dispatch. Tree clean, phase closed and gated.

**Read first:** the three review reports in this directory — `final-review-A-liveness.md`,
`final-review-B-tasks.md`, `final-review-C-decisions.md` — and `final-review-plan.md` for why the
review was scoped as it was. Then `rust/CLAUDE.md` and the 5b global constraints.

**The review found exactly one code defect across 51 commits and 171 files. Everything else below is
an instrument or a false sentence.** Do not go looking for more: a prose pass is out of scope, the
comment-policy tightening will rewrite that layer anyway, and correction rounds on this project have
introduced new false statements at 5, 0, 4, 0 across four rounds. Fix what is listed and stop.

---

## 1. B1 — four silent wrong answers. The only code defect, and this phase caused it

Verified by the controller on a freshly built binary, `ir` engine, three descriptors read separately:

| program | oracle | this crate |
|---|---|---|
| `o~sendWith('M', .array~new(2,2))` | rc 168, `Error 88.913` | **rc 0, `a seen 0`** |
| `o~startWith('M', .array~new(2,2))` | rc 158, `Error 98.913` | **rc 0, `a seen 0`** |

`FORWARD ARGUMENTS` and `~run`'s `A` style with the same array are the other two; strand B has all
four with transcripts.

**All four were loud rc 120 refusals until Task 8 landed `Array~new`.** Making arrays constructible
converted four correct refusals into four wrong answers. Nothing in the tree stands over them: no
gated row, no corpus program, no `LICENSED_DIVERGENCES` row, no DEVIATION, nothing in
`refusal-sites.tsv`, and three of the four have no constructor to enumerate.

**Ruled a fix, not a record.** The oracle's own `arrayArgument` overloads both reject
`isMultiDimensional` (`runtime/MethodArguments.hpp:675` and `:717` — strand B audited both citations
and they are right). Matching that is a check, not a surface, so this does not reopen the
argument-conversion work Task 7 deferred. Reject the multidimensional case where the oracle rejects
it, match its error numbers, and give each of the four a corpus witness. Re-measure the oracle's four
answers yourself before building to them.

## 2. C1 — the flip is silently reversible

Nothing asserts `CLOSED_PHASES`'s contents. Confirmed independently: its only mentions in `crates/`
are the definition at `gate_tables/mod.rs:344`, two uses (`:369`, `:445`), and doc comments. Strand C
measured the consequence — with `"5b"` removed **and a 5b row actually broken underneath**, the gate
still exits 0 while printing `5b: 2 rows, 2 not yet agree`. D65 criterion 4 has no failing witness.

Add one. `corpus.rs`'s `SUBSET_FILES` doc records the same shape for its own headline and that one was
given a check; copy its approach rather than inventing one. The control is to remove `"5b"` and
confirm your new test reddens.

## 3. A2 — a committed control that aborts when spelled literally

`gate_table_c.rs`'s `methodsbyclass` control, applied as written (`subscripts.iter().rev()`), is
`rc 134`, `thread 'rexx-interp' has overflowed its stack` — it reads as infrastructure failure in
both harnesses rather than as a reddened row. The control works when only the offset accumulation is
transposed, and in that form reddens `methodsbyclass` alone.

Correct the control's text to the form that works. A control nobody can apply as written is not a
control.

## 4. A4 — a corpus row claiming coverage it does not have

`phase-5b.txt` introduces `instance_self_reassigned.rex` as "the shape the receiver's rooting turns
on". It stays green under strand A's rooting mutations; its only reddening mutation is its own setup
(`creo-skip-init`). Either make the sentence true or make it accurate — do not delete the row.

## 5. B2 — a false reason beside a true claim

`corpus/refusal-sites.tsv:39`: "The test does not re-run a probe — **nothing in this crate can**".
The claim is true and the reason is false: twelve integration-test binaries under
`crates/rexx-exec/tests/` run the oracle through `support::oracle`, one of them
`licensed_divergences.rs`, which this phase built to do exactly that. Two adjacent copies of the
sentence are already correct. The real obstacle is that the witness column is prose, not a runnable
program; say that instead.

## 6. C4 — D61's text contradicted by a licensed row

D61's literal text forbids a `phase-5b.txt` program depending on class-object termination order
"because D60 has not characterised it yet". Task 5 characterised it, D60's conditional clause
licenses `uninit_class_sweep_order.rex`, and the tree is defensible. The decision text is stale.
Correct the spec, and say in it what was removed and why — D61 governs 5c as well.

## 7. A1 — two pairs the `refusal-sites.tsv` check cannot discriminate

`88.909` is carried by `argument_needs_a_string_value` and `named_argument_needs_a_string_value`;
`88.914` by `argument_not_a_class` and `scope_override_not_a_class`. Transposing either pair's
`answer` **and** `witness` leaves all four tests green. Every other ordered pair is discriminated —
strand A planted every row's answer into every row and no plant survived.

Nothing is wrong today; this is `invalid_position`'s shape latent in a file 5c inherits. **Either
make the check discriminate within a shared answer, or record the limit in the file's own header
next to the contract it already states.** If you record rather than fix, say exactly which pairs are
undiscriminated, so a reader can check the claim rather than trust it.

---

## 8. A3 — investigate, do not assume. This one may be a real hole or may be nothing

Deleting **both** `out.push(*receiver)` (`activation.rs:1351`) and `out.push(*owner)` (`:1362`) from
`Activation`'s GC root walk leaves the corpus at 327/327 and the whole workspace's failing set
identical to baseline. Strand A reports the observable only and does not rule it.

**The likely explanation is that every tested program also holds the receiver somewhere else** — a
caller's variable, the exposed pool, `context_object`. The shape that would not is one where the
activation is the only holder. The controller built a candidate and it agrees on all three sides at
BASE, rc 0, `ok held K`:

```rexx
say .K~new~m
::class K
::method init
  expose v
  v = 'held'
::method m
  expose v
  call gc 'force'
  return 'ok' v self~class~id
```

Apply strand A's mutation and run it. **Either outcome is a result and both are worth committing.**
If it reddens, it is the missing witness — add it with a comment saying what it roots and why the
neighbouring shapes do not. If it does *not* redden, then those two pushes are genuinely redundant
under every shape you can construct, which is a finding about the collector and belongs in the report
with what you tried. Do not remove the pushes either way; that is the collector owner's call and not
this round's.

---

## 9. Record with a named owner, do not build

* **A5** — five 5b behaviours no test distinguishes: `native_set_method`'s option/restricted-check
  order (its `native_run` sibling *is* witnessed), `native_start_with`'s argument-position
  substitution, `native_copy`'s non-instance arm, `started_message`'s `validate_scope_override`, and
  `check_restricted_method`'s no-receiver arm, which the code's own comment predicts is unreachable.
  Note that last one is the [[unreachability]] shape this project has been wrong about six times;
  record it as unwitnessed rather than as unreachable.
* **B3** — `refusal-sites.tsv`'s `verdict` column is held against nothing. The phase already measured
  that check's yield: one bad row in twelve. Strand B re-probed thirteen `agrees` rows by hand and all
  still agree, so the table is accurate today with nothing keeping it so.
* **B4** — Task 0's `trace_oracle.rs` control arm still cannot fail, for a new reason 5c inherits:
  the only `Coverage::WitnessedLive` row is in a phase-4 subset.
* **C2** — D59a is named nowhere under `rust/`.

**C3 is ruled not a defect** — D57 is a pure scoping decision and every mechanism it enumerates is
witnessed under another D-number. Do not build a witness for it.

---

## Rules

* Never run a program in `rust/corpus/oracle-crashes.txt` and never construct one of those shapes.
* No `unsafe`. Stop and say so rather than reach for it.
* Oracle probes from a fresh empty directory; three descriptors read separately, never `2>&1`; both
  engines.
* **Restore from a copy made with `cp`, not `copytree`.** Strand A's Python restore preserved mtimes,
  so cargo kept mutated rlibs after a restore and contaminated a batch of its rows. The project's own
  `mutate-4*.sh` use `cp` and do not have this defect.
* **Rebuild before any sweep that follows a restore.** Strand B got a false divergence on
  `object_copy.rex` from a stale binary and only caught it because rebuilding changed the hash. State
  a binary hash beside every figure.
* A gate run does not survive the turn that starts it: statuses to a file, a waiter that exits on the
  process vanishing, and read them in the turn you commit.
* Tree hash before the first gate and after the last, covering untracked files' bytes.
* Commit with `git commit -F <file>`, naming paths explicitly. Never `git add -A`, never amend, never
  a bare `git stash`, never `rm` with a star glob, never `git checkout --` on a file you have edited.
  Confirm `Cargo.lock` is absent from the staged set.
* Report to `.superpowers/sdd/2026-08-27-phase-5b/final-fix-report.md`. Say plainly what you did not
  do.
