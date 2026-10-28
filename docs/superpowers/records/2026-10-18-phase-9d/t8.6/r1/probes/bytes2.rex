ff = 'ff'x; fffd = 'efbfbd'x
start = directory()
address system 'mkdir d'ff 'd'fffd'; printf "x\n" > d'ff'/x; printf "return \"ext ff\"\n" > d'ff'/ext.rex; printf "return \"ext decoy\"\n" > d'fffd'/ext.rex; printf "x\n" > d'fffd'/x'
address system 'printf "::routine rq public\nreturn \"rq ff\"\n" > d'ff'/rq.cls; printf "::routine rq public\nreturn \"rq decoy\"\n" > d'fffd'/rq.cls'
call directory 'd'ff
dir = start'/d'ff
say 'q' (stream('x','c','query exists') == dir'/x') (qualify('x') == dir'/x') (.file~new('x')~absolutePath == dir'/x')
say 'ext' ext()
r = .routine~newFile('ext.rex'); say 'newFile' r~call c2x(r~package~name)
say 'search' (.file~searchPath('x', dir)~absolutePath == dir'/x') (.file~searchPath('x', '/nonexist:'dir'/')~absolutePath == dir'/x')
s = .stream~new('STDIN'); say 'stdin q' (s~qualify == dir'/STDIN')
s = .stream~new('STDIN'); s~open('read'); s~close; say 'stdin closed q' (s~qualify == dir'/STDIN') c2x(s~qualify)
address system 'pwd' with output stem p.; say 'pwd' (p.1 == dir)
say 'sft' SysFileTree('*', 't.', 'F') t.0
say 'dir' (directory() == dir)
p = .package~new('rq.cls')
call directory start
address system 'rm -rf d'ff 'd'fffd
