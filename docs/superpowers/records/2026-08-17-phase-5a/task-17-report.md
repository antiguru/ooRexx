# Task 17 report: the environment search order, the Directory entry methods, and `.METHODS`

BASE `65d333a59`. Commits `cd06c85c9` (implementation), `e5fc02186` (corpus), `a1c659f0a`
(the two sittings' rows) and `dd83eecdc` (fix round 1). This report is not committed: `.superpowers/` is git-ignored in this
worktree, which is where every earlier task's report in this plan sits too.

Oracle at the time of every run below: `parse version` answers
`REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026`, and this crate answers the same string.

## What the brief asked for, and where each piece is

| brief item | where it landed |
| --- | --- |
| the order `.LOCAL` then `.ENVIRONMENT` | already in `Interp::dot_variable`; what it needed was a way to *set* an entry, which is the row below. `corpus/lang/environment_local_shadows_the_environment.rex` |
| the Directory entry-method mechanism | `native_hash_unknown` (`dispatch.rs`), rows for `Directory~UNKNOWN` and `StringTable~UNKNOWN`. `corpus/lang/environment_directory_entry_method.rex` |
| the `.METHODS` join | `Interp::package_string_table` and `package_table_entries` (`environment.rs`), plus `Primitive::StringTable` and the `StringTable` `[]`/`AT`/`PUT` rows. `corpus/lang/environment_methods_join.rex` |
| the agreement to preserve | `environment_methods_table_attached.rex` and `environment_methods_table_unattached.rex`, unchanged and still green |

**The order itself needed no code change.** `Interp::dot_variable` already asks `.LOCAL` and then
`.ENVIRONMENT` in that loop, and both engines already answered the second step. What could not be
written was a *program that observes it*: putting a name in `.LOCAL` is a `MYTHING=` send to one of
the interpreter's own objects, and that send is what this task built. The brief says this in its
first bullet and it is worth restating, because the diff contains no change to the order.

## The mechanism, and the C++ it is read off

`StringTable` declares `Unknown` (`memory/Setup.cpp:883`), and `Directory` takes it with
`InheritInstanceMethods(StringTable)` (`:933`), so it lands in each class's **own** dictionary --
which is what the traceback says. Measured on the oracle: `.methods~at()` reports
`Compiled method "AT" with scope "StringTable".` where `.environment~at()` reports `"Directory"`.

`StringHashCollection::unknown` (`classes/support/HashCollection.cpp:1015`) is the whole body: a
message name ending in `=` (the test is at `:1020`) stores the send's own first argument under the
name without it, and every other name reads the entry. Both directions go through `entry`/`setEntry`,
which are `get`/`put` with `index->upper()` (`:824`, `:854`) -- so the entry route folds case where
`~at` and `~put` match verbatim. Measured: `.local~"mything="('v')` then `.local["MYTHING"]` is `v`
and `.local["mything"]` is `The NIL object`.

## Why `.ROUTINES` and `.RESOURCES` are in this task at all

They are not in the brief. They are here because **the brief's `.METHODS` work cannot be done
without them.**

`Primitive::Directory`'s own doc comment at BASE said so already: `.methods`, `.routines` and
`.resources` kept the loud arm because "this crate populates none of those tables ... so answering
`~at` on one would answer `.nil` for an index the oracle has an entry for". Admitting a `StringTable`
as a receiver -- which `.methods~class`, `.methods~z` and `.methods["Z"]` all require -- admits all
three, because the receiver test is by class and the three tables are one class. Leaving the other
two empty would have turned that doc comment's warning into three silent wrong answers at rc 0:
measured on the oracle, `.routines~r` is `a Routine` and `.resources~x` is an `Array` of the
resource's own lines.

So all three are filled from the directives that declare them:

* `.METHODS`: a `Method` for every floating `::METHOD`, every `::ATTRIBUTE` accessor the style
  declares, and every `::CONSTANT`. Keys upcased. Measured against the oracle, each shape:
  `::attribute zz` files `ZZ` and `ZZ=`, `::attribute zz get` files `ZZ` alone, `::attribute zz set`
  files `ZZ=` alone, `::method z attribute` files the pair, `::constant c` files `C`,
  `::method "MiXeD"` files `MIXED`, and a `::METHOD` under a `::CLASS` files nothing.
* `.ROUTINES`: a `Routine` per `::ROUTINE`, both access scopes -- `package->routines`
  (`parser/LanguageParser.cpp:1893`), not `publicRoutines`.
* `.RESOURCES`: each `::RESOURCE` body as an `Array` of its own lines
  (`parser/DirectiveParser.cpp:2344`).

**One object per package, not one per evaluation.** Measured, `.methods~identityHash` answers the
same number twice in a row on the oracle, and `identityHash` is a method this crate implements, so a
fresh object per evaluation would have been observable the moment a `StringTable` became a receiver.
Each table is now built once, keyed `(ProgramId, PackageTable)`, and rooted through
`RootSet::add_global` -- the position `Interp::package_objects`'s entries are already in.

## What I tested

### The four programs the brief names, both engines, three descriptors read separately

Every run below used the mandated wrappers, from a fresh directory created for that program alone,
with absolute paths and stdout/stderr never merged.

| program | oracle | crate, both engines |
| --- | --- | --- |
| `.local~MYTHING=` / `.environment~MYTHING=` / `say .MYTHING` | rc 0 `from local` | identical |
| `say .environment~local~class` / `say .environment~hasMethod("LOCAL")` | rc 0 `The Directory class` then `0` | identical |
| `.methods~class` / `~z` / `["Z"]` / `~q` / `~hasMethod("Z")` | rc 0 `The StringTable class`, `a Method`, `a Method`, `The NIL object`, `0` | identical |
| `say .METHODS` / `say .ROUTINES`, no floating directive | rc 0 `.METHODS` then `.ROUTINES` | identical, unchanged |

All four reproduced the controller's own pre-dispatch readings at BASE before I changed anything,
including the two different refusal routes the notes call out.

### Beyond them

Differentialled on both engines, all matching: the `::ATTRIBUTE`/`::CONSTANT`/quoted-name key set;
`::method z attribute`; `.routines` under both access scopes and both index spellings; `.resources`
including `~items`, `~at`, `~makeString('C')` and `~makeString('L','/')` on the array it answers, and
the empty body; `.methods~identityHash` twice; `.methods~put` then reading the entry back;
`.environment~"local"`, `.environment~"ARRAY"`, `.environment~nosuch`, `.local~nosuch`,
`.methods~nosuch`; `.methods~isA(.StringTable)`, `~isA(.Directory)`, `~string`,
`~request('STRINGTABLE')`; `.methods~objectName = 'renamed'` then reading it back; and
`.environment~unknown('ARRAY', .Array~superClasses)`.

### The entry-method route inherits the unbuilt refusal

`.environment` and `.local` hold names the oracle answers and this crate builds nothing for, and
`hash_entry_read`'s per-directory `Unbuilt` check is what keeps those loud rather than `.nil`. The
message route goes through the same function, so it inherits it. Measured: `say .environment~alarm`
is `rexx-exec: directory entry "ALARM" is not implemented (Phase 5)` at rc 120 where the oracle
answers `The Alarm class`, and `say .local~stdout` is the same with `(Phase 7)` against the oracle's
`STDOUT`. Both are refusals, not answers, and the owner is the one that directory's own list carries.

### What is still refused, and what each refusal's instrument is

The corpus gate cannot see a clean refusal becoming a wrong answer, so each of these says its own
instrument explicitly.

* **`~ITEMS` on either class** -- `.environment~items` was loud at BASE and `.methods~items` is loud
  now, both `method "ITEMS" of class "..." is not implemented (Phase 5)`. This is the pre-existing
  shape for every `HashCollection` method with no `NATIVE_METHODS` row; nothing in this task changes
  it and no new instrument is owed.
* **A `Method` or a `Routine` as a receiver** -- `.methods~z~class` is rc 120
  `a message send to one of the interpreter's own objects` where the oracle answers
  `The Method class`. **This is a gap this task moved rather than opened**: at BASE `.methods~z`
  itself was refused. It is the position `.context` is in, and nothing in this phase needs a message
  sent to a `Method` (`CoreClasses.orx:73` passes one as an argument to `~defineClassMethod`).
  Instrument: none beyond `receiver_kind`'s own arm; stated here rather than claimed covered.
* **The set form with no value argument** -- `d~"MYTHING="()`, `(,)` and `(,,)`. **There is no oracle
  behaviour to match.** `unknown` reads `arguments[0]` without consulting the argument count
  (`HashCollection.cpp:1026`), so the oracle stores whatever was on the stack: measured,
  `d~mything = 'v'` then `say 'a' d["MYTHING"]` then `d~"MYTHING="()` leaves the entry holding the
  string `a v` -- the `SAY`'s own value. Instrument: `dispatch.rs`'s
  `an_entry_method_send_with_no_value_is_loud`, which is the whole of it, with the answering
  spellings beside it so a build refusing the entire set form fails rather than passes.
* **`~UNKNOWN` sent by hand with a non-`Array` second argument** -- the oracle converts with
  `requestArray` and this crate implements `MAKEARRAY` for no receiver, so it is the same refusal
  `~request('ARRAY')` already gives. Instrument: `dispatch.rs`'s
  `an_unknown_sent_by_hand_needs_an_array_this_crate_does_not_convert`, with the array-argument row
  that answers beside it.

### `.RESOURCES` has no corpus row, and cannot have one

`rexx-parse`'s `every_corpus_program_tiles` requires every byte of a corpus program to be covered by
a clause node, and a `::RESOURCE` body is source lines the parser never turns into clauses. Measured:
a first draft of `environment_routines_and_resources.rex` produced 19 tiling violations, one per
uncovered byte of the body and the `::END` marker. No program under `corpus/lang/` contains a
`::RESOURCE` directive, which is why -- and `corpus/lang/` is the whole of what that gate walks
(`crates/rexx-parse/tests/gate_walk/mod.rs:399`), so
`corpus/gate-tables/directives/resource__end__subkeyword.rex` and
`resource__library__subkeyword.rex`, which do contain one, are outside it.

So `.RESOURCES`'s rows are `dispatch.rs`'s `the_package_tables_hold_what_their_directives_declare`,
whose expectations were each measured on the oracle, and `corpus/phase-5a.txt` says so at the Task 17
block so a reader looking for the witness is pointed at it.

**A first attempt widened `coverage.rs`'s `is_admitted_directive_kind` to admit `::RESOURCE`** -- the
remedy that file's own assertion message names -- and I reverted it once the tiling gate showed the
program could not be a corpus entry regardless. `coverage.rs`'s pinned directive-kind expectation is
untouched at HEAD.

### The two controls the brief requires

Both were applied to the committed tree, measured, and reverted; the tree is back to `190 of 190`.

1. **Resolving `.MYTHING` from `.ENVIRONMENT` first.** Swapping `dot_variable`'s loop to
   `[EnvScope::Environment, EnvScope::Local]`: the shadowing program prints `from environment` where
   the oracle prints `from local`, and the corpus reads **189 of 190**, the one being
   `lang/environment_local_shadows_the_environment.rex: stdout differ`. The other two lines of that
   program are unmoved, which is what makes the program a discriminator rather than a smoke test.
2. **Answering `.environment~local` through the behaviour.** Adding `LOCAL` to `Directory`'s own
   instance dictionary in `rexx-classes`: `hasMethod("LOCAL")` becomes `1` where the oracle says `0`,
   and the corpus reads **189 of 190**, the one being
   `lang/environment_directory_entry_method.rex: stdout differ`. The entry still answers, which is
   the point: only the `hasMethod` line moves, so the control isolates *where* the answer comes from
   rather than *whether* there is one.

### The seam

`tests/environment_seam.rs` is untouched and still passes. This task added no item to `mod env_seam`,
no second `env_seam::admit(` call and no second `env_seam::directory(` call: `native_hash_unknown`
reaches an entry through `Interp::hash_entry_read`/`hash_entry_write`, which take the receiver the
send already holds and do not ask the seam for a directory -- the position `~at` and `~put` were
already in, and `hash_entry_read`'s own doc gives the C++ reason (the oracle asks its manager per
`getLocal`/`getEnvironment` and not per `~at`). All three of that file's assertions ran green in
every gate below.

