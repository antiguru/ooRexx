/* DATATYPE(x, 'W') rounds x to NUMERIC DIGITS and asks whether the result is
   a whole number, at any width. */
numeric digits 19
say datatype(1~copies(19), 'W') datatype(9~copies(19), 'W') datatype(1~copies(20), 'W'),
  1~copies(19)~datatype('W') 9~copies(19)~datatype('Whole')
numeric digits 20
say datatype(1~copies(19), 'W') datatype(1~copies(20), 'W') datatype('-'9~copies(20), 'W'),
  datatype(1~copies(21), 'W')
numeric digits 30
say datatype(1~copies(25)'.0000', 'W') datatype(1~copies(25)'.5', 'W') datatype('1E+25', 'W'),
  datatype('1E+29', 'W') datatype('1E+30', 'W') datatype(1~copies(29)'.4', 'W')
numeric digits 9
say datatype('1234567890', 'W') datatype('1234567891', 'W') datatype('1E+9', 'W'),
  datatype('12345678.9', 'W') datatype('0.00000000001', 'W')
say datatype('1.5E+8', 'W') datatype('-0', 'W') datatype('9999999999', 'W'),
  datatype('123456789.4', 'W') datatype('123456789.5', 'W') datatype(' 12 ', 'W'),
  datatype('abc', 'W') datatype('', 'W')

-- The rounding is an arithmetic operand, so a trap takes LOSTDIGITS.
signal on lostdigits
say datatype('  1111111111.000 ', 'W')
say 'not raised'
afterlost:
signal off lostdigits
say '1234567890'~datatype('W')

-- And an exponent the rounding carries out of range is 42.
signal on syntax
say datatype('9.9999999999E+999999999', 'W')
say 'not raised'
exit

lostdigits:
  say 'LOSTDIGITS' '['condition('D')']' sigl
  signal afterlost

syntax:
  say 'SYNTAX' condition('O')~code sigl
