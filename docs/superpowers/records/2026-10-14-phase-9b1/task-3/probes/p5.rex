signal on syntax
s.0 = 2; s.1 = .k~new; s.2 = 'a'
call SysStemSort 's.'
say 'sorted'
exit
syntax:
  c = condition('o')
  say 'code' c~code
  do l over c~traceback; say 'tb['l']'; end
  do f over c~stackFrames; say 'sf' f~type f~name f~line; end
  exit
::class k
::method string
  raise syntax 93.900 array('boom')
