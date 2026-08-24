/* Task 21, fix round 3: an activation parked by REPLY keeps its context
   object across a collection taken while it is off every stack.

   That route -- `park_reply` into `RootSet::park` into
   `Activation::object_roots` -- is the one the running and suspended rows
   cannot reach, and it had no differential row: no corpus program contained
   both REPLY and `.context`, which is the same disjoint-sets shape that let
   the forced-collection hole through.

   The parked body prints nothing when it is right, and that is deliberate.
   The oracle runs a replied-to body on another thread, so anything it says
   races with the main line's own output: measured, a version whose parked
   body said its context unconditionally gave two distinct outputs over
   twenty oracle runs. Reading the context and saying nothing is
   order-independent, and it still discriminates in both directions -- a
   collected object refuses the send outright, and a re-minted one answers
   the default and takes the LOST branch. */
say .K~inner()
say gc('force')
do i = 1 to 200
  s = copies("q", i)
end
say gc('force')
say "main done"

::class K
::method inner class
  .context~objectName = "parked"
  reply "replied"
  if .context~objectName \== "parked" then say "LOST" .context~objectName
