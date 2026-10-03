/* The unnamed RexxUtil semaphores are counting semaphores: no owner, no
   re-entry, a timeout of zero waits for ever, one below zero takes nothing,
   and a timed wait that takes nothing answers 121 after its polls. */
h = SysCreateEventSem()
say 'handle' datatype(h, 'W') (h > 0)
say 'post' SysPostEventSem(h) SysPostEventSem(h)
say 'wait' SysWaitEventSem(h) SysWaitEventSem(h, 0)
call time 'r'
say 'timed' SysWaitEventSem(h, 250)
say 'polled long enough' (time('e') >= 0.29)
call time 'r'
say 'timed1' SysWaitEventSem(h, 1)
say 'polled once' (time('e') >= 0.09)
say 'negative' SysWaitEventSem(h, -5)
say 'post' SysPostEventSem(h) 'reset' SysResetEventSem(h) 'wait' SysWaitEventSem(h, 50)
say 'close' SysCloseEventSem(h)
m = SysCreateMutexSem('')
say 'mutex handle' datatype(m, 'W')
say 'request' SysRequestMutexSem(m) SysRequestMutexSem(m, 100)
say 'release' SysReleaseMutexSem(m) SysReleaseMutexSem(m) SysRequestMutexSem(m, 100) SysRequestMutexSem(m, 100)
say 'release' SysReleaseMutexSem(m) 'post' SysPostEventSem(m) SysPostEventSem(m) 'request' SysRequestMutexSem(m, 1) SysRequestMutexSem(m, 1) SysRequestMutexSem(m, 1) SysRequestMutexSem(m, 1)
e = SysCreateEventSem(, 1)
say 'event release' SysReleaseMutexSem(e) SysReleaseMutexSem(e) SysWaitEventSem(e, 1) SysWaitEventSem(e, 1)
say 'close' SysCloseMutexSem(m) SysCloseMutexSem(e)
signal on syntax name e1
say SysPostEventSem()
e1: say 'error' condition('o')~code condition('o')~message
signal on syntax name e2
say SysPostEventSem('abc')
e2: say 'error' condition('o')~code condition('o')~message
signal on syntax name e3
say SysWaitEventSem(h, 'x')
e3: say 'error' condition('o')~code condition('o')~message
signal on syntax name e4
say SysWaitEventSem(h, 1.5)
e4: say 'error' condition('o')~code condition('o')~message
signal on syntax name e5
say SysPostEventSem(h, 1)
e5: say 'error' condition('o')~code condition('o')~message
signal on syntax name e6
say SysCreateEventSem(,,1)
e6: say 'error' condition('o')~code condition('o')~message
signal on syntax name e7
say SysWaitEventSem(h, 3000000000)
e7: say 'error' condition('o')~code condition('o')~message
signal on syntax name e8
say SysPostEventSem(-1)
e8: say 'error' condition('o')~code condition('o')~message
signal on syntax name e9
say SysCreateMutexSem('', 2)
e9: say 'error' condition('o')~code condition('o')~message
