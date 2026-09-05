/* The search family's refusals, which are the half a program of successful
 * sends cannot reach: the argument layer is shared with MutableBuffer's rows,
 * so an error number written once is wrong for two receivers at a time.
 *
 * A missing needle is 93.903 naming argument 1. A start or a range that is not
 * a whole number is 93.924 or 93.923 and quotes the argument's own text; a
 * start of 0 is 93.924 too, because a position is 1-based and 0 is not one
 * below the first, it is not a position at all. A fourth argument is 93.902
 * saying `3 expected`, which is the declared parameter count and not the body.
 *
 * The last send is untrapped so the 93.903 text and the frame line naming the
 * scope are compared as bytes. rc 163.
 */

signal on syntax name trapped
s = 'abcABCabc'
n = 0

next:
n = n + 1
select
  when n = 1 then say n 'answered' s~pos
  when n = 2 then say n 'answered' s~pos('a', 'x')
  when n = 3 then say n 'answered' s~pos('a', 1, 'x')
  when n = 4 then say n 'answered' s~pos('a', 0)
  when n = 5 then say n 'answered' s~pos('a', 1, 2, 3)
  when n = 6 then say n 'answered' s~pos(1)
  when n = 7 then say n 'answered' s~lastPos
  when n = 8 then say n 'answered' s~lastPos('a', 'x')
  when n = 9 then say n 'answered' s~lastPos('a', 1, 'x')
  when n = 10 then say n 'answered' s~lastPos('a', 0)
  when n = 11 then say n 'answered' s~contains
  when n = 12 then say n 'answered' s~contains('a', 'x')
  when n = 13 then say n 'answered' s~caselessPos
  when n = 14 then say n 'answered' s~caselessPos('a', 'x')
  when n = 15 then say n 'answered' s~caselessLastPos
  when n = 16 then say n 'answered' s~caselessLastPos('a', 'x')
  when n = 17 then say n 'answered' s~caselessContains
  when n = 18 then say n 'answered' s~caselessContains('a', 'x')
  /* The same shapes at the MutableBuffer receiver, whose rows this argument
     layer now serves too, so a change that moved one and not the other would
     show here. */
  when n = 19 then say n 'answered' .MutableBuffer~new('abc')~pos
  when n = 20 then say n 'answered' .MutableBuffer~new('abc')~pos('a', 'x')
  when n = 21 then say n 'answered' .MutableBuffer~new('abc')~lastPos('a', 0)
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say s~pos
