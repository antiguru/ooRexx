s = .stream~new('STDIN')
say 'a' s~query('position') s~query('position read') s~query('position write') s~query('seek') s~query('position read line') s~query('position sys')
l = s~linein
say 'b' l s~query('position') s~query('position read line') s~query('position sys') s~query('size') s~chars s~lines
say 'c' .stdin~query('position') .stdin~query('position read line') .stdin~linein .stdin~query('position')
say 'd' stream('STDIN','c','query position') stream('STDIN','c','query size') stream('STDIN','c','query datetime')~length s~query('datetime')~length
say 'e' s~query('timestamp')~length .stdin~query('timestamp')~length
say 'f' s~close s~query('streamtype') s~query('exists') s~qualify s~query('handle') s~state
