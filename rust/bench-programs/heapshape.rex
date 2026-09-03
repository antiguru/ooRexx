/* D1 measurement, C++ side: build a ~1M-object graph of the same shape the
   Rust benchmark builds, then time a FORCED full collection.

   GC('F') calls memoryObject.collectAndUninit (BuiltinFunctions.cpp:3031),
   so the second number is a real full-GC pause and is directly comparable
   with the Rust figure. The build time is NOT comparable -- it includes
   parsing, dispatch and variable lookup that the Rust microbenchmark never
   touches -- which is why it is reported separately rather than netted out. */

outer = .array~new(1000)
root = .directory~new

t0 = time('R')
do i = 1 to 1000
  a = .array~new(1000)
  do j = 1 to 1000
    a[j] = "element-" || j   /* concatenate: a DISTINCT string per slot. A bare
                            literal would be one interned object shared by all
                            1M slots, making the graph ~1001 objects.

                            WIDER THAN SEVEN BYTES, which is the same collapse
                            reached a second way: a string of up to
                            rexx_core::INLINE_TEXT bytes lives in the Rust
                            handle and allocates nothing, so "e" || j builds a
                            ~1001-object graph on that side while the oracle
                            builds ~1,001,001 either way. Measured 2026-09-03,
                            9 interleaved pairs a variant: with "e" || j the
                            forced pause is 0.002113 s on the Rust side against
                            the oracle's 0.016945, and with these widths it is
                            0.014672 against 0.017411. The oracle's own figure
                            barely moves, which is what says whose graph
                            changed. */
  end
  outer[i] = a
  root["KEYNAME-" || i] = a
end
build = time('E')

/* 10% cross-links, so the graph is not a pure tree */
do i = 1 to 100
  outer[i][1] = outer[1001 - i]
end

t0 = time('R')
forced = gc('F')
pause = time('E')

say "build_seconds=" build
say "gc_forced=" forced
say "gc_pause_seconds=" pause
