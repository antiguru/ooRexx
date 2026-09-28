-- RAISE ... ARRAY: the condition object holds the items themselves.
o = .k~new
signal on syntax name h1
raise syntax 93.964 array ('m', o)
h1: c = condition('o'); say c~additional~items c~additional[2]~class~id c~additional[2]~hello c~message
signal on syntax name h2
raise syntax 40.4 array ('R',, .array~of(1))
h2: c = condition('o'); say c~additional~items c~additional~size c~additional~hasIndex(2) c~additional[3]~class~id c~message
signal on syntax name h3
call r
h3: c = condition('o'); say c~additional[1]~class~id c~code
call on user x name hu
raise user x array (o, 'z')
say 'after'
exit
hu: c = condition('o'); say 'user' c~additional~items c~additional[1]~class~id; return
::routine r
  raise syntax 93.964 array (.k~new)
::class k
::method hello; return 'hi'
