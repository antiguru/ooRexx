# Phase 5c Task 1 — the construction route, and surface B

**BASE:** `f3b637fde`. **Committed at `70bb5ea9b`.** Everything below was measured on this machine on 2026-09-02 against the
pinned 5.3 oracle at `/home/moritz/dev/repos/ooRexx/build`.

---

## The one-paragraph version

The route is a **seventh column in `class-set.txt`**, `construction`, holding the Rexx expression a
method row's instance arm binds `o` to. It is not a second fact beside the status: the extractor's
`Coverage::Covered(String)` *carries* it, so `covered` cannot be derived without one, and
`gate_table_c.rs` parses the status and the column back into one `Construction` value that
`method_probe_text` emits the probe from. A `covered` group's oracle shape is tightened from
`AllOrNothing` to `Exactly`, so a class flipped to `covered` over a route that does not construct
is a **structural failure**, red in every mode. Four of surface B's six non-`file` classes moved:
table C's 5c count is **868 → 815**, all 53 rows `agree`. **`Alarm` and `Ticker` did not move**;
they are blocked on interpreter features this task does not own, measured, and left `not-covered`.

---

## What surface B actually is

The plan's surface table sums to 97 over 96 probes. Re-swept, all 96 programs under
`corpus/gate-tables/methods/` run on the crate (`REXX_ENGINE=ir`, release, from a fresh empty
directory):

| rc | probes |
|---|---|
| 0 | 58 |
| 120 | 30 |
| **216** | **7** |
| 159 | 1 (`rexxinfo__instance`) |

Surface B is **seven**, not eight, and `file__instance` is one of the seven rather than an eighth
beside them: `alarm` `caselesscolumncomparator` `columncomparator` `file` `invertingcomparator`
`ticker` `timespan`. `file` is 5d's, so my subject was six. The plan has been corrected in place.

---

## What was built

### 1. `crates/rexx-extract/src/docs/classes.rs`

* **`Coverage`** — `Covered(String) | NotCovered | Unreachable`. `ClassRow.status: Status` becomes
  `ClassRow.coverage: Coverage`; `Status` survives for `class-methods.txt`'s own column and for
  rendering. **There is no way to construct `Coverage::Covered` without naming the expression.**
* **`CONSTRUCTION_PROGRAMS`** — the hand-committed programs, class name to Rexx expression. Four
  entries today. `CONSTRUCTION` is unchanged: it stays the measured record of what a *bare* `~new`
  does on the oracle.
