# Task 15 report: `::METHOD` and `::ATTRIBUTE`'s option surface

## Commits

```
9a422b998 Run the accessors a method-declaring directive generates, and refuse ABSTRACT at the send
0fc365d3d Put the accessor pair and the abstract send in the corpus
d7bf45383 Make gate table D's method and attribute probes send a message
aed89a3e7 Keep a generated method out of the table every body send reads
64bfa30d6 Measure this task against the pin and against its own base
dd01df2b4 Say which rows a generated accessor's own doc is evidence about
e9f1e830b Say what the oracle does under DELEGATE with ATTRIBUTE, and assert the row's width
df48d53f0 Commit the sitting that forced the two-table split
eb85dfdd6 Refuse a DELEGATE attribute's setter instead of missing its name
e243c8755 Measure fix round 2 against the round it follows
6cd464849 Say what the delegate rows catch, not what they used to catch
b1016203c Correct a provenance claim and the set-size shape the last sweep left
```

`aed89a3e7` is a performance fix the sitting demanded. `e9f1e830b` and
`df48d53f0` are fix round 1; `eb85dfdd6` through `6cd464849` are fix round 2;
`b1016203c` is fix round 3, prose only. The sections below the horizontal rule
near the end record what each round changed and why.

## What I implemented

Three behaviours, all at the send rather than at the install.

**The generated accessors.** `Interp::read_attribute` and
`Interp::write_attribute` (`dispatch.rs`) read and write one name in the
declaring scope's pool on the receiver, reached through the same
`Interp::pool_owner` that `EXPOSE` uses. `Interp::set_pool_variable`
(`lib.rs`) is the write, added because `set_exposed_variable` reaches a pool by
way of an activation's slot and an accessor has neither an activation nor a
slot. Neither accessor pushes an activation, which is the oracle's own shape:
`AttributeGetterCode::run` and `AttributeSetterCode::run`
(`execution/CPPCode.cpp:280`, `:330`) read or write and return.

**`::METHOD ... ATTRIBUTE` installs the pair.** It installed the plain name
alone before, which is why `.K~a = 5` was 97.1: the setter's dictionary key was
never added. `methodDirective`'s own order of precedence is followed
(`parser/DirectiveParser.cpp:826`-`:915`): `DELEGATE` first, then `ATTRIBUTE`,
then `ABSTRACT`. **`DELEGATE` under `ATTRIBUTE` installs the pair too**, as the
C++ does, and both keys land on the body path where `method_body_gap` refuses
them -- routing either message to the delegate's value is `FORWARD`'s job and
5b's. Fix round 2 below is why that key is installed at all.

**`ABSTRACT` is refused at the send.** `Raised::abstract_method` (`error.rs`)
is 93.965 substituting the *message* name, raised from `Interp::invoke` with no
frame of its own.

**What carries the distinction.** `Interp::generated_methods` is a second table
keyed by `MethodId`, holding a `GeneratedMethod { program, directive, kind }`
per generated name, where `kind` is `Getter`, `Setter` or `Abstract`.
`Interp::invocable` consults it only where `method_bodies` misses. It has to be
recorded rather than derived because one directive mints an id per key:
`::ATTRIBUTE a` with neither `GET` nor `SET`, and `::METHOD a ATTRIBUTE`, each
install `A` and `A=` from a single directive, and the only thing separating
those ids is which half of the pair each is. `DELEGATE` and `EXTERNAL` stay on the body path,
because what refuses them is `method_body_gap` reading the directive.

**Two things deliberately stayed loud**, and they are new refusal texts rather
than the deleted ones:

* A generated accessor whose variable is a stem or a compound tail
  (`Loud::accessor_variable`). Measured, both oracle rc 0:
  `::attribute "a." class` with `.K~'A.' = 5` then `say .K~'A.'` answers `5`,
  and `::attribute "a.b" class` answers `a.b` for an uninitialised read. The
  first needs a stem object in a scope pool with a default assigned there and
  the second needs aliasing at one tail inside such an object, which is what
  `Loud::compound_expose` already refuses from the other direction. This is a
  narrowing of the deleted refusal, not its survival, but "the
  generated-accessor refusal disappeared" would be false.
* `DELEGATE`, on both directives, which is 5b's. Its message text moved from
  `a generated ::ATTRIBUTE accessor` to `a ::ATTRIBUTE with no body of its own`,
  because the old phrase named a construct that now runs.

**One measured fact drove the variable name and is easy to get wrong.** The
pool key is the directive's name **as written**; the two message names are that
name upcased, and upcased with `=` appended. `getRetriever(name)` is handed the
as-written `name` where `addMethod` receives the upcased `internalname`
(`parser/DirectiveParser.cpp:1656`, `:1716`). Measured, oracle rc 0:
`::attribute "aB" class` with `.K~aB = 5` leaves a class method's `expose ab`
reading the derived name `AB`, so the pool entry is *not* keyed on `AB`.
`corpus/lang/method_attribute_generated.rex`'s `bB` pair is the committed
witness; without it the round trip alone would pass an accessor keyed on the
message name.

## The three live rows

Each probe was run from a fresh empty directory with absolute paths, the oracle
under `ulimit -v 1048576` and the crate under `memcap 1G`, three descriptors
read separately into files and compared with `cmp`, never through `2>&1`, on
both engines.

```
### a1.rex  (.K~a = 5 / say .K~a ; ::method a class attribute)
--- ORACLE rc=0
  out| 5
--- ir: MATCH
--- tree-walker: MATCH
### a2.rex  (.K~b = 7 / say .K~b ; ::attribute b class)
--- ORACLE rc=0
  out| 7
--- ir: MATCH
--- tree-walker: MATCH
### a3.rex  (say "installed" / say .K~m ; ::method m class abstract)
--- ORACLE rc=163
  out| installed
  err|      2 *-* say .K~m
  err| Error 93 running .../a3.rex line 2:  Incorrect call to method.
  err| Error 93.965:  Method M is ABSTRACT and cannot be directly invoked.
--- ir: MATCH
--- tree-walker: MATCH
```

