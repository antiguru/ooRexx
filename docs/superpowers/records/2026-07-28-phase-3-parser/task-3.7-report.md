# Task 3.7 report: the nine `::` directives

**Status: complete, all review rounds closed.** Eleven commits of mine on `plan/rust-rewrite`, `f35b83f6` through `8a98ee40`; `d4d12403` and `bc7dc119` in that range are the coordinator's, not mine. The round-2 section at the end of this file records what the review changed and supersedes any figure above it that it contradicts.
200 crate tests and 433 workspace tests, all green.
`cargo clippy --offline --all-targets -- -D warnings` clean, checked on its own line.
32 mutations, every one applied and every one caught.
The phase gate's grep prints nothing.

## What was built

`rust/crates/rexx-parse/src/directive.rs`, a port of
`LanguageParser::nextDirective` (`DirectiveParser.cpp:64`) and the nine functions
it dispatches to, plus `rust/crates/rexx-parse/src/directive/tests.rs`.
`parse_directive(&ParseCtx, &mut ClauseCursor) -> Result<Directive, ParseError>`
consumes one `::` clause and returns one node.

Three supporting changes came with it.

* `rust/crates/rexx-parse/src/convert.rs` holds the three text conversions both
  grammars need, moved out of `instruction.rs`: `whole_number`, `is_number` and
  `check_trace_setting`. `whole_number` now takes the caller's precision, because
  `TRACE` converts under the parse-time `NUMERIC DIGITS` and `::OPTIONS DIGITS`
  under `Numerics::ARGUMENT_DIGITS`, and the boundary really does differ.
* `TokenCursor` grew `peek_real` and `advance_real`, the blank-skipping walk the
  instruction grammar had inline. Both cursors are built from a clause's token
  range, so the limit they used was already the same one.
* `ParseCtx` grew `resources: &'a [ResourceBody]`, so a `::RESOURCE` can find the
  body `scan` already copied out.

`rust/crates/rexx-parse/src/ast.rs` gained `Directive`, `DirectiveKind` and
seventeen supporting types, all exported from `lib.rs`.

## Tables: nine and forty, and a third table at two points

The token after `::` resolves against `ctx.keywords.directives`, nine rows, and
every token after it against `ctx.keywords.sub_directives`, forty rows, both by
`SymbolId` through `KeywordSet::index_of`. Both counts are asserted in
`the_directive_tables_are_nine_and_forty_entries`, and every index constant is
pinned against its own spelling so a reordering fails loudly.

**The brief's positional rule has two exceptions, and they are not cosmetic.**
Two option arguments resolve against `subKeywords[]` instead, because that is
what the C++ calls there: `::OPTIONS FORM` at `DirectiveParser.cpp:1007` and
`::OPTIONS NUMERIC` at `DirectiveParser.cpp:1339`. `NOINHERIT`, `SCIENTIFIC` and
`ENGINEERING` are rows of `subKeywords[]` **alone**, so `subDirectives[]` cannot
cover them, and `SYNTAX` is the reverse. Measured, both directions:

```
=== OPTIONS NUMERIC NOINHERIT      rc=0
=== OPTIONS NUMERIC SYNTAX         Error 25.935
=== OPTIONS FORM ENGINEERING       rc=0
=== OPTIONS FORM VALUE             Error 25.11
```

A parser that used `subDirectives[]` at those two points would reject
`::OPTIONS NUMERIC NOINHERIT`, which is legal, and accept
`::OPTIONS NUMERIC SYNTAX`, which is not. Mutation 4 pins it.

All five shared spellings are exercised in both positions:
`the_five_shared_spellings_mean_different_things_by_position`.

## `::RESOURCE`

The body was already `scan`'s, keyed by the `::` token index, which equals the
clause's first token index. Consuming it is one integer comparison, and the body
is **not** tokenised: measured, a body holding
`this is 'unmatched and /* unclosed` gets rc 0.

This module owes the rest of `resourceDirective`, and supplies it: 25.926 for a
sub-directive that is not `END`, 19.921 for a missing marker, 21.914 for data
after it. The missing-marker error 99.943 stays the scanner's, and reaches this
module's tests as a whole-file scan failure.

The `.expect()` on the body lookup is safe by construction and not by hope: every
shape `scan` copies a body for is a shape that passes all four checks above, and
every shape it skips fails one of them. The five- and three-real-token shapes
`scan` matches are exactly what surviving `require_name` / `SUBDIR_END` /
`require_name` / `required_end` leaves.

## What is deliberately NOT here

Each needs the accumulated package, which `parse_directive` returning one
directive cannot keep. Every one is measured and named in the module's own doc
comment so the caller inherits a list rather than a surprise.

| left to the caller | measured |
| --- | --- |
| duplicate class, routine, resource, method, attribute, constant | 99.901, 99.903, 99.942, 99.902, 99.931, 99.932 |
| `::CONSTANT` with an expression outside a `::CLASS` | 99.906, and rc 0 with a `::CLASS` above it |
| `::METHOD ... CLASS` with no `::CLASS` | 99.905 |
| `::ANNOTATE` naming a target that does not exist | 99.945 |
| `::OPTIONS FUZZ` not below the package `DIGITS` | 33.1, a `reportException` not a `syntaxError` |
| resolving an `EXTERNAL` library or entry point | 98.903 at rc 158, 90.999 |
| a non-directive clause where a directive was due | 99.916, from the next `nextDirective` |
| a directive inside `INTERPRET` text | 99.915, raised in `translate` before any directive parse |

Block structure, the control stack, the chain and jump indices and the
`translateBlock` errors are untouched: Task 3.7c's. Nothing here needed to know
what block encloses a directive.

## Two things the interpreter defines and the docs would not

**A second `EXTERNAL` on a `::ROUTINE` reports the `::CLASS` error number.**
`routineDirective` passes `Error_Invalid_subkeyword_class` at
`DirectiveParser.cpp:2606`. Reproduced, not corrected. Measured:

```
=== ROUTINE EXTERNAL twice
     1	::routine r external "LIBRARY x" external "LIBRARY y"
Error 25 running ... line 1:  Invalid subkeyword found.
Error 25.901:  Unknown keyword on ::CLASS directive; found "EXTERNAL".
```

**The `EXTERNAL` decode happens at a different point per shape, and the error
order is observable.** `::METHOD`'s plain external shape decodes before
`checkDirective`; its attribute shape and every `::ATTRIBUTE` shape check the
body first. Two sources differing by one keyword give different errors on
different lines:

```
=== METHOD EXTERNAL junk + body
     1	::method m external "junk"
     2	  return 1
Error 99 running ... line 1:  Translation error.
Error 99.917:  Incorrect external name specification "junk".

=== METHOD ATTRIBUTE EXTERNAL junk + body
     1	::method m attribute external "junk"
     2	  return 1
Error 99 running ... line 2:  Translation error.
Error 99.934:  Attribute methods cannot have a method body.
```

`the_external_decode_happens_where_each_shape_does_it` pins all six cases.

## A Task 3.6 defect found and fixed

`whole_number` did not strip the blanks a Rexx number may carry, so
`trace " 9 "` raised 24.1 where the oracle takes it as a skip count of 9. The
same conversion is what `::OPTIONS DIGITS` needs, so the fix landed with the
move. Both directions measured, and both directions asserted:

```
=== TRACE " 9 " blanks
     1	trace " 9 "
     2	nop
--- rc=0

=== TRACE "9 5" internal blank
     1	trace '9 5'
Error 24 running ... line 1:  Invalid TRACE request.
Error 24.1:  TRACE request letter must be one of "ACEFILNOR"; found "9".
--- rc=232
```

`::options digits " 9 "` is rc 0 and `"- 9"`, `"9 5"` and `"1 e2"` are all 26.5,
so blanks may surround a number and not sit inside it.

## Acceptance: the shipped packages

`core_classes_parses` walks every clause of
`interpreter/RexxClasses/CoreClasses.orx`, dispatching `::` clauses to
`parse_directive` and every other clause to Task 3.6's `parse_instruction`. It
asserts 347 directives, decomposing 32 `::CLASS` / 303 `::METHOD` /
12 `::ATTRIBUTE`, and that every span is non-empty and inside the file.
`the_other_shipped_packages_parse` does the same for `StreamClasses.orx`, which
adds `::CONSTANT`.

This is the whole file, minus what Task 3.7c owns: the harness builds no control
stack, so it raises none of the block-structure errors and it does not assemble
the instruction chain. Everything up to that line parses.

## Tests

200 crate tests, up from 167. The new ones:

* 9 in `convert/tests.rs`, every case a `rexxc` measurement.
* 24 in `directive/tests.rs`.

Both directions of every gate. Two tables of forty rows each pair every
sub-directive with a directive that carries it and with one that refuses it,
with the table length asserted against `keywords.sub_directives.len()` so a
row cannot go missing. Thirty-eight of the forty accepted rows are rc 0; the two
that are not cite their non-zero rc deliberately, because reaching a run-time
failure is what proves the parse succeeded.

### Mutation testing

32 mutations of the committed code, each targeting one load-bearing decision.
The script fails on a surviving mutation **and** on one whose pattern matched
anything but exactly once, because a never-applied mutation reads exactly like a
caught one. It exits 0 only when all 32 were applied and all 32 were caught.

Two holes it found, both now closed and committed separately as `04fa320f`:

* Mutation 9, reporting a `checkDirective` error against the directive rather
  than the offending clause, **survived**: nothing asserted the reported byte.
  The pair is only distinguishable with blank lines between the two positions,
  because otherwise moving one moves the other. This is the same shape as the
  missing-`THEN` case that cost this project a round, and the probe printed the
  whole message rather than a field.
* Mutation 24, a `::RESOURCE` accepting any sub-directive rather than only `END`,
  **survived**: `::resource d junk` fires whichever check is in place, because
  `JUNK` is not in the table at all. `::resource d public x` separates them.

The script's own verdict expression was wrong on the first run and reported all
32 as survivors, which is why the second run is the one that counts: Python's
`and` binds tighter than `or`, and the substring it tested was the wrong case.
Both runs are below.

## Deviations from the brief, and one to accept or reject

1. **Tests live in `src/directive/tests.rs`, not `tests/directive.rs`.**
   `parse_directive`, `ParseCtx` and `ClauseCursor` are all `pub(crate)` and an
   integration test is a separate crate. Task 3.6 made the same call.
2. **Four commits for the grammar, not one per directive family.** Each of the
   six commits compiles clean and tests green. Slicing `directive.rs` further
   would have needed a temporary catch-all dispatch arm and a temporarily sliced
   constant table at every boundary, because `clippy -D warnings` fails on an
   unused `DIR_*` or `SUBDIR_*` constant and on a `convert.rs` helper with no
   caller yet. That is code existing only to make a commit boundary. **Say the
   word and I will redo the split that way.**
3. **The brief's "resolution goes through `ParseCtx::keywords`, never a string
   table" holds, with the `subKeywords[]` exception documented above.** No sorted
   string table exists anywhere in the module.

## Concerns

* **`ARGUMENT_DIGITS` is platform-dependent and I hard-coded the 64-bit value.**
  `Numerics.hpp:90` gives 18 on a 64-bit build and 9 on a 32-bit one, and the
  boundary is observable: `::options digits 123456789012345678` is rc 0 here.
  The scanner deliberately did NOT reproduce the analogous `INTEGER_CONSTANT`
  digit limit, on the grounds that it has no observable effect. This one does, so
  I reproduced it, but a 32-bit differential run would disagree with this build.
* **`Terminators::with` moved from a dead-code allowance to `cfg(test)`.** The
  brief said becoming a real caller is what removes the Task 3.7 allowance, and
  it did for every constant in that block; `with` was the one item no caller
  reached, and it is test-only rather than not-yet-called. If you would rather it
  keep an allowance naming a future task, say which task.
* **`decode_external` splits on blanks and tabs only.** `RexxString::subWords`
  is what the C++ uses and I did not read its full whitespace set. Measured that
  both a blank and a tab separate words; a vertical tab or form feed inside an
  `EXTERNAL` string is untested.
* **`upper_value_of` upcases ASCII only.** That matches `SymbolTable::intern`'s
  documented rule and is exact for a symbol, which cannot hold a non-ASCII byte.
  A literal can, and `RexxString::upper`'s behaviour on one is untested here.
  Reachable through `::CLASS c SUBCLASS "ünïcode"` and through an `::ANNOTATE`
  target name.

## Probe log

Every probe below was run into a fresh `mktemp -d`, never the session
scratchpad. The whole oracle message is printed for every one; no field is
extracted. The only text removed is the constant eight-line startup banner,
deleted by an anchored `sed` range from `^Open Object Rexx Version` to the
licence URL.

256 probes, `build/bin/rexxc` unless the source header says `rexx`.

