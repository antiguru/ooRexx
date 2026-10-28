t = .object~new
t~setMethod('findc', .methods['FINDC'])
say 'class:' t~findc('LIST')
signal on syntax
r = t~findc('NOSUCH')
say 'answered:' r
exit
syntax:
  c = condition('o')
  say 'syntax' c~code c~message
  say 'additional' c~additional~toString(,'|')
  say 'position' c~position
::requires 'lib.cls'
::method findc external "LIBRARY orxmethod TestFindClass"