**The probe set, named by what its members are rather than counted.** Every
committed table D probe for these two directives --
`corpus/gate-tables/directives/attribute__*.rex` and `method__*.rex`, which
`gate_table_d` runs and lists itself -- matches on both engines except
`attribute__external__subkeyword.rex` and `method__external__subkeyword.rex`.
Those are `ORACLE_REFUSES` rows owned by Phase 7 (oracle rc 166 `90.998`
against the crate's rc 120 `::ATTRIBUTE EXTERNAL is not implemented (Phase 7)`),
they were already gated at `979522f74`, and this task touched neither probe.
Every corpus program this task added or edited matches, which is what
`177 of 177 matching` asserts. The shapes I measured that are not in the tree
and do **not** match have sections of their own below: a stem or compound
accessor variable, each an rc-120 refusal naming its own attribute, and
`DELEGATE` combined with `ATTRIBUTE`.

## The six agreeing rows

`PUBLIC`, `PACKAGE`, `GUARDED`, `UNGUARDED`, `PROTECTED` and `UNPROTECTED` all
still agree, on both directives and both engines -- the table D listing below
has each as `agree`, and each of those probes now sends.

`UNGUARDED` has no Phase-5-observable effect in the reachable shape, and now
that the accessors run I can say where it would have one: the C++ splits on
`method->isGuarded()` only to `reserve` the variable dictionary against other
activities before touching it (`execution/CPPCode.cpp:289`, `:345`), and this
crate runs one activity, so the two arms of that `if` are the same read. Its
row is green because there is nothing to see, not because a check saw
something.

## What the table D probes now do, and why I changed them

Finding 1 of the dispatch was right that the row set does not change. It was
not enough on its own: **every committed probe for these two directives
installed a directive and printed `'main'`, and none of them sent a message**,
so every row this task owns read `agree` at HEAD while the behaviour under it
was a rc-120 refusal, and the control the brief asks for could not move
anything. I rewrote 19 probes so each sends. Each still prints exactly one
`stdout` line, which is `expected_oracle_lines`' bound, and none of them is an
`ORACLE_REFUSES` row, so `check_refusing_probe_says` does not apply.

Rewritten: `::METHOD`'s `ABSTRACT`, `ATTRIBUTE`, `CLASS`, `GUARDED`, `PACKAGE`,
`PROTECTED`, `PUBLIC`, `UNGUARDED`, `UNPROTECTED`, and `::ATTRIBUTE`'s
`ABSTRACT`, `CLASS`, `GET`, `GUARDED`, `PACKAGE`, `PROTECTED`, `PUBLIC`, `SET`,
`UNGUARDED`, `UNPROTECTED`.

Left alone, with reasons: the two `DELEGATE` rows (5b's, and their probes must
keep reading `agree` on an install), the two `EXTERNAL` rows (`ORACLE_REFUSES`,
whose probes must print nothing while asking for output before the directive),
and the two `PRIVATE` rows -- `PRIVATE` is answered or refused depending on who
is sending, which is Task 11's
`private_sends_are_refused_by_who_is_sending` and
`corpus/lang/method_access_private_attribute.rex`, and giving its probe a send
would file that behaviour under a row whose verdict nobody reads for it.

Each rewritten probe was run against the oracle before and after, both engines,
three descriptors, and all 19 match.

`REXX_CORPUS_GATE=1 REXX_PHASE_GATE=5a cargo test --release -p rexx-exec --test
gate_table_d`, the rows this task owns:

```
  agree  loud=no  5a  ::ATTRIBUTE  ABSTRACT     subkeyword
  agree  loud=no  5a  ::ATTRIBUTE  CLASS        subkeyword
  agree  loud=no  5a  ::ATTRIBUTE  GET          subkeyword
  agree  loud=no  5a  ::ATTRIBUTE  GUARDED      subkeyword
  agree  loud=no  5a  ::ATTRIBUTE  PACKAGE      subkeyword
  agree  loud=no  5a  ::ATTRIBUTE  PROTECTED    subkeyword
  agree  loud=no  5a  ::ATTRIBUTE  PUBLIC       subkeyword
  agree  loud=no  5a  ::ATTRIBUTE  SET          subkeyword
  agree  loud=no  5a  ::ATTRIBUTE  UNGUARDED    subkeyword
  agree  loud=no  5a  ::ATTRIBUTE  UNPROTECTED  subkeyword
  agree  loud=no  5a  ::METHOD     ABSTRACT     subkeyword
  agree  loud=no  5a  ::METHOD     ATTRIBUTE    subkeyword
  agree  loud=no  5a  ::METHOD     CLASS        subkeyword
  agree  loud=no  5a  ::METHOD     GUARDED      subkeyword
  agree  loud=no  5a  ::METHOD     PACKAGE      subkeyword
  agree  loud=no  5a  ::METHOD     PROTECTED    subkeyword
  agree  loud=no  5a  ::METHOD     PUBLIC       subkeyword
  agree  loud=no  5a  ::METHOD     UNGUARDED    subkeyword
  agree  loud=no  5a  ::METHOD     UNPROTECTED  subkeyword

gated by this run: 7 row(s) whose owning phase is closing or closed and whose
verdict is not `agree`
```

Seven before and seven after, the same seven and none of them mine: the five
`::ANNOTATE` rows and the two `EXTERNAL` rows. **The phase gate was already red
at HEAD for those**, so a red exit status is not the signal on this table; a
row's verdict label is.

## The control, live

The brief asks for a control that generates a getter without a setter. There
are two installers, so I ran it twice, one at a time, and each one moves a
different instrument -- which is itself the useful result: a single control
would have left half the new code unwitnessed.

**Control A**, `install_attribute`'s `AttributeStyle::Both` arm returning only
the getter:

```
before:  agree         loud=no 5a ::ATTRIBUTE CLASS subkeyword attribute__class__subkeyword.rex
after:   diverge-both  loud=no 5a ::ATTRIBUTE CLASS subkeyword attribute__class__subkeyword.rex
         gated by this run: 7 row(s)  ->  gated by this run: 14 row(s)
```

`method__attribute__subkeyword.rex` stayed `agree`, because the `::METHOD` side
has its own pair construction.

Control A also reddens five in-crate tests and six corpus programs:

```
test dispatch::tests::a_generated_accessor_pair_reads_and_writes_the_declaring_scopes_pool ... FAILED
test dispatch::tests::a_method_body_this_crate_cannot_run_is_loud_and_its_neighbours_still_run ... FAILED
test tests::an_attribute_with_no_style_installs_both_accessor_names ... FAILED
test tests::an_abstract_accessor_pair_is_abstract_on_both_halves ... FAILED
test dispatch::tests::an_abstract_send_is_refused_at_the_send_naming_the_message ... FAILED
test result: FAILED. 716 passed; 5 failed

  [UNCLASSIFIED] lang/class_method_own_dictionary.rex: stdout differ
  [UNCLASSIFIED] lang/method_access_private_attribute.rex: stdout, stderr differ
  [UNCLASSIFIED] lang/method_attribute_generated.rex: stdout, stderr, exit code differ
  [UNCLASSIFIED] lang/method_attribute_generated_no_result.rex: stdout, stderr, exit code differ
  [UNCLASSIFIED] lang/method_attribute_generated_setter_arguments.rex: stderr, exit code differ
  [UNCLASSIFIED] lang/method_attribute_generated_setter_omitted.rex: stderr, exit code differ
  UNCLASSIFIED: 6
  thread 'corpus_differential' panicked at crates/rexx-exec/tests/corpus.rs:745:5
```

`class_method_own_dictionary.rex` is a program this task did not touch at all,
so the coverage is wider than the rows I added;
`method_access_private_attribute.rex` existed and this task extended it.

**Control B**, `install_method`'s `attribute` arm returning only the getter:

```
before:  agree         loud=no 5a ::METHOD ATTRIBUTE subkeyword method__attribute__subkeyword.rex
after:   diverge-both  loud=no 5a ::METHOD ATTRIBUTE subkeyword method__attribute__subkeyword.rex
         gated by this run: 7 row(s)  ->  gated by this run: 8 row(s)

test dispatch::tests::a_generated_accessor_pair_reads_and_writes_the_declaring_scopes_pool ... FAILED
test tests::a_method_attribute_installs_a_getter_and_a_setter ... FAILED
test tests::an_abstract_accessor_pair_is_abstract_on_both_halves ... FAILED
test result: FAILED. 718 passed; 3 failed
```

`attribute__class__subkeyword.rex` stayed `agree` under control B, for the
mirror-image reason. Both controls were reverted, the tree rebuilt, and the
suite confirmed green (`721 passed; 0 failed`) before anything else was
measured.

## What instrument catches a regression, per removed refusal

The corpus gate cannot see a clean refusal becoming a wrong answer, so this is
stated per refusal rather than assumed.

**The generated-accessor refusal.** Removed. Caught by
`dispatch::tests::a_generated_accessor_pair_reads_and_writes_the_declaring_scopes_pool`,
by `tests::a_method_attribute_installs_a_getter_and_a_setter`, by
`lang/method_attribute_generated.rex` and the `method_attribute_generated_*`
programs beside it, by `lang/method_access_private_attribute.rex`'s new
`allowed` line, and by the
table D rows `attribute__class__subkeyword.rex` and
`method__attribute__subkeyword.rex` -- the last two only under
`REXX_CORPUS_GATE=1 REXX_PHASE_GATE=5a`, which is not one of the five gate
commands, so outside that command they are a report. Every one of these was
shown to move by control A or control B above.

**The abstract-send refusal.** Removed. Caught by
`dispatch::tests::an_abstract_send_is_refused_at_the_send_naming_the_message`,
by `tests::an_abstract_accessor_pair_is_abstract_on_both_halves`, by
`lang/method_abstract_send.rex`, and by the table D rows
`method__abstract__subkeyword.rex` and `attribute__abstract__subkeyword.rex`.
Gate table C is not an instrument for it and its own `abscla` row says so:
"abstract-*class* enforcement is 5b, because the check lives inside `~new`; the
abstract-*method* half is 5a and has no arm of this section's probe."

**Proved live in fix round 1, having been declined in round 0.** Mutation M1
replaces the raise with `Ok(None)` --
`crate::GeneratedKind::Abstract => Ok(None)` in `Interp::invoke` -- and
`REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` answers:

```
test dispatch::tests::an_abstract_send_is_refused_at_the_send_naming_the_message ... FAILED
test result: FAILED. 720 passed; 1 failed
176 of 177 matching
  [UNCLASSIFIED] lang/method_abstract_send.rex: stderr, exit code differ
```

Two witnesses, one of them a corpus program **inside the five gates**, so a
build that dropped the raise fails a named program rather than passing quietly.
`tests::an_abstract_accessor_pair_is_abstract_on_both_halves` did not move, and
correctly so: it asserts what the install recorded, not what the send does. The
mutation was reverted and the suite is back to `721 passed; 0 failed`.

**What no instrument here covers.** Every row above has a class object as its
receiver, because reaching an instance needs `~new`. The instance reading of a
generated accessor -- an instance's own `Body::Instance` pools rather than the
arena object `pool_owner` mints for a class -- is untested rather than
confirmed, and `pool_owner` still refuses a non-class receiver loudly.

**And these instruments can fail the same way the deleted rows could.** They are
hand-written assertions over hand-written sources, not derived from any
authority: a row rewritten to assert the wrong subject passes, and the check
against that is the diff. That was true of the two refusal rows they replace.

## Corpus

171 to 177, all matching.

```
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast
  177 of 177 matching
```

Added to `corpus/phase-5a.txt` and to `EXPECTED_SUBSET_5A`
(`crates/rexx-exec/tests/coverage.rs`):

* `lang/method_attribute_generated.rex` -- the round trip through both
  directives, the as-written variable spelling against the upcased message
  name, the pool entry `EXPOSE` reaches from both sides, one declaration over
  two receivers, the `GET`-only and `SET`-only halves, and a class object
  stored and read back.
* `lang/method_attribute_generated_no_result.rex` -- 91.999, the setter
  answering nothing.
* `lang/method_attribute_generated_getter_arguments.rex` -- 93.902, `0 expected`.
* `lang/method_attribute_generated_setter_arguments.rex` -- 93.902, `1 expected`.
* `lang/method_attribute_generated_setter_omitted.rex` -- 93.903, `argument 1`.
* `lang/method_abstract_send.rex` -- 93.965, with `installed` on `stdout` above
  it so the program shows the install succeeded before the send failed.

The refusal programs are separate programs because a program can only fail
once.

Existing corpus programs were edited where a sentence in them became false, each `sourceline_oracle/<name>.txt` regenerated with the driver
in that test's module comment, and each re-differentialled afterwards:

* `lang/method_attribute_body.rex` -- said the generated accessors "read an
  instance variable and are Phase 5's still".
* `lang/method_attribute_set_body.rex` -- said "a program that printed a stored
  value here would be witnessing instance variables, which this phase has none
  of". It has them now; what is still true is that *those two bodies* store
  nothing, which is a property of the bodies.
* `lang/method_access_private_attribute.rex` -- said "A bodyless private
  attribute read from a caller the check ALLOWS is not here: ... this crate
  refuses that accessor". It is here now, as the `allowed` line, and it is the
  row that separates the access check from the accessor.

The new programs whose refusal is reached before the accessor touches a pool
perform zero collections, and were added to `collect_stress.rs`'s committed
list; that test asserts set equality in both directions, so the ones absent from
it are asserted absent.

## The five gates

Run from `rust/`, in order, on the final tree.

```
cargo fmt --all --check                                            -> FMT OK
cargo clippy --workspace --all-targets -- -D warnings              -> Finished, no warnings
cargo test --release --workspace                                   -> no failures
REXX_CORPUS_GATE=1 cargo test --release --workspace                -> 177 of 177 matching, no failures
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast -> 177 of 177 matching, exit 0
```

**And a sixth command, which is not a gate but which the five cannot replace.**
`cargo doc --no-deps -p rexx-exec --document-private-items` is the only thing
that sees a broken intra-doc link, and fix round 1 exists partly because of what
it found. It is not proposable as a gate by this task alone: the crate carries
15 unresolved links older than this task.

`memcap` was present (`/home/moritz/.local/bin/memcap`).

Failures appeared on the first `--release --workspace` run and were fixed
rather than matched against a name: `coverage.rs`'s
`phase_5a_subset_matches_the_committed_list` (the new corpus entries needed
adding to `EXPECTED_SUBSET_5A`) and `collect_stress.rs`'s
`the_l0_subset_passes_again_under_collect_on_every_allocation` (the
zero-collection programs needed adding to its list).

## The sitting

Three builds in one interleaved sitting -- the Phase 5a pin, this task's own
base revision `979522f74`, and the final build -- five rounds, both engines,
both sizes, eight axes, instrument `instructions:u`. Committed as
`bench-baselines/phase-5a-arms.tsv`, rows `15 / aed89a3e7`.

```
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-15a1ffa98 \
                           --build base=<979522f74 build> \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings \
    --axis varlookup --axis dispatchclass --axis bench-rexxcps/rexxcps.rex \
    --rounds 5 --task 15 --commit aed89a3e7 --baseline bench-baselines/phase-5a-arms.tsv
```

`pinned>head` is the accumulated phase drift and is not this task's delta. My
own contribution is `pinned>head` over `pinned>base`, both taken in this one
interleaved sitting:

| axis | arm | pinned>base | pinned>head | my contribution |
|---|---|---|---|---|
| alloc4c | tw | 1.002978 | 1.002978 | 1.000000 |
| alloc4c | ir | 1.004805 | 1.004805 | 1.000000 |
| arith | tw | 0.999778 | 0.999778 | 1.000000 |
| arith | ir | 0.993066 | 0.993066 | 1.000000 |
| compound | tw | 1.006232 | 1.006232 | 1.000000 |
| compound | ir | 1.010472 | 1.010472 | 1.000000 |
| emptyloop | tw | 0.995434 | 0.995434 | 1.000000 |
| emptyloop | ir | 0.992023 | 0.992022 | 0.999999 |
| strings | tw | 1.008182 | 1.008182 | 1.000000 |
| strings | ir | 1.013410 | 1.013409 | 0.999999 |
| varlookup | tw | 0.996025 | 0.996025 | 1.000000 |
| varlookup | ir | 0.994260 | 0.994260 | 1.000000 |
| **dispatchclass** | tw | 1.012110 | 1.012583 | **1.000467** |
| **dispatchclass** | ir | 1.011789 | 1.012258 | **1.000463** |
| rexxcps | tw | 1.015410 | 1.015420 | 1.000010 |
| rexxcps | ir | 1.018601 | 1.018627 | 1.000026 |

(`small` size shown; the `large` rows agree -- dispatchclass tw 1.000458, ir
1.000466.)

**Four axes read exactly 1.000000 on both arms**: `alloc4c`, `arith`,
`compound` and `varlookup`. `emptyloop`/ir and `strings`/ir read 0.999999 and
`rexxcps` reads 1.000010 and 1.000026, so the earlier draft's "seven axes at
1.000000" and "no layout movement on them at all" were contradicted by this very
table.

**Is it change or layout?** The corrected reading is the stronger one.
`rexxcps` neither declares a class nor sends a message -- checked with
`/bin/grep -c` on `bench-rexxcps/rexxcps.rex`, which answers `0` for `::CLASS`
and `0` for `~` -- so it cannot execute the new code and its 26 ppm is a layout
term and nothing else. That bounds the layout term at 26 ppm. `dispatchclass`,
the one axis whose loop is a class-method send, moved 458 to 467 ppm, about 18
times that bound. So the movement is the change, and it is
2.92 `instructions:u` per pass on the tree-walker and 2.98 on the compiled
engine:

```
dispatchclass  base   tw  6496.432133      head   tw  6499.347899
dispatchclass  base   ir  6417.444253      head   ir  6420.423053
```

**Bounded by this sitting's own round spread, which is the tightest argument
available and is same-sitting.** `value_min`/`value_max` for
`task=15 dispatchclass base per_pass instructions:u` are 6496.358670/6556.351206
(tw) and 6387.393660/6443.388558 (ir) -- a 56 to 60 instruction spread per pass,
against a movement of about 3. On the tree-walker arm head's own minimum,
6469.417368, is *below* base's minimum of 6496.358670; on the compiled arm
head's median, 6420.423053, sits inside base's own round range.

Other bounds exist and both are looser. The 34 per pass the dispatch brief
records is one. The ~13 from the byte-identity control in `1d87d90cc` is the
other, and it is a **cross-sitting** span -- one byte-identical binary read
6417.398606 in one sitting and 6404.359463 in another -- so it is not this arm's
in-sitting floor, and the earlier draft citing it as one mixed sittings. The
in-sitting spread above is wider and is what the conclusion rests on.

**What `cycles:u` did in the same sitting, disclosed rather than omitted.**
`dispatchclass` `pinned>head` on `cycles:u` reads 1.051351 to 1.058532 against
`pinned>base`'s 0.998253 to 1.008736 -- so on that instrument the axis moved
+5.1% to +5.9%. That figure is not the result, and the reason is in the same
sitting: on axes that provably cannot reach the new code, `cycles:u`
`pinned>head` runs from 0.931735 (`arith`/ir) to 1.036705 (`strings`/ir), a
spread wider than dispatchclass's movement, while those same axes are exact to
six decimal places on `instructions:u`. `cycles:u` on this machine cannot
resolve what `instructions:u` resolves, which is why the guard's instrument is
`instructions:u` -- but a reader is entitled to see the number rather than only
the rule.

### The breach sitting, and where it lives now

The sitting taken at `d7bf45383` -- the same protocol, the same base -- read
**+1.86% on dispatchclass**. That first round of measurement was prose only, and
fix round 1 replaced it with a committed artifact:
`bench-baselines/phase-5a-arms.tsv`, `task=15-breach`, `commit=dd01df2b4`, one
interleaved sitting on `dispatchclass` with the base revision first so every
ratio is read against it. **These rows are re-measured, not the original
sitting's**: the original run's rows were reverted out of the working tree
before they were ever committed, and the binaries were kept, so the sitting was
run again from the same binaries rather than relabelled.

```
build                    base>build (ir large)   ir per pass    what it is
a-base-979522f74         1.000000                6417.359       979522f74
b-presplit-d7bf45383     1.018823                6538.416       the breach
c-accessors-outlined     1.018356                6535.436       inline(never) on the accessor pair
d-helpers-reinlined      1.016022                6520.379       and Activation::method/super_scope_for inlined again
e-row-narrowed           1.020226                6547.375       directive as a u32, 16-byte row -- worse
f-one-comparison         1.018198                6534.405       generated arms behind one comparison, out of line
g-accessors-deleted      1.000005                6417.416       accessor code unreachable, so deleted
h-split-dd01df2b4        1.000469                6420.446       two tables
```

`g` is what localises it. With the accessor code reachable from `Interp::invoke`
the body path pays, whatever shape the branch or the row takes; with it
unreachable and therefore deleted, the axis returns to base. That is why none of
`c`, `d`, `e`, `f` recovered it and why `e` -- narrowing the row -- was worse
than doing nothing. `h` is the shape that keeps the accessors and pays nothing
measurable.

The `perf stat -e instructions:u` readings quoted in `GeneratedMethod`'s doc
comment were taken directly on `bench-programs/dispatchclass.rex` while the
attributions were being chosen, and the sitting above is the committed form of
the same comparison. The doc points at `task=15-breach` for it.

## Files changed

Production:

* `rust/crates/rexx-exec/src/lib.rs` -- `GeneratedMethod`, `GeneratedKind`,
  `Interp::generated_methods`, `install_one_method`, both installers,
  `accessor_setter_name`, `accessor_variable`, `set_pool_variable`,
  `Loud::accessor_variable`, `method_body_gap`'s narrowing, `pools_of` widened
  to `pub(crate)`.
* `rust/crates/rexx-exec/src/dispatch.rs` -- `Invocable::Generated`,
  `Interp::invocable`'s second lookup, `read_attribute`, `write_attribute`,
  `Interp::accessor_variable`, and the tests.
* `rust/crates/rexx-exec/src/error.rs` -- `Raised::abstract_method`.
* `rust/crates/rexx-exec/src/run.rs` -- `pool_owner` widened to `pub(crate)`.

Tests and lists:

* `rust/crates/rexx-exec/tests/coverage.rs`, `tests/collect_stress.rs`.
* `rust/corpus/phase-5a.txt`.
* `rust/corpus/lang/` -- the `method_attribute_generated*` and
  `method_abstract_send` programs are new; `method_attribute_body`,
  `method_attribute_set_body` and `method_access_private_attribute` are edited.
* `rust/crates/rexx-parse/tests/sourceline_oracle/` -- one expectation per new
  or edited corpus program, regenerated with that test's own driver.
* `rust/corpus/gate-tables/directives/` -- 19 probes rewritten.
* `rust/bench-baselines/phase-5a-arms.tsv` -- the `15` sitting, fix round 1's
  `15-breach` rows, and fix round 2's `15-fixround-2` rows.

## Self-review findings

* **`install_method`'s `attribute && external` arm records no generated method
  and nothing reaches it**, because `directive_gap` refuses
  `::METHOD ... EXTERNAL` while the package is still installing (measured: the
  `::METHOD EXTERNAL` table D row is rc 120 with empty `stdout`). I wrote the
  arm rather than relying on the unreachability, and no comment claims it is
  unreachable.
* **`accessor_variable`'s free function answers `Some` for any `::METHOD`**,
  including one that generates nothing. It is only called for a `Getter` or a
  `Setter`, and its doc says a `None` here is an internal inconsistency rather
  than claiming the input is constrained.
* **A duplicate accessor key does not fire `record_method_body`'s
  `debug_assert`**, because `add_class_method` mints a fresh `MethodId` per
  call. Checked in a debug build: `::method m class attribute` beside
  `::method 'm=' class` is rc 0 here where the oracle is 99.902 at rc 157. That
  divergence is pre-existing -- the crate detects no duplicate directive name
  at all, and `::attribute a class` beside `::method 'a=' class` was already the
  same shape before this task -- so it is neither introduced nor closed here.
* **I dropped the getter's argument bound while rewiring to two tables** and
  caught it re-reading the diff, not from a test: the signature change removed
  `args` and with it the `93.902` check, and the in-crate rows for it are inside
  the same test file, which was still green because I had not rebuilt. It is
  restored, and the `method_attribute_generated_*_arguments` and
  `_setter_omitted` corpus programs are what would have caught it at the next
  gate.
* The `Rc::clone` I first wrote in `Interp::accessor_variable` was unnecessary
  (the returned `Box<[u8]>` owns its bytes, so the borrow ends before the
  return) and was removed; clippy caught the `&mut self` that went with it.
* **A doc comment I wrote claiming the struct's width bought the 121
  instructions was false and is deleted.** The narrowing experiment made the
  axis worse; the sentence had been written from the hypothesis rather than the
  reading, in the same edit that built the binary to test it.

## Concerns

* **Rewriting that many probes is more than "the row set does not change"
  suggests**, and it is a change to committed evidence rather than to
  behaviour. Every rewritten probe was re-differentialled on both engines and
  the `gated by this run` count is unchanged, but a reviewer should read the
  probe diff as evidence being replaced rather than added.
* **A stem or compound attribute variable is still a loud refusal.** The brief
  says the generated-accessor refusal disappears; it narrows instead. Nothing
  in the corpus or in either bootstrap `.orx` file needs the stem form --
  `/bin/grep -icE '^[[:space:]]*::(attribute|method)[[:space:]]+["\']?[a-z0-9_]*\.'`
  over `interpreter/RexxClasses/CoreClasses.orx` and `StreamClasses.orx`
  answers `0` for both -- but "the refusal disappeared" would be false.
* **`DELEGATE` combined with `ATTRIBUTE` still diverges, and fix round 2
  changed which way.** Both halves now refuse at rc 120 where the oracle
  answers 97.1 at rc 159 naming `Object "P"`, so the crate differs from the
  oracle on the exit status where the setter's name miss used to agree with it.
  That is the trade taken deliberately: the earlier shape matched the status,
  the error number and the sub-number and differed only in the substituted
  receiver, which nothing comparing status or error number could find. What 5b
  inherits is a loud refusal. The oracle's forwarding is still the target, and
  the refusal does not stand in for it.
* **The `DELEGATE` attribute's instrument is an in-crate test and nothing
  else.** Stated in the instrument section, repeated here because it is the
  thinnest coverage this task leaves: no corpus program and no table D row
  reaches the combination.
* **No instance receiver anywhere.** Everything here is a class-side send, and
  `pool_owner` still refuses any other receiver loudly. 5b's `~new` is where
  the instance reading of all of this gets its first test.
* **The lookup order is the one half of the performance fix nothing can
  assert.** The two tables are disjoint by construction, so swapping the order
  in `Interp::invocable` is invisible to behaviour -- mutation M2 confirmed it
  (`177 of 177 matching`) -- and no test can distinguish it. The comment above
  the second lookup is the only defence and there is no mechanical one
  available. The other two halves are covered: the separation by
  `tests::a_method_attribute_installs_a_getter_and_a_setter`, and a field added
  to `InstalledMethodBody` by the width assertion this round added.

---

## Fix round 1

Eight findings. What each one changed, and what it was measured with.

**1. `DELEGATE` under `ATTRIBUTE`: a false C++ citation over a real
divergence.** `install_method`'s comment said the C++ installs the plain name
alone when the two keywords are combined. `DirectiveParser.cpp:826` says the
opposite in its own words -- "A delegate method can also be an attribute, which
really just means we produce two delegate methods" -- and calls
`createDelegateMethod` for `setterName` as well as for `internalname`. I read
those lines and re-measured both halves myself, `::method a class delegate p
attribute` beside `::attribute p class`, oracle under
`( ulimit -v 1048576; timeout -s KILL 10 ... )` and crate under
`( memcap 1G env REXX_ENGINE=<engine> timeout -s KILL 10 ... )`:

```
### d1.rex   .K~a = 5 / say 'stored'
--- ORACLE rc=159
  err|      1 *-* .K~a = 5
  err| Error 97 running .../d1.rex line 1:  Object method not found.
  err| Error 97.1:  Object "P" does not understand message "A=".
--- ir: DIVERGE rc=159
  err|      1 *-* .K~a = 5
  err| Error 97 running .../d1.rex line 1:  Object method not found.
  err| Error 97.1:  Object "The K class" does not understand message "A=".
--- tree-walker: DIVERGE rc=159   (identical to ir)

### d2.rex   say .K~a
--- ORACLE rc=159
  err| Error 97.1:  Object "P" does not understand message "A".
--- ir: DIVERGE rc=120
  err| rexx-exec: a ::METHOD with no body of its own is not implemented (Phase 5)
--- tree-walker: DIVERGE rc=120   (identical to ir)
```

**The setter half is a wrong answer hiding inside a matching refusal**: same
exit status, same error number and sub-number, a different substituted
receiver, because the crate never installed `A=` and the send is an ordinary
name miss on the class. The getter half is a loud refusal naming Phase 5, which
is correct while `DELEGATE` is 5b's.

`DELEGATE` was not implemented -- `gate_table_d.rs:237` assigns it to 5b and
`:202` gives the reason. What round 1 changed is the comment, which says what
the oracle does and carries both transcripts, so the witness is in the tree
rather than only in this report. Round 1 left the arm alone and recorded the
wrong answer as 5b's to inherit; **fix round 2 replaced that decision** -- see
below.

Also stated in round 1 and still true: **nothing covers the `DELEGATE` plus
`ATTRIBUTE` combination outside an in-crate test.** No corpus program reaches
it, and table D's row identity is (directive, keyword, position), so its
`DELEGATE` row's probe exercises the keyword alone.

**2. "no test asserts the separation" was false.** Mutation M2 folds every
generated method into `method_bodies` as well and swaps `Interp::invocable`'s
two lookups, which is behaviour-preserving.
`REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast`:

```
test tests::a_method_attribute_installs_a_getter_and_a_setter ... FAILED
test result: FAILED. 720 passed; 1 failed
177 of 177 matching
```

Exactly one catcher, and the corpus stays green because behaviour did not
change -- the right shape for a perf-only constraint. So the separation is
test-enforced; what is not is the lookup order, which is now its own concern.
The third half, a field added to `InstalledMethodBody`, is enforceable and now
is: `const _: () = assert!(size_of::<InstalledMethodBody>() == 16);` beside the
struct, the idiom `ir.rs:103` and `lib.rs`'s `Argument` already use. Proved live
by adding a `GeneratedKind` field to the struct:

```
error[E0080]: evaluation panicked: assertion failed: size_of::<InstalledMethodBody>() == 16
    --> crates/rexx-exec/src/lib.rs:3218:15
```

The assertion's own doc carries the caveat the reviewer asked for: it catches
the named edit and does **not** guard the mechanism, because the same
measurement narrowed the widened row to 16 bytes and the axis got worse.

**3. "the abstract refusal has no mutation witness" was false.** Mutation M1,
quoted in the instrument section above: one in-crate test fails and the corpus
goes to `176 of 177` naming `lang/method_abstract_send.rex`, which is inside the
five gates.

**4. The sitting prose contradicted its own table.** Rewritten: four axes at
exactly 1.000000, `rexxcps` at 1.000010 and 1.000026 bounding the layout term at
26 ppm, `dispatchclass` at 458 to 467 ppm and so about 18 times that bound, and
the movement bounded by this sitting's *own* `value_min`/`value_max` on the base
arm rather than by the cross-sitting byte-identity figure -- which is now
labelled as cross-sitting. The `cycles:u` movement is disclosed with the reason
it is not the result.

**5. The breach measurement was prose only.** Committed as
`task=15-breach` in `bench-baselines/phase-5a-arms.tsv`: one interleaved
sitting on `dispatchclass`, the base revision first, the pre-split head, each
of the four attributions, the probe that deleted the accessor code, and the
split head. Table and reading in "The breach sitting, and where it lives now"
above. **These rows are re-measured**, from the same binaries, because the
original run's rows were reverted out of the working tree before they were ever
committed; nothing was relabelled.

**6. Four doc links named a type the performance fix removed.** `MethodRole` is
gone from the workspace; the links now name `GeneratedKind` and
`GeneratedMethod`, and `install_attribute`'s second paragraph describes the two
tables instead of describing a generated accessor as an
`InstalledMethodBody` row -- which a test in the same file asserts against.
`cargo doc --no-deps -p rexx-exec --document-private-items` was run, before and
after: it also found one broken link this task had introduced,
`[NativeMethod]: crate::dispatch::NativeMethod` in `error.rs`, pointing at a
private alias in a private module, and that is now plain prose. The crate
reports 15 unresolved links, all older than this task. **Neither `fmt`, nor
`clippy --all-targets -D warnings`, nor `cargo test` sees any of these**, which
is why they survived a commit whose whole subject was two sentences of the same
class.

**7. "30 of 33" named no set.** Replaced by the set itself, with the two
`EXTERNAL` divergences accounted for -- see "The three live rows" above.

**8. Comments naming the size of a set the list beneath them enumerates.** The
sites the review names are rewritten, and I swept the whole diff for the shape
rather than only those: `git diff 979522f74 -- crates/ corpus/` filtered for a
cardinality word followed by a plural noun found one more,
`install_attribute`'s "the same three questions ... only when all three are
`None`", which now names the questions instead.

**The claim I then wrote about what the sweep left was false**, and fix round 3
below is where that is fixed. I wrote that the remaining `two`s were all
back-references to a pair named in the same sentence. They were not: the sweep's
output included "beside the two fields" twice, counting `InstalledMethodBody`'s
own fields -- one of those instances added by this very sweep commit. I had run
the right command and misread its output.

**What this round re-ran, and what it did not.** Re-run on the fix-round build:
the five gates, every committed table D probe for these two directives (the
sweep the corrected probe-set paragraph reports), mutations M1 and M2,
`cargo doc`, the `DELEGATE ATTRIBUTE` pair on both engines, and the
`CoreClasses.orx`/`StreamClasses.orx` grep. **Not re-run:** controls A and B,
which round 0 measured and the reviewer measured independently, and the
eight-axis sitting at `aed89a3e7`, whose rows this round did not touch.

---

## Fix round 2

One change, ruled by the coordinator after round 1 handed the decision back:
install the `DELEGATE ATTRIBUTE` setter's key so the send **refuses loudly**
instead of answering with the wrong receiver.

**What the change is.** `install_method`'s `DELEGATE` arm installs the setter's
key beside the plain one when `ATTRIBUTE` is also present, which is what
`methodDirective` does (`DirectiveParser.cpp:826`-`:848`). Both keys go on the
body path, so `method_body_gap` refuses them; nothing routes a message to a
delegate property's value, which is `FORWARD`'s job and stays 5b's. The comment
keeps round 1's transcripts, because the oracle's `Object "P"` answer is still
the target and the refusal does not stand in for it.

**Measured, both engines, both sides bounded** -- oracle under
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib timeout -s KILL 10 .../rexx FILE )`,
crate under `( memcap 1G env REXX_ENGINE=<engine> timeout -s KILL 10 target/release/rexx-run FILE )`,
three descriptors read into separate files and compared with `cmp`, from a fresh
empty directory:

```
### d1.rex    .K~a = 5 / say 'stored'    ::method a class delegate p attribute + ::attribute p class
--- ORACLE rc=159
  err|      1 *-* .K~a = 5
  err| Error 97 running .../d1.rex line 1:  Object method not found.
  err| Error 97.1:  Object "P" does not understand message "A=".
