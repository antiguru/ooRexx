# Task 1: `::ATTRIBUTE EXTERNAL`

**Status:** DONE_WITH_CONCERNS
**Commits:** `f71703edc` (the change), `f81131f9d` (the two committed lists the first one drifted)
**Base:** `385863502`

---

## 1. The row

`corpus/gate-tables/directives/attribute__external__subkeyword.rex` agrees with the oracle byte for
byte on all three descriptors, both engines. Gate table D reports it, in `g4.log`/`g5.log` from
`REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast`:

```
  agree          loud=no  5a    ::ATTRIBUTE  EXTERNAL  subkeyword   attribute__external__subkeyword.rex
```

and the same run's phase summary:

```
by owning phase -- rows, and rows not yet `agree`:
  5a: 36 rows, 0 not yet `agree`
```

So the whole 5a arm of table D agrees, not only this row. The probe has moved into
`corpus/phase-5a.txt`.

The oracle's answer, and this crate's, under
`( memcap 1G env REXX_ENGINE=<engine> timeout -s KILL 20 rust/target/release/rexx-run <abs> )`
against
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -s KILL 10 /home/moritz/dev/repos/ooRexx/build/bin/rexx <abs> )`:

```
rc 166, stdout empty
     3 *-* ::attribute at external 'LIBRARY REXX zzz_no_entry'
