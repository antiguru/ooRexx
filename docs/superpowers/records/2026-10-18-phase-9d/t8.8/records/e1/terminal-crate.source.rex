say 'open' stream('/dev/pts/13','C','OPEN BOTH')
say 'read' '['charin('/dev/pts/13',,3)']'
say 'write' lineout('/dev/pts/13','xyz')
say 'close' stream('/dev/pts/13','C','CLOSE')
