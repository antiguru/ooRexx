call f 1
call g 2
x = f(3)
do i = 1 to 2; say f(i); end
say 'end'
::routine f
  return arg(1)
::routine g
  return arg(1)
