/* FLOOR, CEILING, ROUND and MODULO as methods: `AddMethod("Floor",
 * RexxString::floor, 0)` and its three neighbours (memory/Setup.cpp), each
 * reaching NumberString through the ArithmeticMethod macro
 * (classes/StringClass.cpp:1058).
 *
 * The value is rounded to the current NUMERIC DIGITS BEFORE the direction is
 * applied, so the answer is a function of the rounded value: at DIGITS 9
 * `'123456789.5'~floor` is 123456790, the ten-digit value having already
 * rounded to something with no decimals left.
 *
 * ROUND takes a half AWAY from zero. The interpreter's own comment calls it
 * `floor(number + .5)`, which would send -2.5 to -2.
 *
 * MODULO is not `//`: a negative remainder gains the divisor, so
 * `'-13'~modulo(5)` is 2 where `-13 // 5` is -3. It keeps the remainder's
 * scale, so `'10.0'~modulo(3)` is 1.0.
 */

say '1.5'~floor '1.5'~ceiling '1.5'~round
say '-1.5'~floor '-1.5'~ceiling '-1.5'~round
say '2.0'~floor '2.00'~ceiling '-2.0'~floor '-2.000'~round
say '0.5'~round '-0.5'~round '2.5'~round '-2.5'~round '3.5'~round
say '0.4'~round '-0.4'~round '0.05'~round '-0.05'~round
say '99.5'~ceiling '-99.5'~floor '9.5'~ceiling '999999999.5'~round
say '0.999999999'~floor '0.999999999'~ceiling '-0.999999999'~floor
say '1E-100'~floor '1E-100'~ceiling '1E-100'~round
say '-1E-100'~floor '-1E-100'~ceiling '-1E-100'~round
say '1E9'~floor '1E20'~floor length('1E100'~floor)
say '123456789.5'~round '123456789.5'~floor
say '0'~floor '0.000'~ceiling '-0.0'~round
say '13'~modulo(5) '-13'~modulo(5) '-10'~modulo(5) '0'~modulo(5)
say '1234567890'~modulo(7) '1E8'~modulo(7) '10.0'~modulo(3) '1.0E1'~modulo(3)
numeric digits 3
say '1234'~floor '1236.7'~round '1236.7'~ceiling
