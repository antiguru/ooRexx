/* The exit context's Throw members, each with a local whose destructor logs. */
call AddCmd 'xd', 'd'
do w over .array~of('THROW0', 'THROW1', 'THROW2', 'THROWA', 'THROWC USER BAR')
  say w':' try(w) Dtors()
end
exit
try: procedure
  call on user bar name h
  signal on syntax
  address xd arg(1)
  return 'returned' rc .rs
syntax:
  c = condition('o')
  return 'trapped' c~code c~additional~items c~message
h:
  c = condition('o')
  say 'handler' c~condition c~description c~additional c~result
  return
::requires 'cmd' LIBRARY
