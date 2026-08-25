# Task 22, fix round 1: scoped re-review

**CHANGES REQUIRED.**

Scope: the eleven findings of `task-22-review.md` and nothing else. Range `ee083bd7c..HEAD`
(`0ced3f020`). No tree mutation, no rebuild. Probes from a fresh empty directory with absolute
paths, three descriptors, both engines.

Nine of the eleven are closed clean. Two are not: **MINOR 1's sweep left a fifth instance standing
in the same file**, and **M2's doc-file half introduced a new false sentence** while the source half
of the same fix is exactly right.

The round's own governing rule -- *strike the clause, do not replace it with a better reason* -- was
followed in `lib.rs` and abandoned in `phase-4-exclusions.txt`. That is where the new falsehood is.

---

## Per-finding verdict

| | verdict | how checked |
|---|---|---|
| M1 orphaned doc block | **CLOSED** | executed (scan + control) |
| M2 falsified caveat | **PARTIAL** -- source half clean, doc half carries a new false sentence | executed |
| M3 deleted refusal message | **CLOSED** | executed (collapsed scan + control) |
| M4 positional claim | **CLOSED** | code read |
| MINOR 1 invocable-kind counts | **NOT CLOSED** -- one instance remains | executed (collapsed scan) |
| MINOR 2 `Arity` doc | **CLOSED** | code read |
| MINOR 3 comment cardinalities | **CLOSED** | executed (diff sweep) |
| MINOR 4 `Activity::display` | **CLOSED** | C++ read |
| MINOR 5 README blemishes | **CLOSED** | read + counted |
| MINOR 6 report's concern 7 | **CLOSED** | code read |
| MINOR 7 bootstrap guard | **CLOSED** | code read + counted |

---

## The false sentence this round added

`docs/superpowers/plans/phase-4-exclusions.txt:494`:

> Two do.

At least **eight** shared libraries in this tree load. Measured, fresh dir, oracle, on
`::method m external 'LIBRARY <name> no_such_entry_xyz'`:

| library | oracle |
|---|---|
| `rexxutil`, `rxmath`, `rxsock`, `rxregexp`, `rxunixsys`, `hostemu`, `orxncurses`, `external_methods` | 90.998 rc 166 -- the library opened, the entry is missing |
| `nosuchlib`, `rexxapi` | 98.903 rc 158 -- did not load |
| `REXXUTIL` | 98.903 rc 158 -- case-sensitive, as the paragraph says |

So the count is wrong under both readings: as a count of shared libraries that load it is at least
eight, and as a count that includes `LIBRARY REXX` it contradicts the very next clause of its own
sentence (`LIBRARY REXX` is "compiled into the interpreter" rather than loaded). It is also a set
cardinality in prose, which is the shape `rust/CLAUDE.md:87` exists to stop, and it arrived in a
correction round -- the habitat that rule names.

The fix is the round's own rule: strike the number. "Shared libraries in this tree do load" carries
the whole argument and cannot rot. The `librexxutil` measurement beside it is right and should stay.

**The source copy of the same fix is clean and is the model.** `lib.rs:1627`-`:1632` states the loss,
names `::ROUTINE` as its cause, cites `directive_gap`, and offers no reason that can rot. Every
clause of it verified: both rows executed (oracle rc 214 / crate rc 120, both engines, three
descriptors), `directive_gap`'s `Routine` arm is unconditional in the library it names
(`lib.rs:1483`), and the "same file with a `::METHOD` in that position" claim holds both for the
corpus row and for the literal substitution (`::method m external 'LIBRARY REXX Filespec'` is
90.998 rc 166 on oracle and both engines).

---

## MINOR 1: one instance remains

`rust/crates/rexx-exec/src/dispatch.rs:1419`, in `Interp::invoke`'s own doc -- the doc of the
function whose `match` this round edited:

> **`None` is a send that produced no value**, and either kind can do it.

Its two examples are a `::METHOD` body and a `NativeMethod`. There are four kinds, and a third one
does it: `Interp::write_attribute` returns `Ok(None)` (`dispatch.rs:1580`). So the sentence is both
a stale two-kind count and incomplete on the facts.

