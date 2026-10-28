t = .T~new
t~install(.methods['FINDC'])
l = .context~package~local
l~put(.array, 'PLFIND')
l~setMethod('UNKNOWN', 'say "unknown called" arg(1); raise syntax 93.900 array("from unknown" arg(1))')
say 'class:' t~findc('LIST')
say 'local:' t~findc('PLFIND')
signal on syntax
r = t~findc('NOSUCH')
say 'answered:' r
exit
syntax:
  c = condition('o')
  say 'syntax' c~code c~message
  say 'additional' c~additional~toString(,'|')
  say 'position' c~position
::method findc external "LIBRARY orxmethod TestFindClass"
::class T
::method install
  use arg m
  self~setMethod('findc', m)
