# Task 17 review: the environment search order, the Directory entry methods, and `.METHODS`

Range `65d333a59..a1c659f0a`. Reviewed at `a1c659f0a`, tree clean before and after.

## Verdicts

**Spec compliance: PASS, with one required correction (F1).** The brief's four programs match the
oracle byte for byte on both engines, the agreement to preserve is still green, both required
controls fire and isolate exactly the line they are meant to isolate, the seam obligation is met
untouched, and the sitting reproduces cell for cell from the TSV. One added comment cites a C++ line
pair that belongs to a branch the sentence's own measured example does not take.

**Task quality: PASS.** The widened scope is the right call and was done correctly. Every instrument
the report names fires when the behaviour is wrong, the sole-instrument status for `.RESOURCES` is
recorded in four places rather than implied, and the report volunteers its own gaps. The remaining
findings are prose-level: four report sentences that do not reproduce as written, and the recurring
cardinality-in-a-comment rule.

## Findings

### F1. `environment.rs` cites the EXTERNAL branch for a plain `::METHOD ... ATTRIBUTE` (required fix)

`rust/crates/rexx-exec/src/environment.rs`, the `DirectiveKind::Method` arm of
`package_table_entries`:

> `ATTRIBUTE` files the accessor pair under both names, which is `methodDirective`'s `addMethod`
> calls at `parser/DirectiveParser.cpp:875` and `:880`. Measured, `::method z attribute` puts `Z`
> and `Z=` in `.METHODS`.

`:875` and `:880` are `addMethod` calls in `methodDirective()` (which opens at `:629`), but they sit
inside `if (externalname != OREF_NULL)` at `:860` -- the `::METHOD ... ATTRIBUTE EXTERNAL ...` path.
A plain `::method z attribute`, the case the sentence measures, takes the `else` at `:889` and files
the pair through `createAttributeGetterMethod` (`:895`, whose own `addMethod` is at `:2418`) and
`createAttributeSetterMethod` (`:896`, `addMethod` at `:2474`).

The behavioural claim is true and the measurement is true; only the citation is wrong, and it is
wrong in the way the last task's was -- it names lines that do the described thing on a path the
reader's example never reaches.

### F2. "No corpus program in the tree has ever contained a `::RESOURCE` directive" is too wide

Report, under "`.RESOURCES` has no corpus row". `rust/corpus/gate-tables/directives/
resource__end__subkeyword.rex` contains a `::RESOURCE` with a body line, and
`resource__library__subkeyword.rex` contains one with an empty body.

The operative claim survives, because `every_corpus_program_tiles`'s `corpus_dir()` is
`corpus/lang` alone (`crates/rexx-parse/tests/gate_walk/mod.rs:398`), so the gate-table programs are
outside it. **I checked the operative claim by running it**: a `::resource "x"` program with a
two-line body planted in `corpus/lang/` produced exactly **19 tiling violations**, the report's own
figure, one per non-whitespace byte of the body and the `::END` marker. Probe removed, tree clean.

### F3. The staleness arithmetic does not reproduce at HEAD

The report gives `git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock
Cargo.toml` as listing 81 commits. At `a1c659f0a` it lists **83**. 81 is the count at BASE
`65d333a59`; the two extra are `cd06c85c9` and `e5fc02186`, the task's own, both named in the plan.
The conclusion -- no foreign commit beneath the phase -- is unaffected. The sentence states a
HEAD-relative command and a BASE-relative number.

### F4. "The bodies are unchanged apart from `hash_entry_read` gaining a comment" is not literally right

Two message strings also changed:

* `hash_entry_write`'s `Loud::receiver_class` text, `"a value that is not a directory"` ->
  `"a value that is not a hash collection"`;
* `native_object_name_set`'s `unreachable!`, `"a package or directory receiver is Body::Native"` ->
  `"each of these receivers is Body::Native"`.

Both arms are unreachable -- every caller has already classified the receiver as
`Primitive::Directory` or `Primitive::StringTable`, both of which are `Body::Native` by construction
-- and neither string is pinned anywhere under `crates/`, so nothing observable rode along. Both
edits are correct for the widened receiver set. The report's summary sentence is what is wrong.

