/* Allocation-churn dimension, restricted to the 4c surface (no message sends).

   `alloc.rex` allocates a fresh `.array` and `.string` every iteration via
   `.array~of`, `.string~new`, `~size` and `~length` -- all message sends, and
   this crate implements none yet (`rexx-exec: a message send is not
   implemented (Phase 5)`; confirmed 2026-08-08 on `.array~of(1,2,3)` alone,
   on `.string~new("item")` alone, and on the combination, all three the same
   error). Allocation throughput does not need the object model, so this
   program covers the same dimension with constructs this crate has: `||`
   concatenation and compound-variable creation.

   What this preserves from alloc.rex: two heap-allocating operations per
   iteration (there, one array plus one string; here, one new compound
   variable plus one concatenated string), and an accumulator whose two
   terms (a constant 3, standing in for the array's fixed size, and a
   string's length) stay in tagged-small-integer range for the whole run, so
   neither the accumulation nor the loop-bound arithmetic itself allocates --
   exactly as alloc.rex's `a~size`/`s~length` additions do not.

   What this does NOT preserve: alloc.rex's array and string are both
   rebound to the same loop-local variable every pass, so on a collector that
   actually swept, they would be garbage before the next iteration starts
   ("none of them retained past the iteration that made them", per that
   file's own header). `tab.i` here is a NEW tail every pass -- tab.1,
   tab.2, ..., tab.n -- so it is a genuinely live, growing table on any
   interpreter, oracle included; it is not collectible churn. This axis
   therefore measures raw allocation throughput, not collection pressure, and
   does not exercise whatever forces a collection in alloc.rex's own design
   intent ("sized to force multiple collections"). It is not a substitute for
   a collection-forcing measurement once message sends land; it is the piece
   of that axis this crate can run today. */
n = 1000000
total = 0
do i = 1 to n
    tab.i = i
    s = "item" || i
    total = total + 3 + length(s)
end
say total
