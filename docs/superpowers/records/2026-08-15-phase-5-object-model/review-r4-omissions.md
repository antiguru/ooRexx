# R4 -- omission review of the Phase 5 spec

Lens: **omission.** I built an independent enumeration of what Phase 5 owns from the code, the
roadmap, the exclusion and gap tables, the parser's owner table, the corpus, and the oracle's own
`.orx` files, and then compared it to the spec. I read
`2026-08-15-phase-5-inherited-surface.md` only after finishing, as a cross-check.

Every probe below ran from a fresh empty directory under the session scratchpad, with absolute
paths, stdout/stderr/status read as three descriptors, and every oracle run inside the required
`ulimit -v 1048576` wrapper.

---

## What each pass searched FOR

**Pass 1 -- refusals in the code.** I searched `rust/crates/` with `/bin/grep -ran` for the string
`Phase 5`, for `is not implemented`, and for `Loud::` construction and call sites, then read
`rexx-exec/src/lib.rs`'s `directive_gap`, `instruction_owner`, `expr_owner` and every `Loud`
constructor in full. **What those terms cannot reach:** a refusal that names no phase and does not
use the words "is not implemented" -- I found one such class by reading the constructors rather
than by grep (`Loud::builtin_option_object`, `Loud::value_selector`, and the unowned
`an environment symbol` message), so I assume more exist that neither term finds. I did not
disassemble `rust/target/release/rexx-run`; I used it only to run programs.

**Pass 2 -- the roadmap.** I searched `docs/superpowers/plans/2026-07-27-rust-rewrite.md` for
`Phase 5`/`phase 5` and read every hit plus the phase table, the crate-layout block, D2 and D12 in
full. **Cannot reach:** a Phase 5 obligation stated without the words "Phase 5", e.g. inside a D-block
that only names its blocked phase in the index table.

**Pass 3 -- exclusion and gap tables.** `phase-4-exclusions.txt` searched for `Phase 5`, `KNOWN GAP`,
`UNASSIGNED`/`No owner`; `corpus/bif-exempt.txt` and `corpus/keyword-exempt.txt` read in full;
`rexx-exec/tests/assertions.rs`'s `EXEMPT` table read; `corpus/builtin-status.txt` read.
**Cannot reach:** a gap row whose owner column is blank and whose prose never says "Phase 5" -- I
caught two of those (`VALUE` on a defined environment name, the `+`-method scope line) only by
reading the `KNOWN GAP` headings, so the grep alone would have missed them.

**Pass 4 -- the parser.** `rexx-exec/tests/owners.rs` (found by grepping for `owners.rs`) read in
full: it is the phase-assignment table, and `EXPECTED_OUT_OF_SCOPE` is the committed set. I took the
Phase 5 rows from that table, not from the spec.

**Pass 5 -- the corpus.** Searched `rust/corpus/` and `rust/corpus-l1/` for `Phase 5`; read
`corpus/README.md`'s `num/` section and `corpus/phase-4a.txt`'s and `phase-4b.txt`'s headers.

**Pass 6 -- the oracle's own surface.** I ran `rexx-run` on all three `.orx` files; read
`CoreClasses.orx`'s entire prolog; enumerated its and `StreamClasses.orx`'s `::` directives,
clause-initial keywords, `.NAME` environment symbols and function-call names with a comment- and
string-stripping scanner I wrote for this session
(`scratchpad/r4/scan.py`, `dots.py`); and cross-checked the function names against
`corpus/builtin-status.txt`. **Cannot reach:** a message name (`~foo`) is not a function call, so
the BIF scan does not see method sends; and my stripper handles `--`, `/* */` and quoted strings but
not continuation-line subtleties, so a name split across a `-` continuation could be missed.

---

## Findings

### HIGH

**H1. The roadmap's Phase 5 exit gate includes the security manager; the spec's exit criteria do
not, and the spec does not say it is removing a gate criterion. CONFIRMED.**

