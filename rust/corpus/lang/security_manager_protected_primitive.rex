/* A primitive Setup.cpp defines protected sends the security manager a
   METHOD event, from a Routine, a Package prolog and a Method; the manager
   may answer the send itself, with or without a result. An unprotected
   primitive sends none. */
call show "setMethod", ".Directory~new~setMethod('unknown')", "pass"
call show "unsetMethod", ".Directory~new~unsetMethod('unknown')", "pass"
call show "superclasses", ".Array~superclasses; return var('result')", "handle"
call show "setSecurityManager", ".Context~executable~setSecurityManager; return result", "result"
call show "define", "k = .Object~subclass('K'); k~define('FOO', 'return 1'); return k~new~foo", "pass"
call show "define handled", "k = .Object~subclass('K'); k~define('FOO', 'return 1'); return k~hasMethod('FOO')", "handle"
call show "superclass", "return .Array~superclass", "result"
call show "metaClass", "return .Array~metaClass~id", "pass"
call show "plain", "return .Directory~new~items", "pass"
call show "plain class", "return .Array~id", "pass"

mgr = .Mgr~new("handle")
p = .Package~new("", ".Directory~new~setMethod('unknown'); return 'prolog'; ::options noprolog")
p~setSecurityManager(mgr)
p~prolog~call
say "package result:" result
say "package events:" mgr~log~makeString(, " | ")

mgr = .Mgr~new("result")
m = .Method~new("", "return .Directory~new~setMethod('unknown')")
m~setSecurityManager(mgr)
say "method result:" .Runner~new~run(m)
say "method events:" mgr~log~makeString(, " | ")

say "isProtected" .Directory~method("SETMETHOD")~isProtected -
  .Directory~method("UNSETMETHOD")~isProtected .Class~method("DEFINE")~isProtected -
  .Class~method("SUPERCLASSES")~isProtected .Method~method("SETSECURITYMANAGER")~isProtected -
  .Routine~method("SETSECURITYMANAGER")~isProtected .Package~method("SETSECURITYMANAGER")~isProtected -
  .Package~instanceMethod("DEFAULTOPTIONS")~isProtected .Method~method("SETPROTECTED")~isProtected
say "unprotected" .Array~method("APPEND")~isProtected .Directory~method("PUT")~isProtected -
  .Class~method("ID")~isProtected .Object~method("STRING")~isProtected
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

::class Runner
::method run
  use arg m
  self~setMethod("RUNMETHOD", m)
  return self~runMethod

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
  if name \= "METHOD" | mode = "pass" then return 0
  if mode = "result" then info~result = "intercepted"
  return 1
