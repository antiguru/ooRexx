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
   of that axis this crate can run today.

   One further asymmetry, disclosed rather than left for a reader to notice:
   alloc.rex's string is a fixed four characters ("item"); this program's
   grows with `i`, from five characters at i=1 to eleven at i=1,000,000
   (four plus up to seven digits). Judged immaterial at this size, checked
   rather than assumed: `do i = 1 to 500000; s = "abcde"; end` and the same
   loop with an eleven-character literal peaked at 69,108 KB and 68,812 KB
   resident respectively on this crate (2026-08-09) -- no measurable
   per-iteration cost from the extra six bytes, consistent with both
   lengths landing in the same allocator size class well under the 96-byte
   `Heap::Slot` overhead that dominates a short string's retained cost
   (`docs/superpowers/plans/phase-4d-retention.md`). */
n = 1000000
total = 0
do i = 1 to n
    tab.i = i
    s = "item" || i
    total = total + 3 + length(s)
end
say total
