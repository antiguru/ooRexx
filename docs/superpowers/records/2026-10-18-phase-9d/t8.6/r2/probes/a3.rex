-- replace + append on read-only file, and append to a file opened write-only retry
'printf "x\n" > ro.txt; chmod 444 ro.txt'
s = .stream~new('ro.txt'); say s~open('write append') s~description
'printf "y\n" > wo.txt; chmod 222 wo.txt'
w = .stream~new('wo.txt'); say w~open('write append'); say w~lineout('w1')
'echo wcmd >> wo.txt'
say w~lineout('w2'); say w~close
'chmod 644 wo.txt'
say 'wo' charin('wo.txt', 1, 80)~changestr('0a'x, '|')
-- big write past buffer
b = .stream~new('big.txt'); b~open('write append'); b~charout('s')
'printf "c" >> big.txt'
b~charout(copies('z', 5000)); b~charout('e'); b~close
x = charin('big.txt', 1, 6000); say 'big' x~length left(x, 3) right(x, 2)
