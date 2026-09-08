tot = 0
do i = 1 to 4
  o = .w~new
  tot = tot + o~spin(3000000)
end
say tot
::class w
::method spin unguarded
  use arg n
  s = 0
  do i = 1 to n
    s = s + i
  end
  return s
