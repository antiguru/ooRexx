/* A SYNTAX condition's ADDITIONAL is the array its value requests: a
   string's lines, one item a line, an INTERPRET's raise too. Another
   condition keeps the value. */
signal on syntax name s1
raise syntax 40.1 additional('abc')
s1: c = condition('O'); say c~additional~class c~additional~items '['c~additional[1]']' c~message
signal on syntax name s2
raise syntax 93.900 additional('one' || '0a'x || 'two')
s2: c = condition('O'); say c~additional~class c~additional~items '['c~additional[1]']' c~message
signal on syntax name s3
raise syntax 93.900 additional(12)
s3: c = condition('O'); say c~additional~class c~additional~items '['c~additional[1]']' c~message
signal on syntax name s4
raise syntax 93.900 additional('')
s4: c = condition('O'); say c~additional~class c~additional~items c~message
signal on syntax name s5
interpret 'raise syntax 40.1 additional("abcdefghij")'
s5: say condition('O')~additional[1]
call on user u name h6
call raiser
say 'after'
exit
raiser: raise user u additional('xyz') return
h6: c = condition('O'); say 'user' c~additional~class c~additional
return
