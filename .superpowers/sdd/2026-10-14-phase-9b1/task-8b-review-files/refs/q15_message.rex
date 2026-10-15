m = mk()
call filler
m~send
say 'q15' m~result
r = mk3()
c = r~copy
drop r
call filler
say 'q15b' length(c~value) left(c~value, 3) c~name
exit
mk: procedure
  y = copies('ms', 13)
  return .message~new(.k~new, 'show', 'I', >y)
mk3: procedure
  y = copies('cp', 13)
  return >y
::class k
::method show
  use arg ref
  return length(ref~value) left(ref~value, 3)
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
