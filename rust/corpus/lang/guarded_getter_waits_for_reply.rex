/* A guarded attribute getter waits for the lock a REPLY continuation
   holds, and reads what the continuation wrote. */
t = .t~new
t~hold
say 'getter:' t~v
::class t
::attribute v
::method hold guarded
  expose v
  v = 'BAD'
  reply
  call SysSleep 0.3
  v = 'GOOD'