```text
=== plain ::class
--- source (rexxc) ---
     1	::class c
--- output ---
--- rc=0

=== ::class with no name
--- source (rexxc) ---
     1	::class
--- output ---
     1 *-* ::class
Error 19 running /tmp/tmp.kvGw9E8tqB/p.rex line 1:  String or symbol expected.
Error 19.901:  String or symbol expected after ::CLASS keyword.
--- rc=237

=== CLASS PUBLIC
--- source (rexxc) ---
     1	::class c public
--- output ---
--- rc=0

=== CLASS PRIVATE
--- source (rexxc) ---
     1	::class c private
--- output ---
--- rc=0

=== CLASS ABSTRACT
--- source (rexxc) ---
     1	::class c abstract
--- output ---
--- rc=0

=== CLASS SUBCLASS
--- source (rexxc) ---
     1	::class c subclass object
--- output ---
--- rc=0

=== CLASS MIXINCLASS
--- source (rexxc) ---
     1	::class c mixinclass object
--- output ---
--- rc=0

=== CLASS METACLASS
--- source (rexxc) ---
     1	::class c metaclass class
--- output ---
--- rc=0

=== CLASS INHERIT
--- source (rexxc) ---
     1	::class m mixinclass object
     2	::class c inherit m
--- output ---
--- rc=0

=== CLASS PACKAGE
--- source (rexxc) ---
     1	::class c package
--- output ---
     1 *-* ::class c package
Error 25 running /tmp/tmp.JJNcXNoD1A/p.rex line 1:  Invalid subkeyword found.
Error 25.901:  Unknown keyword on ::CLASS directive; found "PACKAGE".
--- rc=231

=== CLASS GUARDED
--- source (rexxc) ---
     1	::class c guarded
--- output ---
     1 *-* ::class c guarded
Error 25 running /tmp/tmp.dTcKLUUsoj/p.rex line 1:  Invalid subkeyword found.
Error 25.901:  Unknown keyword on ::CLASS directive; found "GUARDED".
--- rc=231

=== CLASS METACLASS nothing
--- source (rexxc) ---
     1	::class c metaclass
--- output ---
     1 *-* ::class c metaclass
Error 19 running /tmp/tmp.RXKr8znvv2/p.rex line 1:  String or symbol expected.
Error 19.906:  String or symbol expected after METACLASS keyword.
--- rc=237

=== CLASS MIXINCLASS nothing
--- source (rexxc) ---
     1	::class c mixinclass
--- output ---
     1 *-* ::class c mixinclass
Error 19 running /tmp/tmp.43os0QsQcV/p.rex line 1:  String or symbol expected.
Error 19.913:  String or symbol expected after MIXINCLASS keyword.
--- rc=237

=== CLASS SUBCLASS nothing
--- source (rexxc) ---
     1	::class c subclass
--- output ---
     1 *-* ::class c subclass
Error 19 running /tmp/tmp.KZp6C6PWnv/p.rex line 1:  String or symbol expected.
Error 19.907:  String or symbol expected after SUBCLASS keyword.
--- rc=237

=== CLASS INHERIT nothing
--- source (rexxc) ---
     1	::class c inherit
--- output ---
     1 *-* ::class c inherit
Error 19 running /tmp/tmp.FtvOtQyF50/p.rex line 1:  String or symbol expected.
Error 19.908:  String or symbol expected after INHERIT keyword.
--- rc=237

=== CLASS SUBCLASS twice
--- source (rexxc) ---
     1	::class c subclass object subclass object
--- output ---
     1 *-* ::class c subclass object subclass object
Error 25 running /tmp/tmp.AnoQa26gwL/p.rex line 1:  Invalid subkeyword found.
Error 25.901:  Unknown keyword on ::CLASS directive; found "SUBCLASS".
--- rc=231

=== CLASS MIXINCLASS then SUBCLASS
--- source (rexxc) ---
     1	::class c mixinclass object subclass object
--- output ---
     1 *-* ::class c mixinclass object subclass object
Error 25 running /tmp/tmp.GFYzqgPjIe/p.rex line 1:  Invalid subkeyword found.
Error 25.901:  Unknown keyword on ::CLASS directive; found "SUBCLASS".
--- rc=231

=== CLASS SUBCLASS then MIXINCLASS
--- source (rexxc) ---
     1	::class c subclass object mixinclass object
--- output ---
     1 *-* ::class c subclass object mixinclass object
Error 25 running /tmp/tmp.9bsxuBgEAu/p.rex line 1:  Invalid subkeyword found.
Error 25.901:  Unknown keyword on ::CLASS directive; found "MIXINCLASS".
--- rc=231

=== CLASS METACLASS twice
--- source (rexxc) ---
     1	::class c metaclass class metaclass class
--- output ---
     1 *-* ::class c metaclass class metaclass class
Error 25 running /tmp/tmp.Mk24qHqs8V/p.rex line 1:  Invalid subkeyword found.
Error 25.901:  Unknown keyword on ::CLASS directive; found "METACLASS".
--- rc=231

=== CLASS ABSTRACT twice
--- source (rexxc) ---
     1	::class c abstract abstract
--- output ---
     1 *-* ::class c abstract abstract
Error 25 running /tmp/tmp.PhGXoMfd8z/p.rex line 1:  Invalid subkeyword found.
Error 25.901:  Unknown keyword on ::CLASS directive; found "ABSTRACT".
--- rc=231

=== CLASS SUBCLASS namespace form
--- source (rexxc) ---
     1	::class c subclass rexx:object
--- output ---
--- rc=0

=== CLASS SUBCLASS namespace no name
--- source (rexxc) ---
     1	::class c subclass rexx:
--- output ---
     1 *-* ::class c subclass rexx:
Error 20 running /tmp/tmp.Pvg1wBBPgx/p.rex line 1:  Symbol expected.
Error 20.921:  Symbol expected as a class name of qualified class name.
--- rc=236

=== CLASS SUBCLASS literal
--- source (rexxc) ---
     1	::class c subclass "object"
--- output ---
--- rc=0

=== CLASS INHERIT two
--- source (rexxc) ---
     1	::class m1 mixinclass object
     2	::class m2 mixinclass object
     3	::class c inherit m1 m2
--- output ---
--- rc=0

=== CLASS comma mid-clause
--- source (rexxc) ---
     1	::class c, public
--- output ---
     1 *-* ::class c, public
Error 25 running /tmp/tmp.Lq438iE9cb/p.rex line 1:  Invalid subkeyword found.
Error 25.901:  Unknown keyword on ::CLASS directive; found ",".
--- rc=231

=== METHOD plain body
--- source (rexxc) ---
     1	::method m
     2	  return 1
--- output ---
--- rc=0

=== METHOD CLASS
--- source (rexxc) ---
     1	::class c
     2	::method m class
     3	  return 1
--- output ---
--- rc=0

=== METHOD CLASS with no active class
--- source (rexxc) ---
     1	::method m class
--- output ---
     1 *-* ::method m class
Error 99 running /tmp/tmp.gDkaFRT7Ro/p.rex line 1:  Translation error.
Error 99.905:  CLASS keyword on ::METHOD directive requires a matching ::CLASS directive.
--- rc=157

=== METHOD PRIVATE
--- source (rexxc) ---
     1	::method m private
     2	  return 1
--- output ---
--- rc=0

=== METHOD PACKAGE
--- source (rexxc) ---
     1	::method m package
     2	  return 1
--- output ---
--- rc=0

=== METHOD PUBLIC
--- source (rexxc) ---
     1	::method m public
     2	  return 1
--- output ---
--- rc=0

=== METHOD PROTECTED
--- source (rexxc) ---
     1	::method m protected
     2	  return 1
--- output ---
--- rc=0

=== METHOD UNPROTECTED
--- source (rexxc) ---
     1	::method m unprotected
     2	  return 1
--- output ---
--- rc=0

=== METHOD GUARDED
--- source (rexxc) ---
     1	::method m guarded
     2	  return 1
--- output ---
--- rc=0

=== METHOD UNGUARDED
--- source (rexxc) ---
     1	::method m unguarded
     2	  return 1
--- output ---
--- rc=0

=== METHOD ATTRIBUTE
--- source (rexxc) ---
     1	::method m attribute
--- output ---
--- rc=0

=== METHOD ABSTRACT
--- source (rexxc) ---
     1	::method m abstract
--- output ---
--- rc=0

=== METHOD DELEGATE
--- source (rexxc) ---
     1	::method m delegate p
--- output ---
--- rc=0

=== METHOD EXTERNAL unresolvable library
--- source (rexxc) ---
     1	::method m external "LIBRARY nosuch"
--- output ---
     1 *-* ::method m external "LIBRARY nosuch"
Error 98 running /tmp/tmp.QTSVuPPTod/p.rex line 1:  Execution error.
Error 98.903:  Unable to load library "nosuch".
--- rc=158

=== METHOD SUBCLASS
--- source (rexxc) ---
     1	::method m subclass object
--- output ---
     1 *-* ::method m subclass object
Error 25 running /tmp/tmp.Tk4aVahZXr/p.rex line 1:  Invalid subkeyword found.
Error 25.902:  Unknown keyword on ::METHOD directive; found "SUBCLASS".
--- rc=231

=== METHOD ATTRIBUTE with body
--- source (rexxc) ---
     1	::method m attribute
     2	  return 1
--- output ---
     2 *-* return 1
Error 99 running /tmp/tmp.MqW4INoBoc/p.rex line 2:  Translation error.
Error 99.934:  Attribute methods cannot have a method body.
--- rc=157

=== METHOD ABSTRACT with body
--- source (rexxc) ---
     1	::method m abstract
     2	  return 1
--- output ---
     2 *-* return 1
Error 99 running /tmp/tmp.2ynYndLv1M/p.rex line 2:  Translation error.
Error 99.933:  Abstract methods cannot have a method body.
--- rc=157

=== METHOD DELEGATE with body
--- source (rexxc) ---
     1	::method m delegate p
     2	  return 1
--- output ---
     2 *-* return 1
Error 99 running /tmp/tmp.IjFnYM6LjA/p.rex line 2:  Translation error.
Error 99.946:  Delegate methods cannot have a method body.
--- rc=157

=== METHOD EXTERNAL bad spec
--- source (rexxc) ---
     1	::method m external "junk"
--- output ---
     1 *-* ::method m external "junk"
Error 99 running /tmp/tmp.TMVEpDSbnE/p.rex line 1:  Translation error.
Error 99.917:  Incorrect external name specification "junk".
--- rc=157

=== METHOD EXTERNAL too many words
--- source (rexxc) ---
     1	::method m external "LIBRARY a b c"
--- output ---
     1 *-* ::method m external "LIBRARY a b c"
Error 99 running /tmp/tmp.UUfvyMg81z/p.rex line 1:  Translation error.
Error 99.917:  Incorrect external name specification "LIBRARY a b c".
--- rc=157

=== METHOD EXTERNAL empty
--- source (rexxc) ---
     1	::method m external ""
--- output ---
     1 *-* ::method m external ""
Error 99 running /tmp/tmp.3ExCkwLNIi/p.rex line 1:  Translation error.
Error 99.917:  Incorrect external name specification "".
--- rc=157

=== METHOD EXTERNAL LIBRARY only
--- source (rexxc) ---
     1	::method m external "LIBRARY"
--- output ---
     1 *-* ::method m external "LIBRARY"
Error 99 running /tmp/tmp.0vqE9avvkB/p.rex line 1:  Translation error.
Error 99.917:  Incorrect external name specification "LIBRARY".
--- rc=157

=== METHOD EXTERNAL lowercase library, extra blanks
--- source (rexxc) ---
     1	::method m external "  library   x  "
--- output ---
     1 *-* ::method m external "  library   x  "
Error 98 running /tmp/tmp.xPVd2hbOPA/p.rex line 1:  Execution error.
Error 98.903:  Unable to load library "x".
--- rc=158

=== METHOD EXTERNAL tab-separated
--- source (rexxc) ---
     1	::method m external "	LIBRARY	x"
--- output ---
     1 *-* ::method m external "	LIBRARY	x"
Error 98 running /tmp/tmp.IabEvfcvRv/p.rex line 1:  Execution error.
Error 98.903:  Unable to load library "x".
--- rc=158

=== METHOD EXTERNAL libraryx
--- source (rexxc) ---
     1	::method m external "libraryx x"
--- output ---
     1 *-* ::method m external "libraryx x"
Error 99 running /tmp/tmp.LKNh0weDC7/p.rex line 1:  Translation error.
Error 99.917:  Incorrect external name specification "libraryx x".
--- rc=157

=== METHOD EXTERNAL symbol not literal
--- source (rexxc) ---
     1	::method m external nostring
--- output ---
     1 *-* ::method m external nostring
Error 19 running /tmp/tmp.IrQqeCPnxH/p.rex line 1:  String or symbol expected.
Error 19.905:  String expected after EXTERNAL keyword.
--- rc=237

=== METHOD DELEGATE literal
--- source (rexxc) ---
     1	::method m delegate "p"
--- output ---
     1 *-* ::method m delegate "p"
Error 20 running /tmp/tmp.tDgF2Bld9U/p.rex line 1:  Symbol expected.
Error 20.926:  Symbol expected after the DELEGATE keyword.
--- rc=236

=== METHOD duplicate CLASS
--- source (rexxc) ---
     1	::method m class class
--- output ---
     1 *-* ::method m class class
Error 25 running /tmp/tmp.9UjQPvGd78/p.rex line 1:  Invalid subkeyword found.
Error 25.902:  Unknown keyword on ::METHOD directive; found "CLASS".
--- rc=231

=== METHOD PUBLIC then PRIVATE
--- source (rexxc) ---
     1	::method m public private
--- output ---
     1 *-* ::method m public private
Error 25 running /tmp/tmp.Gb9WkhY4Lq/p.rex line 1:  Invalid subkeyword found.
Error 25.902:  Unknown keyword on ::METHOD directive; found "PRIVATE".
--- rc=231

=== METHOD ABSTRACT + EXTERNAL
--- source (rexxc) ---
     1	::method m abstract external "LIBRARY x"
--- output ---
     1 *-* ::method m abstract external "LIBRARY x"
Error 25 running /tmp/tmp.D9Mc5LLDur/p.rex line 1:  Invalid subkeyword found.
Error 25.902:  Unknown keyword on ::METHOD directive; found "EXTERNAL".
--- rc=231

=== METHOD no name
--- source (rexxc) ---
     1	::method
--- output ---
     1 *-* ::method
Error 19 running /tmp/tmp.C3BarKalka/p.rex line 1:  String or symbol expected.
Error 19.902:  String or symbol expected after ::METHOD keyword.
--- rc=237

=== METHOD numeric name
--- source (rexxc) ---
     1	::method 5
--- output ---
--- rc=0

=== METHOD comma option
--- source (rexxc) ---
     1	::method m ,
--- output ---
--- rc=0

=== METHOD quoted lowercase name
--- source (rexxc) ---
     1	::method "abc"
     2	 return 1
--- output ---
--- rc=0

=== METHOD ATTRIBUTE ABSTRACT
--- source (rexxc) ---
     1	::method m attribute abstract
--- output ---
--- rc=0

=== METHOD DELEGATE ATTRIBUTE
--- source (rexxc) ---
     1	::method m delegate p attribute
--- output ---
--- rc=0

=== METHOD ATTRIBUTE EXTERNAL junk + body
--- source (rexxc) ---
     1	::method m attribute external "junk"
     2	  return 1
--- output ---
     2 *-* return 1
Error 99 running /tmp/tmp.SCrqvWXgpO/p.rex line 2:  Translation error.
Error 99.934:  Attribute methods cannot have a method body.
--- rc=157

=== METHOD ATTRIBUTE EXTERNAL junk no body
--- source (rexxc) ---
     1	::method m attribute external "junk"
--- output ---
     1 *-* ::method m attribute external "junk"
Error 99 running /tmp/tmp.welh3DOTQv/p.rex line 1:  Translation error.
Error 99.917:  Incorrect external name specification "junk".
--- rc=157

=== METHOD EXTERNAL junk + body
--- source (rexxc) ---
     1	::method m external "junk"
     2	  return 1
--- output ---
     1 *-* ::method m external "junk"
Error 99 running /tmp/tmp.GIqSgcu0Mz/p.rex line 1:  Translation error.
Error 99.917:  Incorrect external name specification "junk".
--- rc=157

=== METHOD EXTERNAL good + body
--- source (rexxc) ---
     1	::method m external "LIBRARY x"
     2	  return 1
--- output ---
     2 *-* return 1
Error 99 running /tmp/tmp.3oZtOLbTMn/p.rex line 2:  Translation error.
Error 99.936:  External methods cannot have a method body.
--- rc=157

=== ATTRIBUTE plain
--- source (rexxc) ---
     1	::attribute a
--- output ---
--- rc=0

=== ATTRIBUTE GET
--- source (rexxc) ---
     1	::attribute a get
--- output ---
--- rc=0

=== ATTRIBUTE SET
--- source (rexxc) ---
     1	::attribute a set
--- output ---
--- rc=0

=== ATTRIBUTE GET with body
--- source (rexxc) ---
     1	::attribute a get
     2	  return 1
--- output ---
--- rc=0

=== ATTRIBUTE SET with body
--- source (rexxc) ---
     1	::attribute a set
     2	  return 1
--- output ---
--- rc=0

=== ATTRIBUTE BOTH with body
--- source (rexxc) ---
     1	::attribute a
     2	  return 1
--- output ---
     2 *-* return 1
Error 99 running /tmp/tmp.cQ6ZGlDfEJ/p.rex line 2:  Translation error.
Error 99.937:  Attribute methods without a SET or GET designation cannot have a method body.
--- rc=157

=== ATTRIBUTE GET SET
--- source (rexxc) ---
     1	::attribute a get set
--- output ---
     1 *-* ::attribute a get set
Error 25 running /tmp/tmp.mkV5EhJuN2/p.rex line 1:  Invalid subkeyword found.
Error 25.925:  Unknown keyword on ::ATTRIBUTE directive; found "SET".
--- rc=231

=== ATTRIBUTE CLASS
--- source (rexxc) ---
     1	::class c
     2	::attribute a class
--- output ---
--- rc=0

=== ATTRIBUTE ABSTRACT
--- source (rexxc) ---
     1	::attribute a abstract
--- output ---
--- rc=0

=== ATTRIBUTE DELEGATE
--- source (rexxc) ---
     1	::attribute a delegate p
--- output ---
--- rc=0

=== ATTRIBUTE GET ABSTRACT with body
--- source (rexxc) ---
     1	::attribute a get abstract
     2	  return 1
--- output ---
     2 *-* return 1
Error 99 running /tmp/tmp.OBErQkh2fJ/p.rex line 2:  Translation error.
Error 99.940:  Abstract attributes cannot have a method body.
--- rc=157

=== ATTRIBUTE GET DELEGATE with body
--- source (rexxc) ---
     1	::attribute a get delegate p
     2	  return 1
--- output ---
     2 *-* return 1
Error 99 running /tmp/tmp.VSmSa2BfWN/p.rex line 2:  Translation error.
Error 99.947:  Delegate attributes cannot have a method body.
--- rc=157

=== ATTRIBUTE GET EXTERNAL bad
--- source (rexxc) ---
     1	::attribute a get external "junk"
--- output ---
     1 *-* ::attribute a get external "junk"
Error 99 running /tmp/tmp.Z4Wc5z7guf/p.rex line 1:  Translation error.
Error 99.917:  Incorrect external name specification "junk".
--- rc=157

=== ATTRIBUTE GET EXTERNAL junk + body
--- source (rexxc) ---
     1	::attribute a get external "junk"
     2	  return 1
--- output ---
     2 *-* return 1
Error 99 running /tmp/tmp.DRnzLYIzN0/p.rex line 2:  Translation error.
Error 99.935:  External attributes cannot have a method body.
--- rc=157

=== ATTRIBUTE GET EXTERNAL good + body
--- source (rexxc) ---
     1	::attribute a get external "LIBRARY x"
     2	  return 1
--- output ---
     2 *-* return 1
Error 99 running /tmp/tmp.o3SeQ20jKp/p.rex line 2:  Translation error.
Error 99.935:  External attributes cannot have a method body.
--- rc=157

=== ATTRIBUTE BOTH EXTERNAL junk + body
--- source (rexxc) ---
     1	::attribute a external "junk"
     2	  return 1
--- output ---
     2 *-* return 1
Error 99 running /tmp/tmp.zdplJcn2AY/p.rex line 2:  Translation error.
Error 99.937:  Attribute methods without a SET or GET designation cannot have a method body.
--- rc=157

=== ATTRIBUTE PROTECTED UNPROTECTED
--- source (rexxc) ---
     1	::attribute a protected unprotected
--- output ---
     1 *-* ::attribute a protected unprotected
Error 25 running /tmp/tmp.Cn0terxj53/p.rex line 1:  Invalid subkeyword found.
Error 25.925:  Unknown keyword on ::ATTRIBUTE directive; found "UNPROTECTED".
--- rc=231

=== ATTRIBUTE GUARDED UNGUARDED
--- source (rexxc) ---
     1	::attribute a guarded unguarded
--- output ---
     1 *-* ::attribute a guarded unguarded
Error 25 running /tmp/tmp.bPVV3XjRdQ/p.rex line 1:  Invalid subkeyword found.
Error 25.925:  Unknown keyword on ::ATTRIBUTE directive; found "UNGUARDED".
--- rc=231

=== ATTRIBUTE PACKAGE
--- source (rexxc) ---
     1	::attribute a package
--- output ---
--- rc=0

=== ATTRIBUTE INHERIT
--- source (rexxc) ---
     1	::attribute a inherit x
--- output ---
     1 *-* ::attribute a inherit x
Error 25 running /tmp/tmp.DibYgRGuuO/p.rex line 1:  Invalid subkeyword found.
Error 25.925:  Unknown keyword on ::ATTRIBUTE directive; found "INHERIT".
--- rc=231

=== ATTRIBUTE bare
--- source (rexxc) ---
     1	::attribute
--- output ---
     1 *-* ::attribute
Error 19 running /tmp/tmp.NnM6wNHtHw/p.rex line 1:  String or symbol expected.
Error 19.914:  String or symbol expected as ::ATTRIBUTE directive name.
--- rc=237

=== ATTRIBUTE GET body then METHOD
--- source (rexxc) ---
     1	::class c
     2	::attribute a get
     3	  return 1
     4	::method m
     5	  return 2
--- output ---
--- rc=0

=== OPTIONS DIGITS 12
--- source (rexxc) ---
     1	::options digits 12
--- output ---
--- rc=0

=== OPTIONS DIGITS 0
--- source (rexxc) ---
     1	::options digits 0
--- output ---
     1 *-* ::options digits 0
Error 26 running /tmp/tmp.PPhBXnpSct/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "0".
--- rc=230

=== OPTIONS DIGITS abc
--- source (rexxc) ---
     1	::options digits abc
--- output ---
     1 *-* ::options digits abc
Error 26 running /tmp/tmp.JlW3XFmej0/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "ABC".
--- rc=230

=== OPTIONS DIGITS 19 digits
--- source (rexxc) ---
     1	::options digits 1234567890123456789
--- output ---
     1 *-* ::options digits 1234567890123456789
Error 26 running /tmp/tmp.5leFbaONLw/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "1234567890123456789".
--- rc=230

=== OPTIONS DIGITS 18 digits
--- source (rexxc) ---
     1	::options digits 123456789012345678
--- output ---
--- rc=0

=== OPTIONS DIGITS -1
--- source (rexxc) ---
     1	::options digits -1
--- output ---
     1 *-* ::options digits -1
Error 19 running /tmp/tmp.nB2pDow43U/p.rex line 1:  String or symbol expected.
Error 19.917:  String or symbol expected as DIGITS value.
--- rc=237

=== OPTIONS DIGITS 1e2
--- source (rexxc) ---
     1	::options digits 1e2
--- output ---
--- rc=0

=== OPTIONS DIGITS nothing
--- source (rexxc) ---
     1	::options digits
--- output ---
     1 *-* ::options digits
Error 19 running /tmp/tmp.aQzjA3a67A/p.rex line 1:  String or symbol expected.
Error 19.917:  String or symbol expected as DIGITS value.
--- rc=237

=== OPTIONS DIGITS blanks
--- source (rexxc) ---
     1	::options digits " 9 "
--- output ---
--- rc=0

=== OPTIONS DIGITS tab-padded
--- source (rexxc) ---
     1	::options digits "	9	"
--- output ---
--- rc=0

=== OPTIONS DIGITS +9
--- source (rexxc) ---
     1	::options digits "+9"
--- output ---
--- rc=0

=== OPTIONS DIGITS 9.0
--- source (rexxc) ---
     1	::options digits "9.0"
--- output ---
--- rc=0

=== OPTIONS DIGITS 9.5
--- source (rexxc) ---
     1	::options digits "9.5"
--- output ---
     1 *-* ::options digits "9.5"
Error 26 running /tmp/tmp.qWGxBdPPuh/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "9.5".
--- rc=230

=== OPTIONS DIGITS 9.
--- source (rexxc) ---
     1	::options digits "9."
--- output ---
--- rc=0

=== OPTIONS DIGITS .9
--- source (rexxc) ---
     1	::options digits ".9"
--- output ---
     1 *-* ::options digits ".9"
Error 26 running /tmp/tmp.kBozwmEqKb/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found ".9".
--- rc=230

=== OPTIONS DIGITS 0009
--- source (rexxc) ---
     1	::options digits "0009"
--- output ---
--- rc=0

=== OPTIONS DIGITS empty
--- source (rexxc) ---
     1	::options digits ""
--- output ---
     1 *-* ::options digits ""
Error 26 running /tmp/tmp.1sj8FpLV3i/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "".
--- rc=230

=== OPTIONS DIGITS 1E18
--- source (rexxc) ---
     1	::options digits 1E18
--- output ---
     1 *-* ::options digits 1E18
Error 26 running /tmp/tmp.LOwIaxfMfN/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "1E18".
--- rc=230

=== OPTIONS DIGITS 1e-2
--- source (rexxc) ---
     1	::options digits "1e-2"
--- output ---
     1 *-* ::options digits "1e-2"
Error 26 running /tmp/tmp.Ai2oAtDILO/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "1e-2".
--- rc=230

=== OPTIONS DIGITS - 9
--- source (rexxc) ---
     1	::options digits "- 9"
--- output ---
     1 *-* ::options digits "- 9"
Error 26 running /tmp/tmp.uEL5dvgxmT/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "- 9".
--- rc=230

=== OPTIONS DIGITS 9 5
--- source (rexxc) ---
     1	::options digits "9 5"
--- output ---
     1 *-* ::options digits "9 5"
Error 26 running /tmp/tmp.rIXl5vjCvo/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "9 5".
--- rc=230

=== OPTIONS DIGITS 1 e2
--- source (rexxc) ---
     1	::options digits "1 e2"
--- output ---
     1 *-* ::options digits "1 e2"
Error 26 running /tmp/tmp.yrMN28uLpT/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "1 e2".
--- rc=230

=== OPTIONS FUZZ 2
--- source (rexxc) ---
     1	::options fuzz 2
--- output ---
--- rc=0

=== OPTIONS FUZZ 0
--- source (rexxc) ---
     1	::options fuzz 0
--- output ---
--- rc=0

=== OPTIONS FUZZ 9 not below digits
--- source (rexxc) ---
     1	::options fuzz 9
--- output ---
     1 *-* ::options fuzz 9
Error 33 running /tmp/tmp.aKTNwUze2Z/p.rex line 1:  Invalid expression result.
Error 33.1:  Value of NUMERIC DIGITS ("9") must exceed value of NUMERIC FUZZ ("9").
--- rc=223

=== OPTIONS FUZZ abc
--- source (rexxc) ---
     1	::options fuzz abc
--- output ---
     1 *-* ::options fuzz abc
Error 26 running /tmp/tmp.B1nSHKG38i/p.rex line 1:  Invalid whole number.
Error 26.6:  FUZZ value must be zero or a positive whole number; found "ABC".
--- rc=230

=== OPTIONS FUZZ -1 operator
--- source (rexxc) ---
     1	::options fuzz -1
--- output ---
     1 *-* ::options fuzz -1
Error 19 running /tmp/tmp.JIgJD4KXkf/p.rex line 1:  String or symbol expected.
Error 19.918:  String or symbol expected as FUZZ value.
--- rc=237

=== OPTIONS FUZZ "-1" literal
--- source (rexxc) ---
     1	::options fuzz "-1"
--- output ---
     1 *-* ::options fuzz "-1"
Error 26 running /tmp/tmp.VRqqr4BYuW/p.rex line 1:  Invalid whole number.
Error 26.6:  FUZZ value must be zero or a positive whole number; found "-1".
--- rc=230

=== OPTIONS FORM SCIENTIFIC
--- source (rexxc) ---
     1	::options form scientific
--- output ---
--- rc=0

=== OPTIONS FORM ENGINEERING
--- source (rexxc) ---
     1	::options form engineering
--- output ---
--- rc=0

=== OPTIONS FORM VALUE
--- source (rexxc) ---
     1	::options form value
--- output ---
     1 *-* ::options form value
Error 25 running /tmp/tmp.aVZ8mgnrHO/p.rex line 1:  Invalid subkeyword found.
Error 25.11:  NUMERIC FORM must be followed by one of the keywords SCIENTIFIC or ENGINEERING; found "VALUE".
--- rc=231

=== OPTIONS FORM literal
--- source (rexxc) ---
     1	::options form "scientific"
--- output ---
     1 *-* ::options form "scientific"
Error 20 running /tmp/tmp.ohIdb3ztEZ/p.rex line 1:  Symbol expected.
Error 20.925:  Symbol expected after the FORM keyword.
--- rc=236

=== OPTIONS FORM nothing
--- source (rexxc) ---
     1	::options form
--- output ---
     1 *-* ::options form
Error 20 running /tmp/tmp.eMlyo2mROB/p.rex line 1:  Symbol expected.
Error 20.925:  Symbol expected after the FORM keyword.
--- rc=236

=== OPTIONS TRACE r
--- source (rexxc) ---
     1	::options trace r
--- output ---
--- rc=0

=== OPTIONS TRACE zzz
--- source (rexxc) ---
     1	::options trace zzz
--- output ---
     1 *-* ::options trace zzz
Error 24 running /tmp/tmp.V65vDTp5jb/p.rex line 1:  Invalid TRACE request.
Error 24.1:  TRACE request letter must be one of "ACEFILNOR"; found "Z".
--- rc=232

=== OPTIONS TRACE -1
--- source (rexxc) ---
     1	::options trace -1
--- output ---
     1 *-* ::options trace -1
Error 19 running /tmp/tmp.bsSFFzgH0w/p.rex line 1:  String or symbol expected.
Error 19.919:  String or symbol expected as TRACE value.
--- rc=237

=== OPTIONS TRACE nothing
--- source (rexxc) ---
     1	::options trace
--- output ---
     1 *-* ::options trace
Error 19 running /tmp/tmp.PQ9Y9ujhbc/p.rex line 1:  String or symbol expected.
Error 19.919:  String or symbol expected as TRACE value.
--- rc=237

=== OPTIONS NOPROLOG
--- source (rexxc) ---
     1	::options noprolog
--- output ---
--- rc=0

=== OPTIONS PROLOG
--- source (rexxc) ---
     1	::options prolog
--- output ---
--- rc=0

=== OPTIONS NUMERIC INHERIT
--- source (rexxc) ---
     1	::options numeric inherit
--- output ---
--- rc=0

=== OPTIONS NUMERIC NOINHERIT
--- source (rexxc) ---
     1	::options numeric noinherit
--- output ---
--- rc=0

=== OPTIONS NUMERIC SYNTAX
--- source (rexxc) ---
     1	::options numeric syntax
--- output ---
     1 *-* ::options numeric syntax
Error 25 running /tmp/tmp.1AbnKwWLRb/p.rex line 1:  Invalid subkeyword found.
Error 25.935:  Subdirective NUMERIC must be followed by one of the keywords INHERIT or NOINHERIT; found "SYNTAX".
--- rc=231

=== OPTIONS NUMERIC literal
--- source (rexxc) ---
     1	::options numeric "inherit"
--- output ---
     1 *-* ::options numeric "inherit"
Error 20 running /tmp/tmp.rC68yMOuWD/p.rex line 1:  Symbol expected.
Error 20.935:  Symbol expected after the NUMERIC subdirective keyword.
--- rc=236

=== OPTIONS NUMERIC nothing
--- source (rexxc) ---
     1	::options numeric
--- output ---
     1 *-* ::options numeric
Error 20 running /tmp/tmp.mDHz7SXFIO/p.rex line 1:  Symbol expected.
Error 20.935:  Symbol expected after the NUMERIC subdirective keyword.
--- rc=236

=== OPTIONS ALL SYNTAX
--- source (rexxc) ---
     1	::options all syntax
--- output ---
--- rc=0

=== OPTIONS ALL CONDITION
--- source (rexxc) ---
     1	::options all condition
--- output ---
--- rc=0

=== OPTIONS ERROR SYNTAX
--- source (rexxc) ---
     1	::options error syntax
--- output ---
--- rc=0

=== OPTIONS ERROR CONDITION
--- source (rexxc) ---
     1	::options error condition
--- output ---
--- rc=0

=== OPTIONS FAILURE SYNTAX
--- source (rexxc) ---
     1	::options failure syntax
--- output ---
--- rc=0

=== OPTIONS FAILURE CONDITION
--- source (rexxc) ---
     1	::options failure condition
--- output ---
--- rc=0

=== OPTIONS LOSTDIGITS SYNTAX
--- source (rexxc) ---
     1	::options lostdigits syntax
--- output ---
--- rc=0

=== OPTIONS LOSTDIGITS CONDITION
--- source (rexxc) ---
     1	::options lostdigits condition
--- output ---
--- rc=0

=== OPTIONS NOSTRING SYNTAX
--- source (rexxc) ---
     1	::options nostring syntax
--- output ---
--- rc=0

=== OPTIONS NOSTRING CONDITION
--- source (rexxc) ---
     1	::options nostring condition
--- output ---
--- rc=0

=== OPTIONS NOTREADY SYNTAX
--- source (rexxc) ---
     1	::options notready syntax
--- output ---
--- rc=0

=== OPTIONS NOTREADY CONDITION
--- source (rexxc) ---
     1	::options notready condition
--- output ---
--- rc=0

=== OPTIONS NOVALUE SYNTAX
--- source (rexxc) ---
     1	::options novalue syntax
--- output ---
--- rc=0

=== OPTIONS NOVALUE CONDITION
--- source (rexxc) ---
     1	::options novalue condition
--- output ---
--- rc=0

=== OPTIONS NOVALUE ERROR
--- source (rexxc) ---
     1	::options novalue error
--- output ---
--- rc=0

=== OPTIONS ERROR ERROR
--- source (rexxc) ---
     1	::options error error
--- output ---
     1 *-* ::options error error
Error 25 running /tmp/tmp.iSyuJRfEtm/p.rex line 1:  Invalid subkeyword found.
Error 25.927:  Unknown keyword following "ERROR"; found "ERROR".
--- rc=231

=== OPTIONS ALL ERROR
--- source (rexxc) ---
     1	::options all error
--- output ---
     1 *-* ::options all error
Error 25 running /tmp/tmp.2FVk4HYZxR/p.rex line 1:  Invalid subkeyword found.
Error 25.927:  Unknown keyword following "ALL"; found "ERROR".
--- rc=231

=== OPTIONS NOVALUE literal
--- source (rexxc) ---
     1	::options novalue "syntax"
--- output ---
     1 *-* ::options novalue "syntax"
Error 20 running /tmp/tmp.UK9VdmEifu/p.rex line 1:  Symbol expected.
Error 20.929:  Symbol expected after NOVALUE keyword.
--- rc=236

=== OPTIONS NOVALUE nothing after
--- source (rexxc) ---
     1	::options novalue
--- output ---
     1 *-* ::options novalue
Error 20 running /tmp/tmp.SNs7Trr1XD/p.rex line 1:  Symbol expected.
Error 20.929:  Symbol expected after NOVALUE keyword.
--- rc=236

=== OPTIONS junk
--- source (rexxc) ---
     1	::options junk
--- output ---
     1 *-* ::options junk
Error 25 running /tmp/tmp.6Q7NLTntMi/p.rex line 1:  Invalid subkeyword found.
Error 25.924:  Unknown keyword on ::OPTIONS directive; found "JUNK".
--- rc=231

=== OPTIONS bare
--- source (rexxc) ---
     1	::options
--- output ---
--- rc=0

=== OPTIONS literal keyword
--- source (rexxc) ---
     1	::options "digits" 9
--- output ---
     1 *-* ::options "digits" 9
Error 25 running /tmp/tmp.amlPp75DXJ/p.rex line 1:  Invalid subkeyword found.
Error 25.924:  Unknown keyword on ::OPTIONS directive; found "digits".
--- rc=231

=== CONSTANT no value
--- source (rexxc) ---
     1	::constant c
--- output ---
--- rc=0

=== CONSTANT 5
--- source (rexxc) ---
     1	::constant c 5
--- output ---
--- rc=0

=== CONSTANT literal
--- source (rexxc) ---
     1	::constant c "x"
--- output ---
--- rc=0

=== CONSTANT symbol value
--- source (rexxc) ---
     1	::constant c abc
--- output ---
--- rc=0

=== CONSTANT -5
--- source (rexxc) ---
     1	::constant c -5
--- output ---
--- rc=0

=== CONSTANT + 5 with blank
--- source (rexxc) ---
     1	::constant c + 5
--- output ---
--- rc=0

=== CONSTANT -.5
--- source (rexxc) ---
     1	::constant c -.5
--- output ---
--- rc=0

=== CONSTANT -1e2
--- source (rexxc) ---
     1	::constant c -1e2
--- output ---
--- rc=0

=== CONSTANT -5.
--- source (rexxc) ---
     1	::constant c -5.
--- output ---
--- rc=0

=== CONSTANT -abc
--- source (rexxc) ---
     1	::constant c -abc
--- output ---
     1 *-* ::constant c -abc
Error 19 running /tmp/tmp.tKh1Ox866b/p.rex line 1:  String or symbol expected.
Error 19.916:  String or symbol expected as ::CONSTANT value.
--- rc=237

=== CONSTANT -"5"
--- source (rexxc) ---
     1	::constant c -"5"
--- output ---
     1 *-* ::constant c -"5"
Error 19 running /tmp/tmp.I4KBw3FWyI/p.rex line 1:  String or symbol expected.
Error 19.916:  String or symbol expected as ::CONSTANT value.
--- rc=237

=== CONSTANT -.true
--- source (rexxc) ---
     1	::constant c -.true
--- output ---
     1 *-* ::constant c -.true
Error 19 running /tmp/tmp.Air7EsULcs/p.rex line 1:  String or symbol expected.
Error 19.916:  String or symbol expected as ::CONSTANT value.
--- rc=237

=== CONSTANT -5x
--- source (rexxc) ---
     1	::constant c -5x
--- output ---
     1 *-* ::constant c -5x
Error 19 running /tmp/tmp.NBFm1mDP3P/p.rex line 1:  String or symbol expected.
Error 19.916:  String or symbol expected as ::CONSTANT value.
--- rc=237

=== CONSTANT -1e
--- source (rexxc) ---
     1	::constant c -1e
--- output ---
     1 *-* ::constant c -1e
Error 19 running /tmp/tmp.x2Py0ciKKZ/p.rex line 1:  String or symbol expected.
Error 19.916:  String or symbol expected as ::CONSTANT value.
--- rc=237

=== CONSTANT *5
--- source (rexxc) ---
     1	::constant c *5
--- output ---
     1 *-* ::constant c *5
Error 19 running /tmp/tmp.baKkUVNIU6/p.rex line 1:  String or symbol expected.
Error 19.916:  String or symbol expected as ::CONSTANT value.
--- rc=237

=== CONSTANT (1+2)
--- source (rexxc) ---
     1	::constant c (1+2)
--- output ---
     1 *-* ::constant c (1+2)
Error 99 running /tmp/tmp.YPxPQjsFFS/p.rex line 1:  Translation error.
Error 99.906:  A ::CONSTANT directive with an expression requires a matching ::CLASS directive.
--- rc=157

=== CONSTANT (1+2) under a class
--- source (rexxc) ---
     1	::class d
     2	::constant c (1+2)
--- output ---
--- rc=0

=== CONSTANT comma list
--- source (rexxc) ---
     1	::constant c (1,2)
--- output ---
     1 *-* ::constant c (1,2)
Error 99 running /tmp/tmp.piUjp7PIhs/p.rex line 1:  Translation error.
Error 99.906:  A ::CONSTANT directive with an expression requires a matching ::CLASS directive.
--- rc=157

=== CONSTANT ()
--- source (rexxc) ---
     1	::constant c ()
--- output ---
     1 *-* ::constant c ()
Error 35 running /tmp/tmp.DeOM6HiJUW/p.rex line 1:  Invalid expression.
Error 35.936:  Missing expression on ::CONSTANT directive.
--- rc=221

=== CONSTANT unmatched paren
--- source (rexxc) ---
     1	::constant c (1+2
--- output ---
     1 *-* ::constant c (1+2
Error 36 running /tmp/tmp.PVF3Go26UQ/p.rex line 1:  Unmatched "(" or "[" in expression.
Error 36.901:  Left parenthesis "(" in position 14 on line 1 requires a corresponding right parenthesis ")".
--- rc=220

=== CONSTANT extra data
--- source (rexxc) ---
     1	::constant c 5 6
--- output ---
     1 *-* ::constant c 5 6
Error 21 running /tmp/tmp.G938F75JPG/p.rex line 1:  Invalid data on end of clause.
Error 21.913:  Data must not follow the ::CONSTANT value; found "6".
--- rc=235

=== CONSTANT with body
--- source (rexxc) ---
     1	::constant c 5
     2	  return 1
--- output ---
     2 *-* return 1
Error 99 running /tmp/tmp.rOG6GlIunv/p.rex line 2:  Translation error.
Error 99.938:  Constant methods cannot have a method body.
--- rc=157

=== CONSTANT no name
--- source (rexxc) ---
     1	::constant
--- output ---
     1 *-* ::constant
Error 19 running /tmp/tmp.DqlMFJOZwp/p.rex line 1:  String or symbol expected.
Error 19.915:  String or symbol expected as ::CONSTANT directive name.
--- rc=237

=== ANNOTATE PACKAGE bare
--- source (rexxc) ---
     1	::annotate package
--- output ---
--- rc=0

=== ANNOTATE PACKAGE pairs
--- source (rexxc) ---
     1	::annotate package a 1 b "x"
--- output ---
--- rc=0

=== ANNOTATE value missing
--- source (rexxc) ---
     1	::annotate package a
--- output ---
     1 *-* ::annotate package a
Error 19 running /tmp/tmp.GFwVdVdQVB/p.rex line 1:  String or symbol expected.
Error 19.924:  Symbol or string expected as ::ANNOTATE attribute value.
--- rc=237

=== ANNOTATE literal name
--- source (rexxc) ---
     1	::annotate package "a" 1
--- output ---
     1 *-* ::annotate package "a" 1
Error 20 running /tmp/tmp.NSijPCwVrB/p.rex line 1:  Symbol expected.
Error 20.919:  Symbol expected as ::ANNOTATE attribute name; found "a".
--- rc=236

=== ANNOTATE bad value
--- source (rexxc) ---
     1	::annotate package a *
--- output ---
     1 *-* ::annotate package a *
Error 19 running /tmp/tmp.epzXXX9AjZ/p.rex line 1:  String or symbol expected.
Error 19.923:  Symbol or string expected as ::ANNOTATE attribute value; found "*".
--- rc=237

=== ANNOTATE signed value
--- source (rexxc) ---
     1	::annotate package a -1
--- output ---
--- rc=0

=== ANNOTATE signed non-constant
--- source (rexxc) ---
     1	::annotate package a -abc
--- output ---
     1 *-* ::annotate package a -abc
Error 19 running /tmp/tmp.RG0vuIYmfb/p.rex line 1:  String or symbol expected.
Error 19.923:  Symbol or string expected as ::ANNOTATE attribute value; found "-".
--- rc=237

=== ANNOTATE -5x
--- source (rexxc) ---
     1	::annotate package k -5x
--- output ---
     1 *-* ::annotate package k -5x
Error 19 running /tmp/tmp.H3vyyYvCYN/p.rex line 1:  String or symbol expected.
Error 19.923:  Symbol or string expected as ::ANNOTATE attribute value; found "-5X".
--- rc=237

=== ANNOTATE -1e2
--- source (rexxc) ---
     1	::annotate package k -1e2
--- output ---
--- rc=0

=== ANNOTATE CLASS
--- source (rexxc) ---
     1	::class c
     2	::annotate class c k 1
--- output ---
--- rc=0

=== ANNOTATE CLASS missing target
--- source (rexxc) ---
     1	::annotate class nosuch k 1
--- output ---
     1 *-* ::annotate class nosuch k 1
Error 99 running /tmp/tmp.nFtOPHkSGa/p.rex line 1:  Translation error.
Error 99.945:  ::ANNOTATE target class "NOSUCH" not found.
--- rc=157

=== ANNOTATE ROUTINE
--- source (rexxc) ---
     1	::routine r
     2	  return
     3	::annotate routine r k 1
--- output ---
--- rc=0

=== ANNOTATE METHOD
--- source (rexxc) ---
     1	::method m
     2	  return
     3	::annotate method m k 1
--- output ---
--- rc=0

=== ANNOTATE METHOD quoted lowercase
--- source (rexxc) ---
     1	::method "m"
     2	  return
     3	::annotate method "m" k 1
--- output ---
--- rc=0

=== ANNOTATE ATTRIBUTE
--- source (rexxc) ---
     1	::attribute a
     2	::annotate attribute a k 1
--- output ---
--- rc=0

=== ANNOTATE CONSTANT
--- source (rexxc) ---
     1	::class c
     2	::constant k 1
     3	::annotate constant k x 1
--- output ---
--- rc=0

=== ANNOTATE junk type
--- source (rexxc) ---
     1	::annotate junk k 1
--- output ---
     1 *-* ::annotate junk k 1
Error 25 running /tmp/tmp.UgQipmVF3e/p.rex line 1:  Invalid subkeyword found.
Error 25.928:  Unknown keyword on ::ANNOTATE directive; found "JUNK".
--- rc=231

=== ANNOTATE PUBLIC as type
--- source (rexxc) ---
     1	::annotate public k 1
--- output ---
     1 *-* ::annotate public k 1
Error 25 running /tmp/tmp.a0i0LZuwwG/p.rex line 1:  Invalid subkeyword found.
Error 25.928:  Unknown keyword on ::ANNOTATE directive; found "PUBLIC".
--- rc=231

=== ANNOTATE bare
--- source (rexxc) ---
     1	::annotate
--- output ---
     1 *-* ::annotate
Error 20 running /tmp/tmp.o6Ygl10aj0/p.rex line 1:  Symbol expected.
Error 20.924:  Symbol expected for the ::ANNOTATE type.
--- rc=236

=== ANNOTATE literal type
--- source (rexxc) ---
     1	::annotate "package"
--- output ---
     1 *-* ::annotate "package"
Error 20 running /tmp/tmp.lqCR0Yw6z6/p.rex line 1:  Symbol expected.
Error 20.924:  Symbol expected for the ::ANNOTATE type.
--- rc=236

=== ANNOTATE CLASS no name
--- source (rexxc) ---
     1	::annotate class
--- output ---
     1 *-* ::annotate class
Error 19 running /tmp/tmp.neBWuhJb1X/p.rex line 1:  String or symbol expected.
Error 19.925:  Symbol or string expected after ::ANNOTATE CLASS keyword.
--- rc=237

=== REQUIRES nosuch
--- source (rexxc) ---
     1	::requires "nosuch"
--- output ---
--- rc=0

=== REQUIRES LIBRARY
--- source (rexxc) ---
     1	::requires x library
--- output ---
--- rc=0

=== REQUIRES NAMESPACE
--- source (rexxc) ---
     1	::requires "nosuch" namespace ns
--- output ---
--- rc=0

=== REQUIRES LIBRARY then NAMESPACE
--- source (rexxc) ---
     1	::requires x library namespace ns
--- output ---
     1 *-* ::requires x library namespace ns
Error 25 running /tmp/tmp.8PBWFk4mkg/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "NAMESPACE".
--- rc=231

=== REQUIRES NAMESPACE then LIBRARY
--- source (rexxc) ---
     1	::requires x namespace ns library
--- output ---
     1 *-* ::requires x namespace ns library
Error 25 running /tmp/tmp.IPMEN5Uv3b/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "LIBRARY".
--- rc=231

=== REQUIRES NAMESPACE twice
--- source (rexxc) ---
     1	::requires x namespace a namespace b
--- output ---
     1 *-* ::requires x namespace a namespace b
Error 25 running /tmp/tmp.lmvyOVJ9D3/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "NAMESPACE".
--- rc=231

=== REQUIRES NAMESPACE REXX reserved
--- source (rexxc) ---
     1	::requires "nosuch" namespace rexx
--- output ---
     1 *-* ::requires "nosuch" namespace rexx
Error 99 running /tmp/tmp.ysExo585Tv/p.rex line 1:  Translation error.
Error 99.944:  The REXX name is reserved for the language-provided namespace.
--- rc=157

=== REQUIRES NAMESPACE literal
--- source (rexxc) ---
     1	::requires "nosuch" namespace "ns"
--- output ---
     1 *-* ::requires "nosuch" namespace "ns"
Error 20 running /tmp/tmp.1amJWBZbT3/p.rex line 1:  Symbol expected.
Error 20.920:  Symbol expected after NAMESPACE keyword.
--- rc=236

=== REQUIRES bare
--- source (rexxc) ---
     1	::requires
--- output ---
     1 *-* ::requires
Error 19 running /tmp/tmp.ANIcxzZ8uZ/p.rex line 1:  String or symbol expected.
Error 19.904:  String or symbol expected after ::REQUIRES keyword.
--- rc=237

=== REQUIRES junk option
--- source (rexxc) ---
     1	::requires x junk
--- output ---
     1 *-* ::requires x junk
Error 25 running /tmp/tmp.ZRzajDmrC7/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "JUNK".
--- rc=231

=== REQUIRES LIBRARY twice
--- source (rexxc) ---
     1	::requires x library library
--- output ---
     1 *-* ::requires x library library
Error 25 running /tmp/tmp.XyVdovKl2t/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "LIBRARY".
--- rc=231

=== ROUTINE with body
--- source (rexxc) ---
     1	::routine r
     2	  return 1
--- output ---
--- rc=0

=== ROUTINE PUBLIC
--- source (rexxc) ---
     1	::routine r public
     2	  return 1
--- output ---
--- rc=0

=== ROUTINE PRIVATE
--- source (rexxc) ---
     1	::routine r private
     2	  return 1
--- output ---
--- rc=0

=== ROUTINE PACKAGE
--- source (rexxc) ---
     1	::routine r package
     2	 return
--- output ---
     1 *-* ::routine r package
Error 25 running /tmp/tmp.rTn5ufur2R/p.rex line 1:  Invalid subkeyword found.
Error 25.903:  Unknown keyword on ::ROUTINE directive; found "PACKAGE".
--- rc=231

=== ROUTINE PUBLIC PRIVATE
--- source (rexxc) ---
     1	::routine r public private
     2	  return 1
--- output ---
     1 *-* ::routine r public private
Error 25 running /tmp/tmp.NMKx2z85sZ/p.rex line 1:  Invalid subkeyword found.
Error 25.903:  Unknown keyword on ::ROUTINE directive; found "PRIVATE".
--- rc=231

=== ROUTINE EXTERNAL junk
--- source (rexxc) ---
     1	::routine r external "junk"
--- output ---
     1 *-* ::routine r external "junk"
Error 99 running /tmp/tmp.dcfcVCN9M4/p.rex line 1:  Translation error.
Error 99.917:  Incorrect external name specification "junk".
--- rc=157

=== ROUTINE EXTERNAL junk + body
--- source (rexxc) ---
     1	::routine r external "junk"
     2	  return 1
--- output ---
     1 *-* ::routine r external "junk"
Error 99 running /tmp/tmp.iXCzDIA1MI/p.rex line 1:  Translation error.
Error 99.917:  Incorrect external name specification "junk".
--- rc=157

=== ROUTINE EXTERNAL good + body
--- source (rexxc) ---
     1	::routine r external "LIBRARY x"
     2	  return 1
--- output ---
     2 *-* return 1
Error 99 running /tmp/tmp.TQoMqvXEqN/p.rex line 2:  Translation error.
Error 99.939:  External routines cannot have a code body.
--- rc=157

=== ROUTINE EXTERNAL LIBRARY x
--- source (rexxc) ---
     1	::routine r external "LIBRARY x"
--- output ---
     1 *-* ::routine r external "LIBRARY x"
Error 98 running /tmp/tmp.NOgwJxm6AF/p.rex line 1:  Execution error.
Error 98.903:  Unable to load library "x".
--- rc=158

=== ROUTINE EXTERNAL REGISTERED too many
--- source (rexxc) ---
     1	::routine r external "REGISTERED a b c"
--- output ---
     1 *-* ::routine r external "REGISTERED a b c"
Error 99 running /tmp/tmp.g1oOsECKQv/p.rex line 1:  Translation error.
Error 99.917:  Incorrect external name specification "REGISTERED a b c".
--- rc=157

=== ROUTINE EXTERNAL REGISTERED only
--- source (rexxc) ---
     1	::routine r external "REGISTERED"
--- output ---
     1 *-* ::routine r external "REGISTERED"
Error 99 running /tmp/tmp.aSoyb6X0Fz/p.rex line 1:  Translation error.
Error 99.917:  Incorrect external name specification "REGISTERED".
--- rc=157

=== ROUTINE EXTERNAL registered lowercase
--- source (rexxc) ---
     1	::routine r external "registered x"
--- output ---
     1 *-* ::routine r external "registered x"
Error 90 running /tmp/tmp.NOWwLQtWUd/p.rex line 1:  External name not found.
Error 90.999:  Unable to find external routine "R".
--- rc=166

=== ROUTINE EXTERNAL twice
--- source (rexxc) ---
     1	::routine r external "LIBRARY x" external "LIBRARY y"
--- output ---
     1 *-* ::routine r external "LIBRARY x" external "LIBRARY y"
Error 25 running /tmp/tmp.jaLu1N1E4a/p.rex line 1:  Invalid subkeyword found.
Error 25.901:  Unknown keyword on ::CLASS directive; found "EXTERNAL".
--- rc=231

=== ROUTINE CLASS
--- source (rexxc) ---
     1	::routine r class
--- output ---
     1 *-* ::routine r class
Error 25 running /tmp/tmp.QCU77gZ5r8/p.rex line 1:  Invalid subkeyword found.
Error 25.903:  Unknown keyword on ::ROUTINE directive; found "CLASS".
--- rc=231

=== ROUTINE bare
--- source (rexxc) ---
     1	::routine
--- output ---
     1 *-* ::routine
Error 19 running /tmp/tmp.5SftIIpdbN/p.rex line 1:  String or symbol expected.
Error 19.903:  String or symbol expected after ::ROUTINE keyword.
--- rc=237

=== ROUTINE junk
--- source (rexxc) ---
     1	::routine r junk
--- output ---
     1 *-* ::routine r junk
Error 25 running /tmp/tmp.dgILqSLyPQ/p.rex line 1:  Invalid subkeyword found.
Error 25.903:  Unknown keyword on ::ROUTINE directive; found "JUNK".
--- rc=231

=== RESOURCE default marker
--- source (rexxc) ---
     1	::resource data
     2	line1
     3	::END
--- output ---
--- rc=0

=== RESOURCE lowercase marker
--- source (rexxc) ---
     1	::resource data
     2	line1
     3	::end
--- output ---
     1 *-* ::resource data
Error 99 running /tmp/tmp.oA6BnXDspy/p.rex line 1:  Translation error.
Error 99.943:  Missing ::RESOURCE end marker "::END" for resource "DATA".
--- rc=157

=== RESOURCE custom marker literal
--- source (rexxc) ---
     1	::resource data end "%"
     2	line1
     3	%
--- output ---
--- rc=0

=== RESOURCE custom marker symbol upcased
--- source (rexxc) ---
     1	::resource data end stop
     2	line1
     3	STOP
--- output ---
--- rc=0

=== RESOURCE custom marker symbol lower in body
--- source (rexxc) ---
     1	::resource data end stop
     2	line1
     3	stop
--- output ---
     1 *-* ::resource data end stop
Error 99 running /tmp/tmp.g3Osy2Myb8/p.rex line 1:  Translation error.
Error 99.943:  Missing ::RESOURCE end marker "STOP" for resource "DATA".
--- rc=157

=== RESOURCE bad subkeyword
--- source (rexxc) ---
     1	::resource data junk
     2	line1
     3	::END
--- output ---
     1 *-* ::resource data junk
Error 25 running /tmp/tmp.HpfBCZFYnu/p.rex line 1:  Invalid subkeyword found.
Error 25.926:  Unknown keyword on ::RESOURCE directive; found "JUNK".
--- rc=231

=== RESOURCE real sub-directive that is not END
--- source (rexxc) ---
     1	::resource d public x
     2	body
     3	::END
--- output ---
     1 *-* ::resource d public x
Error 25 running /tmp/tmp.9DLETDckvs/p.rex line 1:  Invalid subkeyword found.
Error 25.926:  Unknown keyword on ::RESOURCE directive; found "PUBLIC".
--- rc=231

=== RESOURCE END with no marker
--- source (rexxc) ---
     1	::resource data end
     2	line1
     3	::END
--- output ---
     1 *-* ::resource data end
Error 19 running /tmp/tmp.ajBFsO2dTD/p.rex line 1:  String or symbol expected.
Error 19.921:  String or symbol expected after ::RESOURCE END keyword.
--- rc=237

=== RESOURCE extra data after marker
--- source (rexxc) ---
     1	::resource data end "x" extra
     2	line1
     3	x
--- output ---
     1 *-* ::resource data end "x" extra
Error 21 running /tmp/tmp.Qxt7b46FDU/p.rex line 1:  Invalid data on end of clause.
Error 21.914:  Data must not follow the ::RESOURCE directive; found "EXTRA".
--- rc=235

=== RESOURCE no name
--- source (rexxc) ---
     1	::resource
--- output ---
     1 *-* ::resource
Error 19 running /tmp/tmp.6rUehrLIFA/p.rex line 1:  String or symbol expected.
Error 19.920:  String or symbol expected as ::RESOURCE directive name.
--- rc=237

=== RESOURCE missing terminator
--- source (rexxc) ---
     1	::resource data
     2	line1
--- output ---
     1 *-* ::resource data
Error 99 running /tmp/tmp.KqFmKOEqMY/p.rex line 1:  Translation error.
Error 99.943:  Missing ::RESOURCE end marker "::END" for resource "DATA".
--- rc=157

=== RESOURCE with semicolon
--- source (rexxc) ---
     1	::resource data; say 1
     2	line1
     3	::END
--- output ---
--- rc=0

=== RESOURCE unparseable body
--- source (rexxc) ---
     1	::resource data
     2	this is 'unmatched and /* unclosed
     3	::END
--- output ---
--- rc=0

=== RESOURCE marker indented
--- source (rexxc) ---
     1	::resource data
     2	line1
     3	  ::END
--- output ---
     1 *-* ::resource data
Error 99 running /tmp/tmp.HraeWMuAuT/p.rex line 1:  Translation error.
Error 99.943:  Missing ::RESOURCE end marker "::END" for resource "DATA".
--- rc=157

=== RESOURCE marker with trailing junk
--- source (rexxc) ---
     1	::resource data
     2	line1
     3	::END junk
--- output ---
--- rc=0

=== RESOURCE then METHOD
--- source (rexxc) ---
     1	::resource data
     2	x
     3	::END
     4	::method m
     5	 return 1
--- output ---
--- rc=0

=== RESOURCE duplicate name
--- source (rexxc) ---
     1	::resource d
     2	x
     3	::END
     4	::resource d
     5	y
     6	::END
--- output ---
     4 *-* ::resource d
Error 99 running /tmp/tmp.7irGZndldg/p.rex line 4:  Translation error.
Error 99.942:  Duplicate ::RESOURCE directive instruction.
--- rc=157

=== unknown directive
--- source (rexxc) ---
     1	::junk
--- output ---
     1 *-* ::junk
Error 99 running /tmp/tmp.oQagJjzrYn/p.rex line 1:  Translation error.
Error 99.916:  Unrecognized directive instruction.
--- rc=157

=== directive numeric symbol
--- source (rexxc) ---
     1	:: 5
--- output ---
     1 *-* :: 5
Error 99 running /tmp/tmp.ACrarcQcVK/p.rex line 1:  Translation error.
Error 99.916:  Unrecognized directive instruction.
--- rc=157

=== directive literal
--- source (rexxc) ---
     1	:: "x"
--- output ---
     1 *-* :: "x"
Error 20 running /tmp/tmp.rqERhnmPZL/p.rex line 1:  Symbol expected.
Error 20.916:  Symbol expected after ::.
--- rc=236

=== bare ::
--- source (rexxc) ---
     1	::
--- output ---
     1 *-* ::
Error 20 running /tmp/tmp.s5Md7bImDk/p.rex line 1:  Symbol expected.
Error 20.916:  Symbol expected after ::.
--- rc=236

=== single colon
--- source (rexxc) ---
     1	nop
     2	:junk
--- output ---
     2 *-* :junk
Error 35 running /tmp/tmp.TMt8AOPpLL/p.rex line 2:  Invalid expression.
Error 35.1:  Incorrect expression detected at ":".
--- rc=221

=== directive in INTERPRET
--- source (rexxc) ---
     1	interpret "::class c"
--- output ---
--- rc=0

=== duplicate class
--- source (rexxc) ---
     1	::class c
     2	::class c
--- output ---
     2 *-* ::class c
Error 99 running /tmp/tmp.BKAYowlE1O/p.rex line 2:  Translation error.
Error 99.901:  Duplicate ::CLASS directive instruction.
--- rc=157

=== duplicate routine
--- source (rexxc) ---
     1	::routine r
     2	 return
     3	::routine r
     4	 return
--- output ---
     3 *-* ::routine r
Error 99 running /tmp/tmp.sJj9F2JQd1/p.rex line 3:  Translation error.
Error 99.903:  Duplicate ::ROUTINE directive instruction.
--- rc=157

=== body error line, blank lines between
--- source (rexxc) ---
     1	::method m abstract
     2	
     3	
     4	
     5	  return 1
--- output ---
     5 *-* return 1
Error 99 running /tmp/tmp.wudcbTxKOV/p.rex line 5:  Translation error.
Error 99.933:  Abstract methods cannot have a method body.
--- rc=157

=== directive error line, blank lines before
--- source (rexxc) ---
     1	::class c
     2	
     3	
     4	
     5	::method m junk
--- output ---
     5 *-* ::method m junk
Error 25 running /tmp/tmp.X2JypfnNcw/p.rex line 5:  Invalid subkeyword found.
Error 25.902:  Unknown keyword on ::METHOD directive; found "JUNK".
--- rc=231

=== TRACE " 9 " blanks
--- source (rexxc) ---
     1	trace " 9 "
     2	nop
--- output ---
--- rc=0

=== TRACE "9 5" internal blank
--- source (rexxc) ---
     1	trace '9 5'
     2	nop
--- output ---
     1 *-* trace '9 5'
Error 24 running /tmp/tmp.pQx8jIIwgA/p.rex line 1:  Invalid TRACE request.
Error 24.1:  TRACE request letter must be one of "ACEFILNOR"; found "9".
--- rc=232

=== OPTIONS DIGITS twice, later wins
--- source (rexx) ---
     1	say numeric digits()
     2	::options digits 12
     3	::options digits 5
--- output ---
NUMERIC 5
--- rc=0

=== OPTIONS DIGITS twice reversed
--- source (rexx) ---
     1	say numeric digits()
     2	::options digits 5
     3	::options digits 12
--- output ---
NUMERIC 12
--- rc=0

=== OPTIONS two options in one directive
--- source (rexx) ---
     1	say numeric digits() numeric fuzz()
     2	::options digits 12 fuzz 3
--- output ---
NUMERIC 12 NUMERIC 3
--- rc=0

```