--- ir: DIVERGE rc=120
  err| rexx-exec: a ::METHOD with no body of its own is not implemented (Phase 5)
--- tree-walker: DIVERGE rc=120   (identical to ir)

### d2.rex    say .K~a                   same directives
--- ORACLE rc=159
  err| Error 97.1:  Object "P" does not understand message "A".
--- ir: DIVERGE rc=120
  err| rexx-exec: a ::METHOD with no body of its own is not implemented (Phase 5)
--- tree-walker: DIVERGE rc=120   (identical to ir)
```

Before this round, `d1` read rc 159 with
`Error 97.1:  Object "The K class" does not understand message "A="` -- the
oracle's status, the oracle's error number and sub-number, and a receiver the
oracle does not name.

**What it costs, said as a trade rather than an oversight.** The crate's rc 120
now differs from the oracle's rc 159 on the setter, where the name miss agreed
with it. That agreement was the problem: it made the divergence invisible to
every comparison a harness makes except a byte comparison of `stderr`. The
refusal spends a matching status on a difference a reader can find, and it is
the trade this plan makes everywhere else.

**The instrument, stated exactly.** A row per half of the pair, added to
`dispatch::tests::a_method_body_this_crate_cannot_run_is_loud_and_its_neighbours_still_run`,
**and nothing else** -- no corpus program covers the combination in either
direction, and table D's row identity is one keyword. An in-crate test only. Proved live by reverting the install and running
`cargo test --release -p rexx-exec --lib`:

```
test dispatch::tests::a_method_body_this_crate_cannot_run_is_loud_and_its_neighbours_still_run ... FAILED
test result: FAILED. 720 passed; 1 failed
assertion `left == right` failed: ".K~a = 5\nsay 'stored'\n::class K\n::method a class delegate p attribute\n::attribute p class\n"
  left: (159, "", "     1 *-* .K~a = 5\nError 97 running /t.rex line 1:  Object method not found.\nError 97.1:  Object \"The K class\" does not understand message \"A=\".\n")
 right: (120, "", "rexx-exec: a ::METHOD with no body of its own is not implemented (Phase 5)\n")
