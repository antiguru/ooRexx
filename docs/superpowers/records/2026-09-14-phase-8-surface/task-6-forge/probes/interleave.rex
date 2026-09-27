/* output and error targets that report each line as it lands */
call AddCmd 'xr', 'r'
o = .Loud~new('out'); e = .Loud~new('err')
address xr 'ECHO' with input using (('a', 'b')) output using (o) error using (e)
say 'rc' rc
exit
::requires 'cmd' LIBRARY
::class Loud public inherit OutputStream
::method init
  expose tag
  use arg tag
::method lineout
  expose tag
  use arg line
  say tag line
  return 0
::method charout
  raise syntax 93.963