The four the review named (`:63`, `:85`-`:93`, `:128`, `:136`) are all correctly rephrased over the
set. What is falsified is the report's sentence *"MINOR 1 swept by running a collapsed-comment search
over `dispatch.rs` rather than fixing the list handed to me"* -- the search found exactly the four it
was handed and missed the fifth. I ran a collapsed-comment scan over the file
(`either kind|both kinds|the two kinds|neither kind|both invocable|native or Rexx|both entry points`)
and this is the only hit; nothing else in `dispatch.rs` counts the kinds.

---

## Falsified neighbour, same paragraph as M2

`phase-4-exclusions.txt:483`, left standing when the round added a second row beneath it:

> The row below matched the oracle byte for byte at 62de43c0f and refuses now:

There are two rows below it now, and the paragraph beneath already says "Those rows". As it stands
the singular is wrong and the sentence extends an unmeasured historical claim -- "matched at
62de43c0f" -- to a row that was first probed on 2026-08-25.

---

## Two blemishes

* `phase-4-exclusions.txt:501` runs to 97 columns where the file wraps at ~76
  ("...in the case it was written in. OWNER: Phase 5 for the rest of"). Same class as MINOR 5's
  stray `--`.
* `tests/collect_stress.rs:251` places `directive_method_external_not_a_staged_gap.rex` after
  `_source_order.rex`, out of the list's alphabetical order. Nothing asserts the order; cosmetic.

## One judgement call, with its precedent

`corpus/lang/directive_method_external_not_a_staged_gap.rex:1`, its copy in
`crates/rexx-parse/tests/sourceline_oracle/…_not_a_staged_gap.txt:2`, and `corpus/phase-5a.txt:661`
all say a bound `::METHOD EXTERNAL` "**no longer** preempts" an install-time failure. That is
history, which `rust/CLAUDE.md:96` forbids, and "does not preempt" says the same thing without a
past tense to rot. The same header's "The same file with a `::ROUTINE EXTERNAL` in that position is
refused here instead" is a phase-status claim, which `CLAUDE.md`'s "assert the boundary or do not
write it down" is aimed at -- though `staged_gap`'s table is the assertion, so it is defensible
there.

Weighed against precedent: five `corpus/lang/*.rex` headers already use "used to" / "no longer",
some of them about this crate's own history. The rule is not enforced in corpus headers today, and
MINOR 3 applied only the cardinality rule to a `.rex` header. So this is a note, not a blocker.

---

## What I verified by execution

