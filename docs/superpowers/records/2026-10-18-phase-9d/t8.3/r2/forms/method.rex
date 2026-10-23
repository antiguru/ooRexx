.local~rr = .routines
.routines['M'] = .routine~new('m', 'use arg n; .rr~remove("M"); return "m" n')
o = .thing~new
do n = 1 to 3
  say o~viam(n)
  call filler
end
exit
viaf: return f(arg(1))
filler: procedure
  do i = 1 to 20
    x = copies('a', 1000000)
  end
  return
::routine dummy
  return 0
::class thing
::method viam
  return m(arg(1))
::class cmp
::method compare
  use arg l, r
  say s(l)
  return 0
