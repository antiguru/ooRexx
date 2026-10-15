s. = 'unset'
res = mk()
call filler
say 'p03 arr' length(res[1]~value) left(res[1]~value,2)
say 'p03 dir' length(res[2]['R']~value) left(res[2]['R']~value,2)
say 'p03 stem' length(s.1~value) left(s.1~value,2)
say 'p03 same' (res[1]~value == s.1~value)
exit
mk: procedure expose s.
  y = copies('mn', 17)
  a = .array~new
  a[1] = >y
  d = .directory~new
  d['R'] = >y
  s.1 = >y
  return .array~of(a[1], d)
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
