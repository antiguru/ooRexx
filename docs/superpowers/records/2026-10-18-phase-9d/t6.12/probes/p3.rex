-- .nil entries the manager sets on the other checkpoints
call show "call-nil", "x = nosuchfunction(1); return x", "CALL", "RESULT"
call show "cmd-failure-nil", "signal on failure; address system 'true'; return 'no failure' rc; failure: return 'failure' rc", "COMMAND", "FAILURE"
call show "cmd-error-nil", "signal on error; address system 'true'; return 'no error' rc; error: return 'error' rc", "COMMAND", "ERROR"
call show "cmd-rc-nil", "address system 'true'; return rc", "COMMAND", "RC"
call show "stream-nil", "signal on syntax; return chars('zzz-none'); syntax: return 'syntax' condition('o')~code", "STREAM", "STREAM"
exit

show: procedure
  use arg label, code, event, entry
  mgr = .Mgr~new(event, entry)
  r = .Routine~new("", code)
  r~setSecurityManager(mgr)
  r~call
  if var("RESULT") then say label "result:" result
  else say label "no result"
  say label "events:" mgr~log~makeString(, " | ")
  return

::class Mgr
::method init
  expose log event entry
  use arg event, entry
  log = .Array~new
::method log
  expose log
  return log
::method unknown
  expose log event entry
  use arg name, args
  info = args[1]
  log~append(name)
  if name = event then do
    info[entry] = .nil
    return 1
  end
  return 0