### Rooting

`package_string_table` allocates the table, roots it globally, records it, and only then allocates
each value, storing each into the rooted table before the next allocation. `Body::Native`'s `trace`
walks its entries, so a stored value is reachable. The line arrays are built under a temporaries
frame. `collect_stress.rs` -- collect on every allocation -- runs all four new corpus programs and is
green in both gated runs, which is the instrument for this.

`NO_ALLOCATION_PROGRAMS` gained nothing: every new program allocates.

### What these checks could not see

* The corpus differential cannot see a refusal the oracle does not share, which is why each refusal
  above names an in-crate instrument or says there is none.
* `collect_stress` proves the new allocations are rooted **on the paths those four programs take**.
  It says nothing about the `.RESOURCES` path, which no corpus program reaches; the in-crate test for
  that runs under the ordinary allocator only.
* The two controls each redden exactly one program. Neither says the *rest* of the corpus is
  insensitive to the change they make -- they say the program named is the one that moved, which is
  what `189 of 190` reports.

## Sitting

Instrument `instructions:u` throughout; `cycles:u` was recorded and is not read as a result on its
own. Every figure is the median of 5 rounds, from `bench-baselines/phase-5a-arms.tsv`.

**The pin is not stale, checked rather than assumed.** `bench-baselines/pinned/rexx-run-15a1ffa98`
has sha256 `141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, matching `PINNED.md`.
`git merge-base --is-ancestor 15a1ffa98 HEAD` succeeds. `git log --oneline 15a1ffa98..65d333a59 --
rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml` -- the list as it stood at BASE, which is
what the sitting's `pinned` arm is measured against -- lists 81 commits, and **every one of them is
named in this plan's own SDD records** -- checked by script against every `*.md` in
`.superpowers/sdd/2026-08-17-phase-5a/`, by both short and full hash. Nine of the 81 are named in a
task report rather than in `progress.md` itself (Task 12's `ee6ebbf64` and `0ce35233e`, Task 13's
`b84685f44`, `6d12c033d`, `d3758d520` and `ca7008069`, Task 16's `ebe91e39b`, `1022bc885` and
`8265ead58`); no commit in the list is foreign to the phase. Run with `..HEAD` at `a1c659f0a` the
same command lists 83, the extra two being this task's own `cd06c85c9` and `e5fc02186`.

**Two sittings, interleaved within each.** The brief's command compares `pinned` against `head` and
so measures the accumulation of every task since the pin; my own contribution needs the build
immediately before this task, so a second sitting compares `base` (`65d333a59`, built from a
throwaway worktree, sha256 `1bcf62b19b2a6dc7cfce587ce657a2c6e42ad787decfa262c602d3a20c54e4ca`)
against `changed` (this tree). No figure is compared across the two sittings.

```
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-15a1ffa98 \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup \
    --rounds 5 --task 17 --commit e5fc02186 --baseline bench-baselines/phase-5a-arms.tsv

