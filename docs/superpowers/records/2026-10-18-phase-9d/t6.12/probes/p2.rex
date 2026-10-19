-- a protected copy of a primitive, defined into a subclass
m = .Array~method('APPEND')~copy
m~setProtected
say "copy" m~isProtected .Array~method('APPEND')~isProtected
.MyArr~define('APPEND', m)
call show "copy-append", "a = .MyArr~new; a~append('x'); return a~items", 0
-- a Rexx method made protected at run time
.K~method('FOO')~setProtected
call show "setprotected-rexx", "return .K~new~foo", 0
call show "setprotected-rexx-handled", "return .K~new~foo", 2
-- the environment result
call show "env-nil", "return .RexxInfo", 3
call show "env-value", "return .RexxInfo", 4
call show "local-nil", "return .stdout", 5
exit

show: procedure
  use arg label, code, mode
  mgr = .Mgr~new(mode)
  r = .Routine~new("", code)
  r~setSecurityManager(mgr)
  r~call
  if var("RESULT") then say label "result:" result
  else say label "no result"
  say label "events:" mgr~log~makeString(, " | ")
  return

::class MyArr subclass Array
::class K
::method foo
  return "ran"

::class Mgr
::method init
  expose log mode
  use arg mode
  log = .Array~new
::method log
  expose log
  return log
::method unknown
  expose log mode
  use arg name, args
  info = args[1]
  line = name
  do k over info~allIndexes~sort
    v = info[k]
    if k = "OBJECT" then line = line k"="v~class~id
    else if v~isA(.Array) then line = line k"=["v~items":"v~makeString(, ",")"]"
    else line = line k"="v
  end
  log~append(line)
  if name = "METHOD" & mode = 2 then do
    info~result = "intercepted"
    return 1
  end
  if name = "ENVIRONMENT" & mode = 3 then do
    info~result = .nil
    return 1
  end
  if name = "ENVIRONMENT" & mode = 4 then do
    info~result = "replaced"
    return 1
  end
  if name = "LOCAL" & mode = 5 then do
    info~result = .nil
    return 1
  end
  return 0
