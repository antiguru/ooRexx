/* A lock taken while the program has one activity, at nesting 2, is held
   against an activity started inside it: the started guarded send runs
   only once both levels are released. */
o = .k~new
m = o~outer
m~wait
say 'after inner'
exit
::class k
::method outer
  self~mid
  m = result
  call SysSleep 0.1
  say 'outer done'
  return m
::method mid
  m = self~start('inner')
  call SysSleep 0.1
  say 'mid done'
  return m
::method inner
  say 'inner runs'
