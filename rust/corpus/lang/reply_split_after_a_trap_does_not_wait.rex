/* A trap handled at the REPLY clause's end, then a sleep in the rest of the
   method: the sender has its value before the sleep ends. */
call time 'R'
say 'got' .k~new~m 'after' (time('E') < 0.5)
call SysSleep 1.2
::class k
::method m
  call on notready name nr
  reply 'v' || linein('/nonexistent/file/x')
  call SysSleep 1
  say 'rest done'
  return
nr:
  return
