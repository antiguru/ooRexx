s = .stream~new('STDIN')
say 'unopened' '[' || s~query('size') || ']' '[' || s~query('position') || ']'
say 'open' s~open('read')
say 'size' s~query('size') .stdin~query('size') stream('STDIN', 'c', 'query size')
say 'pos' s~query('position') s~query('position read char') s~query('position read line')
say 'time' (s~query('datetime') \== '') (s~query('timestamp') \== '')
i = .stdin
say '[' || i~linein || ']' i~query('position read char') i~query('position read line')
say 'seek' i~seek('1') i~state '[' || i~linein || ']'
say 'line' i~seek('=2 read line') '[' || i~linein || ']'
say 'back' i~seek('-2 char') i~query('position read char') '[' || i~charin || ']'
say 'end' i~seek('<1') '[' || c2x(i~charin) || ']' chars() lines()
say 'past' i~seek('=99') i~state chars() lines()
say 'lines' i~seek('=99 line') i~query('position read line') i~query('position sys')
say 'again' i~seek('=1 line') '[' || linein() || ']' chars() lines()
