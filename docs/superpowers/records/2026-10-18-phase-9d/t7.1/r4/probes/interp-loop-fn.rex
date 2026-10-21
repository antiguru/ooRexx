/* function-call form inside a loop body that INTERPRET re-runs. INTERPRET
   text may not hold a label, so a raise leaves the text; the trap is in RUN. */
ci = .routines~i~copy
cr = .routines~r~copy
/* A: one text, a loop: k=1 keeps i, rebind to r at k=2 (kept answers),
   k=3 'abc' raises through kept i; next pass is a fresh text */
src = 'do k = 1 to 4;',
      '  if k = 2 then .routines["Y"] = cr;',
      '  q = y(word("1 2 abc 4", k));',
      '  say "A" pass k "answered" q;',
      'end'
do pass = 1 to 3
  .routines['Y'] = ci
  call run src
end
/* B: a DO loop whose body INTERPRETs a function call, rebinding between */
do k = 1 to 6
  select
    when k = 1 then .routines['Y'] = ci
    when k = 3 then .routines['Y'] = cr
    when k = 5 then .routines['Y'] = ci
    otherwise nop
  end
  call run 'q = y(word("1 2 abc def 5 6", k)); say "B" k "answered" q'
end
/* C: canonical objects, shape A */
ci = .routines~i
cr = .routines~r
do pass = 4 to 5
  .routines['Y'] = ci
  call run src
end
exit 0
run: procedure expose ci cr pass k
  signal on syntax name s
  interpret arg(1)
  return
s: say 'raised in pass' pass 'k' k condition('O')~code
  return
::routine i external 'LIBRARY orxfunction TestIntArg'
::routine r external 'LIBRARY orxfunction TestRaiseException0'
