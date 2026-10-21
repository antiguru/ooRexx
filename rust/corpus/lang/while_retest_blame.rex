/* A WHILE test that fails is blamed on the clause that ran it: the DO clause
 * on the first pass, then the END or the ITERATE that sent control back, both
 * for a failure inside a routine the test calls and for the test's own, in
 * the program and inside an INTERPRET. Inside the INTERPRET the lines are
 * compared with their blanks collapsed: the indent is not what they pin. */
call try 'do i = 1 while f(i, 2); nop; end'
call try 'do i = 1 while f(i, 1); nop; end'
call try 'do i = 1 to 3 while f(i, 2); if i = 1 then iterate; end'
v = 1
call try 'do i = 1 while v; v = "x"; end'
v = 1
call try 'do i = 1 to 3 while v; if i = 1 then do; v = "x"; iterate; end; end'
call loops
exit
try:
  signal on syntax
  interpret arg(1)
  return
syntax:
  say condition('O')~code
  do line over condition('O')~traceback
    say '  ['space(line)']'
  end
  return
f: procedure
  signal off syntax
  use arg n, bad
  say '  f' n space(.context~stackframes[2]~traceline)
  if n = bad then return 1 / 0
  return 1
loops:
  signal on syntax name loops_syntax
  do j = 1 while f(j, 2)
    nop
  end
loops_syntax:
  say condition('O')~code
  do line over condition('O')~traceback
    say '  ['line']'
  end
  w = 1
  do k = 1 to 3 while w
    if k = 1 then do
      w = 'y'
      iterate
    end
  end
