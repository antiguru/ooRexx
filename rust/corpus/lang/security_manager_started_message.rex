/* A started message runs with no activation, so no security manager is
   asked about the protected method it sends, even once a manager has been
   installed; a method it enters asks its own package's. */
r = .Routine~new("r", "return 1")
r~setSecurityManager(.Mgr~new)
say "superclass via start:" .Array~start('SUPERCLASS')~result~id
d = .Directory~new
m = d~start('SETMETHOD', 'X', 'return "x ran"')
call wait m
say "setMethod via start:" m~hasResult d~x
m = .Message~new(.Array, 'SUPERCLASS')
m~start
say "message start:" m~result~id
say "protected Rexx method:" .K~new~start('FOO')~result
mgr = .Mgr~new
r = .Routine~new("", "return .Array~start('SUPERCLASS')~result~id")
r~setSecurityManager(mgr)
say "inside a managed routine:" r~call "events:" mgr~log~makeString(, " ")
exit

wait: procedure
  use arg m
  do while \m~completed
    call syssleep 0.01
  end
  return

::class K
::method foo protected
  return "foo ran"

::class Mgr
::method init
  expose log
  log = .Array~new
::method log
  expose log
  return log
::method unknown
  expose log
  use arg name, args
  log~append(name args[1]['NAME'])
  return 0
