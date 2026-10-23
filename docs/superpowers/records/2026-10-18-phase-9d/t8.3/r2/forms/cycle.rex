.local~rr = .routines
.routines['A'] = .routine~new('a', 'use arg n; if n > 0 then return b(n - 1);' -
  '.rr~remove("A"); .rr~remove("B"); return "a" n')
.routines['B'] = .routine~new('b', 'return a(arg(1))')
.local~ra = .routines['A']
do n = 1 to 3
  say .ra~call(2 * n)
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
