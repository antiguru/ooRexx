# A message send in a builtin's argument list corrupts that builtin's arguments

**Found during Phase 5i Task 3, 2026-09-08. Not Phase 5i's defect and not fixed here.** It predates
the phase: every transcript below was taken against the gate worktree's binary, built at Task 2's
commit `3d2c7dd75`, and Task 3's implementer independently confirmed it at its own BASE by restoring
its files from `git show HEAD:`, rebuilding `--release`, and running the repro with no `RexxInfo` in
it.

## What it is

**A message send anywhere in the argument list of a builtin that takes more than one argument
corrupts that call's argument run** -- into a wrong `40.3`, or into a panic that aborts the
interpreter.

The first report of it was "a builtin call nested in another builtin's argument list loses its own
argument". Case 7 below is smaller and shows that is not the shape: there is no nested builtin in it
at all.

## Transcripts

`v = 'hello world'`, oracle answers every line at rc 0; both engines identical to each other.

```
                                            oracle        this crate
1  say length(.Array~id)                     5             5
2  say right(v, length('abcde'))             world         world
3  say right(v, length(v))                   hello world   hello world
6  say length(length(.Array~id))             1             1

7  say right(v, .Array~id~length)            world         40.3 "in invocation of RIGHT;
                                                                 minimum expected is 2"
5  say right(v, 5 + length(.Array~id))       ello world    40.3 LENGTH
8  say right(v, length(.Array~id || ''))     world         40.3 LENGTH
4  say substr(v, 1, length(.Array~id))       hello         PANIC, rexx-interp aborts
```

Case 4's panic: `crates/rexx-exec/src/run.rs:6151`,
`range start index 2 out of range for slice of length 1`.

Case 1 works only because `length` takes one argument.

## The mechanism, read from the site

`Interp::run_over_pushed_args` (`crates/rexx-exec/src/run.rs:6145`-`:6155`):

```rust
let mut values = std::mem::take(&mut self.value_buffer);
let outcome = body(self, &values[mark..]);
values.truncate(mark);
self.value_buffer = values;
```

Its doc says the empty buffer left behind "is exactly what a callee pushing runs of its own should
start from". **That is true for a callee and false for a message send evaluated while an outer
argument run is still half-built.** The send's own `run_over_pushed_args` takes the buffer holding
the outer call's partial run and computes its mark against the empty replacement, so the restore
truncates the outer's arguments away -- or the mark exceeds the length and the slice panics.

## Why no gate saw it

Both engines agree with each other, so `ir_dual` is silent. No `corpus/lang/` program calls a
multi-argument builtin with a message send among its arguments, so the corpus differential is
silent. This is `oorexx-rust-rewrite`'s recurring shape: a criterion quantifying over one set does
not cover that set's product with another.

## Destination

The Phase 4 executor owns it. A panic reachable from a two-line program that the oracle answers at
rc 0 is more severe than anything Phase 5i is closing, and the decision whether to fix it before
Phase 5i closes is Moritz's.