The roadmap's phase table, Phase 5 row, reads: *"`CoreClasses.orx` parses and executes; 32 classes
exist and respond; `::class`/`::method`/`::routine`/`::requires` work; **security manager
interception points in place (D12)**; cold start measured and recorded against C++ (D2)"*. D12 itself
assigns Phase 5 concrete deliverables: *"builds the security manager object, its installation path,
and the hooks that live in dispatch and name resolution -- `.local`/`.environment` lookup and
external function resolution"*, and warns that *"Retrofitting the mechanism after Phases 6-8 means
touching every one of those paths twice."*

The spec's six exit criteria contain nothing about it. It appears once, in "Open questions for the
plan", as *"This spec does not fix it, and the plan must, before the first dispatch call site is
written."* That is not out of scope -- it is an inherited gate criterion silently downgraded to a
plan-level TODO. **Genuinely missing (a).** Worse, D12's own deliverables land in exactly the two
places this spec *does* fix: the dispatch seam (the `resolve`/`define` interface table) and
`.local`/`.environment` lookup (D33). The spec fixes both interfaces without the hook, which is the
retrofit D12 says costs double.

**H2. `StreamClasses.orx` runs is Phase 7's exit gate in the roadmap, and the spec makes it Phase 5
exit criterion 1. CONFIRMED, reverse direction.**

Roadmap phase table, Phase 7 row: *"| 7 | Streams & platform | 5 | **`StreamClasses.orx` runs**;
stream model, `ADDRESS`, file system green on all 5 platforms; the `Sys*` subset ooTest needs (D11)
works |"*. The Phase 5 row names only `CoreClasses.orx`. The crate-layout line the spec cites
(`rexx-lib/ # Phase 5: loads CoreClasses.orx / StreamClasses.orx`) says *loads*, and the spec
converts that to *runs to completion*.

The measurement backs the roadmap. `StreamClasses.orx`'s bodies call `CHARIN`, `CHAROUT`, `LINEIN`
and `LINEOUT`; all four are `excluded` in `corpus/builtin-status.txt`, i.e. Phase 4 excluded them
outright and `phase-4-exclusions.txt` puts the stream builtins in Phase 7. (Method names in
`.orx` come from my scan of that file cross-checked against `builtin-status.txt`.) `CoreClasses.orx`
additionally uses `LINEOUT`, also `excluded`.

The origin is traceable. The survey quotes the Phase 5 gate row in full at `:45`-`:49` -- and that
row names `CoreClasses.orx` and nothing else -- then immediately writes at `:52` *"The three assets
that gate names"* and measures `StreamClasses.orx` among them. The gate names one asset. The spec
inherited the three.

