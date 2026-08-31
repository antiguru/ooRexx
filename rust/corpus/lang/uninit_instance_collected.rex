/* An instance whose class defines UNINIT becomes garbage and a driven
   collection reaches it, so the finalizer runs at the collection rather than
   at termination.  The clauses between ~new and DROP are what let the
   collection reach it: the oracle holds recently allocated objects out of a
   driven collection (Memory::SaveStackSize), and measured, eight allocating
   clauses in their place leave the finalizer for the termination sweep. */
say 'start'
o = .K~new
say 'built' o~class~id
pad = 'keep the new object out of the last ten allocations'
drop o
call gc 'force'
say 'after-gc'

::class k
::method uninit
  say 'uninit ran'
