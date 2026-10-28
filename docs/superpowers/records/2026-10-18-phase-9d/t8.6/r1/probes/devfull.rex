r = lineout('/dev/full', 'x'); say 'lineout' r stream('/dev/full','d')
r = charout('/dev/full', 'y'); say 'charout' r stream('/dev/full','d')
say 'flush' stream('/dev/full','c','flush') stream('/dev/full','d')
r = lineout('/dev/full', copies('z', 5000)); say 'big' r stream('/dev/full','d')
say 'close' stream('/dev/full','c','close') stream('/dev/full','d')
s = .stream~new('/dev/full'); say s~open('write') s~lineout('a') s~description
say 'objclose' s~close s~description
t = .stream~new('/dev/full'); say t~open('write nobuffer') t~lineout('a') t~description
