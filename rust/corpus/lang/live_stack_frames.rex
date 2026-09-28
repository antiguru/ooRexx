-- .context~stackframes, and the frames of a condition taken while they are
-- live, include a running INTERPRET fragment's level and a running native
-- call's; CallRoutine runs its routine directly, under the null string.
call r
call viaInterpret
k = .k~new
say k~send0(.t~new, 'm')
say k~callr(.routine~new('rr', 'do f over .context~stackframes; say f~type "["f~name"]" f~line f~arguments~items; end; return 1'), .array~of(1, 2))
say k~callr(.routine~new('rr', 'return arg() .context~stackframes~items'), .array~new)
signal on syntax
x = k~callr(.routine~new('rr', 'return 1 / 0'), .array~new)
exit
syntax:
  c = condition('O')
  say c~code c~position c~propagated
  do l over c~traceback; say ' ' l; end
  do f over c~stackframes; say ' ' f~type '['f~name']' f~line; end
  signal off syntax
  x = k~callr(.routine~new('rr', 'return 1 / 0'), .array~new)
::routine r
  nop
  interpret 'nop; do f over .context~stackframes; say f~type f~line f~traceline; end'
  interpret 'interpret "say .context~stackframes~items"; say .context~stackframes~firstItem~traceline'
::routine viaInterpret
  interpret 'call inner 1'
  return
inner:
  signal on syntax name h
  x = 1 / 0
  return
h:
  do f over condition('O')~stackframes; say 'trapped' f~type f~line f~traceline; end
  return
::class k
::method send0 external "LIBRARY orxmethod TestSendMessage0"
::method callr external "LIBRARY orxmethod TestCallRoutine"
::class t
::method m
  do f over .context~stackframes; say f~type f~name f~line; end
  return 'm'