Error 90 running <path> line 3:  External name not found.
Error 90.998:  Unable to find external method "GETzzz_no_entry".
```

---

## 2. The plan's premise is false, and that decided the scope

The brief said:

> **The registry exports no `GET*`/`SET*` entry at all** -- measured, ... -- so **every**
> `::ATTRIBUTE ... EXTERNAL 'LIBRARY REXX x'` raises 90.998 on both sides. That makes the work small
> and complete rather than partial.

The first half is true. The conclusion is not, and the C++ says why.

`attributeDirective`'s `ATTRIBUTE_BOTH` arm prepends unconditionally
(`parser/DirectiveParser.cpp:1678`-`:1679`), and so does `methodDirective`'s `isAttribute` arm
(`:867`-`:868`) -- that is the `::METHOD ... ATTRIBUTE` spelling, which has no `GET`/`SET` style to
choose. But the `ATTRIBUTE_GET` and `ATTRIBUTE_SET` arms prepend **only where the decoded procedure
is the default one**:

```c++
// if there was no procedure explicitly given, create one using the GET/SET convention
if (internalname == procedure)
{
    procedure = procedure->concatToCstring("GET");   // :1737, :1739
}
```

That is a pointer comparison, and both operands are interned: `internalname = commonString(name->upper())`
(`:1478`) and `words()` puts every word of the specification through `commonString` too
(`parser/LanguageParser.cpp:4051`-`:4056`), where `commonString` keys the table by string value
(`:2269`-`:2281`). So the test is on the **spelling**: an explicit third word spelled exactly like
the upcased attribute name is the default and takes the prefix, and one spelled differently -- case
included -- resolves unchanged.

Measured, oracle, one program per row, each run from a fresh empty directory:

| program | oracle |
|---|---|
| `::attribute at get external 'LIBRARY REXX zzz_no_entry'` | rc 166, `90.998 ... "zzz_no_entry"` |
| `::attribute at get external 'LIBRARY REXX AT'` | rc 166, `90.998 ... "GETAT"` |
| `::attribute at get external 'LIBRARY REXX file_separator'` | **rc 0** |
| `::attribute at set external 'LIBRARY REXX file_separator'` | **rc 0** |
| `::attribute file_separator get external 'LIBRARY REXX file_separator'` | **rc 0** |
| `::attribute file_separator get external 'LIBRARY REXX FILE_SEPARATOR'` | rc 166, `90.998 ... "GETFILE_SEPARATOR"` |

**The decision that follows, with its cost if wrong.** Those rc-0 forms are cases the oracle answers
and the plan expected not to exist. Two ways forward: raise 90.998 on the miss and keep a loud
refusal for the resolving forms, or bind them. **I bound them**, because the machinery is the one
`::METHOD ... EXTERNAL 'LIBRARY REXX name'` already used -- `InstallBody::Native`, `native_externals`
and the same send path -- so the resolving half cost one match arm in `Interp::install_attribute`
rather than a new mechanism, and leaving it refused would have shipped a divergence on a form this
task is already touching. **Cost if wrong:** a send to a bound accessor is new surface where a silent
wrong answer could enter, which is the defect this plan's constraints call the worst. Section 4 is
what was run against that.

`docs/superpowers/plans/2026-08-26-phase-5a-gate-close.md`'s Task 1 section carries the correction,
so a fix round regenerating this brief does not inherit the false premise.

---

## 3. What changed

**`crates/rexx-exec/src/dispatch/native.rs`** -- the boundary the previous plan drew in
`method_external` now has two halves that share the library test, `method_external` and
`attribute_external`. `MethodExternal::Attribute` carries `Vec<AttributeBind>`: for each accessor,
the dictionary key it fills and the entry point it resolved (or the procedure it did not).
`attribute_binds` is the one place the prefix rule lives, with the measured transcripts above it.
`bound_entry` and `unresolved_entry` are what the installers and the install walk read.

**`crates/rexx-exec/src/lib.rs`** -- `directive_gap`'s `::METHOD` arm no longer refuses `Attribute`,
and its `::ATTRIBUTE` arm refuses only `OtherLibrary`. `unresolved_external` asks both directives one
question, and `Interp::install_directives`' first walk raises 90.998 from it. `Interp::install_method`
and `Interp::install_attribute` both take the entry point per dictionary key, because an attribute's
two accessors name two different procedures.

**Refusal messages that changed**, because the discrimination moved: `::ATTRIBUTE EXTERNAL is not
implemented (Phase 7)` is now `::ATTRIBUTE EXTERNAL naming a library other than REXX ...`, and
`::METHOD ATTRIBUTE EXTERNAL ...` is gone -- that spelling falls into the existing `::METHOD EXTERNAL
naming a library other than REXX ...` when it names another library, and binds otherwise. The library
test moved **ahead** of the `ATTRIBUTE` question because `resolveMethod` loads the library before it
looks for a procedure in it; measured, `::method m attribute external 'LIBRARY nosuchlib x'` is
`98.903 Unable to load library "nosuchlib"` at rc 158 and not 90.998.

**Corpus** -- six programs under `corpus/lang/` plus the row's probe, all in `corpus/phase-5a.txt`,
each with its `crates/rexx-parse/tests/sourceline_oracle/<name>.txt` generated by the driver in that
test's module comment.

**Documentation corrected rather than added to**: `rexx-parse`'s `ExternalSpec::entry` said the
accessors are "each **prefixed** with `GET` or `SET`", which is false for the `GET`/`SET` styles;
`phase-4-exclusions.txt`'s "THE OTHER TWO FORMS ARE UNTOUCHED" block; `gate_table_d.rs`'s
`owning_phase` bullet, which argued about a refusal that no longer exists; `method_body_gap`'s and
`staged_gap`'s sentences naming only the `::METHOD` form.

---

## 4. Probing past the row

Every program below was run from a fresh empty directory with absolute paths, on the oracle and on
both crate engines, three descriptors read separately. **Agrees** means byte for byte on all three.

**Install-time name resolution -- all agree.** `::attribute at external 'LIBRARY REXX zzz_no_entry'`
(`GETzzz_no_entry`); `... 'LIBRARY REXX file_separator'` (`GETfile_separator`); `... 'LIBRARY REXX'`
(`GETAT`); `... get external 'LIBRARY REXX zzz_no_entry'` (`zzz_no_entry`); `... get external
'LIBRARY REXX'` (`GETAT`); `... get external 'LIBRARY REXX AT'` (`GETAT`); `... set external
'LIBRARY REXX'` (`SETAT`); `... set external 'LIBRARY REXX zzz_no_entry'` (`zzz_no_entry`);
`... class external 'LIBRARY REXX zzz_no_entry'` (`GETzzz_no_entry`); `::attribute file_separator get
external 'LIBRARY REXX FILE_SEPARATOR'` (`GETFILE_SEPARATOR`); `::method m attribute external
'LIBRARY REXX zzz_no_entry'` / `'LIBRARY REXX'` / `'LIBRARY REXX file_separator'` (`GETzzz_no_entry`,
`GETM`, `GETfile_separator`); `::method m attribute class external 'LIBRARY REXX'` (`GETM`);
`::attribute a external 'LIBRARY REXX file_separator'` (`GETfile_separator`).

**Resolving, rc 0 -- all agree.** `::attribute at get external 'LIBRARY REXX file_separator'`;
`... set external 'LIBRARY REXX file_separator'`; `... get external 'LIBRARY REXX FILE_SEPARATOR'`;
`::attribute file_separator get external 'LIBRARY REXX file_separator'`.

**Sends to a bound accessor -- all agree.** `.k~at` on a class-side getter bound to `file_separator`
answers `/`, and to `file_path_separator` answers `:`. `.k~at = 5` on a bound setter is `88.922 Too
many arguments in invocation; 0 expected.` at rc 168 with the entry point's own `Compiled method
"AT=" with scope "K".` frame above it, and `.k~at(1)` on a bound getter is the same 88.922. One class
carrying a bound `GET` and a bound `SET` naming different entry points answers `/` and then that
88.922. A `private` bound getter sent from outside is `97.2 ... cannot accept private message "AT"`
at rc 159. `::constant sep (.k~at)` reading a private bound getter is rc 0 printing `/`.
`::class j subclass k` inherits the bound class-side getter, rc 0. `.k~method('AT')` where `AT` is
class-side is `97.1` at rc 159 on both sides.

**Neighbouring translation-time refusals -- all agree.** A duplicate `::attribute at get external`
pair, and a bound one beside a bare `::attribute at`, are both `99.931` at rc 157.
`::attribute at class external ...` with no `::CLASS` above it is `99.905` at rc 157.
`::annotate constant at` naming an attribute target is `99.945` at rc 157. `::annotate attribute at`
above a bound accessor is rc 0. An unattached `::attribute at external 'LIBRARY REXX zzz_no_entry'`
with no `::CLASS` at all is the same 90.998. The row's directive on **either** side of
`::class a subclass zzznotaclass` is 90.998 in both orders, matching the oracle.

**Still divergent, and each is a loud refusal rather than a wrong answer:**

| program | oracle | crate |
|---|---|---|
| `::attribute at external 'LIBRARY zzznolib zzz'` | `98.903` rc 158 | rc 120, `::ATTRIBUTE EXTERNAL naming a library other than REXX is not implemented (Phase 7)` |
| `::attribute at get external 'library rexx file_separator'` | `98.903 ... "rexx"` rc 158 | the same refusal |
| `::method m attribute external 'LIBRARY nosuchlib x'` | `98.903` rc 158 | rc 120, `::METHOD EXTERNAL naming a library other than REXX ...` |
| `.k~at` bound to a **deferred** entry (`stream_chars`) | `48.1` rc 208 | rc 120, `the LIBRARY REXX entry point "stream_chars" is not implemented (Phase 7)` |
| a bound `::ATTRIBUTE GET` followed by a clause | `99.935` rc 157 | rc 120, `99.935: External attributes cannot have a method body.` |
| `::method m delegate p external '...'` | `25.902` rc 231 | rc 120, `25.902: Invalid subkeyword found.` |

The first three are the Phase 7 library-loading boundary, unchanged in kind by this task. The fourth
is the accepted divergence a deferred entry point already had through `::METHOD`. The last two are
the `deferred-parse-error-rendering` deferral: the crate finds the same error and renders it
differently, which is pre-existing and untouched here.

**A shape this task did not have to handle**, checked rather than assumed: `ABSTRACT` beside
`EXTERNAL` and `DELEGATE` beside `EXTERNAL` are mutually exclusive in both parsers
(`parser/DirectiveParser.cpp:686`, `:787`, `:797`, `:1619`, `:1630`), so no directive reaching an installer
carries an `EXTERNAL` and more than one dictionary key except an attribute pair. Measured on the
oracle: `::method m delegate p external 'LIBRARY REXX file_separator'` is `25.902` at rc 231.

---

## 5. The control, run and inverted live

The brief's control: appending `GET` instead of prepending it names `zzz_no_entryGET` and the row
reddens.

`crates/rexx-exec/src/dispatch/native.rs` was copied to the scratchpad
(`sha256 d77ae2fa4885ced5451d1cb18ef78c2cdf43f017287e73f6a6f6c6c1b4927437`), then `attribute_binds`'
one line was mutated:

```rust
-    let prefixed = |prefix: &str| [prefix.as_bytes(), &procedure].concat();
+    let prefixed = |prefix: &str| [&procedure[..], prefix.as_bytes()].concat();
```

After `cargo build --release --bin rexx-run`, the gate row's own program answers, on **both** engines:

```
Error 90.998:  Unable to find external method "zzz_no_entryGET".
```

against the oracle's `"GETzzz_no_entry"` -- **at the same rc 166 with the same empty stdout**, so
only the byte-for-byte `stderr` comparison sees it.

`REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus --test gate_table_d --no-fail-fast`
exits **101** and reddens a nameable set:

```
254 of 258 matching
mismatches (4):
  [UNCLASSIFIED] lang/directive_attribute_external_missing.rex: stderr differ
  [UNCLASSIFIED] lang/directive_attribute_external_set_default.rex: stderr differ
  [UNCLASSIFIED] lang/directive_method_attribute_external_missing.rex: stderr differ
  [UNCLASSIFIED] gate-tables/directives/attribute__external__subkeyword.rex: stderr differ
