/* A generated ::ATTRIBUTE setter's store wakes a GUARD WHEN watching the
   attribute's variable. */
o = .k~new
m = o~start('waiter')
call SysSleep 0.1
say 'setting'
o~flag = 1
m~wait
say m~result
::class k
::attribute flag
::method init
  expose flag
  flag = 0
::method waiter
  expose flag
  guard on when flag = 1
  return 'woke with flag' flag