### Sub-directive reachability, both directions

The forty accepted rows and the forty refused ones, run the same way. `CLASS`'s
accepted row carries a leading `::class c` because the option needs an active
class; `EXTERNAL`'s reaches 98.903 because the library does not exist, and both
of those non-zero rcs are the evidence that the parse succeeded.

```text
=== ACCEPT ABSTRACT
--- source (rexxc) ---
     1	::class c abstract
--- output ---
--- rc=0

=== ACCEPT ALL
--- source (rexxc) ---
     1	::options all syntax
--- output ---
--- rc=0

=== ACCEPT ATTRIBUTE
--- source (rexxc) ---
     1	::method m attribute
--- output ---
--- rc=0

=== ACCEPT CLASS
--- source (rexxc) ---
     1	::class c
     2	::method m class
     3	  return 1
--- output ---
--- rc=0

=== ACCEPT CONDITION
--- source (rexxc) ---
     1	::options novalue condition
--- output ---
--- rc=0

=== ACCEPT CONSTANT
--- source (rexxc) ---
     1	::class c
     2	::constant k 1
     3	::annotate constant k x 1
--- output ---
--- rc=0

=== ACCEPT DELEGATE
--- source (rexxc) ---
     1	::method m delegate p
--- output ---
--- rc=0

=== ACCEPT DIGITS
--- source (rexxc) ---
     1	::options digits 12
--- output ---
--- rc=0

=== ACCEPT END
--- source (rexxc) ---
     1	::resource d end stop
     2	line
     3	STOP
--- output ---
--- rc=0

=== ACCEPT ERROR
--- source (rexxc) ---
     1	::options error syntax
--- output ---
--- rc=0

=== ACCEPT EXTERNAL
--- source (rexxc) ---
     1	::routine r external "LIBRARY x"
--- output ---
     1 *-* ::routine r external "LIBRARY x"
Error 98 running /tmp/tmp.pPPEHs8Os6/p.rex line 1:  Execution error.
Error 98.903:  Unable to load library "x".
--- rc=158

=== ACCEPT FAILURE
--- source (rexxc) ---
     1	::options failure syntax
--- output ---
--- rc=0

=== ACCEPT FORM
--- source (rexxc) ---
     1	::options form scientific
--- output ---
--- rc=0

=== ACCEPT FUZZ
--- source (rexxc) ---
     1	::options fuzz 3
--- output ---
--- rc=0

=== ACCEPT GET
--- source (rexxc) ---
     1	::attribute a get
--- output ---
--- rc=0

=== ACCEPT GUARDED
--- source (rexxc) ---
     1	::method m guarded
     2	  return 1
--- output ---
--- rc=0

=== ACCEPT INHERIT
--- source (rexxc) ---
     1	::class m mixinclass object
     2	::class c inherit m
--- output ---
--- rc=0

=== ACCEPT LIBRARY
--- source (rexxc) ---
     1	::requires x library
--- output ---
--- rc=0

=== ACCEPT LOSTDIGITS
--- source (rexxc) ---
     1	::options lostdigits syntax
--- output ---
--- rc=0

=== ACCEPT METACLASS
--- source (rexxc) ---
     1	::class c metaclass class
--- output ---
--- rc=0

=== ACCEPT METHOD
--- source (rexxc) ---
     1	::method m
     2	  return
     3	::annotate method m k 1
--- output ---
--- rc=0

=== ACCEPT MIXINCLASS
--- source (rexxc) ---
     1	::class c mixinclass object
--- output ---
--- rc=0

=== ACCEPT NAMESPACE
--- source (rexxc) ---
     1	::requires "nosuch" namespace ns
--- output ---
--- rc=0

=== ACCEPT NOPROLOG
--- source (rexxc) ---
     1	::options noprolog
--- output ---
--- rc=0

=== ACCEPT NOSTRING
--- source (rexxc) ---
     1	::options nostring syntax
--- output ---
--- rc=0

=== ACCEPT NOTREADY
--- source (rexxc) ---
     1	::options notready syntax
--- output ---
--- rc=0

=== ACCEPT NOVALUE
--- source (rexxc) ---
     1	::options novalue syntax
--- output ---
--- rc=0

=== ACCEPT NUMERIC
--- source (rexxc) ---
     1	::options numeric inherit
--- output ---
--- rc=0

=== ACCEPT PACKAGE
--- source (rexxc) ---
     1	::annotate package k 1
--- output ---
--- rc=0

=== ACCEPT PRIVATE
--- source (rexxc) ---
     1	::class c private
--- output ---
--- rc=0

=== ACCEPT PROLOG
--- source (rexxc) ---
     1	::options prolog
--- output ---
--- rc=0

=== ACCEPT PROTECTED
--- source (rexxc) ---
     1	::method m protected
     2	  return 1
--- output ---
--- rc=0

=== ACCEPT PUBLIC
--- source (rexxc) ---
     1	::class c public
--- output ---
--- rc=0

=== ACCEPT ROUTINE
--- source (rexxc) ---
     1	::routine r
     2	  return
     3	::annotate routine r k 1
--- output ---
--- rc=0

=== ACCEPT SET
--- source (rexxc) ---
     1	::attribute a set
--- output ---
--- rc=0

=== ACCEPT SUBCLASS
--- source (rexxc) ---
     1	::class c subclass object
--- output ---
--- rc=0

=== ACCEPT SYNTAX
--- source (rexxc) ---
     1	::options novalue syntax
--- output ---
--- rc=0

=== ACCEPT TRACE
--- source (rexxc) ---
     1	::options trace r
--- output ---
--- rc=0

=== ACCEPT UNGUARDED
--- source (rexxc) ---
     1	::method m unguarded
     2	  return 1
--- output ---
--- rc=0

=== ACCEPT UNPROTECTED
--- source (rexxc) ---
     1	::method m unprotected
     2	  return 1
--- output ---
--- rc=0

=== REJECT ABSTRACT on ::REQUIRES
--- source (rexxc) ---
     1	::requires x ABSTRACT
--- output ---
     1 *-* ::requires x ABSTRACT
Error 25 running /tmp/tmp.g00YDQ1f4a/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "ABSTRACT".
--- rc=231

=== REJECT ALL on ::REQUIRES
--- source (rexxc) ---
     1	::requires x ALL
--- output ---
     1 *-* ::requires x ALL
Error 25 running /tmp/tmp.p2kOQw8SiZ/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "ALL".
--- rc=231

=== REJECT ATTRIBUTE on ::REQUIRES
--- source (rexxc) ---
     1	::requires x ATTRIBUTE
--- output ---
     1 *-* ::requires x ATTRIBUTE
Error 25 running /tmp/tmp.ZgSlaOIv5y/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "ATTRIBUTE".
--- rc=231

=== REJECT CLASS on ::REQUIRES
--- source (rexxc) ---
     1	::requires x CLASS
--- output ---
     1 *-* ::requires x CLASS
Error 25 running /tmp/tmp.foY58yzc4N/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "CLASS".
--- rc=231

=== REJECT CONDITION on ::REQUIRES
--- source (rexxc) ---
     1	::requires x CONDITION
--- output ---
     1 *-* ::requires x CONDITION
Error 25 running /tmp/tmp.dhDAcPizdE/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "CONDITION".
--- rc=231

=== REJECT CONSTANT on ::REQUIRES
--- source (rexxc) ---
     1	::requires x CONSTANT
--- output ---
     1 *-* ::requires x CONSTANT
Error 25 running /tmp/tmp.E7zkNYFErp/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "CONSTANT".
--- rc=231

=== REJECT DELEGATE on ::REQUIRES
--- source (rexxc) ---
     1	::requires x DELEGATE
--- output ---
     1 *-* ::requires x DELEGATE
Error 25 running /tmp/tmp.e7uzqHTtJO/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "DELEGATE".
--- rc=231

=== REJECT DIGITS on ::REQUIRES
--- source (rexxc) ---
     1	::requires x DIGITS
--- output ---
     1 *-* ::requires x DIGITS
Error 25 running /tmp/tmp.r6ElZ4p74j/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "DIGITS".
--- rc=231

=== REJECT END on ::REQUIRES
--- source (rexxc) ---
     1	::requires x END
--- output ---
     1 *-* ::requires x END
Error 25 running /tmp/tmp.hOWVD6hAZr/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "END".
--- rc=231

=== REJECT ERROR on ::REQUIRES
--- source (rexxc) ---
     1	::requires x ERROR
--- output ---
     1 *-* ::requires x ERROR
Error 25 running /tmp/tmp.UcwulZNqf7/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "ERROR".
--- rc=231

=== REJECT EXTERNAL on ::REQUIRES
--- source (rexxc) ---
     1	::requires x EXTERNAL
--- output ---
     1 *-* ::requires x EXTERNAL
Error 25 running /tmp/tmp.9RvIpWOsKP/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "EXTERNAL".
--- rc=231

=== REJECT FAILURE on ::REQUIRES
--- source (rexxc) ---
     1	::requires x FAILURE
--- output ---
     1 *-* ::requires x FAILURE
Error 25 running /tmp/tmp.5JPuQEORjy/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "FAILURE".
--- rc=231

=== REJECT FORM on ::REQUIRES
--- source (rexxc) ---
     1	::requires x FORM
--- output ---
     1 *-* ::requires x FORM
Error 25 running /tmp/tmp.OAKN8oxSeM/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "FORM".
--- rc=231

=== REJECT FUZZ on ::REQUIRES
--- source (rexxc) ---
     1	::requires x FUZZ
--- output ---
     1 *-* ::requires x FUZZ
Error 25 running /tmp/tmp.UsmpG6zjyb/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "FUZZ".
--- rc=231

=== REJECT GET on ::REQUIRES
--- source (rexxc) ---
     1	::requires x GET
--- output ---
     1 *-* ::requires x GET
Error 25 running /tmp/tmp.J2KBRf7UdN/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "GET".
--- rc=231

=== REJECT GUARDED on ::REQUIRES
--- source (rexxc) ---
     1	::requires x GUARDED
--- output ---
     1 *-* ::requires x GUARDED
Error 25 running /tmp/tmp.CqdlgRDeAs/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "GUARDED".
--- rc=231

=== REJECT INHERIT on ::REQUIRES
--- source (rexxc) ---
     1	::requires x INHERIT
--- output ---
     1 *-* ::requires x INHERIT
Error 25 running /tmp/tmp.v8lneJnIb8/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "INHERIT".
--- rc=231

=== REJECT LOSTDIGITS on ::REQUIRES
--- source (rexxc) ---
     1	::requires x LOSTDIGITS
--- output ---
     1 *-* ::requires x LOSTDIGITS
Error 25 running /tmp/tmp.ze96srQZGc/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "LOSTDIGITS".
--- rc=231

=== REJECT METACLASS on ::REQUIRES
--- source (rexxc) ---
     1	::requires x METACLASS
--- output ---
     1 *-* ::requires x METACLASS
Error 25 running /tmp/tmp.xZPUHZw9Yz/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "METACLASS".
--- rc=231

=== REJECT METHOD on ::REQUIRES
--- source (rexxc) ---
     1	::requires x METHOD
--- output ---
     1 *-* ::requires x METHOD
Error 25 running /tmp/tmp.xNc5afhHsR/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "METHOD".
--- rc=231

=== REJECT MIXINCLASS on ::REQUIRES
--- source (rexxc) ---
     1	::requires x MIXINCLASS
--- output ---
     1 *-* ::requires x MIXINCLASS
Error 25 running /tmp/tmp.iSenKRdGdD/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "MIXINCLASS".
--- rc=231

=== REJECT NOPROLOG on ::REQUIRES
--- source (rexxc) ---
     1	::requires x NOPROLOG
--- output ---
     1 *-* ::requires x NOPROLOG
Error 25 running /tmp/tmp.1ViiWJutxI/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "NOPROLOG".
--- rc=231

=== REJECT NOSTRING on ::REQUIRES
--- source (rexxc) ---
     1	::requires x NOSTRING
--- output ---
     1 *-* ::requires x NOSTRING
Error 25 running /tmp/tmp.858qbRo0BY/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "NOSTRING".
--- rc=231

=== REJECT NOTREADY on ::REQUIRES
--- source (rexxc) ---
     1	::requires x NOTREADY
--- output ---
     1 *-* ::requires x NOTREADY
Error 25 running /tmp/tmp.Cl6fxsx600/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "NOTREADY".
--- rc=231

=== REJECT NOVALUE on ::REQUIRES
--- source (rexxc) ---
     1	::requires x NOVALUE
--- output ---
     1 *-* ::requires x NOVALUE
Error 25 running /tmp/tmp.8UvBmyJO5Z/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "NOVALUE".
--- rc=231

=== REJECT NUMERIC on ::REQUIRES
--- source (rexxc) ---
     1	::requires x NUMERIC
--- output ---
     1 *-* ::requires x NUMERIC
Error 25 running /tmp/tmp.eQOBsfsRGQ/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "NUMERIC".
--- rc=231

=== REJECT PACKAGE on ::REQUIRES
--- source (rexxc) ---
     1	::requires x PACKAGE
--- output ---
     1 *-* ::requires x PACKAGE
Error 25 running /tmp/tmp.Le5ec0MlEw/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "PACKAGE".
--- rc=231

=== REJECT PRIVATE on ::REQUIRES
--- source (rexxc) ---
     1	::requires x PRIVATE
--- output ---
     1 *-* ::requires x PRIVATE
Error 25 running /tmp/tmp.cP6OdVCamz/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "PRIVATE".
--- rc=231

=== REJECT PROLOG on ::REQUIRES
--- source (rexxc) ---
     1	::requires x PROLOG
--- output ---
     1 *-* ::requires x PROLOG
Error 25 running /tmp/tmp.LyLuTF2Muv/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "PROLOG".
--- rc=231

=== REJECT PROTECTED on ::REQUIRES
--- source (rexxc) ---
     1	::requires x PROTECTED
--- output ---
     1 *-* ::requires x PROTECTED
Error 25 running /tmp/tmp.NwEzNG9bCi/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "PROTECTED".
--- rc=231

=== REJECT PUBLIC on ::REQUIRES
--- source (rexxc) ---
     1	::requires x PUBLIC
--- output ---
     1 *-* ::requires x PUBLIC
Error 25 running /tmp/tmp.CexREfCEkl/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "PUBLIC".
--- rc=231

=== REJECT ROUTINE on ::REQUIRES
--- source (rexxc) ---
     1	::requires x ROUTINE
--- output ---
     1 *-* ::requires x ROUTINE
Error 25 running /tmp/tmp.Rzi4pOpR4f/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "ROUTINE".
--- rc=231

=== REJECT SET on ::REQUIRES
--- source (rexxc) ---
     1	::requires x SET
--- output ---
     1 *-* ::requires x SET
Error 25 running /tmp/tmp.IjKknEoJ43/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "SET".
--- rc=231

=== REJECT SUBCLASS on ::REQUIRES
--- source (rexxc) ---
     1	::requires x SUBCLASS
--- output ---
     1 *-* ::requires x SUBCLASS
Error 25 running /tmp/tmp.3uCQ6ESoaV/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "SUBCLASS".
--- rc=231

=== REJECT SYNTAX on ::REQUIRES
--- source (rexxc) ---
     1	::requires x SYNTAX
--- output ---
     1 *-* ::requires x SYNTAX
Error 25 running /tmp/tmp.DJd8ch9GNa/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "SYNTAX".
--- rc=231

=== REJECT TRACE on ::REQUIRES
--- source (rexxc) ---
     1	::requires x TRACE
--- output ---
     1 *-* ::requires x TRACE
Error 25 running /tmp/tmp.oCiOQSmizG/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "TRACE".
--- rc=231

=== REJECT UNGUARDED on ::REQUIRES
--- source (rexxc) ---
     1	::requires x UNGUARDED
--- output ---
     1 *-* ::requires x UNGUARDED
Error 25 running /tmp/tmp.bWz5W1MgZZ/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "UNGUARDED".
--- rc=231

=== REJECT UNPROTECTED on ::REQUIRES
--- source (rexxc) ---
     1	::requires x UNPROTECTED
--- output ---
     1 *-* ::requires x UNPROTECTED
Error 25 running /tmp/tmp.ERt61QFbOi/p.rex line 1:  Invalid subkeyword found.
Error 25.904:  Unknown keyword on ::REQUIRES directive; found "UNPROTECTED".
--- rc=231

=== REJECT NAMESPACE on ::CLASS
--- source (rexxc) ---
     1	::class c NAMESPACE
--- output ---
     1 *-* ::class c NAMESPACE
Error 25 running /tmp/tmp.RZFpmycNNz/p.rex line 1:  Invalid subkeyword found.
Error 25.901:  Unknown keyword on ::CLASS directive; found "NAMESPACE".
--- rc=231

=== REJECT LIBRARY on ::CLASS
--- source (rexxc) ---
     1	::class c LIBRARY
--- output ---
     1 *-* ::class c LIBRARY
Error 25 running /tmp/tmp.mBc0x4yTqo/p.rex line 1:  Invalid subkeyword found.
Error 25.901:  Unknown keyword on ::CLASS directive; found "LIBRARY".
--- rc=231

```

