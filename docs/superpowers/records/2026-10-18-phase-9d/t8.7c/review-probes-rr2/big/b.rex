numeric digits 20
parse arg size
s = .stream~new('big.dat')
say 'open' s~open('write append') 'size' s~query('size')
do p over (1, 4294967295, 4294967296, size - 1, size, size + 1, size + 2)
  say 'seek ='p s~seek('='p 'write char') s~state
end
say 'seek <0' s~seek('<0 write char') s~state
say 'seek <1' s~seek('<1 write char') s~state
say 'seek -1' s~seek('-1 write char') s~state
say 'charout at size' s~charout('x', size) s~state
say 'charout at size+1' s~charout('x', size + 1) s~state
say 'size' s~query('size')
s~close
s2 = .stream~new('big.dat')
say 'reopen' s2~open('both append') 'size' s2~query('size')
say 'seek size+1' s2~seek('=' || (size + 1) 'write char') s2~state; say 'seek size+3' s2~seek('=' || (size + 3) 'write char') s2~state
say 'seek size+2' s2~seek('=' || (size + 2) 'write char') s2~state
s2~close
