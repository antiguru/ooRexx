call ext1 'x'
r = .routine~new('RR', 'say "rr ran"')
r~call
signal on syntax
r2 = .routine~new('BAD', 'say (')
exit
syntax: say condition('o')~code condition('o')~message
::options trace r