## The mutation script

```python
#!/usr/bin/env python3
"""Apply one mutation to directive.rs or convert.rs, run the crate tests, revert.

Each mutation is a (name, file, old, new) tuple targeting one load-bearing
decision. A mutation that no test catches is a hole in the tests, and a mutation
whose pattern matches nothing was never applied at all -- which reads exactly
like a mutation that was caught. Both are failures, and the script exits
non-zero on either.
"""
import os
import subprocess
import sys

ROOT = "/home/moritz/dev/repos/ooRexx-rust-rewrite/rust"
DIR = os.path.join(ROOT, "crates/rexx-parse/src/directive.rs")
CONV = os.path.join(ROOT, "crates/rexx-parse/src/convert.rs")

MUTATIONS = [
    ("1 the two dispatch errors are swapped", DIR,
     "        let Some(token) = self.next_real() else {\n            return Err(self.error(20, 916));\n        };\n        let TokenKind::Symbol { id, .. } = token.kind else {\n            return Err(self.error(20, 916));\n        };",
     "        let Some(token) = self.next_real() else {\n            return Err(self.error(99, 916));\n        };\n        let TokenKind::Symbol { id, .. } = token.kind else {\n            return Err(self.error(99, 916));\n        };"),

    ("2 the token after :: resolves against subDirectives", DIR,
     "        let Some(index) = self.ctx.keywords.directives.index_of(id) else {",
     "        let Some(index) = self.ctx.keywords.sub_directives.index_of(id) else {"),

    ("3 an option resolves against directives", DIR,
     "            TokenKind::Symbol { id, .. } => self.ctx.keywords.sub_directives.index_of(*id),",
     "            TokenKind::Symbol { id, .. } => self.ctx.keywords.directives.index_of(*id),"),

    ("4 FORM and NUMERIC resolve against subDirectives", DIR,
     "            TokenKind::Symbol { id, .. } => self.ctx.keywords.sub_keywords.index_of(*id),",
     "            TokenKind::Symbol { id, .. } => self.ctx.keywords.sub_directives.index_of(*id),"),

    ("5 DIGITS accepts zero", DIR,
     "                        .filter(|&digits| digits >= 1)\n", ""),

    ("6 FUZZ rejects zero", DIR,
     "                    let fuzz = whole_number(&value, ARGUMENT_DIGITS)\n                        .and_then(|fuzz| usize::try_from(fuzz).ok())",
     "                    let fuzz = whole_number(&value, ARGUMENT_DIGITS)\n                        .filter(|&fuzz| fuzz >= 1)\n                        .and_then(|fuzz| usize::try_from(fuzz).ok())"),

    ("7 the options precision is the TRACE one", DIR,
     "const ARGUMENT_DIGITS: usize = 18;", "const ARGUMENT_DIGITS: usize = 9;"),

    ("8 every condition option accepts ERROR", DIR,
     "            Some(SUBDIR_ERROR) if which == ConditionOption::NoValue => true,",
     "            Some(SUBDIR_ERROR) => true,"),

    ("9 a body error is reported against the directive", DIR,
     "        Err(ParseError::new(code, sub, next.span.start))",
     "        Err(self.error(code, sub))"),

    ("10 hasBody is inverted", DIR,
     "            .is_some_and(|next| self.ctx.tokens[next.tokens.start].kind.tag() != Tag::DColon)",
     "            .is_some_and(|next| self.ctx.tokens[next.tokens.start].kind.tag() == Tag::DColon)"),

    ("11 an attribute pair may have a body", DIR,
     "            AttributeStyle::Both => {\n                self.check_directive(cursor, 99, 937)?;\n            }",
     "            AttributeStyle::Both => {}"),

    ("12 a literal class reference is not upcased", DIR,
     "            TokenKind::Literal { .. } => Ok(ClassRef {\n                namespace: None,\n                name: self.upper_value_of(token),\n            }),",
     "            TokenKind::Literal { .. } => Ok(ClassRef {\n                namespace: None,\n                name: self.value_of(token),\n            }),"),

    ("13 a method name is upcased", DIR,
     "        let name = self.require_name(19, 902)?;",
     "        let name = self.require_upper_name(19, 902)?;"),

    ("14 a routine name is upcased", DIR,
     "        let name = self.require_name(19, 903)?;",
     "        let name = self.require_upper_name(19, 903)?;"),

    ("15 SUBCLASS and MIXINCLASS fill separate slots", DIR,
     "                Some(SUBDIR_MIXINCLASS) if class.subclass.is_none() => {",
     "                Some(SUBDIR_MIXINCLASS) if !class.mixin => {"),

    ("16 a method may be EXTERNAL REGISTERED", DIR,
     "        method.external = self.decode_external(external.as_deref(), false)?;\n            self.check_directive(cursor, 99, 936)?;",
     "        method.external = self.decode_external(external.as_deref(), true)?;\n            self.check_directive(cursor, 99, 936)?;"),

    ("17 an external specification may name four words", DIR,
     "            [library, entry] => (*library, Some(Box::from(*entry))),\n            _ => return Err(bad),",
     "            [library, entry] => (*library, Some(Box::from(*entry))),\n            [library, entry, ..] => (*library, Some(Box::from(*entry))),\n            _ => return Err(bad),"),

    ("18 a signed constant need not be a number", DIR,
     "        if !is_number(&value) {\n            return Err(self.error(code, sub));\n        }\n", ""),

    ("19 a constant may be followed by more data", DIR,
     "        self.required_end(21, 913)?;", ""),

    ("20 a parenthesised constant need not be closed", DIR,
     "        match self.next_real() {\n            Some(token) if token.kind.tag() == Tag::RightParen => Ok(expr),\n            _ => Err(self.error(36, 901)),\n        }",
     "        self.next_real();\n        Ok(expr)"),

    ("21 REXX is not a reserved namespace", DIR,
     "                    if self.ctx.symbols.name(namespace) == \"REXX\" {\n                        return Err(self.error(99, 944));\n                    }\n", ""),

    ("22 LIBRARY and NAMESPACE are not mutually exclusive", DIR,
     "            let taken = requires.library || requires.namespace.is_some();",
     "            let taken = false;"),

    ("23 a resource END keyword needs no marker", DIR,
     "            end_marker = self.require_name(19, 921)?;",
     "            if !self.at_end() {\n                end_marker = self.require_name(19, 921)?;\n            }"),

    ("24 a resource accepts any sub-directive", DIR,
     "            if self.sub_directive(token) != Some(SUBDIR_END) {\n                return Err(self.error(25, 926));\n            }",
     "            if self.sub_directive(token).is_none() {\n                return Err(self.error(25, 926));\n            }"),

    ("25 an annotation target is not upcased", DIR,
     "            Some(SUBDIR_METHOD) => AnnotationTarget::Method(self.require_upper_name(19, 925)?),",
     "            Some(SUBDIR_METHOD) => AnnotationTarget::Method(self.require_name(19, 925)?),"),

    ("26 a second ROUTINE external reports the ROUTINE number", DIR,
     "                    if external.is_some() {\n                        return Err(self.error(25, 901));\n                    }",
     "                    if external.is_some() {\n                        return Err(self.error(25, 903));\n                    }"),

    ("27 a missing annotation value uses the bad-value number", DIR,
     "            None => return Err(self.error(19, 924)),",
     "            None => return Err(self.error(19, 923)),"),

    ("28 a directive's span is its tokens rather than its clause", DIR,
     "        clause_span: parser.clause.span.clone(),",
     "        clause_span: parser.clause_byte..parser.clause_byte + 1,"),

    ("29 a resource takes the first body rather than its own", DIR,
     "            .find(|body| body.directive == self.clause.tokens.start)",
     "            .find(|_| true)"),

    ("30 a number's surrounding blanks are not stripped", CONV,
     "    let mut rest = text;\n    while let Some((&byte, tail)) = rest.split_first()\n        && (byte == b' ' || byte == b'\\t')\n    {\n        rest = tail;\n    }\n    while let Some((&byte, head)) = rest.split_last()\n        && (byte == b' ' || byte == b'\\t')\n    {\n        rest = head;\n    }\n",
     "    let mut rest = text;\n"),

    ("31 is_number goes through whole_number", CONV,
     "    scan_number(text).is_some()",
     "    whole_number(text, 18).is_some()"),

    ("32 an unknown TRACE letter is accepted", CONV,
     "            b'A' | b'C' | b'L' | b'E' | b'F' | b'N' | b'O' | b'R' | b'I' => Ok(()),\n            _ => Err(()),",
     "            _ => Ok(()),"),
]


def run():
    r = subprocess.run(
        ["cargo", "test", "--offline", "-p", "rexx-parse", "--lib"],
        cwd=ROOT, capture_output=True, text=True)
    out = r.stdout + r.stderr
    for line in out.splitlines():
        if line.startswith("test result:"):
            return line
    if "error[" in out or "error:" in out:
        return "DID NOT COMPILE"
    return "NO RESULT"


failures = []
for name, path, old, new in MUTATIONS:
    original = open(path).read()
    count = original.count(old)
    if count != 1:
        print("%-64s NOT APPLIED: pattern found %d times" % (name, count))
        failures.append(name)
        continue
    open(path, "w").write(original.replace(old, new))
    try:
        result = run()
    finally:
        open(path, "w").write(original)
    caught = result == "DID NOT COMPILE" or result.startswith("test result: FAILED")
    print("%-64s %s" % (name, result))
    if not caught:
        failures.append(name)

print()
print("%d mutations, %d survived or were never applied" % (len(MUTATIONS), len(failures)))
for name in failures:
    print("  SURVIVED/UNAPPLIED: %s" % name)
sys.exit(1 if failures else 0)
```

