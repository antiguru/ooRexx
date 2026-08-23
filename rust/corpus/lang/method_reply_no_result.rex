/* A bare REPLY answers the sender with no value at all, so an expression
   position reading the send is 91.999 exactly as a bare RETURN there is. The
   rest of the body still runs: `tail` is on stdout beside the sender's own
   failure, which is what says the remainder is not cancelled by the program
   that asked for it failing. Phase 5a Task 16. */

say '[' .K~m ']'

::class K

::method m class
  reply
  say 'tail'
  return
