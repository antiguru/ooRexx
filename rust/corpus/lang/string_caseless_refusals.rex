/* The comparison family's refusals, and EQUALS is the odd one out.
 *
 * EQUALS and CASELESSEQUALS have NO argument type to refuse: every object
 * renders, and a rendering that is not the receiver simply answers 0.
 * Measured, `'abc'~equals(.nil)` is 0 where `'abc'~compareTo(.nil)` is 88.909
 * -- the same argument, one method reading it leniently and the other with
 * `stringArgument`. That pair is the untrapped tail.
 *
 * COMPARETO's start is a position (93.924) and its length a length (93.923);
 * CASELESSCOMPARE's pad is a pad (93.922) and CASELESSABBREV's length a length.
 *
 * rc 168.
 */

signal on syntax name trapped
n = 0

next:
n = n + 1
select
  /* Omitted is 93.903 for every one of the six. */
  when n = 1 then say n 'answered' 'abc'~equals()
  when n = 2 then say n 'answered' 'abc'~caselessEquals()
  when n = 3 then say n 'answered' 'abc'~compareTo()
  when n = 4 then say n 'answered' 'abc'~caselessCompareTo()
  when n = 5 then say n 'answered' 'abc'~caselessAbbrev()
  when n = 6 then say n 'answered' 'abc'~caselessCompare()
  /* EQUALS takes any object; COMPARETO does not. */
  when n = 7 then say n 'answered' 'abc'~equals(.nil)
  when n = 8 then say n 'answered' 'abc'~caselessEquals(.array)
  when n = 9 then say n 'answered' 'abc'~compareTo(.nil)
  when n = 10 then say n 'answered' 'abc'~caselessCompareTo(.array)
  when n = 11 then say n 'answered' 'abc'~caselessAbbrev(.nil)
  when n = 12 then say n 'answered' 'abc'~caselessCompare(.nil)
  /* COMPARETO's start is a position and its length a length. */
  when n = 13 then say n 'answered' 'abc'~compareTo('a', 0)
  when n = 14 then say n 'answered' 'abc'~compareTo('a', 'x')
  when n = 15 then say n 'answered' 'abc'~compareTo('a', 1, 'x')
  when n = 16 then say n 'answered' 'abc'~compareTo('a', 1, -1)
  /* The other two keep the layers they share with their exact twins. */
  when n = 17 then say n 'answered' 'abc'~caselessAbbrev('a', 'x')
  when n = 18 then say n 'answered' 'abc'~caselessCompare('a', 'xx')
  /* Each row's arity is a maximum. */
  when n = 19 then say n 'answered' 'abc'~equals('a', 'b')
  when n = 20 then say n 'answered' 'abc'~compareTo('a', 1, 2, 3)
  when n = 21 then say n 'answered' 'abc'~caselessCompare('a', 'x', 1)
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say 'abc'~compareTo(.nil)
