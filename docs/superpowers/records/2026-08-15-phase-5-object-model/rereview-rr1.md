# RR1 -- re-review of the revised Phase 5 spec

Re-reviewer RR1. Two parts, as briefed: Part 1 adjudicates every blocking/HIGH/Critical finding from
R1-R5 against the revised spec; Part 2 hunts for false statements the revision **introduced**.

Every oracle run below used the mandated wrapper
(`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx FILE )`) from a fresh empty
directory under the session scratchpad, with absolute paths, reading stdout, stderr and status as
three separate descriptors. Our side is `rust/target/release/rexx-run` as staged. Every count and
every exhaustive search used `/bin/grep -a`. Nothing in the repository was modified except this file.
No program from `rust/corpus/oracle-crashes.txt` was run (I read the file's five entries first;
`CoreClasses.orx` is not among them).

**Headline counts.** Blocking findings adjudicated: **48**. ADDRESSED **27**, PARTIALLY ADDRESSED
**16**, NOT ADDRESSED **5**, WITHDRAWN-CORRECTLY **0**. Newly-introduced false or materially wrong
statements found: **15** (12 CONFIRMED, 3 PLAUSIBLE).

---

# Part 1 -- adjudication of every blocking finding

## R1 (facts) -- ten HIGH

| id | verdict | basis |
|---|---|---|
| **H1** criterion 1 needs Phase 7 work | **ADDRESSED** | Criterion 1 is rewritten to *"`CoreClasses.orx` translates, installs, and its prologue runs to `exit`"* with *"`StreamClasses.orx` and `PlatformObjects.orx` to **install**"* and *"**Executing `StreamClasses.orx`'s method bodies is Phase 7's**"*. D37 registers every `LIBRARY REXX` name; D38 takes `::CONSTANT`. **Re-verified:** `directive_gap`'s `DirectiveKind::Method(m) if m.external.is_some()` arm is `gap("::METHOD EXTERNAL", "Phase 7")` (`rexx-exec/src/lib.rs`). **CONFIRMED.** But see Part 2 **N1** -- the rewritten criterion is unsatisfiable for a different reason the revision did not see. |
| **H2** REPLY/GUARD are run-time | **ADDRESSED** | D32 restated: *"Phase 5 implements the run-time legality check, reached only when the instruction executes, exiting 157"*. **Re-measured all three transcripts:** `if 1=0 then reply` rc 0 prints `reached`; `if 1=0 then guard on` rc 0 prints `reached`; `say "before"; reply` rc 157 stdout `before` then `Error 99.919`. Mechanism re-read: `reportException(Error_Translation_reply)` at `ReplyInstruction.cpp:72`, inside `RexxInstructionReply::execute` which begins at `:66`. **CONFIRMED.** |
| **H3** D24 misquoted | **ADDRESSED** | The spec now quotes D24 in full with the caching clauses and says *"D28 below amends half of it"*; D28 says *"**This amends D24**"*. **Re-verified verbatim** at `2026-08-08-phase-4e-ir-design.md:495`. The five forward constraints at `2026-08-09-phase-4e-ir.md:1141` are now listed and three assigned to this phase. **CONFIRMED.** |
| **H4** four-answer wiring assertion is blind | **ADDRESSED** | `~superClasses` added to D25, criterion 3 and the risk row: *"`~superClasses` is not optional -- without it the assertion cannot see a missing mixin edge"*. **Re-measured on the oracle:** `.DateTime~superClass~id` = `Object`, `.Array~superClass~id` = `Object`, `.DateTime~superClasses~makestring("LINE", ",")` = `The Object class,The Comparable class,The Orderable class`, `.Array~superClasses~...` = `The Object class,The OrderedCollection class`. **CONFIRMED.** |
| **H5** the prologue is model wiring and is never mentioned | **PARTIALLY ADDRESSED** | A whole new section, "The prologue is the object-model workout", with a twelve-row table. Eleven of the twelve rows check out against `CoreClasses.orx:39-126` (I read the range). What is left: the second `DO OVER` form at `:70-72` is missing (Part 2 **N14**); two rows name methods that do not exist in any restored image and the table does not say so (**N2**); and the spec still calls a native port *"a category error"* while arguing the prologue **is** model construction (**N13**). |
| **H6** the interface cannot express `FORWARD CLASS(SUPER)` | **ADDRESSED** | `resolve(receiver_behaviour, name, start_scope, per_object) -> MethodId`, plus *"the flattened dictionary must **retain scope ordering**"* in D29. **Re-verified every citation:** `ObjectClass.cpp:866` is the ordinary `messageSend` (its `behaviour->methodLookup(msgname)` is at `:871`); `:919` is the scope-override overload calling `superMethod(msgname, startscope)` at `:929`; `RexxBehaviour.cpp:584` is `superMethod` delegating to `methodDictionary->findSuperMethod`; `MethodDictionary.hpp:98-100` carries `instanceMethods`, `scopeList`, `scopeOrders`. **CONFIRMED.** |
| **H7** `::OPTIONS`'s effect is not digits/form/fuzz | **ADDRESSED (withdrawal correct)** | The spec now says *"**One supporting sentence from the first draft is withdrawn as false**"* and enumerates 14 subdirectives. **Re-enumerated from the tool's own definition:** `LanguageParser::optionsDirective` runs `DirectiveParser.cpp:948-1367`, and its `case SUBDIRECTIVE_*` labels are DIGITS, FORM, FUZZ, TRACE, NOVALUE, ERROR, FAILURE, LOSTDIGITS, NOSTRING, NOTREADY, ALL, NOPROLOG, PROLOG, NUMERIC -- exactly the spec's list. The original claim really was false. **CONFIRMED.** (Note R1's own gloss *"each with its own `// ::OPTIONS X` comment"* is wrong: only ten carry one.) |
| **H8** criterion 2 vs D31 | **ADDRESSED** | D31 reversed -- `::OPTIONS` stays. Criterion 2 now enumerates the arms by name rather than by cardinal. **CONFIRMED** against `directive_gap`, whose Phase 5 arms are `Requires`, `Options`, `Class`-naming-another-class, `Annotate`-naming-a-target. |
| **H9** "the floor" | **PARTIALLY ADDRESSED** | The mis-citation is gone and a rule is stated (*"under 1% ... is not a finding"*), which is a real improvement. Two halves are left. (a) **The instrument is still unnamed.** `bench-baselines/README.md`'s own column table says *"every figure is emitted on both"* `instructions:u` and `cycles:u`; R1 and R3 both asked for the instrument and the revision does not say. (b) **1% is below the measured false-positive amplitude on two of the six guarded axes.** `phase-4f-record.md:2131-2136` (entry 17) records a null control -- *"By construction it does what BASE did"* -- moving `varlookup` +6.98%/+6.78% and `emptyloop` +2.37%/+3.07% while instruction counts across three builds spread 0.0001%-0.0863%. So on cycles a 1% floor fires on layout; on instructions it is ~12x the null-control ceiling. The rule as written cannot be applied without naming which. |
| **H10** `.File`/criterion 1/L2 disagree | **ADDRESSED** | Criterion 1 no longer requires `StreamClasses.orx` to run, so `.File` stays Phase 7's. **CONFIRMED** against roadmap `:445` (Phase 7 row, *"`StreamClasses.orx` runs"*) and `:443` (Phase 5 row, which names `CoreClasses.orx` alone). |

