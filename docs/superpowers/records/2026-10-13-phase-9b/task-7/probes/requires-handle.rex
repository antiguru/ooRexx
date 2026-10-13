src = .array~of("  call SysDropFuncs", "  ::class test_requires public", "  ::method trigger_local class", "    .stdout~class")
call write 'R_replaced.cls', src
call write 'R_handle.cls', src
replaced = .File~new('R_replaced.cls')~name
handle = .File~new('R_handle.cls')~name
mgr = .collector~new(handle, replaced)
r = .Routine~new("", "return .Package~new('', '')~loadPackage('" || handle || "')~name")
r~setSecurityManager(mgr)
r~call
say 'answer' result~endsWith(replaced) result
say 'events' mgr~events
call SysFileDelete 'R_replaced.cls'
call SysFileDelete 'R_handle.cls'
exit
write: procedure
  use arg name, lines
  s = .Stream~new(name); s~open('write replace'); s~arrayOut(lines); s~close
  return
::class collector
::method init
  expose from to events
  use arg from, to
  events = ''
::method events
  expose events
  return events
::method unknown
  expose from to events
  use arg name, args
  info = args[1]
  events = events name
  if name == 'REQUIRES' then do
    say 'requires name' info['NAME']
    info['NAME'] = info['NAME']~changeStr(from, to)
    return 1
  end
  return 0
