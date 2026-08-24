/* Task 21, fix round 1: `.context` answers one object per activation, which
   is what `~objectName=` and `~identityHash` read.

   The rename row is the one a fresh-object-per-evaluation build cannot pass
   at all: it stores the name on a throwaway and the next `.context` answers
   the default. The identity rows say the same thing without a store, and the
   last one says the object belongs to the activation rather than to the
   program -- the method's `.context` is not the caller's, so passing the
   caller's in and comparing gives 0.

   The loop between the first two rows is deliberate: it allocates enough to
   run the collector, so the row after it also says the held object survived
   a collection. `collect_stress.rs` runs this program with the collector on
   every allocation, which is what makes that half real rather than lucky. */
c = .context
do i = 1 to 300
  s = copies("x", i)
end
say (c~identityHash == .context~identityHash)

.context~objectName = "tagged"
say .context~objectName
say .context~string
say .context

say .K~compare(c)

::class K
::method compare class
  use arg outer
  return (outer~identityHash == .context~identityHash)
