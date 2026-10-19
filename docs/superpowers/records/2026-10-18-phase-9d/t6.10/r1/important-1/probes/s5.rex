.local~c = 0
signal on syntax
call r
exit
syntax:
x = .traceOutput~destination
say 'syntax' condition('o')~code 'c='.local~c
::routine r
.traceOutput~destination("")
do i = 1 to 3
  if i = 2 then trace r
  .local~c = .local~c + 1
end