## R2 (gate) -- seven HIGH

| id | verdict | basis |
|---|---|---|
| **G1** "byte for byte ... stderr" is false | **ADDRESSED** | Criterion 2 now carries *"`corpus.rs`'s comparison runs stderr through `normalize_stderr`. ... the subset needs an unnormalised comparison mode, or the criterion is weaker than it reads. **Build the mode.**"* **Re-verified** the call site: `tests/support/oracle.rs:302` compares `normalize_stderr(&rust.stderr) != normalize_stderr(&cpp.stderr)`, reached from `corpus.rs:285`. **CONFIRMED.** |
| **G2** `ir_dual` cannot see indent | **ADDRESSED** | *"`ir_dual` is not evidence here."* **Re-verified the mechanism:** `crates/rexx-exec/src/trace.rs` holds `push_indent`/`push_prefixed_blanks` and is used from both `run.rs` and `ir.rs`, so a shared indent defect is identical on both arms. **CONFIRMED.** |
| **G3** `coverage.rs` panics on every subset program | **ADDRESSED** | Criterion 2: *"`coverage.rs`'s `assert_program_has_only_routine_directives` ... panics on any program carrying `::CLASS` or `::METHOD` ... **Widen the walker.**"* **Re-verified:** defined at `coverage.rs:151`, applied at `:753`; the filter is `*keyword != "ROUTINE"`. **CONFIRMED.** |
| **G4** the floor | **PARTIALLY ADDRESSED** | Same as R1 H9. |
| **G4b** `emptyloop` unguarded / `rexxcps` unpinned | **ADDRESSED** | D35's guard is now `alloc4c, arith, compound, emptyloop, strings, varlookup`. **Re-verified from the tools' own definitions:** `awk -F'\t' 'NR>1{print $3}' bench-baselines/pre-phase-5-arms.tsv | sort -u` gives exactly those six; `ls bench-programs/` holds no `rexxcps`; `AXES` in `rexx-bench-suite.rs` gives `Role::Loop` to exactly those six. **CONFIRMED.** One overstatement introduced in the process -- Part 2 **N11**. |
| **G5** the mixin-diamond witness cannot fail | **ADDRESSED** | Criterion 2 now requires *"one diamond that **discriminates merge order from a chain walk**"*, and the risk row says *"the discriminating diamond"*. **CONFIRMED** as a text fix; R2's own oracle measurement (`from M1`) established the requirement. |
| **G6** the deferral table has no external third list | **NOT ADDRESSED** | Criterion 7 still reads *"Enforced as a test in `rexx-classes` over its own registry, so the table cannot drift from the code."* That is two in-repo lists, which is exactly the shape R2 and R3 I9 named; `registry ∪ deferral` compared against a hand-written `Setup.cpp` transcription is one edit away from green. The tree's own answer (`rexx-inventory/build.rs` deriving from the C++ at build time, `cargo::rerun-if-changed` on each input) is cited **elsewhere in the same spec**, for the `.orx` files, and not here. **CONFIRMED** by reading criterion 7 and `rexx-inventory/build.rs:12-17`. |

## R3 (decisions) -- five Critical

| id | verdict | basis |
|---|---|---|
| **C1** wiring assertion structurally blind | **PARTIALLY ADDRESSED** | `~superClasses` added (see R1 H4). The **responds-to probe** C1 also asked for -- *"a per-class responds-to probe (`~hasMethod` over the merged name set, or `~methods`)"* -- is not in criterion 3, which remains five class-graph queries. Criterion 2's discriminating diamond covers the merge-order hazard for *one* hand-built case, not per class. C1's third observation (that `~isA` on a class object answers about the metaclass chain) is still unremarked, and criterion 3 still does not say what `~isA` takes as an argument. |
| **C2** `StreamClasses.orx` needs Phase 7's natives | **ADDRESSED** | D37 + criterion 1's "install" scope. |
| **C3** `CoreClasses.orx` needs Phase 6's natives | **ADDRESSED** | The spec names the five entry points. **Re-verified:** `/bin/grep -ain "external" CoreClasses.orx` returns exactly `:1590 alarm_startTimer`, `:1618 alarm_stopTimer`, `:1690 ticker_createTimer`, `:1691 ticker_waitTimer`, `:1692 ticker_stopTimer`. **CONFIRMED.** The eager-bind measurement reproduces: a file whose prologue is `say "prolog ran"` with `::method zz external 'LIBRARY REXX nosuchentry'` is oracle **rc 166, stdout empty**, `Error 90.998: Unable to find external method "nosuchentry"`; the same file naming `alarm_startTimer` is **rc 0** and prints. |
| **C3b** `CoreClasses.orx` uses REPLY/GUARD inside methods | **PARTIALLY ADDRESSED** | The spec now says *"the plan must say what a Phase 5 method containing a `REPLY` does -- this spec does not, and that is an open question below"*. C3b's first consequence is still live and is not an open question but a criterion-1 prerequisite: the legality check must **accept** `REPLY`/`GUARD` inside a method or `CoreClasses.orx` does not install, and D32 specifies only the refusal. `GUARD ON WHEN <expr>`'s parse-and-scope requirement is still unmentioned. |
| **C5** the one-way dependency is refuted by `~new` | **ADDRESSED** | *"The one-way dependency the first draft asserted does not hold ... The boundary that *does* hold is **dependency inversion at a trait**"*, with `~new` running a Rexx `init` given as the reason. **CONFIRMED** as adopted. |

