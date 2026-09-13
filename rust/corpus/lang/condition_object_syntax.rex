/* A `SYNTAX` condition's directory, which carries five indexes no other kind
   does: `CODE`, `ERRORTEXT`, `MESSAGE`, `ADDITIONAL` and `RC`. `ERRORTEXT` is
   the major's own catalogue text and `MESSAGE` the substituted one, which is
   why they differ for the same error.

   `SIGNAL ON` rather than `CALL ON`: a `CALL ON SYNTAX` is a 25.1 translation
   error, so this kind cannot be trapped resumably and needs a file of its
   own. */

signal on syntax name onsyntax
x = 1 / 0
say 'not reached'
exit

onsyntax:
  o = condition('O')
  say 'class      ' o~class~id
  say 'condition  ' o~at('CONDITION')
  say 'instruction' o~at('INSTRUCTION')
  say 'code       ' o~at('CODE')
  say 'rc         ' o~at('RC')
  say 'errortext  ' o~at('ERRORTEXT')
  say 'message    ' o~at('MESSAGE')
  say 'position   ' o~at('POSITION')
  say 'propagated ' o~at('PROPAGATED')
  say 'description[' || o~at('DESCRIPTION') || ']'
  say 'result absent' (o~at('RESULT') == .nil)
  say 'additional ' o~at('ADDITIONAL')~items
  say 'frames     ' o~at('STACKFRAMES')~items o~at('TRACEBACK')~items
