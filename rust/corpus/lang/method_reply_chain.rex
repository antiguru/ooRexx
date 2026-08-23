/* A body a REPLY left owed can send a message whose method replies in its
   turn, so the second remainder is queued while the first is running. Both run
   and neither is lost. Phase 5a Task 16. */

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