```

and table D's row moves to `diverge-stderr loud=no 5a ::ATTRIBUTE EXTERNAL`.
`cargo test --release -p rexx-exec --lib dispatch::native --no-fail-fast` exits 101 with

```
  left: [("AT", Err("zzz_no_entryGET")), ("AT=", Err("zzz_no_entrySET"))]
 right: [("AT", Err("GETzzz_no_entry")), ("AT=", Err("SETzzz_no_entry"))]
```

**`directive_attribute_external_get_third_word.rex` stayed green under the mutation**, and that is
the control's own control: the `GET` style with a differing third word applies no prefix at all, so
a mutation of the prefixing cannot reach it. The two rules are separate and each has a witness.

**Restore.** `cp` from the scratchpad copy, `sha256sum` equal on both files, `git status --porcelain`
empty, `git diff --exit-code` clean, `HEAD` at `f81131f9d`. The release binary was rebuilt afterwards
and the row re-verified: `descriptors: MATCH` on both engines.

---

## 6. The five gate commands, at `f81131f9d`, from `rust/`

Each status read unpiped, each command run in sequence by one script, each figure quoted with the
command that printed it.

| gate | command | exit |
|---|---|---|
| G1 | `cargo fmt --all --check` | **0** |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| G3 | `cargo test --release --workspace --no-fail-fast` | **0** |
| G4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** |
| G5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |

`memcap` was present and used (`command -v memcap` -> `/home/moritz/.local/bin/memcap`).

**`258 of 258 matching`** is the corpus differential's own line, printed by G4 and by G5. The base
`385863502` had 251 rows, counted with
`git show 385863502:rust/corpus/phase-{4a,4b,4c,5a}.txt | grep -v '^#' | grep -v '^$' | wc -l`;
seven rows were added.

**`5a: 36 rows, 0 not yet agree`** is gate table D's own summary, from the same G4 run.

**The first attempt at these gates failed and that is why there are two commits.** At `f71703edc`,
`cargo test --release --workspace --no-fail-fast` exited **101** with exactly two assertions failing:
`coverage.rs`'s `phase_5a_subset_matches_the_committed_list` and `collect_stress.rs`'s
`the_l0_subset_passes_again_under_collect_on_every_allocation`. Both are committed lists policed in
both directions, and adding a row to `corpus/phase-5a.txt` requires updating each of them by hand.
`f81131f9d` does that: the seven subset rows in `EXPECTED_SUBSET_5A`, and the five that allocate
nothing in `NO_ALLOCATION_PROGRAMS` (`_bind` and `_arguments` reach their main body and are not
among them). The table above is the re-run at `f81131f9d`.

---

## 7. Concerns

1. **The plan's premise was wrong and the plan is edited.** I corrected Task 1's section of
   `docs/superpowers/plans/2026-08-26-phase-5a-gate-close.md` in `f71703edc` rather than leaving the
   correction in this report, per the standing rule. If Tasks 2-6 are running concurrently, that is a
   shared file -- the edit is confined to Task 1's own section, but a controller reconciling the plan
   should know it moved.

2. **The resolving forms are new answering surface.** `::ATTRIBUTE ... GET` / `... SET EXTERNAL`
   naming an entry point the `REXX` package exports now installs and a send to it runs. Section 4 is
   what was run against that; the instance-side half of it is **unreachable** to probe, because
   `.k~new` is still `method "NEW" of class "Object" is not implemented (Phase 5)`. Every send probe
   above is class-side. A wrong answer to an instance-side bound accessor would be caught by nothing
   in this tree today.

3. **The setter half of a failing pair is not observable.** For the `BOTH` style and the `::METHOD`
   spelling, the getter is resolved first, so the oracle never reports the `SET`-prefixed name. Those
   cells of `an_attribute_external_resolves_the_procedure_the_oracle_names` come from
   `attributeDirective`'s own two `createNativeMethod` calls (`:1682`, `:1689`) rather than from a
   run, and the test's doc says so rather than claiming they were measured.

4. **`::ROUTINE EXTERNAL` is untouched**, `LIBRARY REXX` included, and it is table D's one remaining
   Phase 7 row. Nothing here changes that.

5. **Two committed lists were not in the brief and only the gate found them.** `EXPECTED_SUBSET_5A`
   and `NO_ALLOCATION_PROGRAMS` are the prerequisites a task adding corpus rows owes; a later task in
   this plan that adds rows will owe them too.
