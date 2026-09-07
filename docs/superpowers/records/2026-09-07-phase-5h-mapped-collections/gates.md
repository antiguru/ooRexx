# Phase 5h — gate readings

## Task 1, over `ce0f4c32e`

G1 0, G2 0, G3 0, G4 0, G5 0, G6 0, G7 0, `failed-suites=0` on each of the
five suite-running gates (`scratchpad/gates-h1.status`).

## Tasks 2 and 3, over `969a74209`

G1 0, G2 0, G3 0, G4 0, **G5 101**, G6 0, G7 0, with `failed-suites=1` on G5
alone (`scratchpad/gates-h3.status`).

**G5's failure is a flaky console test and not this phase's.** The suite that
failed is `input_oracle`'s
`command_line_arguments_and_the_console_agree_with_the_oracle`, whose cases
are `PARSE PULL` and `PARSE LINEIN` reading the console -- nothing a
collection touches. Re-run alone, G5 is exit 0 with no failing suite and the
corpus at 434 of 434.

Five readings of that test across this phase: red in one release run, green in
the next, green in the run after that, red in G5, green in G5's retry. It goes
red under parallel load and green on its own, and both of its failures had the
ORACLE reading a different amount of stdin than it does alone -- once a single
byte where it reads a whole line. So the flake is in the harness's console
handling rather than in either interpreter.

**One retry did not run at all** and has to be discounted rather than read: the
shell's directory had reset, so `cargo` reported `could not find Cargo.toml`
and exited 101 -- which looks exactly like the failure it was meant to check.
The reading above is from the run after that, launched with an explicit
directory.
