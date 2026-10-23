.local~rr = .routines
.routines['F'] = .routine~new('f', 'use arg n; .rr~remove("F"); return "f" n')
do n = 1 to 3
  say .thing~new~viaf(n)
  x = .array~new(20)
end
exit
viaf: return f(arg(1))
::routine dummy
::class thing
::method viaf
  return f(arg(1))
::class cmp
::method compare
  say f(arg(1))
  return 0
