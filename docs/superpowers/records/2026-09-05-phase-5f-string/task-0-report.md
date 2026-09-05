# Phase 5f Task 0 — the module, the wiring, and the receiver check

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`.
BASE `b6d7d1af1`. Landed at **`GATECOMMIT`**.

**No method was added and no behaviour changed.** `crates/rexx-exec/src/dispatch/string.rs` holds an
empty `NATIVE_METHODS` slice, chained into `ObjectModel::build` beside `dispatch.rs`'s own.

## 1. The chain, shown to work by two controls

An empty slice cannot witness its own registration, so both steps were run and both predictions were
written down first.

**Control A — a row naming a method the class does not answer must panic.** Predicted: a panic at the
registration loop naming `String~ZZZPROBE`. Measured, `("String", "ZZZPROBE", Arity::Fixed(0),
native_reverse)`:

```text
thread 'rexx-interp' panicked at crates/rexx-exec/src/dispatch.rs:1203:21:
NATIVE_METHODS names String~ZZZPROBE, which that class's behaviour does not answer
rc 101, both engines
```

**Confirmed**, and it is why the control cannot use a made-up name for step B.

**Control B — a real loud name bound to the wrong body must answer wrongly.** Predicted: `'abc'~lower`
answers `cba`. Measured, `("String", "LOWER", Arity::Fixed(2), native_reverse)`:

```text
REXX_ENGINE=ir           rc 0   stdout cba   stderr empty
REXX_ENGINE=tree-walker  rc 0   stdout cba   stderr empty
```

**Confirmed.** That is the natives map populated from the new slice and a send reaching it, which
control A alone does not show.

**Both removed, and the send is loud again**, rebuilt rather than assumed:

```text
rc 120, stderr `rexx-exec: method "LOWER" of class "String" is not implemented (Phase 5)`, both engines
```

## 2. The receiver measurement

The 5c follow-up review's second open question — does a documented receiver make rows unfalsifiable,
and how wide is it — sending every documented instance name with no arguments to two receivers of the
same class and diffing the answers.

| class | rows | discriminate | identical at any receiver |
|---|---|---|---|
| `String` | 112 | **21** | 91 |
| `Queue` | 43 | 10 | 33 |
| `Array` | 44 | 11 | 33 |
| `List` | 38 | 7 | 31 |

`String`'s 21, under `.String~new('abc')` against `.String~new('')`: `b2x bitAnd bitOr bitXor c2d c2x
decodeBase64 encodeBase64 hashCode length lower makeString reverse space strip translate upper words
x2b x2c x2d`. **So `String` needs no `RECEIVER_OVERRIDES` entry**; its documented receiver already has
teeth on a fifth of the surface.

For the three empty collections a populated receiver would sharpen the rows listed, and **no receiver
can sharpen the rest**: a zero-argument send to a method that needs arguments never reaches the
receiver's contents. That is the answer to how wide the question is, and it is narrower than the
review supposed.

**The instrument was wrong twice before it was right, and both are worth carrying.**

* The first sweep shared one receiver across every probe, so `empty` and `pull` — documented methods
  taking no arguments — drained the queue for every later row. It reported `.Queue~of('a','b')~items`
  as 0 and every row as identical. A fresh receiver per probe is what fixed it, and the tell was that
  the same construction answered 2 when probed alone.
* The second sweep was correct and `diff` reported nothing for `String`, because `x2c` and `d2c` put
  NUL bytes in the output and `diff` fell back to "Binary files differ". `diff -a` gives 21 where the
  default gave 0. Same shape as `grep`'s silent binary skip, in a tool nobody had it written down for.

## 3. What else this task found

`say "abc"~"<<"` is a silent SIGSEGV on the oracle, with `<<=`, `>>`, `>>=`, `\<<` and `\>>`.
Recorded as `corpus/oracle-crashes.txt`'s newest entry at `ce94e245d` with its cause in
`classes/StringClass.cpp`, and carried into the plan as a blocker Task 2 decides before it writes any
of the six bodies.

## 4. Gates

| | |
|---|---|
| G1 `cargo fmt --all --check` | rc 0 |
| G2 `cargo clippy --workspace --all-targets -- -D warnings` | rc 0, zero warnings |
| G3 | **G3** |
| G4 | **G4** |
| G5 | **G5** |
| G6 | **G6** |
| G7 | **G7** |
