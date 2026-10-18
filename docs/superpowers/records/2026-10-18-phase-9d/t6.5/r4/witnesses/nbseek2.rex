parse source . . me
dir = me~left(me~lastPos('/'))
do mode over 'both', 'both nobuffer'
  f = dir || mode~changestr(' ', '_') || '.txt'
  call stream f, 'c', 'open write replace'
  call charout f, 'line1' || '0a'x || 'line2' || '0a'x || 'line3' || '0a'x
  call stream f, 'c', 'close'
  call stream f, 'c', 'open' mode
  a = stream(f, 'c', 'seek <1 write')
  call charout f, 'ab'
  b = stream(f, 'c', 'query position read')
  c = stream(f, 'c', 'seek +3 read')
  d = stream(f, 'c', 'seek =5 write')
  e = stream(f, 'c', 'seek -2 write')
  say mode': <1 write' a', read' b', +3 read' c', =5 write' d', -2 write' e
  call stream f, 'c', 'close'
end
