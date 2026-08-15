# Task 3.3 report: Scanner and tokens

Status: **DONE_WITH_CONCERNS**, after fix round 2.
The remaining concern is one measured behavioural deviation, which the coordinator has escalated to the repo owner and told me to leave alone.
Everything the brief asked for is implemented, tested and differential-tested.

## Commits

| SHA | What |
|---|---|
| `4e549dcf` | `ProgramSource::line_span`, the accessor the caller pre-authorised |
| `121aea8c` | `token.rs`: the 19 token classes, `SymbolTable`, `Keywords`, `ParseCtx`, `TokenCursor`, `ParseError` |
| `3526a813` | `scanner.rs`: the scanner, plus `tests/scanner.rs` |
| `410b2783` | Harness payload dump, the hex/bin literal differential, and the panic sweep |
| `cf5f9852` | Fix round 1: both Importants and all eight Minors |
| `9d263b22` | Fix round 2: program versus interpret is a property of the source |

Base was `fb2924db`.
Test summary: **61 tests pass** in `rexx-parse` (13 `sourceline`, 13 `tokens`, 35 `scanner`), `cargo clippy --offline --all-targets -- -D warnings` is clean, `cargo fmt -p rexx-parse -- --check` is clean, zero `unsafe`.

The brief's Steps map onto the commits as follows.
Step 1 (read the C++) produced no commit.
Steps 2 and 3 were done as written: `tests/scanner.rs` was written with the brief's five tests first and run against a crate with no `scan`, which failed with `E0432: unresolved import rexx_parse::scan`.
The commits are then split by deliverable rather than by Step, because committing a red tree would break bisection: `121aea8c` is the types the tests need to compile, `3526a813` is Step 4 plus the now-passing Step 2 tests, and `410b2783` is the Step 5 work that outgrew the harness.

## What is in the code

`src/token.rs` holds the parser's shared vocabulary: `Token`, the 19-variant `TokenKind` mirroring `TokenClass` (`Token.hpp:77`), `Tag`, `Operator` (all 32 subclasses), `SymbolClass`, `SymbolId`/`SymbolTable`, `KeywordSet`/`Keywords` with all six tables in `KeywordConstants.cpp` order, `ParseCtx`, `TokenCursor` and `ParseError`.
`src/scanner.rs` holds `scan`, `Scanned`, `ResourceBody` and the port itself.
`src/bin/scan-check.rs` is the differential harness, following the pattern `rexx-num/src/bin/` already established.

The port is structural rather than rule-by-rule, and that decision is the reason the surprising cases came out right without being special-cased.
`nextSpecial` calls `locateToken` with blanks insignificant, so it steps over blanks, comments and even a line continuation while looking for the second half of a doubled operator.
That makes `1 = = 1.0`, `1 =/*c*/= 1.0`, `1 =-` / `= 1.0` and `1 =/*` / `*/= 1.0` all strict comparisons.
Each was predicted from the C++ and then measured, and each falls out of the structure.
A scanner written from the prose rules would have got all four wrong.

## Fix round 2 (`9d263b22`)

Closing IMP-1 introduced a new Important, and the coordinator was right about both the defect and the cause.
`ScanMode` fixed the `#!` conditionality and *could not* fix the other two, because two of the three program-only behaviours live in `ProgramSource::new` rather than in the scan.

`INTERPRET` text is exactly one physical line: `LanguageParser::translateInterpret` (`LanguageParser.cpp:450`) builds `new ArrayProgramSource(new_array(interpretString), lineNumber)`, a one-element array.
So a raw LF, a raw CR or a Ctrl-Z inside it is an ordinary invalid character.
I re-measured every case before changing anything, and found two the coordinator had not listed:

