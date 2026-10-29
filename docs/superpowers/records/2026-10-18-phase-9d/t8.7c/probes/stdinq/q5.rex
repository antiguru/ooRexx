say 'size' .stream~new('STDIN')~query('size')
say 'pos' .stream~new('STDIN')~query('position')
say 'dt' (.stream~new('STDIN')~query('datetime') \== '')
say 'stdin size' .stdin~query('size') .stdin~query('position') (.stdin~query('datetime') \== '')
say 'out size' .stdout~query('size') .stdout~query('position')
say 'seekr' .stdin~seek('=2 read line') .stdin~linein
say 'seekc' .stdin~seek('-2 char') .stdin~query('position read char') .stdin~charin
say 'seeke' .stdin~seek('<1') .stdin~charin
say 'seekw' .stdin~seek('=1 write') .stdin~state
say 'seek0' .stdin~seek('=0') .stdin~state
say 'linein' linein() lines() chars()
say 'seek99' .stdin~seek('=99') .stdin~state lines() chars()
say 'seekl99' .stdin~seek('=99 line') .stdin~state lines() chars()
say 'pos' .stdin~query('position read line') .stdin~query('position write') .stdin~query('position sys')
