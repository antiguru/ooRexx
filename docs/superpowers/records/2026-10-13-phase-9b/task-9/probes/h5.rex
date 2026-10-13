s = .Stream~new('HANDLE:0')
say 'init q' (s~qualify == .File~new('HANDLE:0')~absolutePath) 'size [' || s~query('size') || '] time [' || s~query('timestamp') || '] pos [' || s~query('position') || ']'
say 'open' s~open('read') 'q [' || s~qualify || '] size [' || s~query('size') || '] desc' s~description 'state' s~state
say 'string' s~string
say 'close' s~close 'q [' || s~qualify || '] qe [' || s~query('exists') || '] state' s~state s~description
t = .Stream~new('HANDLE:2')
say 't open' t~open 'qs' t~query('streamtype') 'qh' t~query('handle') 'state' t~state 'close' t~close t~state
u = .Stream~new('HANDLE:1')
say 'u open both' u~open('both nobuffer') 'u close' u~close
say 'u open bad' u~open('zork')