### F5. "Every cell is 1.000000 with a min and max that round to the same figure"

`alloc4c` `ir` `small` on the `base>changed` arm has min `0.999999`, which does not round to
`1.000000` at the printed precision. Everything else in that table does. Read from
`bench-baselines/phase-5a-arms.tsv`.

Related, smaller: "Three cells are at or above 1%" is six cells once `size` is kept as a column --
three `(axis, arm)` combinations across two sizes. The six figures listed immediately after
disambiguate it.

### F6. Cardinality in added comments

Clearest instances, all new in this range:

* `crates/rexx-exec/tests/coverage.rs`: "the two package tables a corpus program can carry",
  directly above the four-entry enumeration it counts;
* `corpus/phase-5a.txt`: "The two steps of the order a program with no ::CLASS can observe";
* `corpus/lang/environment_local_shadows_the_environment.rex`: "at the two steps this phase can
  observe", "only the last two are reachable";
* `corpus/lang/environment_methods_join.rex`: "::CONSTANT files one getter";
* in `src/`: "One getter, `createConstantGetterMethod`'s own ...", "Both access scopes, not the
  public ones alone", "**Both directions fold the name to upper case**", "so both arms name a step
  of that send".

**Calibration, because it changes what to do about it.** The pre-existing neighbourhood in both
`corpus/phase-5a.txt` and `coverage.rs` is thick with the same shape -- the manifest already carries
"the two ways it fails", "the three ways a primitive method's", "the four directories", and
`coverage.rs`'s own module doc carries "five pinned items" and "six out-of-scope variants". So this
is a consistency question for the whole file rather than a regression this task introduced, and the
`src/` instances are all the "both"/"one" form rather than the numeral form the rule's examples give.
The one I would actually change is `coverage.rs`'s, which counts a set the four lines under it
enumerate.

### F7. Pre-existing, carried as unchanged context: `CoreClasses.orx:66`

`dispatch.rs`'s `native_hash_put` doc says the item/index order is what `CoreClasses.orx:66` writes,
quoting `.environment~put(class, name)`. That statement is at **`:65`**; `:66` is
`rexxPackage~addPublicClass(name, class)`. The line arrives in this diff only as context. Not this
task's, recorded so it does not get certified again by being read past.

## What I verified, and how

### C++ and `.orx` citations

Every new citation re-read with `/bin/grep`/`sed` against `/home/moritz/dev/repos/ooRexx/interpreter/`.

Correct: `memory/Setup.cpp:881` (`InheritInstanceMethods(IdentityTable)` in the `StringTable`
block), `:883` (`AddMethod("Unknown", StringHashCollection::unknownRexx, 2)`), `:933`
(`InheritInstanceMethods(StringTable)` in the `Directory` block);
`classes/support/HashCollection.cpp:1015` (`StringHashCollection::unknown`), `:1020`
(`msgname->endsWith('=')`), `:1026` (`RexxObject *value = arguments[0];`), `:824`
(`entry` = `get(index->upper())`), `:854` (`setEntry`);
`parser/DirectiveParser.cpp:617` (`unattachedMethods->setEntry(name, method)`), `:2536`
(`addMethod(name, method, false)` under `activeClass == OREF_NULL`), `:2344`
(`resources->put(resource, internalname)`); `RexxClasses/CoreClasses.orx:73`
(`.String~defineClassMethod(name~upper, .methods[...])`).

`parser/LanguageParser.cpp:1893` is the `if (!routines->isEmpty())` guard; the assignment
`package->routines = routines` is at `:1895`. Same statement group, usable, worth tightening if the
line is touched anyway. I also confirmed the runtime end the sentence asserts: `.ROUTINES` reaches
`settings.parentCode->getRoutines()` (`execution/RexxActivation.cpp:2877`-`:2879`) ->
`package->getRoutines()` -> the `routines` field, and `publicRoutines` is a separate field nothing
on that path reads.

