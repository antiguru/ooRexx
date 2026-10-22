do i = 1 while i < 3 | f(i)
  nop
end
exit
f: procedure
  use arg n
  if n = 2 then return 1 / 0
  return 0
