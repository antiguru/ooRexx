/* GUARD is legal only in a method invocation. A ::ROUTINE is not one, so this
   is 99.911 -- and the traceback carries the routine's own clause and then the
   CALL that reached it, which is what makes this the wider of the two shapes
   the check has. Phase 5a Task 16. */

call sub

::routine sub
  guard on
  return
