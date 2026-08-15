# Task 3.3 re-review: fix round for the scanner review

Re-reviewed at `cf5f9852` against `review-410b2783..cf5f9852.diff` and
`task-3.3-review.md`. Scope is the twelve findings from that review only; the
original scanner implementation was not re-reviewed.

## Verdict: not yet

All twelve of the review's own findings are genuinely closed. But the fix
round's own new surface -- `ScanMode::Interpret` -- introduces a real,
oracle-confirmed divergence that the review never anticipated and the diff's
tests never exercise. Suppressing the `#!` skip under `INTERPRET` was the
named defect; the round fixed exactly that and left a sibling defect in the
same code path unaddressed. Details below.

## The twelve findings

| # | Finding | Verdict |
|---|---|---|
| IMP-1 | `#!` skip must be suppressed under `INTERPRET` | fixed |
| IMP-2 | A token span must be resolvable to bytes | fixed |
| MIN-1 | Per-clause `Vec` allocation in `scan_resource_if_directive` | fixed |
| MIN-2 | Self-contradicting `Operator` doc comment | fixed |
| MIN-3 | Panic-sweep test named as if panicking were the property | fixed |
| MIN-4 | Missing scanner-level span test across CRLF/bare-CR | fixed |
| MIN-5 | `Scanned::tokens`' unstated field invariant | fixed |
| MIN-6 | `ResourceBody` not saying what Task 3.7 owes | fixed |
| MIN-7 | Re-bound `Scanner::new` parameter | fixed |
| MIN-8 | `translate_char`'s unused return value | fixed |

12 of 12 genuinely fixed against what each finding literally asked for.

### Notes per item