./target/release/rexx-arms --build base=<scratch>/rexx-run-base-65d333a59 \
                           --build changed=target/release/rexx-run \
    <the same six axes> --rounds 5 --task 17-contribution --commit e5fc02186 \
    --baseline bench-baselines/phase-5a-arms.tsv
```

### Accumulated: `pinned>head`, `instructions:u`

```
axis       arm size   ratio      [min..max]
alloc4c    tw  small  1.003574   [1.003574..1.003575]
alloc4c    ir  small  1.004807   [1.004806..1.004807]
alloc4c    tw  large  1.003483   [1.003483..1.003483]
alloc4c    ir  large  1.004622   [1.004622..1.004623]
arith      tw  small  0.999969   [0.999968..0.999969]
arith      ir  small  0.993067   [0.993067..0.993067]
arith      tw  large  0.999934   [0.999934..0.999934]
arith      ir  large  0.993017   [0.993017..0.993017]
compound   tw  small  1.006965   [1.006965..1.006965]
compound   ir  small  1.010472   [1.010472..1.010472]
compound   tw  large  1.006966   [1.006965..1.006966]
compound   ir  large  1.010473   [1.010473..1.010473]
emptyloop  tw  small  0.995434   [0.995434..0.995435]
emptyloop  ir  small  0.992023   [0.992022..0.992023]
emptyloop  tw  large  0.995434   [0.995434..0.995434]
emptyloop  ir  large  0.992022   [0.992022..0.992022]
strings    tw  small  1.008736   [1.008736..1.008736]
strings    ir  small  1.013411   [1.013411..1.013411]
strings    tw  large  1.008736   [1.004294..1.008736]
strings    ir  large  1.013411   [1.013410..1.013411]
varlookup  tw  small  0.997161   [0.997161..0.997161]
varlookup  ir  small  0.994260   [0.994260..0.994260]
varlookup  tw  large  0.997161   [0.997161..0.997161]
varlookup  ir  large  0.994260   [0.994260..0.994260]
```

**Six cells are at or above 1%: `compound` `ir` and `strings` on both arms, each at both sizes.**
They are not this task's. The identical cells stand in the TSV at `task = 16`, measured against the
same pin before this task existed: `compound` `ir` 1.010472 small and 1.010473 large, `strings` `tw`
1.008736 at both sizes, `strings` `ir` 1.013411 small and 1.013410 large. Every one of those matches the row above it to six decimal places, one
of them to five with the sixth differing by a unit in the last place. So the accumulation this
sitting reports was already standing at BASE, and the tie-breaker for it is the contribution sitting
below rather than a layout argument.


### This task's own contribution: `base>changed`, `instructions:u`

```
axis       arm size   ratio      [min..max]
alloc4c    tw  small  1.000000   [1.000000..1.000000]
alloc4c    ir  small  1.000000   [0.999999..1.000000]
alloc4c    tw  large  1.000000   [1.000000..1.000000]
alloc4c    ir  large  1.000000   [1.000000..1.000000]
arith      tw  small  1.000000   [1.000000..1.000000]
arith      ir  small  1.000000   [1.000000..1.000000]
arith      tw  large  1.000000   [1.000000..1.000000]
arith      ir  large  1.000000   [1.000000..1.000000]
compound   tw  small  1.000000   [1.000000..1.000000]
compound   ir  small  1.000000   [1.000000..1.000000]
compound   tw  large  1.000000   [1.000000..1.000000]
compound   ir  large  1.000000   [1.000000..1.000000]
emptyloop  tw  small  1.000000   [1.000000..1.000000]
emptyloop  ir  small  1.000000   [1.000000..1.000000]
emptyloop  tw  large  1.000000   [1.000000..1.000000]
emptyloop  ir  large  1.000000   [1.000000..1.000000]
strings    tw  small  1.000000   [1.000000..1.000000]
strings    ir  small  1.000000   [1.000000..1.000000]
strings    tw  large  1.000000   [1.000000..1.000000]
strings    ir  large  1.000000   [1.000000..1.000000]
varlookup  tw  small  1.000000   [1.000000..1.000000]
varlookup  ir  small  1.000000   [1.000000..1.000000]
varlookup  tw  large  1.000000   [1.000000..1.000000]
varlookup  ir  large  1.000000   [1.000000..1.000000]
```

**Every cell's median is 1.000000**, and every min and max rounds to the same figure but one:
`alloc4c` `ir` `small` has min `0.999999`. So this task adds no retired instruction to any path these
six axes execute, and the accumulated cells above are entirely
earlier tasks'. That is the expected shape rather than a surprise: `NATIVE_METHODS` gains rows that
are hashed once at bootstrap, `receiver_kind` gains one comparison reached only by a `Body::Native`
receiver (which none of these axes construct), and `package_string_table` is reached only by a
`.METHODS`/`.ROUTINES`/`.RESOURCES` lookup.

**What this could not see**: the axes are the six the guard names, and none of them sends a message
to a directory or resolves a package table. A cost inside the code this task added would not appear
here at all -- what the sitting establishes is that the *existing* hot paths are untouched, which is
what the guard is for.


## Commits

| commit | what |
| --- | --- |
| `cd06c85c9` | `dispatch.rs`, `environment.rs`, `lib.rs` -- the entry-method mechanism, `Primitive::StringTable`, and the filled package tables |
| `e5fc02186` | the four corpus programs, their `sourceline_oracle` expectations, `phase-5a.txt` and `coverage.rs`'s `EXPECTED_SUBSET_5A` |
| `a1c659f0a` | `bench-baselines/phase-5a-arms.tsv` -- the two sittings' rows |
| `dd83eecdc` | fix round 1: the F1 citation in `environment.rs`, F7's `CoreClasses.orx:65` in `dispatch.rs`, F6's cardinality in `coverage.rs` |

## Files changed

* `rust/crates/rexx-exec/src/dispatch.rs`
* `rust/crates/rexx-exec/src/environment.rs`
* `rust/crates/rexx-exec/src/lib.rs`
* `rust/crates/rexx-exec/tests/coverage.rs`
* `rust/corpus/phase-5a.txt`
* `rust/corpus/lang/environment_local_shadows_the_environment.rex`
* `rust/corpus/lang/environment_directory_entry_method.rex`
* `rust/corpus/lang/environment_methods_join.rex`
* `rust/corpus/lang/environment_routines_table.rex`
* `rust/crates/rexx-parse/tests/sourceline_oracle/environment_local_shadows_the_environment.txt`
* `rust/crates/rexx-parse/tests/sourceline_oracle/environment_directory_entry_method.txt`
* `rust/crates/rexx-parse/tests/sourceline_oracle/environment_methods_join.txt`
* `rust/crates/rexx-parse/tests/sourceline_oracle/environment_routines_table.txt`

### Renames, so a reviewer reading a diff hunk knows why

`native_directory_at` -> `native_hash_at`, `native_directory_put` -> `native_hash_put`,
`directory_index` -> `hash_index`, `Interp::directory_entry_read` -> `Interp::hash_entry_read`,
`Interp::directory_entry_write` -> `Interp::hash_entry_write`. Each now serves a `StringTable`
receiver as well as a `Directory` one, because the C++ behind them is one `HashCollection` function
reached through two donations. The bodies are unchanged apart from `hash_entry_read` gaining a
comment saying why the unbuilt-entry refusal is a directory's alone, and two message strings that
widened with the receiver set: `hash_entry_write`'s `Loud::receiver_class` text, `"a value that is
not a directory"` -> `"a value that is not a hash collection"`, and `native_object_name_set`'s
`unreachable!`, `"a package or directory receiver is Body::Native"` -> `"each of these receivers is
Body::Native"`. Both arms are unreachable -- every caller has already classified the receiver as
`Primitive::Directory` or `Primitive::StringTable`, and both are `Body::Native` by construction --
and neither string is pinned anywhere under `crates/`, so nothing observable moved.

