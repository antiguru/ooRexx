/* ABBREV's and COMPARE's refusals.
 *
 * Both take a required string first -- omitted is 93.903 and a value with no
 * string value is 88.909 -- and differ in their second: ABBREV's is a length
 * (93.923) and COMPARE's is a pad (93.922, exactly one byte, and an empty one
 * is as wrong as a two-byte one).
 *
 * The untrapped tail is COMPARE's 93.922, whose substituted text names what
 * was found. rc 163.
 */

signal on syntax name trapped
p = 'Print'
n = 0

next:
n = n + 1
select
  /* The first argument is required, and must have a string value. */
  when n = 1 then say n 'answered' p~abbrev()
  when n = 2 then say n 'answered' p~abbrev(.nil)
  when n = 3 then say n 'answered' p~compare()
  when n = 4 then say n 'answered' p~compare(.nil)
  /* ABBREV's second is a length. */
  when n = 5 then say n 'answered' p~abbrev('Pri', 'x')
  when n = 6 then say n 'answered' p~abbrev('Pri', -1)
  when n = 7 then say n 'answered' p~abbrev('Pri', '2.5')
  /* COMPARE's second is a pad, and empty is not one byte. */
  when n = 8 then say n 'answered' p~compare('Print', 'xx')
  when n = 9 then say n 'answered' p~compare('Print', '')
  when n = 10 then say n 'answered' p~compare('Print', .nil)
  /* Two is the maximum for both. */
  when n = 11 then say n 'answered' p~abbrev('Pri', 2, 3)
  when n = 12 then say n 'answered' p~compare('Print', 'x', 1)
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say p~compare('Print', 'xx')
