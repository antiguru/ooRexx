# Task 15 review: `::METHOD` and `::ATTRIBUTE`'s option surface

Reviewed at `dd01df2b4`. Diff `979522f74..dd01df2b4`, six commits.

## Verdicts

* **Spec compliance: PASS.** All three "Done when" clauses were verified independently, on both
  engines, from a fresh empty directory with both sides bounded. Commands and output below.
* **Task quality: CHANGES REQUESTED.** One arm of the new installer is wrong against the oracle's
  own source and the comment beside it cites that source for the opposite of what it says; four
  further statements about coverage and measurement are false in the report or in doc comments,
  three of them in the safe direction and one in the unsafe one.

## What I verified for the spec verdict

Every probe ran from `/tmp/.../scratchpad/t15probe` or `.../t15dd`, absolute paths, three
descriptors read into separate files and compared with `cmp`, never `2>&1`. Oracle under
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib timeout -s KILL 10 .../build/bin/rexx FILE )`;
crate under `( memcap 1G env REXX_ENGINE=<engine> timeout -s KILL 10 target/release/rexx-run FILE )`.

**The three live rows, byte-identical on both engines:**

```
### a1  ORACLE rc=0        (.K~a = 5 / say .K~a ; ::method a class attribute)
  out| 5
--- ir: MATCH
--- tree-walker: MATCH
### a2  ORACLE rc=0        (.K~b = 7 / say .K~b ; ::attribute b class)
  out| 7
--- ir: MATCH
--- tree-walker: MATCH
### a3  ORACLE rc=163      (say 'installed' / say .K~m ; ::method m class abstract)
  out| installed
  err|      2 *-* say .K~m
  err| Error 93 running .../a3.rex line 2:  Incorrect call to method.
  err| Error 93.965:  Method M is ABSTRACT and cannot be directly invoked.
--- ir: MATCH
--- tree-walker: MATCH
```

**The six agreeing rows still agree**, on both directives and both engines. I ran all 25 committed
`::METHOD`/`::ATTRIBUTE` table D probes through the same harness: `PUBLIC`, `PACKAGE`, `GUARDED`,
`UNGUARDED`, `PROTECTED` and `UNPROTECTED` are MATCH on `method__*` and on `attribute__*`. The
`gate_table_d` listing agrees, and each of those probes now sends a message rather than printing
`'main'`.

**The control reddens the row the brief names.** Control A, `install_attribute`'s
`AttributeStyle::Both` arm reduced to `vec![(upper, generated)]`:

```
REXX_CORPUS_GATE=1 REXX_PHASE_GATE=5a cargo test --release -p rexx-exec --test gate_table_d
  diverge-both loud=no 5a ::ATTRIBUTE CLASS subkeyword ... attribute__class__subkeyword.rex
  gated by this run: 14 row(s)          (7 at dd01df2b4)
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast
  5 in-crate tests FAILED, corpus 171 of 177, plus
  collect_stress::the_l0_subset_passes_again_under_collect_on_every_allocation FAILED
```

That is the `::ATTRIBUTE CLASS` row, `agree` to `diverge-both`, and not something adjacent.
Control B, `install_method`'s `attribute` arm reduced the same way, reddens
`method__attribute__subkeyword.rex` (gated 7 to 8), three in-crate tests and
`lang/method_attribute_generated.rex`. Both mutations were reverted; `git status` is clean at
`dd01df2b4` and `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` is back to
98 `test result: ok` with 177 of 177.

## Findings

### 1. `DELEGATE` under `ATTRIBUTE` installs one name where the oracle installs the pair, and the comment cites the oracle's source for the opposite (prose defect, over a pre-existing behaviour divergence)

`install_method` (`rust/crates/rexx-exec/src/lib.rs:4110`) carries:

```
// `methodDirective`'s own order of precedence over the generating
// options (`parser/DirectiveParser.cpp:831`-`:915`): `DELEGATE`
// first, and it installs the plain name **alone** even under
// `ATTRIBUTE`; then `ATTRIBUTE`, which installs the pair; then
// `ABSTRACT` on its own.
```

`parser/DirectiveParser.cpp:826`-`:848` says the opposite, and says why:

```
// handle delegates first.  A delegate method can also be an attribute, which really
// just means we produce two delegate methods
if (delegateName != OREF_NULL)
{
    ...
    if (isAttribute)
    {
        RexxString *setterName = commonString(internalname->concatWithCstring("="));
        checkDuplicateMethod(setterName, isClass, Error_Translation_duplicate_method);
        createDelegateMethod(setterName, retriever, isClass, accessFlag, protectedFlag, guardFlag, true);
    }
    createDelegateMethod(internalname, retriever, isClass, accessFlag, protectedFlag, guardFlag, isAttribute);
    return;
}
```

Measured, `::method a class delegate p attribute` beside `::attribute p class`:

```
### d1  (.K~a = 5 / say 'stored')          ORACLE rc=159
  err| Error 97.1:  Object "P" does not understand message "A=".
