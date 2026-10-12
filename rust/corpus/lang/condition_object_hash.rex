/* A condition object is a Directory: its hash methods answer, CONDITION('O')
   is a copy of it, and re-raising it through ~result keeps its identity. */
m = .message~new(.t~new, 'BOOM')
signal on syntax name s1
m~send
s1:
c = m~errorCondition
say c~class c~items
s = ''; do i over c~allIndexes; s = s i; end
say s
say c~hasIndex('CODE') c~hasIndex('NOPE') c~hasIndex('code')
say c~at('CODE') c['CODE'] c~code
say c~isEmpty c~hasItem(c~at('CODE')) c~index('42.3')
say c~allItems~items c~makeArray~items c~supplier~class
o = condition('O')
say (o == c) o~items
o['NEW'] = 1
say o~items c~items (m~errorCondition == c)
signal on syntax name s2
r = m~result
s2:
c3 = m~errorCondition
say (c3 == c) (condition('O') == c) c~propagated c3~items c~position
call on user foo name h1
call raiser
signal on novalue name h2
say zz
h2:
call show 'novalue', condition('O')
signal on syntax name h3
call deep
h3:
c = condition('O')
call show 'propagated' c~propagated, c
do k over c; nop; end
say k
exit
h1:
call show 'call-on', condition('O')
return
show: procedure
  use arg tag, c
  s = ''
  do i over c~allIndexes; s = s i; end
  say tag c~items':'s
  return
raiser:
  raise user foo description 'd1' additional (.array~of(1,2)) return
::routine deep
  signal on syntax name inner
  x = 1/0
inner:
  raise propagate
::class t
::method boom
  return 1/0
