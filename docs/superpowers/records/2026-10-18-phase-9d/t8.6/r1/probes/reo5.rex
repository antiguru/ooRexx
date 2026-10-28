.stdout~open('write')
.stdout~lineout('x')
say 'y'
.stdout~flush
say 'z'
