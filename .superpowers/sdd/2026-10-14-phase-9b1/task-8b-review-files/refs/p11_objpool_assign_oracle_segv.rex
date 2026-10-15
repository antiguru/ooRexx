r = .k~new~ref
call filler
say 'p10a' length(r~value) left(r~value,3)
r~value = copies('nw', 7)
call filler
say 'p10b' length(r~value) left(r~value,3)
o = .k~new
r2 = o~ref
drop o
call filler
say 'p10c' length(r2~value)
exit
::class k
::method ref
  expose v
  v = copies('ov', 13)
  return >v
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