```
$ build/bin/rexx r3/lf.rex        # interpret "say 1" || '0a'x || "say 2"
Error 13.1:  Incorrect character in program "\n" ('0A'X).
$ build/bin/rexx r3/cr.rex        # interpret "say 1" || '0d'x || "say 2"
Error 13.1:  Incorrect character in program "" ('0D'X).
$ build/bin/rexx r3/crlf.rex      # interpret "say 1" || '0d0a'x || "say 2"
Error 13.1:  Incorrect character in program "" ('0D'X).
$ build/bin/rexx r3/ctlz.rex      # interpret "say 1" || '1a'x || "say 2"
Error 13.1:  Incorrect character in program "?" ('1A'X).
$ build/bin/rexx r3/ctlz_end.rex  # interpret "say 1" || '1a'x
Error 13.1:  Incorrect character in program "?" ('1A'X).
$ build/bin/rexx r3/lf_only.rex   # interpret "say 1" || '0a'x
Error 13.1:  Incorrect character in program "\n" ('0A'X).
$ build/bin/rexx r3/semi.rex      # interpret "say 1; say 2"
1
2
$ build/bin/rexx r3/lit_ctlz.rex  # interpret "say c2x('" || '1a'x || "')"
1A
$ build/bin/rexx r3/empty.rex     # interpret "" then say "after"
after
$ build/bin/rexx r3/srcline.rex   # interpret "say sourceline()"
1
```

The last two are the ones I added.
A Ctrl-Z inside a literal survives as data, which confirms the absence of truncation from the other side, and `interpret ""` is accepted, which fixes what an empty interpret source has to look like.

`ScanMode` is deleted and `SourceKind` replaces it as `ProgramSource::new`'s second parameter, so the distinction is expressed once.
`scan` reads `source.kind()` and is back to `scan(&ProgramSource) -> Result<Scanned, ParseError>`, the signature the brief specified.
For `Interpret`, `new` does essentially nothing: one line spanning the whole text, no splitting and no truncation.
The invalid bytes then stay on that line and `characterTable` rejects them exactly as it rejects any other invalid character, so the scanner needed no new special-casing.
Empty text is one empty line under `Interpret` and no lines under `Program`, matching the array that always holds its element.

Pinned in both directions, as asked: each of LF, CR, CRLF, a mid-text Ctrl-Z, a trailing Ctrl-Z and a trailing LF is 13.1 under `Interpret` while the same bytes split or truncate under `Program`; `#!` is still skipped on a program's line 1 and is still 13.1 under `Interpret`; and `;` still separates interpret clauses, which is the case that had to keep working while the others started failing.

### Re-run after fix round 2

Everything re-swept, since this touched every `ProgramSource` construction.
12,059 corpus-l1 files and 985 other files: identical to before.
518 hex and binary literals: identical.
The interpret differential: 28 of 29, the twenty-ninth being `x += 1`, a runtime 41.1.

The sweep also caught a bug in its own fixtures, which is the change working rather than a problem: my interpret fixtures had a trailing newline, and under `SourceKind::Interpret` that newline is now correctly 13.1 while the oracle side reads the line with `linein`, which strips it.
I confirmed the scanner was right, not the fixture, by measuring `interpret "say 1" || '0a'x` directly.
Recorded for anyone reusing that harness: an interpret fixture file must have no trailing newline, because the two sides otherwise see different bytes.

### One thing the coordinator needs to do

`docs/superpowers/plans/2026-07-28-phase-3-parser.md` records `ScanMode` at lines 416, 425 and 1719, and `ScanMode` no longer exists.
I did not edit it, because `docs/` is outside this task's scope.

## Fix round 1 (`cf5f9852`)

### IMP-1: the `#!` skip is now suppressed under `INTERPRET`

`ArrayProgramSource::setup` (`ProgramSource.cpp:594`) guards the skip with `interpretAdjust == 0`.
I re-measured both directions before changing anything:

```
$ cat r2/i1.rex ; build/bin/rexx r2/i1.rex
interpret "#! nothing here"
say "after"
     1 *-* #
     1 *-* interpret "#! nothing here"
Error 13 running .../r2/i1.rex line 1:  Invalid character in program.
Error 13.1:  Incorrect character in program "#" ('23'X).
$ cat r2/i2.rex ; build/bin/rexx r2/i2.rex
#! nothing here
say "after"
after
$ cat r2/i3.rex ; build/bin/rexx r2/i3.rex     # the same text via a variable
x = "#! nothing here"
interpret x
say "after"
Error 13.1:  Incorrect character in program "#" ('23'X).
$ build/bin/rexx r2/i4.rex                     # interpret "#!"
Error 13.1:  Incorrect character in program "#" ('23'X).
```

