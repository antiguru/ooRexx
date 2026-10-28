l = .context~package~local
l~put(.array, 'PLFIND')
l~setMethod('UNKNOWN', 'say "unknown called" arg(1); raise syntax 93.900 array("from unknown" arg(1))')
say 'class:' fcc('LIST')
say 'local:' fcc('PLFIND')
signal on syntax
r = fcc('NOSUCH')
say 'answered:' r
exit
syntax:
  c = condition('o')
  say 'syntax' c~code c~message
  say 'additional' c~additional~toString(,'|')
  say 'position' c~position
::routine fcc external "LIBRARY orxfunction TestFindContextClass"
