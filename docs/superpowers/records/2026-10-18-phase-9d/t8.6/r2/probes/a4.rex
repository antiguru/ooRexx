s = .stream~new('q.txt'); s~open('write append'); s~lineout('q1')
'echo cmdq >> q.txt'
s~lineout('q2')
call lineout 'STDERR', 'pos' s~query('position write char') s~query('position write line') s~query('size')
s~lineout('q3')
x = 1 + 'a'
