# Final whole-branch review -- 2026-08-14-pre-phase-5-defects

Range reviewed: `625d653fb..613983c0b` (22 commits), branch `plan/rust-rewrite`.
Reviewer state: tree clean at `613983c0b` on entry and on exit (`git diff --quiet`
checked after every mutation); `rexx-run` rebuilt from HEAD before and after every
mutation experiment.

**VERDICT: the behavioural changes are correct and independently re-measured against
the live oracle; merge is blocked only on five false statements in prose, all
correctable without touching code.**

---

## What I re-measured, and what came back right

Everything in this section I ran. Nothing here is read-and-believed.

| Claim | How verified | Result |
|---|---|---|
| Every stanza of `ir_dual_cases/loop-header-boundaries` is the oracle's bytes except the row whose comment says otherwise | extracted all 11 stanzas, ran each on the live oracle from a fresh empty dir, compared rc/stdout/stderr with the path substituted | exactly one row differs (`row 3`, the failing-handler row), and it is the row whose own comment says so |
| Same for `ir_dual_cases/loop-header-values` | same method, 4 stanzas | all four byte-identical to the oracle |
| Every `tests/trace_indent/*.expected` is the oracle's bytes | ran all 10 `.rex` under the oracle, substituted the path, compared all three descriptors | all 10 byte-identical, rc included |
| `Entry`'s 6x2 oracle table (`activation.rs:611-620`) | wrote one program per cell, 12 programs, ran each on the oracle | every cell matches, incl. rc 239 / rc 158 / rc 157 / rc 0 |
| The crate agrees with the oracle on those cells | ran the same 12 on both engines | 8 byte-identical on all three descriptors; the two `USE LOCAL`-in-a-label cells differ only in the pre-existing parse-error report shape (rc 120 vs 157, `rexx-parse` untouched by this plan), and the two `::METHOD` cells are Phase 5 |
| `settle_block_indent`'s 7-row doc table (`run.rs:6233-6238`) | 5 rows against the committed `.expected` files, plus the 2 rows with no committed case reconstructed and run on the oracle | all 7 match; the crate is byte-identical to the oracle on both uncommitted rows, both engines |
| The new `DO OVER ... FOR` unit-test transcripts (`run.rs:11386-11412`) | ran both programs under the live oracle | byte-identical under `trace i` and `trace r`, both engines |
| POS/LASTPOS correctness | generated a 7788-row grid (6 haystacks x 6 needles x every start x every range), ran on oracle + both engines | zero divergences |
| DEVIATION 3's bound ("only a needle whose last byte is `'00'x`, window reaching the end") | re-derived from `find_forward`'s own arithmetic (`over + needle.len() <= haystack.len() + 1`) and from `StringUtil.cpp:206-253`; probed 8 NUL-needle cases | bound holds; the licensed divergence reproduces (`pos('a'||'00'x,'aa')` 2 vs 0) and nothing outside the bound diverges |
| `StringUtil::lastPos` has no rescan (`find_backward`'s doc) | read `StringUtil.cpp:341-403` | correct: clips once, fixed count, plain `memcmp`, no `memchr` fast path |
| The DO clause-boundary causal account | read `step`'s `Do` arm -> `run_loop`, `in_stepped_clause_with`, `leave_stepped_clause` -> `clause.rs:493` `leave_clause` -> `deliver_pending_trap`; read `ir/compile.rs:427-430` (`Op::LoopRun` then `close_region`); read `SimpleDoInstruction.cpp:71` | every link confirmed; see "Attempts to refute" below |
| `oracle-crashes.txt` entry 2's C++ citations | read `BuiltinFunctions.cpp:1163`, `RexxDateTime.cpp:457-475`, `:526` (**not run**, per the file's own rule) | all three cite what the file says they cite; `monthNames[month - 1]` is exactly line 526 |
| The new `assert!(!owns_frame)` at `run.rs:2431-2434` is not a new abort path | traced every `owns_frame` writer; `Activation::for_internal_call` sets it false and the only setter to true is `exec_procedure` itself, which is gated on `first_instruction` | unreachable from user input |
| Gates | full workspace suite at HEAD, before and after all mutation work; corpus gate | 83 `ok` blocks / 0 `FAILED`, exit 0; `REXX_CORPUS_GATE=1` -> 53 of 53 |

