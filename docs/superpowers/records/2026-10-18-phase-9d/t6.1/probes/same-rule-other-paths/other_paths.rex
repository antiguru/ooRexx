dir = qualify('rqo')
ok = .File~new(dir)~makeDir
f = .stream~new(dir'/ext_r.rex'); f~open('write replace'); f~lineout('return "external"'); f~close
r = .routine~new(dir'/any.zzz', ("call ext_r", "return result"))
say 'findProgram' (r~package~findProgram('ext_r') \== .nil)
signal on syntax
say 'call' r~call
exit
syntax: say 'syntax' condition('o')~code
