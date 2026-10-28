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
::requires 'lib.cls'