Also checked because the crate depends on it: `activeClass` is assigned only at
`parser/DirectiveParser.cpp:355` (`::CLASS`) and cleared only in
`parser/LanguageParser.cpp:843`/`:1112`, both before any directive is processed. So it is never
reset mid-file, and `package_table_entries`'s monotone `seen_class` flag is the right model. The
negative witness for that flag already exists and is still green:
`corpus/lang/environment_methods_table_attached.rex`.

### The two required controls -- both run, both fire

Applied to the committed tree, corpus run in debug (`corpus.rs` runs the identical comparison in
both modes), then restored and rebuilt.

1. **`.ENVIRONMENT` before `.LOCAL`** -- `environment.rs:487` swapped to
   `[EnvScope::Environment, EnvScope::Local]`. Result: **189 of 190**, the single line
   `[UNCLASSIFIED] lang/environment_local_shadows_the_environment.rex: stdout differ`. I then ran
   that program under the mutated build from a fresh directory: rc 0, stdout
   `from environment` / `only in the environment` / `.NEITHERTHING`. **Only line 1 moves**, which is
   the report's claim exactly and is what makes the program a discriminator rather than a smoke test.

2. **`LOCAL` in `Directory`'s own dictionary** -- `registry.add_instance_method(class, "Local")` for
   the `Directory` block in `crates/rexx-classes/src/native_classes.rs`'s `replay`, plus a
   `("Directory", "LOCAL", Arity::Fixed(0), ...)` `NATIVE_METHODS` row answering
   `hash_entry_read(receiver, b"LOCAL")` so the entry still answers. Result: **189 of 190**, the
   single line `lang/environment_directory_entry_method.rex: stdout differ`. Running the program
   under the mutated build: rc 0, and the **only** changed byte in the whole program is line 2,
   `1` where HEAD and the oracle print `0`. The control isolates where the answer comes from rather
   than whether there is one, as reported.

Both mutations reverted; `sha256sum -c` against the pre-mutation record confirms
`environment.rs`, `dispatch.rs` and `native_classes.rs` are byte-identical to `a1c659f0a`, and
`git status` is clean.

### `.RESOURCES`'s sole instrument -- run, twice, and confirmed sole

`dispatch.rs`'s `the_package_tables_hold_what_their_directives_declare` reddens under both
mutations I applied to `package_table_entries`'s `Resource` arm, and in each case it was the **only**
failing test in the whole `rexx-exec` package under `--no-fail-fast`, with the corpus staying at
**190 of 190**:

| mutation | result |
| --- | --- |
| key lower-cased instead of upper-cased | 726 passed, 1 failed -- that test; corpus 190 of 190 |
| first body line dropped (`.skip(1)`) | 726 passed, 1 failed -- that test; corpus 190 of 190 |

That is both halves of what was asked: the witness can fail, and nothing else can see the defect, so
"in-crate only" is measured rather than assumed. The status is recorded in four places -- the test's
own doc, `corpus/phase-5a.txt`'s Task 17 block, `coverage.rs`'s `EXPECTED_SUBSET_5A` comment and the
report -- not implied.

**Its expectations are oracle-accurate.** I ran both programs the test embeds against the oracle from
a fresh directory: rc 0 and stdout
`The StringTable class / The Array class / 2 / line one / line two / line two / The NIL object /
The NIL object`, and rc 0 with `0` / `[]` for the empty body. Byte for byte what the test asserts.

**The reverted `coverage.rs` widening left nothing behind.** `is_admitted_directive_kind` has zero
hits in the range's diff and still reads `DirectiveKind::Resource(_) => false` at HEAD.

### The seam

`tests/environment_seam.rs` is untouched in the range (empty `--stat`), and `mod env_seam` has zero
hits in the `environment.rs` diff -- no item added, no `env_seam::admit(` or `env_seam::directory(`
call added. All three of its assertions ran green in the release gate below.