## R4 (omissions) -- five HIGH

| id | verdict | basis |
|---|---|---|
| **H1** security manager dropped from the gate | **ADDRESSED** | New criterion 5. **CONFIRMED** against roadmap `:443` (the Phase 5 row does name it) and D12 at `:326-335`. One introduced inaccuracy in the restatement -- Part 2 **N10**. |
| **H2** `StreamClasses.orx runs` is Phase 7's | **ADDRESSED** | Criterion 1 rescoped, and the survey's sentence is marked-corrected in place. **Re-verified** the correction's own claims: `wc -l` gives 4,193 and 1,010; `/bin/grep -ac "^::class\|^::CLASS" CoreClasses.orx` gives 32; roadmap `:445` and `:518` say what the correction says. *"The measurements in the paragraph are sound; the ownership claim in its first six words is not"* is exactly right. **CONFIRMED.** |
| **H3** D33 names the wrong `VALUE` form | **PARTIALLY ADDRESSED** | The right gap is now targeted and the transcript block is right (I re-measured all four rows; see below). But the new label is also wrong -- Part 2 **N9**. |
| **H4** method-frame traceback divergence | **ADDRESSED** | New paragraph plus a criterion-2 item. **Re-measured:** `say b. + 1` is rc 215 on both sides; the oracle's stderr opens `       *-* Compiled method "+" with scope "String".` and this crate's does not. **CONFIRMED.** |
| **H5** the prologue's requirements | **PARTIALLY ADDRESSED** | See R1 H5 and Part 2 **N2**, **N14**. |

## R5 (readiness) -- twenty-one blocking

R5 had no shell. I re-ran its load-bearing claims. **Its factual claims about `CoreClasses.orx`'s
structure, `directive_gap`, and `owners.rs` all hold** -- I found no false statement in R5's report.
Specifically verified this session: `CoreClasses.orx`'s executable top level is `:39`-`:126`
terminated by `exit`, with the clause inventory F1 lists at the lines F1 gives; `directive_gap`
refuses at install time and enumerates directive *forms* (read in full, `lib.rs:1006-1066`);
`owners.rs:393` is `SPLIT_TABLE_PHASES: &[&str] = &["4b", "4c", "Phase 5", "Phase 7"]`, asserted at
`:409` with the message F23 quotes, and the `Owner::Phase("Phase 5")` rows are `InstructionKind::`
`Expose`, `Options`, `Message`, `Guard`, `Reply`, `Forward`; `Call::Qualified`; `ExprKind::`
`QualifiedCall`, `ClassResolver`, `List`, `Message`; and `LoopKind::With`. F12's PLAUSIBLE claim
about `--axis rexxcps` is right in both halves: `rexx-arms` resolves a non-`/` axis through
`rexx_bench::program_path`, and its module doc line 32 says *"an `--axis` entry holding a `/` is a
path"*, so the absolute path is the only spelling that could work.