```

The control was reverted and the suite is back to `721 passed; 0 failed`.

**Prose around the edit.** Sentences that were true before this change and
false after it, all rewritten: "What I implemented"'s claim that the `DELEGATE`
arm is where this crate and the oracle part on the *install*; the concern that
called the combination "a wrong answer this task did not close"; round 1's
finding 1, which recorded the arm as left alone; and `install_method`'s own
comment, which now says what the setter's key is for in the present tense rather
than narrating the shape it replaced. The new test comment needed the same
treatment on a second pass and got it. Round 1's finding 1 keeps its transcripts
and gained a pointer to this round.

**The sitting.** Rows `15-fixround-2 / eb85dfdd6` in
`bench-baselines/phase-5a-arms.tsv`: the pin, `df48d53f0` as base, and this
build as head, five rounds, both engines, both sizes, all eight axes. It was run
rather than skipped: the diff is inside `install_method`, which runs at install
time and which no bench axis reaches with a `DELEGATE` directive, so nothing new
*executes* on any axis -- but this task has already been surprised once by code
that never runs moving a per-pass figure, and the guard is cheaper than the
argument.

My own contribution, `pinned>head` over `pinned>base` from that one sitting,
`instructions:u`:

```
axis          arm size    pinned>base  pinned>head       mine
alloc4c       ir  large      1.004621     1.004621   1.000000
alloc4c       ir  small      1.004805     1.004805   1.000000
alloc4c       tw  large      1.002902     1.002902   1.000000
alloc4c       tw  small      1.002978     1.002978   1.000000
arith         ir  large      0.993016     0.993016   1.000000
arith         ir  small      0.993066     0.993066   1.000000
arith         tw  large      0.999744     0.999744   1.000000
arith         tw  small      0.999778     0.999778   1.000000
compound      ir  large      1.010473     1.010473   1.000000
compound      ir  small      1.010472     1.010472   1.000000
compound      tw  large      1.006232     1.006232   1.000000
compound      tw  small      1.006232     1.006232   1.000000
dispatchclass ir  large      1.012281     1.012279   0.999998
dispatchclass ir  small      1.012257     1.012257   1.000000
dispatchclass tw  large      1.012597     1.012597   1.000000
dispatchclass tw  small      1.012581     1.012572   0.999991
emptyloop     ir  large      0.992022     0.992022   1.000000
emptyloop     ir  small      0.992022     0.992023   1.000001
emptyloop     tw  large      0.995434     0.995434   1.000000
emptyloop     tw  small      0.995434     0.995434   1.000000
rexxcps       ir  small      1.018643     1.018638   0.999995
rexxcps       tw  small      1.015428     1.015430   1.000002
strings       ir  large      1.013409     1.013409   1.000000
strings       ir  small      1.013410     1.013410   1.000000
strings       tw  large      1.008182     1.008182   1.000000
strings       tw  small      1.008182     1.008182   1.000000
varlookup     ir  large      0.994260     0.994260   1.000000
varlookup     ir  small      0.994260     0.994260   1.000000
varlookup     tw  large      0.996025     0.996025   1.000000
varlookup     tw  small      0.996025     0.996025   1.000000
```

**Every axis and arm within 9 ppm of 1.000000**, the widest being
`dispatchclass`/tw/small at 0.999991 and `rexxcps`/ir at 0.999995, both
*negative*. Nothing here is a movement; the guard is answered.

**The accumulated ratio is over 1% on two axes, and this task's contribution
against it is nil.** The plan's guard rule -- quoted in the re-review at
`docs/superpowers/plans/2026-08-17-phase-5a.md:470`-`:472` -- asks a task in
that position to say so and to name the task that raised it. Derived from the
TSV, `scope=across_builds`, `build=pinned>head`, `instrument=instructions:u`:

```
dispatchclass, by task, in commit order
  13-fixround-1  ir/large 1.009290  ir/small 1.009267  tw/large 1.009953  tw/small 1.009940
  14             ir/large 1.011803  ir/small 1.011780  tw/large 1.012133  tw/small 1.012110
  14-fixround-1  ir/large 1.011804  ir/small 1.011790  tw/large 1.012128  tw/small 1.012118
  14-fixround-2  ir/large 1.011802  ir/small 1.011786  tw/large 1.012132  tw/small 1.012119
  15             ir/large 1.012276  ir/small 1.012258  tw/large 1.012597  tw/small 1.012583
  15-fixround-2  ir/large 1.012279  ir/small 1.012257  tw/large 1.012597  tw/small 1.012572

