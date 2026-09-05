/* The counting and word-search family's refusals.
 *
 * A missing needle or phrase is 93.903 naming argument 1. countStr declares
 * one parameter, so a second argument is 93.902 saying `1 expected`, where
 * wordPos declares two and refuses a third; those numbers come from the
 * declared count and never from the body. A start that is not a whole number,
 * or is 0, is 93.924 and quotes the argument's own text.
 *
 * The last two sends are at a MutableBuffer receiver, whose rows share this
 * argument layer, so a change that moved one receiver and not the other would
 * show here.
 *
 * The final send is untrapped so the 93.903 text and the frame line naming the
 * scope are compared as bytes. rc 163.
 */

signal on syntax name trapped
s = 'abcABCabc'
w = '  now is  the TIME  '
n = 0

next:
n = n + 1
select
  when n = 1 then say n 'answered' s~countStr
  when n = 2 then say n 'answered' s~countStr('a', 'b')
  when n = 3 then say n 'answered' s~countStr(1)
  when n = 4 then say n 'answered' s~caselessCountStr
  when n = 5 then say n 'answered' s~caselessCountStr('a', 'b')
  when n = 6 then say n 'answered' w~wordPos
  when n = 7 then say n 'answered' w~wordPos('is', 'x')
  when n = 8 then say n 'answered' w~wordPos('is', 0)
  when n = 9 then say n 'answered' w~wordPos('is', 1, 2)
  when n = 10 then say n 'answered' w~caselessWordPos
  when n = 11 then say n 'answered' w~caselessWordPos('is', 'x')
  when n = 12 then say n 'answered' w~containsWord
  when n = 13 then say n 'answered' w~containsWord('is', 'x')
  when n = 14 then say n 'answered' w~caselessContainsWord
  when n = 15 then say n 'answered' w~caselessContainsWord('is', 'x')
  when n = 16 then say n 'answered' .MutableBuffer~new('abc')~countStr
  when n = 17 then say n 'answered' .MutableBuffer~new('a b')~wordPos('b', 'x')
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say s~countStr
