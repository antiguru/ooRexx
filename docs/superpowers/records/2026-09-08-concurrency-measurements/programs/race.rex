c = .counter~new
o1 = .bumper~new
o2 = .bumper~new
m1 = o1~start('BUMP', c, 20000)
m2 = o2~start('BUMP', c, 20000)
r1 = m1~result
r2 = m2~result
say 'final' c~value 'expected 40000'
::class counter
::method init
  expose value
  value = 0
::method value attribute
::class bumper
::method bump
  use arg c, n
  do n
    c~value = c~value + 1
  end
  return 1