### Attempts to refute the DO causal account (`loop-header-boundaries:257-274`)

I tried four discriminators and could not refute it.

1. **Empty body** (the file's own control): oracle and both engines print `G ran 5 / after set 5`. Kills "the boundary is missing".
2. **Nested plain DOs**, outer line 5 / inner line 6: oracle `G ran 5`, both engines `G ran 6`. Kills both "absent" and "correctly placed".
3. **Under `trace r`**: the handler's own five lines are byte-identical between oracle and crate; only their position and `SIGL` differ. Confirms "right level, wrong place" as against the plain-`SELECT` defect, which is the reverse.
4. **Triple requeue** (`h` -> `g` -> `k`, body `say 'body'`): oracle `G ran 5 / body / K ran 6 / after 5`, both engines `body / G ran 6 / K ran 6 / after 5`. Consistent with the account; note the crate's second delivery lands before `say 'after'`, i.e. where the account puts the still-open `DO` boundary, but reports the stale clause line 6 rather than 5, so it does not discriminate on `SIGL`.

The account survives. It is the best-supported causal statement in the branch.

### Can the new tests fail, and do they add coverage

Method: apply one mutation, run the **whole workspace suite** with `--no-fail-fast`,
and read off *every* catcher, so "the new test caught it" is measured rather than
assumed. Restored from a copy after each, verified with `git diff --quiet`.

| Mutation | Catchers (whole suite) | New test is the only catcher? |
|---|---|---|
| A: drop `activation_indent`/`indent_offset` reset in `Flow::Signal` (`run.rs:1351-1354`) | `trace_indent` only, cases `signal_from_a_called_label`, `..._intermediates`, `signal_on_syntax_raised_in_a_callee` | **yes** |
| B: drop `settle_block_indent(outcome.entered(), do_indent)` (`run.rs:6442`) | `trace_indent` (2 loop-header cases) **and** `ir_dual` row 3 of `loop-header-boundaries` (a pre-existing stanza whose expected byte this plan changed) | no |
| C: drop `settle_block_indent(!held, do_indent)` (`run.rs:6552`) | `trace_indent` only, `call_on_at_an_until_test_that_ends_the_loop` | **yes** |
| C2: *invert* the same call (`!held` -> `held`) | `trace_indent`, **both** UNTIL cases | proves `call_on_at_an_until_test_that_runs_another_pass` can fail -- the deletion mutation is blind to it (the recorded deletion-blindness pattern), the inversion is not |
| D: drop `settle_block_indent(true, indent)` in `select_case` (`run.rs:5508`) | `trace_indent` only, both `select_case` cases | **yes** |
| W: settle a `WHEN`'s boundary like a block instruction's | `trace_indent` only, `call_on_at_a_when_condition_boundary` alone | **yes**; the adjacent success is a live pin, and it isolates |
| E: revert `HeaderRole::OverFor`'s `FOR` keyword | the new unit test, the renamed golden IR test, **and** `ir_dual`'s `loop-header-values` row | no -- see finding I3 |
| F: remove POS's window overrun | `builtin::string::tests::the_searching_builtins_answer_the_oracles_own_bytes` only under a plain `cargo test`; **plus** `lang/pos_window.rex` under `REXX_CORPUS_GATE=1` (52 of 53) | see minor M7 |
| G: let `Entry::Routine` count as an internal call | the two new `run.rs` unit tests, `corpus_differential`, `both_engines_agree_across_every_population`, `the_l0_subset_passes_again_under_collect_on_every_allocation` | no, and that is healthy |

I also reproduced the `.wrong` files mechanically rather than trusting the
gitignored report: mutation A reproduces all three `signal_*.wrong` **byte for
byte on both engines**, and mutations B+C reproduce three of the four loop
`.wrong` files. `call_on_at_an_until_test_that_runs_another_pass.wrong` is not
reproduced by any deletion (same blindness as C above); it is only an input to
`deviation_0_collapses_...`, so this is provenance, not correctness.

I additionally checked the one comment the deferred list calls out as
understating itself: removing the UNTIL clause's **boundary** (keeping the line)
leaves the entire workspace suite green, so `run.rs:6528-6530`'s "this clause's
boundary is unobservable on every probe tried" is still **true** at HEAD. The
load-bearing line is the `settle_block_indent` call inside the closure, not the
boundary. Measured, not assumed.

---

## Findings by severity

### Critical

None.

### Important

**I1. `rust/crates/rexx-exec/tests/ir_dual_cases/loop-header-boundaries:270-274` -- the
causal account's last paragraph attributes crate behaviour to the oracle, and is
false under its own subject.** The paragraph reads "The oracle's
`RexxInstructionSimpleDo::execute` returns once it has opened the block, so its
clause ends before any body instruction runs. Delivery then goes to whichever
boundary arrives first: with a body that is the body's first clause, and with no
body it is the `DO`'s own." Verified by running the file's own named program
(`:232-240`): the **oracle** with a body prints `G ran 5` *before* `body` -- it
delivers at the `DO`'s own boundary, never at the body's first clause. The
sentence is only true of *this crate*. Introduced by `613983c0b`, which split one
paragraph in two and left "The oracle's" orphaned on its own line at `:270`,
stranding the following sentence under the wrong subject; it contradicts the
transcript recorded 25 lines above it at `:245-246`.

**I2. `loop-header-boundaries:55-58` still states the attribution the same file
withdraws at `:67-77`.** The `UNTIL` bullet ends "The `END` is an instruction the
oracle executes and this crate never steps -- it echoes it and jumps past it",
which is the elided-instruction mechanism; twelve lines later the file says "**That
attribution is withdrawn rather than replaced**: nothing anyone can run supports
it." Both sentences are live. Verified by reading the committed diff: the bullet's
text was untouched while the paragraph beneath it was rewritten -- the recorded
"correction lands at the summary, the copy survives at the finding" shape. Two
further measurements bear on it, both run: (a) the file's own committed `UNTIL`
row at `:381-398` prints `H ran 5` naming the `END` on **both** sides, so this
crate does run a boundary attributed to the `END`; (b) I reconstructed a program
that reproduces the bullet's **exact numbers** -- oracle `G ran 5`, both engines
`G ran 6` -- in which line 5 is the *body clause* and line 6 is the `END`, i.e.
the opposite construct attribution to the bullet's own words, and in which the
crate delivers *earlier* rather than later. Program (`untilrequeue.rex`):
`call on user zx name h` / `call on user zy name g` / `zn = 0` /
`do until done()` / `zn = zn + 1` / `end` / `say 'after' zn` / `exit` / `done:` /
`if zn = 1 then raise user zx return 0` / `return zn >= 2` / `h:` /
`raise user zy return 1` / `g:` / `say 'G ran' sigl` / `return`. This is a fourth
reconstruction that matches the numbers and not the words, which is stronger
evidence for "the bullet cannot be checked" than the file currently carries.

**I3. `rust/crates/rexx-exec/src/run.rs:11376-11382` -- "Nothing else in the tree
pins this line" is false, and so is its reason.** The doc comment on
`a_do_over_for_echoes_the_for_keyword_the_oracle_prints` says "Nothing else in the
tree pins this line: `trace_oracle.rs` carries no witness for this shape, and
`ir_dual_cases/loop-header-values` compares the two engines to each other rather
than to the oracle". Measured (mutation E, whole suite, `--no-fail-fast`): **three**
tests go red -- this unit test, `ir::golden_tests::a_block_has_an_empty_header_
region_and_a_do_over_for_echoes_both_its_target_and_count`, and
`both_engines_agree_on_every_case_file` at `loop-header-values:192`. The stated
reason is also wrong: `render_both_engines` compares the two engines *and* the
tree-walker's bytes against the stanza's recorded block, which that file's header
states was measured against the oracle -- and which I re-ran against the live
oracle today and found identical. The `trace_oracle.rs` half of the sentence is
correct (checked: no `DO ... OVER ... FOR` in any of its 28 files). The ledger
itself records "all three catchers going red", so the shipped comment contradicts
the run's own record.

**I4. `loop-header-values:184-185` -- "no other test or case file runs that shape
under trace at all" is false, falsified by the same commit that wrote it.**
`84873b191` added `run::tests::a_do_over_for_echoes_the_for_keyword_the_oracle_
prints`, which runs `do qq over zs for 1` under `trace i` *and* `trace r`. Read
directly from the commit; the two statements (this and I3) were shipped together
and contradict each other as well as the tree.

**I5. `rust/crates/rexx-exec/tests/trace_indent.rs:33-36` -- a count that its own
list falsifies.** "Both quantities are the oracle's own `settings.traceIndent` ...
The two places the derivation parts company with the counter are what these cases
hold:" is immediately followed by three bullets. Verified against git: `932e6b710`
(Task 5) added the `SELECT` bullet and left "Both"/"The two places" untouched. It
is both a false statement and an instance of the house rule against naming a set's
cardinality.

### Minor

**M1. `loop-header-boundaries:1-2` -- the subject line no longer describes the
file.** "Where a `DO`/`LOOP` header's clause boundaries fall" now heads a file that
also holds a `SELECT CASE` traceback quantity, a plain-`SELECT` boundary indent,
and a plain-`DO` stdout ordering defect. Read, not run. (This is the deferred minor;
confirmed.)

**M2. Set-cardinality prose in `loop-header-boundaries`, in more places than the
deferred list names, and one of them is now an undercount.** `:17` ("Two rows are
about..."), `:23` ("Three rows are about..."), `:31` ("One row differs...") and `:39`
("TWO LOOP-HEADER BOUNDARIES DIVERGE ... AND ARE NOT ROWS HERE"). `:17`, `:23` and
`:31` I verified true by counting the stanzas and by the oracle re-run above. `:39`
is now misleading: the file records further non-row divergences (`:168-179`,
`:188-215`, `:224-253`) inside the row-3 comment block. All four predate this plan's
edits to the surrounding text.

**M3. `rust/corpus/oracle-crashes.txt:15` -- "Three of the four are deterministic
memory-safety defects" names the cardinality of the file's own entries.** Read, not
run (running the entries is forbidden by the file itself and by the brief). Deferred
minor, confirmed.

**M4. `trace_indent.rs` cosmetics, all read and confirmed at HEAD:** `:232` the
`&String` in `Vec<(&String, Vec<String>)>` is collected and never read (the name is
dropped by both consumers); `:244` "in {descriptors} descriptors" has no plural
agreement at 1; `:269-270` a doc-comment paragraph runs on into the next without a
`///` separator.

**M5. `trace_indent.rs:113-128` / `:300-311` -- `every_case_ships_all_three_files`
cannot see a half-added case.** `case_names()` enumerates `*.rex`, so an
`.expected`+`.wrong` pair with no `.rex` is invisible to the very test meant to
catch a half-added case. Read; the failure mode is one-directional and benign
(nothing runs), so it is a gap rather than a wrong answer. Deferred minor, confirmed.

**M6. `loop-header-boundaries:69` cites `task-4bp-report.md`, which exists only
under the gitignored `.superpowers/` tree.** Verified with `find`: the only copy is
`.superpowers/sdd/2026-08-09-phase-4e-ir/task-4bp-report.md`. A tracked test file
citing an untracked path is a dangling reference in a fresh clone. All other paths
cited by added lines in this diff exist (checked mechanically).

**M7. `corpus_differential` only asserts under `REXX_CORPUS_GATE`, so the new corpus
programs do not gate a plain `cargo test`.** `corpus.rs:545` is
`assert!(!gate || mismatches.is_empty(), ...)`. Measured: under mutation F the
plain workspace suite stayed at 83 `ok` / 0 `FAILED` for the corpus binary and only
the `string.rs` unit test caught it; with `REXX_CORPUS_GATE=1` the run went 52 of 53
and named `lang/pos_window.rex`. Pre-existing design, documented in that file's own
module doc, and not a defect of this plan -- but `phase-4c.txt`'s new note that "a
witness the oracle re-measures on every run belongs here too" is only true of the
report, not of the gate. Worth stating once so nobody reads the new corpus programs
as suite-enforced.

### Not defects of this branch, but found while reviewing and worth Task 6's time

**N1. A new, unrecorded, pre-existing divergence, on stdout with no `TRACE`: an
empty `do`/`end` with a *double* requeue elides the `END`'s boundary.** Program
(`emptytriple.rex`): `call on user zx name h` / `call on user zy name g` /
`call on user zw name k` / `zn = raiser1()` / `do` / `end` / `say 'after' zn` /
`exit` / `raiser1:` / `raise user zx return 5` / `h:` / `raise user zy return 1` /
`g:` / `say 'G ran' sigl` / `raise user zw return 1` / `k:` / `say 'K ran' sigl` /
`return`. Oracle `G ran 5 / K ran 6 / after 5`; both engines `G ran 5 / after 5 /
K ran 7`. This is exactly the elided-`END` mechanism the file withdrew as
unsupported (I2) -- with a named program. It also qualifies the file's "An empty
body agrees, and that is what pins it" (`:276`): the empty body agrees for the
one-requeue program recorded there, and does not for this one.

**N2. Both N1 and the I2 reconstruction are pre-existing.** Verified with a control
build: I reverted every behavioural change this plan makes to `run.rs` and
`string.rs` (SIGNAL indent reset, all three `settle_block_indent` calls, the `Entry`
rule, the `OverFor` keyword, the POS overrun), rebuilt, and re-ran. Both programs
and the recorded plain-`DO` program produce byte-identical output to HEAD on both
engines. Nothing in this plan caused them.

---

## Triage of the DEFERRED MINORS

| Ledger item | Verdict | Basis |
|---|---|---|
| Task 1: `run.rs` `method_invocation` match returns false on every arm | **carry** | Read at `run.rs:2600-2602`. Deliberate, documented at the site and in `Entry`'s own doc; the exhaustive no-wildcard match is what forces the Phase 5 revisit. The `else` arm is still reachable for a non-first `USE LOCAL`. |
| Task 2: case enumeration keys on `*.rex` | **carry** (M5) | One-directional gap: a `.rex`-less case silently does not run, it cannot produce a wrong answer. Cheap to fix later by unioning the three extensions. |
| Task 2: `run.rs` "boundary is unobservable" comment above a load-bearing call | **carry, and the claim is TRUE** | Measured: removing the boundary while keeping the line leaves the whole suite green (83 ok / 0 FAILED). The `settle_block_indent` call inside the closure is what is load-bearing, and mutation C reddens a case for it. Only the adjacency invites misreading. |
| Task 2: shapes the fix repaired that nothing pins | **carry** | Not a wrong answer; a coverage note. Two of them (`do while` test-true, `do i = 1 to raiser()` zero-pass) I re-measured against the oracle and the crate is byte-identical today, so nothing is silently broken. |
| Task 2: `trace i` pinned for instance three only | **carry** | Confirmed by reading the ten case programs: `signal_from_a_called_label_intermediates` is the only `trace i` case. Coverage note. |
| Task 2: `.wrong` provenance only in the gitignored report | **downgrade, effectively closed** | I regenerated all three `signal_*.wrong` byte for byte from mutation A and three of the four loop `.wrong` from B+C. Provenance is mechanically recoverable; only `call_on_at_an_until_test_that_runs_another_pass.wrong` is not, and that file is not compared to any run. |
| Task 2: `settle_block_indent` takes two same-typed usize indents (**flagged SUBSTANTIVE**) | **already fixed -- the ledger entry is stale** | `git show` across the range: the signature was `(entered: bool, do_indent: usize, loop_indent: usize)` from `691266bd9` to `a87682b4c`, and `932e6b710` (Task 5) changed it to `(open: bool, clause_indent: usize)`. At HEAD it takes one `usize`, the level is derived inside, and the function's own doc at `run.rs:6240-6245` explains that this was done precisely so a swapped call is unwritable. **Nothing to do; delete the item.** |
| Task 2: `git ls-files` reads the index; doc run-on; unused `&String`; "in 1 descriptors" | **split** | The `git ls-files` item lives only in an untracked report (`.superpowers/` is ignored) -- outside the merge, nothing to do. The other three are real and in the tree at `trace_indent.rs:269-270`, `:232`, `:244`; carry as M4, fix opportunistically. |
| Task 3: no tracked corpus `.rex` contains `DO OVER ... FOR` (**flagged SUBSTANTIVE**) | **carry, but tighten the prose** | Confirmed: I searched every tracked `.rex` in the repo; `DO OVER` appears in four corpus programs and none pairs it with `FOR`. The construct is nonetheless pinned three ways (mutation E reddens three tests), one of which -- the `loop-header-values` stanza -- I re-ran against the *live* oracle today and found identical. So the risk is "static expectations rather than a live differential", not "unpinned". The merge-blocking part of this item is not the coverage gap; it is that two comments overstate the gap (I3, I4). Add the corpus program when convenient; **correct the two sentences before merge.** |
| Task 4: `oracle-crashes.txt` header names the cardinality of its own entries | **carry** (M3) | The house rule carves out no same-file exception, but the rot risk really is low and the entry sits beside the entries it counts. |
| Task 5: pre-existing set-cardinality prose at `loop-header-boundaries:31` and `:39` | **carry, and there are two more** (M2) | `:17` and `:23` are the same class. `:39` has additionally drifted into an undercount. All predate this plan; fix them together with M1 in one pass. |
| Task 5: `loop-header-boundaries:1-2` subject line | **carry** (M1) | Confirmed. |
| Task 5: the `UNTIL` bullet's cause is unestablished and its program unnamed | **promote to Important** (I2) | Not because the bullet is wrong, but because the file simultaneously asserts and withdraws the attribution. I also supplied a reconstruction that reproduces the bullet's numbers, which changes what "closing it needs a measurement nobody has made" means. |

### Must fix before merge

I1, I2, I3, I4, I5 -- all five are false statements in tracked files, all
correctable in prose with no behavioural change. Nothing else blocks.

### Fine to carry

M1-M7 and every remaining deferred minor above. The `settle_block_indent` item
should be struck from the list rather than carried, because it no longer describes
the tree.

---

## Method notes

* Oracle runs: always `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../bin/rexx
  /abs/path )` from a freshly `mktemp -d`'d directory, with stdout, stderr and exit
  status captured to three separate files and never merged. Programs lived in a
  dedicated directory holding nothing else.
* The four forbidden programs were not run. `oracle-crashes.txt`'s entries were
  verified by reading the cited C++ only.
* `/bin/grep -a` throughout; no exit status read through a pipe; `cargo` run only
  from `rust/`.
* The oracle tree was not modified (`git -C /home/moritz/dev/repos/ooRexx status`
  untouched by this review; only `sed -n` reads were issued against it).
* Every mutation was applied to a byte copy taken before the first mutation and
  restored from that copy, with `git diff --quiet` checked after each restore and
  the binary rebuilt. Final state: tree clean at `613983c0b`, workspace suite
  exit 0 / 83 `ok` / 0 `FAILED`, corpus gate 53 of 53.

---

# Scoped re-review: 613983c0b..7a0b31245

Tree at `7a0b31245`, clean. `cargo build --release --bin rexx-run` reported nothing
to do, so the binary I measured with is current for this HEAD. I did not re-run
fmt/clippy/the suite (verified by the controller) and did not re-open anything
cleared in the first pass.

## Per finding

| Finding | Verdict |
|---|---|
| I1 `loop-header-boundaries:259-273` | **ADDRESSED WITH A NEW PROBLEM** (two of them, NEW-1 and NEW-2 below) |
| I2 the `UNTIL` bullet | **ADDRESSED** in the bullet; **ADDRESSED WITH A NEW PROBLEM** in the withdrawal paragraph it points at (NEW-3) |
| I3 `run.rs:11372-11380` | **ADDRESSED** |
| I4 `loop-header-values:184` | **ADDRESSED** |
| I5 `trace_indent.rs:33-36` | **ADDRESSED** |
| M3 `oracle-crashes.txt` cardinality | **ADDRESSED** (bonus); replacement re-verified true -- every entry carries an `Effect:` line naming its mode |
| M2 cardinality at `:17`, `:23`, `:31`, `:39` | **ADDRESSED** (bonus), with a residue at `:62` (NEW-4) and a side effect at `:39` (NEW-5) |
| M6 dangling `task-4bp-report.md` citation | **ADDRESSED** (bonus); no tracked file references it now |

## New problems this round introduced

**NEW-1 (Important) `loop-header-boundaries:272-273` -- the evidence pointer added by
the I1 rewrite points the wrong way, at a transcript that cannot show what it is
cited for.** The new sentence ends "-- which is what the transcript below shows,
`G ran 5` ahead of `body`." Grepped the whole file: the only two transcript lines
containing `body` are at `:247-248`, *above* this paragraph. The transcript that is
actually below (`:296-297`) is the empty-body variant, `oracle G ran 5 / after set
5`, which has no `body` line at all -- so a reader who follows the pointer lands on
a transcript that cannot corroborate the claim. The claim itself is true (re-measured
at HEAD: oracle `G ran 5 / body / after set 5`); only the direction is wrong. Fix is
one word.

**NEW-2 (Important) `loop-header-boundaries:271-272` -- "the oracle *always* delivers
at the `DO`'s own boundary" is falsified by the program this same round added to the
plan.** Re-measured at HEAD on the plan's own Step 1c program (empty `do`/`end`,
double requeue): oracle `G ran 5 / K ran 6 / after 5` -- the first delivery is at the
`DO`'s own boundary and the **second is at the `END`'s**. Same construct, same file's
subject. The intended contrast (this crate's delivery point varies with whether there
is a body; the oracle's does not) is real and worth keeping; the universal quantifier
is what is false. Scoping it to the first pending condition, or to "wherever the
pending condition is waiting when the `DO` clause is reached", costs nothing.

**NEW-3 (Important) `loop-header-boundaries:71-74` contradicts the plan's new Step 1c,
across the two commits of this same round.** The case file still says the
elided-instruction attribution -- "the oracle runs a clause boundary at a position
where this crate has no instruction to run one at" -- is "withdrawn rather than
replaced: **nothing anyone can run supports it**". Commit `00fc6d4fc` added Step 1c
to the plan, which names a program doing exactly that and calls it "**the elided-`END`
mechanism: a boundary that is genuinely absent, not misplaced**", and I re-ran it at
HEAD. So something anyone can run now supports the *mechanism*; what nothing supports
is the *bullet's own observation*. Commit `7a0b31245` edited this very paragraph (it
is where the `task-4bp-report.md` citation was removed) and left the sentence two
lines later standing -- the same "correction lands at the quoted sentence, the
neighbour survives" shape this task has produced every round. Narrow fix: say nothing
anyone can run reproduces *the bullet's observation*, and point at Step 1c for the
mechanism on a different construct.

**NEW-4 (Minor) `loop-header-boundaries:62` -- "only one of *the two* has an
established one" is the cardinality residue of the same pass.** `:39` lost its "TWO"
this round; the paragraph immediately below still counts the set. Still **true**
(there are two bullets and one has an established cause), so this is house-rule
residue rather than a false statement.

**NEW-5 (Minor) `loop-header-boundaries:39-42` -- removing the cardinality turned a
statement about two named items into a general rule the file's own row 3 does not
follow.** It now reads "LOOP-HEADER BOUNDARIES THAT DIVERGE FROM THE ORACLE ARE NOT
ROWS HERE, because a row's expected block is what this crate prints and these are
places where that is not what the oracle prints." Re-ran all 11 stanzas against the
live oracle at HEAD: row 3's expected block still is *not* what the oracle prints,
and `:31` in the same block says so ("A row that differs from the oracle says so in
its own comment"). The universal is defensible only if "boundary divergence" is read
as excluding row 3's traceback divergence, which the text does not say. Under the old
wording "these two" bounded it and the tension did not arise. Smallest fix: say the
two bullets are not rows *because their divergence is in the delivery, which a row's
expected block cannot express*, rather than giving a general rule about differing
from the oracle.

## On the fix shape (removal vs corrected value)

**Your reading is right for four of the five, and this round supplies the evidence
for it: every new defect landed in the one finding that was fixed by rewriting, and
none landed in the four fixed by deleting.** For I3, I4, I5 and the M2/M3
cardinalities the value really was incidental -- I4's uniqueness claim was falsified
by the very commit that wrote it, I5's count by the bullet list three lines under it,
and `:31`'s and `:39`'s counts are recomputable by scrolling. Deleting them removes a
thing that rots and costs a reader nothing they cannot see.

**One removal did cost something.** I3's replacement dropped the *true* half of the
old sentence along with the false halves: "`trace_oracle.rs` carries no witness for
this shape", which I verified in the first pass (no `DO ... OVER ... FOR` in any of
that harness's files) and which is the actual answer to "why is this transcript baked
into a unit test instead of living in the trace-oracle harness?". A reader editing
that test now has no answer to that question in the file. Not blocking; worth one
clause if anyone touches the comment.

## Minors not taken -- still safe to carry?

Yes, all of them; re-checked at HEAD rather than assumed.

* **M1** `loop-header-boundaries:1-2` subject line -- unchanged, still understates the
  file. Read-only staleness.
* **M4** `trace_indent.rs:232` unused `&String`, `:245` "in {n} descriptors" plural,
  `:269-270` doc-comment run-on -- all three still present at those lines after the
  module-doc edit. Cosmetic.
* **M5** `trace_indent.rs:117` case enumeration keys on `*.rex` -- unchanged.
  One-directional gap; a half-added case does not run rather than answering wrongly.
* **M7** `corpus_differential` asserts only under `REXX_CORPUS_GATE` -- pre-existing
  design, unchanged by this round.
* **NEW-4** and **NEW-5** above join this list if they are not taken.

## Merge verdict

**Blocking: NEW-1, NEW-2 and NEW-3.** All three are prose in tracked files, all three
are false or self-contradictory as written, and all three are one-or-two-sentence
edits with no behavioural change. NEW-3 is the one I would insist on, because it
contradicts the plan step the branch is handing to Task 6 and it is about the exact
mechanism Task 6 has just been told to decide between.

**Not blocking:** NEW-4, NEW-5, and every carried Minor. With NEW-1 to NEW-3 corrected
the branch is clean by my reading: the behavioural changes were verified against the
live oracle in the first pass and this round changed no behaviour, and the five
Importants are otherwise properly closed.
