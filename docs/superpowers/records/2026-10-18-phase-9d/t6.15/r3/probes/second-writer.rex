/* Line positions while another stream object, or another process, writes
   the same file. */
f = 'c.txt'
call charout f, 'a' || '0a'x || 'b' || '0a'x; call charout f
r = .stream~new(f); w = .stream~new(f)
zz = r~open('read'); zz = w~open('write append')
say 'before' r~lines('C')
zz = w~lineout('c'); zz = w~lineout('d'); zz = w~close
say 'after ' r~lines('C') 'fresh' .stream~new(f)~lines('C')
zz = r~close
g = 'd.txt'
call charout g, 'aaaa' || '0a'x || 'bbbb' || '0a'x || 'cccc' || '0a'x; call charout g
r = .stream~new(g)
zz = r~open('read')
say 'A' r~linein r~linein r~query('position read line') r~query('position read char')
o = .stream~new(g)
zz = o~charout('x' || '0a'x || 'y' || '0a'x, 1)
zz = o~close
say 'B' r~query('position read line') r~lines('C')
zz = r~close
h = 'e.txt'
call charout h, 'aaaa' || '0a'x || 'bbbb' || '0a'x || 'cccc' || '0a'x; call charout h
r = .stream~new(h)
zz = r~open('read')
say 'C' r~linein r~query('position read line')
address system "printf 'x\ny\n' | dd of=e.txt conv=notrunc status=none"
say 'D' r~query('position read line') r~lines('C') r~linein r~query('position read line')
zz = r~close