**And the seam is still on the path**, which is the part a green test cannot establish on its own.
`dot_variable` (`environment.rs:477`) still reads a directory only through the loop at `:487`-`:492`,
one `env_seam::admit` per scope, feeding the single `env_seam::directory` call at `:528`. The new
work never asks the seam for a directory because it never needs one: `native_hash_at`,
`native_hash_put` and `native_hash_unknown` all take the receiver the send already holds and go to
`hash_entry_read`/`hash_entry_write`, which is the position `~at` and `~put` were already in at BASE
and which `hash_entry_read`'s own doc states.

One route worth naming because it looks like a bypass and is not: `.environment~local` now hands a
program the `.local` object without any `admit` call. That entry was already in `.environment`'s map
at BASE (`environment.rs:389` at `65d333a59`, `(b"LOCAL", local)`) and `.environment["LOCAL"]` already
reached it through `native_directory_at`, so this task opens no new route to a directory handle. The
seam's invariant is over Rust code obtaining a handle from `Directories`, and that is intact.

### The sitting

Both tables in the report reproduce from `bench-baselines/phase-5a-arms.tsv` cell for cell, filtering
`instrument = instructions:u` and keeping `instrument` and `size` in the projection. `cycles:u` not
read as a result.

The accumulation attribution holds. All 24 `pinned>head` cells match Task 16's rows against the same
pin, and the six cells at or above 1% -- `compound` `ir` at both sizes, `strings` `tw` at both,
`strings` `ir` at both -- match to six decimals, with `strings ir large` matching to five and
differing by one unit in the last place. That is the report's sentence, verified.

**The 1.000000 contribution is genuine rather than a copied row.** The underlying `absolute` rows
differ between the two builds: `alloc4c tw small` is 2,527,484,804 for `base` against 2,527,484,279
for `changed`, a 2e-7 relative delta, and the other axes are the same order. A change whose only
executed-path cost is a handful of extra bootstrap hashes cannot move a six-decimal ratio on billions
of instructions, so 1.000000 everywhere is the expected shape. The pin itself checks out: sha256
`141c3fa96...` matches `PINNED.md`, and `git merge-base --is-ancestor 15a1ffa98 HEAD` succeeds.

### The renames

`native_directory_at`/`put` -> `native_hash_at`/`put`, `directory_index` -> `hash_index`,
`Interp::directory_entry_read`/`write` -> `Interp::hash_entry_read`/`write`. Read the diff hunk by
hunk: `hash_index`'s body is unchanged (doc and signature only), `native_hash_at` and
`native_hash_put` change only their callees' names, and `hash_entry_read`'s `directory_scope` guard
-- the per-directory unbuilt refusal -- is unchanged with a comment added saying why a package table
never takes it. The only body edits are the two unreachable strings under F4.

`package_table_root_key` cannot collide with `package_root_key`: "the methods/routines/resources of
program N" against "the package of program N" and "the REXX package".

### Extra oracle-differential I ran, beyond what the controller had already measured

All from fresh directories, absolute paths, three descriptors read separately, both sides bounded.

| probe | oracle | crate |
| --- | --- | --- |
| `say .METHODS` / `.ROUTINES` / `.RESOURCES`, no directive | rc 0, the three literal spellings | identical on `ir` **and** `tree-walker` |
| `.local~"MYTHING="('v')`, `.local~MYTHING = 'w'`, `.local~"mything="('x')`, then both index spellings | rc 0, `v` `w` `x` `[The NIL object]` | byte-identical on both engines (`cmp` clean) |
| `.environment~unknown('ARRAY')` | rc 163, 93.903 argument 2 required | byte-identical, traceback included |
| `.environment~unknown()` | rc 163, 93.903 argument 1 required | byte-identical |
| `say .methods~at()` | rc 168, scope `"StringTable"` | -- confirms the report's scope reading |
| `say .methods~z~class` | rc 0 `The Method class` | rc 120, empty stdout, refusal on stderr, both engines |

The second row matters because the quoted set form and its case folding are in
`an_entry_method_send_with_no_value_is_loud`'s answering rows but in no corpus program; they are
oracle-accurate.

