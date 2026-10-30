o = .bad~new
r = >o
call t 'say r'
call t "say r || 'x'"
call t "say 'p'r"
call t 'say length(r)'
call t 'say substr(r, 1)'
call t "say 'abc'~pos(r)"
call t 'say 1 + r'
call t 'do r; end'
call t 'say r~string'
call t 'say r~nosuch'
call t 'say .array~of(r)~toString'
call t 'say .array~of(r)~makestring'
call t 'say .array~of(r)'
call t 's. = r; say s.'
call t 's. = r; say s.~string'
call t 'say .array~of(1, .array~of(r))~makestring'
call t "say r~request('STRING')"
call t "say r == 'x'"
call t "say 'x' == r"
call t "q = .queue~of(r); say q~makearray~makestring"
call t "say r~objectname"
call t "say r~value"
call t "call charout , r; say"
call t "parse value r with v; say v"
exit
t:
  signal on syntax name s
  signal on nostring name ns
  interpret arg(1)
  say '  ' arg(1) '-> ran'
  return
s: say '  ' arg(1) '-> SYNTAX' condition('o')~code condition('o')~message; return
ns: say '  ' arg(1) '-> NOSTRING' condition('d'); return
::class bad
::method defaultname
  raise syntax 98.900 array('boom')
