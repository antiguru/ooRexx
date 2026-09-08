o1 = .w~new
say o1~spin(3000000)
::class w
::method spin unguarded
  use arg n
  s = 0
  do i = 1 to n
    s = s + i
  end
  return s
