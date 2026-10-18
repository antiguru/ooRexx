f = 'ro.txt'
call lineout f, 'abc'; call lineout f, 'def'; call lineout f
'chmod a-w' f
say linein(f) stream(f,'c','query position write') stream(f,'c','query position write line') stream(f,'d')
say stream(f,'c','query position read line')
