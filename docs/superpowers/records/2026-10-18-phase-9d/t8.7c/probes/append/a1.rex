'printf "aaaa\nbbbb\n" > f.txt'
s = .stream~new('f.txt'); say s~open('write append')
say 'pos0' s~query('position write char') s~query('position write line')
call charout , ''
say 'w1' s~lineout('L1')
say 'pos1' s~query('position write char') s~query('position write line')
say 'seek' s~seek('=1 write char')
say 'pos2' s~query('position write char')
say 'w2' s~lineout('L2')
say 'pos3' s~query('position write char') s~query('size')
say 'w3' s~charout('C', 1)
say 'pos4' s~query('position write char')
say 'w4' s~lineout('L4', 1)
say 'pos5' s~query('position write char') s~query('position write line')
say 'close' s~close
say 'file' charin('f.txt', 1, 80)~changestr('0a'x, '|')
