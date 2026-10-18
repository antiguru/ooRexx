parse source . . me
d = me~left(me~lastPos('/'))
f = d || 'small.txt'
call stream f, 'c', 'open write replace'
call charout f, 'line1' || '0a'x || 'line2' || '0a'x || 'line3' || '0a'x
call stream f, 'c', 'close'
call stream f, 'c', 'open both nobuffer'
call lineout f, 'new', 2
w1 = stream(f, 'c', 'query position write')
call lineout f, 'tail'
w2 = stream(f, 'c', 'query position write')
call linein f
call lineout f, 'more'
say 'small' w1 w2 stream(f, 'c', 'query position write') c2x(charin(f, 1, stream(f, 'c', 'query size')))
call stream f, 'c', 'close'
g = d || 'big.txt'
call stream g, 'c', 'open write replace'
do i = 1 to 600
  call lineout g, 'L' || i~right(6, 0) || '.'
end
call stream g, 'c', 'close'
call stream g, 'c', 'open both nobuffer'
call lineout g, 'new', 2
w1 = stream(g, 'c', 'query position write')
call linein g
call lineout g, 'tail'
call stream g, 'c', 'close'
say 'big' w1 stream(g, 'c', 'query size') charin(g, 1, 27)~changestr('0a'x, '|') charin(g, 4092, 27)~changestr('0a'x, '|')
call stream g, 'c', 'close'
