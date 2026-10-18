signal on syntax name h
say f(0)
exit
h: say 'h' condition('O')~code
exit
::routine f
  use arg n
  if n = 2000 then exit
  return f(n+1)