--- ir: DIVERGE rc=159   Error 97.1:  Object "The K class" does not understand message "A=".
--- tree-walker: DIVERGE rc=159   (same)
```

Same rc, wrong receiver in the report, and **not loud** -- the crate never installed `A=`, so the
send is an ordinary name miss on the class. The getter half is fine: `say .K~a` is a rc-120 loud
refusal naming Phase 5, which is correct while `DELEGATE` is 5b's.

The wrong answer itself is pre-existing: at `979522f74` `install_method` minted the upcased plain
name and nothing else for every `::METHOD`, so `A=` was absent there too. What is new is the
citation. This task rewrote the installer specifically to model `methodDirective`'s precedence and
recorded a claim about the C++ that the C++ contradicts, which is what licenses the one-name arm.
Nothing in the corpus or in table D covers the `DELEGATE ATTRIBUTE` combination, so nothing would
have caught either the arm or the claim.

The other three arms check out against the same function: `isAbstract && isAttribute` creates the
abstract pair (`createAbstractMethod(internalname, ...)`, `createAbstractMethod(setterName, ...)`)
and the crate records `Abstract` on both halves; `externalname && isAttribute` creates a native
pair and is unreachable here because `directive_gap` refuses `::METHOD ... EXTERNAL` at install
(measured: `method__external__subkeyword.rex` is rc 120 with empty `stdout`).

### 2. "no test asserts the separation" is false, and I proved the assertion live (prose defect; one test-level gap remains and is enforceable)

The report's last concern says `GeneratedMethod`'s doc carries the measurement and "no test asserts
the separation". A test written in this same task does:

```rust
assert!(
    interp.method_bodies.is_empty(),
    "a generated accessor is not a row of the body table"
);
```

in `tests::a_method_attribute_installs_a_getter_and_a_setter`. I ran the exact scenario the concern
names -- "folds `generated_methods` back into `method_bodies` for tidiness" -- as a
behaviour-preserving mutation: `record_method_body` inserts every generated method into
`method_bodies` as well, and `Interp::invocable`'s two lookups are swapped so the generated table
is consulted first. `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast`:

```
test tests::a_method_attribute_installs_a_getter_and_a_setter ... FAILED
test result: FAILED. 720 passed; 1 failed
177 of 177 matching
```

Exactly one catcher, and the corpus stays green because behaviour is unchanged -- which is the
right shape for a perf-only constraint. So the separation is test-enforced.

Two halves genuinely are not, and the report does not distinguish them from the half that is:

* **The lookup order.** My swap was invisible to the entire workspace. Order is unobservable in
  behaviour because the two tables are disjoint by construction, so no test can see it. The doc
  comment `// **After the body table and never before it**` is the only defence and there is no
  feasible mechanical one. Say so rather than folding it into a general claim.
* **A field added to `InstalledMethodBody`**, the report's other named risk. This one *is*
  enforceable in one line with an idiom already used twice in the same crate:
  `const _: () = assert!(size_of::<Op>() == 16);` (`src/ir.rs:103`) and
  `const _: () = assert!(size_of::<Argument>() == 40);` (`src/lib.rs:3368`). A
  `size_of::<InstalledMethodBody>() == 16` assert fails on precisely the change the concern names.
  Note the honest caveat: the report's own measurement says width was not the mechanism (narrowing
  the wide struct back to 16 bytes read *worse*, 26,243,886,462), so such an assert catches the
  named change without guarding the mechanism. That is worth stating beside it rather than leaving
  the assert to read as a proof of the constraint.

### 3. "The abstract refusal has no mutation witness of its own" is over-pessimistic; a gate catches it (prose defect)

I did the mutation the report declined: `crate::GeneratedKind::Abstract => Err(Raised::abstract_method(name).into())`
(`src/dispatch.rs:1234`) replaced by `Ok(None)`.

```
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast
  test dispatch::tests::an_abstract_send_is_refused_at_the_send_naming_the_message ... FAILED
  test result: FAILED. 720 passed; 1 failed
  176 of 177 matching
    [UNCLASSIFIED] lang/method_abstract_send.rex: stderr, exit code differ
```

