ff = 'ff'x; fffd = 'efbfbd'x; e9 = 'e9'x
start = directory()
address system 'mkdir d'ff 'd'fffd'; printf "in\n" > d'ff'/i'e9'; printf "decoy\n" > d'fffd'/i'fffd
address system 'printf "real\n" > f'ff'; printf "decoy\ndecoy\n" > f'fffd'; touch .h'ff
f = .file~new('f'ff)
say 'name' c2x(f~name) 'abs' (f~absolutePath == start'/f'ff) 'parent' (f~parent == start)
say 'isdir' .file~new('d'ff)~isDirectory .file~new('d'ff)~isFile 'len' f~length 'canw' f~canWrite
say 'lm' (f~lastModified \== .nil); f~lastModified = .datetime~new(2020,1,1)
say 'lm2' f~lastModified~year
say 'hidden' .file~new('.h'ff)~isHidden .file~new('f'ff)~isHidden
l = .file~new('d'ff)~list; say 'list' l~items c2x(l[1])
lf = .file~new('d'ff)~listFiles; say 'listFiles' lf~items c2x(lf[1]~name) lf[1]~length
say 'mkdir' .file~new('m'ff)~makeDir .file~new('m'ff)~isDirectory
say 'delete' .file~new('m'ff)~delete .file~new('m'ff)~exists
say 'qualify' (stream('f'ff, 'c', 'query exists') == start'/f'ff) (qualify('f'ff) == start'/f'ff) (.stream~new('f'ff)~qualify == start'/f'ff)
say 'datetime' (stream('f'ff, 'c', 'query datetime') \== '') (stream('f'ff, 'c', 'query timestamp') \== '')
say 'openw' stream('w'ff, 'c', 'open write') lineout('w'ff, 'written') stream('w'ff, 'c', 'close')
address system 'cat w'ff
call directory 'd'ff
say 'rel' linein('i'e9) stream('i'e9, 'c', 'query exists') == start'/d'ff'/i'e9
call stream 'i'e9, 'c', 'close'
'cd' start
say 'after cd' c2x(right(directory(),2))
call directory start
call SysFileTree 'd'ff'/*', 't.', 'FO'
say 'tree' t.0 (t.1 == start'/d'ff'/i'e9)
say 'sysfileexists' SysFileExists('f'ff) SysIsFile('f'ff) SysIsFileDirectory('d'ff)
say 'sysmkdir' SysMkDir('k'ff) SysRmDir('k'ff)
say 'sysdel' SysFileDelete('w'ff) SysFileExists('w'ff)
address system 'rm -rf d'ff 'd'fffd 'f'ff 'f'fffd '.h'ff
