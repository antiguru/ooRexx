# Task 9 -- the real 98.972, and `String~MAKEARRAY`

**BASE:** `76b61b3e8`. **Committed at `64258d712`.** Run by the controller rather than an
implementer: two agents were dispatched for this task and both ended on `API Error: 529 Overloaded`
before doing any work, so a third dispatch was not attempted.

---

## Part 1 -- 98.972 is raised where the oracle raises it

`Raised::lostdigits` replaces `Loud::lostdigits_option`, which is deleted.

**The brief was wrong about the substitution and measuring settled it.** It said the operand's
rendering, from `string_value_text` on the parsed number. The oracle carries the operand's **bytes**,
measured at DIGITS 3 from a fresh directory:

```
1.23456789      ->  Number 1.23456789 has more digits than the current precision.
0001.23456789   ->  Number 0001.23456789 ...        leading zeros kept
1.234567890     ->  Number 1.234567890 ...          trailing zero kept
1.23456789e2    ->  Number 1.23456789E2 ...         not 123.456789
'1.23456789'    ->  Number 1.23456789 ...
```

A re-rendering through `Number::format` would have been wrong on three of the five: `format(digits)`
rounds first and then chooses plain or exponential form from `digits`, so `1.23456789e2` renders
`123.456789`. The `e` becoming `E` is not a normalisation this crate has to perform -- a numeric
literal is a symbol and tokenising uppercases it, which is why the quoted spelling is unchanged.

So the check takes the operand's `ObjRef` alongside its `Number` and reads
`string_value_text` **only on the raising path**. The hot path is one already-hot bool exactly as
before; the byte fetch is last inside the `#[cold]` arm because it is the only step that allocates.

**A fifth call site the earlier report did not list** -- `header_number_body`, the controlled `DO`
header's `Initial`/`TO`/`BY` -- was found by the compiler, not by reading.

**Verified**, both engines, three descriptors separately, from a fresh empty directory: eight shapes
byte-identical, covering the raise, `1 + 1` staying clean, the left operand blamed first, the
`CONDITION` spelling agreeing at rc 0, and the `DO ... TO` header.

## Part 2 -- `String~makeArray` answers the receiver's lines

**Not the one-item array the plan assumed.** Measured on the oracle:

| receiver | items | first |
|---|---|---|
| `'abc'` | 1 | `abc` |
| `''` | **0** | -- |
| `'  '` | 1 | `  ` |
| `'a' LF 'q'` | 2 | `a` |
| `'a' LF` | 1 | `a` |
| `LF 'a'` | 2 | empty |
| `'a' CRLF 'q'` | 2 | `a` |
| `'a' CR 'q'` | 1 | `a`CR`q` |
| `'a' LF LF 'q'` | 3 | `a` |

So: LF separates, a trailing LF terminates rather than separating, a CR immediately before an LF
goes with it while a lone CR is data, and the empty string has no lines at all.

Eleven shapes byte-identical on both engines, `~request('ARRAY')` among them -- which is why
`dispatch.rs`'s `a_conversion_this_phase_does_not_model_is_loud_where_the_ones_it_models_answer`
lost its `String` row. Before removing it the crate's new answer was checked against the oracle
rather than assumed: `say 'abc'~request('ARRAY')` is `abc` on both sides, as is `say` of the array
itself.

## What I did not do

**`~unknown`'s argument list still refuses**, and deliberately. It converts through
`unconverted_array_argument`, which does not send `MAKEARRAY`. `native_request` shows the shape the
fix takes -- `lookup` then `send_message` -- but `array_argument` then tests
`is_multi_dimensional_array` on the **original** value, and a correct fix moves that subject onto the
converted array. That is the argument-conversion surface this phase deferred twice, and doing it at
the close is how a silent wrong answer ships. Transcript: `vr = 5 ; o = >vr` under `signal on
syntax`, `say o~unknown('LENGTH','notanarray')` is rc 120 both engines against the oracle's rc 0
`end`.

**I did not close the corpus-coverage gap I found** (below) -- it belongs to whoever writes
`corpus/phase-5c.txt`.

## Witnesses, both shown to fail at BASE

BASE built in its own extract with its own `CARGO_TARGET_DIR`, binary sha256 `059910e537d8`:

* `corpus/lang/string_makearray.rex` -- **rc 120 at BASE**, rc 0 and byte-identical here.
* the converted lostdigits table -- **FAILED at BASE**, `plus on Ir: stderr differs from the oracle`.

## Three defects of my own, and how each was caught

* **An insertion orphaned a doc block.** Anchoring the new function on `"fn native_reverse(\n"` put
  it between `native_reverse`'s doc comment and `native_reverse`, so that comment became the first
  line of the new function's block and `native_reverse` was left bare. **`fmt` and `clippy` both
  passed.** Caught by Moritz reading the diff. A doc comment attaches downward, so a `fn` line is
  *inside* the item -- anchor on the blank line above the doc block, or re-emit the following block
  explicitly as this task's `error.rs` insertion did.
* **The witness was not running.** `corpus/lang/` holds 342 programs; the differential runs the 331
  named in phase subset files, and a program in neither is silently unrun. Caught by the corpus
  headline staying at `331 of 331` after a program was added. Fixed with a binary of its own, the
  same answer `variable_reference.rs` and `stem_object.rs` use, since `corpus/phase-5c.txt` cannot
  exist before the flip. **The gap itself is not fixed** and is recorded in the plan.
* **Two probe-syntax errors**, both traps `rust/CLAUDE.md` names: a variable called `b` before a
  quoted string parsed as a binary literal, and bracket-quoting parsed as a routine call. Both
  showed up as the crate and the oracle failing *identically*, which is the tell that the program
  rather than the implementation is wrong.

## Gates

At tree hash `cbc29697834702ea`, unchanged before the first and after the last: fmt 0, clippy 0,
release 0, release+corpus 0, debug+corpus+memcap 0, each with `--no-fail-fast` and zero
`test result: FAILED`. The `ok` binary count went 109 to 110, the one being the new witness --
which is also the evidence that it runs. Corpus **331 of 331**. Table C's 5c count **110** and table
D's **2**, neither moved, as a task closing divergences rather than rows should not move them.

The phase gate is rc 101, which is the expected pre-flip state and not a result of this task.
