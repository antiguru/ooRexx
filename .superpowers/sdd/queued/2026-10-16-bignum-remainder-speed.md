**DEFERRED to Phase 9.1, ruled by Moritz 2026-10-18.** probe: output 78 on both, 2000 iterations under 1 s on the oracle and over it here.

# Large-digit `*` and `//` are about 33x slower than the oracle

Found 2026-10-16 by an external-corpus probe (rx2cpp `bignum_modmul.rex`, scratch only). Controller
re-run, 9b1 close binary, same output (78) on both:

    numeric digits 2000
    p = 2**255 - 19
    x = 2**250 + 12345
    y = 2**251 + 67890
    do i = 1 to 2000
      z = (x*y)//p; x = (z+i)//p; y = (y-z)//p
    end
    say length(x)

Oracle 0.10 s, crate 3.44 s wall. At 20000 iterations the probe saw 1.1 s vs 35.7 s. No bench
program covers digits in the hundreds; the perf budget cannot see it. Profile first (callgrind on
a reduced count) before guessing whether `*` or `//` dominates.
