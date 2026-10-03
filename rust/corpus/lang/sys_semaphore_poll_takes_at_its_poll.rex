/* A timed request tries the semaphore only at its polls: a unit released
   between two polls goes to whoever tries first, here the releaser. */
h = SysCreateMutexSem('')
say SysRequestMutexSem(h)
msg = .t~new~start('w', h)
call SysSleep 0.15
r = SysReleaseMutexSem(h); a = SysRequestMutexSem(h, 1)
say 'main rel' r 'req' a
if a = 0 then do; call SysSleep 0.05; say 'main rel2' SysReleaseMutexSem(h); end
say 'w' msg~result
::class t
::method w
  use arg h
  return SysRequestMutexSem(h, 1000) SysReleaseMutexSem(h)