| id | verdict | one line |
|---|---|---|
| **F1** discovery is an unbounded loop | **ADDRESSED** | *"The discovery is bounded by the prologue table above, and that is what makes it a task list rather than an open loop."* |
| **F2** the instrument changes character | **ADDRESSED** | Stated almost in F2's own words: *"once `::CLASS`, `::METHOD` and `::ATTRIBUTE` land, the file installs silently and every later failure arrives one clause at a time"*. |
| **F3** `CALL 'StreamClasses.orx'` vs Phase 7 | **ADDRESSED** | *"`rexx-lib` intercepts the three bootstrap names ahead of that search and never touches the search itself"*. Verified `Loud::unresolved_call`'s doc (`lib.rs:595-596`): *"External routine resolution is **Phase 7's**"*. |
| **F4** `owners.rs` is shared mutable state | **NOT ADDRESSED** | `/bin/grep -aic "owners.rs"` over the spec = **0**. The five pinned items an ownership move must edit together are named nowhere. |
| **F5** `phase-5.txt` has four `read_subset` callers | **PARTIALLY ADDRESSED** | Criterion 2 names `corpus.rs` and `coverage.rs`. **Verified there are four `const SUBSET_FILES` sites** -- `collect_stress.rs:137`, `corpus.rs:470`, `coverage.rs:513`, `ir_dual.rs:1181` -- plus the unguarded inline list at `trace_oracle.rs:661`. Three of the five are still unnamed. |
| **F6** "asserted against the oracle" has two readings | **PARTIALLY ADDRESSED** | Criterion 2 now admits *"the wiring assertion of criterion 3 for each"* into `phase-5.txt`, which picks reading A. `~isA`'s argument is still unstated. |
| **F7** `Body::Instance` has two readings | **NOT ADDRESSED** | The sentence is re-asserted verbatim, and it is wrong about the type -- Part 2 **N12**. |
| **F8** "touched an execution path" | **ADDRESSED** | Replaced by *"lands code in `rexx-exec`, `rexx-core` or `rexx-classes` -- that is the decision procedure"*. (`rexx-lib` is deliberately outside it.) |
| **F9** "at minimum" | **PARTIALLY ADDRESSED** | The floor is much longer and its mixin half is now decidable (`MIXINCLASS`). No closure rule; the spec moves the problem to an open question instead. |
| **F12** the guard command | **PARTIALLY ADDRESSED** | The axis set now matches the pinned baseline exactly. The "What a guard run is" block still shows bare flags with no binary; `rexx-arms` is named only in the value-representation section. |
| **F13** "the floor" has no value | **PARTIALLY ADDRESSED** | A value is stated (1%). The instrument is not -- see R1 H9. |
| **F14** no signatures | **PARTIALLY ADDRESSED** | `resolve(...)` and `define(...)` now have shapes, not types: whether `receiver_behaviour` is a `BehaviourId` or an `ObjRef` is the question F14 said decides whether a user-defined class can be a receiver, and it is still open. F14's `self~define("COPY", "return self")` case is untouched -- **verified at `CoreClasses.orx:3980`**, inside `Singleton~new`, a define from a string produced at run time. |
| **F15** no message texts or exit codes | **NOT ADDRESSED** | The spec still names no refusal string it retires or creates. |
| **F16** no task order; `EXPOSE`/`FORWARD`/`With` | **PARTIALLY ADDRESSED** | `FORWARD` is now mentioned (four times, all inside the `FORWARD CLASS(SUPER)` resolve argument) but decided nowhere. `/bin/grep -aic` over the spec: **`EXPOSE` 0, `LoopKind` 0, `DO WITH` 0, `Qualified` 0, `UNINIT` 0.** Still no ordering statement beyond D30/D33/D34. |
| **F17** `DO OVER` on a collection | **PARTIALLY ADDRESSED** | The prologue table has the row (*"`DO OVER` a **collection object**, not an array"*). The silent-divergence shape F17 named -- an implementation keeping the once-yielding-itself behaviour populates `.environment` with one wrong entry and criterion 1 stays green -- is not stated. |
| **F18** criterion 1 vs the open question | **ADDRESSED** | The open question is gone; the file's content is stated. |
| **F20** which red tests to expect | **NOT ADDRESSED** | `assertions.rs`, `bif-exempt.txt`, `owners.rs`, `collect_stress.rs` are named nowhere (`/bin/grep -aic` = 0 for each). |
| **F21** criterion 2 vs D31 | **ADDRESSED** | D31 reversed. |
| **F22** criterion 2 requires an unmeasured `::ANNOTATE` | **ADDRESSED** | Criterion 2 now conditions it: *"if the plan's first task finds it in scope"*. The premise it rests on is false -- Part 2 **N5**. |
| **F23** D31's owner cannot be spelled | **ADDRESSED** | D31 reversed, so `"Phase 5"` stays the owner string and `SPLIT_TABLE_PHASES` is untouched. |
| **F24** "two over-refusals" vs the survey | **PARTIALLY ADDRESSED** | The spec still says *"they delete two over-refusals"*. `phase-4-exclusions.txt:423` heads its list **"THE TWO DELIBERATE OVER-REFUSALS"** and names `::CLASS NAMING ANOTHER CLASS` and `::REQUIRES`; `:453` says *"`::OPTIONS` IS REFUSED FOR THE SAME REASON AND WITHOUT THE TRADE"*. Both readings are defensible under the file's own definition of an over-refusal (*"rejects a program the oracle runs"*), which `::OPTIONS` does satisfy -- I measured `options 'anything at all'` as oracle **rc 0** stdout `ok` against this crate's **rc 120**. The two documents still say it oppositely and neither is corrected. **PLAUSIBLE.** |

## Panel disagreements resolved by measurement

1. **R1 M1 against R3's D30 paragraph, on what `::REQUIRES` breaks.** R1 is right; R3 is wrong.
   `ir.rs`'s `CallSite` doc, read this session: *"**`Interp::routines` never rebinds a name it has
   already bound** ... **The property the table needs is that one, and not "the map is written
   once"**, which is a stronger thing that happens to be true today and would stop being the reason
   if a `::REQUIRES` or an external-file call ever installed a routine mid-run. Append-only is what
   the refusal enforces, and append-only is enough."* The comment names `::REQUIRES` as the breaker
   of *"written once"*, which it explicitly says is **not** the needed property, and it names an
   external-file call beside it. **The revision adopted R3's reading and thereby introduced a false
   statement -- Part 2 N4.** CONFIRMED.

2. **R1 ("the spec's use of the bare `OPTIONS` instruction is accurate") against R3 M3.** R3 is
   right. Measured this session: a genuinely bare `options` is oracle **rc 221** with
   `Error 35.913: Missing expression following OPTIONS keyword.`, and this crate answers `rexx-exec:
   35.913: Missing expression...` at rc 120. The over-refusal is on `OPTIONS <expression>`. The
   revision removed the sentence, so the defect is gone, but R1's clearance of it was wrong.
   CONFIRMED.

3. **R2 G15 against R3 M5, on the `.orx` build script.** R3 is right and R2's suspected CI break is
   not present. `git ls-files` names `interpreter/RexxClasses/CoreClasses.orx`,
   `StreamClasses.orx` and `interpreter/platform/unix/PlatformObjects.orx` in **this** repository;
   `sha256sum` of `CoreClasses.orx` here and in the oracle checkout are both
   `c00727ab66e0fd862f12a7c023656c948259837b8432fa0d149bd6938fe2ff47`; `rexx-inventory/build.rs:12-17`
   reads the C++ tree relatively with `cargo::rerun-if-changed`. The revision adopted this. CONFIRMED.

4. **R1 M2 / R3 M2 / R4 L12 on `Comparator`'s subclasses.** All three said seven and all three are
   right: `DescendingComparator`, `CaselessComparator`, `CaselessDescendingComparator`,
   `ColumnComparator`, `InvertingComparator`, `NumericComparator`, `CaselessColumnComparator`
   (`::class` lines `:1312`, `:1317`, `:1322`, `:1327`, `:1338`, `:1351`, `:1369`). The revision says
   seven. CONFIRMED.