rexxcps, by task
  15             ir/small 1.018627  tw/small 1.015420
  15-fixround-2  ir/small 1.018638  tw/small 1.015430
```

* **`dispatchclass` crossed 1% during Task 14, and Task 14 owns the answer.**
  Task 13's fixround is the axis's first sitting and sits under the line on
  every cell, 1.009267 to 1.009953. Task 14's own contribution is the crossing:
  `pinned>head` over `pinned>base` in its own sitting is 1.002492 on both ir
  cells, 1.002154 on tw/large and 1.002149 on tw/small, against a base of
  1.009265 to 1.009958. Task 15's main sitting adds 458 to 467 ppm and this
  round adds nil.
* **`rexxcps` has no owner, and the plan already licenses that.** The axis has
  rows under `task` values `15` and `15-fixround-2` and no others, because it
  joined the guard at this task, so its 1.015430 and 1.018638 are the phase's
  whole accumulated drift on a path nothing measured before. A first reading is
  a position rather than a delta, which is what the plan's own axis paragraph
  says of it -- not something to attribute and not something to apologise for.

A direct check before the sitting agreed and is worth quoting because of its
shape. `perf stat -e instructions:u` on `bench-programs/dispatchclass.rex`,
alternating the two builds:

```
base-df48d53f0 ir 25735865082      r2head ir 25787985073
base-df48d53f0 ir 25788056106      r2head ir 25735931693
```

**Each build produced both values.** The 52-million spread, about 0.2%, is a
run-to-run artifact of this arm and not a difference between the binaries --
which is exactly the signature a byte-identity control gives, and the reason a
single pair of runs on this axis cannot settle anything.

**`cargo doc`.** `cargo doc --no-deps -p rexx-exec --document-private-items`
after the change reports 15 unresolved links, the same 15 as after round 1 and
all of them older than this task, and none naming a type or field this task
introduced (checked by grepping the warnings for `MethodRole`, `GeneratedKind`,
`GeneratedMethod`, `InstalledMethodBody`, `NativeMethod`, `generated_methods`
and `accessor`, which matches nothing).

**The five gates**, re-run on the frozen tree at `6cd464849` after every edit
this round: `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean, `cargo test --release --workspace` no
failures, and both `REXX_CORPUS_GATE=1` runs -- release and debug under
`memcap 8G` -- at `177 of 177 matching`.

