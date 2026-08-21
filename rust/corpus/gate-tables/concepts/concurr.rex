/* provide.xml `concurr`: Rexx supports concurrency -- more than one method
   running on a single object. REPLY returns a result to the sender and lets
   the method go on running, and GUARD controls the object's guarded state. */
say 'reply' .k~go
say 'guarded' .k~guarded
say 'unguarded' .k~unguarded

::class k
::method go class
  reply 'replied'

::method guarded class guarded
  guard off
  guard on
  return 'ran guarded'

::method unguarded class unguarded
  return 'ran unguarded'