`scan` took a `ScanMode`, `Program` or `Interpret`.
Fix round 2 replaced that with `SourceKind` on `ProgramSource::new`, for the reason above; the parameter-not-a-second-entry-point argument survived the change, it just applies to the constructor instead.
The harness takes `--interpret`, so both paths are reachable and diffable.

The interpret path is now differential-tested the way the program path was, through a driver that reads one line of a file and `interpret`s it.
`INTERPRET` takes a single source line, so the corpus is the single-line cases; 28 of 29 agree exactly, covering 6.1, 6.2, 6.3, 13.1, 30.1 and all six of 15.1 to 15.6, plus `#!`, `#!x`, `#!` alone and `#! /bin/sh`.
The twenty-ninth is `x += 1`, where the interpreter raises a runtime 41.1 because `x` is undefined in that context, and no scanner raises that.

### IMP-2: `ProgramSource::span_bytes`

A span is an absolute offset into the retained text, so it cannot be sliced out of a line, and there was no way out of the crate.
`span_bytes(Range<usize>) -> Option<&[u8]>` is named for what callers need rather than exposing the buffer, and the doc says why it is the only accessor: a whole-text getter would let a caller re-derive line boundaries, which is what the terminator rules in `ProgramSource` exist to prevent.
`None` rather than a panic or a clamp, because a span from anywhere other than the scanner may be out of range or assembled backwards.

The reviewer was right about the test at `tests/scanner.rs:436`, and about why.
It indexed a line-relative slice with an absolute span and passed only because `spans[0]` is `0..3`.
`spans[1]` is `12..15` and line 1 is 7 bytes, so it would have panicked.
The assertion now goes through `span_bytes` and checks all three occurrences, two of which are not on line 1, and also asserts `line_of` for them so the test states the thing it was meant to prove.

### The eight Minors

The resource matcher's per-clause `Vec` is gone.
It now tests `Tag::DColon` on the clause's first token before doing anything else, which allocates nothing for the clauses that are not directives, and collects into a five-element array for the ones that are, because neither accepted shape has more than five real tokens.
The `Operator` doc no longer says three operators are never scanned and then that `Concatenate` is.
`no_input_makes_the_scanner_panic` is now `scan_always_answers_with_tokens_or_an_error_number`.
`Scanned::tokens` states its three invariants on the field, and `ResourceBody` says what a directive parser still owes: the upcased table key, the duplicate-name check, and the malformed-directive rejection that is why a malformed one leaves no `ResourceBody`.
`spans_stay_absolute_across_every_line_terminator` covers CRLF, a bare CR and LF-then-CR at scanner level.
`Scanner::new` binds `mut symbols` in its signature.
`translate_char` is folded into `is_symbol_char`, with its documentation kept, since nothing used the upcased value.
`ParseError.byte`'s doc now says its name reads like the offending byte and is not one, and that filling `subs` needs the offending position as a second field, because 13.1 quotes `"ä" ('C3A4'X)` and 15.3 quotes `found "g"`.

### Re-run after the fixes

No regression anywhere.
12,059 corpus-l1 files: identical to before, 12,055 agree, one exact error match, two parse errors correctly accepted, one ordering deviation.
933 files across `ootest/`, `samples/`, `support/`, the rest of the repository, the class libraries and the crafted error cases: every scanner-class error still matches on number, sub-number and line, with the only non-matches the 22 `98.903` `::requires` link failures.
518 hex and binary literals still agree.
`setupoorexx.rex` still yields six resources with line counts 18, 28, 17, 30, 37, 38, and `createPortable.rex` 42, 42, 75.

## Deviations, and why

### 1. `Scanned` has a fourth field, `resources` (accepted in review)

The brief mandates in Step 1 that the `::RESOURCE` raw-text mode "must be designed in here", and specifies `Scanned { tokens, symbols, keywords }`, which has nowhere to put a body.
I added `resources: Vec<ResourceBody>`, where `ResourceBody { directive: usize, lines: Vec<Range<usize>> }` and `directive` is the token index of the `::` that opened the clause.

This is not optional.
Measured: a file whose resource body holds `this is 'unmatched and /* unclosed` gets rc 0 from `rexxc`, so a scanner that tokenised the body would raise 6.2 and 6.1 that the interpreter does not.
The alternative to a field was for Task 3.7 to re-derive the body's extent from `ProgramSource`, which duplicates the marker rule.

