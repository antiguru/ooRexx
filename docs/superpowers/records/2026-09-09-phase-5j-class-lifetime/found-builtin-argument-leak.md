# A message send inside a builtin call leaks its arguments into the call

**Found 2026-09-09 while deriving the Phase 5 close's gap list**, not by looking for it. The
method-body table's only two `diverge` rows are `DateTime~date` and `DateTime~timeOfDay`; both trace
to this.

## The repro

```rexx
say length('abc'~copies(1))
```

| | answer |
|---|---|
| oracle | `3` |
| crate, `ir` | `Error 40.4:  Too many arguments in invocation of LENGTH; maximum expected is 1.` |
| crate, `tree-walker` | the same |

Deterministic, both engines, rc differs.

## What it is

The inner send's arguments end up in the outer builtin's argument list. `LENGTH` is given two
arguments — the send's result and the send's own `1`.

## The shape, measured rather than assumed

| program | result |
|---|---|
| `substr(s, 2, 3)` | agrees — no send |
| `substr(s~copies(1), 2, 3)` | diverges; `SUBSTR argument 2 ... found "abcdef"` |
| `substr(s~copies(1), 2)` | diverges |
| `word('a b c'~copies(1), 2)` | diverges |
| `length(s~copies(1))` | diverges — **one** argument, so this is not misordering |
| `length(s~substr(1))` | diverges |
| `length(s~upper)` | **agrees** — the send takes no arguments |
| `length(.string~new('abc'))` | diverges — a class-side send with an argument |
| `r(s~copies(1), 2)` where `r` is a Rexx routine | **agrees** — not the builtin path |

So it needs a *builtin* call, and a send in an argument position that itself passes arguments. A
user-defined routine in the same position is clean, and a no-argument send is clean.

`d~standardDate` — no arguments — also diverges inside `DateTime~date`, which does not fit "the send
passes arguments". That case is a Rexx-coded library method rather than a native one, so the
condition is probably "the inner send disturbs the argument stack", of which passing arguments is
one way and running a Rexx body is another. **Stated as unresolved rather than asserted**: the two
clean cases and the seven diverging ones are what was run, and the mechanism is not.

## Where to look

`Interp::value_buffer` is documented as "**A stack, and the only one**: every call's arguments,
innermost run on top… What is lent is the whole stack and what comes back has the caller's own run
removed, so a callee that pushes runs of its own starts from an empty one." The design anticipates
exactly this hazard, so the defect is in that lending discipline on the builtin path rather than in
the idea.

## Why nothing caught it

It is not an object-model gap and no Phase 5 row is about it; it is the Phase 4 builtin argument
path, reachable only when a builtin's argument is a send. The method-body table found it because it
*sends every documented method name* and the library's own Rexx bodies use the idiom. The
differential corpus did not, which is a coverage-shape finding about the corpus, not about this bug.

---

# Fixed 2026-09-09

`take_value_buffer` no longer clears, and `give_value_buffer` truncates to the caller's depth —
the mark discipline `Interp::run_over_pushed_args` already used. Two call sites, both sends:
`dispatch.rs`'s message send and `run.rs`'s `FORWARD`.

`corpus/lang/builtin_send_argument.rex` is the witness, filed in `corpus/phase-5j.txt` so the
differential runs it against the live oracle. It separates the parts: a send with arguments, one
without, one that is not the first argument, one whose result the builtin must convert, and a user
routine in the same position, which never went through this path and agreed throughout.

**The method-body table now has no `diverge` rows at all** — 1192 `answers`, up from 1190, and the
two `DateTime` rows that led here are `answers [rc 0]`. `regressions this run: 0`.

## The controls, and one of them was not clean

**B — `give_value_buffer` stops truncating.** Predicted: the witness reddens on line 1,
`length(s~copies(1))`, the original repro. **Confirmed**, with the original message:
`Error 40.4: Too many arguments in invocation of LENGTH; maximum expected is 1.`

**A — `take_value_buffer` clears again.** Predicted: the witness reddens on
`substr(s, s~length - 4, 3)`, the one line with arguments pushed before the send. **Falsified as
stated.** It reddens, but by panicking — `range start index 1 out of range for slice of length 0` —
because restoring the clear while keeping the mark leaves the two inconsistent. So A varies two
things at once and is not a single-variable control: it shows the halves are coupled, not that the
clear alone produced a wrong answer.

The honest summary is that B isolates the residue half and nothing here isolates the loss half. A
control for that would have to restore the whole pre-fix shape, and its evidence is the original
measurement instead: before the fix, `substr(s~copies(1), 2, 3)` reported argument 2 as `"abcdef"`,
which is a four-argument list whose first entry is the send's `1` — both halves at once.
