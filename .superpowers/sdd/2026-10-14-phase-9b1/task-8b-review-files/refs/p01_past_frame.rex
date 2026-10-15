r = mk(20)
call filler
say 'p01' length(r~value) left(r~value, 4)
s = mk(21)
call filler
say 'p01b' length(r~value) length(s~value)
exit
mk: procedure
  use arg n
  y = copies('ab', n)
  return >y
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