**What this round did not re-run.** The three live rows, the six agreeing rows,
controls A and B, mutations M1 and M2, and the `15-breach` sitting. All were
measured in earlier rounds against code this round does not touch. The corpus is
unchanged by this round: no program covers `DELEGATE` with `ATTRIBUTE`, so the
count stays where it was and nothing in it moved.

---

## Fix round 3

Prose only, which is also the answer to the sitting question: **no sitting**,
because nothing this round changes what any binary executes -- the round's whole
diff is comments and one corpus-manifest sentence.

### N1. The accumulated ratio is over 1% on two axes, and this task says so

The disclosure the plan's guard rule asks for was missing and is now in the
round-2 sitting section above, with the figures derived from the TSV rather than
recalled: `dispatchclass` at 1.012257 to 1.012597 and `rexxcps` at 1.015430 and
1.018638, against this task's contribution of nil. **`dispatchclass` crossed
during Task 14** -- Task 13's fixround sits at 1.009267 to 1.009953 and Task 14's
own contribution is 1.002149 to 1.002492 -- so Task 14 owns the answer.
**`rexxcps` has rows under `task` values `15` and `15-fixround-2` only**, because
the axis joined the guard here, so its reading is the phase's whole accumulated
drift on a path nothing measured before and no task can be named for it. The
plan already licenses that: a first reading is a position rather than a delta.

