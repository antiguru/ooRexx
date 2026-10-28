call show '.stdin', .stdin
call show 'STDIN', .stream~new('STDIN')
call show 'stdin:', .stream~new('stdin:')
call show 'HANDLE:0', .stream~new('HANDLE:0')
s = .stream~new('STDIN'); say 'lines-first' s~lines s~query('streamtype') s~state
s = .stream~new('STDIN:'); say 'chars-first' s~chars s~query('streamtype') s~state s~query('exists') s~qualify~right(7)
s = .stream~new('STDIN:'); say 'exists-first' s~query('exists') s~query('size') s~query('streamtype')
say 'bn' stream('STDIN','c','query exists') stream('STDIN','c','query streamtype') stream('STDIN','c','query handle') stream('STDIN','c','query size') stream('STDIN', 'c', 'query position')
say 'bn:' stream('STDIN:','c','query exists') stream('STDIN:','c','query streamtype') stream('STDIN:','c','query handle')
exit
show: procedure
  use arg n, s
  say n 'st='s~query('streamtype') 'h='s~query('handle') 'ex='s~query('exists') 'q='s~qualify~right(7) 'state='s~state 'size='s~query('size') 'pos='s~query('position') 'seek='s~query('seek')
  say n 'chars='s~chars 'then st='s~query('streamtype') 'h='s~query('handle') 'ex='s~query('exists') 'q='s~qualify~right(7) 'state='s~state 'size='s~query('size') 'pos='s~query('position')
  return
