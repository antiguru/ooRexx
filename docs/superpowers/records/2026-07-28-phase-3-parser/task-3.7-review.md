# Task 3.7 review: the nine `::` directives

Reviewed `d8bf0088..c62fcd69` against `task-3.7-brief.md` and `task-3.7-report.md`.
Oracle probes run into fresh `mktemp -d` working directories, whole message printed, exit status read on its own line.

## Verdicts

**Spec compliance: PASS.**
Every numbered step of the brief is done and done the way the brief asked.
Nine and forty are extracted from the two tables and asserted, the Rust tables are byte-identical to the C++ rows in C++ order, resolution goes through `KeywordSet::index_of` with no string table anywhere, `::RESOURCE` was implemented on the scanner mode Task 3.3 left room for and its body is never tokenised, all forty sub-directives are paired with a directive that accepts one and a directive that refuses one, and `CoreClasses.orx` parses end to end with its 32 / 303 / 12 decomposition pinned.
The two `::OPTIONS` `subKeywords[]` exceptions are a correction to the brief, not a deviation from it, and they are implemented at exactly the two points the C++ makes them.

**Code quality: CHANGES REQUIRED.**
One Critical: the number scanner both grammars now share rejects a legal spelling, and a test pins the wrong rule to a measurement that cannot support it.
Three Important, two of them silent acceptance of programs the oracle rejects at translate time, one of them a test that asserts far less than its doc comment claims.
The port itself is otherwise faithful at a level I could not fault: I read all nine C++ directive functions against their Rust counterparts and every guard, every mutual exclusion, every error code and every ordering between `checkDirective`, `decodeExternalMethod` and `hasBody` matches.

Findings: 1 Critical, 3 Important, 7 Minor.

## Critical

### C1. `whole_number` rejects a blank between the sign and the digits, and a test locks it in

`scan_number` strips leading and trailing blanks and tabs, then reads an optional sign, then requires a digit or a `.` immediately.
`NumberString::parseNumber` (`interpreter/classes/NumberStringClass.cpp:2586`) has a `NUMBER_SIGN_WHITESPACE` state: after a sign it scans off any number of spaces and tabs before the digits.
So the oracle takes `"+ 9"` as 9 and this code takes it as not a number.

Measured, all three paths:

| input | oracle | this code |
| --- | --- | --- |
| `trace "+ 9"` then `nop` | rc 0, skip count 9 | Error 24.1 |
| `trace "- 9"` then `nop` | rc 0 | Error 24.1 |
| `trace " + 9"` then `nop` | rc 0 | Error 24.1 |
| `::options digits "+ 9"` | rc 0 | Error 26.5 |
| `::options digits "+<TAB>9"` | rc 0 | Error 26.5 |
| `::options fuzz "+ 2"` | rc 0 | Error 26.6 |

`whole_number(b"+ 9", 18)` is `None`, confirmed by compiling `convert.rs` standalone.
`TRACE` then falls through to `check_trace_setting(b"+ 9")`, whose first byte is `+`, which is not in `ACEFILNOR`, so it is 24.1.

This is collateral damage in the instruction grammar, and it is the shape the brief warned about.
It is Critical for two reasons beyond the divergence itself.

First, it is a wrong **rejection**: a legal program fails to parse, in `TRACE` as well as in `::OPTIONS`, and `TRACE` is code Task 3.6 already shipped.

Second, `rust/crates/rexx-parse/src/convert/tests.rs:57-60` pins it:

```rust
// Measured: `"- 9"`, `"9 5"` and `"1 e2"` are all Error 26.5.
assert_eq!(whole_number(b"- 9", ARGUMENT_DIGITS), None);
```

The measurement is real and the conclusion drawn from it is false.
`::options digits "- 9"` is 26.5 because `-9` fails the `digits >= 1` filter, not because `- 9` is not a number.
The probe cannot distinguish the two, so it confirmed the hypothesis that chose it.
The positive control the probe needed is `"+ 9"`, and that control says rc 0.
The same false rule then propagated into two doc comments: `scan_number`'s "so the blanks may surround the number and not sit inside it" and the test's own name, `blanks_may_surround_a_number_but_not_sit_inside_one`.

