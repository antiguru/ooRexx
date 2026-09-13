/* Where an external routine call looks, and in what order.

   The call is a name no label, builtin or ::ROUTINE answers, so it reaches the
   file search. The current directory, `REXX_PATH` and `PATH` each hold a file
   only they have, and two hold a name in common, so the order is read off the
   answers rather than asserted. A name none of them has stays 43.1. */

say 'cwd      ' zhere()
say 'rexxpath ' zrp()
say 'path     ' zpath()
say 'all three' zall()
say 'two of us' zpair()
say 'nowhere  ' missing()

exit 0

missing: procedure
  signal on syntax name absent
  return znowhere()
absent:
  raised = condition('O')
  return raised['CODE'] raised['MESSAGE']