The scanner recognises the directive by shape over the clause's non-blank tokens, either `:: RESOURCE name` or `:: RESOURCE name END marker`, then copies lines until a line *begins with* the marker.
A malformed one is left alone, matching the C++, which rejects it in the directive parser before reading any line: measured, `::resource data junk` is error 25.926 on line 2.

### 2. A `#!` line is skipped, and the scanner does it (accepted in review; the conditionality was IMP-1)

This is a gap in Task 3.2's deliverable that only surfaces here.
`BufferProgramSource::buildDescriptors` (`ProgramSource.cpp:448`) sets `firstLine = 2` when the buffer opens with `#!`, and `LanguageParser::translate` (`LanguageParser.cpp:764`) positions there.
`SOURCELINE` still returns the line, so `ProgramSource` needs no change and I made none; the scanner computes it from `source.line(1)`.

Found by differential testing, not by reading: 494 of the 790 files under `ootest/` and `samples/` open with `#!/usr/bin/env rexx`, and every one of them was error 13.1 on line 1 here against rc 0 from `rexxc`.
With the fix, all 790 agree, including two files where the oracle reports 13.1 on **line 2** and so does this scanner.

### 3. An eager scan can report a later scan error where the oracle reports an earlier parse error (escalated, not mine to settle)

The interpreter interleaves scanning with parsing clause by clause, so a scan error further down the file is never reached if an earlier clause fails to parse.
`scan(&ProgramSource) -> Result<Scanned, ParseError>` is eager by construction, so it sees the whole file.

Measured, deliberately: a file whose line 1 is `say )` and line 3 is `x = 'unclosed` reports `Error 37.2` on line 1 under `rexxc`, where this scanner reports 6.2 on line 3.
Measured, in the wild: exactly one file in the 12,059-file corpus shows it, `rust/corpus-l1/LINES_test_stdin_normal.rex`, where a stray `*/` on line 22 is error 35.1 and an unclosed `/*` on line 24 is what this scanner reports.
It cannot be fixed without making scanning lazy, which the mandated signature rules out.

Two corrections from review, both of which I accept.
This produces a different error *number*, not merely a different line, and the project's parse-error relaxation explicitly keeps number and sub-number, so the relaxation does not cover it.
And my frequency estimate was optimistic: the reviewer found it fires on any clause boundary including a mid-line `;`, hitting 144 of 4,000 adversarial inputs rather than once in 12,059.
The coordinator has put the question to the repo owner and told me not to restructure `scan`, so the behaviour is unchanged.

### 4. `ParseCtx` and `TokenCursor` are `pub`, not `pub(crate)` (accepted for now; owed to Task 3.5)

The brief specifies `pub(crate)`.
Nothing in the crate constructs either type yet, so `pub(crate)` makes both `dead_code`, and gate criterion 8 runs clippy with `-D warnings`.
The choice was `pub` or an `#[allow(dead_code)]`; `pub` also lets `tests/tokens.rs` exercise `TokenCursor`, including that `back` before the clause start panics.
Tasks 3.5 to 3.7 can narrow it once they have callers.
Recorded for whoever does: narrowing also means moving the four `TokenCursor` tests out of `tests/tokens.rs` into a `#[cfg(test)]` module inside `token.rs`, or it will not compile and will simply be reverted.

### 5. Three things the brief listed that are deliberately not reproduced

`ParseError::subs` exists for interface stability and is always empty, per the global constraint that this phase does not reproduce message text or substitutions.

`TokenKind::Symbol` does not carry the C++'s `INTEGER_CONSTANT` numeric tag.
That flag only chooses an internal number representation (`LanguageParser.cpp:2371` builds an integer object rather than a string plus number-string) with no observable effect, and reproducing it would mean reproducing a platform-dependent digit limit, 9 on a 32-bit build and 18 on a 64-bit one.

`TokenKind::Literal` does not carry the `LITERAL_STRING`/`LITERAL_HEX`/`LITERAL_BIN` subclass.
A grep over `interpreter/` finds those three constants only in `Scanner.cpp` and the enum declaration, so nothing downstream distinguishes them once the value is packed.
`Null`, `Prefix`, `Point` and `Continue` *are* present, so the enum mirrors all 19 classes, and each is documented as never emitted; the same grep shows the C++ never constructs those four either.

