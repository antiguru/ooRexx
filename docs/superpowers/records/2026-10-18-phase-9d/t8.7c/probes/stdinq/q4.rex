s = .stream~new('STDIN')
say 'seek' s~seek('1') s~state
say 'type' s~query('streamtype')
say 'size' s~query('size')
say 'pos' s~query('position')
t = .stream~new('STDIN')
say 'open' t~open('read') t~query('streamtype')
say 'seek2' t~seek('1') t~state
say 'type2' .stdin~query('streamtype')
say 'seek3' .stdin~seek('1')
