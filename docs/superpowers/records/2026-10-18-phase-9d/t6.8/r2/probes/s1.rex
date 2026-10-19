say a.; say h.; say g.; say k.; say m.; say n.; say o.
call t 'innerThenOuter', >a.
say 'a.1' a.1
call t 'innerTwiceOuter', >h.
say 'h.1' h.1
call t 'deep1', >g.
say 'g.1' g.1
call t 'outerThenInner', >k.
say 'k.1' k.1
call t 'interpTwice', >m.
say 'm.1' m.1
call t 'dropThenRebind', >n.
say 'n.1' n.1
call t 'deepInner', >o.
say 'o.1' o.1
exit
t: procedure
  use arg name, ref
  signal on syntax name caught
  interpret 'call' name 'ref'
  say name 'answers' result
  return
caught:
  say name condition('o')~code condition('o')~message
  return
innerThenOuter: procedure
  call inner1 arg(1)
  use arg >abc.
  abc.1 = 'set'
  return 'ok'
inner1:
  use arg >abc.
  return
innerTwiceOuter: procedure
  call inner2 arg(1)
  use arg >abc.
  abc.1 = 'set'
  return 'ok'
inner2:
  use arg >abc.
  use arg >abc.
  return
deep1: procedure
  use arg >x.
  call deep2 >x.
  return result
deep2: procedure
  use arg >y.
  call deep3 >y.
  return result
deep3: procedure
  use arg >z.
  use arg >z.
  z.1 = 'set'
  return 'ok'
outerThenInner: procedure
  use arg >abc.
  call inner1 arg(1)
  abc.1 = 'set'
  return 'ok'
interpTwice: procedure
  interpret 'use arg >abc.'
  interpret 'use arg >abc.'
  abc.1 = 'set'
  return 'ok'
dropThenRebind: procedure
  use arg >abc.
  drop abc.
  use arg >abc.
  abc.1 = 'set'
  return 'ok'
deepInner: procedure
  use arg >x.
  call di2 >x.
  return result
di2: procedure
  call di3 arg(1)
  use arg >w.
  w.1 = 'set'
  return 'ok'
di3:
  use arg >w.
  return