Two witnesses, one of them a committed corpus program inside the five gates. Had the raise been
absent the corpus gate would have failed with a named program, not passed quietly. The report's
"the mutation was not done" is honest, but the concern it produces -- carried forward into the
plan's record -- states a gap that does not exist. `tests::an_abstract_accessor_pair_is_abstract_on_both_halves`
did *not* move, correctly: it asserts the install-time recording, not the send.

### 4. "the other seven axes at 1.000000" and "no layout movement on them at all" are contradicted by the report's own table (prose defect; the conclusion survives)

Re-derived from `rust/bench-baselines/phase-5a-arms.tsv`, `instrument` retained in every
projection, `task=15`, `scope=across_builds`, `instrument=instructions:u`:

```
axis           arm  size   pinned>base  pinned>head   head/base
dispatchclass  tw   small  1.012110     1.012583      1.000467
dispatchclass  ir   small  1.011789     1.012258      1.000464
dispatchclass  tw   large  1.012133     1.012597      1.000458
dispatchclass  ir   large  1.011804     1.012276      1.000466
rexxcps        tw   small  1.015410     1.015420      1.000010
rexxcps        ir   small  1.018601     1.018627      1.000026
emptyloop      ir   small  0.992023     0.992022      0.999999
strings        ir   small  1.013410     1.013409      0.999999
(alloc4c, arith, compound, varlookup: 1.000000 on every arm and size)
```

Every figure in the report's table matches the TSV. But four axes read exactly 1.000000 on both
arms, not seven: `rexxcps` reads 1.000010 and 1.000026, and `emptyloop`/ir and `strings`/ir read
0.999999. The report prints those numbers itself two paragraphs above the sentence that denies
them.

The quantitative conclusion holds and is worth restating correctly, because the corrected version
is still strong. `rexxcps` neither declares a class nor sends a message (verified: no `::CLASS`,
no `~` in `bench-rexxcps/rexxcps.rex`), so its 26 ppm is a layout term and it bounds the layout
term at 26 ppm. `dispatchclass` moved 464-467 ppm, roughly 18x that, on the one axis whose loop
is a class-method send. And the movement is inside the arm's resolution by three separate bounds:
2.92 (tw) and 2.98 (ir) `instructions:u` per pass, against the 34 the dispatch brief records, the
~13 from the byte-identity control in `1d87d90cc`, and this sitting's *own* base-arm round span --
`value_min`/`value_max` for `task=15 dispatchclass base per_pass instructions:u` are
6496.359/6556.351 (tw) and 6387.394/6443.389 (ir), a 56-60 instruction spread per pass, with
head's minimum below base's minimum on both arms. The report does not use its own min/max columns,
which are the tightest argument available and are same-sitting.

Two smaller notes on the same section. The ~13 figure is a cross-sitting span (one byte-identical
binary read 6417.398606 in one sitting and 6404.359463 in another), so citing it as this arm's
floor mixes sittings; harmless here only because the in-sitting span is wider. And the report's
"A `cycles:u` figure is not quoted anywhere above" discharges the letter of the rule but leaves
the reader unable to see that `dispatchclass` moved +5.1% to +5.9% on `cycles:u` in the same
sitting. That is defensible -- on axes that provably cannot reach the new code, `cycles:u`
`pinned>head` ranges from 0.931735 (arith/ir) to 1.036705 (strings/ir) in this sitting, a spread
wider than dispatchclass's -- but the defence belongs in the report, not in the reviewer's head.

### 5. The measurement that justifies the load-bearing design is not in the tree (test/instrument defect)

The +1.86% breach sitting at `d7bf45383`, the four attributions, and the direct `perf stat` pairs
(25,723,898,929 against 26,207,891,553; 26,243,886,462; 26,191,934,770; 25,723,986,475) exist only
as prose -- in the report and in `GeneratedMethod`'s doc comment. `phase-5a-arms.tsv` has task-15
rows at `aed89a3e7` only, so neither the breach nor any attribution can be re-derived from a
committed artifact. That is the whole evidentiary basis for a constraint the report itself calls
load-bearing, and the "121 more instructions per send" figure in the doc is the sentence a future
task will read before deciding whether the separation still matters. The breach sitting's own
`--baseline` rows, under a `15-breach` task label, would fix this the way earlier tasks committed
`13-fixround-1-sendpath` and `14-fixround-2`.

### 6. Four doc references to a type that does not exist, one describing the design the performance fix replaced (prose defect)

