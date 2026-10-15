w = mk()
call filler
say 'q13' (w~value == .nil)
r = mk2()
w2 = .WeakReference~new(r)
call filler
say 'q13b' length(w2~value~value) left(w2~value~value, 2)
exit
mk: procedure
  y = copies('wk', 10)
  return .WeakReference~new(>y)
mk2: procedure
  y = copies('w2', 10)
  return >y
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
