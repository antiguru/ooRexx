/* The Method a ::CONSTANT expression runs as can be run again: sent for a
   result it answers none (91.999); RUN is private to it as to any object. */
e = .c~e
.c~define('ZZ', e)
signal on syntax name s1
say .c~new~zz
s1: say condition('o')~code condition('o')~message
signal on syntax name s2
say .object~new~run(e)
s2: say condition('o')~code condition('o')~message
.c~new~zz
say 'instruction' var('result')
::class c
::constant e (.context~executable)