`MethodRole` appears at `src/lib.rs:1589`, `:1668`, `:1669` and `:4154` and exists nowhere in the
workspace (`grep -rn MethodRole --include=*.rs` returns those four lines only). The worst is
`:4153`-`:4156`, on `install_attribute`:

```
/// One [`InstalledMethodBody`] per accessor, each carrying its own
/// [`MethodRole`]: the `Both` style's two names come from a single
/// directive and are a getter and a setter, which the directive alone
/// does not say.
```

False twice over. A generated accessor is not an `InstalledMethodBody` row -- the test three
hundred lines away asserts `method_bodies.is_empty()` for exactly this directive -- and there is no
role type on it. This is the pre-`aed89a3e7` shape left in the one doc a reader consults about the
two-table separation. `:1589`'s "Why a resolved [`MethodRole::Body`] method's directive cannot be
entered" and `:1593`'s "**Only the `Body` role reaches here**" name a discriminant the design
deliberately does not have; the real mechanism is that `invocable` returns `Invocable::Rexx` only
where `generated_methods` has no row.

`cargo doc --no-deps -p rexx-exec --document-private-items` reports all four as
`rustdoc::broken_intra_doc_links`. Neither `cargo fmt`, nor `cargo clippy --all-targets -D warnings`,
nor `cargo test` can see them, which is why they survived `dd01df2b4` -- a commit whose entire
subject was two sentences of exactly this class in the neighbouring file. (The crate already
carries 16 other broken links, so `cargo doc` is not currently clean and cannot be made a gate
by this task alone.)

### 7. "30 of 33" names no set, and the enumerable part has two more divergences than the accounting covers (prose defect)

The report says "Thirty-three probes were run in total on the final build; 30 match and 3 diverge,
and the three are the stem and compound accessor variables named above". Neither the 33 nor the 3
is enumerated, so the negative claim is as wide as an unnamed set.

The reproducible part: the 25 committed `::METHOD`/`::ATTRIBUTE` table D probes plus the brief's
three live rows give 26 MATCH and 2 DIVERGE. The two are
`attribute__external__subkeyword.rex` (oracle rc 166 `90.998`, crate rc 120
`::ATTRIBUTE EXTERNAL is not implemented (Phase 7)`) and `method__external__subkeyword.rex`
(same shape). Both are legitimate non-identities -- `ORACLE_REFUSES` rows owned by Phase 7, two of
the seven rows already gated at `979522f74` -- but they are not among the three the report names.
So either they were excluded from the 33 without saying so, or the count is wrong. The 19 figure
does check out: 19 probes rewritten (9 `::METHOD`, 10 `::ATTRIBUTE`), all 19 MATCH on both engines,
all 19 read `agree` in `gate_table_d`, and the gated count is 7 both sides -- the five `::ANNOTATE`
rows and the two `EXTERNAL` rows, none of them this task's.

### 8. Comments state the size of sets the list beneath them enumerates (prose defect, low)

* `tests/collect_stress.rs`: "The three refusals a generated accessor's argument bounds produce".
* `corpus/phase-5a.txt`: "the four beside it are the refusals a program can read out of the pair
  ... which is why they are four programs and not one".
* `tests/coverage.rs`: "The value program and the four refusals a program can read out of the
  pair, one fatal each."

All three counts are correct; the constraint is to name the set and never its size. The
`collect_stress.rs` comment's negative half -- "The rest of Task 15's programs are absent" -- is
sound, because that test asserts set equality in both directions
(`assert_eq!(observed, expected, ...)`).

`GeneratedMethod`'s doc adds two of the same shape: "Three narrower shapes were measured", and
"narrowing the struct **back** to 16 bytes", which frames against a width that never shipped in
this tree. The measurements themselves keep their numbers; the cardinality and the "back to" do
not need to be there.

## Checked and clear

