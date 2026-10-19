do mode = 1 to 2
  mgr = .Mgr~new(mode)
  r = .Routine~new("", "signal on syntax; return .Package~new('', '')~loadPackage('p4req.cls')~name; syntax: return 'syntax' condition('o')~code")
  r~setSecurityManager(mgr)
  r~call
  say mode "result:" result~substr(result~lastPos('/') + 1)
  say mode mgr~log~makeString(, " | ")
end
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
  log~append(name)
  if name = "REQUIRES" then do
    if mode = 1 then info['NAME'] = .nil
    else info['SECURITYMANAGER'] = .nil
    return 1
  end
  return 0
