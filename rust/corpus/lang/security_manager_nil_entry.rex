/* An info directory entry the security manager sets to .nil is set: the
   checkpoint answers .nil, or for COMMAND's FAILURE and ERROR raises the
   condition. An entry the manager leaves unset is not. */
call show "environment nil", "return .RexxInfo", "ENVIRONMENT", "RESULT", .nil
call show "environment value", "return .RexxInfo", "ENVIRONMENT", "RESULT", "replaced"
call show "environment unset", "return .RexxInfo~class~id", "ENVIRONMENT", "", ""
call show "local nil", "return .stdout", "LOCAL", "RESULT", .nil
call show "local unset", "return .stdout~class~id", "LOCAL", "", ""
call show "method nil", "return .Directory~new~setMethod('x')", "METHOD", "RESULT", .nil
call show "method unset", "d = .Directory~new; d~setMethod('x'); return var('result')", "METHOD", "", ""
call show "call nil", "x = nosuchfunction(1); return x", "CALL", "RESULT", .nil
call show "command rc nil", "address system 'true'; return rc", "COMMAND", "RC", .nil
call show "command rc unset", "address system 'true'; return rc", "COMMAND", "", ""
call show "command failure nil", "signal on failure; address system 'true'; return 'no failure' rc; failure: return 'failure' rc", "COMMAND", "FAILURE", .nil
call show "command error nil", "signal on error; address system 'true'; return 'no error' rc; error: return 'error' rc", "COMMAND", "ERROR", .nil
exit

show: procedure
  use arg label, code, event, entry, value
  mgr = .Mgr~new(event, entry, value)
  r = .Routine~new("", code)
  r~setSecurityManager(mgr)
  r~call
  if var("RESULT") then say label "result:" result
  else say label "no result"
  say label "events:" mgr~log~makeString(, " ")
  return

::class Mgr
::method init
  expose log event entry value
  use arg event, entry, value
  log = .Array~new
::method log
  expose log
  return log
::method unknown
  expose log event entry value
  use arg name, args
  info = args[1]
  log~append(name)
  if name \= event then return 0
  if entry \== "" then info[entry] = value
  return 1
