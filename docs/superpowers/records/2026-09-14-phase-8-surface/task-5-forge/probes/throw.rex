do w over .array~of('0', '1', '2', 'A')
  say w':' try(w) Dtors()
end
say 'user:' tryuser() Dtors()
say 'method:' .m~new~try('1') Dtors()
say 'method user:' .m~new~tryuser Dtors()
exit
try: procedure
  signal on syntax
  r = Throw(arg(1), 'Fred')
  return 'no trap' r
syntax:
  c = condition('o')
  return 'trapped' rc c~code c~additional~items c~message
tryuser: procedure
  call on user bar name h
  r = Throw('USER BAR', 'add')
  return 'returned' r
h:
  c = condition('o')
  say 'handler' c~condition c~description c~additional c~result
  return
::requires 'reach' LIBRARY
::class m
::method throw external "LIBRARY reach MThrow"
::method try
  signal on syntax
  r = self~throw(arg(1), 'Fred')
  return 'no trap' r
syntax:
  c = condition('o')
  return 'trapped' rc c~code c~additional~items
::method tryuser
  signal on user bar
  r = self~throw('USER BAR', 'add')
  return 'returned' r
bar:
  c = condition('o')
  return 'signalled' c~condition c~description c~additional c~result
