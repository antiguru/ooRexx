trace r
signal on syntax
call sub
say 'not reached'
exit 0
sub:
return 1/0
syntax:
say 'in handler'
