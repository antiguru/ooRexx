/* A security manager that renames a REQUIRES: the file is still found, and a
   miss still reported, by the name as written, and the second event names
   the original resolved. */
src = .array~of("::class test_requires public")
call write 'rq_replaced.cls', src
call write 'rq_handle.cls', src
replaced = .File~new('rq_replaced.cls')~name
handle = .File~new('rq_handle.cls')~name

say '-- an existing file renamed to another'
mgr = .collector~new('rq_handle', 'rq_replaced')
r = .Routine~new("", "return .Package~new('', '')~loadPackage('" || handle || "')~name")
r~setSecurityManager(mgr)
r~call
say 'loaded the replaced file' result~endsWith('rq_replaced.cls')
do name over mgr~names
  say 'event name ends with the original' name~endsWith('rq_handle.cls')
end

say '-- a missing file renamed to an existing one'
mgr = .collector~new('rq_missing', 'rq_replaced')
code = "signal on syntax; return .Package~new('', '')~loadPackage('rq_missing.cls')~name;" -
  "syntax: return condition('o')~code condition('o')~message"
r = .Routine~new("", code)
r~setSecurityManager(mgr)
r~call
say result
say mgr~names~items 'events'

call SysFileDelete 'rq_replaced.cls'
call SysFileDelete 'rq_handle.cls'
exit

write: procedure
  use arg name, lines
  s = .Stream~new(name)
  s~open('write replace')
  s~arrayOut(lines)
  s~close
  return

::class collector
::method init
  expose from to names
  use arg from, to
  names = .array~new
::method names
  expose names
  return names
::method unknown
  expose from to names
  use arg name, args
  info = args[1]
  if name \== 'REQUIRES' then return 0
  names~append(info['NAME'])
  info['NAME'] = info['NAME']~changeStr(from, to)
  return 1