Mutation 30 does not catch this either: it deletes the stripping entirely, which the `" 9 "` assertions catch, so the mutation is caught while the rule it protects is still wrong.

**Fix:** `scan_number` must allow spaces and tabs between the sign and the first digit or `.`, port the state machine rather than re-deriving it, and the test must assert `whole_number(b"+ 9", ..) == Some(9)` and `whole_number(b"- 9", ..) == Some(-9)` with `::options digits "+ 9"` as the citation.

## Important

### I1. Error 99.925 is missing at four call sites and is not in the deferred list

`getRetriever` (`interpreter/parser/LanguageParser.cpp:2507`) raises `Error_Translation_invalid_attribute`, 99.925, when a name is not a valid variable, stem or compound name.
`DirectiveParser.cpp` calls it four times: 831 (`::METHOD ... DELEGATE` target), 892 (`::METHOD ... ATTRIBUTE` name), 1656 (`::ATTRIBUTE` name) and 1707/1761/1826 (`::ATTRIBUTE ... DELEGATE` target).
`directive.rs` has none of them, and the module doc's "left to the caller" table does not mention 99.925.

It does not belong on that table either.
The check reads only the clause's own name text, so it needs nothing the accumulated package holds.
It was missed, I think, because the `syntaxError` call is in `LanguageParser.cpp` and not in the 2,867 lines the task was scoped to: my sweep of every `syntaxError`/`reportException` in `DirectiveParser.cpp` found 53 distinct codes and every one of them is either implemented here or on the deferred list.

Measured:

| input | oracle | this code |
| --- | --- | --- |
| `::attribute 3` | Error 99.925 at rc 157 | accepted |
| `::attribute .a` | Error 99.925 | accepted |
| `::attribute "a b"` | Error 99.925 | accepted |
| `::attribute 3 get` | Error 99.925 | accepted |
| `::attribute a delegate 5` | Error 99.925 | accepted |
| `::method m delegate 5` | Error 99.925 | accepted |
| `::method 3 attribute` | Error 99.925 | accepted |
| `::attribute a.` (control) | rc 0 | accepted |
| `::attribute a.b` (control) | rc 0 | accepted |
| `::method 3` (control) | rc 0 | accepted |

The two controls matter: a stem and a compound name are legal, and a bare `::METHOD 3` with no `ATTRIBUTE` is legal, so the check is on the attribute shapes only.

The ordering is observable and needs recording with the fix.
`::METHOD ... DELEGATE`'s retriever is taken **before** `checkDirective`: measured, `::method m delegate 5` with `return 1` under it is 99.925 on line 1, not 99.946 on line 2.
`::METHOD ... ATTRIBUTE`'s is taken **after**: `::method 3 attribute` with a body is 99.934 on line 2, and it is skipped entirely for the external shape, since `::method 3 attribute external "LIBRARY x"` reaches 98.903.

**Fix:** implement the four sites with the C++'s ordering, or, if it is deferred, say so with these measurements. It cannot stay silent.

### I2. `is_number` has no exponent-magnitude limit, so three shapes are accepted that the oracle rejects

`NumberStringBuilder` (`NumberStringClass.cpp:2519`) applies three limits `scan_number` does not:

* the accumulated exponent may not exceed nine nines (`invalidExponent`),
* `abs(numberExponent) > Numerics::MAX_EXPONENT` after folding in the decimals,
* `numberExponent + digitsCount - 1 > MAX_EXPONENT`.

`whole_number` is shielded by its own width check, so only `is_number` is exposed, and `is_number`'s one caller is `signed_constant`, which `::CONSTANT` and `::ANNOTATE` both use.
A leading digit makes the token a `SymbolClass::Constant`, so all of these reach `is_number`.

