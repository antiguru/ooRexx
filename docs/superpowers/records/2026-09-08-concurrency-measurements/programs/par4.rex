objs = .array~new
msgs = .array~new
do i = 1 to 4
  objs[i] = .w~new
  msgs[i] = objs[i]~start('SPIN', 3000000)
end
tot = 0
do i = 1 to 4
  tot = tot + msgs[i]~result
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
