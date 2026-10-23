.local~rr = .routines
.routines['F'] = .routine~new('f', 'use arg n; .rr~remove("F"); return "f" n')
do n = 1 to 3
  say f(n)
  do j = 1 to 20; x = copies("a", 1000000); end
end
exit
viaf: return f(arg(1))
::routine dummy
