.local~c = 0
signal on syntax
.o~new~m
exit
syntax:
x = .traceOutput~destination
say 'syntax' condition('o')~code 'c='.local~c
::class o
::method m
  trace r
  .traceOutput~destination("")
  .local~c = .local~c + 1