## One surprising span, recorded because it looks like a bug

A blank produced by a line continuation gets a span on the *next* line, overlapping the token after it.
`say 'a',` / `'b'` yields a `Blank` at `9..10` and a `Literal` at `9..12`.
This is faithful: `sourceNextToken` calls `startLocation` after `locateToken` returns, and by then the continuation has already stepped to the next line, so the C++ records the same overlap.
It is the only case where token spans are not disjoint, and `every_token_span_lies_inside_the_source_and_is_ordered` pins the weaker invariant that holds everywhere: ordered, non-decreasing in start, inside the text.

## Measurement the brief asked for in Step 5

The brief asked for the real symbol occurrence to distinct symbol ratio, and said to treat any number quoted before the scanner existed as an estimate.
Measured with the scanner:

| file | symbol occurrences | distinct upcased symbols | ratio |
|---|---|---|---|
| `interpreter/RexxClasses/CoreClasses.orx` | 8,118 | 526 | 15.4x |
| `interpreter/RexxClasses/StreamClasses.orx` | 2,121 | 273 | 7.8x |
| `interpreter/platform/unix/PlatformObjects.orx` | 0 | 0 | n/a |

The brief's estimate of 10x to 16x was right for `CoreClasses.orx` and high for the second file.
Note the third row: on this platform `PlatformObjects.orx` is a single line, `-- Nothing to do currently`, so it contributes no tokens at all and cannot be one of the two files the estimate came from.
`StreamClasses.orx` is used above in its place.
Of `CoreClasses.orx`'s 526 distinct symbols, 74 are spellings the keyword tables had already interned, so the table grows by 452 for that file.

## Differential testing

`build/bin/rexxc FILE` for the verdict, `build/bin/rexx FILE` only where both spellings parse and the printed result is the discriminator.
The harness compares `rexxc`'s error number, sub-number and reported line against `scan-check`'s.
It was checked for discrimination rather than assumed: on `say )` it reports `E37.2 line 1` against `ok`.

| corpus | files | result |
|---|---|---|
| `rust/corpus-l1/*.rex` | 12,059 | 12,055 agree; 1 exact error match (6.1 line 7); 2 parse errors this scanner correctly accepts; 1 ordering deviation (item 3 above) |
| `ootest/`, `samples/`, `support/` | 790 | 788 agree, 2 exact error matches (13.1 line 2) |
| rest of the repository | 142 | 120 agree; 22 are 98.903 `::requires` link failures, not scanner errors |
| interpreter class libraries and `build/bin/*.cls` | 17 | all agree |
| crafted error cases | 40 | all agree on number, sub-number and line |
| hex and binary literal contents | 518 | all agree on decoded bytes or error sub-number |
| `rust/corpus/lang` | 14 | all agree |

Token spans were checked for the ordering and bounds invariant over 3,000 corpus files plus the class libraries, with no violations.

### Raw oracle output

Every probe below was run against `build/bin/rexx` or `build/bin/rexxc` in this session.

**The significant blank, the deciding case.**

```
$ cat p1a.rex ; build/bin/rexx p1a.rex
say abs (2.5)
ABS 2.5
$ cat p1b.rex ; build/bin/rexx p1b.rex
say abs(2.5)
2.5
```

**A comment separates without inserting a blank.**

```
$ cat p2.rex
a = 1; b = 2
say a/*c*/b
say a b
$ build/bin/rexx p2.rex
12
1 2
```

**A continuation is a blank, unless the previous token suppresses it.**

```
$ cat p3.rex
say "a"-
"b"
say "a"||-
"b"
say "a",
"b"
$ build/bin/rexx p3.rex
a b
ab
a b
```

**`--` against `-`, and nested comments, and doubled quotes.**

```
$ cat p4.rex ; build/bin/rexx p4.rex
say 1 -- 2
say 1 - 2
1
-1
$ cat p5.rex ; build/bin/rexx p5.rex
/* a /* b */ c */ say 1
1
$ cat p6.rex ; build/bin/rexx p6.rex
say 'it''s'
it's
```

**`nextSpecial` skips blanks, comments and continuations. This is the one the structural port bought.**

