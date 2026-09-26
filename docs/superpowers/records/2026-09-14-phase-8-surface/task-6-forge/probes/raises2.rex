/* A condition a direct handler raises with no RESULT, and the ::OPTIONS escalations.
   (Directory~hasIndex refuses on a condition object here, so an entry present
   as .nil and an absent one read alike.) */
call AddCmd 'xd', 'd'
call on error name onerror
call on user foo name onuser
address xd 'RAISENORES ERROR'
say 'error nores' rc .rs
address xd 'RAISENORES USER FOO'
say 'user nores' rc .rs
signal on error name sigerr
address xd 'RAISE ERROR'
say 'not here'
sigerr:
  c = condition('o')
  say 'sigerr' c~condition c~rc c~description c~additional c~result rc sigl
call escalate
exit
onerror:
  c = condition('o')
  say 'onerror' c~condition c~rc c~description c~additional c~result rc
  return
onuser:
  c = condition('o')
  say 'onuser' c~condition c~rc c~description c~result rc
  return
::requires 'cmd' LIBRARY
::requires 'raises2.cls'
