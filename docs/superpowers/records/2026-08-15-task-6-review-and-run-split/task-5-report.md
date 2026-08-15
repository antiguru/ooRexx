# Task 5 report: the ANSI spec's premise, the `clause.rs` comment the drain falsified, and the shapes to record

**Status: complete.** One commit, `c77339dab`. No behaviour changed: every edit is a comment, a spec
document, a plan document or a record.

**Tree found: `9c465989a`**, not `56d9d1c86`. `git log 56d9d1c86..HEAD` was non-empty when this task
started, including Task 1's `INTERPRET` fix. Every transcript below was measured at
`9c465989a`, with `target/release/rexx-run` rebuilt from that tree before the first probe.

**Oracle-crash rule checked before building any probe.** `rust/CLAUDE.md` forbids running a clause
that queues the same `CALL ON` condition twice (`zr = ra() + ra()` with both raising the same name),
which runs the handler once and then segfaults the oracle at rc 139. Every probe below queues each
condition name at most once per clause: `c1` by the body clause, `c2` by `h1`, `c3` by `h2`, and the
handlers never re-raise the name that reached them. No probe ran from the scratchpad root; all ran
from `.../scratchpad/task5-probe`, created for this task, with absolute paths for the program and
for all three redirects. stdout, stderr and exit status were read as three separate descriptors
throughout, never `2>&1`.

---

## Step 1: every transcript, measured

### Item 1, the drain program

Program (`drain.rex`), reconstructed from the brief's first seven lines plus the handlers it
describes:

```rexx
 1  call on user c1 name h1
 2  call on user c2 name h2
 3  call on user c3 name h3
 4  do zi = 1 to 2
 5    zr = raiser()
 6  end
 7  say 'after' zr
 8  exit
 9  raiser: raise user c1 return 5
10  h1: say 'h1' sigl; raise user c2 return
11  h2: say 'h2' sigl; raise user c3 return
12  h3: say 'h3' sigl; return
```

| | rc | stdout | stderr |
|---|---|---|---|
| oracle | 0 | `h1 5` / `h2 6` / `h3 5` / `h1 5` / `h2 6` / `after 5` / `h3 7` | empty |
| `REXX_ENGINE=tree-walker` | 0 | identical | empty |
| `REXX_ENGINE=ir` | 0 | identical | empty |

All three agree byte for byte. This reproduces the brief's oracle line exactly, which is what says
the reconstruction is the program the brief measured. The third and fourth stdout lines are one
boundary running two handlers: on the loop's second pass the clause at line 5 delivers `h3`, left
over from the previous pass's `END`, and then `h1`, queued by that same clause, both at `SIGL 5`.
**This crate drains too**, on both engines.

### Item 3, the `ITERATE` shape

Program (`iterate.rex`):

```rexx
 1  call on user c1 name h1
 2  call on user c2 name h2
 3  call on user c3 name h3
 4  zn = 0
 5  do while zn < 2
 6    zn = zn + 1
 7    zr = raiser()
 8    iterate
 9  end
10  say 'after' zr
11  exit
12  raiser: raise user c1 return 5
13  h1: say 'h1' sigl; raise user c2 return
14  h2: say 'h2' sigl; raise user c3 return
15  h3: say 'h3' sigl; return
```

rc 0 and empty stderr on the oracle and on both engines. stdout:

```
oracle            tree-walker and ir (identical to each other)
h1 7              h1 7
h2 8              h2 8
h3 6              h3 8
h1 7              h1 7
h2 8              h2 8
after 5           h3 8
h3 10             after 5
```

`56d9d1c86`'s drain did **not** change this shape: the transcripts match the ones the brief quotes,
line for line, so there is no news to report here beyond confirming them at this tree.

### Item 4, the bare `OPTIONS` instruction

Program: `options 'nothing'` on line 1, `say 'ran'` on line 2.

| | rc | stdout | stderr |
|---|---|---|---|
| oracle | 0 | `ran` | empty |
| `REXX_ENGINE=tree-walker` | 120 | empty | `rexx-exec: OPTIONS is not implemented (Phase 5)` |
| `REXX_ENGINE=ir` | 120 | empty | `rexx-exec: OPTIONS is not implemented (Phase 5)` |

**My re-measurement agrees with the controller's transcript exactly.** Nothing to report as a
disagreement.

---

## The four items: what shape of correction each got, and why