### First run: the script's own verdict was wrong

`caught = A or B and C` parses as `A or (B and C)`, and `B` tested for
`"failed. "` where the output says `"FAILED. "`. Every mutation was reported as a
survivor. The run is kept because it is the failure mode the brief warns about,
seen from the other side: the script cried wolf rather than staying silent, which
is the safe direction for that bug to fall.

```text
(the first run's per-mutation lines are the same 32 names; every one printed 'test result: FAILED. ...' yet was listed as SURVIVED/UNAPPLIED, and the script exited 1)
```

### Second run, after the one-line fix

```text
1 the two dispatch errors are swapped                            test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
2 the token after :: resolves against subDirectives              test result: FAILED. 182 passed; 18 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
3 an option resolves against directives                          test result: FAILED. 183 passed; 17 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
4 FORM and NUMERIC resolve against subDirectives                 test result: FAILED. 198 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
5 DIGITS accepts zero                                            test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
6 FUZZ rejects zero                                              test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
7 the options precision is the TRACE one                         test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
8 every condition option accepts ERROR                           test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s
9 a body error is reported against the directive                 test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
10 hasBody is inverted                                           test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
11 an attribute pair may have a body                             test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
12 a literal class reference is not upcased                      test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
13 a method name is upcased                                      test result: FAILED. 198 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
14 a routine name is upcased                                     test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s
15 SUBCLASS and MIXINCLASS fill separate slots                   test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
16 a method may be EXTERNAL REGISTERED                           test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
17 an external specification may name four words                 test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
18 a signed constant need not be a number                        test result: FAILED. 198 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
19 a constant may be followed by more data                       test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
20 a parenthesised constant need not be closed                   test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
21 REXX is not a reserved namespace                              test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
22 LIBRARY and NAMESPACE are not mutually exclusive              test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s
23 a resource END keyword needs no marker                        test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.09s
24 a resource accepts any sub-directive                          test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
25 an annotation target is not upcased                           test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
26 a second ROUTINE external reports the ROUTINE number          test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
27 a missing annotation value uses the bad-value number          test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
28 a directive's span is its tokens rather than its clause       test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
29 a resource takes the first body rather than its own           test result: FAILED. 199 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
30 a number's surrounding blanks are not stripped                test result: FAILED. 197 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
31 is_number goes through whole_number                           test result: FAILED. 197 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
32 an unknown TRACE letter is accepted                           test result: FAILED. 197 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s

32 mutations, 0 survived or were never applied
```

