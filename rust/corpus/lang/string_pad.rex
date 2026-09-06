/* CENTER/CENTRE/LEFT/RIGHT/COPIES as methods.
 *
 * `AddMethod("Center", RexxString::center, 2)` and its four siblings
 * (memory/Setup.cpp:582, :583, :588, :592, :618). The receiver is the
 * builtin's first argument, so every argument position here is the builtin's
 * less one.
 *
 * The pad is one byte and defaults to a blank; a width equal to the receiver's
 * own length answers the receiver, and a shorter one truncates rather than
 * pads. CENTER truncates from both ends and drops the extra byte from the
 * left when the surplus is odd, which is what the two four/five-byte
 * receivers below separate.
 *
 * A length argument is converted before it is used, so a whole number spelled
 * as "2.0" is a width of two.
 */

s = 'abc'
e = ''
d = 'abcd'
f = 'abcde'
say '['s~center(7)']' '['s~centre(7)']' '['s~center(7, '.')']'
say '['s~center(3)']' '['s~center(2)']' '['s~center(0)']'
say '['s~center(4, '*')']' '['s~center(5, '*')']' '['s~center(6, '*')']'
say '['d~center(1)']' '['d~center(2)']' '['d~center(3)']'
say '['f~center(1)']' '['f~center(2)']' '['f~center(4)']'
say '['s~left(5)']' '['s~left(5, '-')']' '['s~left(3)']' '['s~left(0)']'
say '['s~right(5)']' '['s~right(5, '-')']' '['s~right(3)']' '['s~right(2)']'
say '['s~copies(3)']' '['s~copies(1)']' '['s~copies(0)']'
say '['e~center(4, '*')']' '['e~left(2)']' '['e~right(2, '.')']' '['e~copies(3)']'
say '['s~center(2.0)']' '['s~left('3')']' '['s~copies('2')']'
