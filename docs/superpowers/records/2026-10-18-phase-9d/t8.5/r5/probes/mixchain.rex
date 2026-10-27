/* instances and references to stems alternating: t_i[] = >s_i., s_i. = t_i+1; the last stem's default is 'leaf', or t_1 (cyc) */
parse arg n mode
signal on syntax
t1 = .m~new
do i = 1 to n
  interpret 't'i'[] = >s'i'.'
  if i < n then interpret 't'i + 1' = .m~new; s'i'. = t'i + 1
end
if mode = 'cyc' then interpret 's'n'. = t1'
else interpret 's'n'. = "leaf"'
call time 'R'
x = length(t1)
e = time('E')
say 'len' x 'in' e
say t1~string
say 'end'
exit
syntax: say 'SYNTAX' condition('O')~code; exit
::class m subclass stem
