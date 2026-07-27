/* NUMERIC settings and the errors they raise. The error numbers are part of
   the contract:
     26  the value is not a positive whole number (DIGITS) or is negative (FUZZ)
     33  FUZZ would be >= DIGITS, whichever of the two is being set
     25  the FORM is neither SCIENTIFIC nor ENGINEERING                        */

say "defaults:" digits() fuzz() form()
call t "numeric digits 0"
call t "numeric digits -1"
call t "numeric digits 1"
call t "numeric digits 1e3"
call t "numeric digits 1.5"
call t "numeric fuzz -1"
call t "numeric fuzz 0"
call t "numeric digits 5; numeric fuzz 5"
call t "numeric digits 5; numeric fuzz 4"
call t "numeric fuzz 4; numeric digits 4"
call t "numeric form value 'ENGINEERING'"
call t "numeric form value 'BOGUS'"
exit 0

t:
  parse arg code
  signal on syntax name bad
  interpret code
  say "OK  " code "->" digits() fuzz() form()
  call reset
  return
bad:
  say "ERR" rc code
  call reset
  return

reset:
  numeric digits 9
  numeric fuzz 0
  numeric form scientific
  return
