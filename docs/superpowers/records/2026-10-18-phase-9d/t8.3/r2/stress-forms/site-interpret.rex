.local~rr = .routines
.routines['F'] = .routine~new('f', 'use arg n; .rr~remove("F"); return "f" n')
interpret 'do n = 1 to 3; say f(n); x = .array~new(20); end'
::routine dummy
