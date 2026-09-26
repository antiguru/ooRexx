say Keep(.array~of(1,2,3))
call collect
say Kept()~items
say Release() Kept()~items
say Release() Kept()~items
say LocalRelease('abc')
say Register('myreg') Register('myreg') Register('rxmath')
say RegisteredEcho('hi')
say Nested(.array~of(4,5))
exit
collect: procedure
  do i = 1 to 2000; x = .array~new(10); end
  return
::requires 'reach' LIBRARY
