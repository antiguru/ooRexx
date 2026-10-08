/* processExecutableSource's conversions: requestArray, then makeString for a
 * value that is not a primitive. A Directory is its index array. */
call t .environment
call t .directory~new
call t .ms~new
call t .ma~new
call t .list~of('return "L"')
s.1 = 'return "S"'; s.0 = 1
call t s.
call t .queue~of('return "Q"')
call t .mb~new
call t .mutablebuffer~new('return "B"')
exit
t: procedure
  use arg src
  signal on syntax
  m = .method~new('m', src)
  say "ok" m~source~items m~source[1]
  return
syntax: say 'err' condition('o')~code condition('o')~message
  return
::class ms
::method makestring
  return 'return "MS"'
::class ma
::method makearray
  return .array~of('return "MA"')
::class mb
::method makestring
  return 'return "MB"'
::method makearray
  return .array~of('return "MBA"')
