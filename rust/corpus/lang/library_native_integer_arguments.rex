/* A native argument declared as a C integer: each row converts both ends of
   its range and refuses one past each, and a value comes back as its digits.
   A string converts through its number, rounded to twenty digits, and the
   overflow check misses a multiplication by ten that wraps upward. */
t = .T~new
call row t, 'int', '-2147483648', '2147483647', '-2147483649', '2147483648'
call row t, 'int8', '-128', '127', '-129', '128'
call row t, 'int16', '-32768', '32767', '-32769', '32768'
call row t, 'int32', '-2147483648', '2147483647', '-2147483649', '2147483648'
call row t, 'int64', '-9223372036854775808', '9223372036854775807', -
  '-9223372036854775809', '9223372036854775808'
call row t, 'intptr', '-9223372036854775808', '9223372036854775807', -
  '-9223372036854775809', '9223372036854775808'
call row t, 'ssize', '-9223372036854775808', '9223372036854775807', -
  '-9223372036854775809', '9223372036854775808'
call row t, 'wholenumber', '-999999999999999999', '999999999999999999', -
  '-1000000000000000000', '1000000000000000000'
call row t, 'uint8', '0', '255', '-1', '256'
call row t, 'uint16', '0', '65535', '-1', '65536'
call row t, 'uint32', '0', '4294967295', '-1', '4294967296'
call row t, 'uint64', '0', '18446744073709551615', '-1', '18446744073709551616'
call row t, 'uintptr', '0', '18446744073709551615', '-1', '18446744073709551616'
call row t, 'size', '0', '18446744073709551615', '-1', '18446744073709551616'
call row t, 'stringsize', '0', '999999999999999999', '-1', '1000000000000000000'
call row t, 'nonneg', '0', '999999999999999999', '-1', '1000000000000000000'
call row t, 'positive', '1', '999999999999999999', '0', '1000000000000000000'
do v over .array~of('1.0', '1E2', ' 12 ', '+5', '10E-1', '5.', '-0', '1.5', '.5', 'abc', '')
  call try t, 'int32', v
end
call try t, 'int', .object~new
call try t, 'uint64', .nil
call try t, 'int', .mutablebuffer~new('12')
call try t, 'uint64', '1234567890123456789.95'
call try t, 'uint64', '1234567890123456789.45'
call try t, 'int', '1.99999999999999999995'
call try t, 'int', '1.9999999999999999995'
call try t, 'int64', '-0.999999999999999999999'
call try t, 'uint64', '-0.999999999999999999999'
call try t, 'int64', '-9223372036854775808.0'
call try t, 'uint64', '30000000000000000000'
call try t, 'int64', '21000000000000000000'
call try t, 'int', '21000000000000000000'
call try t, 'uint64', '99999999999999999999'
say 'routine' TestInt8Arg(-128) TestUint16Arg('65535.0') TestSizeArg('1E19')
exit

row: procedure
  use arg t, name, min, max, below, above
  line = name':'
  do v over .array~of(min, max, below, above)
    line = line converted(t, name, v)
  end
  say line
  return

converted: procedure
  use arg t, name, v
  signal on syntax name notconverted
  return '['v']='t~send(name, v)
notconverted:
  return '['v']!'condition('O')~code

try: procedure
  use arg t, name, v
  signal on syntax name refused
  say name '['v']=' t~send(name, v)
  return
refused:
  c = condition('O')
  say name '['v']!' c~code '|' c~message
  return

::requires 'orxfunction' LIBRARY
::class T
::method int external "LIBRARY orxmethod TestIntArg"
::method int8 external "LIBRARY orxmethod TestInt8Arg"
::method int16 external "LIBRARY orxmethod TestInt16Arg"
::method int32 external "LIBRARY orxmethod TestInt32Arg"
::method int64 external "LIBRARY orxmethod TestInt64Arg"
::method intptr external "LIBRARY orxmethod TestIntPtrArg"
::method ssize external "LIBRARY orxmethod TestSSizeArg"
::method wholenumber external "LIBRARY orxmethod TestWholeNumberArg"
::method uint8 external "LIBRARY orxmethod TestUint8Arg"
::method uint16 external "LIBRARY orxmethod TestUint16Arg"
::method uint32 external "LIBRARY orxmethod TestUint32Arg"
::method uint64 external "LIBRARY orxmethod TestUint64Arg"
::method uintptr external "LIBRARY orxmethod TestUintPtrArg"
::method size external "LIBRARY orxmethod TestSizeArg"
::method stringsize external "LIBRARY orxmethod TestStringSizeArg"
::method nonneg external "LIBRARY orxmethod TestNonnegativeWholeNumberArg"
::method positive external "LIBRARY orxmethod TestPositiveWholeNumberArg"
