parse arg n
o = .k~new
do i = 1 to n
  m = .Method~new('m', 'return "' || copies('abcdefgh', 20) || i '"')
  x = o~go(m)
end
say length(x)
::class k
::method go
  use arg m
  return self~run(m)