| input | oracle | this code |
| --- | --- | --- |
| `::constant c -1e1000000000` | Error 19.916 at rc 237 | accepted |
| `::constant c -99e999999999` | Error 19.916 | accepted |
| `::constant c -.1e-999999999` | Error 19.916 | accepted |
| `::annotate package a -1e1000000000` | Error 19.923 | accepted |
| `::constant c -9e999999999` (control) | rc 0 | accepted |
| `::constant c -1e999999999` (control) | rc 0 | accepted |

The controls place the boundary exactly: `-9e999999999` is legal and `-99e999999999` is not, which is the third limit and not the first, so a single `exponent > 999_999_999` guard is not enough.

Silent acceptance is the direction the differential method cannot see, which is why this is Important rather than Minor.

### I3. `the_other_shipped_packages_parse` asserts almost nothing while its doc comment claims counts

```rust
/// The other shipped package ... 7 `::CLASS`, 139 `::METHOD`, 5 `::ATTRIBUTE` and 2 `::CONSTANT`.
...
assert!(!directives.is_empty(), "{name} produced no directives");
```

The counts are correct, I checked them against the file, and none of them is asserted.
`core_classes_parses` pins its count, its decomposition and every span; this test passes if `StreamClasses.orx` yields one directive out of 153.
The report calls it "does the same for `StreamClasses.orx`", which is not what it does.

The `PACKAGES` slice also holds one row despite the plural name and the loop, so the second acceptance file the doc comment implies is not there either.

**Fix:** assert `(7, 139, 5, 2)` and the 153 total, the way `core_classes_parses` does.

## Minor

### M1. `is_number` is new code presented as moved code

The report says `convert.rs` "holds the three text conversions both grammars need, moved out of `instruction.rs`".
Two were moved. `is_number` did not exist before this task: the `instruction.rs` diff removes only `whole_number` and `check_trace_setting`, and `is_number`'s only callers are `directive.rs:828` and `convert/tests.rs`.
It matters because "moved" implies Task 3.6's tests already cover it, and they do not. C1 and I2 are both in `is_number`'s new territory.

### M2. A structuring semicolon in a doc comment

`ast.rs:1046`: "nothing echoes these bytes; they are kept because ...". CLAUDE.md forbids structuring semicolons in comments. The same sentence also wraps as "an install\n/// -time error", which reads as `install -time`.

### M3. The 33.1 note records one of its two directions

The deferred list says "33.1, a `::OPTIONS FUZZ` that is not less than the package's `DIGITS`".
The `SUBDIRECTIVE_DIGITS` arm raises the same exception when `digits <= fuzz` (`DirectiveParser.cpp:990`). Measured: `::options fuzz 5` then `::options digits 3` is 33.1 with the message naming DIGITS "3" and FUZZ "5". Whoever implements it from this note will implement half of it.

### M4. `ExternalSpec::entry`'s "the directive's own name supplies it" is three different defaults

* `::ROUTINE`: `entry = name`, the name **as written** (`DirectiveParser.cpp:2662`).
* `::METHOD`: `procedure = methodName`, the **upcased** internal name (`decodeExternalMethod`, `DirectiveParser.cpp:1405`).
* `::ATTRIBUTE`/`::METHOD ATTRIBUTE`: the same, with `"GET"` or `"SET"` appended, and for the GET/SET styles only when no third word was given (`DirectiveParser.cpp:1747`, `1804`).

One sentence covering three rules will be got wrong downstream. The distinction is unobservable through `rexxc` here only because the library never loads.

### M5. `resource()`'s `.expect()` is sound, and nothing local says why

I checked the invariant shape by shape rather than accepting it, and it holds.
The scanner accepts exactly 3 or 5 real tokens with `RESOURCE` at 1, a symbol-or-literal name at 2, and for 5 a symbol `END` at 3 and a symbol-or-literal at 4 (`scanner.rs:970-1045`); `resource()` succeeds on exactly that set, and 2, 4 and 6-or-more real tokens each fail one of `require_name`, `SUBDIR_END`, `require_name` or `required_end` first.
Both modules also index by the clause's first token, which is never a blank, so the keys agree.
The residual risk is that the invariant is spread across two files with nothing linking them: a `ParseCtx` built with `resources: &[]` panics rather than erroring. All four construction sites pass `&scanned.resources` today.

