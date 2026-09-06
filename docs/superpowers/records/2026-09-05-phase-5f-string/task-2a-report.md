# Phase 5f Task 2a — a verdict for a row the oracle cannot be asked about

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, D87.
BASE `009230209`. Landed at **`GATECOMMIT`**.

**Lands alone, ahead of any operator body, and moves no row.**

## 1. What it is for

Six of Task 2's rows -- `<< <<= >> >>= \<< \>>` -- segfault the oracle when sent with no arguments,
and `method-bodies.txt` classifies every row by exactly that send. They are `loud` today and reach
that verdict in the crate-only pass, so nothing runs the oracle on them; the first commit that binds
any of the six would send the refresh to the oracle and take the crash.

`Body::Uncomparable` is the verdict for that: **this crate answered and the oracle cannot be asked.**
Not `Unanswered`, which means the send never reached the method -- here it reaches it and answers,
and what is missing is the other side.

`ORACLE_CRASHING_SENDS` names the six, each an instance of `corpus/oracle-crashes.txt`'s
strict-ordering entry. It names a property of the oracle rather than of this tree, so it does not go
stale when a phase lands, and `check_crashing_sends` asserts every pair it names is a row that
exists -- the guard `check_overrides` provides one table over.

**The classification order is crate-loud first, then the list, then the oracle.** A row this crate
still refuses stays `loud`, which is the more informative verdict. That is what lets this land while
it moves nothing.

## 2. The control, in two halves

The second half alone would prove little: `uncomparable` could be a verdict the run reaches for some
other reason. So `String~"<<"` was temporarily bound to an existing body -- enough to make the crate
answer rather than refuse, which is all it takes to push the classification past the crate-loud
branch -- and the refresh run twice.

**Half A, `ORACLE_CRASHING_SENDS` emptied.** Predicted: the refresh runs the oracle, takes the
segfault, and reports a structural failure. Measured, rc 101:

```text
String << (instance): the oracle did not finish: Signaled. Whatever it left on its
descriptors is where it was interrupted, not an answer to compare
```

**Half B, the list as committed.** Predicted: the row classifies without the oracle being run.
Measured, rc 0, and the refreshed table carries

```text
String	<<	instance	uncomparable	the oracle segfaults on this send
```

Both halves as written down beforehand. The temporary binding and the table were then restored, and
`cargo test --release -p rexx-exec --test method_bodies` passes with the committed table unchanged.

## 3. `regressed` gained eleven arms and no rule

The match is one literal arm per ordered pair, so a new verdict is eleven new arms the compiler
demands. `Uncomparable` is read the way `Unanswered` is: `answers` -> `uncomparable` is a row losing
its evidence and so a regression, nothing else into it is, and out of it nothing is -- the row never
carried a claim about the oracle. It cannot reach `diverge`, which needs two sides.

## 4. Gates

| | |
|---|---|
| G1 `cargo fmt --all --check` | rc 0 |
| G2 `cargo clippy --workspace --all-targets -- -D warnings` | rc 0, zero warnings |
| G3 `cargo test --release --workspace --no-fail-fast` | **G3** |
| G4 | **G4** |
| G5 | **G5** |
| G6 | **G6** |
| G7 | **G7** |
