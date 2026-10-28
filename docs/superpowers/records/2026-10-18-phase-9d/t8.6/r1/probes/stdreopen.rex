say 's0'
say .stdout~open('write')
.stdout~lineout('a')
say 's1'
.stdout~lineout('c')
.stderr~lineout('e1')
say .stderr~open('write')
.stderr~lineout('e2')
call lineout 'STDERR', 'e3'
.stdout~close
.stdout~lineout('after close')
say 's2'
.output~lineout('output')
say 's3'
