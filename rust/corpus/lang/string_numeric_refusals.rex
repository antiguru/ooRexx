/* ABS, TRUNC and FORMAT's refusals.
 *
 * A receiver that is not a number is 93.943, and the message names the METHOD.
 * **It beats everything**: measured, `'abc'~trunc('x')` and
 * `'abc'~format(1,-1)` are both the target's 93.943, where the same mistakes
 * on a numeric receiver report the argument. The builtin forms order it the
 * other way -- `trunc('abc','x')` is 40.12 -- so the two do not share a layer.
 *
 * The arguments themselves are counts rather than lengths: 93.906, not the
 * 93.923 the pad family raises.
 *
 * The untrapped tail is the target's 93.943 with an argument that is also
 * wrong, which is the ordering claim as bytes. rc 163.
 */

signal on syntax name trapped
n = 0

next:
n = n + 1
select
  /* A receiver that is not a number, for each of the three. */
  when n = 1 then say n 'answered' 'abc'~abs
  when n = 2 then say n 'answered' 'abc'~trunc
  when n = 3 then say n 'answered' 'abc'~format
  /* The target beats a bad argument, of either kind. */
  when n = 4 then say n 'answered' 'abc'~trunc('x')
  when n = 5 then say n 'answered' 'abc'~trunc(-1)
  when n = 6 then say n 'answered' 'abc'~format('x')
  when n = 7 then say n 'answered' 'abc'~format(1, -1)
  /* On a numeric receiver the argument reports, and it is a count. */
  when n = 8 then say n 'answered' '1.55'~trunc(-1)
  when n = 9 then say n 'answered' '1.55'~trunc('x')
  when n = 10 then say n 'answered' '3.14'~format(-1)
  when n = 11 then say n 'answered' '3.14'~format(, -1)
  when n = 12 then say n 'answered' '3.14'~format(, , -1)
  when n = 13 then say n 'answered' '3.14'~format(, , , -1)
  /* Each row's arity is a maximum. */
  when n = 14 then say n 'answered' '12'~abs(1)
  when n = 15 then say n 'answered' '1.55'~trunc(1, 2)
  when n = 16 then say n 'answered' '3.14'~format(1, 2, 3, 4, 5)
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say 'abc'~format(1, -1)
