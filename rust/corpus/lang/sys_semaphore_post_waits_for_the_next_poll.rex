/* A post between two polls of a timed wait stays in the semaphore until
   the next poll, so a try that comes first takes it. */
h = SysCreateEventSem()
msg = .t~new~start('w', h)
call SysSleep 0.12
call SysPostEventSem h
call SysSleep 0.03
say 'main took' SysWaitEventSem(h, 1)
call SysPostEventSem h
say 'w' msg~result
say 'close' SysCloseEventSem(h)
::class t
::method w
  use arg h
  return SysWaitEventSem(h, 1000)
