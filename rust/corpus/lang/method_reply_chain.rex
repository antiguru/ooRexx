/* The rest of a replied method sends a message whose method replies in its
   turn: both remainders run and neither is lost. Phase 5a Task 16. */

say .K~outer

::class K

::method outer class
  reply 'outer-replied'
  say 'outer-tail:' self~inner
  return

::method inner class
  reply 'inner-replied'
  say 'inner-tail'
  return
