/* A stream opened without NOBUFFER holds its writes, up to a buffer of 4096
   bytes, until it is flushed or closed or reads, measures or positions its
   file; until then another stream or process does not see them. */
sh = .stream~new('sh.txt')
address system 'echo hi' with output using (sh)
say 'redirected:' rc lines('sh.txt')
s = .stream~new('a.txt')
s~lineout('obj')
call lineout 'b.txt', 'byname'
address system 'cat a.txt b.txt; echo cmd-end'
say 'by name:' lines('b.txt') 'other object:' chars('a.txt')
say 'size by name:' stream('a.txt', 'c', 'query size') 'size of the object:' s~query('size')
address system 'cat a.txt; echo after-size'
t = .stream~new('c.txt'); t~open('write replace nobuffer'); t~lineout('nb')
say 'nobuffer:' lines('c.txt')
e = .stream~new('e.txt'); e~lineout('flushed'); say 'flush:' e~flush lines('e.txt')
f = .stream~new('f.txt'); f~charout(copies('x', 5000)); say 'past the buffer:' chars('f.txt')
g = .stream~new('g.txt'); g~charout(copies('y', 3000)); g~charout(copies('y', 3000))
say 'buffer filled:' chars('g.txt')
h = .stream~new('h.txt'); h~lineout('one'); h~lineout('three', 1); say 'out of order:' chars('h.txt')
call stream 'b.txt', 'c', 'close'; say 'closed:' chars('b.txt')
d = .stream~new('d.txt'); d~lineout('at the end')
/* a program's own STDOUT and STDERR streams are buffered too; .STDOUT and
   .STDERR are opened NOBUFFER */
o = .stream~new('STDOUT'); o~lineout('o1'); say 'say 1'; o~lineout('o2')
call lineout 'STDOUT', 'by name'; .stdout~lineout('.stdout'); o~flush; say 'say 2'
n = .stream~new('STDOUT'); n~open('write nobuffer'); n~lineout('n1'); say 'say 3'
q = .stream~new('STDOUT'); q~charout(copies('z', 5000)); say ''; say 'say 4'
r = .stream~new('STDOUT'); r~lineout('at the end too'); say 'last say'
x = .stream~new('STDERR'); x~lineout('x1'); call lineout 'STDERR', 'x2'; .stderr~lineout('x3'); x~close
address system 'rm -f sh.txt a.txt b.txt c.txt d.txt e.txt f.txt g.txt h.txt'
