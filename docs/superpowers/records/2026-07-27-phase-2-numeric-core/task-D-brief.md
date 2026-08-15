# Task D — Error messages from the generated table (Phase 2, Task 2.8)

## What already exists

Error *numbers* are already carried by the enums and match the interpreter,
verified across ~100,000 differential cases:

- `ArithError` in `lib.rs` — `Overflow` and `DivideByZero` are 42,
  `NotWholeNumber` is 26
- `SettingsError` in `settings.rs` — 26, 33, 25
- `FormatError` in `format.rs` — `FORMAT`'s bad-argument error, 93; added by
  Task 2.7 after this brief was first written, so include it

One verified fact you will need, because it is easy to get backwards: a value
that is not a number raises **93** when it reaches `FORMAT` or `TRUNC` as an
argument, not the 41 that a bad numeric *literal* in source text raises. Both
builtins give 93 for `abc` and for an exponent below the representable range
alike. Choosing between them is the caller's job, not `Number::parse`'s. A
harness in this repo had it wrong until 2026-07-27; do not copy 41 from
anywhere without re-checking it against `build/bin/rexx`.

What does not exist is the error *text*.

## What to add

The `rexx-inventory` crate already generates the complete message table from
`interpreter/messages/rexxmsg.xml` at build time: 704 messages keyed by
`(major, sub)`, with `symbol` and `text` fields, verified against the
interpreter's own generated header with zero mismatches.

Wire the numeric errors to that table so a raised error can produce the
interpreter's exact message text, substitutions included. Do not hand-write
message strings — that is the whole point of the generated table.

Note the interpreter distinguishes sub-messages: error 42 alone is
"Arithmetic overflow/underflow", while 42.903 is "Arithmetic underflow; zero
raised to a negative power". Find which sub-message each raise site should
use by provoking it in the interpreter and reading what it prints.

## Substitutions

Message text carries `&1`, `&2` … placeholders. Provide a way to fill them.
Rendering rules are already implemented in `rexx-inventory`'s build script
and documented there.

## Verification

For each error the crate can raise, write a Rexx program that provokes it,
run it under `build/bin/rexx`, and confirm the message your code produces
matches what the interpreter prints — text and sub-number both.

    build/bin/rexx yourprobe.rex

## Constraints

- `rexx-num` must depend on `rexx-inventory` as a path dependency.
- Do not modify `rexx-inventory`'s build script or generated output.
- Do not weaken any existing error *number*; they are verified and correct.