So criterion 1 as written is satisfiable only in the vacuous sense -- the file's directives install
and its prolog is short -- while the classes it defines cannot answer, because their bodies need
Phase 7's builtins. The spec's own criterion 2 ("a registry of thirty-two empty class objects
satisfies criterion 1") is the right instinct applied to the wrong file.

**H3. D33 names the wrong `VALUE` form, and implementing it as written leaves the silent wrong
answer the spec's own body says must close. CONFIRMED by measurement.**

The spec's body describes the gap correctly: *"`say value('.LOCAL')` gives `The Local Directory` on
the oracle and `.LOCAL` here"*. That is `VALUE`'s **two-argument** form (no selector at all). D33
then writes: *"the class registry, environment-symbol lookup and `VALUE`'s **empty-selector form**
all resolve through them."*

Those are different forms with different oracle answers. Measured this session, both interpreters,
same source, from a fresh directory:

```
say value('.LOCAL')        oracle rc 0 "The Local Directory"   rexx-run rc 0 ".LOCAL"
say value('.LOCAL',,'')    oracle rc 0 "..LOCAL"               rexx-run rc 120
                                                               "VALUE's external-selector form is not implemented"
say value('.ARRAY')        oracle rc 0 "The Array class"       rexx-run rc 0 ".ARRAY"
```

The empty-selector form is a *loud* refusal (`Loud::value_selector`, which fires on the third
argument's mere presence) and its oracle answer is `..LOCAL`, not `The Local Directory`. Closing
"the empty-selector form" therefore does not touch the silently-wrong two-argument path at all.
`phase-4-exclusions.txt` keeps them as two separate `KNOWN GAP` rows ("VALUE ON A DEFINED
ENVIRONMENT NAME IS SILENTLY WRONG AT RC 0" and "VALUE'S EXTERNAL-SELECTOR FORM MIXES TWO OWNERS
BEHIND ONE ARGUMENT") for precisely this reason, and the second row records that only branch (a) of
three is Phase 5's. **Genuinely missing (a):** the two-argument form is named nowhere in the
decisions.

**H4. A live, Phase-5-attributed stderr byte divergence -- error tracebacks through method frames --
is absent from the spec. CONFIRMED by re-measurement.**

`phase-4-exclusions.txt` records a divergence and attributes it: *"The missing line is
method-dispatch bookkeeping ... which needs the object model 4a does not have, so it is Phase 5's."*
Re-measured this session, `say b. + 1` with `b.` untouched, both sides rc 215:

```
oracle stderr:
       *-* Compiled method "+" with scope "String".
     1 *-* say b. + 1
     Error 41 running <abs>/g.rex line 1:  Bad arithmetic conversion.
     Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.

rexx-run stderr: identical except the first line is absent.
```

This is not a trace prefix, so the spec's "Trace: `>M>` and `>N>`" section does not cover it, and I
confirmed from the tool's own definition that it could not: `trace_oracle.rs`'s `PREFIX_COVERAGE`
lists exactly `>M>` and `>N>` as `Coverage::Owned("Phase 5")` (the other Owned row is `+++`, Phase
7). The general subject -- what an error traceback prints once the activation stack contains method
frames -- appears nowhere in the spec. **Genuinely missing (a)**, and it is an *active* divergence
today, so any `phase-5.txt` program that touches it fails criterion 2 on stderr.

**H5. The bootstrap prolog's actual requirements are never enumerated, and several of them are not
class objects at all. CONFIRMED by reading the prolog and probing.**

`CoreClasses.orx`'s executable prolog (everything before the first `::`) needs, in order:

* `use arg rexxPackage`, then `rexxPackage~addClass(...)`, `rexxPackage~addPublicClass(...)`,
  `rexxPackage~publicClasses`, `rexxPackage~objectname = ...` -- a live **Package object** with those
  four behaviours. The spec lists `PackageClass` among `Setup.cpp`'s classes and says nothing about
  which of its methods the bootstrap needs.
* `.environment~objectname = ...` -- an attribute-assignment message send (`InstructionKind::Message`).
* `.context~package~publicClasses` -- **`.context`**, the per-activation `RexxContext` reflection
  object, live during the bootstrap. This is not a registry entry; it is state the executor must
  materialise. Probed: `say .context` is `rc 120, rexx-exec: an environment symbol is not implemented`.
* `publicClasses[name]` -- the `[]` index message on a collection.
* `do name over publicClasses` -- `DO OVER` a **collection object**. `LoopKind::Over` is
  `Owner::InScope` in `owners.rs` today, which is true only for what Phase 4 can iterate; over an
  object it must send `makearray`/`supplier`. The spec's dispatch section never mentions it.
* `do name over "nl", "cr", ...` -- `DO OVER` a comma-separated expression list.
* `.methods[("string_cls_" || name)~upper]` -- **`.methods`**, the package's unattached-method
  directory. Probed: `say .methods` is `rc 120, an environment symbol is not implemented`. Not a class
  and not in `Setup.cpp`'s `createInstance()` list; the spec's registry section never names it.
* `.String~defineClassMethod(...)`, `~inheritInstanceMethods(...)`, `~inherit(...)` -- class-side
  methods that mutate behaviour after definition, i.e. the `updateSubClasses` cascade the spec does
  describe.
* `call 'StreamClasses.orx' rexxPackage` -- a `CALL` on a **program name**. The spec says D26 embeds
  the files, but never says how a `CALL` on a filename resolves to an embedded blob. That resolution
  step is precisely the external-file search `Loud::unresolved_call`'s doc assigns to **Phase 7**
  (*"the one step behind those three that this crate skips is the external file search, which is
  Phase 7's"*). **Genuinely missing (a)** -- either rexx-lib intercepts the name before the search,
  which is a mechanism nothing describes, or Phase 5 depends on Phase 7's search.

Note also that the spec's "Verified in this session" claim about the first refusal is correct; I
reproduced it (`rexx-run CoreClasses.orx` -> rc 120,
`rexx-exec: ::CLASS naming another class is not implemented (Phase 5)`), and `StreamClasses.orx`
gives the same message.

---

### MEDIUM

**M6. Seven Phase-5-owned AST rows from `owners.rs` are never named in the spec, two of them used
throughout both `.orx` files. CONFIRMED.**

`owners.rs`'s `EXPECTED_OUT_OF_SCOPE` is the committed set. Its `Phase 5` rows are:
`InstructionKind::Call::Qualified`, `Expose`, `Options`, `Message`, `Guard`, `Reply`, `Forward`;
`ExprKind::QualifiedCall`, `ClassResolver`, `List`, `Message`; `LoopKind::With`.

The spec names `Options` (D31), `Guard` and `Reply` (D32), `ExprKind::List` (D34), and
`ClassResolver` once, incidentally, inside the trace section. It never names **`Expose`**,
**`Forward`**, **`InstructionKind::Message`**, **`Call::Qualified`**, **`ExprKind::QualifiedCall`**
or **`LoopKind::With`**.

`EXPOSE` and `FORWARD` are not edge cases here. Counting clause-initial keywords with
`/bin/grep -aciE "^[[:space:]]*KEYWORD([[:space:]]|;|$)"` over the two files: `CoreClasses.orx` has
112 `expose` and 30 `forward` clauses; `StreamClasses.orx` has 15 and 7. Both are required for the
classes to respond, i.e. for criterion 2. **Genuinely missing (a)** for `Expose`, `Forward` and
`InstructionKind::Message`; the three namespace-related rows are **implied (b)** by D30's
`::REQUIRES` paragraph but are not stated as deliverables.

**M7. The `startup` axis is scoped to Phase 5 by the roadmap, is the denominator of every classic
axis's reported figure, and the spec's performance section never mentions it. CONFIRMED.**

Roadmap `:474`: the debt route *"survives only for `dispatch`, `alloc.rex` and **`startup`**, where
it scopes work to Phase 5 rather than conceding a bar."* D35 names `dispatch`, `alloc` and
`heapshape` and stops.

In `rexx-bench/src/bin/rexx-bench-suite.rs`, `startup` carries `Role::Offset`, whose doc reads
*"Its two sides are **not** comparable: this crate has no `CoreClasses.orx` bootstrap yet, so it
starts fast by not doing the work the oracle does at startup"*, and `write_offset` prints the same
claim into every report naming Phase 5 explicitly. `write_axes` computes
`row.iterations as f64 / (stats.median - side_offset)` -- so the offset is subtracted from **every
classic axis** on both sides.

Two consequences the spec has no rule for. First, that report sentence becomes false the moment the
bootstrap lands, and **nothing asserts it** -- contrast `every_blocked_axis_still_fails_on_this_crate`,
which does assert the `Role::Blocked` rows and which D35 correctly anticipates. Second, once the
offset is tens of milliseconds, `stats.median - side_offset` is a small difference of two large
numbers, which widens the interval criterion 4's floor comparison is read against. **Genuinely
missing (a).**

**M8. `::CONSTANT` is used by `StreamClasses.orx`, is currently installed-and-ignored, and the
spec's directive section never names it. CONFIRMED.**

Directive census over the two files: `CoreClasses.orx` has `::method`, `::class`, `::attribute` only
-- the spec's claim checks out. `StreamClasses.orx` additionally has **`::constant`**.
`directive_gap` returns `None` for `DirectiveKind::Constant`, and the doc above it says a directive
that installs cleanly is *"**ignored**, not implemented"*. Ignored is correct for Phase 4 and wrong
for a phase whose criterion 2 requires classes to respond. The spec's directive section is
`::CLASS`/`::METHOD`/`::ATTRIBUTE` plus `::OPTIONS`/`::REQUIRES`/`REPLY`/`GUARD`.
**Genuinely missing (a)**, conditional on H2's resolution: if `StreamClasses.orx` is Phase 7's, so is
this.

**M9. Committed tables that this project polices in *both* directions and that Phase 5 must retire
are not named in the gate section. CONFIRMED.**

The spec's gate says only *"It extends the existing `corpus.rs` mechanism"*. The following are all
committed sets whose own tests fail when a row *starts passing*, so Phase 5 cannot land without
editing each:

* `rexx-exec/tests/assertions.rs`'s `EXEMPT`. Its module doc: *"All 35 rows are unblocked only by
  Phase 5"*, and `the_exempt_set_matches_the_current_blocked_rows` *"asserts the set
  unconditionally, in every mode"*.
* `corpus/bif-exempt.txt` -- the `Phase 5` rows (`D2C::test10`, the `XRANGE::test_xrange_*` rows) and
  the `UNATTRIBUTED:an environment symbol` rows (`LENGTH::test025`, `REVERSE::test017/021/026`,
  `VALUE::test019`, `STREAM::test_relative_file_exists`/`_exists2`/`_not_exists`/`_not_exists2`). Its
  header states the assertion *"IS NOT BEHIND AN ENV VAR"*.
* `trace_oracle.rs`'s `PREFIX_COVERAGE` (`>M>`, `>N>` move from `Owned` to `Witnessed`) and the
  committed coverage number beside it.
* `owners.rs`'s own module doc names five pinned items an ownership move must edit together
  (`EXPECTED_OUT_OF_SCOPE`, `coverage.rs`'s `EXPECTED_SUBSET`,
  `variant_counts_match_the_audited_split`, `loud.rs`'s witness tables, `lib.rs`'s
  `instruction_owner`/`expr_owner`).
* `corpus/builtin-status.txt`, whose rows are derived from a live differential.

The `UNATTRIBUTED` rows deserve their own line: `expr_owner` gives `ExprKind::DotVariable` **no**
phase, so the refusal is the unowned `rexx-exec: an environment symbol is not implemented`
(reproduced above). D33 closes it, but the spec never says the attribution column changes, and this
is the one row class where the harness has nothing to derive an owner from. **Genuinely missing (a),
mechanical but load-bearing.**

**M10. `phase-5.txt` touches five harnesses, not one; four self-police and one does not. CONFIRMED.**

`SUBSET_FILES` (or an inline equivalent) hardcodes `["phase-4a.txt", "phase-4b.txt",
"phase-4c.txt"]` at `corpus.rs:470`, `coverage.rs:513`, `ir_dual.rs:1181`, `collect_stress.rs:137`
and `trace_oracle.rs:661`. The first four each carry a `phase_subset_files_on_disk()` read of the
corpus directory and an assertion against it, so adding `phase-5.txt` reddens them until wired in --
good. **`trace_oracle.rs:661` has no such guard.** The spec naming only `corpus.rs` is
**implied (b)** for the four self-policing sites and **genuinely missing (a)** for the fifth.
Note that `coverage.rs` is where criterion 1's parse-coverage witnesses come from, so a Phase 5
variant moving `InScope` demands a witness the tool will look for in the 4a/4b/4c union.

**M11. Criterion 2's "the four `directive_gap` over-refusals this phase retires" names a set that
does not exist. CONFIRMED.**

`directive_gap` has seven gap arms: `::ROUTINE EXTERNAL`, `::METHOD EXTERNAL`, `::ATTRIBUTE
EXTERNAL` (Phase 7) and `::REQUIRES`, `::OPTIONS`, `::CLASS naming another class`, `::ANNOTATE
naming a target` (Phase 5). Three problems with the phrase:

* `phase-4-exclusions.txt` names **two** deliberate over-refusals -- `::CLASS NAMING ANOTHER CLASS`
  and `::REQUIRES` -- and says of the third, *"`::OPTIONS` IS REFUSED FOR THE SAME REASON AND WITHOUT
  THE TRADE ... There is no unused `::OPTIONS`."* `::ANNOTATE naming a target` is in that file's
  *"REFUSED BY THE ORACLE"* list (99.945 rc 157), so it is not an over-refusal either.
* D31 removes `::OPTIONS` from this phase, so the spec's own decisions cannot retire four.
* The spec never says which four it means, in a criterion that has to be checkable.

**Genuinely missing (a)**, and it is a gate criterion whose subject is unidentifiable.

---

### LOW

**L12. The mixin enumeration is short by one and mixes two different things; criterion 2 rests on
it. CONFIRMED.**

The spec writes *"`Comparator` and its six subclasses"*. Enumerated from the file's own `::CLASS`
directives, `Comparator`'s `MIXINCLASS Comparator` subclasses are `DescendingComparator`,
`CaselessComparator`, `CaselessDescendingComparator`, `ColumnComparator`, `InvertingComparator`,
`NumericComparator` and `CaselessColumnComparator` -- **seven**, and the spec names none of them.
Criterion 2 requires *"one program per mixin in `CoreClasses.orx`"*, so an unnamed set with a wrong
count under-specifies the gate. (This is also the one place the spec breaks the project's own "name
a set, never its size" rule, and it broke on the first count.)

Separately, the spec calls `SupplierMixin`, `ManyItemMixin`, `SetMixin` and `BagMixin` mixins. Read
at the file, none of the four carries the `MIXINCLASS` keyword (`::class "SupplierMixin"`,
`::class "ManyItemMixin"`, `::CLASS 'SetMixin'`, `::CLASS 'BagMixin'`); they are plain classes
consumed by `~inheritInstanceMethods` in the prolog. So "one program per mixin" is ambiguous by
exactly those four.

**L13. `::class "Singleton" mixinclass class` is a mixin on the *class* behaviour, and the flattened
-dictionary section covers only instance behaviour. CONFIRMED.**

Read at `CoreClasses.orx:3974`. `MIXINCLASS class` inherits into the metaclass side, which is
`RexxClass::createClassBehaviour`, not `createInstanceBehaviour` -- the only one the spec cites.
`inheritInstanceMethods` in the prolog is the other half of the same distinction. **Implied (b)** at
best; the spec's own risk row ("the flattened dictionary is built as a chain walk") mitigates with
*"a mixin diamond in `phase-5.txt`"*, which does not exercise the class-behaviour side.

**L14. `PlatformObjects.orx` already runs to completion, so one third of criterion 1 cannot fail.
CONFIRMED.**

The spec's open question says *"it is small, and nothing has read it yet."* I read it:
`interpreter/platform/unix/PlatformObjects.orx` is one line, `-- Nothing to do currently`. And
`rexx-run` on it today exits 0 with empty stdout and empty stderr. The Windows file is
`call 'orexxole.cls'`, which this project does not measure. Criterion 1's third conjunct is
satisfied at the tree as committed.

**L15. The registry section never enumerates what `.environment` and `.local` must hold, and some of
it is Phase 7's. CONFIRMED.**

Enumerating `.NAME` symbols from the two `.orx` files (comment- and string-stripped) gives, among
others: `.FILE`, `.STREAM`, `.REXXQUEUE`, `.STREAMSUPPLIER`, `.STDIN`, `.STDOUT`, `.STDERR`,
`.INPUT`, `.OUTPUT`, `.ERROR`, `.DEBUGINPUT`, `.TRACEOUTPUT`, `.METHODS`, `.CONTEXT`, `.LOCAL`,
`.ENVIRONMENT`. **`.File` is explicitly Phase 7's**: D11 records *"`.File` is resolved: it **is** an
environment class ... so Phase 7 owns it alongside the file-system `Sys*` calls."* D33 makes
`.environment`/`.local` real without saying what goes in them or which entries another phase fills,
so the Phase 5/Phase 7 line inside the registry is undrawn. **Genuinely missing (a)**, though a small
one given D25's "discovered" method.

**L16. D32 leaves some `CoreClasses.orx` classes definable but unexercisable, and the spec does not
say which. PLAUSIBLE.**

`guard`/`reply` clauses in `CoreClasses.orx` sit at lines 1554-1555, 1560, 1606 (`Alarm`) and
1662-1663, 1670, 1676 (`Ticker`). D32 gives both instructions their translation-time check only and
sends the run-time half to Phase 6. Criterion 1 is unaffected (directives install, bodies do not
run), but any criterion-2 program that instantiates `Alarm` or `Ticker` cannot pass in this phase.
The spec never lists which classes Phase 5 leaves unexercisable. I did not run such a program, hence
PLAUSIBLE.

**L17. The roadmap's stated architectural justification for the IR is a send-dispatch claim that D28
declines. PLAUSIBLE, and addressed to the human partner as a challenge to a settled call's
*reasoning*, not to the call.**

Roadmap `:482`: the IR's reasons *"are prior and architectural: **it founds OO dispatch for Phase 5
by making a call site a patchable slot**"*. D28 says no send goes through `CallSite`. D28's own
argument is good and I do not dispute the decision; but the spec never notes that it retires the
roadmap's first-listed reason for the IR's existence, and `:482` will now read as a claim nothing
delivers. The plan should either amend `:482` or record why the sentence survives.

**L18. `SysFileTree` dropped from the restatement of the L2 blocker. CONFIRMED.**

The spec: *"the ooTest framework cannot start without `SysFileExists` and `.File`, which are Phase
7's."* The roadmap `:304`: *"the suite cannot start ... without `SysFileExists` and `.File`, and the
framework's own runner additionally needs `SysFileTree`."* Immaterial to the decision (L2 is reported,
not gated) -- recorded for accuracy only.

---

## Checked and correctly out of scope (c)

* **The internal-package routines** `BEEP`, `FILESPEC`, `DIRECTORY`. `phase-4-exclusions.txt` is
  explicit: *"NO OWNER. This is not a builtin gap, so Phase 7's file-and-stream rows do not cover it;
  it is not dispatch, so Phase 5's do not either."* The spec is right to be silent.
* **The time-zone database.** The `DATE`/`TIME` `KNOWN GAP` row says *"a time-zone database is Phase
  5-or-later infrastructure"* -- not assigned to Phase 5. Silence is defensible; a one-line
  disclaimer would be better, since "Phase 5-or-later" is the kind of phrase a later reader converts
  into "Phase 5's".
* **The large-result / `ulimit -v` reservation causes.** Explicitly *"UNASSIGNED ... not a Phase 5 or
  Phase 7 feature"*.
* **`corpus/keyword-exempt.txt`.** Its header states *"No body here is blocked by Phase 5"*; I read
  every row and confirmed the `unblocked_by` column contains only `Phase 7`, `4c` and `RAISED`.
* **`>I>`/`<I<` for `::METHOD`.** `phase-4-exclusions.txt` notes *"`::method` travels with
  `::routine` here ... but is Phase 5's by the object model"*; `PREFIX_COVERAGE` already carries both
  as `WitnessedLive`, so the prefixes exist and only the method route is new. The spec's trace section
  is complete on the *prefix* axis: I enumerated `PREFIX_COVERAGE` from the table itself, and `>M>`
  and `>N>` are the only two `Owned("Phase 5")` rows.

---

## Small corpus omission

**Three programs are parked for Phase 5 in the corpus directory and in no subset file.**
`corpus/num/digits_rounding.rex`, `corpus/num/exponential.rex` and `corpus/num/operators.rex`.
`corpus/phase-4a.txt`'s header: *"They stay in `corpus/num/` for Phase 5, once `List` exists"*, and
`corpus/README.md` repeats it. D34 makes `List` a real Array; nothing in the spec admits these three
to a subset, and a corpus program in no subset file is compared against nothing. CONFIRMED by reading
both files. **Genuinely missing (a)**, trivially fixed.

---

## Cross-check against the two lists I withheld

I read `2026-08-15-phase-5-inherited-surface.md` only after finishing, and the result splits my
findings in two, which changes what each one means.

**Raised by the survey and dropped by the spec.** These are not things nobody knew; they are things
the evidence half recorded and the decision half did not carry:

* H1, the security manager. Survey `:49` quotes the gate row including it, `:85`-`:91` restates D12's
  Phase 5 half, and Q14 asks what shape the interception design takes. The spec answers Q1-Q13 and
  leaves Q14 in "Open questions".
* H3's measurement. Survey `:183` carries the same `value('.LOCAL')` transcript. The *misnomer* also
  originates there: `:617` writes "`VALUE`'s empty-selector form" for the same two-argument case
  while `:743` uses "empty-selector branch" correctly for `Loud::value_selector`'s branch (a). The
  spec inherited the wrong one of the two usages.
* H4. Survey `:194` records the `Compiled method "+" with scope "String".` line verbatim.
* M6's `EXPOSE`/`FORWARD`. Survey `:149`-`:150` measure both refusals and `:159` classes them as
  execution-time, against `REPLY`/`GUARD`'s translation-time. The spec decided `REPLY`/`GUARD` (D32)
  and said nothing about the other two.
* M7. Survey `:463`-`:471` quotes the `Role::Offset` caveat and roadmap `:474` including `startup`,
  and `:716` lists among its open items *"a decision on `startup`, which stops being 'not comparable'
  the moment a bootstrap exists"*. D35 does not make that decision.
* M9's `assertions.rs` (`:402`) and the `UNATTRIBUTED:an environment symbol` rows (`:363`).
* M11. Survey `:216` is headed *"The two declared over-refusals"*, and Q7 asks *"In what order are
  the four `directive_gap` over-refusals retired, **and which are actually Phase 5's**?"* The spec's
  criterion 2 answers with "the four ... over-refusals" and never distinguishes them.
* The parked `num/` programs (`:415`).
* H2's origin: survey `:45`-`:49` quotes the Phase 5 gate row (which names `CoreClasses.orx` alone)
  and `:52` then says *"The three assets that gate names"*, adding `StreamClasses.orx` and
  `PlatformObjects.orx`. The spec's criterion 1 is that sentence, not the gate row.

**Absent from both.** These my passes found independently: `.context` and `.methods` and the Package
methods the prolog needs (H5) -- neither string occurs in the survey; `::CONSTANT` (M8) -- neither
spelling occurs; the content of `PlatformObjects.orx` (L14); `Comparator`'s subclasses (L12); and the
five harnesses that hardcode the subset list (M10) -- `SUBSET_FILES` does not occur in the survey.

---

## Count by severity

| severity | count |
|---|---|
| HIGH | 5 (H1-H5) |
| MEDIUM | 6 (M6-M11) |
| LOW | 7 (L12-L18) |
| corpus | 1 |

Confirmed: H1, H2, H3, H4, H5, M6, M7, M8, M9, M10, M11, L12, L13, L14, L15, L18, and the corpus
item. Plausible (reasoned, not run): L16, L17.