* **`coverage_of`** (was `status_of`) — a committed program is what makes a class `covered` where a
  bare `~new` raises. Two assertions guard it: a program may not be committed for a class whose
  sentence makes it `Status::Unreachable` (D73's guard), and a program's class must have a bare
  `~new` that raises, so a program is never a route to something the bare `~new` already reaches.
* **`class_rows`** — asserts every `CONSTRUCTION_PROGRAMS` key is a name the class set carries.
* **`NO_PROGRAM`** (`"-"`) — the sentinel the column holds for a row that is not `covered`. Spelled
  once, and `gate_table_c.rs` imports it rather than repeating the literal.

### 2. `corpus/docs/class-set.txt`

Gains the `construction` field, appended so `status` stays at index 4 for the two readers that
address it positionally. Every `covered` row carries an expression and every other row carries `-`.
A class whose bare `~new` constructs carries that bare `~new` (`.Array~new`); the four opted in
carry the book's own syntax. The header states the invariant.

### 3. `crates/rexx-exec/tests/gate_table_c.rs`

* **`enum Construction { Constructs(String), Raises }`**, parsed by `construction_of` from the row's
  `status` and `construction` together. `covered` with `-`, or a non-`covered` row with a program,
  is a **panic** naming the class.
* **`method_probe_text`** takes the `Construction` and can only emit a construction program out of
  `Constructs`. `Raises` still emits `o = .{class}~new` and still writes the row set's own `status`
  and `reason` into the header, which is what the row's evidence is.
* **The oracle shape for a `covered` instance group is `OracleShape::Exactly(rows.len())`**, where
  every instance group used to be `AllOrNothing`. This is the half that runs. `AllOrNothing` admits
  a group that answered nothing, which is exactly how a wrongly-`covered` class would have gone
  green on two sides raising alike.
* `check_interpolated_text` now also polices the construction expression for `*/`, a newline and
  untrimmed whitespace — it is written into a Rexx block comment **and** into the probe's `o = `
  line.
* `MethodGroup` replaces the five-tuple the grouping used, since the group now carries six things.

### 4. `crates/rexx-extract/tests/extract_docs.rs`

`every_committed_construction_program_has_an_instance_arm_to_run_it` — a committed program for a
class with no instance-arm method row would make it `covered` with nothing on either side ever
constructing. See "what would still let someone flip a class" below; this closes the hand-written
half of that hole.

### 5. The probe programs

40 of the 96 change: the 4 newly `covered` classes get a new opening line and header, and the 36
already-`covered` instance probes get a header that names the expression they construct with
(`asked of the instance \`.Array~new\` answers`) in place of `asked of a bare ~new instance`. Their
code is byte-identical to before. They were regenerated by a scratch script and then verified by
`check_probe_text`, which re-derives every one of the 96 from the row set and compares in both
directions — so a script that had drifted from `method_probe_text` would have reddened rather than
shipped.

---

## The measurements

### The four that moved

Each construction expression is the book's own constructor syntax
(`utilityclasses.xml:768`, `:858`, `:1099`, `mthTimeSpanInit`), and each full probe is **rc 0 and
byte-identical to the oracle on stdout, stderr and exit status, on both `REXX_ENGINE=ir` and
`REXX_ENGINE=tree-walker`**, run from a fresh empty directory:

| class | program | rows | oracle | ir | tree-walker |
|---|---|---|---|---|---|
| `TimeSpan` | `.TimeSpan~new(1)` | 47 | rc 0, 47 lines | identical | identical |
| `ColumnComparator` | `.ColumnComparator~new(3, 100)` | 2 | rc 0, 2 lines | identical | identical |
| `CaselessColumnComparator` | `.CaselessColumnComparator~new(3, 100)` | 2 | rc 0, 2 lines | identical | identical |
| `InvertingComparator` | `.InvertingComparator~new(.Comparator~new)` | 2 | rc 0, 2 lines | identical | identical |

`InvertingComparator`'s argument is not the book's — the book's example passes a user-defined
`LengthComparator`. `.Comparator~new` is the shortest thing that is documented, is `covered` in this
same row set, and satisfies the argument's stated type.

### The two that did not

Each was run in several argument shapes -- the ones the book names for `atime` and
`interval` -- from a fresh directory, on the oracle and on both engines.

**`Alarm`.** `CoreClasses.orx:1472`'s `init` reaches `DateTime`/`TimeSpan` arithmetic on **every**
path — `atime - current` for a `DateTime`, `current + atime` for a `TimeSpan`,
`current + .timespan~fromSeconds(atime)` for a numeric string. This crate answers, at rc 120:

```
rexx-exec: the operator `+` applied to an instance of a user class is not implemented (Phase 5)
```

for `.Alarm~new(86400, …)`, `.Alarm~new(.TimeSpan~new(86400000000), …)` and
`.Alarm~new(.DateTime~new + …, …)` alike. The oracle answers `instance Alarm` at rc 0 for all three.

**`Ticker`.** rc 120 at `method "SIGN" of class "String" is not implemented (Phase 5)`, for both a
seconds string and a `TimeSpan` interval. Past that, `CoreClasses.orx:1627`'s `init` reaches
`!createTimer` (`EXTERNAL 'LIBRARY REXX ticker_createTimer'`), `guard off` and `reply`.

**A second blocker they share, independent of the above.** A live `Alarm` or `Ticker` keeps the
interpreter running until it fires: measured, `.Alarm~new(86400, .Message~new(.Object~new,
"STRING"))` printed `instance Alarm` on the oracle and then did not exit inside a 60 s bound. The
same program with `; o~cancel` exits rc 0 immediately. So both need a construction cell holding
**more than one clause**, and today's cell is a single expression. I did not build multi-clause
support: nothing I can commit would exercise it, and an unexercised path here is precisely the
shape of defect this task exists to remove. It is written into the plan for whoever takes them.

### Table C, before and after

`cargo test --release -p rexx-exec --test gate_table_c`, report read from stderr:

| | BASE `f3b637fde` | after |
|---|---|---|
| 5c rows | 1347 | 1347 |
| **5c not yet `agree`** | **868** | **815** |
| `agree` (whole table) | 620 | 673 |
| `diverge-both` | 371 | 371 |
| `unanswered` | 497 | 444 |
| 5a / 5b not yet `agree` | 0 / 0 | 0 / 0 |

53 rows moved, which is 47 + 2 + 2 + 2 exactly. Nothing else in the table moved.

---

## How "this class is `covered`" was made to mean "the derived probe constructs an instance"

Four links, and the last one is the only one that *runs*:

1. **The extractor cannot say `covered` without a program.** `Coverage::Covered(String)` carries it,
   and `class_set_rows` renders the status and the column out of that one value. There is no code
   path that produces one without the other.
2. **The committed file cannot disagree with the extractor.** `extract_docs.rs`'s both-directions
   check compares `class-set.txt` against a fresh derivation, so a hand-edit of either half of the
   pair reddens.
3. **The gate cannot read them apart.** `construction_of` panics on `covered` with `-` and on a
   non-`covered` row carrying a program, and `method_probe_text` takes the resulting value rather
   than the status string — a `Constructs` is the only thing that can emit a program, and a `Raises`
   is the only thing that can emit a bare `~new` for a class the row set says nothing about.
4. **The oracle has to answer.** A `covered` instance group's shape is `Exactly(rows.len())`. A
   route that does not construct answers zero lines and is a structural failure, which is red in
   every mode and is not a verdict `REXX_CORPUS_GATE` can relax.

Link 4 is what makes the status a claim rather than a string; links 1–3 are what stop the status and
the probe from drifting apart. All four were run against, below.

## What would still let someone flip a class without it

**A class with no instance-arm method row.** Nothing runs the claim, because no instance probe
exists to run it. Measured over `class-methods.txt`: those classes are `Buffer`, `Singleton`,
`Validate` and `ArgUtil`. `Buffer` is separately blocked by `coverage_of`'s `Unreachable`
assertion. For the other three the hand-written half is now closed by
`every_committed_construction_program_has_an_instance_arm_to_run_it`, so **what remains is a false
`CONSTRUCTION` entry** — writing `"new"` for `Singleton`, `Validate` or `ArgUtil` makes the class
`covered` over a bare `~new` nothing ever sends. `Validate` and `ArgUtil` already claim `"new"` on
the strength of the 2026-08-21 sweep, and that const's own doc says nothing re-measures it.

**A program that constructs on the oracle and not here is not this hole**, and I measured which way
it falls rather than reasoning about it: it turns the group's rows `diverge-both`, never `agree`
(control B below). That is the correct behaviour — the row is red until the crate constructs — but
it is worth saying that it does *not* fail `gate_table_c` today, because 5c is not in
`CLOSED_PHASES`. It would fail the phase gate Task 6 turns on.

**`REXX_DOCS_LESS=1` turns off links 1 and 2**, not 3 and 4. On a platform without `oodocs/` the
derivation checks return early, so a hand-edited `class-set.txt` is unchecked against the book —
but `construction_of`'s panic and the `Exactly` shape still fire, and `check_probe_text` still
re-derives all 96 probes from the committed row set.

---

## The controls

All four were run, and each is reported from the output of the run.

**A — `covered` with no committed program**, the control the brief asks for. Hand-edited
`corpus/docs/class-set.txt`'s `Alarm` row from `not-covered` to `covered`, leaving `-` in the
construction cell:

```
cargo test --release -p rexx-exec --test gate_table_c            ->  exit 101
  panicked at crates/rexx-exec/tests/gate_table_c.rs:243:36:
  class-set.txt records Alarm as `covered` and carries no construction program for it.
  `covered` is exactly the claim that one is committed
cargo test -p rexx-extract --test extract_docs                   ->  exit 101, 6 passed 2 failed
  assertion `left == right` failed: Alarm's status disagrees between the two row sets
  rows derived and not committed (1) / rows committed and no longer derived (1)
```

**C — `covered` over a route that does not construct**, which is the same flip made *consistently*
so that A's panic cannot see it. Edited `CONSTRUCTION`'s `("ALARM", "93.901")` to `("ALARM",
"new")` — a false claim about the oracle — then regenerated the row sets and all 96 probes, so
every one of links 1, 2 and 3 is satisfied and `alarm__instance.rex` opens with `o = .Alarm~new`:

```
cargo test --release -p rexx-exec --test gate_table_c            ->  exit 101
  structural failures, which are red in every mode and are not verdicts REXX_CORPUS_GATE
  could relax:
    gate-tables/methods/alarm__instance.rex: the oracle answered 0 line(s) where this row's
    probe asks for exactly 7. ...
```

This is the one that matters: it is the failure mode `gate_table_c.rs:815` named at BASE, and it is
now caught by running rather than by reading.

**B — a program that constructs on the oracle only.** Added
`("MESSAGE", ".Message~new(.Object~new, \"STRING\")")` to `CONSTRUCTION_PROGRAMS` and regenerated:

```
cargo test --release -p rexx-exec --test gate_table_c            ->  exit 0
  Message   instance  loud=yes  5c  20  row(s): diverge-both=20
  5c: 1347 rows, 815 not yet `agree`      (unchanged; the 20 moved unanswered -> diverge)
```

**D — the new `extract_docs` test.** Added `("SINGLETON", ".Singleton~new(1)")`, a class with no
instance-arm row:

```
cargo test -p rexx-extract --test extract_docs every_committed_construction_program
  ->  test result: FAILED. 0 passed; 1 failed; 8 filtered out
  SINGLETON has the committed construction program .Singleton~new(1) and no instance-arm
  method row, so no probe ever runs it
```

The run count is non-zero, so this is a failure and not a filter that matched nothing.

After each control the edited files were restored from copies under the session scratchpad — never
`git checkout --` — and the restore verified with `cmp` and `diff -r` before the next step.

---

## Gates

Tree hash (`git diff HEAD | sha256sum`) before the first gate and after the last:
`23a483fc631d8651763292531a85aaff9cc7e0346522a5d953f616dfec66adfe`, unchanged. The report file
itself is the one thing added after the runs; it is not read by any gate.

| # | command (from `rust/`) | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace` | **101** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace` | **101** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **101** |

Clippy re-checked the two crates this change touches -- `Checking rexx-extract`, `Checking
rexx-exec` -- so its green is evidence the linter looked at the new code rather than reusing a
result.

**Gates 3, 4 and 5 fail on one test, and it is not this change's.**
`refusal_sites::the_table_holds_every_constructor_the_source_defines` reports 41 rows of
`corpus/refusal-sites.tsv` naming `crates/rexx-exec/src/lib.rs` lines exactly **13 higher** than the
source scan finds them -- `accessor_variable` at `:1125` against `:1112`, and so on for all 41.

The cause is `964dc68ba` *Bring the watchdog's doc comments under the tightened policy*, two commits
before BASE: it is the last commit to touch `lib.rs` (`9 insertions, 23 deletions`, the first hunk
at `lib.rs:164`, net -13 for everything below it) and it did not re-derive the table, whose last
commit is `cbeb7f2ce`. **Neither of that test's two inputs is in this change**: `git diff HEAD --
corpus/refusal-sites.tsv crates/rexx-exec/src crates/rexx-exec/tests/refusal_sites.rs` is empty, and
the test reads nothing else (its only `read_to_string` calls are the table and the files under
`src`). I did not re-derive the table: the fix belongs to whoever owns `964dc68ba`, and putting an
unrelated derived artifact in this commit would misattribute it. **It is reported to the team lead.**

**Gates 3 and 4 also stopped early because of it.** `cargo test` without `--no-fail-fast` halts at
the first failing binary, and `refusal_sites` sorts after `parse_version_oracle`, so those two runs
never reached `run_*`, `sourceline_*`, `trace_*` or `rexx-extract`'s tests at all. **Gate 5 is the
one that covers the workspace**, and it ran everything: exactly one failing target,
`-p rexx-exec --test refusal_sites`, with `concept_and_class_gate_table`,
`every_row_set_is_exactly_what_its_extractor_derives_today` and
`every_committed_construction_program_has_an_instance_arm_to_run_it` all `ok` in it.

---

## What I did not do

* **`Alarm` and `Ticker` are not `covered`.** The brief's done-when asks for seven of surface B at
  rc 0 on both engines; four moved and two are blocked on features outside this task (the operator
  applied to a user-class instance; `String~sign` plus a native timer, `guard` and `reply`). Both
  blockers are measured above and written into the plan.
* **The construction cell holds a single expression, not a clause sequence.** `Alarm`, `Ticker` and
  probably `VariableReference` will need more. I did not build it because nothing I can commit would
  run it.
* **`file__instance` is untouched**, per the brief: it is 5d's.
* **`CONSTRUCTION` is still not re-measured by anything.** Its doc says so by decision and I did not
  change that decision; the residual hole above is the consequence.
* **No method body was implemented and no gate row moved by implementing one.** That is F1's
  correction and it is out of this phase.
* **I did not run the phase gate** (`REXX_PHASE_GATE=5c`): Task 6 turns it on and 5c is not in
  `CLOSED_PHASES` yet.