### N2. The provenance sentence in `GeneratedMethod`'s doc

It called `task=15-breach` "the sitting behind those figures". The totals above
it are direct `perf stat` readings and each differs from its committed
counterpart by about five million, so the committed rows did not produce them.
The doc now names the direct runs as their source and the committed sitting as
the re-measurement of the same comparison, which reproduces the 121 per send and
the ordering of the attributions rather than these totals. **The numbers were
not adjusted to match.**

### N3. The shape round 1's own sweep relocated

"beside the two fields" stood twice in `lib.rs`, counting
`InstalledMethodBody`'s fields two and six lines above the struct that names
them -- and one of those was added by the sweep commit itself. Both now name
`program` and `directive`.

**The report sentence is now derived from the command, not from recollection.**
The sweep is committed as a script in the session scratchpad and reads:

```bash
git diff 979522f74 -- crates/ corpus/ \
  | grep '^+' \
  | grep -E '^\+[[:space:]]*(//|#|\*)' \
  | grep -inE '\<(one|two|three|four|five|six|seven|eight|nine|ten)\>[[:space:]]+[A-Za-z_]+s\>'
```

Before this round I read its whole output as back-references. Several were not,
and each is rewritten: the field counts in the width assertion's doc and in
`GeneratedMethod`'s; `Interp::accessor_variable`'s "own two reads";
`a_generated_accessor_pair_reads_and_writes_the_declaring_scopes_pool`'s "one
property with two halves"; `GeneratedMethod`'s "mints two ids" and "add two
names"; `install_attribute`'s "the `Both` style's two names";
`a_method_attribute_installs_a_getter_and_a_setter`'s "recorded as two getters";
`generated_kinds`' "two halves of the same kind"; "Two tables recover all of
it"; and the corpus manifest's "one declaration over two receivers". After this
round the same command returns:

