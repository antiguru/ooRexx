/* Every way an Array subscript can be wrong. The three errors are not
 * interchangeable and which one arrives says where the check sits: 93.901 for
 * no subscript at all, 93.926 for more than one -- raised by the method's own
 * body, because `[]` and `At` are declared A_COUNT and so the send refuses no
 * count of its own -- and 93.907 for one subscript that is not a positive
 * whole number within ARGUMENT_DIGITS.
 *
 * `~size` and `~items` are the contrast: they declare a count, so an extra
 * argument to either is 93.902 from the send rather than 93.926 from a body.
 *
 * A lone array argument is the subscript list, counted by its items and read
 * from its slots, so a one-item array answers where a two-item one is 93.926.
 * The rows that answer are what stops "refuse every subscript" from passing.
 *
 * The last send is untrapped so the 93.907 text and the frame line are
 * compared as bytes. rc 163.
 */

signal on syntax name trapped
a = (1,2)
n = 0

next:
n = n + 1
select
  when n = 1 then say n 'answered' a~at()
  when n = 2 then say n 'answered' a[]
  when n = 3 then say n 'answered' a~at(,)
  when n = 4 then say n 'answered' a~at('x')
  when n = 5 then say n 'answered' a~at(0)
  when n = 6 then say n 'answered' a~at(-1)
  when n = 7 then say n 'answered' a~at(1.5)
  when n = 8 then say n 'answered' a~at(.nil)
  when n = 9 then say n 'answered' a~at(.array)
  when n = 10 then say n 'answered' a~at(1000000000000000000)
  when n = 11 then say n 'answered' a~at(1,2)
  when n = 12 then say n 'answered' a[1,2]
  when n = 13 then say n 'answered' a~at(,1)
  when n = 14 then say n 'answered' a~at((1,2))
  when n = 15 then say n 'answered' a~at((1,,3))
  when n = 16 then say n 'answered' a~size(1)
  when n = 17 then say n 'answered' a~items(1)
  when n = 18 then say n 'answered' a~at(999999999999999999)
  when n = 19 then say n 'answered' a~at((2,))
  when n = 20 then say n 'answered' a~at(2)
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say a~at('x')
