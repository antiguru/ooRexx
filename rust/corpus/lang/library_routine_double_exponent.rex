/* A double argument past either end of a double's range converts to what
   strtod answers, at once and in little memory; an exponent past the
   language's own limit has no numeric value at all. */
say 'start'
call t '1E+999999999'
call t '1E-999999999'
call t '-9.99E+999999999'
call t '-1E-999999999'
call t '1E+308'
call t '1.7976931348623157E+308'
call t '1.8E+308'
call t '4.9E-324'
call t '2E-324'
call t '1E-400'
call t '123456789012345678901234567890E-10'
call t '0.000000000000000000000000000000000000001'
call t '1E+1000000000'
call t '1E-1000000000'
call t '12345678901234567890123456789012345678901234567890'
exit
t: procedure
  parse arg x
  signal on syntax
  say x '->' RxCalcSqrt(x, 16) RxCalcPower(x, 1, 16)
  return
syntax:
  say x '->' condition('O')~code
  return
::requires 'pk.cls'
