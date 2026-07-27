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
    a[j] = "e" || j     /* concatenate: a DISTINCT string per slot. A bare
                            literal would be one interned object shared by all
                            1M slots, making the graph ~1001 objects. */
  end
  outer[i] = a
  root["K" || i] = a
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
