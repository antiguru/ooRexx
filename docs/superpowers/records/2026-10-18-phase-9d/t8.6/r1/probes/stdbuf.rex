e = .stream~new('STDERR'); e~lineout('e1'); call lineout 'STDERR', 'e2'; .stderr~lineout('e3'); e~close
o = .stream~new('STDOUT'); o~open('write nobuffer'); o~lineout('nb1'); say 'nb2'
p = .stream~new('STDOUT:'); p~lineout('colon'); say 'after colon'
q = .stream~new('STDOUT'); q~charout(copies('x', 5000)); say ''; say 'after big'
r = .stream~new('STDOUT'); r~lineout('never closed'); say 'last say'