## Gates

Run from `rust/`, each read unpiped.

```
cargo fmt --all --check                                        exit 0
cargo clippy --workspace --all-targets -- -D warnings           exit 0
cargo test --release --workspace --no-fail-fast                 exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast   exit 0, 190 of 190 matching
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast   exit 0, 190 of 190 matching,
                                                                     98 `test result: ok`, no FAILED
```


## Fix round 1

Review at `task-17-review.md`, both verdicts PASS. Seven findings; three were code, four were report
sentences that did not reproduce as written.

**F1 (required), `environment.rs`.** The `DirectiveKind::Method` arm cited
`parser/DirectiveParser.cpp:875` and `:880` for `ATTRIBUTE` filing the accessor pair. Those two
`addMethod` calls are real and they file both names, but they sit inside `if (externalname !=
OREF_NULL)` at `:860` -- the `EXTERNAL` path, which the sentence's own measured example
(`::method z attribute`) does not take. Re-read with `sed -n '855,900p'`: that spelling takes the
`else` at `:889` and files through `createAttributeGetterMethod` (`:895`) and
`createAttributeSetterMethod` (`:896`), whose own `addMethod` calls are at `:2418` and `:2474`
(both re-read, both are `addMethod(name, _method, classMethod);` under a `setAttribute()`). The
comment now cites those four lines and says which branch the example takes. The behaviour and the
measurement were right and are unchanged.

