.local~rr = .routines
.routines['A'] = .routine~new('a', 'use arg n; if n > 0 then return b(n - 1);' -
  '.rr~remove("A"); .rr~remove("B"); return "a" n')
.routines['B'] = .routine~new('b', 'return a(arg(1))')
.local~ra = .routines['A']
do n = 1 to 3
  say .ra~call(2 * n)
  x = .array~new(20)
end
::routine dummy
