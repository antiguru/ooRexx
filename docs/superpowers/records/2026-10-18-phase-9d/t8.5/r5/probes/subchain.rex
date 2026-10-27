/* n Stem subclass instances, each the next one's default; the last's default is 'leaf', or the first (cyc) */
parse arg n mode
signal on syntax
first = .m~new
t = first
do i = 2 to n
  u = .m~new
  t[] = u
  t = u
end
if mode = 'cyc' then t[] = first
else t[] = 'leaf'
say 'len' length(first)
say first~string
say 'end'
exit
syntax: say 'SYNTAX' condition('O')~code; exit
::class m subclass stem
