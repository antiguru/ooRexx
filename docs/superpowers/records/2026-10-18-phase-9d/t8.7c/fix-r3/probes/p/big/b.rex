s = .stream~new('big.dat')
say 'open' s~open('write append') 'size' s~query('size')
say 'seek =4294967290' s~seek('=4294967290 write char') s~state
say 'seek =4294967296' s~seek('=4294967296 write char') s~state
say 'seek =5368709120' s~seek('=5368709120 write char') s~state
say 'charout 5000000000' s~charout('x', 5000000000) s~state
say 'lineout' s~lineout('L') s~state
say 'seek =5368709122' s~seek('=5368709122 write char') s~state
say 'size' s~query('size')
s~close