### 1. The ANSI spec, `docs/superpowers/specs/2026-08-15-ansi-condition-delivery.md` — **deletion plus a marked replacement**

The entry was **removed from the disagreement list**, not restated. In its place the opening now says
plainly that on the question 8.2.4 answers, may one clause boundary run more than one handler,
**ANSI 8.2.4 and ooRexx agree**, and carries the program and the transcript above as the evidence.
The list's remaining entry (8.3.6/8.3.6.1, the `DO UNTIL` attribution) is untouched, and the sentence
that used to say "that second one is inference" now says "that one".

The block bounds what the measurement settles. `RAISE` and `USER` are ooRexx extensions, so the
program measures ooRexx and this crate rather than conformance, exactly as the document's own scope
limit says. What it settles is that neither interpreter has a one-per-boundary rule, which is the
premise the removed entry rested on. Whether ooRexx sweeps the four ANSI conditions in 8.2.4's fixed
`HALT FAILURE ERROR NOTREADY` order rather than arrival order is a separate, unmeasured question and
stays open as the delivery-order item in the adversarial list.

Adversarial item 1 carried the same false premise as a live suspicion ("the single most likely real
divergence"). It now states that its premise is measured false, with the date and the commit, and
names what remains unrun: the same shape built from ERROR and NOTREADY, which needs command issuance
and streams. The adversarial section's preamble said "I did not run any of them"; that sentence would
have contradicted the new item 1, so it now records that the first item's premise has been measured
since, by a later task, with a `RAISE`/`USER` program rather than an ANSI condition.

**Why not deletion of item 1 outright.** Deletion is this plan's measured-safest shape, but item 1 is
a checklist Phase 7 works from, and the ANSI-condition half of the check is genuinely unrun. Deleting
the row would have deleted the open half with the closed half.

### 2. `clause.rs`'s comment block — **rewrite of the false clause, bounded to what was measured**

**What survives.** Everything except the mechanism clause. The block's three exemptions to
`enter_clause`'s tripwire are unchanged in substance: a `DO`/`LOOP`'s control setup and first header
test, an `INTERPRET` fragment's clause-line override, and a trap queued by a handler during a
delivery. For the third, the *conclusion* still holds -- the wait is the design rather than a missing
call, and the oracle waits too -- and I established that from `deliver_pending_traps` (`run.rs`),
which snapshots `owed = self.pending_traps.len()` before the loop, so anything a handler queues lands
beyond the prefix that boundary is answering for. The false part was the *reason*: the boundary does
not "deliver at most one and does not re-check", it drains, bounded to the entries queued when it
began. That is also what `Interp::pending_traps`' own doc comment already said, and the corrected
comment points there for the measurement rather than restating it.

A second falsehood in the same sentence: "this function delivers at most one" appears inside
`enter_clause`, which delivers nothing at all. Delivery is `leave_clause`'s, through
`deliver_pending_traps`. The rewritten sentence names the boundary rather than a function.

**The whole comment block was checked**, as the brief asked, and so was the neighbourhood. Grepping
for the sentence found it restated at four more sites, all falsified by the same commit:

* `crates/rexx-exec/src/lib.rs`, `PendingTrap::queued_during_delivery`'s doc.
* `crates/rexx-exec/src/run.rs`, `end_promoted_branch`'s doc.
* `crates/rexx-exec/tests/ir_dual.rs`, the `BRANCH_CASES` entry for a handler that queues again at a
  matched `WHEN`'s branch end.
* `crates/rexx-exec/tests/ir_dual_cases/assignment-and-say`, the boundary case's `#` comment.

All four keep their conclusion and get the same corrected reason. None of the four measured
transcripts they carry changed.

### 3. The `ITERATE` shape — **recorded as a new KNOWN GAPS row**

Recorded in `docs/superpowers/plans/phase-4-exclusions.txt`, section **KNOWN GAPS -- neither excluded
nor deviated, just not closed yet**. **Why there:** that section's own preamble is explicit that
adding a row needs no plan amendment ("recording a divergence you have just measured should never
require permission, or the incentive runs the wrong way and gaps go unrecorded"), while removing one
does. It is the section for a real divergence with no owner assigned, which is exactly this. The
alternative shape this project also uses, a standalone open plan file like
`2026-08-13-over-for-keyword.md`, is for a divergence somebody intends to fix; this one is explicitly
not to be fixed and has nobody assigned, so a row is the right size.

The row carries the numbered program, all three descriptors on the oracle and both engines, and the
mechanism difference. **It attributes rather than restates**: the transcripts are mine and are dated
to `9c465989a` and to no other commit; that the shape is byte-identical before and after `e74780054`
and unchanged by `56d9d1c86` is credited to
`docs/superpowers/plans/2026-08-15-task-6-review-and-run-split.md` as that document's measurement, not
mine. I did not run those commits.

### 4. The bare `OPTIONS` over-refusal — **recorded as a second KNOWN GAPS row, plus a plan amendment**

Same section, same reason, with one difference stated in the row itself: this row **has** an owner,
Phase 5, in `instruction_owner` (`crates/rexx-exec/src/lib.rs`), so it says so rather than reading as
an unassigned divergence. It is in KNOWN GAPS rather than EXCLUSIONS because the EXCLUSIONS section
covers the `::OPTIONS` **directive** only, and the reason recorded there is the directive's own: its
whole effect is to change package settings, so there is no unused `::OPTIONS`. That reason does not
carry to the instruction.

The row also records **why the oracle runs the program**, from the C++ rather than by inference:
`RexxInstructionOptions::execute` (`interpreter/instructions/OptionsInstruction.cpp`) traces the
instruction and evaluates its expression as a string, so an error inside the expression still raises;
the word scan that would act on the result sits inside `#ifdef _DEBUG` under a comment saying
processing is currently disabled. So in the build this project measures against, an `OPTIONS`
instruction whose expression evaluates has no effect beyond that tracing and the interactive pause,
and refusing the program is an over-refusal in the sense the directives section defines. **`OPTIONS`
was not implemented.**

Separately, the fourth item and Moritz's decision were **written into the plan**
(`docs/superpowers/plans/2026-08-15-task-6-review-and-run-split.md`, Task 5, new subsection 4), not
left in the dispatch. `rust/CLAUDE.md`'s own rule is that a correction written into a dispatch is read
once and lost, because briefs regenerate from the plan.

### Two more sites the same false premise had reached, corrected in the same commit

* **`docs/superpowers/plans/2026-07-27-rust-rewrite.md`, the roadmap's Phase 7 entry — rewritten.**
  It read "8.2.4 drains a boundary; this crate delivers at most one ... `Interp::leave_clause` takes
  one and does not re-check, which matches ooRexx on the extension path", and set as Phase 7's work
  the very measurement this task took. This is the document the ANSI spec's own opening says
  summarises its consequences, so it is the single most load-bearing copy of the false premise, and
  the brief's stated reason for the whole item ("Phase 7 would design against a false premise") is
  about this bullet as much as the spec. It now says the measurement was taken, points at the spec
  for the program and the transcripts, and names the *order* as what Phase 7 still owes. Its closing
  sentence "It also records the two places ooRexx departs from the standard" became false when the
  list lost an entry; the count is deleted rather than decremented.
* **`docs/superpowers/plans/2026-08-09-phase-4e-ir.md` — marked correction beside the original.**
  A completed plan, so the original paragraph stays and a dated blockquote sits under it, matching
  the shape `9c465989a` used one commit earlier on `2026-08-08-phase-4e-ir-design.md`. The correction
  states the endpoint I checked myself: the sentence was accurate at `7a7f58491` (2026-08-11), the
  commit that added the paragraph, where `Interp` holds `pending_trap: Option<PendingTrap>` -- read
  at that commit with `git show`, and it claims nothing about the commits between.

---

## Files changed

```
docs/superpowers/plans/2026-07-27-rust-rewrite.md                     |  4 +-
docs/superpowers/plans/2026-08-09-phase-4e-ir.md                      |  2 +
docs/superpowers/plans/2026-08-15-task-6-review-and-run-split.md      | 31 ++++--
docs/superpowers/plans/phase-4-exclusions.txt                         | 87 +++++++++++++++
docs/superpowers/specs/2026-08-15-ansi-condition-delivery.md          | 56 ++++++++--
rust/crates/rexx-exec/src/clause.rs                                   | 13 ++-
rust/crates/rexx-exec/src/lib.rs                                      |  7 +-
rust/crates/rexx-exec/src/run.rs                                      |  7 +-
rust/crates/rexx-exec/tests/ir_dual.rs                                |  9 +-
rust/crates/rexx-exec/tests/ir_dual_cases/assignment-and-say          |  7 +-
```

Staged by exact path. `docs/superpowers/specs/2026-08-15-phase-5-inherited-surface.md` is untracked in
this tree and is **not** mine; it was left alone and not staged. It happens to carry an `options 'x'`
row of its own, which I noticed while grepping and did not touch.

## Gates

Each run unpiped from `rust/`, exit status read on its own.

| gate | status |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `memcap 8G cargo test --release --workspace` | 0, `test result: ok` on every binary |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --release --workspace` | 0, `test corpus_differential ... ok` |

All four were run again after the self-review edits, and the table is the second run.

## Self-review of my own diff

Two passes over `git diff`. **Nine findings, all fixed before the commit.**

Pass 1, six findings:

1. `lib.rs`: my replacement opened "the boundary that would have taken it drains only ...", which
   reads as if that boundary never ran. It did run; it just did not owe this trap. Changed to "the
   boundary running that delivery".
2. `run.rs`: my edit left two consecutive sentences both opening on "so"/"So". Reflowed.
3. Roadmap: I had written that running the order check needs "command issuance and streams", but
   `HALT` is one of the four ANSI conditions and needs neither. Replaced with a pointer to the
   paragraph above, which already says why all four are behind Phase 7.
4. **The one that would have shipped a false claim.** The `ITERATE` row said "Neither interpreter
   delivers that condition at the boundary that queued it", asserting it of *both*. For the oracle
   that is witnessed by `SIGL 6` against a queuing boundary at line 8. For this crate `SIGL 8` is
   consistent with delivery at the same boundary *and* with delivery at a second boundary carrying
   the same line, and the three descriptors cannot separate them. I had inferred the crate's half
   from `deliver_pending_traps` rather than measured it. The row now claims only the `SIGL` values
   and the ordering, and says so.
5. `OPTIONS` row: "has no observable effect" was false under `trace`, since `execute` calls
   `traceInstruction` and `pauseInstruction`. Bounded to "no effect beyond the tracing and the
   interactive pause that `execute` performs for it".
6. I removed the plan's "Three things are left" but then wrote "two shapes to record" in the same
   heading, putting a count back one line away from where I had just taken one out. Heading is now
   "the shapes to record".

Pass 2, three findings:

7. The spec's adversarial preamble ("I did not run any of them") contradicted my new item 1, which
   carries a measurement. Preamble amended, and it says the measurement used a `RAISE`/`USER`
   program rather than an ANSI condition, so it does not read as conformance evidence.
8. The spec said the order question "stays open as item 2 of the adversarial list", a positional
   reference that rots the moment the list is edited. Changed to name the item by its content.
9. The `ITERATE` row's ALL-CAPS heading asserted "AN ITERATE'S BOUNDARY TAKES A REQUEUED HANDLER",
   the same unwitnessed claim finding 4 removed from the body. Heading now leads with what was
   measured, the `SIGL`.

Finding 4 is the reason to keep doing this: it is a claim I would have defended from the source, and
the probe I ran cannot distinguish it from its opposite. It is the same failure mode
`rust/CLAUDE.md` records as "when tempted to explain why something cannot happen, run it instead",
arriving as an explanation of something that *did* happen.

## Concerns

* **`clippy` finished in about 2 seconds against a warm `target/`.** It did print `Checking
  rexx-exec`, so it re-examined the crate my edits are in, and the edits are comments only. But
  `rust/CLAUDE.md` records that a same-session green clippy is provisional and that a clean target
  directory is what makes it evidence. I did not run from a clean target; this is not a phase
  boundary, and the reviewer should know the green is the provisional kind.
* **Where this crate's `ITERATE` delivery actually sits is unmeasured**, per self-review finding 4.
  If the reviewer wants it pinned, the instrument is `trace i` or `trace r` on the same program,
  which shows the clause echoes around the handler and would separate a second boundary at line 8
  from the first. I did not run it: the row does not need it, and the plan's rule is to record and
  move on.
* **The `RAISE`/`USER` limit on the ANSI claim.** The removed disagreement is removed on the strength
  of a program the standard rejects before it runs. That is enough to falsify the entry's premise
  about ooRexx, which is what the entry rested on, but it is not conformance evidence, and both the
  spec's opening and its adversarial item 1 say so. A reviewer who disagrees with that reading should
  look there first.
* **The exclusions file's directives section still reads "THE TWO DELIBERATE OVER-REFUSALS".** It is
  scoped to directives, so the new instruction row does not falsify it, and I left it alone rather
  than widen this task's edit surface. It is a count of an in-repo aggregate and will need attention
  whenever a third directive over-refusal appears.

## Found and not fixed

Nothing beyond the two rows recorded above. No divergence was discovered during this task that is
not written into `phase-4-exclusions.txt`.

---

# Fix round 1

**Commit `dd00b2d3d2fb4a5ecb362bb8e26bdee0c4a24a6b`.** Both Important findings addressed, all four
Minors fixed, M6 left alone as instructed.

## I1. The grep table

Command run, from the repository root:

```
/bin/grep -rn -a -i -E "at most one|does not re-check|does not recheck|one-per-boundary|one per boundary|without re-checking|delivers one condition" . --exclude-dir=.git --exclude-dir=target
```

`/bin/grep` rather than `grep`, and `-a`, because the wrapper on `PATH` passes `-I` and skips binary
files silently. I widened the coordinator's pattern with four more spellings. **What the widening
actually bought, checked rather than assumed:** the unhyphenated `one per boundary` added live-tree
hits at `2026-07-27-rust-rewrite.md:2459` and `ansi-condition-delivery.md:168`, both verdicted true;
`without re-checking` added only `.superpowers` hits; `does not recheck` and `delivers one condition`
matched nothing anywhere. **No widened spelling found a defect** -- the missed fifth site contains
"at most one" and the coordinator's own pattern reaches it. **Exclusions:** `.git` is object storage
and `target/` is derived from the sources scanned.

Every hit outside `.superpowers/sdd/`, with a disposition:

| file:line | text | verdict |
|---|---|---|
| `rust/crates/rexx-exec/tests/ir_dual_cases/loop-header-boundaries:19` | "a clause boundary delivers at most one without re-checking" | **FALSE. Fixed this round.** The fifth site, and the one whose whole subject is where a loop header's boundaries deliver |
| `rust/crates/rexx-exec/src/clause.rs` | corrected in `c77339dab` | true |
| `rust/crates/rexx-exec/src/lib.rs` | corrected in `c77339dab` | true |
| `rust/crates/rexx-exec/src/run.rs:5518` | corrected in `c77339dab` | true |
| `rust/crates/rexx-exec/tests/ir_dual.rs:652` | corrected in `c77339dab` | true |
| `rust/crates/rexx-exec/tests/ir_dual_cases/assignment-and-say:290` | corrected in `c77339dab` | true |
| `rust/crates/rexx-exec/src/run.rs:4620` | "before `ExprKind::Call` at most one activation could be entered per clause" | unrelated. Activations, not conditions |
| `rust/crates/rexx-exec/src/run.rs:13077` | same claim, doc form | unrelated |
| `rust/crates/rexx-exec/src/builtin/convert.rs:605` | "digits around at most one point" | unrelated. Number syntax |
| `rust/crates/rexx-exec/src/builtin/string.rs:236` | "At most one byte past the haystack" | unrelated. `POS` window overrun |
| `rust/crates/rexx-exec/src/builtin/datetime.rs:394` | "at most one leap-aware year's own day count" | unrelated |
| `rust/crates/rexx-exec/src/invocation.rs:15` | "a command line can supply at most ONE argument string" | unrelated |
| `rust/corpus/phase-4c.txt:216` | "every earlier program leaves at most one pending at a time" | **true**, and about the corpus rather than the rule: it is why the suite could not see the single-slot defect. Kept |
| `docs/.../phase-4-exclusions.txt:660` | "at most one byte past the end of the haystack" | unrelated. `POS`, DEVIATION 3 |
| `docs/.../2026-08-15-task-6-review-and-run-split.md:358, :361` | the plan quoting the false comment as the thing to fix | correct as a quotation. Kept |
| `docs/.../2026-07-27-rust-rewrite.md:2459` | my corrected bullet, which names what it used to say | true |
| `docs/.../2026-08-09-phase-4e-ir.md:961` | the original paragraph under a marked correction | kept by design, that is what a marked correction is |
| `docs/.../2026-08-09-phase-4e-ir.md:963` | my correction blockquote | true |
| `docs/.../2026-08-15-ansi-condition-delivery.md:44, :168, :175, :310-314` | statements about the standard, and my corrected item 1 | true |
| `build/bin/yaml.cls:2076`, `extensions/yaml/yaml.cls:2076` | "at most one digit (1-9), at most one chomp indicator" | unrelated, and the C++ tree is read-only |

**`.superpowers/sdd/` hits are out of scope and I did not edit any**: they are dated briefs, reports,
review texts, `review-*.diff` captures and progress ledgers. A brief that quoted the comment as it
stood is a correct record of what the brief said. Editing them would falsify the record rather than
correct it.

**One `.superpowers` hit I am flagging rather than fixing.**
`.superpowers/sdd/2026-08-15-task-6-review-and-run-split/progress.md:512` records my own claim as
"Four comments carried ... -- clause.rs, lib.rs, run.rs, ir_dual.rs and a case file". Two problems:
the count says four and the list that follows it names five, and the enumeration is short by
`loop-header-boundaries` regardless. It is the controller's live ledger and another agent may be
writing to it, so I did not touch it. **The controller should correct it**, since the ledger is what
the project greps.

**What the miss says.** My enumeration was built from a grep I ran with the shell's default `grep`
and a narrower pattern, then written down as a list. The list was the artifact, not the command, and
a list cannot be re-run. The row that got missed is the one whose subject is closest to the claim,
which is the shape this project has already recorded: an enumeration copied out of a search is
unverified the moment the search is not re-runnable.

## I2. The `ITERATE` row's mechanism, measured rather than inferred

I ran `trace i` myself rather than taking the coordinator's excerpt, on the same program with
`trace i` as a new first line, which moves every line number up by one and makes `h2:` line 18,
matching the coordinator's numbering. Oracle wrapper as standard, fresh directory, stderr read
separately from stdout. **rc 0 everywhere; both engines byte-identical to each other on stdout and
on stderr.**

Clause echoes around the first `ITERATE`:

```
oracle                        this crate, both engines
 9 *-*   iterate               9 *-*   iterate
 6 *-* do while zn < 2        18 *-*     h2:
18 *-*     h2:                 6 *-* do while zn < 2
```

The oracle's re-test emits its value traces before `h2:`; this crate's after. Traced stdout:

```
oracle    h1 8 / h2 9 / h3 7 / h1 8 / h2 9 / after 5 / h3 11
engines   h1 8 / h2 9 / h3 9 / h1 8 / h2 9 / h3 9 / after 5
```

**The coordinator's finding is confirmed and the row was inverted.** It said the oracle carries the
condition past the re-test; the oracle is the one that carries nothing past it, because the re-test
has already run by the time `h2` executes. **This crate is the one that carries something past the
re-test**, queueing the third condition before it and delivering after it.

**A second thing the trace settles, which neither the row nor the coordinator's excerpt had.** `h2`
reports **SIGL 9 on the oracle and on both engines** -- the `ITERATE`'s own line, on both, despite
the delivery sitting on opposite sides of the re-test. That is why the one-boundary shift is
invisible on `h2` and visible only on `h3`. The row now says this.

`:1996-1997` and `:2028-2050` rewritten. The heading is now observational, "A HANDLER REQUEUED AT AN
ITERATE IS DELIVERED BEFORE THE LOOP'S RE-TEST HERE AND AFTER IT ON THE ORACLE", and a new
"WHAT REMAINS UNOBSERVABLE" paragraph says that which clause *owns* either delivery is settled by
neither `trace i` nor `SIGL`, for **both** interpreters rather than only this crate. The measured
content -- the `SIGL` values, the ordering, `rc`, stderr -- is unchanged, as the coordinator said it
should be.

## Minors

* **M3 -- fixed, and the proposed fix does not work either.** The coordinator suggested claiming the
  drain half from my re-measurement matching "the pre-drain transcripts". Those transcripts are not
  pre-drain: the plan's own Context says every transcript it quotes "was measured at `56d9d1c86`",
  which *is* the drain commit. So matching them establishes 56d9d1c86 to 9c465989a and nothing
  earlier. The row now claims exactly that as its own and credits the `e74780054` half to the plan,
  naming `e74780054` as the commit before the drain so a reader can see what the chain does and does
  not cover. **No commit older than `9c465989a` was run by me.**
* **M4 -- fixed.** The roadmap bullet now says running the order check needs two of the four pending
  at one boundary, so at least one of `ERROR`, `FAILURE` or `NOTREADY` must be raisable, and that
  `HALT` alone would not put the test in Phase 7.
* **M5 -- fixed.** `run.rs:3435` `Interp::pending_trap` to `Interp::pending_traps`.
* **M7 -- fixed.** `run.rs:5516-5517` and `lib.rs:1479-1480` reflowed.
* **M6 -- left alone**, as instructed.

## Gates, second run

| gate | status |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings`, **`CARGO_TARGET_DIR` a fresh empty directory** | 0 |
| `memcap 8G cargo test --release --workspace` | 0 |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --release --workspace` | 0, `test corpus_differential ... ok` |

**The clippy provisional note is withdrawn.** It ran with `CARGO_TARGET_DIR` pointed at a directory I
`rm -rf`'d first, so nothing was cached: the log shows dependencies compiling from `proc-macro2`
onward and every workspace crate `Checking`, `rexx-exec` included. A fresh target directory rather
than deleting the shared `target/`, because another agent is working in this tree. Note for the
record that `cargo fmt` initially exited 1 for me with "could not find `Cargo.toml`" -- an agent
thread resets its working directory between calls, so every gate below was run with an explicit
`cd` into `rust/` in the same command.

## Self-review of the fix diff

Two passes. **Three findings, all fixed before the commit**, and the first is the same class as
round 1's finding 4:

1. **My own replacement text said "NOTHING IS CARRIED PAST THE RE-TEST in either", which is false
   for this crate.** The crate queues the third condition before the re-test and delivers it after,
   so it is precisely the one that carries something past. I wrote the sentence while correcting an
   inverted mechanism and inverted it again in the other direction. The row now names the direction
   explicitly and says which interpreter does which.
2. The rewrite asserted "the ITERATE clause's boundary" and "the ITERATE-clause delivery" as
   ownership facts, which is the claim the "WHAT REMAINS UNOBSERVABLE" paragraph two lines below
   says cannot be witnessed. Reworded to place *deliveries* rather than to attribute *boundaries*.
3. "the value traces the re-test emits" replaced an enumeration of trace prefixes (`>V>`, `>L>`,
   `>O>`, `>K>`) that read as exhaustive and was not asserted anywhere.

A fourth, found while re-reading this report before sending: the grep-table paragraph claimed my
widened pattern caught sites the coordinator's would have missed, "including one of the corrected
sites". I had not checked it. Re-run per spelling, no widened spelling found a defect and the missed
fifth site is reachable by the coordinator's own pattern. The paragraph now says which spelling
bought what.

**Running total for this task: 13 self-review findings.**

## Concerns after this round

* **`progress.md:512`'s count and list**, above. Controller's file, controller's fix.
* The `ITERATE` row is now the longest KNOWN GAPS row for a divergence nobody is assigned to. It grew
  because two rounds of correction each added a bound. If a reviewer thinks the bounds outweigh the
  finding, the "WHAT REMAINS UNOBSERVABLE" paragraph is the one to cut, not the measurements.

---

# Fix round 2

**Commit `37af882ac6e653a7f55fbb03ad9dda6ec32edb51`.** N1, M8 and M9 addressed; the cold-clippy log
excerpt is quoted below.

## N1. The trace excerpt's line numbers

**Confirmed, and the re-reviewer's numbers are right.** I re-ran the program the row *prints* -- the
one with one-line handlers -- with `trace i` as a new first line, at `9c465989a`, oracle wrapper as
standard, fresh directory, three descriptors:

```
oracle clause echoes, first ITERATE onward     this crate, both engines
 9 *-*   iterate                                9 *-*   iterate
 6 *-* do while zn < 2                         15 *-*     h2:
15 *-*     h2:                                  6 *-* do while zn < 2
 7 *-*   zn = zn + 1                           16 *-*     h3:
16 *-*     h3:                                  7 *-*   zn = zn + 1
```

`h2:` at **15**, `h3:` at **16**. My block said 18 and 21, which are the three-line-handler variant's,
and I had labelled them as the printed program's. Traced stdout is `h1 8 / h2 9 / h3 7 / h1 8 /
h2 9 / after 5 / h3 11` on the oracle and `h1 8 / h2 9 / h3 9 / h1 8 / h2 9 / h3 9 / after 5` on both
engines -- identical to the variant's, which is exactly why no stdout check could have caught it.

**I took option B: renumber against the program the row prints**, rather than reprinting a second
program. One program in the row is less that can be wrong than two, and the row's untraced transcript
already uses that one.

**The numbers are quoted from the run, not derived.** The sentence that caused this said "which moves
every line number up by one" and invited the reader (and me) to do arithmetic. It is gone. In its
place the row states the traced line of every clause it goes on to cite -- `DO WHILE` 6, `ITERATE` 9,
`h2:` 15, `h3:` 16, `SAY` 11 -- and says the echoes are quoted rather than derived.

Two improvements to the same block while I was in it: it now runs from the first `ITERATE` through
the following pass's first body clause, so the `16 *-* h3:` citation in the prose is anchored in the
block a reader can see rather than in a number with no visible source; and it says **read each column
down its own side, the rows are not pairs**, because a two-column block whose whole subject is
interleaving invites being read as aligned pairs and it is not.

I also re-verified on this program, not the variant, that the re-test's own value traces sit between
`6 *-* do while zn < 2` and `15 *-* h2:` on the oracle, and that `SIGL => "9"` is what h2 reads there.

**What the miss says.** Round 1 fixed an inferred mechanism by measuring, then labelled the
measurement with numbers from a *different* program because the two shared a stdout. The measurement
was right and its provenance was not. A transcript is only evidence for the program it came from, and
the check that would have caught it is cheap: run the program you are printing.

## M8. The heading

Fixed. "A HANDLER REQUEUED AT AN ITERATE" reads most naturally as `h3`, and `h3` is delivered after
the re-test on **both** interpreters, so under that reading the heading was false. The claim is about
`h2`, which is requeued at the `zr = raiser()` clause and *delivered* at the `ITERATE`. Now: **"A
REQUEUED HANDLER DELIVERED AT AN ITERATE RUNS BEFORE THE LOOP'S RE-TEST HERE AND AFTER IT ON THE
ORACLE."**

## M9. `loop-header-boundaries`' SIGL sentence

Bounded rather than deleted, because in that file's rows it is the diagnostic that makes a row mean
anything. It now reads that the second delivery's SIGL says which boundary took it **whenever the
candidate boundaries sit on different lines, which is what a row there has to arrange**, and that
where two share a line, SIGL names the line and not the boundary. That is the same fact the
`phase-4-exclusions.txt` row states, scoped rather than contradicted.

## Cold clippy, quoted

`CARGO_TARGET_DIR` pointed at a directory `rm -rf`'d immediately before the run, so nothing was
cached:

```
$ rm -rf "$S/clean-target2"
$ CARGO_TARGET_DIR="$S/clean-target2" memcap 8G cargo clippy --workspace --all-targets -- -D warnings
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.47
   ...
    Checking rexx-extract v0.1.0 (/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-extract)
    Checking rexx-oracle v0.1.0 (/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-oracle)
    Checking rexx-bench v0.1.0 (/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-bench)
   Compiling rexx-inventory v0.1.0 (/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-inventory)
    Checking rexx-num v0.1.0 (/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-num)
    Checking rexx-parse v0.1.0 (/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-parse)
    Checking rexx-core v0.1.0 (/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core)
    Checking rexx-exec v0.1.0 (/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.63s
$ echo $?
0
```

Dependencies compile from `proc-macro2` onward, every workspace crate is `Checking`/`Compiling` with
`rexx-exec` among them, and `grep -c "^warning\|^error"` over the whole log is **0**. A fresh target
directory rather than deleting the shared `target/`, because another agent is working in this tree.

## Gates, third run

| gate | status |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings`, cold target | 0, log above |
| `memcap 8G cargo test --release --workspace` | 0 |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --release --workspace` | 0, `test corpus_differential ... ok` |

## Self-review of the round-2 diff

One pass, **two findings, both fixed before the commit**:

1. The two-column trace block reads as aligned pairs, which would be a new false reading of the very
   thing the block exists to show -- the columns diverge, so row three of one side is not row three
   of the other. Added the explicit "read each column down its own side".
2. My first wording of the M9 bound said the file's rows *all* have their candidate boundaries on
   distinct lines. That is a universal over an in-repo enumeration I had not checked, which is the
   shape `rust/CLAUDE.md` forbids. Rewritten as a conditional -- "whenever the candidate boundaries
   sit on different lines" -- which is true without enumerating anything.

**Running total for this task: 15 self-review findings.**

## Concerns after this round

* None new. The `ITERATE` row has now been corrected twice and re-measured three times; the standing
  caution is that its prose is long relative to a divergence nobody is assigned to, and the
  "WHAT REMAINS UNOBSERVABLE" paragraph is the one to cut if a reviewer wants it shorter.
