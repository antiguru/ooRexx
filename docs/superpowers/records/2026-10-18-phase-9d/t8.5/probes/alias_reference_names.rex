call c1 >loc
.k~new~go
exit
c1: procedure
  use arg >q
  say 'cell' (>q)~name
  call c2 >q
  return
c2: procedure
  signal on syntax name e2
  use arg >s.
  return
e2: say 'e2' condition('o')~message; return
::class k
::method go
  expose v
  self~m1(>v)
::method m1
  use arg >q
  say 'inst' (>q)~name
  self~m2(>q)
  self~m3(>q)
  self~show
::method m2
  signal on syntax name e3
  use arg >s.
  return
e3: say 'e3' condition('o')~message; return
::method m3
  use arg >r
  r = 'via r'
  say 'r=' (>r)~name
::method show
  expose v
  say 'v=' v
