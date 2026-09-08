c = .shared~new
o1 = .bumper~new
o2 = .bumper~new
m1 = o1~start('BUMP', c, 2000, 1)
m2 = o2~start('BUMP', c, 2000, 2)
r1 = m1~result
r2 = m2~result
say 'switches observed:' c~switches
::class shared
::method init
  expose mark switches
  mark = 0
  switches = 0
::method mark attribute
::method switches attribute
::class bumper
::method bump
  use arg c, n, id
  s = 0
  do n
    c~mark = id
    call charout '/dev/null', '.'
    if c~mark \= id then s = s + 1
  end
  c~switches = c~switches + s
  return 1
