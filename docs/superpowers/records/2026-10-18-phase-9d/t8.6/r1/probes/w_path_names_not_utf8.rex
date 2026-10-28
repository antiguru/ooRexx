/* Names that are not UTF-8 name the file they spell. Every file is made by a
   shell command, which passes the bytes through, and each site has a decoy
   named with the bytes a lossy conversion would produce ('efbfbd'x). */
ff = 'ff'x; fffd = 'efbfbd'x
start = directory()
address system 'mkdir d'ff 'd'fffd'; echo decoy > d'fffd'/decoy'
address system 'printf "real\n" > f'ff'; printf "decoy\ndecoy\n" > f'fffd
address system 'printf "return \"from ff\"\n" > r'ff'.rex; printf "return \"from decoy\"\n" > r'fffd'.rex'
/* DIRECTORY enters the directory the bytes name, and answers its bytes */
here = directory('d'ff)
say 'directory:' (here == start'/d'ff) c2x(right(directory(), 1))
/* a command's working directory */
address system 'pwd' with output stem p.
say 'pwd:' (p.1 == start'/d'ff)
call directory start
/* a .Stream: queries by name, an explicit open and an implicit one */
say 'exists:' (stream('f'ff, 'c', 'query exists') == start'/f'ff) 'size:' stream('f'ff, 'c', 'query size')
s = .stream~new('f'ff)
say 'open:' s~open('read') s~lines s~linein s~close
say 'implicit:' lines('f'ff) linein('f'ff) chars('f'ff)
call stream 'f'ff, 'c', 'close'
/* .File */
f = .file~new('f'ff)
say 'file:' f~exists f~isFile f~length f~canRead
say 'list:' .file~new('d'ff)~list~items
say 'search:' (.file~searchPath('f'ff, start)~absolutePath == start'/f'ff)
say 'rename:' f~renameTo(.file~new('h'ff)) .file~new('h'ff)~exists .file~new('f'fffd)~exists
/* an external file read by .Routine~newFile */
say 'newFile:' .routine~newFile('r'ff'.rex')~call
address system 'rm -rf d'ff 'd'fffd 'h'ff 'f'fffd 'r'ff'.rex r'fffd'.rex'
