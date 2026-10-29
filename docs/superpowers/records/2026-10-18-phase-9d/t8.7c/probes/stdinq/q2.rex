s = .stream~new('STDIN')
say 'open' s~open('read')
say 'size' s~query('size')
say 'pos' s~query('position')
say 'posr' s~query('position read char') s~query('position read line')
say 'dt' (s~query('datetime') \== '')
say 'ts' (s~query('timestamp') \== '')
say 'exists' s~query('exists')
say 'type' s~query('streamtype')
say 'linein' s~linein
say 'pos2' s~query('position read char') s~query('position read line')
say 'seek' s~seek('1') s~state
say 'linein2' s~linein
say 'seekl' s~seek('=2 read line') s~state
say 'linein3' s~linein
say 'chars' s~chars 'lines' s~lines
s2 = .stream~new('STDIN')
say 'seek-new' s2~seek('1') s2~state
say 'cmd-size' stream('STDIN', 'c', 'query size')