**F7, `dispatch.rs`.** `native_hash_put`'s doc cited `CoreClasses.orx:66` for
`.environment~put(class, name)`; that statement is at `:65` and `:66` is
`rexxPackage~addPublicClass(name, class)`. Re-read with `sed -n '63,67p'`. Pre-existing, arriving as
context, fixed rather than read past again.

**F6, `coverage.rs`.** "the two package tables a corpus program can carry", sitting above the
four-line enumeration that counts them, now names them instead: "the `.METHODS` and `.ROUTINES`
tables". The reviewer's calibration was to change this one and not sweep the file, and I did not
sweep it.

**F2 through F5, report only.** Each was checked by running rather than by rereading:

* F2: "No corpus program in the tree has ever contained a `::RESOURCE` directive" is too wide --
  `corpus/gate-tables/directives/resource__end__subkeyword.rex` and `resource__library__subkeyword
  .rex` both do. The operative claim survives because `every_corpus_program_tiles` walks
  `corpus/lang` alone (`gate_walk/mod.rs:399`, re-read), and the sentence now says so and names the
  two files it was wrong about.
* F3: the command was written `..HEAD` and the number taken at BASE. Measured: 81 at `65d333a59`, 83
  at `a1c659f0a`, the extra two being this task's own. The sentence now gives the BASE-relative
  command with the BASE number, and states the HEAD figure beside it.
