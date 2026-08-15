# Review of fix round 1 (commit 7f664aa7)

Verdict: **CHANGES NEEDED**

Scope reviewed: `review-H-round1.diff` (583 lines), the "Fix round 1" section of `task-H-report.md`, the full post-commit state of `rust/crates/rexx-num/src/{lib,muldiv,settings,compare}.rs`, and the C++ oracle sources the commit cites.
All claims below marked "probed" were run against `build/bin/rexx` in this session; all Rust outputs were produced by running the committed crate at `7f664aa7`.

## Critical

### C1: The fix's central premise is false, and the error-5 boundary it ports exists for `+`, `-`, `*`, and `**` too

The commit's comments state, twice, that "Division is the one operation whose working storage is sized by DIGITS itself rather than by its operands" (`rust/crates/rexx-num/src/lib.rs`, `SystemResources` doc comment; `rust/crates/rexx-num/src/muldiv.rs`, the reservation comment).
That claim is false, and the C++ sources show it directly:

* `NumberString::addSub` allocates `new_buffer(digits * 2 + 1)` whenever `digits > FAST_BUFFER` (`interpreter/classes/NumberStringMath.cpp:808-811`).
* `NumberString::Multiply` allocates `new_buffer(((digits + 1) * 2) + 1)` past the same cutoff (`interpreter/classes/NumberStringMath2.cpp:134-146`).
* The power operation allocates `new_buffer(accumLen * 2)` unconditionally, with `accumLen = (2 * (digits + extra + 1)) + 1` derived from DIGITS inflated by the exponent's digit count (`interpreter/classes/NumberStringMath2.cpp:896-902`).

Concrete failure scenarios, every row confirmed this session at `NUMERIC DIGITS 999999999999999999` (a legal setting, reached via `numeric digits 18` then `numeric digits 999999999999999999`):

| expression | oracle (`build/bin/rexx`) | rexx-num at 7f664aa7 |
|---|---|---|
| `'1.0' + 1` | error 5 | `2.0` |
| `'3.0' - 1` | error 5 | `2.0` |
| `'1.5' * 2` | error 5 | `3.0` |
| `'2.0' ** 3` | error 5 | `8` |
| `'4.0' / 2` | error 5 | `<E5>` (the one case this round fixed) |

The right output is error 5 in all five rows; the crate returns a value in four of them.

To be precise about provenance: the behavioral divergence predates this commit — it was opened by the u64 DIGITS widening in `e91a333d`, and the H3 finding named division only.
What this commit introduces is the false uniqueness claim, committed as the design rationale, plus a fix structure that declares the class closed when four fifths of it is still open.
No differential set can see this: all twelve sets cap at DIGITS 20, so the 128,368-case zero stands and is simply blind here — the same error-boundary blind spot the phase already knew about.

Guidance for the fix, from evidence gathered here:

* The request sizes differ per operation (addSub `2D+1`, mul `2D+3`, div `3(2D+3)`, pow `2(2(D+e+1)+1)` with `e` the exponent's digit count), so one shared probe with one formula will not reproduce the oracle's per-op boundaries in the machine-dependent middle band.
* Do not place a reservation inside `sub` or `mul` themselves: `compare` calls `sub` internally (`compare.rs:145`) and the `%`/`//` remainder tail calls `mul` and `sub` at inflated precision (`muldiv.rs`), and the oracle's comparison at max DIGITS succeeds (probed: `'1.0' = '1'` is 1, `'1.2345' < '1.2346'` is 1) because `NumberString::comp`'s fast path never allocates by DIGITS.
  The reservation belongs at the operator entry points, mirroring where each C++ routine allocates, exactly as this round already did for `div`.

## Important

None.

## Minor

### M1: signblank docstring miscounts the other sets

`rust/crates/rexx-num/tests/gen-curated-sets.py`, `signblank()` docstring: "None of the other ten sets can see this".
There are eleven other sets (addsub, addsub2, muldiv, md2, pow, cmp, fmt, fmt2, fmt3, fmtedge, fmtcarry).
The substantive claim (no other generator emits a blank after a sign) is correct.

### M2: parse doc comment cites the validity scanner, not the parser that governs

`rust/crates/rexx-num/src/lib.rs`, `Number::parse` doc: "Blank handling mirrors `numberStringScan` (`NumberStringClass.cpp:1264-1296`) exactly".
The conversion that actually governs arithmetic operands is `NumberString::parseNumber` (the state machine at `NumberStringClass.cpp:2586`, validated by `NumberStringBuilder::finish` at `:2519`); `numberStringScan` is a separate validity pre-check.
I traced every blank-bearing shape through the FSM and they agree with the Rust code (`"+ ."`, `"+ "`, all-blanks, `"3 "`, `"3 e2"`, `"3e "`, `"1e\t2"`, `"+. 5"`, sign-whitespace-point-digit), so this is citation-only, but the citation is the load-bearing reference for a port and points at the wrong function.

## Unconfirmed suspicions

* The machine-dependent middle band (around DIGITS 1e10 on this machine) may not have exactly coincident boundaries: the C++ request goes through the Rexx heap (`new_buffer`, object header and segment overhead) while the Rust probe is a bare `try_reserve_exact` through malloc.
  Both fail the same way outside the band, the implementer documented the band as unasserted, and I did not attempt to find a DIGITS value inside it that splits the two.
* `rust/crates/rexx-num/tests/muldiv.rs` comments that `1 / 7` at DIGITS 1e9 "computes and underflows (42.902)" in the oracle.
  Not verified (it needs minutes of oracle compute); if wrong it is comment-only, nothing asserts it.

## Verified clean (for the record, beyond the controller's checks)

* **H1 parse port.** Traced state-by-state against the `parseNumber` FSM; oracle probes for `'3e'`, `'3e '`, `'3.'`, `'+ .'`, `'.'`, `'+ 0'`, tab-after-sign (`'09'x` variants), and LF-prefixed strings all match the committed behavior, in both directions.
* **Settings collateral.** `unsigned_whole_number` (`settings.rs:163`) feeds NUMERIC DIGITS/FUZZ through the changed `parse`, and no signblank case covers that path, so I probed it: `numeric digits '+ 10'` sets 10, `'+'||'09'x||'10'` sets 10, `'- 10'` is 26.5, `'0a'x||'10'` is 26.5, `numeric fuzz '+ 1'` sets 1 — all matching the post-fix crate.
  Two pre-existing divergences on this path (rejecting sign-blank spellings, accepting Unicode whitespace via `str::trim`) were silently fixed by this round.
* **Compare collateral.** LF-bearing strings now reroute from the numeric path to the string fallback; `string_order` already uses the space/tab class, and oracle probes (`('0a'x||'3') = '3'` is 0, `'+ 3' = '3'` is 1, `'+ 3' == '3'` is 0, `'+'||'09'x||'3' = ' 3 '` is 1) all match.
* **H3 ordering.** The Rust `div` ordering (zero divisor, zero dividend, operand truncation, `calc_exp < 0` early returns, then the reservation) matches `NumberString::Division` (`NumberStringMath2.cpp:351-417`) line for line, and oracle probes at max DIGITS confirm each boundary: `'4.0' / '0.0'` is 42.3, `'0.0' / 2` is 0, `'0.001' // 7` is 0.001, `'0.001' % 7` is 0, `'0.001' / 7` is error 5.
* **H2.** The saturating give-up bound in `long_divide` is semantics-preserving for all reachable values, and the `u64::MAX` entry now dies at the reservation first; the saturation is correct defense-in-depth behind it.
* No new narrowing casts (the one `usize::try_from(...).unwrap_or(usize::MAX)` saturates), no rounding point moved (no arithmetic reformulation touches a rounding site), no comment dropped, zero `unsafe`.
