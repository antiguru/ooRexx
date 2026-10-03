/* A GUARD ON WHEN that held the lock before its wait reserves it again on
   waking, so another activity's guarded method waits until the woken
   method ends. */
o = .k~new
m = o~start('waiter')
call SysSleep 0.1
o~set(1)
call SysSleep 0.05
o~peek
m~wait
::class k
::method init
  expose v
  v = 0
::method waiter
  expose v
  guard on when v = 1
  say 'woke'
  call SysSleep 0.2
  say 'waiter done'
::method set
  expose v
  use arg v
::method peek
  say 'peek'
