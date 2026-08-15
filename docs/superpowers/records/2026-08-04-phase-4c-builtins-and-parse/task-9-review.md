# Task 9 review: the `ADDRESS` instruction and environment tracking

Reviewed: `2070cd9d..27606888` (one commit), against `task-9-brief.md` and `task-9-report.md`.
Tree read at `9a260d2f` (the controller's own follow-up commit); where that commit changes what the diff left behind, it is called out.

## Verdict 1 -- spec compliance: PASS

| brief requirement | verdict | evidence |
|---|---|---|
| Scope is the environment name: implement `environment` and `dynamic` | ✅ | `exec_address` (`run.rs:6053-6081`) handles the constant, both computed spellings and the bare form |
| `command` and `io` stay Phase 7's and still fail loudly | ✅ | ran `target/debug/rexx-run` on five sources: `address cmd ''`, `address with output stem o.`, `address value 'q' with input stem i.` all rc 120 with `ADDRESS is not implemented (Phase 7)`; `address cmd` and `address envA` rc 0 |
| Step 0(a): the constant form upcases, the `VALUE` form does not | ✅ | `only_the_symbol_form_upcases_the_environment_name`, four spellings; the upcase is the parser's and nothing here folds case |
| Step 0(b): bare `ADDRESS` is a toggle, not a stack | ✅ | `bare_address_is_a_toggle_and_not_a_stack`, five rows, **both halves** of the pair; see the mutation below |
| Step 0(c): per-activation, inherited on call, discarded on return | ✅ (return half asserted; call half honestly unasserted and forwarded) | `Activation::address` + `Inherited::address`; `a_callees_own_environment_does_not_survive_the_return` |
| Step 1: state on `Activation`, not `Interp` | ✅ | `activation.rs:110`, initialised in `new`, taken from `Inherited` in `nested` |
| Step 1: probe the two-deep swap, the swap with no prior environment, survival across `CALL`/`RETURN` | ✅ | report P1, P2, P5; the flagged gap (bare `ADDRESS` with no prior environment) was closed and written back into the plan |
| Step 2: split the `loud.rs` witness arm-grained rather than deleting it | ✅ (with the corrected source) | `Address::Command` / `Address::Environment`, following the `Call::Qualified` pattern |
| Step 3: move the `owners.rs` row | ✅ | `EXPECTED_OUT_OF_SCOPE` row `Address` `4c` -> `Address::Command` `Phase 7`; counts 43->44, InScope 34->35, `4c` 1->0, `Phase 7` 1->2 |
| Step 4: shared verify block, one commit | ✅ | report's four commands with unpiped statuses; `git status --porcelain` empty at `27606888` |
| Verify block "the gate must stay at 42 of 42" | deviated, accepted by the controller | now 47 of 47; see the scope section |

### The two corrections the brief needed

**`address cmd` is the constant-environment form.** Verified in this tree, not re-derived from the oracle: `address cmd` now runs at rc 0, and the new witness `address cmd ''` fails at rc 120 with the exact suffix `(Phase 7)`. So the new witness fails for the reason claimed, and the old one would have asserted a loud failure for a construct this task makes run.

**Both directions still answer for the command and `WITH` forms.**
* Phase-owned -> loud: `assert_witness_set_is_complete` pins `INSTRUCTION_WITNESSES`' tag set equal to `owners.rs`'s phase-owned rows (still exactly 9), and `every_out_of_scope_variant_fails_loudly` runs the witness and `ends_with`-checks the suffix built from `lib.rs`'s own `instruction_owner`. `assert_constructs` first checks the source parses into `Address::Command`.
* InScope -> must run and be witnessed: `Address::Environment` is now an `Owner::InScope` row, so `coverage.rs`'s `every_in_scope_variant_is_witnessed_by_the_phase_subsets` (`Coverage::unwitnessed`) requires a corpus program constructing it -- `lang/address_env.rex` supplies it, and `EXPECTED_SUBSET_4C` pins the line.
* `run.rs`'s `the_command_and_with_forms_stay_loud_and_name_phase_7` covers the three shapes the single `loud.rs` witness does not spell, including `WITH` with no command.
* `loud.rs`, `owners.rs` and `coverage.rs` all pass here.

**"Wrong from the third bare `ADDRESS`" was one toggle too generous, and the test discriminates at the tightest point.** Verified by mutation (file backed up with `cp`, restored from the backup, `git status --porcelain` empty afterwards): replacing `std::mem::swap` with `self.current = self.alternate.take()` reddens `bare_address_is_a_toggle_and_not_a_stack` **at `after 1 bare ADDRESS`**, `left: (Some("ENVA"), None)` vs `right: (Some("ENVA"), Some("ENVB"))`. Asserting both halves of the pair is what buys that; the current half alone would not diverge until the second toggle. The correction is right -- but see finding 1: the production doc comment still carries the uncorrected version.

## Verdict 2 -- task quality: findings

### Important

**1. `activation.rs:92` restates the claim this task corrected everywhere else.**
`AddressState`'s doc says "A stack gets the first two right and is wrong from the third onward." That is the brief's un-corrected sentence. Under any pop-stack reading, the alternate is wrong after one toggle and the current after two -- which is exactly what the plan (`2026-08-04-...:1260`) and `run.rs`'s own test doc now say, and what I measured above. The corpus-facing prose, the plan and the test doc were fixed; the production doc comment was not. This is the "correction rounds leave a false statement in the neighbourhood" shape rather than a new error.

**2. `corpus.rs:441` dates the 47 to a commit at which it is 42.**
"Expected result at commit `2070cd9d`: **47 of 47 matching**". `2070cd9d` is this diff's *parent*: there `corpus.rs` passed two files and `lang/address_env.rex` did not exist, so the harness reported 42 of 42. The precedent row above it names `a9420630` -- the commit that *contains* the union change, added later by `ebbfb3d7` -- so the convention is "the commit at which the number is true", i.e. `27606888` here. The number itself is right and is counted by the harness (`subset.len()`, 30 + 12 + 5), not hardcoded.

**3. `corpus.rs:202-204` still claims the two-file call site.**
`read_subset`'s doc: "The caller below passes `phase-4a.txt` and `phase-4b.txt` since 4b's Task 1." The call site is now three files, forty lines below. The module doc that was wrong is now right (checked: the union sentence and the new "that is exactly what had happened again" paragraph are both accurate, and the report banner names three files); this second statement about the same fact, in the same file, was missed.

**4. `corpus/lang/address_env.rex:5` says "exactly four readers" and then lists five.**
`toggleAddress`, `setAddress`, `CommandInstruction`, `ADDRESS()`, the external-`.rex` default. The report's own prose says "That is five readers of `currentAddress`", and the plan's Task 10 Step 0a lists the five without a count. The same bytes are duplicated into the generated fixture `rexx-parse/tests/sourceline_oracle/address_env.txt:6`, so the fix is two files.

**5. `keyword-exempt.txt:43` labels the new group "The `ADDRESS ... WITH` bodies", and one of the three rows is not a `WITH` form.**
`ADDRESS::test_environment_path_null` is `address "path" ""` -- the *command* form (`corpus-l1/ADDRESS_test_environment_path_null.rex:3`). The other two are genuine `WITH` bodies (`address value environment with input normal`, `address (environment) with output normal error normal`). The `Phase 7` attribution is derived from the loud message and policed both directions, so all three rows are correctly *owned*; only the group's headline sentence is wrong, and the report repeats it ("three `ADDRESS ... WITH` rows moved"). The following sentence does mention "issuing a command to one", so the fix is one clause.
`ASSIGNMENT::test_6`'s removal is correct: its body contains `Address nowhere`, the constant form this task implements.
The header's stale counts were deleted, not restated -- checked: the only surviving numeric header claim is "6 bodies" for the defect group, which matches (6 rows; groups now 102 `4c` / 6 defect / 3 `Phase 7`).

### Minor

**6. `activation.rs:114`: "the one reader of this state that is not a command dispatch".**
The external-`.rex` default (`RexxActivation.cpp:3147`) is also a non-dispatch reader, as the same commit's corpus header says. "The one reader a Rexx program can reach" would be true.

**7. `address_env.rex:24`: "prints three extra lines" matches no clause count in block A.**
Block A has two constant/literal clauses and four clauses that trace nothing. "Three" is the number of *forms* named in the preceding sentence, not lines; the companion figure ("two fewer") is right.

**8. The report's flagged inference about Windows is false.**
"The Windows limit differs from 250" -- `platform/windows/MiscSystem.cpp:74` is also `MAX_ADDRESS_NAME_LENGTH = 250` with byte-identical logic. The shipped comments only say "a separate one under `windows/`" and "a per-platform value", which stay true, so no code or comment change is required; the report's flagged-inference list should not carry it.

**9. `AddressState` derives three traits nothing uses.**
Measured: reducing `#[derive(Clone, Debug, Default, PartialEq, Eq)]` to `#[derive(Clone, Default)]` leaves `-p rexx-exec` compiling and its tests passing (backed up and restored; tree clean).

**10. `a_bare_address_with_no_prior_environment_changes_nothing` does not discriminate a toggle from a no-op.**
It stayed green under the pop mutation, and a `toggle` gutted to `{}` also leaves it green. Its only failing edit is a change to how the default is represented -- which is a live Task 10 decision, so it is not vacuous, but it is the weakest of the seven and its doc reads stronger than that.

**11. `collect_stress.rs:103` now carries the same false claim `corpus.rs`'s module doc did.**
"the caller below passes `phase-4a.txt` and `phase-4b.txt`, so criterion 4's stress run covers every later phase's programs too". Explicitly Task 15's remainder and recorded as such by the controller in `9a260d2f`; noted only because it is the identical defect this task's deviation was justified by.

**12. `coverage.rs:437`'s "union 42, which is criterion 1's own headline" now reads as the current total.**
It is a dated 4a+4b figure, and its conclusion ("a run over the committed files never takes the `seen.insert` false branch") now silently depends on `phase-4c.txt` also not overlapping. It does not overlap; the premise is just unstated.

## The scope call: done correctly

Not relitigated. Checked:

* **Union order and de-duplication.** `corpus.rs` passes `phase-4a.txt`, `phase-4b.txt`, `phase-4c.txt` in that order to the same `read_subset` shape `coverage.rs` uses, which unions in first-seen order through a `HashSet`. `coverage.rs`'s call site already used the identical three-file list, so the two harnesses now read the same set in the same order.
* **The 47 is counted, not written down.** `total` is `subset.len()`; the only `47` in any `.rs` file is the dated prose row (finding 2). 30 + 12 + 5 = 47, counted from the files.
* **Nothing previously passing changed meaning.** The 42 earlier programs are unaffected; `EXPECTED_SUBSET_4C` gained one line, in file order, and `phase_4c_subset_matches_the_committed_list` still pins it. `sourceline_oracle` is directory-scanned, so the new `.rex` *required* a new fixture; it is byte-identical to the program and its `count 78` matches.
* **The module doc that was wrong is now right**, and `corpus/README.md`'s "read alongside both earlier files" -- previously false -- is now true. The plan's Task 15 Step 4 still claimed the two-file call site at `27606888`; the controller corrected it in `9a260d2f`.

## The central negative result: checked independently, and it holds

The equivalent claim in Task 8's brief was false, so I looked for an observer the enumeration would miss rather than re-reading it.

* The C++ grep is complete and I reproduced it: `getAddress|currentAddress|alternateAddress` over `interpreter/` outside `platform/` returns 21 lines and nothing outside the five readers named. Positive control: `ddress` matches 20+ files under `classes/`, so the pattern finds hits where they exist -- and `ContextClass.*` has no `address` member, so there is no `.context~address` observer even in Phase 5.
* `PARSE SOURCE` is not an observer: `RexxActivation::sourceString` (`:4582`) builds platform + calltype + program name.
* Reader 4 is unreachable here: external routine resolution is `Loud::unresolved_call` (`lib.rs:517`), owner `4c`.
* `INTERPRET` does not push an activation, so a fragment's `ADDRESS` writes the enclosing activation's own pair -- no hidden inheritance path.
* Ran it: `address envA; say address()` gives `routine "ADDRESS" is not implemented (4c)`, rc 120. `builtin-status.txt:43` records `loud`, and `tests/builtin_status.rs` derives that table by running each name, so the boundary claims in the new comments are machine-asserted rather than prose.

The obligation is in **Task 10's own section** of the plan as "Step 0a", and it names both properties: the swap at least three deep, and inheritance into a callee with both halves, plus the file to extend and the fact that `None` needs a rendering. One consequence worth stating: `resolve_and_run_call`'s `caller.address.clone()` is exercised by no test today (replacing it with `AddressState::default()` is unobservable in-crate for the same reason), which is precisely what Step 0a is charged with.

## Other things checked and found sound

* `Raised::environment_name_too_long`'s substitution order matches the catalogue: `errors.rs` 29.1 is `Environment name exceeds &1 characters; found "&2".`, and the call passes `[limit, name]`. `substitute` does not truncate, so the doc's "untruncated" is accurate for this path.
* Order of operations matches the C++: `AddressInstruction.cpp:176-186` traces then validates on the `VALUE` form, and validates the constant form at execute time (not parse); `toggleAddress` validates nothing. `MAX_ADDRESS_NAME_LENGTH` is 250 with a strict `>`, as implemented.
* `set`/`toggle` match `RexxActivation::setAddress`/`toggleAddress` exactly, including `set` discarding the old alternate.
* `exec_address` follows the crate's `eval` -> `push_temp` -> `to_text` -> `trace_result(current_value_indent)` pattern used by the `TRACE VALUE` arm directly above it, and stores plain `Rc<[u8]>` rather than a GC value.
* The `tags!` macro's single widened rule subsumes the deleted one; the zero-split invocation (`expr_tag`) still compiles, and both `split` sections stay wildcard-free (`bool` is exhaustive).
* No `unsafe`, no em-dashes in any changed file, `cargo test -p rexx-exec --lib address|environment`, `--test loud --test owners --test coverage` all green here.
* `git status --porcelain` empty after every mutation I applied.

## Cannot verify from diff

* ⚠️ **M6's "invisible to the entire suite without the wiring"** -- I did not re-run the suite with `phase-4c.txt` removed from `corpus.rs`.
* ⚠️ **M2-M5's "and nothing else in the suite"** -- only M1 was re-run here.
* ⚠️ **The oracle transcripts P1-P9 and the corpus program's byte identity** -- taken from the report and the controller's independent 47-of-47 confirmation.
