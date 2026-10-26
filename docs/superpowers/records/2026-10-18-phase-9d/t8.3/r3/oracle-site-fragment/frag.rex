.local~rr = .routines
do n = 1 to 3
  call setf
  interpret 'do k = 1 to 3; say f(n k); x = .array~new(20); end'
end
exit
setf: .routines['F'] = .routine~new('f', 'use arg n; .rr~remove("F"); return "f" n'); return
::routine dummy