```
$ cat p8.rex ; build/bin/rexx p8.rex
say 1 == 1.0
say 1 = 1.0
say 1 = = 1.0
0
1
0
$ cat p8b.rex ; build/bin/rexx p8b.rex
say 1 =/*c*/= 1.0
0
$ cat p9.rex ; build/bin/rexx p9.rex
say 2 < = 2
say 2 <= 2
1
1
$ cat p10.rex ; build/bin/rexx p10.rex
say 1 =-
= 1.0
0
$ cat p11.rex ; build/bin/rexx p11.rex
say 1 =/*
*/= 1.0
0
```

**The exponent sign, and the backup when no digits follow.**

```
$ cat p12.rex ; build/bin/rexx p12.rex
say 1e+5
y = 5
say 1e+y
1E+5
     3 *-* say 1e+y
Error 41 running .../p12.rex line 3:  Bad arithmetic conversion.
Error 41.1:  Nonnumeric value ("1E") used in arithmetic operation.
$ cat v/d.rex ; build/bin/rexxc v/d.rex >/dev/null
x = 1e+
say x
     1 *-* x = 1e+
Error 35 running .../v/d.rex line 1:  Invalid expression.
Error 35.1:  Incorrect expression detected at "+".
```

**Hex and binary literals, valid and invalid. Note that these are parse-time: line 1 of `p13.rex` never printed.**

```
$ cat p13.rex
say c2x('41'x)
say c2x('414 2'x)
say c2x('a'x)
say c2x(''x)
say c2x('1000 0001'b)
say c2x('1'b)
$ build/bin/rexxc p13.rex >/dev/null ; echo rc=$?
     2 *-* say c2x('414 2'x
Error 15 running .../p13.rex line 2:  Invalid hexadecimal or binary string.
Error 15.5:  Hexadecimal strings must be grouped in units that are multiples of two characters.
rc=241
$ build/bin/rexxc probes/p15.rex >/dev/null    # say c2x(' 41'x)
Error 15.1:  Incorrect location of whitespace character in position 1 in hexadecimal string.
$ build/bin/rexxc probes/p16.rex >/dev/null    # say c2x('4g'x)
Error 15.3:  Only 0-9, a-f, A-F, and whitespace characters are valid in a hexadecimal string; found "g".
$ build/bin/rexxc v/a.rex >/dev/null           # say c2x('101 01'b)
Error 15.6:  Binary strings must be grouped in units that are multiples of four characters.
$ cat mean/m20_bin_literal.rex ; build/bin/rexx mean/m20_bin_literal.rex
say c2x('1000 0001'b)
say c2x('1'b)
say c2x('101 0101'b)
81
01
55
```

`'101 0101'b` is accepted, which I had expected to be 15.6 before running it: a short *leading* group is legal in binary exactly as `'a'x` is in hex, because the validation scans right to left.

**Unmatched quotes and comments, and the reported line is the clause's, not the error's.**

```
$ build/bin/rexxc probes/p17.rex >/dev/null    # say 'abc
Error 6 ... line 1:  Unmatched "/*" or quote.
Error 6.2:  Unmatched single quote (').
$ build/bin/rexxc probes/p18.rex >/dev/null    # say "abc
Error 6.3:  Unmatched double quote (").
$ cat probes/p33.rex ; build/bin/rexxc probes/p33.rex >/dev/null
say 1,
'unclosed
     1 *-* say 1,'unclosed
Error 6 ... line 1:  Unmatched "/*" or quote.
Error 6.2:  Unmatched single quote (').
$ cat probes/p34.rex ; build/bin/rexxc probes/p34.rex >/dev/null
say 1,
/* unclosed
     1 *-* say 1,/* unclosed
Error 6 ... line 1:  Unmatched "/*" or quote.
Error 6.1:  Unmatched comment delimiter ("/*") on line 2.
$ cat probes/p35.rex ; build/bin/rexxc probes/p35.rex >/dev/null
say 1,
c2x('4g'x)
     1 *-* say 1,c2x('4g'x
Error 15 ... line 1: ...
$ cat probes/p36.rex ; build/bin/rexxc probes/p36.rex >/dev/null
say 1,
bäc
     1 *-* say 1,bä
Error 13 ... line 1:  Invalid character in program.
Error 13.1:  Incorrect character in program "ä" ('C3A4'X).
```

