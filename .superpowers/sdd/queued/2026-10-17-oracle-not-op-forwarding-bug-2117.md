**NOT A DIVERGENCE, ruled by Moritz 2026-10-18.** upstream report SF #2117; probe: the oracle answers 1 0 and the crate the right 0 1.

# Oracle bug SF #2117: `\` on an object forwarding to .true/.false is inverted

Measured 2026-10-17, fresh empty dir, oracle `build/` vs 9b1 close binary
(`ooRexx-p9-pin/rexx-run-9b1-d58e12600`). An object whose UNKNOWN forwards to `.true` or `.false`:

    t = .fwd~new(.true); f = .fwd~new(.false)
    say (\.true) (\.false)      -- oracle 0 1   crate 0 1
    say (\t) (\f)               -- oracle 1 0   crate 0 1
    say t~'\' f~'\'             -- oracle 1 0   crate 0 1
    say (t & 1) (f & 1)         -- both 1 0
    ::class fwd
    ::method init; expose v; use arg v
    ::method unknown; expose v; use arg m, a; forward to (v) message (m) arguments (a)

The oracle's `RexxInteger::notOp` path answers the wrong value (oorexx-devel 2026-10-17, Josep
Maria: "RexxInteger::notOp lacks a !"; bug #2117). The crate is right; not copied (Moritz: we
don't keep oracle bugs). A corpus witness cannot go in the oracle differential as is: it needs the
corpus's oracle-bug exemption mechanism, or a crate-only test asserting `0 1`. Owner: whoever next
touches the corpus exemption list; low priority.
