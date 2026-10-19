.local~c = 0
signal on syntax
call r
exit
syntax:
x = .traceOutput~destination
say 'syntax' condition('o')~code 'c='.local~c
::routine r
trace r
.traceOutput~destination("")
.local~c = .local~c + 1