---

# Part 2 -- false statements the revision introduced

Every added line in `revision.diff` was treated as an unverified claim. Findings are ordered by
consequence.

## N1. Criterion 1 has no oracle, and its two halves contradict each other. **CONFIRMED.**

**The added text:** *"1. **`CoreClasses.orx` translates, installs, and its prologue runs to `exit`,
on both engines**, byte-identical to the oracle on all three descriptors."*

**What I ran.** The oracle on a copy of `CoreClasses.orx` in a fresh empty directory:

```
rc 159, stdout empty
    47 *-* rexxPackage~addClass('LOCALSERVER', .LocalServer)
Error 97.1:  Object "REXXPACKAGE" does not understand message "ADDCLASS".
```

`use arg rexxPackage` with no argument leaves the symbol as its own name, so the prologue dies at
its first clause. Then, with all three `.orx` files in the directory and a driver that supplies a
real Package object -- `call 'CoreClasses.orx' .context~package` -- the oracle gets past the
`addClass` calls, past `.environment~objectname`, past the `publicClasses` loop, and dies at line 73:

```
rc 159, stdout empty
    73 *-*   .String~defineClassMethod(name~upper, .methods[("string_cls_" || name)~upper])
Error 97.1:  Object "The String class" does not understand message "DEFINECLASSMETHOD".
```

**The mechanism, read directly.** `RexxClass::removeSetupMethods()` (`ClassClass.cpp:923-941`), whose
own doc comment is *"Remove the special class methods that are defined just for image building"*,
deletes `DEFINECLASSMETHOD` and `INHERITINSTANCEMETHODS` from the class behaviour, the instance
behaviour, the instance method dictionary and `TheObjectClass`. It is called at `Setup.cpp:1809` --
that is, **after** `CoreClasses.orx` runs at `:1798` and **before** `saveImage` at `:1812` and
`exit(0)` at `:1814`.

**So the prologue can only reach `exit` inside `rexximage`'s `createImage()`, whose output is an
image file and whose exit is `exit(0)` with nothing on stdout.** There is no invocation of the oracle,
through the wrapper the ground rules mandate or through any driver program, in which
`CoreClasses.orx`'s prologue runs to `exit`. Therefore "its prologue runs to `exit`" and
"byte-identical to the oracle on all three descriptors" cannot both be satisfied: every reachable
oracle transcript is a traceback at rc 159.

This is the same defect class the spec's own header calls out in the first draft -- a headline
criterion that the phase cannot reach -- reintroduced in the criterion written to fix it. It is also
the concrete answer to R2 G19's *"no instrument is named"*: the instrument cannot exist in the stated
form, and the revision's new paragraph *"`createImage()` is the image-build path ... **That does not
weaken the point**"* is the sentence that let it through.

## N2. The prologue table presents two image-build-only methods as ordinary requirements. **CONFIRMED.**

Same evidence as N1. Two of the table's rows --

* *"`.String~defineClassMethod(name~upper, ...)` | a class-side method that mutates behaviour after
  definition"*
* *"`.supplier~inheritInstanceMethods(.SupplierMixin)` and four more | method donation without a
  superclass edge"*

-- name the exact two methods `removeSetupMethods()` deletes. `Setup.cpp:460` confirms the first is
installed as `AddProtectedMethod("DefineClassMethod", ...)` during setup. The table is billed as
*"the single densest statement of what the phase must build"* and does not record that these two
exist only during image build, which is a design question the plan now has to discover: whether
`rexx-classes` implements them permanently (diverging from the oracle's shipped surface -- measurable
the moment a corpus program sends `~defineClassMethod`), or reproduces the deletion.

## N3. The `EXTERNAL` section's `::CONSTANT` exit code is wrong for the file it describes. **CONFIRMED.**

