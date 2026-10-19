o = .t~new
o~put('PUT2', 'return 123', 'OBJECT')
o~put('FL', 'return 1', 'FLOAT')
o~put('PUT', 'return 5', 'OBJECT')
o~hide('ZAP')
call show 'nil', o~instanceMethods(.nil)
call show 't', o~instanceMethods(.t)
call show 'all', o~instanceMethods
exit
show: procedure
  use arg tag, s
  out = .array~new
  do while s~available
    out~append(s~index s~item~class~id s~item~source~makestring(, "|"))
    s~next
  end
  out = out~sort
  do l over out; if wordpos(word(l,1), 'PUT PUT2 FL ZAP HIDE') > 0 then say tag l; end
  return
::class t
::method put
  use arg n, m, sc
  self~setMethod(n, m, sc)
::method hide
  self~setMethod(arg(1))
::method zap
  return 0
