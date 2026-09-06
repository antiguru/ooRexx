/* The word readers' and verify's refusals.
 *
 * verify's option is its own error: an unrecognised one is 93.915, which names
 * the accepted set, and it is raised before the start is looked at. words
 * declares no parameters, so one argument is 93.902.
 *
 * The final send is untrapped so the 93.915 text and the frame line naming the
 * scope are compared as bytes -- 93.915 rather than the family's usual 93.903,
 * because that is the error this family's own argument is refused with. rc 163.
 */

signal on syntax name trapped
s = 'abcabc'
w = '  now is  the time  '
n = 0

next:
n = n + 1
select
  when n = 1 then say n 'answered' s~word
  when n = 2 then say n 'answered' s~word(0)
  when n = 3 then say n 'answered' s~word('x')
  when n = 4 then say n 'answered' w~wordIndex
  when n = 5 then say n 'answered' w~wordLength
  when n = 6 then say n 'answered' w~words(1)
  when n = 7 then say n 'answered' s~verify
  when n = 8 then say n 'answered' s~verify('ab', 'Z')
  when n = 9 then say n 'answered' s~verify('ab', 'M', 'x')
  when n = 10 then say n 'answered' s~verify('ab', 'M', 0)
  /* The same shapes at a MutableBuffer receiver, whose rows share this layer. */
  when n = 11 then say n 'answered' .MutableBuffer~new('abc')~verify('ab', 'Z')
  when n = 12 then say n 'answered' .MutableBuffer~new('a b')~word(0)
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say s~verify('ab', 'Z')