**The added text:** *"a file whose prologue is `say "prolog ran"` and which carries `::CONSTANT sep
(.NoSuchClass~getThing)` exits **159** with **empty stdout**."*

**Measured, exactly that file:** oracle **rc 157**, stdout empty,
`Error 99.906: A ::CONSTANT directive with an expression requires a matching ::CLASS directive.` The
expression is never evaluated at all; the refusal is structural.

**rc 159 requires a `::CLASS` directive in the file**, which is what the `phase-4-exclusions.txt` row
the same revision added correctly states (`::CLASS K` then `::CONSTANT c (.NoSuchClass~m)`). I
measured that program too: oracle rc 159, stdout empty, stderr naming line 4 then line 3,
`Error 97.1: Object ".NOSUCHCLASS" does not understand message "M"`. The two added passages describe
different programs and quote the same number for both; only one is right.

## N4. D30's surviving reason is false against the comment it cites. **CONFIRMED.**

**The added text:** *"`ir.rs`'s `CallSite` argument does rest on `Interp::routines` being append-only
and `::REQUIRES` does break that"*, and D30: *"Its reason is the `CallSite` table's append-only
argument"*.

`ir.rs`'s doc says the opposite in as many words (quoted in full under "Panel disagreements", item 1):
append-only is what the 99.903 duplicate-`::ROUTINE` refusal enforces and *"append-only is enough"*;
the property `::REQUIRES` would break is *"the map is written once"*, which the comment says is **not**
the reason. The comment also names an external-file call beside `::REQUIRES` -- and
`CoreClasses.orx:122` and `:124` are external-file calls, which criterion 1 requires.

The withdrawal beside it is **correct**: *"the plan cache's `BodyKey` pairing does not depend on it"*
holds -- `ProgramId`'s doc (`plan.rs`) says `Interp::programs` holds an `Rc` for every id issued, so
loading a program appends a fresh key space. So the revision withdrew the false half and rewrote the
true half into a false one.

## N5. "Nothing in this project has measured what `::ANNOTATE` does" is false. **CONFIRMED.**

**The added text:** *"`::ANNOTATE naming a target` is the fourth `directive_gap` Phase 5 arm and
**nothing in this project has measured what it does**."*

Two measurements exist, one of them in the very function the sentence cites:

* `directive_gap`'s own arm comment (`rexx-exec/src/lib.rs`): *"Resolves its target against the
  accumulated package: measured, `::annotate routine nosuchrtn` is 99.945 rc 157. `::ANNOTATE
  PACKAGE` names nothing and is ignored with the rest."*
* `phase-4-exclusions.txt:418`, inside the list headed **"REFUSED BY THE ORACLE BEFORE main, stdout
  EMPTY in every case"**: `::annotate routine nosuchrtn    99.945 rc 157`.

The consequence matters beyond the sentence: because the oracle refuses it too, `::ANNOTATE naming a
target` is **not** one of the file's two declared over-refusals, so "retiring" it is not a
divergence-closing task at all. R4 M11 said this and the revision recorded the opposite.

## N6. Criterion 9's instrument is not the committed method, and the baseline forbids it. **CONFIRMED.**

**The added text:** *"**`hyperfine` is not installed on this machine**, so the instrument is
`rexx-bench-suite`'s offset line, which measures the same thing through the same wrapper on both
sides and **is already the committed method**."*

`hyperfine` really is absent (`command -v hyperfine` finds nothing; `~/.cargo/bin` does not hold it).
The rest is false:

* `perf-baseline.md:98` -- *"`rust/crates/rexx-bench/src/bin/rexx-time.rs` **is the substitute**"*,
  with the exact invocation `--warmup 10 --runs 50` and the median 5.119 ms at `:109`.
* `perf-baseline.md:113` -- *"**This is the number D2's gate compares against** -- not the criterion
  `interpreter/startup` row above."*
* `perf-baseline.md:1064-1065` -- *"**Measure D2 with hyperfine against `build/bin/rexx`, as D2 says,
  not by subtracting numbers out of this table.**"* The suite's own 5.823 ms is described at `:1063`
  as *"this suite's `/bin/sh` plus `ulimit` wrapper"*.
* `rexx-bench-suite.rs`'s `Role::Offset` doc: *"Its two sides are **not** comparable: this crate has
  no `CoreClasses.orx` bootstrap yet, so it starts fast by not doing the work the oracle does at
  startup."*

So criterion 9 substitutes the one instrument the pinned baseline the spec leans on explicitly says
not to use for D2, and calls it the committed method. (The substitution may still be the right call
once a bootstrap exists -- the wrapper cancels in a same-run delta -- but it is a decision that
contradicts a cited document, presented as a citation of it.) R4 M7's separate point rides along: the
`Role::Offset` doc becomes false the moment the bootstrap lands, `write_offset` prints it into every
report, `write_axes` subtracts the offset from **every** guarded axis, and nothing asserts any of it.

## N7. A false command result is now committed to `phase-4-exclusions.txt`. **CONFIRMED.**

**The added text** (new `::CONSTANT` KNOWN GAP row): *"Measured 2026-08-15 with the release rexx-run
built from b029abe77, whose code is HEAD's -- git diff --name-only b029abe77..HEAD names nothing
outside docs/."*

`git diff --name-only b029abe77..HEAD | /bin/grep -av "^docs/"` names
**`rust/bench-baselines/pre-phase-5-arms.tsv`** (133 insertions). The claim is inherited from the
ground rules, where it is also false, but the revision copied it out of a session-scoped brief into a
permanent tracked project document, where it will be read as a checkable fact. The substance it is
offered for -- that the binary's *code* is HEAD's code -- survives; the command result quoted for it
does not.

**The rest of that row is sound**, and I checked every claim in it: the two transcripts reproduce
exactly (oracle rc 159 / stderr naming the `::CONSTANT` line then the `::CLASS` line; ir rc 0 /
stdout `prolog` / stderr empty); the well-formed control `::constant c (.K~m)` is rc 0 with stdout
`prolog` on **both** sides; the simple form `::constant k 5` is rc 0 both sides;
`StreamClasses.orx:548-549` are `::constant separator (.File~getSeparator)` and `::constant
pathSeparator (.File~getPathSeparator)` with `:546-:547` declaring `external "LIBRARY REXX
file_separator"` and `"LIBRARY REXX file_path_separator"`. **The row is filed in the right section:**
it sits inside `KNOWN GAPS` (lines 770-2153), whose own header says *"ADDING a row needs no
amendment"*.

## N8. "Only four of its positions carry a stated reason" is false. **CONFIRMED.**

**The re-asserted text:** *"The rest of the order has no comment on it and should not be assumed
arbitrary"*, and the added open question *"only four of its positions carry a stated reason."*

Reading `createImage()` (`Setup.cpp:227-323`) end to end, the ordering comments are:

* *"Class and integer has some special stuff, so get them created first"* -- `RexxClass`, `RexxInteger`.
* *"string and object are fairly critical"* -- `RexxString`, `RexxObject`.
* **"The pointer class needs to be created early because other classes use the instances to store
  information."** -- `PointerClass`. That is an explicit, causal ordering reason, and it is a fifth
  position.
* *"Buffer also can be used for internal data"* -- `BufferClass`, arguably a sixth.

The remaining comments (*"the different collection classes"*, *"create more of the exported
classes"*, *"NOTE: The number string class lies about its identity"*) are groupings, not reasons. So
at least five positions carry a stated reason and the sentence "the rest of the order has no comment
on it" is false. **The class list and its order are otherwise exact** -- I enumerated
`/bin/grep -an "createInstance()" interpreter/memory/Setup.cpp` independently and the 31 names match
the spec's block name for name and position for position.

## N9. "`VALUE`'s two-argument form" is the wrong name for the measured program. **CONFIRMED.**

The measured program is `say value('.LOCAL')` -- **one** argument. `VALUE(name, newvalue, selector)`'s
two-argument form is `VALUE(name, newvalue)`, which *writes*. The spec now commits D33, the body text
and criterion 2 to *"`VALUE`'s **two-argument** form"* and *"the two-argument `VALUE` route"*. The
correction replaced the first draft's wrong name (`empty-selector`, correctly flagged by R1 M4 and R4
H3) with a second wrong name; `phase-4-exclusions.txt:2565` uses neither, calling it *"VALUE's
local-pool read of a name that classifies as a leading-dot Literal"*. The accurate name is the
**no-selector** form.

