-- The >C> line an exposed tail traces, for EXPOSE and PROCEDURE EXPOSE.
o = .t~new
o~m
call p
exit
p: procedure expose z.2
  return
::options trace i
::class t
::method m
  expose a.1 b c.q
  a.1 = 5
