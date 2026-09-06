# Phase 5f Task 4d — hashCode, and the phase's last row

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 4.
BASE `b41a5e5ba`. Landed at `PENDING`.

`HASHCODE`. **String now reads 129 `answers` and 6 `uncomparable`, with no
`loud` row left: all 112 of the phase's rows are bound.** The refresh moved
four rows and regressed none:

| row | to |
|---|---|
| `String hashCode` | `answers [rc 0]` |
| `DateTime hashCode` | `answers [rc 0]` |
| `TimeSpan hashCode` | `answers [rc 0]` |
| `Object hashCode` | `unstable [the oracle]` |

`File hashCode` stays `unanswered` — `.File~new` cannot construct, blocked on
the `file_qualify` entry point deferred to Phase 7.

## 1. The `Object` row was settled before the task started, and landed as ruled

`RexxObject::hashCode` (`classes/ObjectClass.cpp:398`) is `getHashValue()`
rendered as its own eight bytes, and `getHashValue` is virtual with
`identityHash() { return ((uintptr_t)this) ^ UINTPTR_MAX; }`
(`classes/ObjectClass.hpp:340`) as its base. That is the same value
`~identityHash` returns through another door, and `native_identity_hash`
already answers the handle for it under deviation 4's licence.

So this was never a decision about what to implement, and the `unstable`
verdict was **assigned by the harness rather than chosen**: it runs the oracle
twice, sees the value move, and records that neither `answers` nor `diverge`
would be a claim the run made.

## 2. The override set is by receiver kind, and a plausible rule is wrong

`getHashValue` is overridden by `NilObject`, `String`, `Pointer`, `Integer`,
`NumberString` and `Class`; every other receiver takes the identity hash.

The tempting rule is "anything with a string value hashes its text", and it is
false. Measured, two `.MutableBuffer~new('abc')` hash differently from each
other, differently from `'abc'`, and differently between the oracle's own runs
— a `MutableBuffer` renders as its contents and still takes the identity hash.
The witness's `kind` and `diff` rows are that fact, and mutation P4 is what
says they carry it.

`Integer` and `NumberString` delegate to their string value
(`IntegerClass.cpp:83`, `NumberStringClass.cpp:129`) and answer `String` for
their class, so one `Primitive::String | SmallInt` arm covers all three.
**The hash is of the number as it renders, not of its digits**: `(2**40)~hashCode`
equals `'1.09951163E+12'~hashCode` at DIGITS 9.

`Class` needed a row of its own — the oracle registers `RexxClass::hashCode`
separately (`memory/Setup.cpp:495`) rather than inheriting Object's, and
without it `.String~hashCode` refused.

## 3. The byte is signed, and that is the whole of the difficulty

`getStringHash` (`classes/StringClass.hpp:328`) accumulates `h = 31 * h + stringData[i]`
in a 64-bit register that wraps, and `stringData` is `char *` — **signed on
this platform**. An unsigned accumulator is right for every ASCII string and
wrong above `0x7f`, so a witness of letters alone cannot see it. Measured:
`'ff'x~hashCode` is eight `FF` bytes, the single byte having contributed -1
rather than 255, and `'80'x~hashCode` is `80FFFFFFFFFFFFFF`.

The formula was checked against ten measured values before any code was
written, and the one that did not match was my own assumption about the *text*
rather than the arithmetic — `2**40` renders exponentially at DIGITS 9.

## 4. DateTime and TimeSpan cost nothing

Neither is a C++ class with a `getHashValue` override. Both are Rexx-level
classes in `interpreter/RexxClasses/CoreClasses.orx` carrying an explicit
`::METHOD hashCode` that returns `timestamp~hashcode` — a number's hash, so
two equal instants agree and two different ones do not. Implementing the
number's hash was all they needed. `File`'s, at `StreamClasses.orx:803`, is
`self~qualifiedPath~hashCode` and will fall out the same way once `File`
constructs.

**This was found by re-measuring rather than by reading.** A grep of
`classes/DateTimeClass.hpp` returned nothing and I read that as "DateTime does
not override the hash" — the file does not exist, so the empty result was a
false negative from a wrong path, not a finding. The probe contradicted it,
and chasing the contradiction is what found `CoreClasses.orx`.

## 5. The control

Instruments: **A** `cargo test -p rexx-exec --lib the_string_hash`, the hash
against the oracle's own answers; **B** the corpus differential. Predictions
named the witness *lines*, not just the file, and each mutation's effect was
then measured line by line rather than read off "the file differs".

| mutation | predicted lines | measured |
|---|---|---|
| P1 the byte is unsigned | A red; B `high` only | **exact** — `['high']` |
| P2 the `Class` arm removed | A green; B `cls` only | **exact** — `['cls']` |
| P3 the `.nil` arm removed | A green; B `nil` only | **exact** — `['nil']` |
| P4 the plausible-but-wrong rule: anything that renders hashes its text | A green; B `diff` and `kind`, `same` unmoved | **exact** — `['diff', 'kind']` |
| P5 `to_le_bytes` becomes `to_be_bytes` | A green; B every value line | **exact** — `['one','abc','wrap','high','nil','cls','num','dt','ts']` |

**P4 and P5 are the pair worth keeping.** P4 applies the rule measurement
rejected in §2 and the witness catches it, so those two rows are load-bearing
rather than decorative. P5 is its complement and says the unit test alone is
not enough: `string_hash` can be perfect and the *rendering* still wrong, and
only the differential sees it. Note P5 leaves `null` alone — eight zero bytes
read the same either way, so the null string cannot witness endianness.

## 6. Gates

| gate | command | status |
|---|---|---|
| G1 | `cargo fmt --all --check` | PENDING |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | PENDING |
| G3 | `cargo test --release --workspace --no-fail-fast` | PENDING |
| G4 | G3 with `REXX_CORPUS_GATE=1` | PENDING |
| G5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | PENDING |
| G6 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1` | PENDING |
| G7 | `REXX_PHASE_GATE=5d REXX_CORPUS_GATE=1` | PENDING |

Pre-commit chain: method-bodies refresh rc 0 (the four rows above, no row on
any other class moved), `cargo fmt --all --check` rc 0, clippy rc 0, strict
corpus 405 of 405, `coverage`, `collect_stress`, `refusal_sites` and
`sourceline_oracle` each rc 0. `refusal-sites.tsv` is untouched -- `hashCode`
adds no constructor, having no refusal but its arity.
