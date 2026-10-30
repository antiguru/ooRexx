b = .bad~new
q = >b
call t "say 'abc'~left(q)"
call t "say left('abc', q)"
call t "x = q + 1"
call t "raise syntax 93.900 array(q)"
call t "say q~nosuch"
call t "x = .array~new(q)"
call t "a.q = 1; say 'tail ok'"
call t "say c2x(q)"
call t "say .list~of(q)~makestring"
call t "d = .directory~new; d~x = q; say d~x~string"
call t "say q~request('string')"
call t "say .array~of(q)~toString"
call t "say (q)~left(1)"
call t "say 'x'~equals(q)"
call t "say 'x' = q"
call t "say q = 'x'"
call t "say q == 'x'"
call t "say \q"
call t "if q then say 'y'"
call t "say q~objectname"
call t "say .array~of(q)~string"
call t "x = 'abc'~pos(q)"
call t "numeric digits (q)"
call t "say 5~max(q)"
call t "say 'abc'~substr(2, q)"
exit
t:
  signal on syntax name s
  signal on nostring name ns
  interpret arg(1)
  say '  ' arg(1) '-> returned'
  return
s: say '  ' arg(1) '-> SYNTAX' condition('o')~code condition('o')~message; return
ns: say '  ' arg(1) '-> NOSTRING' condition('d'); return
::class bad
::method defaultname
  say '    (bad dn)'
  raise syntax 98.900 array('boom')
