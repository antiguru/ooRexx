say '1 call' ; call 'lib.rxo'; say result
say '2 req'  ; say pub()
say '3 rnf'  ; r = .Routine~newFile('lib.rxo'); say r~call; say r~source~items r~package~sourceSize
say '4 mnf'  ; m = .Method~newFile('lib.rxo'); say m~source~items
lines = .stream~new('libe.rxo')~arrayin
say '5 rnew' ; r = .Routine~new('rn', lines); say r~call; say r~package~name
say '6 mnew' ; m = .Method~new('mn', lines); say m~source~items m~package~name
say '7 pnew' ; p = .Package~new('pa', lines); say p~routines~items p~sourceSize p~name
say '8 lp'   ; p = .context~package~loadPackage('lp', lines); say p~routines~items p~sourceSize
say '9 pstr' ; p = .Package~new('ps', .stream~new('libe.rxo')~charin(1, .stream~new('libe.rxo')~chars)); say p~routines~items p~sourceSize
say '10 fn'  ; say .Routine~newFile('lib.rxo')~package~findRoutine('PUB')~call
say '11 cls' ; say .k~new~hi
::requires 'lib.rxo'