The last row is the parked gap. It is loud on all three descriptors -- rc 120 against 0, empty
stdout against `The Method class`, and
`rexx-exec: a message send to one of the interpreter's own objects is not implemented (Phase 5)` on
stderr -- so it is a refusal and not a silent wrong answer. The loud message names Phase 5 as owner;
the report's Concern 2 describes the gap and the design choice it needs but does not name 5b or a
task. Worth a word in the ledger, not a defect.

One edge I chased and cleared: `native_hash_unknown`'s `strip_suffix(b"=")` would yield an empty
index for a message named exactly `=`, but that name resolves to `Object`'s `=` before `UNKNOWN` is
reached, on both sides -- oracle `rc 163` at the `~""` line, crate a pre-existing loud
`method "=" of class "Object" is not implemented`. Not reachable, no divergence, nothing to fix.

### Binding constraints

* **No `unsafe`** anywhere in the added lines, and no comment claims the lint rules anything out --
  no `forbid` in the diff at all.
* **ASCII, no em-dashes**: zero non-ASCII bytes in the four `.rex` programs, `phase-5a.txt`,
  `dispatch.rs`, `environment.rs` and `coverage.rs`. `lib.rs` has four, all pre-existing (no added
  line carries one).
* **No historical framing** in the added `src/` comments. The one marker match is
  `dispatch.rs`'s pointer to "the Task 17 block" of `corpus/phase-5a.txt`, which names where a note
  lives rather than what the code used to be; `dispatch.rs` already carried a `Task N` reference at
  BASE.
* **Set cardinality**: F6.

## Gates I ran

From `rust/`, at `a1c659f0a` with a clean tree:

```
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast    exit 0
                                                                      190 of 190 matching
                                                                      98 `test result: ok`, no FAILED
REXX_CORPUS_GATE=1 memcap 8G cargo test -p rexx-exec --test corpus --no-fail-fast
                                                                      190 of 190 matching, ok
```

I did not re-run `fmt` or `clippy`; the controller had already read them and I changed nothing that
could move them.

## The `arguments[0]` question: CONFIRMED, and worth filing

**`StringHashCollection::unknown` reads `arguments[0]` without consulting the argument count.**

* Declared `RexxObject *StringHashCollection::unknown(RexxString *msgname, RexxObject **arguments,
  size_t argCount)` at `classes/support/HashCollection.cpp:1015`. `argCount` is never read in the
  body: the assignment branch does `RexxObject *value = arguments[0];` at **`:1026`** and hands it to
  `setEntryRexx`, and the retrieval branch at `:1033` ignores the arguments entirely.
* Both entry points supply a count it discards. `processUnknown` (`:989`) forwards the interpreter's
  own argument array and `count` straight through, and `unknownRexx` (`:966`) forwards
  `argumentList->messageArgs()` with `argumentList->messageArgCount()`.
* **The intended behaviour is visible one function away.** `setEntry` (`:854`) opens with "set entry
  is a little different than put, in that the value argument is optional. no argument is a remove
  operation" and removes on `OREF_NULL`. So a value-less `NAME=` send is meant to remove the entry;
  the missing count check is exactly what stops it reaching that path.
* **Measured, independently of the report.** On the oracle, a fresh `.directory~new` with
  `d~mything = 'v'`, then `say 'a' d["MYTHING"]`, then `d~"MYTHING="()`, leaves `d["MYTHING"]`
  holding `a v` -- the string the intervening `SAY` had just built. rc 0, nothing on stderr.

That is a read one slot past the argument list, reachable from ordinary Rexx, landing whatever the
evaluation stack happens to hold into a collection. Real upstream defect. The crate's decision to
refuse it loudly is right, and `an_entry_method_send_with_no_value_is_loud` with its answering rows
beside it is the correct instrument for a case with no oracle behaviour to agree with.

## State of the tree

`a1c659f0a`, `git status` clean, no file left modified. All three mutations restored and verified
byte-identical by `sha256sum -c`. The `corpus/lang/` tiling probe was removed. Debug artifacts were
rebuilt from restored HEAD sources and the debug corpus reads 190 of 190; the release gate above was
run after all mutation work and exits 0, so `target/release` matches HEAD.
