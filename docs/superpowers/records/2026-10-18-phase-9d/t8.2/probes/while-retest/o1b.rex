n = 0
do while n < 3 & h(n)
  n = n + 1
end
exit
h: procedure
  use arg n
  if n = 1 then return 1 / 0
  return 1
