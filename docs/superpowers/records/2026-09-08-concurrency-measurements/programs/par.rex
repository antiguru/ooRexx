o1 = .w~new
o2 = .w~new
m1 = o1~start('SPIN', 3000000)
m2 = o2~start('SPIN', 3000000)
say m1~result m2~result
::class w
::method spin unguarded
  use arg n
  s = 0
  do i = 1 to n
    s = s + i
  end
  return s
