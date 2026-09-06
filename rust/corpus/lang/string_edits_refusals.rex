/* The range writes' refusals, and the one row of the shared set whose
 * argument layer is not shared with MutableBuffer.
 *
 * insert and overlay agree with their buffer twins on every refusal --
 * a non-whole offset is 93.906 for insert and 93.924 for overlay's position,
 * at either receiver. replaceAt does not: RexxString::replaceAt takes its
 * arguments positionally where MutableBuffer::replaceAt takes them by name,
 * so the same send raises 93.903 here and 88.901 there, 93.924 here and
 * 88.912 there, 93.922 here and 88.910 there. Only .nil in the first
 * position agrees, at 88.909. The pairs below are sent to both receivers so
 * that divergence is a committed fact rather than a comment.
 *
 * The final send is untrapped so the 93.924 text and the frame line naming
 * the scope are compared as bytes. rc 163.
 */

signal on syntax name trapped
s = 'abcabc'
n = 0

next:
n = n + 1
select
  when n = 1 then say n 'answered' s~insert
  when n = 2 then say n 'answered' s~insert('X', 'x')
  when n = 3 then say n 'answered' s~insert('X', 1, 'x')
  when n = 4 then say n 'answered' s~insert('X', 1, 1, 'ab')
  when n = 5 then say n 'answered' s~insert('X', 1, 1, ' ', 5)
  when n = 6 then say n 'answered' s~insert('X', -1)
  when n = 7 then say n 'answered' s~overlay
  when n = 8 then say n 'answered' s~overlay('X', 0)
  when n = 9 then say n 'answered' s~overlay('X', 'x')
  when n = 10 then say n 'answered' s~delStr('x')
  when n = 11 then say n 'answered' s~delStr(0)
  when n = 12 then say n 'answered' s~delStr(1, 'x')
  when n = 13 then say n 'answered' s~delWord
  when n = 14 then say n 'answered' s~delWord(0)
  when n = 15 then say n 'answered' s~delWord(1, 'x')
  /* replaceAt at both receivers, one pair per row. */
  when n = 16 then say n 'str' s~replaceAt
  when n = 17 then say n 'buf' .MutableBuffer~new('abcabc')~replaceAt
  when n = 18 then say n 'str' s~replaceAt('X')
  when n = 19 then say n 'buf' .MutableBuffer~new('abcabc')~replaceAt('X')
  when n = 20 then say n 'str' s~replaceAt('X', 0, 1)
  when n = 21 then say n 'buf' .MutableBuffer~new('abcabc')~replaceAt('X', 0, 1)
  when n = 22 then say n 'str' s~replaceAt('X', 'x', 1)
  when n = 23 then say n 'buf' .MutableBuffer~new('abcabc')~replaceAt('X', 'x', 1)
  when n = 24 then say n 'str' s~replaceAt('X', 1, 1, 'ab')
  when n = 25 then say n 'buf' .MutableBuffer~new('abcabc')~replaceAt('X', 1, 1, 'ab')
  when n = 26 then say n 'str' s~replaceAt(.nil, 1, 1)
  when n = 27 then say n 'buf' .MutableBuffer~new('abcabc')~replaceAt(.nil, 1, 1)
  /* insert, for contrast: the two receivers agree here. */
  when n = 28 then say n 'str' s~insert('X', 'x')
  when n = 29 then say n 'buf' .MutableBuffer~new('abcabc')~insert('X', 'x')
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say s~replaceAt('X', 0, 1)
