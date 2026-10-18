/* GUARD WHEN inside INTERPRET */
o = .k~new
m = o~start('waiter')
call SysSleep 0.1
o~bump
say m~result
::class k
::method init
  expose v
  v = 0
::method bump
  expose v
  v += 1
::method waiter
  expose v
  interpret 'guard on when v > 0'
  return 'woke' v