**IMP-1.** `first_line` now reads `mode == ScanMode::Program && ...`, so the
skip is conditional and not merely present. `ScanMode` has no `Default` impl
and no other constructor path that could omit it -- `scan`, `Scanner::new`,
and the one other caller (`scan-check.rs`) all require it positionally, so
the original defect (unconditional skip) cannot silently return. Verified
independently (not by re-running the diff's own test): built `scan-check`
and ran it with `--interpret` against `#! nothing here` (13.1 line 1),
`#!/usr/bin/env rexx\nsay 1` (accepted in Program mode, 13.1 line 1 in
Interpret mode), and, per the explicit checklist item, `say 1\n#! not line
one` (`#!` on line **2**) -- this is 13.1 line 2 in *both* modes, confirming
line-2 shebangs are still errors regardless of mode. Program mode's own
line-1 skip is untouched and still correct.

**IMP-2.** `span_bytes` is `self.text.get(span)`, i.e. `<[u8] as
Index<Range<usize>>>::get`, which by construction cannot panic for any
`Range<usize>` -- it returns `None` for a reversed range, a range whose end
exceeds the slice length, or a start past the length, and `Some` (including
an empty slice) otherwise. Verified with a standalone probe program (not the
crate's own tests) exercising: empty span at 0, at `len`, and mid-text; the
full-text span; a span ending exactly at `len`; one byte past `len`;
reversed spans two different ways; a start past `len`; and a span into a
source truncated by `0x1A` (`ProgramSource::new` truncates `text` itself, so
a span that would have been valid pre-truncation correctly returns `None`
post-truncation, and a span inside the retained prefix correctly returns
`Some`). All matched expectations, no panics.

**MIN-1.** The `Vec<usize>` collect is gone, replaced by a `[usize; 5]` plus
a manual counter, and the DColon check on `self.tokens[self.clause_first]`
now runs before the loop, so a non-directive clause (the overwhelming
majority) allocates nothing. Verified resource detection still works
end-to-end (built `scan-check`, ran the well-formed two-line resource
example -- correctly found "resource at line 3: 2 lines") and that the
new fixed-size array can't be overrun: a clause with 6+ real tokens after
`::` (`::resource data end marker extra junk`) increments `count` to 5, then
on the 6th real token hits `if count == real.len() { return Ok(()); }` and
bails cleanly -- checked this exact case against `rexxc`, which rejects it
with 21.914 (a directive-parser error, correctly not the scanner's to
raise), and confirmed the Rust scanner leaves it as 0 resources, no panic.

**MIN-2 through MIN-8.** Read each in place in `token.rs` / `scanner.rs` /
`source.rs` and in the rewritten `tests/scanner.rs`. `Operator`'s doc no
longer claims "three... never scanned" while admitting `Concatenate` is
scanned; it now says "two... never scanned" and states `Concatenate` is
both. The panic-sweep test is renamed
`scan_always_answers_with_tokens_or_an_error_number`, matching its own doc
comment. `spans_stay_absolute_across_every_line_terminator` covers both CRLF
and bare CR with real symbol spans and `line_of` checks; reproduced its CRLF
case independently with `scan-check --tokens` and got matching absolute
offsets (`say 1` at 0..5, CRLF is 2 bytes, `say 22` starts at 7). `Scanned::tokens`'
doc states the three invariants (no adjacent terminators, none first, last
is `Eoc` if any tokens exist, no-clause programs produce zero tokens).
`ResourceBody`'s doc now names concretely what a directive parser still
owes: upcased-name keying, duplicate-name rejection
(`Error_Translation_duplicate_resource`), and malformed-directive rejection
(25.926) as the reason a malformed directive leaves no `ResourceBody` at
all. `Scanner::new`'s signature takes `mut symbols: SymbolTable` directly;
the re-bind line is gone. `translate_char` is deleted outright (not merely
documented) and `is_symbol_char` is a direct `matches!` over the same byte
set the old match arms used minus the upcasing, which is a strictly
stronger fix than the review asked for; confirmed no other caller
referenced `translate_char`.

## New defect this round introduced

**Important -- `ScanMode::Interpret` fixes the shebang check but leaves
`ProgramSource`'s line-splitting model wrong for `INTERPRET`, so the
scanner accepts interpret text the oracle rejects.**

The C++ oracle never splits an `INTERPRET` argument into physical lines at
all: `LanguageParser::translateInterpret` builds the source with
`new ArrayProgramSource(new_array(interpretString), lineNumber)`
(`LanguageParser.cpp:450`), and `new_array(interpretString)` is always a
**one-element** array. `ArrayProgramSource::setup` then sets
`lineCount = array->lastIndex()` (`ProgramSource.cpp:583`), i.e. 1. An
`INTERPRET` string is therefore always exactly one physical line, no matter
what bytes it contains -- a raw `0x0A` or `0x0D` byte inside it is just an
ordinary (invalid) character, never a line terminator.

Measured against `build/bin/rexx`:

```
s = "say 1" || '0a'x || "say 2"
interpret s
```

gives `Error 13.1: Incorrect character in program "` (newline) `" ('0A'X)`
-- the interpreter fails on the embedded byte itself, before it would ever
reach `say 2`.

`ProgramSource::new` in the Rust crate has no mode concept: it
unconditionally splits any input on `\n`/`\r` (`source.rs`, `buildDescriptors`
port), and `ScanMode` is only consulted later, inside `scan`/`Scanner::new`,
by which point the line index already exists. So for the identical bytes
`b"say 1\nsay 2"`, `scan(&source, ScanMode::Interpret)` returns `Ok` with 8
tokens and no error at all -- confirmed directly with `scan-check
--interpret`:

```
$ scan-check --interpret interp_two_valid_lines.rex
... ok 8 tokens, ... 0 resources
```

This is an oracle mismatch of the kind the differential harness exists to
catch: the oracle raises 13.1 on an INTERPRET string the scanner silently
accepts. It doesn't fire today because `parse_interpret` doesn't exist yet
(same "nothing is wrong today" as the original IMP-1), but the moment Task
3.9 builds a `ProgramSource` from an interpret string and calls
`scan(&source, ScanMode::Interpret)`, any interpret text containing a
literal `\n` or `\r` byte -- which does happen in practice, since
concatenating `'0a'x` is a normal way to write a multi-clause `INTERPRET`
argument -- will be mis-scanned as multiple valid lines instead of raising
13.1 on the embedded byte. `ScanMode` as designed cannot fix this: the line
split happens in `ProgramSource::new`, before any `ScanMode` value exists.
This is exactly the shape IMP-1's own review anticipated and warned against
("whatever the fix, it belongs to this task's interface, not to 3.9
discovering it") but the chosen fix (a mode parameter on `scan`/`Scanner`
alone) does not reach the part of the interface where this particular
divergence lives.

Failure scenario: Task 3.9 constructs
`ProgramSource::new(interpret_string.as_bytes().to_vec())` for
`s = "say 1"||'0a'x||"say 2"` and calls
`scan(&source, ScanMode::Interpret)`. The oracle raises `Error 13.1` on the
embedded newline; the Rust scanner returns `Ok(Scanned { .. })` with two
clauses' worth of tokens and no error.
