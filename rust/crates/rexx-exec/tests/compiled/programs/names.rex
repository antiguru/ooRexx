say 'newfile' .Routine~newFile('named.rxo')~call
say 'new' .Routine~new('rn', .stream~new('namede.rxo')~arrayin)~call
call 'named.rxo'
say 'call' result
signal on syntax name s
.Routine~newFile('fail.rxo')~call
s: do l over condition('o')~traceback; say l; end
