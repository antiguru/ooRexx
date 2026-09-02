# Task 9 report: the instance-side reading of 5a's limits

**Status: COMMITTED at `08799132c`**, read back with `git log` rather than written from memory.

BASE: `b0faac922`, tree clean at start and clean again after the commit.

## 0. The gates

Every status read unpiped from the file the runner wrote as each finished, in the same turn as the
commit. The run started 09:22 and wrote its `DONE` marker at 09:40; no `cargo` process was alive
when the statuses were read.

| # | command | status |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |
| 6 | `REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **0** |

`/bin/grep -ch FAILED` over gates 3, 4 and 5 is **0, 0, 0**. Gate 4's corpus line is **`327 of 327
matching`** -- 326 before this task, plus `array_allocation_refused.rex`. Gate 6: **every 5b row in
both tables reads `agree`** (table C's six and table D's two), and each table prints `gated by this
run: 0 row(s) whose owning phase is closing or closed and whose verdict is not agree`. So no red row
is mine.

**The tree did not move under the run.** A sha256 of every tracked and untracked file's *bytes*
(`git ls-files -co --exclude-standard -z | sort -z | xargs -0 sha256sum`) was taken before gate 1 and
after gate 6, and the two are identical -- `git status --porcelain` names an untracked path without
reading it and `git diff` skips untracked files entirely, so neither would have seen a change to the
committed table or the new corpus program.

**Three earlier gate runs were discarded**, and the reason is worth recording: each spanned an edit
of mine and so certified neither version. The first died on `fmt` and `clippy` over the new test
file; the second was voided when the derived table changed mid-run; the third when the walk's
`reached` correction landed. Only the fourth ran start to finish on the bytes that were committed.

## 0b. The citation audit

* Every `interpreter/` citation in the staged diff, read back with `sed -n "Np"`:
  `runtime/MethodArguments.hpp:675` (`inline ArrayClass *arrayArgument(RexxObject *object, size_t
  position)`) and `:717` (the closing brace of the named-argument overload) are this task's and are
  right. `classes/ArrayClass.hpp:329` and `:62` appear in the diff's context lines and are
  pre-existing; both were checked anyway and both are right.
* Every crate citation added: `dispatch.rs:5057`, `:4213`, `:2306`, `:2860`, `error.rs:1322`,
  `builtin.rs:1123` and `:1144`, all read back.
* **All 174 definition citations in `corpus/refusal-sites.tsv` were checked independently of the
  test that also checks them** -- a script re-read each `file:line` and required a matching
  `fn <name>(` there. 174 checked, 0 wrong.

## 1. The enumeration

### What the brief's three candidate figures actually measure, re-measured

All three reproduce at BASE:

```
/bin/grep -rhao "Loud::[a-z_][a-z_0-9]*" crates/rexx-exec/src | sort -u | wc -l      -> 42
/bin/grep -ac "^\s*pub(crate) fn " crates/rexx-exec/src/error.rs                     -> 122
/bin/grep -rhao "Raised::[a-z_][a-z_0-9]*" crates/rexx-exec/src | sort -u | wc -l    -> 120
```

**All three are wrong as counts of the surface, in both directions, and each for its own reason.**

* `Loud::`'s 42 includes `Loud::parse`, which **does not exist**. It appears only inside comments --
  `lib.rs:1423` ("There is no `Loud::parse`, and its absence is the fix") and `run.rs:10782` ("this
  used to be `Loud::parse`"). `-rhao` reads comments, so a constructor named only to say it was
  deleted counts as one that exists.
* `error.rs`'s 122 counts every `pub(crate) fn` in the file, including accessors that build no error
  (`report`, `reportable`, `rc`, `description`, `message`).
* `Raised::`'s 120 is a count of *call-site spellings*, and it misses every `Raised` value built by a
  helper that is not spelled `Raised::` at its call site -- `run.rs`'s `raised_if_not_logical`,
  `raised_when_not_logical`, `raised_leave_no_loop`, `raised_iterate_wrong_kind` and their family,
  and `trace.rs`'s `raised_invalid_trace_letter` and `raised_numeric_trace_interactive_only`.
  `dispatch/native.rs`'s `deferred_send` is a `Loud` constructor the `Loud::` grep misses for the
  same reason. The whole set is the committed table's rows whose fourth column names neither
  `error.rs` nor `lib.rs`:
  `/bin/grep -v '^#' corpus/refusal-sites.tsv | awk -F'\t' '$4 !~ /error\.rs|lib\.rs/'`.

So the derivation is over **definition sites, not name mentions**: every `fn` in
`crates/rexx-exec/src` whose return type is `Loud` or `Raised`, with `#[cfg(test)] mod` blocks
excluded. That is **174** constructors, and it is neither 42, nor 120, nor their union.

### The pick

The goal is *limits 5a pinned with a class as the receiver*. Neither the `Loud` surface nor the
`Raised` surface is that set, and neither is their union: most of the union is clause-level errors
(`X2D`'s invalid digit, `DATE`'s format, a loop's `UNTIL` not logical) whose choice no receiver
enters. Taking the union as the walk would have been an enumeration chosen for its defensibility
rather than for the goal.

**The derived list is the union of all 174** -- that is what is committed, so a constructor added
later cannot go unlisted -- **and it is partitioned by a second derived column, the file each
constructor's construction sites are in.** The send machinery is `dispatch.rs` (outside its
`#[cfg(test)] mod tests`) and `dispatch/native.rs`. `Interp::send_message` (`dispatch.rs:2860`) and
`Interp::enter_method_body` (`dispatch.rs:2306`) are both there, and so is every native method body
a send resolves to. **The partition does not rest on that being true**, which matters because it is
the kind of claim this project's record says comes out wrong: what it rests on is where each
constructor's own construction sites are, which the test re-derives from the source and the table's
third column records. That partition is:

| surface | constructors |
|---|---|
| `send` only | 41 |
| `body` + `send` | 14 |
| `body` only | 106 |
| `ir` only | 9 |
| `body` + `ir` | 4 |

**The walk is the 55 with a construction site on the send surface.** Those are exactly the limits
whose *choice* the receiver is an input to -- the ones 5a could only ever pin with a class object in
the receiver position. What the other 119 contain that these do not is stated in section 5, which
is the "what I did not walk" section.

**Those two columns are the tree as this task leaves it, and one row moved under this task's own
hand**: the array-allocation fix in section 2 put `Raised::system_resources` on the send surface, so
it is a `body+send` row here where at BASE it was `body`, and the walk is 55 where it was 54 when the
enumeration was first derived. The moment that happened is recorded as control 2b in section 3.

## 2. The walk

`corpus/refusal-sites.tsv` holds all 174 rows. Its `verdict`, `reached`, `answer` and `witness`
columns are filled for the 55 on the send surface. `crates/rexx-exec/tests/refusal_sites.rs`
re-derives columns 1 to 4 from `crates/rexx-exec/src` on every run and fails if the committed file
disagrees, and holds `answer` against the constructor's own source in both directions -- which is
what makes the coverage claim an assertion rather than a paragraph. The rule and its controls are in
their own subsection below.

The derivation, committed as that test rather than as a shell command, because a command in a report
is not re-run: every `fn` in `crates/rexx-exec/src` whose return type is `Loud` or `Raised`, with
`#[cfg(test)] mod` blocks dropped and line comments stripped. The scratchpad script that produced the
first copy is the same rule in Python and the two agree row for row; the Rust one is what ships.

Each of the 55 was probed with an instance receiver where an instance can hold one, and with the
message that reaches the site otherwise. Probes ran from a fresh directory per probe, three
descriptors read separately, on `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, against the oracle
under the standard wrapper. `parse version` was checked first and answers
`REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026` on all three.

**Every probe's engines agreed with each other.** No row in this walk separates `ir` from
`tree-walker`.

### What the walk closed

**An array size the allocator cannot satisfy aborted this crate.** Four paths, all on the send
surface, all measured before the fix at rc 134 with `memory allocation of 15999999999999984 bytes
failed` on stderr and no traceback:

| program | oracle | this crate, before |
|---|---|---|
| `.array~new(999999999999999)` | rc 251, `Error 5 ... System resources exhausted.` | rc 134, abort |
| `.array~new(999999999999999,1)` | rc 251, same | rc 134, abort |
| `a = .array~new(0) ; a[999999999999999] = 1` | rc 251, same | rc 134, abort |
| `a = .array~new(1,1) ; a[999999999,999999] = 1` | rc 251, same | rc 134, abort |

The oracle's answer is a **trappable** condition, measured: `signal on syntax` over the first
program is rc 0 printing `trapped SYNTAX`, where this crate died. The neighbour above the cap,
`.array~new(100000000000000001)`, agreed byte for byte before and after -- that is 93.959 from
`MAX_FIXED_ARRAY_SIZE`, a different check, and it is what says the fix did not simply refuse
everything.

The fix is `Vec::try_reserve_exact` at the four sites, raising `Raised::system_resources`. That is
not a new mechanism: `error.rs:1300`-`:1322` already states it as the crate's way of asking the
allocator rather than a threshold, and `builtin.rs`'s result buffers already ask fallibly --
`try_reserve_exact` at `:1123` and `try_reserve` at `:1144`, the difference being lending rather
than fallibility. The four array sites were simply not on it. After the fix all five programs agree on both
engines.

**This is not the licensed OOM divergence.** That licence (`docs/superpowers/specs/2026-08-27-phase-5b-instances.md:34`-`:40`)
is about class objects never being collected, so a program the oracle answers can be OOM-killed
here. It says nothing about an unguarded allocation, and `Raised::system_resources`'s own doc had
already ruled the opposite way for every other site that sizes a buffer from user input.

### Divergences the walk found and did not close, each with an owner

Written into the plan's Task 9 section, which is where the next reader looks:

* **A multi-dimensional array as an argument array is a silent wrong answer** in `FORWARD
  ARGUMENTS`, `~sendWith`, `~startWith` and `~run`'s `A` style. Oracle 98.946 / 88.913 / 98.913
  against rc 0 here. This is a **correction to a plan row that said something else** -- see section 4.
  Owner: the `Array` argument-conversion surface. **Not `Raised::message_array_shape`**, whose own
  93.946 is the message-*name* array's shape check and agrees: `say o~sendWith(.array~new(2,2))` is
  163 on both sides.
* **`Package~classes`** is `.Array~package~classes~items` oracle rc 0 `67` against rc 120. Owner:
  the `Package` class surface.
* **`setMethod` with a source that is not a string, an array or a method object** is oracle rc 163
  93.974 against rc 120. Not closed here because the same arm is what a `Method` object will reach
  once `.Method~new` exists. Owner: whoever lands `.Method~new`.
* **`setMethod` sent to a class object from one of its own class methods.** `say .K~cm` then
  `say .K~zz`, where `::method cm class` does `self~setMethod('ZZ', "return 'from-set'")` and
  returns `done set`, is oracle rc 0 printing both lines against rc 120 `a receiver with no scope of
  its own is not implemented (Phase 5)`. **Sent from the main body it is 98.991 on both sides**, the
  access check rather than this gap, so the sender being the receiver is what the row turns on. Loud,
  not silent. **Owner: whoever owns per-object methods** -- it is `Loud::object_method`'s own
  disclosed gap, reached from the one direction that gets past the access checks. Re-measured at the
  committed tree, and independently reproduced by the controller.
* **`Package~publicClasses`, where the REXX package is the condition and the method name is not.**
  `say .Array~package~publicClasses~class~id` is oracle rc 0 `StringTable` against rc 120 `the REXX
  package's class table is not implemented (Phase 5)`. **The obvious spelling agrees**:
  `say .context~package~publicClasses~class~id` is `StringTable` on both sides with empty stderr,
  because `native_package_public_classes` splits on `which_package` and only `Package::Rexx` refuses
  (`dispatch.rs:5057`). `.Array~package~name` is `REXX` on both sides, so obtaining the REXX package
  object is not the gap either. **Owner: whoever owns the `Package` class surface**, with
  `Package~classes` above.
  **My first witness was worse than this one and would have misled a reader.** It was
  `.Array~package~publicClasses~items`, which reaches the site but is confounded: `~items` is rc 120
  on the *program* package too, because `StringTable~items` is a separate gap. The controller could
  not reproduce the row from the method name, tried `.context~package`, got byte-identical output,
  and asked for the reaching program verbatim. Both are now in the row and in the plan, with the
  agreeing neighbour beside them.
* **A quoted compound `::ATTRIBUTE`'s generated accessor**, `o~"A.B"` over `::attribute "A.B"`, is
  oracle rc 0 printing `A.B` against rc 120. This is `Loud::accessor_variable`'s own documented gap
  and it carries no owner string by design -- the scope-pool storage it needs is what
  `Loud::compound_expose` and `Loud::delegate_variable` also need, and nothing has been scheduled to
  build it. What this task adds is that the instance side reads identically to the class side the
  doc measured, so the refusal does not widen with `~new` landed.

The committed table's send-surface verdicts are 40 `agrees`, 12 `diverges`, 2 `recorded` and 1
`not-run`, and its `reached` column is 50 `yes` against 5 `no`.
`/bin/grep -v '^#' corpus/refusal-sites.tsv | awk -F'\t' '$3 ~ /send/ {print $5, $6}' | sort |
uniq -c` prints both.

### Rows the walk confirmed rather than moved

Re-measured at BASE, both engines, three descriptors: the `~define`-then-send row; the `defaultName`
override inside a 97.1's substitution; the `::METHOD string` override under a truth test; `.Array~of`
through `FORWARD ARGUMENTS`; `~dimensions`; `'abc'~copy`; `.Message~new`; `.Array~subclass('K')~new`;
the stem tail order (`a seen 4 [0] [3] [2] [1]` against `[0] [1] [2] [3]`); a `.StringTable`, a
`.List` and a `.Queue` as `FORWARD ARGUMENTS`; and the `DELEGATE` stem variable. All ten reproduce
exactly as the plan records them.

**`USE STRICT ARG`'s instance side was the unmeasured half and is now measured.** The plan pinned it
with `::METHOD m CLASS`. `.K~new~m` over `use strict arg v` is oracle rc 163 `Error 93.901:  Not
enough arguments for method; 1 expected.` against rc 216 `Error 40.3:  Not enough arguments in
invocation of M; minimum expected is 1.` -- the same wrong pair, so the receiver does not enter it.
`.Alarm~new` reaches it through a prologue class exactly as the plan predicted: oracle rc 163
93.901 `2 expected` against rc 216 40.3.

### Rows that are already owned refusals, confirmed reached from a second direction

* `Loud::entry_method_without_a_value` is reached through `~sendWith` as well as through the direct
  `d~"NAME="()` spelling: `.local~sendWith('ZZZ=', .array~new(1))` is oracle rc 165 91.999 against
  rc 120. The oracle's answer here is the uninitialised read its own doc records, so the refusal
  stands.
* `Loud::receiver_class` for a stem receiver (`a. = 'dflt'` then `a.~length`, oracle rc 0 `4`
  against rc 120) is the gap that constructor's own doc names.
* `Loud::unreadable_collection` (`.K~defineMethods(.local)`, oracle rc 163 93.974 against rc 120)
  and `Loud::deferred_send` (`.Stream~new('x')~lineIn(1,2,3,4,5)`, oracle rc 168 88.922 against
  rc 120 naming `stream_init`) are Phase 7's, named in the refusal text itself.

## 3. The controls

Constraint 2 of `global-constraints.md`: a witness must be run against a control that makes it fail,
and the control must be recorded as run.

Controls 5a, 5b and 5c are in section 2 beside the rule they test, because a control that is not read
next to its subject is decoration. They are part of this list and were run the same way.

**Control 1, the harness itself.** Before trusting an `AGREE`, the same harness was run over the ten
divergences the plan already records. All ten reported `DIVERGE` on both engines:
`plan_define_send`, `plan_defaultname_zzz`, `plan_string_truth`, `plan_usestrict_class`,
`plan_usestrict_inst`, `plan_array_of`, `plan_array_dimensions`, `plan_string_copy`,
`plan_message_new`, `plan_array_subclass_new`. So an `AGREE` in this walk is a comparison that can
come out the other way.

**Control 2, the derived-list test against a new site.** A constructor was added to `dispatch.rs`
that the table does not list:

```rust
fn control_only_constructor() -> Loud {
    Loud::object_method("a control")
}
```

`cargo test --release -p rexx-exec --test refusal_sites` then fails
`the_table_holds_every_constructor_the_source_defines`, reporting one row under "in the source, not
the table" -- `Loud`, `control_only_constructor`, an empty surface, `crates/rexx-exec/src/dispatch.rs:5102`
-- and nothing under "in the table, not the source". Restored from a scratchpad copy, not with
`git checkout --`.

**Control 2b, and it was not planned.** The same test went red on this task's *own* change before
either control was written: guarding the array allocations put `Raised::system_resources` on the
send surface, its `surface` column moved from `body` to `body+send`, and the committed table said
`body`. That is the mechanism firing on a real edit rather than on a mutation, which is stronger
evidence than the planted one.

**Control 3, the walk-coverage test against an unwalked row.** `Raised no_method`'s verdict and
witness were set to `off-send-surface` and `-` in the committed table.
`every_send_surface_row_is_walked` then fails with
`no_method: "off-send-surface" is not one of the four verdicts`. Restored from a scratchpad copy.

**Control 4, the corpus witness against the unfixed crate.** `crates/rexx-exec/src/dispatch.rs` was
replaced with `git show b0faac922:rust/crates/rexx-exec/src/dispatch.rs` (BASE, no guard) and
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` run. It exits **101**, the test
binary dying with `signal: 6, SIGABRT` and `memory allocation of 15999999999999984 bytes failed` --
the harness runs the program in process, so before the fix the new row does not produce a mismatch
line, it kills the harness. Restored from a scratchpad copy of the fixed file.

**The first attempt at control 4 was a no-op and is recorded because it looked exactly like a
result.** The backup it restored from had been taken *after* the fix, so "reverting" wrote the same
bytes back; cargo printed no `Compiling` line and the corpus stayed at `327 of 327` and exit 0. Read
without checking, that is a control saying the corpus cannot see the defect. What caught it was
`/bin/grep -c empty_slots` on the supposedly-reverted file answering 4 rather than 0.

## 4. Corrections made to the plan

`docs/superpowers/plans/2026-08-27-phase-5b.md`, Task 9's section:

1. **The multi-dimensional `FORWARD ARGUMENTS` row said the wrong thing and now says what is true.**
   It read "`arguments (.array~new(2,2))` is oracle rc 158 98.946 against rc 120 on `~new`". That was
   true when written and Task 8 falsified it by landing `.Array~new(2,3)`: `~new` answers now, so the
   send goes through and the crate is **rc 0 with `a seen 0`**. A loud refusal became a silent wrong
   answer without anything in the tree noticing -- the phase's own worst-defect class, arriving by a
   route the constraints do not warn about, which is a *neighbouring* task closing the gap that was
   holding the loud refusal in place.
   The row was split in two: the `.List`/`.Queue` half is unchanged and re-measured, and the
   multi-dimensional half is rewritten with its three newly-found siblings (`~sendWith`,
   `~startWith`, `~run`'s `A` style), the C++ that decides all four
   (`runtime/MethodArguments.hpp:675`-`:717`, `arrayArgument`'s `array->isMultiDimensional()` check),
   and an owner.
2. **Task 9's own findings were added**, with the closed one marked closed and the two open ones
   given owners.
3. **The confirmations were added** so a later reader does not re-run ten probes to learn nothing
   moved, and `USE STRICT ARG`'s instance side -- the half this task existed to measure -- is now
   recorded beside the class side.
4. **The three enumeration candidates the section offers were measured and all three are wrong as
   counts of the surface.** The section still offers them as candidates, which is correct as
   history; a paragraph after it records what each miscounts and what was derived instead. This is
   the plan's own instruction ("derive the enumeration from the tree; do not choose it") reaching a
   different answer from any of its three suggestions, which is what deriving means.

Nothing in `docs/superpowers/specs/2026-08-27-phase-5b-instances.md` was found wrong. The one place
this task had to read it closely -- the OOM licence at `:34`-`:40` -- says what it means and does not
cover an unguarded allocation, checked before the fix rather than after.

## 5. What I did not walk, and why

This is the load-bearing section.

**119 of the 174 constructors were not probed.** They are the rows whose `surface` column is `body`,
`ir`, `body+ir`, and they are in the committed table under `off-send-surface`. What they contain that
the send surface does not:

* **`body` (106) is the clause-level error surface**: `X2D`'s invalid digit, `DATE`'s format,
  `NUMERIC`'s, a loop's `UNTIL` not logical, `PARSE`'s trigger, `SIGNAL`'s label, `INTERPRET`'s.
  A `::METHOD` body can contain any clause, so **every one of them is reachable from inside a method
  on an instance** -- that is a fact about Rexx, not a guess, and it is why "reachable from a send"
  does not narrow anything and was not used as the criterion. What the receiver does *not* do at any
  of them is enter the choice: the same clause in a routine, in a class method and in an instance
  method picks the same constructor. The receiver reaches them only through the value it may be
  standing in for and through the traceback frame above.
* **`ir` (9) and `body+ir` (4) are compiled-stream invariants** -- `chunk_map_too_short`,
  `jump_out_of_range`, `register_not_logical`, the four `*_op_off_its_node`. No Rexx program is
  supposed to reach one at all.

**What that leaves unmeasured, stated as a risk and not as a reassurance.** For a `body` row, the two
ways a receiver can still change the answer are (a) the error quoting a value that happens to be an
instance, and (b) the `Compiled method "X" with scope "Y".` frame. This task measured (a) at the
sites where the plan already had a row -- the 97.1 substitution, the truth test -- and confirmed both;
it did **not** sweep the `body` surface for (a). The mechanism behind both plan rows is one function,
`Interp::string_value_text`, which is infallible and does not send, so any `body` error quoting a
value inherits the same gap the moment an override is in play. **That is a mechanism claim and this
task did not run it at 106 sites**, which is exactly the kind of claim this project's record says
comes out wrong. A later task sweeping the `body` surface with a `::METHOD defaultName` override in
place would be measuring something real.

**Two send-surface rows were not run at all, on purpose:**

* `Loud::array_index_hole` -- the shape is `corpus/oracle-crashes.txt`'s entry 6 (SF #2085) and must
  never be handed to the oracle. Verdict `not-run`.
* `Raised::insufficient_stack` -- reached by the self-forward, whose oracle half is
  `corpus/oracle-crashes.txt`'s last entry. Verdict `recorded`. Neither the shape nor its
  `REPLY`-preceded family was constructed at any point in this task.

### The coverage claim is over the sites a probe reached, and it is asserted

**The trap this walk nearly shipped.** A test asserting that every derived site has a row would have
been green over rows whose probe reached a check that fires first -- the witness-that-cannot-fail
shape, one level up from the rows it is about. `every_send_surface_row_is_walked` alone is exactly
that test. The relation that had to be asserted is *reached*, not *has-a-row*.

**The rule, stated so it can be checked.** A constructor's own identifiers are derived from the
source: every `M.N` it builds with `syntax(M, N)` -- the test reads the digits after the paren and
after the comma, so a wrapped signature does not hide one -- together with the text of its own
definition and of every one of its construction sites. A row's `answer` column names what its probe
answered. `reached=yes` requires that answer to be one of those identifiers; `reached=no` requires it
not to be, and to say why there is no route.
`a_reached_row_carries_its_own_site_identifier_and_an_unreached_one_does_not` enforces both
directions, so a row cannot claim a site it did not reach and cannot disclaim one it did.

**What the rule does with a constructor that names no number.** `Raised::nostring` builds a condition
rather than a `syntax(...)` and `Raised::nomethod` re-wraps another report, so neither has an `M.N`
of its own. Those rows carry the condition name the constructor does write -- `NOSTRING`, `NOMETHOD`
-- which the text half of the rule admits. That half is also what carries every `Loud` row, whose
identifier is a message rather than a number, including the ones whose text lives at the construction
site rather than in the constructor (`Loud::method_from_source`, `Loud::object_method`).

**Two identifiers can collide and the rule does not pretend otherwise.** `88.909` is both
`argument_needs_a_string_value` and `named_argument_needs_a_string_value`; the number cannot separate
them and the witness does -- `Argument 1 must have a string value.` against `Argument scope option
must have a string value.` The table header says so.

**What the rule cannot do.** It does not re-run a probe; nothing in this crate reaches the oracle from
a unit test. It ties each row to the *source*, so a constructor's number or message changing under a
row reddens it, and it cannot see the crate's answer drifting away from the oracle's --
`tests/corpus.rs` is what sees that, for the rows that reach it.

**The controls, all three run and all three red.**

| control | mutation | result |
|---|---|---|
| 5a | `abstract_class`, `reached` yes -> no, answer left at `98.989` | red: *"98.989 is one of this constructor's own identifiers, so the probe did reach it"* |
| 5b | `package_scope_method`, `reached` no -> yes, answer left at its reason text | red: *"is none of this constructor's own identifiers (its syntax(M, N) numbers are [\"97.3\"])"* |
| 5c | `invalid_position`, answer `93.924` -> `93.907`, its neighbour's number | red: *"93.907 is none of this constructor's own identifiers"* |

**Re-running every diverging row's witness at the committed tree caught a third bad row, and it is
the same defect as 5c wearing different clothes.** `message_array_shape` was marked `diverges` from
`o~sendWith('M', .array~new(2,2))` while its `answer`, `93.946`, had been measured on
`o~sendWith(.array~new(2,2))` -- two programs, one row. The second is the message-*name* array's own
shape check and it agrees; the first is the *argument* array's multidimensional case, which is one of
the silent wrong answers section 2 hands to the `Array` argument-conversion surface and is not this
constructor's at all. The row is now `agrees` with the program that produced its answer, and the
silent divergence stays where it belongs. The check that found it: run every `diverges` row's witness
again and confirm it still diverges. Eleven did; this one did not.

**5c is the load-bearing arm and the other two are not three of a kind.** 5a and 5b move a verdict
against text that is obviously right or obviously wrong for it, and a careful reader could have caught
either. 5c changes nothing but one error number, to `93.907` -- a real error that this row's own probe
really produced, on the same receiver, one check earlier. Nothing distinguishes it by inspection:
the row reads as a measured result either way. Only holding the answer against *that site's* own
identifiers separates them, which is the whole reason the rule exists rather than a reviewing pass.
The table was restored from a scratchpad copy after each, and the suite is green again at the
committed bytes.

### The audit shrank the gap rather than relabelling it

Applying the rule the first time said 17 of the 55 rows had reached a different check. Each was then
either probed better or recorded as having no route. **Twelve were closed by a better probe**, and the
probes are the interesting part, because each names the check that was in the way:

* `missing_named_argument` (88.901), `not_one_of` (88.916) and
  `named_argument_needs_a_string_value` (88.909) sit behind `setMethod`'s access checks, and are
  reached by sending from a method of the receiving object itself.
* `not_enough_method_arguments` (`(1,2,3)~at()`, 93.901) and `invalid_position` (`.array~new(2,3)`
  then `m[1.5,1]`, 93.924) sit behind a neighbouring subscript check.
* `copy_not_supported` (`.K~copy`, 93.970), `recursive_inherit` (a mixinclass inheriting itself,
  98.944), `inherit_base_class` (a mixin of an unrelated base, 98.943), `rexx_defined_class`
  (`.Array~define('ZZZ', "return 1")`, 98.985), `invalid_length` (`.stringtable~new('abc')`, 93.923),
  `constant_not_initialized` (a class-side `INIT` reading `::constant c (2+3)`, 97.4),
  `message_name_shape` (93.972) and `too_many_external_arguments` (`.K~sep(1)` on a class method
  bound to `file_separator`, 88.922) each needed a shape nothing in the walk had sent.

**Two more were reached and both diverge**, which is how the audit produced findings rather than only
corrections: `object_method` and `rexx_package_classes`, both in section 2's list with owners.

**Five rows remain unreached, and each says why in its own `answer` column.** Three have no route from
here: `missing_body`, whose every arm guards a directive index the parser fills;
`package_scope_method`, whose second package needs `::REQUIRES`; and `setup_method`, whose two methods
`Setup.cpp` removes before an image ships. Two must never be run: `array_index_hole` and
`insufficient_stack`, both `corpus/oracle-crashes.txt` shapes.

**So the walk covers 50 of the 55 send-surface sites**, and the five it does not are named in the
committed table rather than in this report alone.

### Three rows in this phase have now been green over an absence, and it is a pattern

`accessor_variable` was the third. Its probe was `::attribute "A.B"` read as `o~a`, which is 97.1 on
both sides and never reaches the constructor; the accessor a quoted compound `::ATTRIBUTE` generates
is named `A.B`, so the send has to be `o~"A.B"`. Sent that way it is oracle rc 0 printing `A.B`
against rc 120 here -- the gap `Loud::accessor_variable`'s own doc records with a class receiver,
confirmed identical on the instance side. `restricted_method`'s witness named a program that stops at
97.2, a different check, and was rewritten to the one that reaches 98.991.

The other two are gate table D's `DELEGATE` rows, which this phase's constraints already record as
green over a mechanism that was not implemented -- one of them because the probe never made the send
at all. **The shape is the same every time**: a check whose subject is absent, passing because absence
and success share an outcome. None of the three was caught by rereading. What caught this one was
running a rule over the whole set, which is why the rule became a test rather than an appendix here.


**No performance sitting was run.** `crates/rexx-exec/src/dispatch.rs` is in `src/` of `rexx-exec`,
so the guard applies by its own terms. The change is four `try_reserve_exact` calls on paths that
allocate a slot vector: `.Array~new`, a write past the end, and a reshape. None is on any of the
eight axes -- `alloc4c`, `arith`, `compound`, `emptyloop`, `strings`, `varlookup`, `dispatchclass`,
`rexxcps` -- and `try_reserve_exact` followed by `resize` is the allocation `vec![None; n]` already
performed, with the failure branch cold. **I am recording this as a sitting I judged unnecessary and
did not run, not as one that came back flat.** If the controller wants it, it is one command and the
figures belong in the ledger.

## 6. The two items that are the controller's, not mine

Neither was built. Both were measured only as far as reporting them needs.

**The self-forward with a preceding `REPLY`.** Not constructed, not run, not capped. Every member is
in `corpus/oracle-crashes.txt`, and nothing in this task went near the family. The plan's account
stands unchanged.

**The stem tail order under `FORWARD ARGUMENTS`, and what it would cost.** Re-measured at BASE, both
engines: `s.0=3`, `s.1='p'`, `s.2='q'`, `s.3='r'` then `arguments (s.)` is oracle rc 0 `a seen 4 [0]
[3] [2] [1]` against this crate's `a seen 4 [0] [1] [2] [3]`, stderr empty on both.

**The cost is larger than "keep insertion order", and the probe above is what says so.** The tails
were assigned `0`, `1`, `2`, `3` in that order and the oracle emits `0`, `3`, `2`, `1`. So recording
insertion order in `Body::Stem` and replaying it would produce `0 1 2 3` -- the answer this crate
already gives -- and close nothing. The oracle's order is a walk of the balanced tree
`CompoundVariableTable` builds, and the tree's *shape*, not the insertion sequence, is what the walk
reads; the sequence only reaches the answer through the balancing rule. Reproducing it means
reproducing that tree, so the change to `Body::Stem` is a second index structure maintained on every
tail write, on the hottest path in the interpreter, and not a `Vec` appended to on insert.

**I judged it out of scope and am saying so before rather than after.** It is a representation change
whose benefit is one differential row and whose cost lands on `compound`, the axis a stem write
*is*. If the controller wants it, the sitting it owes is `compound` and `varlookup` at minimum, and
the design question to settle first is whether this crate reproduces `CompoundVariableTable`'s tree
or finds a cheaper function with the same output -- which nobody has shown exists.
