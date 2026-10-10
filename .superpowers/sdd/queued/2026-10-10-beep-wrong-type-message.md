# BEEP's wrong-type argument message differs

Found by the Phase 6.1 final review (minor 6, `probes/s2/beep_msg.rex`); pre-existing at the 6.1 base. Queued by the Phase 6.1 final fix (2026-10-10). Same rc and code; the message and the `running` line differ.

The oracle checks the whole-number range before the frequency range. `whole_argument`'s own doc says the non-numeric answer was not measured.

Probe `beep.rex`, run from a fresh empty directory:

    say beep(.object~new)

Oracle, rc 168:

    [stdout]
    (empty)
    [stderr]
           *-* Compiled routine "BEEP".
         1 *-* say beep(.object~new)
    Error 88 running REXX:  Invalid argument.
    Error 88.907:  Argument 1 must be in the range -999999999999999999 to 999999999999999999; found "an Object".

This crate (`rexx-run` built from the final fix's finding 1 tree), rc 168, `PROGRAM` standing for the program's path:

    [stdout]
    (empty)
    [stderr]
           *-* Compiled routine "BEEP".
         1 *-* say beep(.object~new)
    Error 88 running PROGRAM line 1:  Invalid argument.
    Error 88.907:  Argument frequency must be in the range 37 to 32767; found "an Object".

Suspected site: `builtin/platform.rs`, `beep` and `whole_argument`, and the delivery flag that makes a native routine's error line read `running REXX:`.