---

# Round 2: closing the review

**Status: all findings addressed.** Three further commits, `cd12a305`,
`f37b4a86`, `c0e42d39`. 204 crate tests and 437 workspace tests. 47 mutations,
all applied and all caught. `fmt --check`, clippy `-D warnings`, both suites and
the gate grep all pass, each checked on its own line.

## The number-acceptance question, answered

**`rexx-parse` should not have its own number-acceptance rule, and no longer
does.** `is_number` and `whole_number`'s first step both call
`rexx_num::Number::parse`, and `rexx-num` moved from a dev-dependency to a real
one.

Three reasons, in the order I weigh them:

1. **It mirrors the C++'s own layering.** `LanguageParser` and `DirectiveParser`
   never implement number syntax. They call `RexxString::numberString()`, which
   is the numeric runtime's. The dependency edge is the same edge the interpreter
   has, so this is not a shortcut, it is the faithful structure.
2. **One rule cannot drift from itself.** A second port plus a drift test still
   has two rules; the test only tells you *afterwards* that they parted. The
   review is right that this defect class was already solved once, and a drift
   test would not have prevented the reintroduction, only dated it.
3. **`Number::parse` is verified across 128,368 differential cases**, including
   the 2,320-case `signblank` set that exists for exactly this defect. Nothing I
   write in this crate will be tested that hard.

The delegation fixed **I2** for free and without my writing an exponent rule:
`Number::parse` already carries both limits, and both are independently
observable. Measured, with the review's controls:

```
=== I2 constant -1e999999999      rc=0
=== I2 constant -1e1000000000     Error 19.916    (limit on the exponent AS WRITTEN)
=== I2 constant -9e999999999      rc=0
=== I2 constant -99e999999999     Error 19.916    (limit on the ADJUSTED exponent)
```

### What is still local, and why

`whole_number`'s second half is `NumberString::numberValue`
(`NumberStringClass.cpp:588`): round an accepted number to a precision, then ask
whether the result is an integer that fits. That is a conversion, not a syntax,
and it exists nowhere else in the workspace. It needs the mantissa and exponent,
and `Number` keeps both `pub(crate)`, so `decompose` re-walks the text to recover
them.

`decompose` is a **valuation**, not a second acceptance rule: it runs only on
text `Number::parse` accepted. `the_local_walk_never_fails_where_rexx_num_accepted`
pins that, over `SIGNBLANK_A` from `rexx-num/tests/gen-curated-sets.py:151` plus
every shape this crate's callers reach. The invariant is one-directional on
purpose, and one shape shows why: `decompose` accepts `-1e1000000000` because it
is syntactically a number, while `Number::parse` rejects it for range. Requiring
equality there would push the exponent limits back into this crate, which is the
whole thing the arrangement avoids. **Over the signblank shapes alone the two
agree in both directions**, and that is asserted separately, because no shape
there is out of range so any disagreement would be a real divergence in the blank
rule.

**By rights `numberValue` belongs in `rexx-num`** beside `Number::parse`, whose
`round_to` it partly duplicates. It is in `convert.rs` only because this task was
scoped not to touch that crate. **This is the one thing I would like a decision
on.** `Number::format` is *not* a usable substitute, and measurably so:
`::options digits "1e18"` is Error 26.5 at nineteen digits, while Rexx's display
rule puts an adjusted exponent equal to `DIGITS` in plain notation, so a
`format`-based check would accept it.

## C1, and a second divergence it uncovered

The sign-blank rule is fixed, and the test that was supposed to cover it is
replaced. The review's diagnosis is exactly right and worth restating in the
terms this project uses for it: `::options digits "- 9"` **really is** 26.5, so
the probe agreed with the implementation, but for a different reason than the
implementation had. `-9 < 1` fails the range check. `"+ 9"` is rc 0 and is the
only input that separates the two.

Chasing it turned up a **second divergence the review had not reached**, and a
larger one: `requestNumber` **rounds to the precision before asking whether the
result is an integer**, so a fraction can survive the conversion. Every case
below was rejected by the old implementation and is rc 0 in the oracle:

```
=== ROUND trace 999999999.4        rc=0     ten digits truncate to nine, 4 does not carry
=== ROUND trace "1.0000000001"     rc=0     eleven truncate to nine, surviving decimals all zero
=== ROUND trace "0.9999999999"     rc=0     the dropped digit carries, all-nine decimals give 1
```

and the two controls that bound it:

```
=== ROUND trace "999999999.6"      Error 24.1   that carry makes the value ten digits wide
=== ROUND trace "99999999.6"       Error 24.1   nine digits do not exceed the precision, so
                                               nothing rounds and the 6 is simply not whole
```

`NumberString::numberValue`, `checkIntegerDigits` and `createUnsignedValue` are
now ported in full, including one quirk reproduced rather than corrected: the
carry-only return path is `carry ? 1 : 0` with **no** `* numberSign`, so a
negative pure fraction that rounds up converts to +1. I could not find an
observable case for the sign and have reproduced the code path as written rather
than guessing.

This reaches `TRACE`, so 3.6's suite was re-run and **extended**:
`the_trace_number_gate_is_exactly_the_oracles` now covers the sign blank, all
five rounding cases and the `1e8`/`1e9` width boundary.

## I1: error 99.925 at four `getRetriever` sites

Implemented, both directions, with the controls. The review's point about *why* it
was missed is the useful part and is now recorded in the code: the `syntaxError`
lives in `LanguageParser.cpp:2507`, outside the 2,867 lines of
`DirectiveParser.cpp` the task was scoped to. **Any task scoped to one file has
the same exposure**, and a brief that names a file should probably also name the
functions that file calls out to.

The placement differs per shape and is observable, which one test would have
missed:

| shape | name check | measured |
| --- | --- | --- |
| `::ATTRIBUTE` name | before everything | `::attribute 3` + body is 99.925 on line 1, not 99.937 on line 2 |
| `::METHOD DELEGATE` | before the body check | `::method m delegate 5` + body is 99.925 on line 1, not 99.946 |
| `::METHOD ATTRIBUTE` | after the body check | `::method 3 attribute` + body is 99.934 on line 2 |
| `::ATTRIBUTE` BOTH delegate | after the body check | `::attribute a delegate 5` + body is 99.937 on line 2 |
| `::ATTRIBUTE` GET/SET delegate | before the body check | `::attribute a get delegate 5` + body is 99.925 on line 1 |

And `::METHOD ATTRIBUTE` only checks in the sub-branch that generates the accessor
pair: `::method 3 attribute abstract` is rc 0 and
`::method 3 attribute external "LIBRARY x"` reaches 98.903, both past the check.

`scanSymbol`'s exponent wrinkle is reproduced: `::attribute "a.e+5"` is rc 0 while
`"a-b"` and `"1e+5"` are 99.925. `MAX_SYMBOL_LENGTH` and `is_symbol_char` became
crate-visible rather than restated, so the 250-byte bound has one definition.

## I3 and the seven Minors

* **I3** `the_other_shipped_packages_parse` now asserts `StreamClasses.orx`'s
  decomposition, 7 `::CLASS` / 139 `::METHOD` / 5 `::ATTRIBUTE` / 2 `::CONSTANT`,
  through a named `Counts` struct rather than a tuple, because clippy's
  `type_complexity` rejects the tuple.
* `is_number` is new code, not moved code, and the round-1 commit message called
  it moved. It is now genuinely different code anyway: a call to `Number::parse`.
* The structuring semicolon at `ast.rs:1046`, split.
* The 33.1 note carries both directions. Measured: `::options fuzz 5 digits 3` is
  33.1 on the DIGITS and `::options digits 3 fuzz 5` is 33.1 on the FUZZ.
* `ExternalSpec::entry`'s `None` is documented as "no third word" and the three
  defaults it is *not* are spelled out with their line numbers, because a
  routine, a method and an attribute each default differently.
* `resource()`'s `.expect()` invariant now states the scanner-side conditions
  inline and maps each to the check above it that enforces it, so the invariant
  reads in one place.

## Both hedges retired

The review is right that both resolve from the C++ and both resolve in my favour.
The hedging comments are gone and the facts are stated with citations:

* `decode_external` splits on space and tab and nothing else.
  `words` goes through `subWords`, which drives `RexxString::WordIterator`, whose
  `skipBlanks` and `skipNonBlanks` both test `*scan != ' ' && *scan != '\t'`
  (`StringClass.hpp:155` and `:178`).
* `upper_value_of` is ASCII-only and exactly right. `RexxString::upper` upcases
  through `Utilities::toUpper`, which is `isLower(c) ? c & ~0x20 : c` with
  `isLower` spelled `c >= 'a' && c <= 'z'` (`common/Utilities.hpp:52`), so a
  non-ASCII byte is left alone by the interpreter too.

## Mutation testing, 47 mutations

Fifteen new ones cover the rounding conversion, the delegation, the name check
and the five placements. All 47 applied, all 47 caught, script exits 0. The
unapplied-pattern guard was re-confirmed the same way the reviewer did it: a
deliberately bogus pattern printed `NOT APPLIED: pattern found 0 times` and the
script exited 1.

**Two candidate mutations turned out to be equivalent mutants and were replaced,
which is worth recording rather than hiding.** Removing `number(text)?` from
`whole_number` survived, and it survived because it is not a behaviour change:
every text with an out-of-range exponent is also too wide for any precision, so
`unsigned_value`'s width check already rejects it. Likewise widening
`unsigned_value`'s pre-check to `i64::MAX` survived, because the `> max` check
below subsumes it. Both were replaced with mutations that do change behaviour: an
`is_number` routed through `decompose`, and a carry-only return of `0` instead of
`carry ? 1 : 0`.

## Round-2 probe log

Fresh `mktemp -d`, whole messages, only the constant banner removed. 64 probes.

```text
=== C1 trace "+ 9"
--- source ---
     1	trace "+ 9"
     2	nop
--- rexxc ---
--- rc=0

=== C1 trace "- 9"
--- source ---
     1	trace "- 9"
     2	nop
--- rexxc ---
--- rc=0

=== C1 digits "+ 9"
--- source ---
     1	::options digits "+ 9"
--- rexxc ---
--- rc=0

=== C1 digits "- 9" fails the RANGE check not the blank rule
--- source ---
     1	::options digits "- 9"
--- rexxc ---
     1 *-* ::options digits "- 9"
Error 26 running /tmp/tmp.4zzsJRmtV4/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "- 9".
--- rc=230

=== C1 fuzz "- 9" unsigned rejects negative
--- source ---
     1	::options fuzz "- 9"
--- rexxc ---
     1 *-* ::options fuzz "- 9"
Error 26 running /tmp/tmp.IKuJpmwN3N/p.rex line 1:  Invalid whole number.
Error 26.6:  FUZZ value must be zero or a positive whole number; found "- 9".
--- rc=230

=== C1 digits "+  9"
--- source ---
     1	::options digits "+  9"
--- rexxc ---
--- rc=0

=== C1 digits "+<tab>9"
--- source ---
     1	::options digits "+	9"
--- rexxc ---
--- rc=0

=== C1 trace "+ - 9"
--- source ---
     1	trace '+ - 9'
     2	nop
--- rexxc ---
     1 *-* trace '+ - 9'
Error 24 running /tmp/tmp.sQ9A4OA23W/p.rex line 1:  Invalid TRACE request.
Error 24.1:  TRACE request letter must be one of "ACEFILNOR"; found "+".
--- rc=232

=== ROUND trace 999999999.4
--- source ---
     1	trace 999999999.4
     2	nop
--- rexxc ---
--- rc=0

=== ROUND trace "1.0000000001"
--- source ---
     1	trace "1.0000000001"
     2	nop
--- rexxc ---
--- rc=0

=== ROUND trace "0.9999999999"
--- source ---
     1	trace "0.9999999999"
     2	nop
--- rexxc ---
--- rc=0

=== ROUND trace "999999999.6"
--- source ---
     1	trace "999999999.6"
     2	nop
--- rexxc ---
     1 *-* trace "999999999.6"
Error 24 running /tmp/tmp.wHYudh4foz/p.rex line 1:  Invalid TRACE request.
Error 24.1:  TRACE request letter must be one of "ACEFILNOR"; found "9".
--- rc=232

=== ROUND trace "99999999.6"
--- source ---
     1	trace "99999999.6"
     2	nop
--- rexxc ---
     1 *-* trace "99999999.6"
Error 24 running /tmp/tmp.0P05dexzsF/p.rex line 1:  Invalid TRACE request.
Error 24.1:  TRACE request letter must be one of "ACEFILNOR"; found "9".
--- rc=232

=== ROUND trace 1.4 control
--- source ---
     1	trace 1.4
     2	nop
--- rexxc ---
     1 *-* trace 1.4
Error 24 running /tmp/tmp.IIo6ejE66P/p.rex line 1:  Invalid TRACE request.
Error 24.1:  TRACE request letter must be one of "ACEFILNOR"; found "1".
--- rc=232

=== ROUND trace 1e8
--- source ---
     1	trace 1e8
     2	nop
--- rexxc ---
--- rc=0

=== ROUND trace 1e9
--- source ---
     1	trace 1e9
     2	nop
--- rexxc ---
     1 *-* trace 1e9
Error 24 running /tmp/tmp.8ejy1t69M4/p.rex line 1:  Invalid TRACE request.
Error 24.1:  TRACE request letter must be one of "ACEFILNOR"; found "1".
--- rc=232

=== ROUND digits 18-nines .4
--- source ---
     1	::options digits "999999999999999999.4"
--- rexxc ---
--- rc=0

=== ROUND digits 20 sig digits
--- source ---
     1	::options digits "1.0000000000000000001"
--- rexxc ---
--- rc=0

=== ROUND digits "0.9999999999"
--- source ---
     1	::options digits "0.9999999999"
--- rexxc ---
     1 *-* ::options digits "0.9999999999"
Error 26 running /tmp/tmp.NrZDkDnMiw/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "0.9999999999".
--- rc=230

=== ROUND digits "0.4"
--- source ---
     1	::options digits "0.4"
--- rexxc ---
     1 *-* ::options digits "0.4"
Error 26 running /tmp/tmp.w95YgiTNxT/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "0.4".
--- rc=230

=== ROUND digits 1e17
--- source ---
     1	::options digits "1e17"
--- rexxc ---
--- rc=0

=== ROUND digits 1e18
--- source ---
     1	::options digits "1e18"
--- rexxc ---
     1 *-* ::options digits "1e18"
Error 26 running /tmp/tmp.uMj3P1O2xp/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "1e18".
--- rc=230

=== ROUND digits 19 written out
--- source ---
     1	::options digits "1000000000000000000"
--- rexxc ---
     1 *-* ::options digits "1000000000000000000"
Error 26 running /tmp/tmp.9GhhWJzaXe/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "1000000000000000000".
--- rc=230

=== I2 constant -1e999999999
--- source ---
     1	::constant c -1e999999999
--- rexxc ---
--- rc=0

=== I2 constant -1e1000000000
--- source ---
     1	::constant c -1e1000000000
--- rexxc ---
     1 *-* ::constant c -1e1000000000
Error 19 running /tmp/tmp.NYe6YTNXwQ/p.rex line 1:  String or symbol expected.
Error 19.916:  String or symbol expected as ::CONSTANT value.
--- rc=237

=== I2 constant -1e-999999999
--- source ---
     1	::constant c -1e-999999999
--- rexxc ---
--- rc=0

=== I2 constant -1e-1000000000
--- source ---
     1	::constant c -1e-1000000000
--- rexxc ---
     1 *-* ::constant c -1e-1000000000
Error 19 running /tmp/tmp.X7p4YGclQl/p.rex line 1:  String or symbol expected.
Error 19.916:  String or symbol expected as ::CONSTANT value.
--- rc=237

=== I2 constant -9e999999999
--- source ---
     1	::constant c -9e999999999
--- rexxc ---
--- rc=0

=== I2 constant -99e999999999
--- source ---
     1	::constant c -99e999999999
--- rexxc ---
     1 *-* ::constant c -99e999999999
Error 19 running /tmp/tmp.Utw7LZwxow/p.rex line 1:  String or symbol expected.
Error 19.916:  String or symbol expected as ::CONSTANT value.
--- rc=237

=== I2 digits 1e1000000000
--- source ---
     1	::options digits "1e1000000000"
--- rexxc ---
     1 *-* ::options digits "1e1000000000"
Error 26 running /tmp/tmp.ofuKf9Gkxx/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "1e1000000000".
--- rc=230

=== I2 digits 1e999999999
--- source ---
     1	::options digits "1e999999999"
--- rexxc ---
     1 *-* ::options digits "1e999999999"
Error 26 running /tmp/tmp.M2n2VAYkrR/p.rex line 1:  Invalid whole number.
Error 26.5:  DIGITS value must be a positive whole number; found "1e999999999".
--- rc=230

=== I1 ::attribute 3
--- source ---
     1	::attribute 3
--- rexxc ---
     1 *-* ::attribute 3
Error 99 running /tmp/tmp.IZ1BL3gYeQ/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "3".
--- rc=157

=== I1 ::attribute .a
--- source ---
     1	::attribute .a
--- rexxc ---
     1 *-* ::attribute .a
Error 99 running /tmp/tmp.0lFwskQR3s/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found ".A".
--- rc=157

=== I1 ::method m delegate 5
--- source ---
     1	::method m delegate 5
--- rexxc ---
     1 *-* ::method m delegate 5
Error 99 running /tmp/tmp.2yA1k4vHdA/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "5".
--- rc=157

=== I1 ::method m delegate .p
--- source ---
     1	::method m delegate .p
--- rexxc ---
     1 *-* ::method m delegate .p
Error 99 running /tmp/tmp.o945w589uZ/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found ".P".
--- rc=157

=== I1 control ::attribute a. stem
--- source ---
     1	::attribute a.
--- rexxc ---
--- rc=0

=== I1 control ::attribute a.b compound
--- source ---
     1	::attribute a.b
--- rexxc ---
--- rc=0

=== I1 control ::method 3
--- source ---
     1	::method 3
--- rexxc ---
--- rc=0

=== I1 control ::method .a
--- source ---
     1	::method .a
     2	  return 1
--- rexxc ---
--- rc=0

=== I1 ::attribute "3" literal
--- source ---
     1	::attribute "3"
--- rexxc ---
     1 *-* ::attribute "3"
Error 99 running /tmp/tmp.5xh1f9y9kT/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "3".
--- rc=157

=== I1 ::attribute "a b"
--- source ---
     1	::attribute "a b"
--- rexxc ---
     1 *-* ::attribute "a b"
Error 99 running /tmp/tmp.zkpuHgcAxQ/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "a b".
--- rc=157

=== I1 ::attribute "a-b"
--- source ---
     1	::attribute "a-b"
--- rexxc ---
     1 *-* ::attribute "a-b"
Error 99 running /tmp/tmp.Z2TR9YV2CQ/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "a-b".
--- rc=157

=== I1 ::attribute "1e+5"
--- source ---
     1	::attribute "1e+5"
--- rexxc ---
     1 *-* ::attribute "1e+5"
Error 99 running /tmp/tmp.jnRvVVs2gy/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "1e+5".
--- rc=157

=== I1 ::attribute "a.e+5" quirk accepted
--- source ---
     1	::attribute "a.e+5"
--- rexxc ---
--- rc=0

=== I1 ::attribute "aB" mixed case
--- source ---
     1	::attribute "aB"
--- rexxc ---
--- rc=0

=== I1 ::attribute 3 get
--- source ---
     1	::attribute 3 get
--- rexxc ---
     1 *-* ::attribute 3 get
Error 99 running /tmp/tmp.3EvsuEIXfz/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "3".
--- rc=157

=== I1 ::attribute 3 abstract
--- source ---
     1	::attribute 3 abstract
--- rexxc ---
     1 *-* ::attribute 3 abstract
Error 99 running /tmp/tmp.MLlq3xWkNM/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "3".
--- rc=157

=== I1 ::method 3 attribute
--- source ---
     1	::method 3 attribute
--- rexxc ---
     1 *-* ::method 3 attribute
Error 99 running /tmp/tmp.vhEQnndmpl/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "3".
--- rc=157

=== I1 ::method .a attribute
--- source ---
     1	::method .a attribute
--- rexxc ---
     1 *-* ::method .a attribute
Error 99 running /tmp/tmp.firiv1ABqZ/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found ".A".
--- rc=157

=== I1 control ::method 3 attribute abstract
--- source ---
     1	::method 3 attribute abstract
--- rexxc ---
--- rc=0

=== I1 control ::method 3 attribute external
--- source ---
     1	::method 3 attribute external "LIBRARY x"
--- rexxc ---
     1 *-* ::method 3 attribute external "LIBRARY x"
Error 98 running /tmp/tmp.ZvAuESjzlq/p.rex line 1:  Execution error.
Error 98.903:  Unable to load library "x".
--- rc=158

=== I1 ::attribute a delegate 5
--- source ---
     1	::attribute a delegate 5
--- rexxc ---
     1 *-* ::attribute a delegate 5
Error 99 running /tmp/tmp.RLwmMMnnmO/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "5".
--- rc=157

=== I1 ::attribute a get delegate 5
--- source ---
     1	::attribute a get delegate 5
--- rexxc ---
     1 *-* ::attribute a get delegate 5
Error 99 running /tmp/tmp.jBwsMhUJiH/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "5".
--- rc=157

=== I1 control ::attribute a delegate p.
--- source ---
     1	::attribute a delegate p.
--- rexxc ---
--- rc=0

=== I1 order ::attribute 3 + body
--- source ---
     1	::attribute 3
     2	  return 1
--- rexxc ---
     1 *-* ::attribute 3
Error 99 running /tmp/tmp.00yFXAhYgp/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "3".
--- rc=157

=== I1 order ::attribute 3 external
--- source ---
     1	::attribute 3 external "LIBRARY x"
--- rexxc ---
     1 *-* ::attribute 3 external "LIBRARY x"
Error 99 running /tmp/tmp.Ro6PKNCtTT/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "3".
--- rc=157

=== I1 order ::method m delegate 5 + body
--- source ---
     1	::method m delegate 5
     2	  return 1
--- rexxc ---
     1 *-* ::method m delegate 5
Error 99 running /tmp/tmp.a7VQz5bIba/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "5".
--- rc=157

=== I1 order ::method 3 attribute + body
--- source ---
     1	::method 3 attribute
     2	  return 1
--- rexxc ---
     2 *-* return 1
Error 99 running /tmp/tmp.xTTFnj530k/p.rex line 2:  Translation error.
Error 99.934:  Attribute methods cannot have a method body.
--- rc=157

=== I1 order ::attribute a delegate 5 + body
--- source ---
     1	::attribute a delegate 5
     2	  return 1
--- rexxc ---
     2 *-* return 1
Error 99 running /tmp/tmp.IN02C46hls/p.rex line 2:  Translation error.
Error 99.937:  Attribute methods without a SET or GET designation cannot have a method body.
--- rc=157

=== I1 order ::attribute a get delegate 5 + body
--- source ---
     1	::attribute a get delegate 5
     2	  return 1
--- rexxc ---
     1 *-* ::attribute a get delegate 5
Error 99 running /tmp/tmp.LryOn5jTSM/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "5".
--- rc=157

=== MINOR 33.1 the DIGITS direction
--- source ---
     1	::options fuzz 5 digits 3
--- rexxc ---
     1 *-* ::options fuzz 5 digits 3
Error 33 running /tmp/tmp.TCECUksMpW/p.rex line 1:  Invalid expression result.
Error 33.1:  Value of NUMERIC DIGITS ("3") must exceed value of NUMERIC FUZZ ("5").
--- rc=223

=== MINOR 33.1 the FUZZ direction
--- source ---
     1	::options digits 3 fuzz 5
--- rexxc ---
     1 *-* ::options digits 3 fuzz 5
Error 33 running /tmp/tmp.LlqBXWReS5/p.rex line 1:  Invalid expression result.
Error 33.1:  Value of NUMERIC DIGITS ("3") must exceed value of NUMERIC FUZZ ("5").
--- rc=223

=== I1 250-byte literal name
--- source ---
     1	::attribute "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
--- rexxc ---
--- rc=0

=== I1 251-byte literal name
--- source ---
     1	::attribute "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
--- rexxc ---
     1 *-* ::attribute "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
Error 99 running /tmp/tmp.a7O8foER3n/p.rex line 1:  Translation error.
Error 99.925:  An ATTRIBUTE method name must be a valid variable name; found "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".
--- rc=157

```

