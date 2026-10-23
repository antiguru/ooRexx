.local~rr = .routines
.routines['F'] = .routine~new('f', 'use arg n; .rr~remove("F"); return "f" n')
interpret 'do n = 1 to 3; say viaf(n); call churn; end'
exit
viaf: return f(arg(1))
churn: procedure
  do i = 1 to 200000; x = .array~new(20); end
  return
::routine dummy