### M6. The `decode_external` whitespace concern is answerable and the answer is "correct"

`words()` goes to `RexxString::subWords`, which uses `WordIterator`, whose `skipBlanks` tests `*scan != ' ' && *scan != '\t'` (`interpreter/classes/StringClass.hpp:148-182`).
Space and tab, nothing else. A vertical tab or form feed is part of the word in the oracle and part of the word here, so the code is exact and the hedge in the doc comment should be retired rather than carried. A one-line test would cost nothing.

### M7. The `upper_value_of` concern is likewise answerable and correct

`RexxString::upper` upcases through `Utilities::toUpper`, which is `isLower(c) ? c & ~0x20 : c` with `isLower(c) = c >= 'a' && c <= 'z'` (`common/Utilities.hpp:52`).
ASCII-only, so `make_ascii_uppercase` is not an approximation of it, it is the same function. `::CLASS c SUBCLASS "ünïcode"` behaves identically. Retire the hedge; a test on one non-ASCII literal is cheap.

## Item 5: the cross-directive checks left to the caller

**Leaving them is right.** Every one on the list needs state `parse_directive` structurally cannot hold: the class/routine/method/attribute/constant/resource tables for the six duplicates, the active class for 99.905 and 99.906, the already-defined targets for 99.945, the package's other numeric setting for 33.1, and the loaded libraries for 98.903 and 90.998/90.999. 99.916 for a non-directive clause where a directive was due and 99.915 for a directive inside `INTERPRET` are the caller's loop and `translate`'s guard, not this function's.

**Recorded precisely enough, with two exceptions.** I verified the numbers against the oracle rather than accepting them: duplicate `::CLASS` is 99.901, duplicate `::RESOURCE` is 99.942, duplicate `::CONSTANT` is 99.932, duplicate `::ATTRIBUTE` is 99.931, and 33.1's message and rc 223 are as stated. The list names the C++ site for 99.915 and gives a positive control for 99.906 (rc 0 with a `::CLASS` above it), which is the standard 3.7b will need.

The two exceptions are M3, the 33.1 direction the note omits, and I1, which is on no list at all and is not a package-level check.

## Item 6: the three smaller claims

**`Terminators::with` moved to `#[cfg(test)]`: right, and for the stated reason.** It is used, at `expr/tests.rs:557-558`, so `cfg(test)` states a true contract where a dead-code allowance naming a future task would have stated a false one. The lib.rs note explaining the shrinking allowance set is the right place for it.

**`decode_external` splitting on blanks and tabs only: correct, not a gap.** See M6. The concern is resolvable from the C++ and resolves in the code's favour, so the right action is to state it as a fact and stop calling it untested.

**`upper_value_of` being ASCII-only: correct, not a gap.** See M7. Same conclusion. Neither of these is a coverage gap worth a finding on its own; carrying them as open concerns when the oracle source settles both is the only thing to fix.

## Mutation testing

**Re-applied five of the 32 myself**, with my own copy of the script under a fresh `mktemp -d`, restoring each file and asserting the restore.
All five reproduced the report's counts exactly:

| mutation | reported | mine |
| --- | --- | --- |
| 4 FORM and NUMERIC resolve against subDirectives | 198 passed; 2 failed | 198 passed; 2 failed |
| 9 a body error is reported against the directive | 199; 1 | 199; 1 |
| 24 a resource accepts any sub-directive | 199; 1 | 199; 1 |
| 29 a resource takes the first body rather than its own | 199; 1 | 199; 1 |
| 30 a number's surrounding blanks are not stripped | 197; 3 | 197; 3 |

**The unapplied-pattern guard works.** I added a sixth row whose `old` string appears nowhere, and the script printed `NOT APPLIED: pattern found 0 times`, added it to `failures`, and exited 1. `git status --porcelain` was empty afterwards.

Two weaknesses in the harness, neither a finding:

* `caught` treats `DID NOT COMPILE` as caught, so a mutation the compiler rejects is scored as tested by the test suite. Mutation 16 and 23 are the ones where that could matter; both actually compile and fail tests, per the counts.
* `run()` returning `NO RESULT` scores as a survivor, which is the safe direction.

The report's own account of the first run's bad verdict expression (`A or (B and C)` with a case-mismatched substring) is honest and it fell the safe way.

## What I verified directly, and it all matched

* The two tables. `DIRECTIVES` and `SUB_DIRECTIVES` in `token.rs` are line-for-line identical to the `directives[]` and `subDirectives[]` rows extracted from `KeywordConstants.cpp`, in order, 9 and 40. `diff` is empty for both.
* The four `SUB_KEYWORDS` indices. `subKeywords[]` 0-based gives ENGINEERING 12, INHERIT 22, NOINHERIT 29, SCIENTIFIC 37. All four constants match.
* **The `::OPTIONS` exception is not widened and not narrowed.** `DirectiveParser.cpp` calls `token->subKeyword()` exactly twice, at 1007 and 1339, and `directive.rs` calls `self.sub_keyword` exactly twice, at 929 and 994. Every other position in both files uses the sub-directive table. Both directions probed: `::options numeric noinherit` and `::options form engineering` are rc 0 with `NOINHERIT`/`ENGINEERING` absent from `subDirectives[]`, and `::options numeric syntax` is 25.935 with `SYNTAX` present in it.
* **`::RESOURCE` bodies stay raw.** `scanner.rs` stores byte ranges and nothing in the diff touches `scanner.rs` at all. A body holding `this is 'unmatched and /* unclosed` is rc 0 through `rexxc`, and `.Package~new` on a copy outside this repository returns it verbatim through `~resources~at("DATA")`.
* All nine C++ directive functions read against their Rust ports. `classDirective` (334), `methodDirective` (629), `attributeDirective` (1457 and the style switch at 1656), `decodeExternalMethod` (1403), `constantDirective` (1854), `annotateDirective` (1940), `processAnnotation` (2209), `resourceDirective` (2266), `routineDirective` (2565), `requiresDirective` (2779), `optionsDirective` (948). Every guard, every mutual exclusion, every error code and the `checkDirective` / decode / `hasBody` ordering match, including the `::ROUTINE` second-`EXTERNAL` oddity that reports the `::CLASS` number, `INHERIT` swallowing the rest of the clause, `SUBCLASS` and `MIXINCLASS` sharing one slot, and `ATTRIBUTE_BOTH` checking the body before decoding the external string where `::METHOD`'s plain external shape does the reverse.
* `check_trace_setting` against `TraceSetting::parseTraceSetting` (`interpreter/execution/TraceSetting.cpp:135`): any number of `?`, then one letter from the nine, then the loop breaks. Exact.
* The deferred numbers. 99.901, 99.942, 99.931, 99.932 and 33.1 all measured as stated; `::requires "nosuch"` and `::requires "nosuch" namespace ns` are rc 0, so the accepted-row table's claim that 38 of 40 are rc 0 holds where I checked it.
* Comment style. No em-dash anywhere in the added Rust; one structuring semicolon, M2.

## Not verifiable from the diff

* **`ARGUMENT_DIGITS` on a 32-bit build.** `Numerics.hpp:90` and `:98` give 18 and 9; only the 64-bit branch is reachable here. The concern is correctly raised in the report, and the constant's doc comment records the exposure. A 32-bit differential run will disagree with this build and there is no way to tell from here whether the eventual answer is a `cfg` or a fixed 18.
* **The mutation script's first run.** Its per-mutation output is summarised in one prose line rather than pasted, so the claim that all 32 printed `FAILED` while being listed as survivors cannot be checked. The second run's numbers are checkable and five of them check out.
* **Whether `::RESOURCE` bodies keep a trailing newline.** `~resources~at` printed one line with no separator, which is consistent with either. Nothing in this task depends on it, but Task 3.7b's package assembly will.