## Round-2 mutation log

```text
1 the two dispatch errors are swapped                            test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
2 the token after :: resolves against subDirectives              test result: FAILED. 184 passed; 20 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
3 an option resolves against directives                          test result: FAILED. 185 passed; 19 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
4 FORM and NUMERIC resolve against subDirectives                 test result: FAILED. 202 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
5 DIGITS accepts zero                                            test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
6 FUZZ rejects zero                                              test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
7 the options precision is the TRACE one                         test result: FAILED. 201 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
8 every condition option accepts ERROR                           test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
9 a body error is reported against the directive                 test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
10 hasBody is inverted                                           test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
11 an attribute pair may have a body                             test result: FAILED. 202 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
12 a literal class reference is not upcased                      test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
13 a method name is upcased                                      test result: FAILED. 202 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
14 a routine name is upcased                                     test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
15 SUBCLASS and MIXINCLASS fill separate slots                   test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
16 a method may be EXTERNAL REGISTERED                           test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
17 an external specification may name four words                 test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
18 a signed constant need not be a number                        test result: FAILED. 202 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
19 a constant may be followed by more data                       test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
20 a parenthesised constant need not be closed                   test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
21 REXX is not a reserved namespace                              test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
22 LIBRARY and NAMESPACE are not mutually exclusive              test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
23 a resource END keyword needs no marker                        test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
24 a resource accepts any sub-directive                          test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
25 an annotation target is not upcased                           test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
26 a second ROUTINE external reports the ROUTINE number          test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
27 a missing annotation value uses the bad-value number          test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
28 a directive's span is its tokens rather than its clause       test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
29 a resource takes the first body rather than its own           test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
30 is_number's acceptance is this crate's own walk               test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
31 is_number goes through whole_number                           test result: FAILED. 200 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
33 the blanks after a sign are not skipped                       test result: FAILED. 201 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
34 a number's surrounding blanks are not stripped                test result: FAILED. 200 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
35 the conversion never rounds to the precision                  test result: FAILED. 202 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
36 a rounded fraction need not be all zeros                      test result: FAILED. 198 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
37 the carry compares against zero rather than nine              test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
38 the value width is not checked against the precision          test result: FAILED. 201 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
39 a carry that rounds a pure fraction up yields zero            test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
40 an attribute name may be any symbol                           test result: FAILED. 202 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
41 an attribute name may hold any character                      test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
42 an attribute name has no length bound                         test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
43 the ::ATTRIBUTE name is not checked at all                    test result: FAILED. 202 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
44 a DELEGATE target is not checked                              test result: FAILED. 202 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
45 the ::METHOD ATTRIBUTE name check ignores the sub-branch      test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
32 an unknown TRACE letter is accepted                           test result: FAILED. 201 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
46 the ::ATTRIBUTE BOTH delegate check runs before the body check test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
47 the ::METHOD DELEGATE check runs after the body check         test result: FAILED. 203 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s

47 mutations, 0 survived or were never applied
```

---

# Round 3: the `numberValue` move

**Status: done.** One commit, `05d40f1d`, on its own so it can be reverted alone.

## The twelve differential sets, 0 divergences

Run twice: once before touching `rexx-num`, to prove the harness reproduces
Phase 2's number rather than merely printing zeros, and once after the move.
Identical both times.

```text
set            cases  diverge
addsub          8712        0
addsub2         8112        0
muldiv         17424        0
md2            20184        0
pow             2112        0
cmp            32368        0
signblank       2320        0
fmt             1800        0
fmt2            6720        0
fmt3           12136        0
fmtedge          640        0
fmtcarry       15840        0
TOTAL         128368        0
```

`128368` is exact, not rounded: the twelve set sizes sum to it. Each set is
regenerated from `rust/crates/rexx-num/tests/gen-curated-sets.py`, answered by
`build/bin/rexx` through `data-addsub-oracle.rex` or `data-format-oracle.rex`,
answered again by `cargo run --release -p rexx-num --bin muldiv|fmt-check`, and
the two outputs diffed. The runner is kept at `scratchpad/t37/run-sets.sh`.

`rexx-num`'s own suites are green, and so are the workspace's.

## What moved, and what the move deleted

`Number::whole_value(&self, digits: usize) -> Option<i64>` is
`NumberString::numberValue`. The public API grew by that method and by
`ARGUMENT_DIGITS`, and by nothing else: **the mantissa and exponent stayed
`pub(crate)`**, per your third condition, because the method returns the `i64` the
caller wants rather than the fields it computes from.

**The move deleted `decompose`, and with it the one-directional invariant you
asked me to keep.** I did not keep it, and I think that is right rather than a
liberty: `decompose` existed *only* to re-walk the text for a mantissa and an
exponent that `Number` already holds. Once `numberValue` is a method on `Number`
there is nothing left to walk. The asymmetry was a mitigation for a duplication
that no longer exists, and carrying it would have meant keeping the duplication to
justify the mitigation. `convert.rs` now holds no number syntax and no number
arithmetic at all: `is_number` is one call, `whole_number` is two, and the only
local rule left is the `TRACE` setting alphabet.

If you would rather have the invariant back, it costs restoring `decompose`, and I
would argue against it.

## Two measurements the carry rule was owed

Writing `tests/whole.rs` caught me asserting the carry rule **from reasoning
rather than from the oracle**, and the assertion was wrong: I claimed
`whole("0.99999999989", 9)` is `None`. It is `Some(1)`. Measured:

```
=== trace "0.99999999989"                  rc=0
=== trace "0.99999999999"                  rc=0
=== trace "0.99999999899" ninth digit is 8  Error 24.1
=== trace "0.4999999999" no carry           Error 24.1
```

The rule is that `checkIntegerDigits` compares the **nine kept** digits against
nine, not the dropped ones, so the tenth digit only decides whether there is a
carry at all. My reasoning had conflated the two. The test now carries all four
rows and the reason. That is the seventh instance of this project's recurring
probe error and the first where the code was right and the assertion was wrong.

## The carry sign is resolved, and I am calling it a suspected upstream defect

Your measurement settles it: `numberValue`'s carry-only return is
`carry ? 1 : 0` with no `* numberSign`, and it cannot surface, because a numeric
`TRACE` is rejected at **run** time with error 24.901, *"Numeric TRACE requests
are valid only from interactive debugging"*, whatever the parse produced. The code
comment and `the_carry_only_path_drops_the_sign_as_the_cpp_does` now say
*unobservable because numeric TRACE is runtime-rejected outside interactive
debugging*, not *one attempt failed*.

**Stated plainly: I believe this is an upstream defect, not intended behaviour.**
Every other return path in `numberValue` multiplies by `numberSign`, this one does
not, and there is no comment marking it deliberate. It is latent rather than
live — `requestNumber`'s other callers would have to reach a value whose entire
mantissa rounds away, and `TRACE` is the only one this project has traced there.
Worth filing, low severity. Say the word and I will write it up.

## Mutation testing follows the code

47 mutations, all applied, all caught, script exits 0. Seven now target
`rexx-num/src/lib.rs`, because that is where the code they were written for went;
leaving them pointed at `convert.rs` would have turned all seven into
never-applied patterns, which is the failure mode the guard exists for.

The runner itself needed a fix that is worth recording. It ran one crate and
returned the **first** `test result:` line; with two crates a mutation caught only
in the second would have read as a survivor. It now scans every line and treats
any failing suite as a catch.

Four patterns went stale in the move and printed `NOT APPLIED: pattern found 0
times` before I corrected them — the guard working exactly as intended, twice in
one task.

## Round-3 mutation log

```text
1 the two dispatch errors are swapped                            test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
2 the token after :: resolves against subDirectives              test result: FAILED (1 suite(s))  176 passed; 20 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
3 an option resolves against directives                          test result: FAILED (1 suite(s))  177 passed; 19 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
4 FORM and NUMERIC resolve against subDirectives                 test result: FAILED (1 suite(s))  194 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
5 DIGITS accepts zero                                            test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
6 FUZZ rejects zero                                              test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
7 the argument precision is the TRACE one                        test result: FAILED (1 suite(s))  6 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
8 every condition option accepts ERROR                           test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
9 a body error is reported against the directive                 test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
10 hasBody is inverted                                           test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
11 an attribute pair may have a body                             test result: FAILED (1 suite(s))  194 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
12 a literal class reference is not upcased                      test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
13 a method name is upcased                                      test result: FAILED (1 suite(s))  194 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
14 a routine name is upcased                                     test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
15 SUBCLASS and MIXINCLASS fill separate slots                   test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
16 a method may be EXTERNAL REGISTERED                           test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
17 an external specification may name four words                 test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
18 a signed constant need not be a number                        test result: FAILED (1 suite(s))  194 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
19 a constant may be followed by more data                       test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
20 a parenthesised constant need not be closed                   test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
21 REXX is not a reserved namespace                              test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
22 LIBRARY and NAMESPACE are not mutually exclusive              test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
23 a resource END keyword needs no marker                        test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
24 a resource accepts any sub-directive                          test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
25 an annotation target is not upcased                           test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
26 a second ROUTINE external reports the ROUTINE number          test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
27 a missing annotation value uses the bad-value number          test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
28 a directive's span is its tokens rather than its clause       test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
29 a resource takes the first body rather than its own           test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
30 whole_number ignores the caller's precision                   test result: FAILED (1 suite(s))  194 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
31 is_number goes through whole_number                           test result: FAILED (1 suite(s))  194 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
33 the blanks after a sign are not skipped                       test result: FAILED (1 suite(s))  10 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
34 a number's leading blanks are not stripped                    test result: FAILED (1 suite(s))  8 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
35 the conversion never rounds to the precision                  test result: FAILED (1 suite(s))  5 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
36 a rounded fraction need not be all zeros                      test result: FAILED (1 suite(s))  6 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
37 the carry compares against zero rather than nine              test result: FAILED (1 suite(s))  6 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
38 the value width is not checked against the precision          test result: FAILED (1 suite(s))  6 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
39 a carry that rounds a pure fraction up yields zero            test result: FAILED (1 suite(s))  6 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
40 an attribute name may be any symbol                           test result: FAILED (1 suite(s))  194 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
41 an attribute name may hold any character                      test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
42 an attribute name has no length bound                         test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
43 the ::ATTRIBUTE name is not checked at all                    test result: FAILED (1 suite(s))  194 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
44 a DELEGATE target is not checked                              test result: FAILED (1 suite(s))  194 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
45 the ::METHOD ATTRIBUTE name check ignores the sub-branch      test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
32 an unknown TRACE letter is accepted                           test result: FAILED (1 suite(s))  193 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
46 the ::ATTRIBUTE BOTH delegate check runs before the body check test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
47 the ::METHOD DELEGATE check runs after the body check         test result: FAILED (1 suite(s))  195 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

47 mutations, 0 survived or were never applied
```

---

# Round 4: closing notes

**One commit, `8a98ee40`.** Plus one write-up that is not code and not committed,
since `.superpowers/` is ignored.

## The carry rule now says the rule, and the old wording was wrong

Stating it in words caught a second error in the same place. The comment said
`0.4999999999` is rejected *"because nothing carries"*. **The carry does happen
there.** Its dropped digit is a 9, so `carry` is set; the value is rejected because
the FIRST KEPT digit is a 4 rather than a 9. The values were right and the
explanation was wrong, which is the direction you flagged as the more dangerous
one, and it had survived one round of review with me having just written a paragraph
about that exact hazard.

The rule as it now reads, in `whole.rs` and mirrored in `whole_value`'s doc comment:

> The digits have two separate jobs and it is easy to give them one. The FIRST
> DROPPED digit decides only whether there is a carry, and nothing else. The KEPT
> digits then decide whether the value is whole, and what they must equal depends on
> that carry: every surviving decimal must be a `0` normally but a `9` when the
> carry set, because only an all-nines tail can absorb the +1 and leave zeros.

Two rows were added to make it decisive rather than illustrative: **identical kept
digits, different dropped digit, opposite outcomes.**

```
=== dropped 4, no carry, kept nines must be zeros and are not
     1  trace "0.9999999994"                             Error 24.1
=== dropped 9, carry, kept nines must be nines and are
     1  trace "0.99999999999"                            rc=0
=== dropped 0, no carry, kept decimals all zero
     1  trace "1.0000000004"                             rc=0
```

The third covers the no-carry branch reaching a whole number, so the
compare-against-zero arm is tested in both directions too. Six rows total.

## The runner hole is closed, with a negative control

You were right that counting `diff` output cannot tell "the two sides agree" from
"both sides are empty", and that it did not bite here. It is fixed rather than
noted, because the runner is referenced from this report and will be reused: it now
prints the oracle and Rust line counts alongside the case count, asserts all three
are equal and non-zero, and reports `broken_sets` separately from `divergences`.

Re-verified after the fix, all three columns equal on every row:

```text
set            cases   oracle     rust  diverge
addsub          8712     8712     8712        0
addsub2         8112     8112     8112        0
muldiv         17424    17424    17424        0
md2            20184    20184    20184        0
pow             2112     2112     2112        0
cmp            32368    32368    32368        0
signblank       2320     2320     2320        0
fmt             1800     1800     1800        0
fmt2            6720     6720     6720        0
fmt3           12136    12136    12136        0
fmtedge          640      640      640        0
fmtcarry       15840    15840    15840        0
TOTAL         128368        -        -        0
label=post-review-final cases=128368 divergences=0 broken_sets=0
```

And a negative control on the guard itself, because a guard nobody has seen fire is
a guard nobody has tested. Pointing the runner at a generator that emits nothing:

```text
addsub             0        0        0        0
  BROKEN: addsub expected 0 answers, oracle gave 0 and rust gave 0
...
label=negative-control cases=0 divergences=0 broken_sets=12
guard_exit=1
```

Twelve broken sets, exit 1. The old runner would have printed twelve zeros and
exited 0 on exactly that input.

The corrected runner is at `scratchpad/t37/run-sets.sh` and is reproduced in full
below so it outlives the scratchpad.

## The upstream write-up

At `.superpowers/sdd/2026-07-28-phase-3-parser/upstream-numbervalue-carry-sign.md`.
Written up, **not filed**, and it says so in its first three lines along with the
reason: filing is Moritz's call, and I could not construct an observable case.

Auditing the source for the write-up **widened the finding from one site to two**.
There are four carry-only return branches in `NumberStringClass.cpp`, and the
asymmetry between them is the evidence:

| line | function | signed | rejects negatives | applies `numberSign` |
| --- | --- | --- | --- | --- |
| 595 | `numberValue` | yes | no | **no** — defect |
| 679 | `unsignedNumberValue` | no | yes, line 650 | no, correctly |
| 1083 | `int64Value` | yes | no | **no** — defect |
| 1181 | `unsignedInt64Value` | no | yes, line 1150 | no, correctly |

The same expression is right in two functions and wrong in two, which reads as a
line copied between them rather than a decision taken four times. Every other
return path in both signed functions multiplies by `numberSign`; only this branch
does not; and no comment marks it deliberate where the file explains its other
numeric edge cases at length.

I re-measured the three 24.901 rows myself rather than quoting yours:
`trace "-0.9999999999"`, `trace -1` and `trace 1` all reach
`Error 24.901: Numeric TRACE requests are valid only from interactive debugging`
at rc 232, so the rejection is of numeric `TRACE` as a category and the converted
value is discarded before anything can observe its sign.

The write-up states plainly that I did not audit the other callers of
`requestNumber` and `int64Value` for one that both reaches the branch and exposes
its result, that this is the work someone filing would need to do, and that it is
the difference between the document and a reproducible report.

## Final state

`fmt --check`, clippy `-D warnings`, `rexx-num`, `rexx-parse` and the workspace all
pass; the gate grep prints nothing. 437 workspace tests. 47 mutations, all applied,
all caught.

## The corrected set runner

```bash
#!/bin/bash
# Re-runs Phase 2's twelve curated differential sets and reports divergences.
#
# Usage: run-sets.sh <workdir> <label>
#
# The construction hole this closes, found by the coordinator reviewing the
# first version: counting `diff` output alone cannot tell "the two sides agree"
# from "both sides are empty". Two empty files diff clean, so a broken oracle
# invocation or a harness that failed to build would have printed twelve zeros.
# Every side is now asserted to have exactly as many lines as the case file, and
# any mismatch aborts rather than being reported as a pass.
set -u
R=/home/moritz/dev/repos/ooRexx-rust-rewrite
W=$1; LABEL=$2
GEN="$R/rust/crates/rexx-num/tests/gen-curated-sets.py"
mkdir -p "$W"
total=0; bad=0; broken=0
printf '%-11s %8s %8s %8s %8s\n' set cases oracle rust diverge
for s in addsub addsub2 muldiv md2 pow cmp signblank fmt fmt2 fmt3 fmtedge fmtcarry; do
  case "$s" in
    fmt|fmt2|fmt3|fmtedge|fmtcarry) bin=fmt-check; oracle=data-format-oracle.rex ;;
    *)                              bin=muldiv;   oracle=data-addsub-oracle.rex ;;
  esac
  python3 "$GEN" "$s" > "$W/$s.cases"
  n=$(wc -l < "$W/$s.cases")
  ( cd "$R" && LD_LIBRARY_PATH=$R/build/lib $R/build/bin/rexx \
      "rust/crates/rexx-num/tests/$oracle" "$W/$s.cases" ) > "$W/$s.oracle" 2>"$W/$s.oracle.err"
  ( cd "$R/rust" && cargo run --offline --release -q -p rexx-num --bin "$bin" -- "$W/$s.cases" ) \
      > "$W/$s.rust" 2>"$W/$s.rust.err"
  o=$(wc -l < "$W/$s.oracle"); u=$(wc -l < "$W/$s.rust")
  d=$(diff "$W/$s.oracle" "$W/$s.rust" | grep -c '^<')
  printf '%-11s %8d %8d %8d %8d\n' "$s" "$n" "$o" "$u" "$d"
  # A side that produced the wrong number of answers makes its zero meaningless.
  if [ "$n" -eq 0 ] || [ "$o" -ne "$n" ] || [ "$u" -ne "$n" ]; then
    echo "  BROKEN: $s expected $n answers, oracle gave $o and rust gave $u" >&2
    broken=$((broken + 1))
  fi
  total=$((total + n)); bad=$((bad + d))
done
printf '%-11s %8d %8s %8s %8d\n' TOTAL "$total" - - "$bad"
echo "label=$LABEL cases=$total divergences=$bad broken_sets=$broken"
[ "$bad" -eq 0 ] && [ "$broken" -eq 0 ]
```