All four report line 1, the clause's start, while 6.1's substitution names line 2.
This is what fixes `ParseError.byte` to the clause start.

**Non-ASCII in a symbol, and the symbol length limit.**

```
$ build/bin/rexxc probes/p20.rex >/dev/null    # bäc = 2
Error 13.1:  Incorrect character in program "ä" ('C3A4'X).
$ build/bin/rexxc probes/p21.rex >/dev/null    # 251 x's
Error 30.1:  Name or symbol exceeds 250 characters:  "XXX...".
$ build/bin/rexxc probes/p22.rex >/dev/null ; echo rc=$?   # 250 x's
rc=0
```

**`::RESOURCE`.**

```
$ cat probes/p25.rex ; build/bin/rexxc probes/p25.rex >/dev/null ; echo rc=$?
say .resources~index~makestring
...
::resource data
this is 'unmatched and /* unclosed
line two
::END
rc=0
$ cat probes/p26.rex ; build/bin/rexx probes/p26.rex
d = .resources['DATA']
say d~items
do l over d
  say '['l']'
end
exit
::resource data
this is 'unmatched and /* unclosed
line two
::END
2
[this is 'unmatched and /* unclosed]
[line two]
$ cat probes/p27.rex ; build/bin/rexx probes/p27.rex
say .resources['D2']~items
exit
::resource d2 end 'STOP'
::END is just data here
STOPPING? no, prefix match
1
$ cat probes/p28.rex ; build/bin/rexxc probes/p28.rex >/dev/null
say 1
::resource data
never terminated
     2 *-* ::resource data
Error 99 ... line 2:  Translation error.
Error 99.943:  Missing ::RESOURCE end marker "::END" for resource "DATA".
$ cat probes/p29.rex ; build/bin/rexx probes/p29.rex     # `;` then more on the line
say .resources['DATA']~items
exit
::resource data; say 'skipped?'
body1
::END
1
$ cat probes/p30.rex ; build/bin/rexx probes/p30.rex     # a symbol marker is upcased
...
::resource d3 end stop
stop is lowercase, no match
STOP
1
[stop is lowercase, no match]
$ build/bin/rexxc v/e.rex >/dev/null                     # ::resource data junk
Error 25.926:  Unknown keyword on ::RESOURCE directive; found "JUNK".
```

Per-resource line counts against the interpreter's own `~items`, for `support/portable/setupoorexx.rex`:

```
interpreter:  UNIX_LEADIN 38, UNIX_RXENV 17, UNIX_SETENV2RXENV 30,
              WINDOWS_LEADIN 37, WINDOWS_RXENV 18, WINDOWS_SETENV2RXENV 28
scan-check:   line 509: 18, line 530: 28, line 561: 17, line 581: 30,
              line 615: 37, line 655: 38
```

Six for six, including `::RESOURCE windows_leadin     -- CPL license`, whose directive line carries a `--` comment.

**Continuations at the edges.**

```
$ cat probes/p23.rex ; build/bin/rexx probes/p23.rex
say "a",

"b"
a
/bin/sh: 1: b: not found
     3 *-* "b"
$ cat mean/m03_trailing_comma.rex ; build/bin/rexx mean/m03_trailing_comma.rex
say 1,
1
```

A continuation onto an *empty* line ends the clause, and a continuation with no next line is simply consumed.

**Blank significance on the other classes, and tabs.**

```
$ cat probes/p32.rex ; build/bin/rexx probes/p32.rex
say ('x') ('y')
say ('x')('y')
x y
xy
$ cat probes/p24.rex ; build/bin/rexx probes/p24.rex
a = 1
say a<TAB>2
1 2
$ cat v/f.rex ; build/bin/rexx v/f.rex
say a<TAB> <TAB>2
A 2
```

**Literals and markers.**

```
$ cat mean/m07_hex_then_symbol.rex ; build/bin/rexx mean/m07_hex_then_symbol.rex
xy = '!'
say 'a'xy
a!
$ cat mean/m08_abutted_hex.rex ; build/bin/rexx mean/m08_abutted_hex.rex
say '41'x'42'x
AB
$ cat mean/m09_doubled_double.rex ; build/bin/rexx mean/m09_doubled_double.rex
say "a""b"
say "a" "b"
a"b
a b
$ cat v/c.rex ; build/bin/rexx v/c.rex
say 'a"b'
say c2x('41<TAB>42'x)
a"b
4142
```

