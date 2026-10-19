.local~c = 0
signal on syntax
call r
exit
syntax:
x = .traceOutput~destination
say 'syntax' condition('o')~code 'c='.local~c
::routine r
.traceOutput~destination("")
x = trace("R")
.local~c = .local~c + 1
