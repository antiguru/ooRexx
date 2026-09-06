# Phase 5f Task 4c — Base64

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 4.
BASE `1502f13c4`. Landed at `ac732b8ae`.

`ENCODEBASE64 DECODEBASE64`. The refresh moved exactly those two rows from
`loud` to `answers` with zero regressions. String now reads 128 `answers`, 6
`uncomparable` and 1 `loud`: **111 of the phase's 112 bound**, and the one left
is `hashCode`.

## 1. A codec, not a lift

There is no `ENCODEBASE64()` builtin and no base64 anywhere in the crate tree,
so this is the phase's only wholly new algorithm. It went into
`builtin/convert.rs` beside the other byte conversions: `BASE64`,
`base64_digit`, `encode_base64_bytes`, `decode_base64_bytes`.

The alphabet is RFC 2045's, with `+` and `/` rather than the URL-safe pair, and
that is measured rather than assumed from the RFC number: `'-w=='~decodeBase64`
is 93.962 where `'+w=='` answers.

## 2. `=` closes the last quartet and appears nowhere else

`RexxString::decodeBase64` (`classes/StringClassConversion.cpp:153`) tests
three things at once when it meets a character the alphabet does not hold:

```c
if (ch == '=' && inputLength <= 4 && (i == 3 || (i == 2 && *source == '=')))
```

`inputLength` is decremented per quartet, so `<= 4` means *the last one*. Every
consequence was predicted from that line and then measured, all 93.962 except
where noted:

| send | answer | which conjunct |
|---|---|---|
| `'AB=='~decodeBase64` | one `'00'x` byte | the legal `i == 2` pair |
| `'YWJ='~decodeBase64` | `ab` | the legal `i == 3` singleton |
| `'YW=j'~decodeBase64` | 93.962 | `i == 2` without its partner |
| `'Y=WJ'~decodeBase64` | 93.962 | position 1 is never legal |
| `'YW==YWJj'~decodeBase64` | 93.962 | a well-formed pair, wrong quartet |
| `'YWJj===='~decodeBase64` | 93.962 | padding reaching position 0 |
| `'===='~decodeBase64` | 93.962 | the same, with nothing before it |

A null string encodes and decodes to itself, and the length must be a multiple
of four — `'YWJ'` and `'YWJj='` are both 93.962.

## 3. One refusal, and it names nothing

93.962 is `Invalid Base64 encoded string.` with no substitutions at all: it
names neither the method nor the offending value. So the refusals witness is
told apart by *which rows raise*, not by their messages, and every row above is
in it. `ENCODEBASE64` has no refusal — every byte string encodes.

## 4. The control

Two instruments, and the control was built to separate them:

* **A** — `cargo test -p rexx-exec --lib builtin::convert`, the alphabet table
  test and the 256-value round-trip sweep.
* **B** — `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`.

Predictions written before any mutation ran.

| mutation | predicted | measured |
|---|---|---|
| N1 `BASE64`'s `+/` becomes the URL-safe `-_`, leaving `base64_digit` alone | A red on the table test, B red | **confirmed, under-predicted** — the round-trip reddened too, encode and decode having stopped agreeing |
| N2 padding allowed in any quartet | A green, B red on refusals row 7 | **confirmed** — and on the exit code, the untrapped tail being that very send |
| N3 a lone `=` in position 2 accepted | A green, B red on row 8 | **confirmed exactly** — `8 answered a` against the oracle's `8 raised 93.962` |
| N4 the length rule removed | A green, B red on rows 1-3 | **confirmed exactly** — `1 answered`, `2 answered`, `3 answered abc` |
| N5 a short group's fourth character is a digit | A red on the sweep, B red | **confirmed** |
| N6 `& 0x03` becomes `& 0x01` in the second digit | A red; B red **only on the non-ASCII lines** | **confirmed** |

**N6 was aimed at the plan rather than at the code.** The plan says the witness
"wants a non-ASCII case", so rather than take that on trust the control drops a
bit that every ASCII letter in the witness leaves unchanged -- `'a'` is `0x61`,
whose low two bits are `01`, so `& 0x03` and `& 0x01` cannot be told apart by
it. Measured, the mutant's output differs from the oracle's on exactly three
lines:

```
< x1 /w==            > x1 /Q==
< x2 AAECA//+        > x2 AAECAf/+
< rx 00010203FFFE    > rx 00010201FFFE
```

All three are the `'ff'x` and `'00010203fffe'x` rows. Every ASCII line is
untouched. So the instruction is load-bearing rather than decorative: a witness
built from letters alone would have shipped this mutation green. (Predicted x1
and x2 and named two; the round-trip line is a third, and is non-ASCII for the
same reason.)

N2, N3 and N4 are green on A and red on B, which is where the corpus witnesses
earn their place: the padding rules and the length rule are properties of
`decode_base64_bytes`'s contract with the *oracle*, and the round-trip sweep
never builds a string that violates either.

## 5. Bookkeeping

`corpus/refusal-sites.tsv` gained `invalid_base64` and 18 rows moved a line
number, re-derived from the source. `collect_stress.rs` is untouched -- both
witnesses allocate.

Two clippy findings on the new code, both style, both fixed before the commit:
`manual_is_multiple_of` and `chunks_exact_to_as_chunks`. Worth a line because
`cargo build` had already succeeded on the same code: under `-D warnings` they
surfaced as `error: could not compile ... due to 2 previous errors`, which
reads exactly like a compile failure and is two lints.

## 6. Gates

| gate | command | status |
|---|---|---|
| G1 | `cargo fmt --all --check` | rc 0 |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | rc 0 |
| G3 | `cargo test --release --workspace --no-fail-fast` | rc 0, 0 failed suites |
| G4 | G3 with `REXX_CORPUS_GATE=1` | rc 0, 0 failed suites |
| G5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | rc 0, 0 failed suites |
| G6 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1` | rc 0, 0 failed suites |
| G7 | `REXX_PHASE_GATE=5d REXX_CORPUS_GATE=1` | rc 0, 0 failed suites |

Run by `scratchpad/gates-4c.sh`, its status file opening with `sha
2f041fd49c3113dfe55e14b8f347f3821b8cab1c` -- the collections survey commit,
which sits on top of this task's and carries no code, so the gated tree is this
task's code.

Pre-commit chain: method-bodies refresh rc 0 (the two rows above `loud` ->
`answers`, no row on any other class moved), `cargo fmt --all --check` rc 0,
clippy rc 0, strict corpus 403 of 403, `coverage`, `collect_stress`,
`refusal_sites`, `sourceline_oracle` and `builtin::convert` each rc 0.