* **M2's second row.** `::class a` / `::constant kk (1/0)` / `::routine r external 'LIBRARY REXX
  Filespec'`: oracle **rc 214**, `42.3 Arithmetic overflow; divisor must not be zero.` blaming the
  `::CONSTANT`'s own line; crate **rc 120**, `rexx-exec: ::ROUTINE EXTERNAL is not implemented
  (Phase 7)`, both engines. The row reproduces exactly as the doc records it.
* **M2's first row.** `::requires 'helper.rex'` present: oracle rc 214 with `helper ran` on stdout;
  crate rc 120. So "Both rows are probes … and this crate answers rc 120 instead" is true of both.
* **The `::METHOD` substitution.** `::method m external 'LIBRARY REXX Filespec'` in that position is
  90.998 rc 166, oracle and both engines byte-identical.
* **Both new corpus rows**, three descriptors, both engines, byte for byte:
  `_before_duplicate.rex` 90.998 rc 166 echoing the `EXTERNAL` directive, stdout empty;
  `_not_a_staged_gap.rex` 42.3 rc 214 with both clause echoes.
* **Do the new rows witness what their comments claim, and can they redden?** Yes to both, and the
  discriminating counterfactual runs. `_before_duplicate.rex`'s header says the `::METHOD` standing
  first answers "even though the pair below it is a duplicate" -- so I made the entry point
  resolvable (`file_separator`) and left the pair intact: **99.902 rc 157, `Duplicate ::METHOD
  directive instruction`**, oracle and both engines. The duplicate check really is live and the
  90.998 is genuinely preempting it. A build that ran the duplicate check ahead of source order
  answers 99.902 and reddens the row; a build where `::METHOD EXTERNAL` is still a Phase 7 gap
  answers rc 120 and reddens it. `_not_a_staged_gap.rex` reddens the same way if the bind stops
  succeeding or if the gap staging comes back.
* **M3's reverse-order pair.** `::method m external "LIBRARY nosuchlib nosuchfn"` above a plain
  `::method m`: oracle 98.903 rc 158, crate rc 120 `::METHOD EXTERNAL naming a library other than
  REXX is not implemented (Phase 7)`, both engines -- exactly the corrected sentence.
* **M3's sweep.** Collapsed every `//`/`///`/`//!` run in `rust/` to one line (8,555 blocks) and
  searched. No comment quotes `::METHOD EXTERNAL is not implemented`. **Control:** a phrase this
  round hard-wrapped across two `///` lines (`nosuchfn"` first is the oracle`) is found by the
  collapsed scan, so the instrument can see what plain `grep` cannot. I also enumerated every comment
  mention of `is not implemented` in the crate; the only quoted refusal messages left are live ones
  (`a namespace-qualified class lookup …` is emitted at `lib.rs:1376` and asserted in
  `tests/spike.rs:136`).
* **M1's sweep.** Independent method rather than the implementer's: over `git diff -U15
  7c23bf891..HEAD`, flag every *added* item declaration whose walk-back reaches an unchanged `///`
  or `#[` line -- the exact orphan signature. **Zero at HEAD. Control:** the same scan over
  `7c23bf891..ee083bd7c` flags exactly one, `pub struct NativeEntryPoint`. So the instrument fires,
  and `run_program` was the only item that lost its doc.
* **Library loads.** The eleven-library table above.
* **Counts.** `find corpus -name '*.rex'` is **577** (the README's new figure); non-comment lines
  across `phase-*.txt` is **240**; `gate-tables/directives/method__external__subkeyword.rex` is the
  only `gate-tables/` path in any `phase-*.txt`, so the README's "is one that has" is right.
* **`.text` hash.** `objcopy -O binary --only-section=.text` + `sha256sum` on the current
  `target/release/rexx-run` gives **`32c2549a40c437f2abb3d63f325d2c6395c066dd491765190db6b8fc2c98a4bf`**,
  the report's figure to the digit. Neither `debug_assert` message string is in the release binary.

## What I verified by reading code or the C++

* **M1's anchoring.** `// ---- the public entry point ----` and the whole 28-line block sit directly
  above `pub fn run_program` (`lib.rs:6234`-`:6272`); the moved items sit under their own heading
  `// ---- the registry projection ----` (`:6360`) after `render_ir`, each with its own doc, and
  `ir_bodies`' doc below them is intact. Both headings are the only two in the file.
* **M4.** `blame_native_method` is called in `invoke` at `dispatch.rs:1443` (`Native`) and `:1487`
  (`External`'s implemented half) and nowhere else in the function. Neither the `Rexx` arm nor the
  `Generated` arm blames, and `External` does sit between them in source. The narrowed comment is
  true of the arms it names.
* **The module doc's `Cleared` rewrite.** `invocable` reads four tables and returns one variant per
  table (`dispatch.rs:1384`-`:1404`), so "one variant per table `Interp::invocable` reads" is exact.
  Every function that runs a resolved method takes `cleared` by value -- `entry.run`,
  `enter_method_body`, `External`'s `run`, `read_attribute`, `write_attribute`, `read_constant` --
  and the two arms `invoke` answers inline (`Generated::Abstract`, `External::Deferred`) are inside a
  function that already holds one.
* **MINOR 2.** `Raised::too_many_external_arguments`' doc does carry the 88.922/93.902 pair measured
  on two programs, so `Arity`'s new cross-reference lands somewhere real.
* **MINOR 4.** `concurrency/Activity.cpp`: `display` opens at `:1414`, `displayDebug` at `:1492`,
  `displayCondition` at `:536`, and `:1453`-`:1459` is the `POSITION`-guarded ` line <n>` block
  inside `display`. The corrected function name is right.
* **MINOR 6.** `native.rs:162` does name `Cleared` in a doc comment; `code_occurrences`
  (`tests/dispatch_seam.rs`) skips any line whose trimmed start is `//`, and that file's module doc
  at `:87`-`:90` still claims the opposite. The report's corrected concern 7 states all of this
  accurately and correctly marks the module-doc half pre-existing.
* **MINOR 7.** `PlatformObjects.orx` holds zero `EXTERNAL` lines (`CoreClasses.orx` 5,
  `StreamClasses.orx` 61). `bootstrap_files` (`native.rs:398`-`:403`) is the only list of the three
  files in that test module. `the_bootstrap_files_and_the_registry_name_the_same_entry_points`
  builds `declared` from the files and compares it to `held` from `LIBRARY_REXX_METHODS`, so
  dropping a name-contributing file shrinks one side only -- the mechanism the report says it
  inverted. The new doc paragraph is accurate.
* **The `dlopen` claim.** `common/platform/unix/SysLibrary.cpp:88`-`:90`:
  `snprintf(nameBuffer, …, "lib%s%s", name, ORX_SHARED_LIBRARY_EXT)` then `dlopen(nameBuffer, …)`.
  The library name does reach `dlopen` in the case it was written in.
* **MINOR 3.** All four named cardinalities are struck, and so are the extras the report claims
  (`::ROUTINE`/`::ATTRIBUTE` count, the `.rex` header's "the other two ways", `gate_table_d.rs`'s
  "the two forms", `ast.rs`'s "two entry points", `lib.rs`'s "one of the two"). A sweep of every
  added line for count words turns up no new one: "three descriptors" is a measurement idiom, "one
  variant per table" is a mapping, "either side of it" is not a set size.
* **Comment rules.** Zero non-ASCII bytes and zero em- or en-dashes on any added line. `Family`'s
  "Stream, queue and file are Phase 7's" matches `owner()` exactly (`Timer => "Phase 6"`, the rest
  `"Phase 7"`).
* **`collect_stress.rs`'s new comment.** `directive_constant_expression_fails.rex` is above the new
  row in the list and is the same `1/0` shape, so "for the reason … above does" is fair.
* **Sourceline fixtures.** Both new `.txt` files say `count 11`, both `.rex` files are 11 lines, and
  each fixture body is byte-identical to its program.

## The no-codegen claim: sound, and stronger than it looks

Comparing the `.text` section rather than the whole binary is the right instrument here, for a
reason the report does not give:

* The release profile carries `debug = true` (`rust/Cargo.toml`), so DWARF is in the binary. A
  comment-only edit shifts line numbers, which rewrites `.debug_line`/`.debug_info` and changes the
  whole-binary hash while touching nothing executable. The whole-binary hash is therefore *unusable*
  for this question, not merely noisy.
* `.text` is not just "the instructions". In this binary `.rodata` sits at file offset `0x21ce0`,
  **before** `.text` at `0x606b0`, and every reference from code to it is RIP-relative. A data
  section that changed size would move `.text` and change the displacements baked into it. A
  byte-identical `.text` therefore also rules out a `.rodata` size change -- the layout artifact this
  project has measured as worth up to several percent on a bench axis.
* The only `src/` change this round made that could have reached codegen is a `debug_assert!`
  message string, and I confirmed neither the old nor the new spelling is present in
  `target/release/rexx-run` at all: `debug_assertions` is off in release, so the string never existed
  in the binary.

So skipping the sitting is justified. The residue is that I could only reproduce the *after* hash;
the *before* hash needs a rebuild at `ee083bd7c`, which the dispatch forbids.

## What I did not check

* **The five gates, the 240-of-240 corpus and the 99 `test result: ok`.** Run by the controller.
* **The `.text` hash at `ee083bd7c`**, and the MINOR 7 inversion (`bootstrap_files` minus
  `StreamClasses.orx`) -- both need a build or a tree mutation. Read instead; the mechanisms hold.
* **`rexx-diff`'s self-test "0 divergences, exit 0"** in `gate-tables/README.md`. The figure the
  round changed (575 -> 577) I counted; the self-test itself I did not re-run.
* **The `librexxutil` finding** was confirmed independently by the controller and I did not repeat
  it, though my eleven-library table reproduces both halves of it.

## Outside scope, one line

`corpus/phase-5a.txt:155` carries a pre-existing phase-status comment ("once the file's classes no
longer install in source order") of the kind `rust/CLAUDE.md` says to assert rather than write down.
Not this round's, not this task's.