* F4: "bodies unchanged apart from a comment" missed two message strings that widened with the
  receiver set. Both are on unreachable arms and neither is pinned under `crates/`, so nothing
  observable moved -- but the sentence now says what changed rather than that nothing did.
* F5: "every cell is 1.000000" -- `alloc4c` `ir` `small` has min `0.999999`, read back off the TSV.
  And "three cells at or above 1%" was a projection that had dropped `size`; it is six, three
  `(axis, arm)` pairs at two sizes each. Corrected in both places, and the sitting's own tables
  always carried `size`.

Gates re-run after these edits at `dd83eecdc`; the figures under "Gates" above are from that run.

## Concerns

1. **`.ROUTINES` and `.RESOURCES` are work the brief did not ask for.** The reasoning is under
   "Why `.ROUTINES` and `.RESOURCES` are in this task at all": admitting a `StringTable` receiver
   admits all three tables at once, and leaving two of them empty would have been three silent wrong
   answers rather than a gap. If the reviewer would rather they had been loud refusals instead of
   built values, that is a defensible alternative and I did not take it -- a `Method` and a `Routine`
   are the same object shape, so refusing one while building the other would have been arbitrary, and
   the `.RESOURCES` array is what makes `~items`/`~at`/`~makeString` on it agree today.

2. **A `Method` or `Routine` object cannot receive a message.** `.methods~z~class` is rc 120 where
   the oracle answers `The Method class`. This gap existed before the task in a different place
   (`.methods~z` was itself refused) and is now one step further out. Closing it needs a
   `Primitive` variant per class or a general "any registered native class" receiver arm, which is a
   design decision I did not think was mine to take in this task. Nothing in 5a or 5c that I can see
   sends a message to one -- `CoreClasses.orx:73` passes the `Method` as an argument.

3. **`.RESOURCES` has no corpus row and cannot get one** under `every_corpus_program_tiles`. Its
   witness is an in-crate test with expectations measured on the oracle, which is weaker than a live
   differential: it will not notice the oracle changing. Stated rather than papered over, and
   `corpus/phase-5a.txt` carries the pointer so the next reader finds the witness.

4. **The `~UNKNOWN`-by-hand path is a refusal where the oracle answers**, for any second argument
   that is not already an `Array`. That follows from `MAKEARRAY` being unimplemented for every
   receiver, so it is not new surface, but the by-hand send is a route that did not exist before this
   task made `UNKNOWN` reachable at all.

5. **The staleness test's literal phrasing is stricter than what I could satisfy from `progress.md`
   alone.** Nine of the 81 crate-source commits since the pin are named in a task report rather than
   in the ledger file; all 81 are named somewhere in this plan's SDD workspace, and none is foreign
   to the phase. I read the constraint's intent as "no foreign commit landed beneath the phase" and
   report the arithmetic rather than a verdict.

