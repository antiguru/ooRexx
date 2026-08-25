# Task 18: what the controller measured before dispatch

Both discriminators re-measured at BASE `dd83eecdc`, fresh directory, absolute paths, three
descriptors read separately, both sides bounded. Both reproduce. **They fail in two different
ways, and the brief does not say so.**

## The `INIT`/`ACTIVATE` pair is a SILENT WRONG ANSWER today

    oracle   rc 0   stdout: K init,     hasMethod MM = 0
                            K activate, hasMethod MM = 1
                            prologue
    crate    rc 0   stdout: prologue          (both engines, stderr empty)

**rc 0 on both sides, stdout differing.** The crate does not refuse this program -- it runs it,
exits clean, and silently never fires `INIT` or `ACTIVATE` at all. That is this project's worst
failure mode, not a loud refusal, and it means the corpus row for this discriminator is load-bearing
in a way a `NOT_IMPLEMENTED_EXIT` row would not be: nothing else in the tree can notice this.

## The constant forward reference is a wrong raise, not a refusal

    oracle   rc 0    stdout: from B
    crate    rc 159  stderr: Error 97.1:  Object "The B class" does not understand message "M".
                             blaming `3 *-* ::CONSTANT c (.B~m)` then `4 *-* ::CLASS B`

Both engines identical. The crate resolves the constant while installing `A`, before `B` exists,
so it raises where the oracle answers. This is the second pass's absence, exactly as the brief says.

## What this means for the two required controls

The brief asks for two: firing `ACTIVATE` before the merge must redden the first program, and
resolving constants inside the class pass must redden the second. **Run both mutations and read the
corpus result, do not argue them.** A control that cannot fail is worth nothing, and a green run
cannot witness one -- this plan has shipped that mistake in several tasks.

## Prerequisites confirmed in the tree

Task 7's `MIXINCLASS`/`INHERIT` landed; the `self~init:super` scope-override send the brief
substitutes for `FORWARD` is the old plan's Task 5 and is present. `FORWARD` itself is 5b's and the
spec's own program is therefore unreachable in 5a -- the substitution is what makes this runnable,
and the brief's transcript is the one to match.

`M`'s method must carry `CLASS`: `self` in a class-side `init`/`activate` is the class object, so a
plain `::METHOD mm` makes both lines read `0` and the transcript discriminates nothing.