**The transcript block itself is right.** I re-measured all four rows on both interpreters:
`value('.LOCAL')` oracle rc 0 `The Local Directory` / ours rc 0 `.LOCAL`; `value('.ARRAY')` oracle
rc 0 `The Array class` / ours rc 0 `.ARRAY`; `value('.LOCAL',,'')` oracle rc 0 `..LOCAL` / ours rc 120
`rexx-exec: VALUE's external-selector form is not implemented`; `say .LOCAL` ours rc 120
`rexx-exec: an environment symbol is not implemented`. The block's fourth row leaves the oracle column
blank under a heading that says *"both interpreters"*; the oracle answers rc 0 `The Local Directory`.
The claim that *"`phase-4-exclusions.txt`'s external-selector row records that only one of its three
branches is Phase 5's"* is **true** -- `:2611-2620` gives (a) empty selector = `.environment`,
Phase 5's; (b) `'ENVIRONMENT'` = OS variables; (c) anything else = *"only THIS branch is unambiguously
Phase 7's"*.

## N10. Criterion 5 drops one of D12's three Phase 5 hook sites. **CONFIRMED.**

**The added text:** *"D12 assigns this phase the manager object, its installation path, and the hooks
in dispatch and in `.local`/`.environment` lookup -- **both** of which are surfaces this phase
builds"*.

D12 (`2026-07-27-rust-rewrite.md:332`): *"**Phase 5** builds the security manager object, its
installation path, and the hooks that live in dispatch and name resolution -- `.local`/`.environment`
lookup **and external function resolution**."* Three hook sites, not two. The dropped one is the
awkward one: external function resolution is Phase 7's by `Loud::unresolved_call`'s own doc, which is
worth saying rather than silently deleting from a restatement of what D12 assigns.

## N11. The `rexxcps` argument is half a non-reason and half an overstatement. **CONFIRMED mechanism, PLAUSIBLE consequence.**

**The added text:** *"`rexxcps` is not an axis in `bench-programs/` and it self-calibrates its own
workload from wall clock, so it cannot serve as a regression guard at all."*

* *"not an axis in `bench-programs/`"* is **true** (`ls bench-programs/` holds `alloc4c`, `alloc`,
  `arith`, `compound`, `dispatch`, `emptyloop`, `heapshape`, `startup`, `strings`, `varlookup`) but is
  **not a reason**: `rexx-arms`' module doc line 32 says *"an `--axis` entry holding a `/` is a path"*,
  which is precisely how an absolute path into the oracle tree would be measured.
* Self-calibration is **real** -- `samples/rexxcps.rex:160-163` is
  `if total>1 | trial=2 then leave` / `count=(1%total + 1) * count`. But R2 measured that it does not
  bite today (2.1 s run, no recalibration, three `perf stat` runs spreading 0.008%). *"cannot serve
  as a regression guard **at all**"* is stronger than the evidence: the hazard is latent and
  asymmetric.

The conclusion (drop it) is right; both stated reasons are weaker than they read.

## N12. "A new kind arrives boxed behind `Body::Instance`" is wrong about the type. **CONFIRMED.**

`rexx-core/src/body.rs:113` is `Instance(Vec<(String, ObjRef)>)`, doc-commented *"A user-defined
object: its instance variables"* -- an association list, not a boxing mechanism. The doc the spec is
paraphrasing (`body.rs:130-133`) says *"widening the arena for a cold value kind should be a
deliberate act **with a boxed alternative weighed**"*, i.e. `Body::Class(Box<...>)`. R5 F7 named both
readings and their opposite costs (a linear scan on every dictionary access under one, a tripped
assertion under the other); the revision re-asserted the sentence unchanged. The two new sentences
around it are correct: `size_of::<Body>() <= 80` at `body.rs:134` really is an upper bound, and *"Phase
5 adds variants and is expected to trip this"* is verbatim.

## N13. The spec now argues both sides of "category error". **PLAUSIBLE.**

*"A native port of the file, Q1's option (b), would have been a category error"* is retained, in a
document whose next section says the prologue *"is the object-model workout"*, that *"every clause in
it is object-model machinery"*, and that it installs *"real mixin edges, and the `updateSubClasses`
cascade"*. If the file's prologue is model construction, porting it is a bad trade (4,193 lines by
hand) but not a category error. R1 H5 made this point; the revision added the evidence for it and kept
the conclusion it refutes.

## N14. Two prologue requirements are missing from the table that claims to enumerate them. **CONFIRMED.**

Read at `CoreClasses.orx:39-126`:

* **`:70-72`, the second `DO OVER`:** `do name over "nl", "cr", -` continued over two lines -- `DO OVER`
  a comma-separated **list of expressions** with a line continuation, a different construct from
  `do name over publicClasses` at `:63`. Both R4 H5 and R5 F1 named it; the table has a row for the
  collection form only.
* **`.LocalServer`, `.SupplierMixin`, `.ManyItemMixin`, `.SetMixin`, `.BagMixin` at `:47-:52` are
  environment symbols naming classes declared later in the same file with **no `PUBLIC` keyword**
  (`::CLASS "LocalServer"` at `:974`, `::class "SupplierMixin"` at `:172`, `::class "ManyItemMixin"`
  at `:218`, `::CLASS 'SetMixin'` at `:411`, `::CLASS 'BagMixin'` at `:557`). A non-public class is not
  in `.environment`, so `.NAME` resolution must consult the running package's own class table first.
  D33 says the opposite shape: *"the class registry, environment-symbol lookup and `VALUE`'s ... form
  all resolve through them"*, "them" being `.environment`, `.local`, `.context`, `.methods`.