* **The narrowed refusal.** `Loud::accessor_variable` names no owner, deliberately, on the
  precedent `Loud::compound_expose` sets and reasons about at length ("unlike `unresolved_call`'s
  `4c`, the steps behind this are not another phase's to build -- nothing has been scheduled to
  build them"). Its remaining scope is stated, not implied: a stem or a single compound tail, with
  both oracle readings recorded (`::attribute "a." class` round-trips `5`; `::attribute "a.b" class`
  answers `a.b` uninitialised). The report's negative claim that neither bootstrap `.orx` file
  needs the stem form holds --
  `grep -inE '^\s*::(attribute|method)\s+["'"'"']?[a-z0-9_]*\.' CoreClasses.orx StreamClasses.orx`
  matches nothing. This deviates from the review brief's expectation that the refusal name an
  owner, but by a documented in-tree decision rather than by omission.
* **No instance receiver.** A genuine boundary, not self-narrowing. `o = .K~new` is rc 120,
  `method "NEW" of class "Object" is not implemented (Phase 5)`, on both engines -- so no program
  can reach the untested instance path, and `pool_owner` refuses a non-class receiver loudly if one
  ever did.
* **The removed refusals' replacement instruments.** Both stated per refusal and both verified live
  by mutation (findings 2 and 3 above, plus controls A and B). Control A also reddens
  `lang/class_method_own_dictionary.rex`, a program this task did not touch, and
  `collect_stress::the_l0_subset_passes_again_under_collect_on_every_allocation`, which the report
  does not list -- understated coverage, not overstated.
* **`UNGUARDED`.** Kept as a row and described as green because there is nothing to see, with the
  C++ reason (the `isGuarded()` split only `reserve`s the dictionary against other activities;
  verified at `execution/CPPCode.cpp:288`-`:301`) and with `dd01df2b4` narrowing the claim to
  `::ATTRIBUTE`'s two rows, which are the ones that reach the accessor.
* **The C++ citations**, other than finding 1. `AttributeGetterCode::run` and
  `AttributeSetterCode::run` at `CPPCode.cpp:281`/`:331` with `count > 0` / `count > 1` and
  `count == 0 || *argPtr == OREF_NULL`, matching the crate's `args.is_empty()`, `args.len() > 1`
  and `let Some(Some(value))`; `AbstractCode::run` at `:527` substituting `messageName`;
  `getRetriever(name)` at `DirectiveParser.cpp:1656` against
  `createAttributeGetterMethod(internalname, ...)` at `:1716`; `concatWithCstring("=")` at `:853`
  and `:1665`. All accurate.
* **The duplicate-directive divergence** the self-review discloses is genuinely pre-existing and
  independent of generated accessors: `::method a class / ::method a class`, both with bodies, is
  oracle rc 157 `99.902` against crate rc 0 printing `second`, on both engines. The crate detects
  no duplicate directive name at all.
* **The edited corpus headers** carry no historical framing. Each describes the code as it stands
  and points at the program that now runs the behaviour the old sentence denied. The
  `method_attribute_generated.rex` header defers the name-miss assertion to
  `a_generated_accessor_pair_reads_and_writes_the_declaring_scopes_pool`, and that test does assert
  it -- not a header claiming a route it cannot see.
* **Binding constraints.** No `unsafe` and no non-ASCII byte anywhere in the diff. No new IR op, so
  no `Op::Generic` question; both engines matched on every probe I ran. `bench-baselines` keeps the
  `instrument` column and I kept it in every projection above.
* **Tree state.** Four mutations applied and reverted (M1 the abstract raise, control A, control B,
  M2 the two-table fold). Restored from copies taken before the first edit; `git status` clean at
  `dd01df2b4`, and `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` back to
  98 `test result: ok` and 177 of 177.

## What a fix round should do

1. Correct `install_method`'s precedence comment against `DirectiveParser.cpp:826`-`:848`, and
   install the `DELEGATE ATTRIBUTE` pair so the setter half refuses loudly instead of answering a
   97.1 that names the wrong receiver. Add the probe as a corpus row or say why the combination is
   left uncovered.
2. Replace the "no test asserts the separation" concern with what is actually true: the separation
   is asserted by `a_method_attribute_installs_a_getter_and_a_setter`; the lookup order cannot be
   asserted; a field added to `InstalledMethodBody` can be, by a `size_of` assert, with the caveat
   that width was measured not to be the mechanism.
3. Replace the "no mutation witness" concern with the measured result: deleting the raise fails
   `an_abstract_send_is_refused_at_the_send_naming_the_message` and takes the corpus to 176 of 177
   on `lang/method_abstract_send.rex`.
4. Fix the sitting prose to four axes at 1.000000 and a layout term bounded at 26 ppm by `rexxcps`,
   and use this sitting's own `value_min`/`value_max` rather than a cross-sitting floor.
5. Commit the breach sitting's rows under their own task label.
6. Delete or repoint the four `MethodRole` references, and rewrite `install_attribute`'s second
   paragraph to describe the two tables.
7. Enumerate the probe set behind "30 of 33" or drop the count, and account for the two `EXTERNAL`
   rows.
8. Strike the four set-size phrases in `collect_stress.rs`, `phase-5a.txt`, `coverage.rs` and
   `GeneratedMethod`'s doc, and the "back to 16 bytes" framing.
