.k~new~go
::class k
::method go
  expose v w.
  signal on syntax name c1
  say self~twice(>v)
  say 'v' v
  say self~twiceStem(>w.)
  say 'w.1' w.1
  return
c1: say 'c1' condition('o')~code condition('o')~message
  return
::method twice
  use arg >q
  q = 'set through q'
  return self~again(>q)
::method again
  use arg >r
  return 'again' r
::method twiceStem
  use arg >s.
  s.1 = 'tail'
  use arg >s.
  return 'not refused'
