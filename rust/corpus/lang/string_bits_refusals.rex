/* The bit operations' refusals.
 *
 * There is almost nothing to refuse: the operand is optional and any length,
 * and only the pad has a rule -- exactly one byte, 93.922, or 88.909 for a
 * value with no string value at all. Two is the maximum.
 *
 * The untrapped tail is the 93.922. rc 163.
 */

signal on syntax name trapped
a = '13'x
n = 0

next:
n = n + 1
select
  /* A pad that is not exactly one byte. */
  when n = 1 then say n 'answered' a~bitAnd('11'x, 'zz')
  when n = 2 then say n 'answered' a~bitOr('11'x, '')
  when n = 3 then say n 'answered' a~bitXor('11'x, 'ab')
  /* A value with no string value, in either position. */
  when n = 4 then say n 'answered' a~bitAnd(.nil)
  when n = 5 then say n 'answered' a~bitOr('11'x, .nil)
  /* Two is the maximum. */
  when n = 6 then say n 'answered' a~bitAnd('11'x, '00'x, 1)
  when n = 7 then say n 'answered' a~bitXor('11'x, '00'x, 1)
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say a~bitAnd('11'x, 'zz')
