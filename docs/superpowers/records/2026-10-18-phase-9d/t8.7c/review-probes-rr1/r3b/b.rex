s = .stream~new('big.dat')
say 'open' s~open('write append') 'size' s~query('size')
say 'seek =4294967290' s~seek('=4294967290 write char') s~state
say 'seek =4294967296' s~seek('=4294967296 write char') s~state
say 'seek =5000000000' s~seek('=5000000000 write char') s~state
say 'charout 5000000000' s~charout('x', 5000000000) s~state
say 'size' s~query('size')
s~close
