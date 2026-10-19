o = .k~new
o~go
::class k
::method go
  signal on syntax name c1
  self~refAutoExposed(>a)
  say 'not raised 1'
  signal skip
c1: say 'c1' condition('o')~code
skip:
  signal on syntax name c2
  self~refAutoExposedStem(>a.)
  say 'not raised 2'
  return
c2: say 'c2' condition('o')~code
::method refAutoExposed
  use local
  use arg >abc
::method refAutoExposedStem
  use local
  use arg >abc.
::options novalue error
