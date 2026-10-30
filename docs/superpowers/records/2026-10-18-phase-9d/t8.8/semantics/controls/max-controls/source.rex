numeric digits 5
call trial 'say 5~max("1234567890")'
call trial 'say 5~max(.key~new("1234567890"))'
call trial 'say 5~min(.key~new("-1234567890"))'
call trial 'say 5~max("1234567890", .key~new("2"))'
call trial 'say 5~max("1234567890", .bad~new)'
call trial 'say 5~max("1234567890.1", .bad~new)'
call trial 'say 5~max("bad", .bad~new)'
call trial 'say 5~max(, .bad~new)'
call trial 'say 5~max(.key~new(" 1234567890 "))'
call trial 'say 5~max(.key~new("bad"))'
exit
trial:
  signal on lostdigits name ld
  signal on syntax name sy
  interpret arg(1)
  return
ld: say 'LOSTDIGITS' condition('d'); return
sy: say 'SYNTAX' condition('o')~code; return
::class key
::method init
  expose text
  use arg text
::method makeString
  expose text
  say 'make' text
  return text
::class bad
::method makeString
  say 'bad conversion'
  raise syntax 98.900 array('bad')