## N15. Minor: `.context` "named nowhere in `Setup.cpp`'s `createInstance()` list". **PLAUSIBLE (misleading).**

True of the environment *symbol* and false of the class: `RexxContext::createInstance()` is at
`Setup.cpp:312` and `RexxContext` appears in the spec's own quoted class list two sections earlier.
The sentence's own conclusion is right and I confirmed it: `say .context` and `say .methods` are both
rc 120 `rexx-exec: an environment symbol is not implemented` here, against oracle rc 0 `a RexxContext`
and rc 0 `.METHODS`.

---

## The nine criteria under R2's own test

For each: a defect it trips on, and a defect in its own subject that it does not.

| # | trips on | blind to |
|---|---|---|
| 1 | today's state -- measured, `rexx-run CoreClasses.orx` is rc 120 `::CLASS naming another class is not implemented (Phase 5)` | **it cannot be satisfied at all (N1)**; and as far as it *can* be read, a `DO OVER` that yields the collection once (F17) populates `.environment` with one wrong entry and the prologue still reaches `exit` |
| 2 | a class that does not respond; the diamond; the traceback line; the `VALUE` routes; `::CONSTANT` | an indent error on `>M>`/`>N>` **until the unnormalised mode it asks for exists**; the size of the native set, which is chosen by the implementation (G13, unfixed) |
| 3 | a missing mixin edge -- now genuinely, via `~superClasses`, which I measured separates a wired from an unwired build | a wrong **merge order** inside a correct edge set: all five probes read the class graph, none reads the flattened dictionary (G25/C1) |
| 4 | a wrong `>M>`/`>N>` byte string; a stale `PREFIX_COVERAGE` row | an expected string typed rather than captured -- the criterion requires the provenance and no instrument checks it (G12) |
| 5 | a missing manager object or installation path | external function resolution, one of D12's three hook sites, now absent from the criterion (N10) |
| 6 | a real >1% regression **on instructions** | on cycles, nothing: 1% is below the null control's own +6.98% on `varlookup` (H9). The instrument is unnamed, so both readings are permitted |
| 7 | a class in neither list | a class dropped from **both** lists in one edit -- there is no external third list (G6) |
| 8 | a grown `unsafe` count or a new `deny` root | nothing; it is well formed and cheap |
| 9 | nothing. It is a measurement and says so | -- and its named instrument is the one the pinned baseline forbids for this comparison (N6) |

## Checked and found sound

Named individually, not counted. The `Setup.cpp` list and order (re-enumerated independently). The
`MIXINCLASS` classification: I checked all 32 `::CLASS` directives in `CoreClasses.orx` against the
spec's three buckets and **every name is in the right bucket** -- 17 declared `MIXINCLASS`
(`MessageNotification`, `AlarmNotification`, `Collection`, `OrderedCollection`, `MapCollection`,
`SetCollection`, `Comparable`, `Comparator` and its seven subclasses, `Orderable`, `Singleton`), 4
named "mixin" without the keyword (`SupplierMixin`, `ManyItemMixin`, `SetMixin`, `BagMixin`, at
`:172`, `:218`, `:411`, `:557`), 11 ordinary (`LocalServer`, `Monitor`, `Alarm`, `Ticker`,
`CircularQueue`, `Properties`, `DateTime`, `TimeSpan`, `ArgUtil`, `Validate`, `TraceObject`); 17+4+11
= 32, the file's whole population. `~inheritInstanceMethods` really is what the prologue uses for the
four (`:80-:87`) and `~inherit` for the rest. The prologue's "and four more" and "and fifteen more"
counts are exact (5 `inheritInstanceMethods` calls, 16 `inherit` calls). "Eight map collections onto
`MapCollection`" is exact (`:103-:110`). All five `EXTERNAL` line numbers and names in
`CoreClasses.orx`. `StreamClasses.orx:546-549`. `PlatformObjects.orx` on unix is 27 bytes, one line
(`-- Nothing to do currently`), and our crate runs it at rc 0 with both streams empty; the Windows
file is two lines calling `'orexxole.cls'`, and Global Constraint `:35` does require five platforms,
which criterion 1 still does not address. `.class~class == .class` answers `1`. `(1,)~size` is 2 and
`(1,,)~size` is 3. `bench-baselines/README.md`'s two numbers and the `arm_ratio`-carries-no-placement-
difference claim. `bench-baselines/pinned/rexx-run-pre-phase-5` exists at 14,191,256 bytes. The
survey's marked correction, in every one of its claims. `directive_gap`'s seven arms and their owner
strings. `trace_oracle.rs:627`/`:631` carrying `Coverage::Owned("Phase 5")`.

## What I searched *for*, and what those terms cannot reach

* For the prologue I read `CoreClasses.orx:36-130` in full rather than grepping, and enumerated
  `::class` lines with `/bin/grep -ain`. A requirement expressed inside a method body that the
  prologue reaches indirectly is outside that read.
* For image-build-only methods I searched `removeSetupMethods` across `interpreter/`, which found the
  one call site. A method restricted by a different mechanism -- a private flag, a scope check --
  would not appear in that term, so **I cannot claim `defineClassMethod` and `inheritInstanceMethods`
  are the only two prologue operations missing from a restored image**; they are the two that
  `removeSetupMethods` deletes and the ones the oracle actually stopped on.
* For the guard I read `AXES`, `rexx-arms`' argument handling, `bench-programs/`, the pinned TSV's
  distinct `axis` values, and `bench-baselines/README.md` in full. I ran **no** performance sitting;
  H9/N11 rest on reading and on `phase-4f-record.md` entry 17, which I quote.
* For the ownership tables I read `owners.rs` and grepped the spec for each file name. A pinned table
  under a name I did not grep would be missed.
* I did not run the workspace test suite and did not rebuild anything.
