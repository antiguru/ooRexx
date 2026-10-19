-- METHOD events from protected primitives, each run from a Routine with a manager
call show "setMethod", ".Directory~new~setMethod('unknown')", 0
call show "superclasses", ".Array~superclasses; return var('result')", 1
call show "setSecurityManager", ".Context~executable~setSecurityManager; return result", 2
call show "define", ".Object~subclass('K')~define('FOO', 'return 1')", 0
call show "plain", ".Directory~new", 0
say "flags" .Directory~method("SETMETHOD")~isProtected .Class~method("DEFINE")~isProtected -
   .Package~instanceMethod("DEFAULTOPTIONS")~isProtected .Method~method("SETSECURITYMANAGER")~isProtected -
   .Array~method('APPEND')~isProtected
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
  if name = "METHOD" & mode = 1 then return 1
  if name = "METHOD" & mode = 2 then do
    info~result = "intercepted"
    return 1
  end
  return 0