```
52:+    /// so the two arms of that `if` are the same read. `::ATTRIBUTE`'s
144:+            // leading one is.** Measured, both at rc 163: `(,)` reaches the
167:+        // **Installing one is not sending one**: the directive is rc 0 with
315:+        // **The refusal is a divergence from the oracle and is the one this
```

Three of the four are the pattern matching a pronoun or an adjective rather than
a count -- "a leading one", "installing one is not sending one", "is the one this
crate chooses". The fourth counts the arms of an `if`, which is not a set the
code enumerates and cannot change, and the split it refers to is named in the
sentence before it. That is a classification of the command's output, not a
claim that the output is empty.

### N4 and N6, taken because I was already in the file

**N4.** "Files changed" named the `15` sitting and the `15-breach` rows; it now
names `15-fixround-2` as well.

**N6.** `install_attribute`'s comment said the presence of a body decides "for
each style". It decides for `GET` and `SET` (`hasBody()` at
`DirectiveParser.cpp:1773` and `:1836`); the `BOTH` style admits no body at all,
`checkDirective` refusing one at `:1670`. The comment now says that, with the
refusal measured rather than guessed: `::attribute a class` with a following
clause is `Error 99.937: Attribute methods without a SET or GET designation
cannot have a method body.` **My first draft of that sentence wrote 99.934**,
carried over from a neighbouring doc about `::METHOD ... ATTRIBUTE`, and the
measurement is what caught it before it shipped.

### N5, left as the re-review parked it

The `cycles:u` interval quoted in the sitting section is the `small` rows, set
beside a `dispatchclass` range that spans all four cells. Across both sizes the
interval is wider, 0.931535 to 1.059877, which makes the argument stronger
rather than weaker. Not changed, and recorded here so the mismatch is visible
rather than silent.

### Gates

`cargo fmt --all --check` clean, `cargo clippy --workspace --all-targets --
-D warnings` clean, and `cargo doc --no-deps -p rexx-exec
--document-private-items` at 15 unresolved links with none naming a type or
field of this task's. The three test gates are recorded in the status reply.
