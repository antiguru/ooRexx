call a
call filler
say 'p02a' length(.local['R']~value) left(.local['R']~value, 3)
call c
say 'p02c' length(cv) left(cv, 3)
exit
a: procedure
  x = copies('qz', 15)
  call b >x
  return
b: procedure
  use arg ref
  .local['R'] = ref
  return
c:
  cv = copies('cc', 9)
  call d >cv
  call filler
  return
d: procedure
  use arg >z
  z = copies('dd', 13)
  call filler
  return
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
