e = .c~e
.c~define('ZZ', e)
signal on syntax name s1
say .c~new~zz
s1: say condition('o')~code condition('o')~message
signal on syntax name s2
say .object~new~run(e)
s2: say condition('o')~code condition('o')~message
signal on syntax name s3
o = .object~new~enhanced(.table~of(('ZZ', e)))
say o~zz
s3: say condition('o')~code condition('o')~message
.object~new~run(e)
say 'call' var('result')
::class c
::constant e (.context~executable)