**Symbol shapes and operators.**

```
$ cat mean/m04_dot_symbols.rex ; build/bin/rexx mean/m04_dot_symbols.rex
say .5
say 1.5
say .nil
.5
1.5
The NIL object
$ cat mean/m05_dummy_dot.rex ; build/bin/rexx mean/m05_dummy_dot.rex
parse value 'p q' with . y
say y
q
$ cat v/b.rex ; build/bin/rexx v/b.rex
a?b!c_d = 1
say a?b!c_d
1
$ cat mean/m02_assign_op.rex ; build/bin/rexx mean/m02_assign_op.rex
x = 5
x += 1
say x
6
$ cat mean/m23_percent_op.rex ; build/bin/rexx mean/m23_percent_op.rex
say 7 % 2
say 7 // 2
say 2 ** 3
3
1
8
$ cat mean/m24_not_variants.rex ; build/bin/rexx mean/m24_not_variants.rex
say \1
say 1 \= 2
say 1 \== 2
0
1
1
$ cat mean/m11_dtilde.rex ; build/bin/rexx mean/m11_dtilde.rex
s = 'abc'
say s~~length
say s~length
abc
3
$ cat mean/m01_brackets.rex ; build/bin/rexx mean/m01_brackets.rex
a = .array~of(10,20)
say a[1]
say a [1]
     3 *-* say a [1]
Error 35.1:  Incorrect expression detected at "[".
```

`say a [1]` is a parse error rather than a scan error, which is consistent with `[` being on the right-hand side of the blank rule: the blank token is emitted and the parser then rejects it.

**Labels, semicolons, case folding.**

```
$ cat v/g.rex ; build/bin/rexxc v/g.rex >/dev/null ; echo rc=$?
label: nop
'MiXeD': nop
say 1
rc=0
$ cat mean/m26_semicolons.rex ; build/bin/rexx mean/m26_semicolons.rex
say 1;;say 2;
1
2
$ cat mean/m19_symbol_case.rex ; build/bin/rexx mean/m19_symbol_case.rex
aBc = 1
say ABC
say sourceline(1)
1
aBc = 1
```

**Shebang lines.**

```
$ build/bin/rexxc v/h.rex >/dev/null ; echo rc=$?   # only a #! line
rc=0
$ build/bin/rexxc v/j.rex >/dev/null ; echo rc=$?   # #!x then say 1
rc=0
$ build/bin/rexxc err/ok_shebang_bad_line2.rex >/dev/null
Error 13.1 ... line 2
```

**The eager-scan ordering deviation, measured on purpose.**

```
$ cat probes/p31.rex ; build/bin/rexxc probes/p31.rex >/dev/null
say )
say 2
x = 'unclosed
     1 *-* say )
Error 37 ... line 1:  Unexpected ",", ")", or "]".
Error 37.2:  Unmatched ")" in expression.
```

This scanner reports 6.2 on line 3.

## Process note

I ran `.Package~new` on `support/portable/setupoorexx.rex` in the repository to read its resource table, and its prolog executed and wrote `support/portable/rxenv.sh` and `support/portable/setenv2rxenv.sh`.
Both were untracked, both were created at 16:43 while every tracked file in that directory dates from the previous day, and I removed them; `git status` is clean of them and no tracked file was touched.
That is exactly the hazard the brief warns about, running a program with a side effect for a verdict.
I repeated the measurement on a copy in the scratchpad.

## What a later task should know

`Program::labels` must be a `BTreeMap<Box<str>, usize>` keyed by the token value, not by `SymbolId`.
`TokenKind::Literal` carries its decoded value for exactly this reason, and `'MiXeD': nop` gets rc 0 from `rexxc`, so a literal label is legal and keeps its case.
There is deliberately no `SymbolTable` lookup-by-name method.

Task 3.4 has no continuation rule to implement: both continuation characters are resolved inside `locate_token`, so `split_clauses` never sees an `Eoc` at a continued line end.

Task 3.7 reads a resource body from `Scanned::resources`, matching on `ResourceBody::directive` against the `DColon` token index of the directive clause it is parsing.
